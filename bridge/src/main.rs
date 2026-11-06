// A console exe loses its stdout under XMCL's `shell: true` + `detached: true` (SPIKE.md), so this is a GUI-subsystem exe.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::io::Write;
use zenith_bridge::args::collect_args;
use zenith_bridge::builds::{Store, embedded_keys, ensure_latest, http_fetch};
use zenith_bridge::owner::{self, MINECRAFT_API};
use zenith_bridge::play::Stream;
use zenith_bridge::prepare::pinned_minecraft;
use zenith_bridge::run::{Deps, run};
use zenith_bridge::setup;
use zenith_bridge::shim;

/// No console to read errors from: show them.
#[cfg(windows)]
fn show_error(text: &str) {
    if std::env::var_os("ZENITH_NO_DIALOG").is_some() {
        return; // test seam: a modal box would hang the test run
    }
    message_box(text, 0x10); // MB_ICONERROR
}

#[cfg(windows)]
fn message_box(text: &str, kind: u32) {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(
            hwnd: *mut std::ffi::c_void,
            text: *const u16,
            caption: *const u16,
            kind: u32,
        ) -> i32;
    }
    let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            wide(text).as_ptr(),
            wide("Zenith").as_ptr(),
            kind,
        )
    };
}

/// Prints to the parent console if there is one (or to a redirected stdout); otherwise a dialog.
#[cfg(windows)]
fn show_text(text: &str, ok: bool) {
    use std::os::windows::io::AsRawHandle;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn AttachConsole(pid: u32) -> i32;
        fn GetStdHandle(which: u32) -> isize;
        fn GetConsoleMode(h: isize, mode: *mut u32) -> i32;
        fn WriteConsoleW(
            h: isize,
            buf: *const u16,
            len: u32,
            written: *mut u32,
            reserved: *const std::ffi::c_void,
        ) -> i32;
    }
    // ATTACH_PARENT_PROCESS = -1; STD_OUTPUT_HANDLE = -11; a GUI exe has a null/invalid handle unless redirected
    let attached = unsafe { AttachConsole(u32::MAX) } != 0;
    let mut h = unsafe { GetStdHandle(-11i32 as u32) };
    let con = if (h == 0 || h == -1) && attached {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("CONOUT$")
            .ok()
    } else {
        None
    };
    if let Some(f) = &con {
        h = f.as_raw_handle() as isize;
    }
    if h == 0 || h == -1 {
        message_box(text, if ok { 0x40 } else { 0x10 }); // MB_ICONINFORMATION / MB_ICONERROR
        return;
    }
    let mut mode = 0u32;
    if unsafe { GetConsoleMode(h, &mut mode) } != 0 {
        // a console: UTF-16, so accents show whatever its code page is
        // `lines()` normalises \r\n and \n once
        let lines: Vec<&str> = text.lines().collect();
        let wide: Vec<u16> = format!("{}\r\n", lines.join("\r\n"))
            .encode_utf16()
            .collect();
        let mut written = 0u32;
        let wrote = unsafe {
            WriteConsoleW(
                h,
                wide.as_ptr(),
                wide.len() as u32,
                &mut written,
                std::ptr::null(),
            )
        } != 0;
        if !wrote {
            message_box(text, if ok { 0x40 } else { 0x10 });
        }
    } else if writeln!(std::io::stdout(), "{text}").is_err() {
        // a pipe or file: UTF-8; never lose the text
        message_box(text, if ok { 0x40 } else { 0x10 });
    }
}

#[cfg(not(windows))]
fn show_text(text: &str, _: bool) {
    println!("{text}");
}

/// Starts XMCL without waiting for it or sharing our handles.
fn spawn_xmcl(xmcl: &std::path::Path) -> Result<(), String> {
    let mut cmd = std::process::Command::new(xmcl);
    cmd.current_dir(xmcl.parent().unwrap_or(std::path::Path::new(".")))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("No puedo abrir {}: {e}", xmcl.display()))
}

/// `start`, `setup`, `status`, `use`, `help`. None for anything else (that is XMCL calling).
fn cli(argv: &[String], store: &Store) -> Option<i32> {
    let env = setup::Env::from_env(&store.root);
    let result: Result<String, String> = match argv.first().map(String::as_str)? {
        "help" => Ok(setup::HELP.into()),
        "status" => {
            let java = zenith_bridge::java::find_installed(
                &store.root,
                "java-runtime-epsilon",
                &zenith_bridge::java::system_candidates(),
                &zenith_bridge::java::probe_major,
            );
            Ok(setup::status(&env, java).join("\n"))
        }
        "use" => match argv.get(1) {
            Some(v) => setup::use_version(&store.root, v),
            None => Err("Usage: zenith-bridge use <version|latest>".into()),
        },
        "start" => std::env::current_exe()
            .map_err(|e| e.to_string())
            .and_then(|exe| {
                // no network here: the version comes from the installed build, else the fallback
                let minecraft = || {
                    let v = store
                        .current()
                        .and_then(|b| pinned_minecraft(&b).ok())
                        .unwrap_or_else(|| setup::FALLBACK_MINECRAFT.into());
                    (v, None)
                };
                setup::start(&env, &exe, minecraft, spawn_xmcl).map(|l| l.join("
"))
            }),
        "setup" => std::env::current_exe()
            .map_err(|e| e.to_string())
            .and_then(|exe| {
                setup::setup(&env, &exe, || {
                    let keys = embedded_keys();
                    let found = ensure_latest(store, &keys, http_fetch)
                        .and_then(|b| pinned_minecraft(&b));
                    match found {
                        Ok(v) => (v, None),
                        Err(e) => {
                            store.log(&format!("setup: {e}"));
                            (
                                setup::FALLBACK_MINECRAFT.into(),
                                Some(format!(
                                    "No connection to Zenith's builds ({e}): using Minecraft {}. Run \"setup\" again with internet to confirm it.",
                                    setup::FALLBACK_MINECRAFT
                                )),
                            )
                        }
                    }
                })
                .map(|l| l.join("\n"))
            }),
        _ => return None,
    };
    let (text, code) = match result {
        Ok(t) => (t, 0),
        Err(e) => (e, 1),
    };
    if code != 0 || !text.is_empty() {
        show_text(&text, code == 0);
    }
    Some(code)
}

#[cfg(not(windows))]
fn show_error(_: &str) {}

fn main() {
    let store = Store::from_env();
    // the panic message only: never the arguments, which carry the access token
    let panic_store = Store {
        root: store.root.clone(),
    };
    std::panic::set_hook(Box::new(move |info| {
        panic_store.log(&format!("panic: {info}"))
    }));
    let mut argv = collect_args(std::env::args_os().skip(1));
    let exe = std::env::current_exe().unwrap_or_default();
    let shim_mode = shim::is_shim_name(&exe);
    if shim_mode {
        match shim::probe(&argv) {
            Some(shim::Reply::Stderr(t)) => {
                let _ = writeln!(std::io::stderr(), "{t}");
                std::process::exit(0);
            }
            Some(shim::Reply::Stdout(t)) => {
                let _ = writeln!(std::io::stdout(), "{t}");
                std::process::exit(0);
            }
            // a game launch: XMCL's own view of the command line starts with the Java's path
            None => argv.insert(0, exe.to_string_lossy().into_owned()),
        }
    }
    if !shim_mode && argv.is_empty() {
        show_text(setup::HELP, true); // double-clicked: XMCL always passes arguments
        std::process::exit(0);
    }
    if let Some(code) = cli(
        &argv,
        &Store {
            root: store.root.clone(),
        },
    ) {
        std::process::exit(code);
    }
    let java_home = store.root.clone();
    let deps = Deps {
        find_java: Box::new(move |component| {
            zenith_bridge::java::find_or_install(
                &java_home,
                component,
                &zenith_bridge::java::system_candidates(),
                &http_fetch,
                &zenith_bridge::java::probe_major,
            )
        }),
        home: store.root,
        keys: embedded_keys(),
        check_owner: Box::new(|token| owner::check(MINECRAFT_API, token)),
        fetch: Box::new(http_fetch),
        emit: Box::new(|stream, line| {
            // a closed pipe is not worth dying for
            let _ = match stream {
                Stream::Out => writeln!(std::io::stdout(), "{line}"),
                Stream::Err => writeln!(std::io::stderr(), "{line}"),
            };
        }),
        show_error: Box::new(show_error),
    };
    std::process::exit(run(&argv, &deps));
}
