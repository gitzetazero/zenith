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

/// No console to read errors from: show them.
#[cfg(windows)]
fn show_error(text: &str) {
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
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn AttachConsole(pid: u32) -> i32;
        fn GetStdHandle(which: u32) -> isize;
    }
    // ATTACH_PARENT_PROCESS = -1; STD_OUTPUT_HANDLE = -11; a GUI exe has a null/invalid handle unless redirected
    let attached = unsafe { AttachConsole(u32::MAX) } != 0;
    let h = unsafe { GetStdHandle(-11i32 as u32) };
    if h != 0 && h != -1 {
        let _ = writeln!(std::io::stdout(), "{text}");
    } else if attached && let Ok(mut con) = std::fs::OpenOptions::new().write(true).open("CONOUT$")
    {
        let _ = writeln!(con, "{text}");
    } else {
        message_box(text, if ok { 0x40 } else { 0x10 }); // MB_ICONINFORMATION / MB_ICONERROR
    }
}

#[cfg(not(windows))]
fn show_text(text: &str, _: bool) {
    println!("{text}");
}

/// `setup`, `status`, `use`, `help`. None for anything else (that is XMCL calling).
fn cli(argv: &[String], store: &Store) -> Option<i32> {
    let env = setup::Env::from_env();
    let result: Result<String, String> = match argv.first().map(String::as_str)? {
        "help" => Ok(setup::HELP.into()),
        "status" => Ok(setup::status(&env, &store.root).join("
")),
        "use" => match argv.get(1) {
            Some(v) => setup::use_version(&store.root, v),
            None => Err("Uso: zenith-bridge use <versión|latest>".into()),
        },
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
                                    "Sin conexión con las builds de Zenith ({e}): uso Minecraft {}. Vuelve a ejecutar «setup» con internet para confirmarlo.",
                                    setup::FALLBACK_MINECRAFT
                                )),
                            )
                        }
                    }
                })
                .map(|l| l.join("
"))
            }),
        _ => return None,
    };
    let (text, code) = match result {
        Ok(t) => (t, 0),
        Err(e) => (e, 1),
    };
    show_text(&text, code == 0);
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
    let argv = collect_args(std::env::args_os().skip(1));
    if argv.is_empty() {
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
    let deps = Deps {
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
