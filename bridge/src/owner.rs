use serde_json::{Map, Value};
use std::path::Path;
use std::time::Duration;

pub const MINECRAFT_API: &str = "https://api.minecraftservices.com";

/// How long a positive check keeps an offline start working.
const CACHE_SECS: u64 = 30 * 86_400;

#[derive(Debug)]
pub enum OwnerCheck {
    Owner,
    NotOwner,
    /// Mojang does not accept the token at all (401/403): an offline profile, or an expired sign-in.
    InvalidToken,
    Unreachable(String),
}

/// Same rules as the Horizon launcher's `owns_game`: `game_minecraft` or `product_minecraft` (Game Pass included).
pub fn check(base_url: &str, token: &str) -> OwnerCheck {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(15))
        .build();
    let res = agent
        .get(&format!("{base_url}/entitlements/mcstore"))
        .set("Authorization", &format!("Bearer {token}"))
        .call();
    match res {
        Ok(resp) if resp.status() == 200 => match resp.into_json::<Value>() {
            Ok(v) => {
                let owns = v["items"].as_array().is_some_and(|items| {
                    items.iter().any(|i| {
                        matches!(
                            i["name"].as_str(),
                            Some("game_minecraft" | "product_minecraft")
                        )
                    })
                });
                if owns {
                    OwnerCheck::Owner
                } else {
                    OwnerCheck::NotOwner
                }
            }
            Err(e) => OwnerCheck::Unreachable(format!("unreadable answer: {e}")),
        },
        Ok(resp) => OwnerCheck::Unreachable(format!("unexpected status {}", resp.status())),
        Err(ureq::Error::Status(401 | 403, _)) => OwnerCheck::InvalidToken,
        Err(ureq::Error::Status(code, _)) => OwnerCheck::Unreachable(format!("status {code}")),
        Err(e) => OwnerCheck::Unreachable(e.to_string()),
    }
}

/// Temp file + rename, so a crash never leaves a half-written cache. A failed write only costs the offline grace.
fn save(cache: &Path, owners: Map<String, Value>) {
    if let Some(dir) = cache.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = cache.with_extension("json.tmp");
    if std::fs::write(&tmp, Value::Object(owners).to_string()).is_ok()
        && std::fs::rename(&tmp, cache).is_err()
    {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// Decides whether to start. `cache` is `owners.json`: UUID -> unix time of the last positive check.
pub fn gate(result: OwnerCheck, uuid: &str, cache: &Path, now_secs: u64) -> Result<(), String> {
    let key = uuid.replace('-', "").to_lowercase();
    let mut owners: Map<String, Value> = std::fs::read(cache)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    match result {
        OwnerCheck::Owner => {
            owners.insert(key, now_secs.into());
            save(cache, owners);
            Ok(())
        }
        OwnerCheck::NotOwner => {
            if owners.remove(&key).is_some() {
                save(cache, owners);
            }
            Err(
                "Sign in to XMCL with a Microsoft account that owns Minecraft: Java Edition."
                    .into(),
            )
        }
        // An offline profile (a name, no Microsoft sign-in) plays only on a PC where an account that owns
        // Minecraft: Java Edition signed in within the grace period: ownership is still required, just not
        // re-proven for every profile.
        OwnerCheck::InvalidToken => {
            let fresh = |v: &Value| {
                v.as_u64()
                    .is_some_and(|t| t <= now_secs && now_secs - t < CACHE_SECS)
            };
            if owners.values().any(fresh) {
                Ok(())
            } else {
                Err("Offline profiles need Minecraft: Java Edition to be verified on this PC first: sign in once with the Microsoft account that owns it (in the last 30 days), then you can play with an offline profile.".into())
            }
        }
        OwnerCheck::Unreachable(why) => {
            // a future-dated entry (clock moved back) is not trusted
            let last = owners.get(&key).and_then(Value::as_u64);
            if last.is_some_and(|t| t <= now_secs && now_secs - t < CACHE_SECS) {
                Ok(())
            } else {
                Err(format!(
                    "Could not check that the account owns Minecraft ({why}). Connect to the internet and try again."
                ))
            }
        }
    }
}
