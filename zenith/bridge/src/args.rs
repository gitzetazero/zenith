use std::fmt;
use std::path::PathBuf;

/// The command line as text. A non-UTF-8 argument is converted lossily instead of panicking
/// (`std::env::args` would, invisibly, in a GUI-subsystem exe).
pub fn collect_args(args: impl Iterator<Item = std::ffi::OsString>) -> Vec<String> {
    args.map(|a| a.to_string_lossy().into_owned()).collect()
}

/// What XMCL passed to the game, reduced to what Horizon needs. `java` is the bridge itself
/// (it is the instance's "Java"): nothing is derived from it.
pub struct LaunchArgs {
    pub java: PathBuf,
    pub version: String,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub xuid: Option<String>,
    pub game_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fullscreen: bool,
    pub join: Option<String>,
    pub world: Option<String>,
}

/// The value after `flag`; XMCL leaves unknown `${...}` variables as literal text, which counts as absent.
fn value<'a>(argv: &'a [String], flag: &str) -> Option<&'a str> {
    argv.windows(2)
        .find(|w| w[0] == flag && !w[1].starts_with("${"))
        .map(|w| w[1].as_str())
}

fn required(argv: &[String], flag: &str) -> Result<String, String> {
    value(argv, flag)
        .map(str::to_string)
        .ok_or_else(|| format!("XMCL did not pass {flag}."))
}

/// A size of -1 (or 0) means "no size", not an error.
fn size(argv: &[String], flag: &str) -> Result<Option<u32>, String> {
    match value(argv, flag) {
        None => Ok(None),
        Some(v) => v
            .parse::<i64>()
            .map(|n| u32::try_from(n).ok().filter(|&n| n > 0))
            .map_err(|_| format!("{flag} is not a number: {v}")),
    }
}

impl LaunchArgs {
    /// `argv` is XMCL's command line without the bridge's own path: `argv[0]` is `javaw.exe`.
    /// Unknown arguments (JVM options, the classpath) are ignored.
    pub fn parse(argv: &[String]) -> Result<Self, String> {
        let java = argv.first().ok_or("XMCL passed no command line.")?;
        if argv.iter().any(|a| {
            (a.starts_with("-javaagent:") && a.to_lowercase().contains("authlib-injector"))
                || a.starts_with("-Dauthlibinjector")
        }) {
            return Err("Sign in to XMCL with a Microsoft account that owns Minecraft: Java Edition (ely.by and LittleSkin accounts do not work).".into());
        }
        let join = value(argv, "--quickPlayMultiplayer")
            .map(str::to_string)
            .or_else(|| {
                let host = value(argv, "--server")?;
                Some(match value(argv, "--port") {
                    Some(port) => format!("{host}:{port}"),
                    None => host.to_string(),
                })
            });
        Ok(Self {
            java: PathBuf::from(java),
            version: required(argv, "--version")?,
            username: required(argv, "--username")?,
            uuid: required(argv, "--uuid")?,
            access_token: required(argv, "--accessToken")?,
            xuid: value(argv, "--xuid").map(str::to_string),
            game_dir: required(argv, "--gameDir")?.into(),
            assets_dir: required(argv, "--assetsDir")?.into(),
            width: size(argv, "--width")?,
            height: size(argv, "--height")?,
            fullscreen: argv.iter().any(|a| a == "--fullscreen"),
            join,
            world: value(argv, "--quickPlaySingleplayer").map(str::to_string),
        })
    }

    /// The JSON Horizon's Launch API reads with `--session-stdin`.
    pub fn session_json(&self) -> String {
        let mut v = serde_json::json!({
            "name": self.username,
            "uuid": self.uuid,
            "access_token": self.access_token,
            "user_type": "msa",
        });
        if let Some(x) = &self.xuid {
            v["xuid"] = x.as_str().into();
        }
        v.to_string()
    }

    /// `--assetsDir` is always `<root>/assets`.
    pub fn minecraft_root(&self) -> PathBuf {
        self.assets_dir
            .parent()
            .map(PathBuf::from)
            .unwrap_or_default()
    }
}

impl fmt::Debug for LaunchArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LaunchArgs")
            .field("java", &self.java)
            .field("version", &self.version)
            .field("username", &self.username)
            .field("uuid", &self.uuid)
            .field("access_token", &"<hidden>")
            .field("xuid", &self.xuid.as_ref().map(|_| "<hidden>"))
            .field("game_dir", &self.game_dir)
            .field("assets_dir", &self.assets_dir)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("fullscreen", &self.fullscreen)
            .field("join", &self.join)
            .field("world", &self.world)
            .finish()
    }
}
