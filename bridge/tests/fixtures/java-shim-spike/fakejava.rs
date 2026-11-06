// Fake "java" for the Zenith spike. Answers XMCL's Java probes like OpenJDK 25 and
// dumps what it receives when launched like a game. Build: rustc -O fakejava.rs
// (console) or rustc -O --cfg winsub fakejava.rs (windows subsystem).
#![cfg_attr(winsub, windows_subsystem = "windows")]
use std::io::Write;
#[cfg(old)] const V: &str = "21.0.5";
#[cfg(not(old))] const V: &str = "25.0.5";
extern "system" { fn GetCommandLineW() -> *const u16; }
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let a: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();
    if let Ok(out) = std::env::var("FAKEJAVA_LOG") { // log every call (probe or launch)
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(out) {
            let _ = writeln!(f, "CALL exe={} argc={} first={:?}", args[0], a.len(), a.iter().take(3).collect::<Vec<_>>());
        }
    }
    if a == ["-version"] {
        eprintln!("openjdk version \"{v}\" 2026-10-20 LTS
OpenJDK Runtime Environment Temurin-{v}+8 (build {v}+8-LTS)
OpenJDK 64-Bit Server VM Temurin-{v}+8 (build {v}+8-LTS, mixed mode, sharing)", v = V);
        return;
    }
    if a == ["JavaProxy", "-jvm_info"] {
        println!("{{\"java_arch\":\"64\",\"java_version\":\"25.0.5\"}}");
        return;
    }
    if a.first() == Some(&"-XshowSettings:properties") {
        eprintln!("Property settings:\n    java.version = 25.0.5\n    os.arch = amd64\n    sun.arch.data.model = 64");
        return;
    }
    // game launch
    let raw = unsafe { let p = GetCommandLineW(); let mut n = 0; while *p.add(n) != 0 { n += 1; } String::from_utf16_lossy(std::slice::from_raw_parts(p, n)) };
    if let Ok(out) = std::env::var("ARGDUMP_OUT") {
        let mut f = std::fs::File::create(out).unwrap();
        writeln!(f, "RAW_LEN={}", raw.chars().count()).unwrap();
        for x in &args { writeln!(f, "ARG={}", x).unwrap(); }
        writeln!(f, "PID={}", std::process::id()).unwrap();
        writeln!(f, "CWD={}", std::env::current_dir().unwrap().display()).unwrap();
    }
    println!("LWJGL Version: zenith");
    let _ = std::io::stdout().flush();
    eprintln!("stderr line from shim");
}
