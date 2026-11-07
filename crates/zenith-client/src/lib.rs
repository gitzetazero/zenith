//! Gameplay and startup, above the reusable client library.
// No `#![recursion_limit]` or other inner attribute horizon-mixins doesn't know here: the
// launcher's horizon-mixins then rejects every native mod targeting this crate (#8169).
#![forbid(unsafe_code)]
extern crate self as zenith_client;
pub use zenith_client_lib::*;
pub mod addons_hud;
pub mod entry;
pub mod game;
pub mod mod_api;
