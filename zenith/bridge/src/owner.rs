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
        Err(ureq::Error::Status(401 | 403, _)) => OwnerCheck::NotOwner,
        Err(ureq::Error::Status(code, _)) => OwnerCheck::Unreachable(format!("status {code}")),
        Err(e) => OwnerCheck::Unreachable(e.to_string()),
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
            // a cache that can't be written only costs the offline grace
            if let Some(dir) = cache.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(cache, Value::Object(owners).to_string());
            Ok(())
        }
        OwnerCheck::NotOwner => Err(
            "Inicia sesión en XMCL con una cuenta de Microsoft que posea Minecraft: Java Edition."
                .into(),
        ),
        OwnerCheck::Unreachable(why) => {
            let last = owners.get(&key).and_then(Value::as_u64);
            if last.is_some_and(|t| now_secs.saturating_sub(t) < CACHE_SECS) {
                Ok(())
            } else {
                Err(format!(
                    "No se pudo comprobar que la cuenta posee Minecraft ({why}). Conéctate a internet e inténtalo de nuevo."
                ))
            }
        }
    }
}
