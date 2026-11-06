# Maintainer guide

This guide is for people who maintain or contribute to the code. If you only want to play, read the [README](../README.md).

## How the fork works

Zenith comes from [Horizon](https://github.com/horizon-rust/horizon) (Minecraft in Rust, MIT licence) and is now a **hard fork**: it has its own names and its own layout, and it takes Horizon's work as translated patches, once a week. Rules:

1. **Zenith names.** Crates are `zenith-*` (`crates/zenith-core`, `zenith-world`...), the game is the binary `zenith`, the tools are `zenith-ctl`, `zenith-prepare` and `zenith-release-sign`. Rust code says `zenith_core::...`. What stays Horizon's on purpose is listed under "What keeps Horizon's name".
2. **Our own code has its own folders** at the root: `launcher/`, `bridge/` (its own Cargo workspace, so the root `Cargo.lock` is not involved), `branding/`, `ci/`. Changes inside the engine crates are small and marked `// zenith:` where they are hooks, so a ported Horizon patch keeps applying.
3. **Look upstream first.** Before fixing something, check with `gh pr list -R horizon-rust/horizon --search` that Horizon is not already doing it.
4. **Send generic work to Horizon.** It shrinks the difference between the two, and makes the weekly port apply cleanly.
5. **Inherited safety:** `tools/check_public_tree.py` must pass, no Mojang file is ever committed, and every test run uses temporary folders for `HORIZON_HOME` and `HORIZON_LAUNCHER_HOME`.

## Repository layout

```
README.md  LICENSE  THIRD_PARTY_NOTICES.md  CONTRIBUTING.md  SECURITY.md  CODE_OF_CONDUCT.md   (GitHub and licences expect these in the root)
Cargo.toml  Cargo.lock  clippy.toml  .gitattributes  .gitignore
native-hooks.json  research-lock.json   read by crates through a path from the repository root (see below)
AGENTS.md  CLAUDE.md                    instructions for coding agents
crates/        the game and its libraries, all zenith-* (the client is crates/zenith-client, binary `zenith`)
xtask/         build tasks: `package` (the game folder), `icons`, `tokens`
launcher/      Zenith Launcher: our fork of XMCL (Electron + Vue, MIT, credits kept), a self-contained pnpm workspace
bridge/        the bridge (own workspace): ownership check, signed installs, preparing resources, launch
branding/      logo, palette, banners, icons and the tools that regenerate them
ci/            sync (port_upstream.py, rename_map.py, sync.py), the intact check, the link check, the release publisher and their tests
docs/          documentation: this guide, PLAYER-GUIDE.md, SERVERS.md, GRAPHICS-SETTINGS.md, media/zenith/ (ours) and Horizon's engine documentation
tests/ tools/  Horizon's Python harnesses and scripts, kept because `xtask`, `zenith-prepare` and the parity tooling use them (tools/ci holds Horizon's CI helpers; ours are in ci/)
java/          the Java oracles and exporters `zenith-prepare` compiles against the player's own Minecraft (was mods/horizon)
packaging/     icons (zenith.ico, zenith.icns, hicolor/) and the files `xtask package` copies
third_party/   patched wgpu-hal, naga and mimalloc (Cargo `[patch]`)
.github/       zenith-sync.yml, zenith-release.yml, issue and PR templates
```

Why two JSON files stay in the root: `zenith-mod-api-gen`'s build script, the client (`include_str!`) and `zenith-prepare` read `native-hooks.json` and `research-lock.json` through the repository root, and `zenith-prepare` finds its root as the nearest folder with `research-lock.json` (in a release folder it sits next to the executable). Moving them means changing that contract. `java/` is not called `exporters/` because `exporters/` is where the release build puts the compiled bundle (it is git-ignored).

| Ownership | Paths |
|---|---|
| **Ours** | `docs/ZENITH.md`, `docs/PLAYER-GUIDE.md`, `docs/SERVERS.md`, `docs/GRAPHICS-SETTINGS.md`, `docs/media/zenith/`, `docs/superpowers/`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `launcher/`, `bridge/`, `branding/`, `ci/`, `.github/`, `crates/zenith-client/src/zenith_versions.rs`, `crates/zenith-client-core/src/zenith_brand.rs`, `crates/zenith-client/assets/zenith/`, the translator files under `crates/zenith-protocol/src/adapter/` |
| **Ours, though Horizon has the file too** (`merge=ours` in `.gitattributes`; the port skips Horizon's changes to them) | `README.md`, `CONTRIBUTING.md`, `.github/ISSUE_TEMPLATE/config.yml` |
| **Horizon's, with a small Zenith edit** (`// zenith:` markers) | a few dozen places under `crates/` and `xtask/src/main.rs` (the list is `ci/zenith-manifest.json`) |
| **Inherited and kept as they are** | everything else: `crates/**`, `tests/`, `tools/`, `docs/`, `AGENTS.md`... They describe Horizon's project, in Zenith's names |
| **Not carried** (Zenith never uses them; the port drops every Horizon change to them) | the dedicated server and bots, Horizon's own Rust launcher (`launcher/` in Horizon's tree), the telemetry server, the mod registry service, historical `benchmarks/`, Horizon's planning board and the screenshots in `docs/media/` (except `docs/media/brand/`) and its CI workflows. The list is `ci/prune-paths.txt`, in **Horizon's** names and layout |

Zenith is client-only: the integrated single-player server lives inside `zenith-client`, so the dedicated server is not needed to build or test the client.

### What keeps Horizon's name (on purpose)

These are protocols, file formats and attribution, not names, and renaming them would break compatibility or hide where the code comes from:

- the wire-protocol client brand `"horizon"` used to negotiate extensions with Horizon servers, and the friends protocol labels (`"horizon-friends ..."`);
- language and resource keys `horizon.*` and the folder `crates/zenith-client/assets/horizon/`;
- file formats and folders that players' files or other programs read: the game's mod folder `horizon-mods/` and its list `horizon-mods.txt` (and `overrides/horizon-mods` in modpacks), `horizon-shaders.toml` (shader pack manifest), `horizon-prepare.json`, `.horizon-mod-data`, `horizon-graphics.txt`, `horizon-extras.txt`;
- environment variables `HORIZON_*`. The four the bridge sets have `ZENITH_*` names accepted first (`ZENITH_TELEMETRY_URL`, `ZENITH_NO_HEARTBEAT`, `ZENITH_COSMETICS`, `ZENITH_OLDER_SERVERS`) with the `HORIZON_*` name as a fallback; the rest (`HORIZON_HOME`, `HORIZON_TEST_RESOURCES`...) are unchanged;
- Horizon's release asset names and its signing key: `horizon-windows-x64.zip` (the game zip the bridge downloads from our `nightly` release; renaming it would break installed launchers), `horizon-linux-x86_64.tar.gz` (the exporters are taken from Horizon's release), `crates/zenith-release-sign/release-keys.txt` and the signed message `horizon release <tag>`;
- upstream links and attribution: "Built on Horizon" in the README, `LICENSE` and `THIRD_PARTY_NOTICES.md` keep Horizon's copyright.

## Sync with Horizon

The workflow `zenith-sync` (`.github/workflows/zenith-sync.yml`) runs **every Monday at 03:07 UTC** (`7 3 * * 1`) or by hand, and is fully automatic. Because crates, binaries and folders have other names, git cannot merge Horizon's branch; the work is done by translating Horizon's patches. All the logic is in `ci/`, so you can run it locally (`python ci/sync.py run --upstream-ref upstream/main --dry-run` after `git remote add upstream https://github.com/horizon-rust/horizon.git` and a fetch).

1. `ci/upstream-synced.txt` holds the last Horizon commit ported. If Horizon's `main` is that commit, the run ends green at once.
2. `ci/port_upstream.py` takes the diff from that commit to Horizon's tip and **translates** it (`ci/rename_map.py`): paths (`crates/horizon-core` becomes `crates/zenith-core`, `mods/horizon` becomes `java/horizon`, icon names...) and text (`horizon_core::` becomes `zenith_core::`, `-p horizon-client` becomes `-p zenith-client`, `target/release/horizon-client` becomes `target/release/zenith`), leaving the names above that stay Horizon's and upstream URLs alone. It **drops** the changes to what Zenith does not carry (`ci/prune-paths.txt`), to the files Zenith owns although Horizon has them (`merge=ours`), to our folders and to `Cargo.lock` (regenerated with `cargo metadata`). Then it applies the patch with `git apply`, writes the new SHA in `ci/upstream-synced.txt` and commits.
3. **The strict gate** (`ci/sync.py checks`): the `ci/` unit tests, `check_zenith_intact.py` (every Zenith-own file exists, no file has fewer `// zenith:` markers, the key identifiers are there), `check_public_tree.py`, `check_links.py`, `cargo fmt --check`, `cargo check --workspace --tests --locked`, `cargo test -p zenith-protocol`, the bridge's fmt/clippy/tests, the client build and `check_launch_api.py`. Anything red means nothing is pushed.
4. **All green:** `ci/sync.py land` pushes the ported commit to `main` with a normal push (never forced; it refuses if the commit does not contain the current `main`, and the next run starts again from the new `main`), and the open `sync` issue and the old `sync/*` pull requests close.
5. **A patch that does not apply, or a red gate:** `main` is not touched. The attempt goes to a branch `sync/<date>-<short>` (for a patch that does not apply, with what applied plus `*.rej` files for the rest), a pull request is opened or updated, **one** issue titled "Sync blocked: conflict with upstream" with the label `sync` is opened or commented on, and the run ends red. `main` then stays behind Horizon until a person (or an agent) finishes the port: check out the branch, resolve the `*.rej` files by hand keeping both sides' intent, make `ci/sync.py checks` green, update `ci/upstream-synced.txt` if the branch does not, and push to `main`.

The policy is honest: this is a best-effort weekly attempt. A Horizon change that touches code we edited, a new crate that needs a rename rule, or a new `horizon-*` name that must stay Horizon's (add it to `ci/rename_map.py`, with a test) will block it; the report names the files. Nothing is lost, and the next run after a manual fix continues from the recorded SHA.

### Releases and the nightly rule

The sync lands continuously, but a **game build is published only from a commit whose Horizon part is covered by a Horizon nightly**: Zenith's release reuses the `exporters/` of a Horizon nightly Release, and if the code moved on, those classes could mismatch (unverified). After a landed port (or when already level) the sync dispatches `zenith-release` only if the ported Horizon commit is an ancestor of (or equal to) the commit of the newest `nightly-source/*` tag (`git merge-base --is-ancestor`). Otherwise the job summary says "release pending: waiting for the next Horizon nightly". The release gate still allows at most one release per UTC day (`force` publishes again), and the repository variable `ZENITH_AUTOPUBLISH=off` stops publishing.

### Files Horizon also has

`.gitattributes` marks `README.md`, `CONTRIBUTING.md` and `.github/ISSUE_TEMPLATE/config.yml` with `merge=ours`; the port reads that list to skip Horizon's changes to them. If you add an own file that Horizon also has, mark it the same way and add it to `EXTRA_FILES` in `ci/check_zenith_intact.py`.

### Not carrying more (or less)

Edit `ci/prune-paths.txt` (Horizon's names; directory prefixes end in `/`, `!` keeps). It only affects future ports. Never list a path that a client crate, `xtask` or the bridge builds from.

After changing Zenith's own work on purpose run `python ci/check_zenith_intact.py --update` and commit the manifest.

## What Zenith changes in the game

- On-screen name "Zenith" and none of Horizon's servers.
- Server version translator, on by default, with a per-server selector (`crates/zenith-client/src/zenith_versions.rs`). It comes from Horizon PR #8718, which is still a draft. See [`docs/SERVERS.md`](SERVERS.md).
- Bridge and launcher (`bridge`, `launcher`): checks the account owns the game, installs the signed builds and starts the client.
- Client only: see "Repository layout". The one edit in Horizon's files for this is the `TOOLS` list in `xtask/src/main.rs` (marked `// zenith:`).

### Version translator (PR #8718)

It was merged from commit `cc3ee16e4664eeced430dc05d68074602d79ba63` of branch `feature/older-servers-1.21.8`. To bring it again:

```sh
git fetch https://github.com/TheApple05/horizon.git feature/older-servers-1.21.8
git merge --no-ff FETCH_HEAD   # only from before the hard fork; now port it by hand
```

When Horizon merges the PR, the weekly port may not apply in `crates/zenith-protocol/src/adapter*`, `item.rs` and `tools/generate_registry_id_adapter.py`: resolve by taking Horizon's version. When merging, `..BEFORE_26_1` was added by hand in 1.21.8 and `dropped_components: &[]` in `v1_21_8_ids.rs`, and `item.rs` only keeps the component patch without an active adapter.

## Build, test and publish

```sh
cargo build --release -p zenith-client              # the game
cargo check --workspace --tests --locked          # everything compiles, tests included
cargo test -p zenith-protocol -p zenith-client     # protocol and version selector
cd bridge && cargo test                       # the bridge (another workspace)
python tools/check_public_tree.py                    # public tree check
python -m unittest discover -s ci             # port, sync, intact and link checks
python ci/check_links.py                      # relative links in our markdown
```

Publish: the `zenith-release` workflow (`workflow_dispatch`, with the UTC date) builds `main`, signs `SHA256SUMS` and publishes `nightly-<date>` (the launcher installer and portable zip only: what players download) and the moving `nightly` release (the signed game channel the launcher and game fetch from) (`ci/zenith_publish.sh`). The installer and portable are built by the `launcher` job from `launcher` (typecheck, tests, electron-builder, a check of `app.asar`), with the bridge built in the same run bundled inside; the portable exe is published zipped.

Names: the game is `zenith` (package `zenith-client`), the tools are `zenith-ctl` and `zenith-prepare`; most environment variables are still `HORIZON_*` (see "What keeps Horizon's name").

## Tests that fail on purpose or are known

- `entities::drop_motion_tests::drops_match_vanillas_motion_bit_for_bit` also fails on Horizon (the last bit of an `f64` differs on some machines).
- Zenith has its own `README.md`, so `tests/test_parity.py` and `tests/test_parity_gaps.py::test_the_readme_agrees_with_itself` fail (they expect Horizon's parity tables). The sync does not run them; skip them if you run the whole Python suite. `tools/parity_gaps.py` will not find its block in the README either.

## Status

Verified: signed versions and tamper tests, bridge tests, smoke test of the real client with an injected owner.

Not verified: real Play from the launcher, the real owner API response, server adapters against real servers (the real matrix has not been run) and Linux.

## Launcher

Zenith Launcher is our own fork of XMCL (Voxelum, MIT, Electron + Vue), renamed and re-branded, with the Zenith bridge built in. Its source lives here, in `launcher/` (the former repository `gitzetazero/zenith-launcher` is archived); releases: <https://github.com/gitzetazero/zenith/releases>. It replaces the earlier plan of porting XMCL to Rust (archived as `gitzetazero/zenith-launcher-rust-port`) and the interim installer that bundled unmodified XMCL (removed).

- Base: XMCL v0.73.1, kept mergeable with upstream (`git fetch upstream && git merge <tag>`); our changes are marked `// zenith:` or live in new files. Details, and the table of third-party IDs and endpoints XMCL embedded and what we did about each, are in `launcher/docs/ZENITH-LAUNCHER.md`. Run every launcher command from `launcher/`.
- No telemetry, no XMCL-hosted services. XMCL's Microsoft application ID is **not** used.
- On first start the launcher creates the `Zenith` instance, whose Java is a copy of the bundled bridge.
- The release workflow bundles the bridge it builds in the same run (`xmcl-electron-app/build/zenith/`). `tools/zenith/fetch-bridge.mjs` (signature and SHA-256 of the `nightly` release) is only for local builds. The launcher checks the app's update feed against this repository's Releases.
- **Open problem:** Microsoft sign-in uses our own Azure app (repository variable `ZENITH_MSA_CLIENT_ID`, embedded at build time); Mojang must still approve the app registration before sign-in works for players. Without the variable the launcher shows a clear message instead. The first releases are unsigned (SmartScreen warns) and not verified on third-party PCs.

## Legal notice

Zenith is not affiliated with, endorsed or sponsored by Mojang AB or Microsoft. Minecraft is a trademark of Mojang Synergies AB. This repository contains no Mojang files.
