//! Test double for `zenith-client` and `zenith-prepare`: tests copy this binary under either name
//! into a fake build folder. It records what it was given in `record.log` and acts as
//! `behavior.json` (next to it) says.
use serde_json::{Value, json};
use std::io::{Read, Write};

fn main() {
    let me = std::env::current_exe().unwrap();
    let dir = me.parent().unwrap().to_path_buf();
    let tool = me.file_stem().unwrap().to_string_lossy().into_owned();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let b: Value = std::fs::read(dir.join("behavior.json"))
        .ok()
        .and_then(|t| serde_json::from_slice(&t).ok())
        .unwrap_or(json!({}));
    let after = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let mut stdin = String::new();
    if tool == "zenith" && args.iter().any(|a| a == "--session-stdin") {
        let _ = std::io::stdin().read_to_string(&mut stdin);
    }
    let env: Vec<(String, String)> = std::env::vars().collect();
    let record = json!({ "tool": tool, "args": args, "env": env, "stdin": stdin });
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("record.log"))
        .unwrap();
    writeln!(log, "{record}").unwrap();

    let lines = |key: &str| -> Vec<String> {
        b[key]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|l| l.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    };
    let code = if tool == "zenith-prepare" {
        let out = after("--output").map(std::path::PathBuf::from);
        if args.iter().any(|a| a == "--launch-api") {
            println!(
                "{}",
                json!({ "launch_api": "0.1.0", "minecraft": b["minecraft"] })
            );
            0
        } else if args.iter().any(|a| a == "--check") {
            i32::from(!out.is_some_and(|o| o.join("ok").is_file()))
        } else if b["prepare_fails"] == true {
            println!("{}", json!({ "error": "boom" }));
            1
        } else {
            println!("{}", json!({ "message": "Importing the game files" }));
            println!("{}", json!({ "fraction": 0.5 }));
            let out = out.unwrap();
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(out.join("ok"), "").unwrap();
            0
        }
    } else {
        for l in lines("stdout") {
            println!("{l}");
        }
        for l in lines("stderr") {
            eprintln!("{l}");
        }
        b["exit"].as_i64().unwrap_or(0) as i32
    };
    std::process::exit(code);
}
