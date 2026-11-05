// A console exe loses its stdout under XMCL's `shell: true` + `detached: true` (SPIKE.md), so this is a GUI-subsystem exe.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::io::Write;
use zenith_bridge::args::collect_args;
use zenith_bridge::builds::{Store, embedded_keys, http_fetch};
use zenith_bridge::owner::{self, MINECRAFT_API};
use zenith_bridge::play::Stream;
use zenith_bridge::run::{Deps, run};

/// No console to read errors from: show them.
#[cfg(windows)]
fn show_error(text: &str) {
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
    // 0x10 = MB_ICONERROR
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            wide(text).as_ptr(),
            wide("Zenith").as_ptr(),
            0x10,
        )
    };
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
