//! The bridge as the instance's "Java" (SPIKE.md section 10): answers XMCL's Java probes.

use std::path::Path;

/// What the shim claims to be. The major must be >= the version JSON's `javaVersion.majorVersion` (25 for 26.3).
pub const SHIM_JAVA_VERSION: &str = "25.0.5";

/// Whether the executable is named like a Java launcher (`java.exe` / `javaw.exe`, any case).
pub fn is_shim_name(exe: &Path) -> bool {
    exe.file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .is_some_and(|s| s == "java" || s == "javaw")
}

pub enum Reply {
    Stderr(String),
    Stdout(String),
}

/// The answer to XMCL's probes, if `args` (without the exe) is one; None means a game launch.
/// No network, no files.
pub fn probe(args: &[String]) -> Option<Reply> {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    match a.as_slice() {
        ["-version"] => {
            let v = SHIM_JAVA_VERSION;
            Some(Reply::Stderr(format!(
                "openjdk version \"{v}\" 2026-01-20 LTS\nOpenJDK Runtime Environment Zenith (build {v}+0)\nOpenJDK 64-Bit Server VM Zenith (build {v}+0, mixed mode)"
            )))
        }
        ["JavaProxy", "-jvm_info"] => Some(Reply::Stdout("{\"java_arch\":\"64\"}".into())),
        _ => None,
    }
}
