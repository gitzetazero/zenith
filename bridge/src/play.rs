//! Starting `zenith-client` and relaying its output to XMCL.

use crate::args::LaunchArgs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;

/// What XMCL waits for to call the game started (SPIKE.md section 4).
pub const READY_LINE: &str = "LWJGL Version: zenith";
const CRASH_LINE: &str = "Zenith: crash report saved to ";
/// Builds made before the rename print Horizon's name there.
const CRASH_LINE_OLD: &str = "Horizon: crash report saved to ";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Out,
    Err,
}

pub fn tool(build: &Path, name: &str) -> PathBuf {
    build.join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

/// `new` in current builds, `old` (Horizon's name) in builds made before the rename.
pub fn tool_or(build: &Path, new: &str, old: &str) -> PathBuf {
    let z = tool(build, new);
    if z.is_file() { z } else { tool(build, old) }
}

/// The game executable: `zenith` in current builds, `horizon-client` in older ones.
pub fn client(build: &Path) -> PathBuf {
    tool_or(build, "zenith", "horizon-client")
}

/// The resource preparer: `zenith-prepare`, or `horizon-prepare` in older builds.
pub fn preparer(build: &Path) -> PathBuf {
    tool_or(build, "zenith-prepare", "horizon-prepare")
}

/// A GUI-subsystem bridge has no console: without this a console child would open its own window.
pub fn hide_console(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd
}

/// The session goes on stdin (`--session-stdin`), never here.
pub fn client_cmd(build: &Path, a: &LaunchArgs, prepared: &Path) -> Command {
    let mut c = Command::new(client(build));
    c.arg("--resources")
        .arg(prepared)
        .arg("--game-dir")
        .arg(&a.game_dir)
        .arg("--home")
        .arg(&a.game_dir);
    if let Some(w) = a.width {
        c.args(["--width", &w.to_string()]);
    }
    if let Some(h) = a.height {
        c.args(["--height", &h.to_string()]);
    }
    if a.fullscreen {
        c.arg("--fullscreen");
    }
    if let Some(j) = &a.join {
        c.args(["--quickPlayMultiplayer", j]);
    }
    if let Some(w) = &a.world {
        c.args(["--quickPlaySingleplayer", w]);
    }
    c.arg("--session-stdin")
        .env("ZENITH_TELEMETRY_URL", "off")
        .env("ZENITH_NO_HEARTBEAT", "1")
        .env("ZENITH_COSMETICS", "0")
        .env("HORIZON_TELEMETRY_URL", "off") // builds before the ZENITH_ names
        .env("HORIZON_NO_HEARTBEAT", "1")
        .env("HORIZON_COSMETICS", "0")
        .env("HORIZON_HOME", &a.game_dir);
    c
}

/// Reads both pipes on their own threads (a full pipe would stall the child) and hands every
/// complete line, in arrival order, to `on_line` on the calling thread. Returns the exit code.
pub(crate) fn pump(
    mut child: Child,
    on_line: &mut dyn FnMut(Stream, &str),
) -> std::io::Result<i32> {
    let (tx, rx) = mpsc::channel();
    let pipes: [(Stream, Option<Box<dyn std::io::Read + Send>>); 2] = [
        (Stream::Out, child.stdout.take().map(|p| Box::new(p) as _)),
        (Stream::Err, child.stderr.take().map(|p| Box::new(p) as _)),
    ];
    for (stream, pipe) in pipes {
        let Some(pipe) = pipe else { continue };
        let tx = tx.clone();
        std::thread::spawn(move || {
            let mut r = BufReader::new(pipe);
            let mut buf = Vec::new();
            while r.read_until(b'\n', &mut buf).unwrap_or(0) > 0 {
                let line = String::from_utf8_lossy(&buf);
                let _ = tx.send((stream, line.trim_end_matches(['\r', '\n']).to_string()));
                buf.clear();
            }
        });
    }
    drop(tx);
    for (stream, line) in rx {
        on_line(stream, &line);
    }
    Ok(child.wait()?.code().unwrap_or(1))
}

/// Puts `child` in a job that kills it when the bridge dies, so XMCL's Stop (which may kill only
/// the bridge) cannot leave the game running. Best effort: false if Windows refuses.
/// The job handle is deliberately never closed: it lives as long as the bridge.
#[cfg(windows)]
pub fn kill_with_bridge(child: &Child) -> bool {
    use std::ffi::c_void;
    use std::os::windows::io::AsRawHandle;
    #[repr(C)]
    #[derive(Default)]
    struct ExtendedLimits {
        per_process_time: i64,
        per_job_time: i64,
        limit_flags: u32,
        min_working_set: usize,
        max_working_set: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
        io_counters: [u64; 6],
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory: usize,
        peak_job_memory: usize,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attrs: *mut c_void, name: *const u16) -> *mut c_void;
        fn SetInformationJobObject(
            job: *mut c_void,
            class: i32,
            info: *mut c_void,
            len: u32,
        ) -> i32;
        fn AssignProcessToJobObject(job: *mut c_void, process: *mut c_void) -> i32;
    }
    const EXTENDED_LIMIT_INFORMATION: i32 = 9;
    const KILL_ON_JOB_CLOSE: u32 = 0x2000;
    let mut limits = ExtendedLimits {
        limit_flags: KILL_ON_JOB_CLOSE,
        ..Default::default()
    };
    unsafe {
        let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
        !job.is_null()
            && SetInformationJobObject(
                job,
                EXTENDED_LIMIT_INFORMATION,
                (&raw mut limits).cast(),
                std::mem::size_of::<ExtendedLimits>() as u32,
            ) != 0
            && AssignProcessToJobObject(job, child.as_raw_handle().cast()) != 0
    }
}

#[cfg(not(windows))]
pub fn kill_with_bridge(_: &Child) -> bool {
    false
}

/// Runs the game: prints the ready line, gives it the session on stdin, relays its output (the crash
/// line translated to what XMCL looks for) and returns its exit code.
pub fn run_game(
    mut cmd: Command,
    session_json: &str,
    emit: &mut dyn FnMut(Stream, &str),
) -> std::io::Result<i32> {
    hide_console(&mut cmd);
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    kill_with_bridge(&child);
    emit(Stream::Out, READY_LINE);
    if let Some(mut stdin) = child.stdin.take() {
        // a game that already exited just closes the pipe
        let _ = stdin.write_all(session_json.as_bytes());
    }
    pump(
        child,
        &mut |stream, line| match line
            .strip_prefix(CRASH_LINE)
            .or_else(|| line.strip_prefix(CRASH_LINE_OLD))
        {
            Some(path) => {
                if stream == Stream::Err {
                    emit(stream, line);
                }
                emit(Stream::Out, &format!("Crash report saved to: #@!@# {path}"));
            }
            None => emit(stream, line),
        },
    )
}
