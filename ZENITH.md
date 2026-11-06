# Maintainer guide

This guide is for people who maintain or contribute to the code. If you only want to play, read the [README](README.md).

## How the fork works

Zenith comes from [Horizon](https://github.com/horizon-rust/horizon) (Minecraft in Rust, MIT licence). Zenith adds a layer on top and receives Horizon's work by merging. Rules:

1. **Do not rename internals.** Crates, paths, internal binaries and `HORIZON_*` variables stay as they are. Renaming them would conflict on every sync.
2. **Our own code goes in `zenith/`**, with its own Cargo workspace where it is Rust (the bridge), so Horizon's `Cargo.toml` and `Cargo.lock` stay untouched. In Horizon's files only small hooks fit, marked `// zenith:`, with no reformatting.
3. **Look upstream first.** Before fixing something, check with `gh pr list -R horizon-rust/horizon --search` that Horizon is not already doing it.
4. **Send generic work to Horizon.** It shrinks the difference between the two.
5. **Inherited safety:** `tools/check_public_tree.py` must pass, no Mojang file is ever committed, and every test run uses temporary folders for `HORIZON_HOME` and `HORIZON_LAUNCHER_HOME`.

## Repository layout

```
README.md  ZENITH.md  LICENSE  THIRD_PARTY_NOTICES.md  CONTRIBUTING.md  SECURITY.md  CODE_OF_CONDUCT.md   (GitHub expects these in the root)
Cargo.toml  Cargo.lock  .gitattributes  native-hooks.json  research-lock.json
crates/        Horizon's engine and client crates (+ a few Zenith files marked `zenith:` or named zenith_*)
xtask/         Horizon's build tasks (one marked edit: the TOOLS list)
tests/ tools/ mods/ third_party/ docs/ packaging/   Horizon's, kept because the client, `horizon-prepare` (exporters in mods/, tools/native-hook-fragments) and `xtask package`/`icons` build from them; `third_party/` carries the patched wgpu-hal, naga and mimalloc
.github/       zenith-sync.yml, zenith-release.yml, issue and PR templates (ours); CODEOWNERS and friends (Horizon's)
zenith/
  README.md          player guide (install, update, uninstall)
  SERVERS.md         server versions and how the translator picks one
  bridge/            the bridge (own workspace): ownership check, signed installs, launch
  launcher/          Zenith Launcher: our fork of XMCL (Electron + Vue, MIT, credits kept), a self-contained pnpm workspace
  branding/          logo, palette, banners, icons and the tools that regenerate them
  ci/                sync, prune, intact check, link check, release publisher and their tests
```

| Ownership | Paths |
|---|---|
| **Ours** (Horizon does not have them, no conflict possible) | `ZENITH.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `zenith/**`, `docs/superpowers/**`, `.github/ISSUE_TEMPLATE/**`, `.github/PULL_REQUEST_TEMPLATE/**`, `.github/workflows/zenith-*.yml`, `crates/horizon-client/src/zenith_versions.rs`, `crates/horizon-client-core/src/zenith_brand.rs`, `crates/horizon-client/assets/zenith/**`, the translator files under `crates/horizon-protocol/src/adapter/` |
| **Ours, but Horizon has the file too** (`merge=ours` in `.gitattributes`) | `README.md`, `CONTRIBUTING.md`, `.github/ISSUE_TEMPLATE/config.yml` |
| **Horizon's, with a small Zenith edit** (`// zenith:` markers) | a dozen files under `crates/` and `xtask/src/main.rs` (the list is in `zenith/ci/zenith-manifest.json`) |
| **Inherited and kept as they are** | everything else: `crates/**`, `tests/`, `tools/`, `docs/` (except pruned files), `AGENTS.md`, `CLAUDE.md`, `THIRD_PARTY_NOTICES.md`... They describe Horizon's project |
| **Pruned** (Zenith never uses them; deleted and kept deleted by the sync) | the dedicated server and bots, Horizon's Rust launcher, the telemetry server, the mod registry service, historical `benchmarks/`, Horizon's planning board (`roadmap/`, `ROADMAP.md`), the screenshots in `docs/media/` (except `docs/media/brand/`) and Horizon's CI workflows. The list is `zenith/ci/prune-paths.txt` |

Zenith is client-only: the integrated single-player server lives inside `horizon-client`, so `horizon-server` and `horizon-bots` are not needed to build or test the client. A crate that a client crate depends on is never pruned (`horizon-server-files`, `horizon-mcp`, `horizon-bench`... stay).

## Sync with Horizon

The workflow `zenith-sync` (`.github/workflows/zenith-sync.yml`) runs **every 6 hours** (`7 */6 * * *`; it was hourly, which cost too much compute) or by hand. The goal: `main` stays level with Horizon's `main` (GitHub's "N commits behind" stays at 0) and **never loses anything of ours**. All the logic is in `zenith/ci/sync.py`, so you can run it locally.

1. `git fetch upstream main`. If Horizon's tip is already in `main`, the run ends green at once.
2. Otherwise it merges the tip with a **real merge commit** (no rebase, never a force push to `main`), so every commit of ours stays in the history.
3. Automatic conflict handling, in this order:
   - `zenith/ci/prune.py` deletes what is in `prune-paths.txt` (this settles modify/delete conflicts on pruned files and drops pruned files that Horizon adds, such as new workflows) and removes pruned crates from the root `Cargo.toml`;
   - files marked `merge=ours` in `.gitattributes` keep our side;
   - `Cargo.toml` and `Cargo.lock` take Horizon's side, then pruning and `cargo metadata` bring them back to our crate set;
   - any **other** conflict blocks the sync.
4. `zenith/ci/check_zenith_intact.py` compares the merged tree with `zenith/ci/zenith-manifest.json`: every Zenith-own file still exists, no Horizon file has fewer `// zenith:` markers than before, and the key identifiers are still there (`zenith_versions`, `zenith_brand`, older servers on by default, cosmetics `ON_BY_DEFAULT` false, the telemetry endpoint test, the workspace members). This is what catches a merge that is clean for git but silently drops our work. When you change Zenith's own work on purpose, run `python zenith/ci/check_zenith_intact.py --update` and commit the manifest.
5. Checks (`sync.py checks`): the zenith/ci unit tests, `check_public_tree.py`, `check_links.py`, `cargo fmt`, `cargo clippy -D warnings` on the whole (pruned) workspace, `cargo test -p horizon-protocol`, the bridge's fmt/clippy/tests, the client build and `check_launch_api.py`.
6. All green: the merge commit is pushed to `main` (a normal push; if `main` moved meanwhile the next scheduled run retries) and any open `sync` issue is closed.

**When it is blocked** (a conflict, the intact check or a failed check) `main` is not touched. The attempt goes to a branch `sync/<date>-<short>`, a pull request is opened (or updated), **one** issue titled "Sync blocked: conflict with upstream" with the label `sync` is opened (or commented on), and the run ends red. `main` then stays behind Horizon until a person (or an agent) resolves it: merge `upstream/main` locally, fix the conflicts keeping both sides' intent, run the checks and push. The next scheduled run closes the issue.

### Releases and the nightly rule

The sync merges continuously, but a **game build is published only from a commit whose Horizon part is covered by a Horizon nightly**: Zenith's release reuses the `exporters/` of a Horizon nightly Release, and if the code moved on, those classes could mismatch (unverified). After a successful push (or when already level) the sync dispatches `zenith-release` only if the merged Horizon commit is an ancestor of (or equal to) the commit of the newest `nightly-source/*` tag (`git merge-base --is-ancestor`). Otherwise the job summary says "release pending: waiting for the next Horizon nightly". The release gate still allows at most one release per UTC day (`force` publishes again), and the repository variable `ZENITH_AUTOPUBLISH=off` stops publishing. Consequence to know: while Horizon commits land faster than its nightlies, Zenith's `main` is level but the published nightly lags.

### Files Horizon also has

`.gitattributes` marks `README.md`, `CONTRIBUTING.md` and `.github/ISSUE_TEMPLATE/config.yml` with `merge=ours` and the workflow enables the driver (`git config merge.ours.driver true`). If you add an own file that Horizon also has, mark it the same way and add it to `EXTRA_FILES` in `zenith/ci/check_zenith_intact.py`.

### Pruning more (or less)

Edit `zenith/ci/prune-paths.txt` (directory prefixes end in `/`, `!` keeps). Run `python zenith/ci/prune.py`, then `cargo metadata` and the checks, and commit. Never prune a path that a client crate, `xtask` or the bridge builds from (check with `cargo metadata`).

## What Zenith changes in the game

- On-screen name "Zenith" and none of Horizon's servers.
- Server version translator, on by default, with a per-server selector (`crates/horizon-client/src/zenith_versions.rs`). It comes from Horizon PR #8718, which is still a draft. See [`zenith/SERVERS.md`](zenith/SERVERS.md).
- Bridge and launcher (`zenith/bridge`, `zenith/launcher`): checks the account owns the game, installs the signed builds and starts the client.
- Client only: see "Repository layout". The one edit in Horizon's files for this is the `TOOLS` list in `xtask/src/main.rs` (marked `// zenith:`).

### Version translator (PR #8718)

It was merged from commit `cc3ee16e4664eeced430dc05d68074602d79ba63` of branch `feature/older-servers-1.21.8`. To bring it again:

```sh
git fetch https://github.com/TheApple05/horizon.git feature/older-servers-1.21.8
git merge --no-ff FETCH_HEAD
```

When Horizon merges the PR, the sync may conflict in `crates/horizon-protocol/src/adapter*`, `item.rs` and `tools/generate_registry_id_adapter.py`: resolve by taking Horizon's version. When merging, `..BEFORE_26_1` was added by hand in 1.21.8 and `dropped_components: &[]` in `v1_21_8_ids.rs`, and `item.rs` only keeps the component patch without an active adapter.

## Build, test and publish

```sh
cargo build --release -p horizon-client              # the game
cargo test -p horizon-protocol -p horizon-client     # protocol and version selector
cd zenith/bridge && cargo test                       # the bridge (another workspace)
python tools/check_public_tree.py                    # public tree check
python -m unittest discover zenith/ci                # sync, prune, intact and link checks
python zenith/ci/check_links.py                      # relative links in our markdown
```

Publish: the `zenith-release` workflow (`workflow_dispatch`, with the UTC date) builds `main`, signs `SHA256SUMS` and publishes `nightly-<date>` (the launcher installer and portable zip only: what players download) and the moving `nightly` release (the signed game channel the launcher and game fetch from) (`zenith/ci/zenith_publish.sh`). The installer and portable are built by the `launcher` job from `zenith/launcher` (typecheck, tests, electron-builder, a check of `app.asar`), with the bridge built in the same run bundled inside; the portable exe is published zipped.

Binary and variable names are still Horizon's: `horizon-client`, `horizon-prepare`, `HORIZON_HOME`, `HORIZON_OLDER_SERVERS`, etc.

## Tests that fail on purpose or are known

- `entities::drop_motion_tests::drops_match_vanillas_motion_bit_for_bit` also fails on Horizon (the last bit of an `f64` differs on some machines).
- Zenith has its own `README.md`, so `tests/test_parity.py` and `tests/test_parity_gaps.py::test_the_readme_agrees_with_itself` fail (they expect Horizon's parity tables). The sync does not run them; skip them if you run the whole Python suite. `tools/parity_gaps.py` will not find its block in the README either.

## Status

Verified: signed versions and tamper tests, bridge tests, smoke test of the real client with an injected owner.

Not verified: real Play from the launcher, the real owner API response, server adapters against real servers (the real matrix has not been run) and Linux.

## Launcher

Zenith Launcher is our own fork of XMCL (Voxelum, MIT, Electron + Vue), renamed and re-branded, with the Zenith bridge built in. Its source lives here, in `zenith/launcher/` (the former repository `gitzetazero/zenith-launcher` is archived); releases: <https://github.com/gitzetazero/zenith/releases>. It replaces the earlier plan of porting XMCL to Rust (archived as `gitzetazero/zenith-launcher-rust-port`) and the interim installer that bundled unmodified XMCL (removed).

- Base: XMCL v0.73.1, kept mergeable with upstream (`git fetch upstream && git merge <tag>`); our changes are marked `// zenith:` or live in new files. Details, and the table of third-party IDs and endpoints XMCL embedded and what we did about each, are in `zenith/launcher/docs/ZENITH-LAUNCHER.md`. Run every launcher command from `zenith/launcher/`.
- No telemetry, no XMCL-hosted services. XMCL's Microsoft application ID is **not** used.
- On first start the launcher creates the `Zenith` instance, whose Java is a copy of the bundled bridge.
- The release workflow bundles the bridge it builds in the same run (`xmcl-electron-app/build/zenith/`). `tools/zenith/fetch-bridge.mjs` (signature and SHA-256 of the `nightly` release) is only for local builds. The launcher checks the app's update feed against this repository's Releases.
- **Open problem:** Microsoft sign-in needs an Azure app of our own, approved by Mojang (`ZENITH_MSA_CLIENT_ID`). Until then it shows a clear message instead of signing in. The first releases are unsigned (SmartScreen warns) and not verified on third-party PCs.

## Legal notice

Zenith is not affiliated with, endorsed or sponsored by Mojang AB or Microsoft. Minecraft is a trademark of Mojang Synergies AB. This repository contains no Mojang files.
