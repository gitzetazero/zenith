# Zenith: design of the Horizon fork

> Historical design document (translated to English). Later decisions changed some parts: the Rust port of XMCL (phase 6) was replaced by a fork of XMCL (Zenith Launcher), and the repository was reorganised so everything of ours lives under `zenith/`. `ZENITH.md` is the current maintainer guide.

## Context

Zenith is a fork of [horizon-rust/horizon](https://github.com/horizon-rust/horizon), Minecraft: Java Edition 26.3 rewritten in Rust (MIT licence). It started from a real problem: the user could not join `<test server>` ("Unsupported client! Please use 1.7.2-26.2"). Horizon already ships a version translator, but it is off by default (`older_servers_enabled()`, `crates/horizon-client/src/net.rs:564`).

Goal: a client with more compatibility (among other things, servers of older versions) that is used from [XMCL](https://xmcl.app/en/) and later from a complete translation of XMCL to Rust, with more FPS and shaders that work, always receiving Horizon's patches.

### What that server answers today (measured with status pings, no login)

- It is a Velocity proxy with a single address and announces `Velocity 1.7.2-26.3`. The capture said 26.2, so it was most likely updated since. **Today it may already be possible to join without the translator; this has not been tried.**
- It returns the version number it is told: 777, 776, 775, 767 and 47 answered the same in the last measurement. Its ping cannot tell which version to use.
- With consecutive pings it cuts the connections from the fourth on. With 15 s pauses it answered all of them. It looks like a rate limit.

## The user's decisions

- **Name:** Zenith.
- **Launcher:** XMCL translated completely to Rust (1:1), working with the fork. Until then, regular XMCL with a bridge. The four fronts asked for earlier remain the goal: more game options, easier mods and shaders, appearance and diagnostics.
- **Versions:** the game stays a single version inside and translates for servers, like ViaFabric. It does not "really" play on old versions.
- **Target servers:** 1.20.2 or newer, including networks that accept ranges. The minimum asked for at first, 1.21.8, is mandatory.
- **Audience:** private first (the user and friends), prepared to open up later.
- **Repository:** public on GitHub, not announced.
- **Downloads:** public GitHub Releases, not announced, before the legal side is reviewed. The user accepts that anyone with the link can download them.
- **Publishing:** automatic. If the sync passes the checks, it publishes and installs itself.
- **Performance:** more FPS without installing anything. No Sodium `.jar` or similar is loaded, and there is no catalogue of performance mods.
- **Shaders:** Complementary, BSL, Bliss or Photon, and Sildur's or light ones.
- **Patches:** Zenith stays up to date with the original repository's.

## Goals, non-goals and anti-goals

**Goals**
- Open XMCL, press Play and be in Zenith with the player's Microsoft account.
- Join servers from 1.20.2 to 26.3 without configuring anything, including `<test server>`.
- Bring in Horizon's patches every day, with minimal manual work.
- Improve FPS and shaders with measurable gains.

**Non-goals for now**
- Servers that only accept versions older than 1.20.2 (for example 1.8 PvP).
- Playing with the rules or the world of an old version.
- Running Java mods in the Rust engine.
- Improving Horizon's Rust launcher: it stays in the repo untouched and Zenith does not distribute it.
- A website, a mod registry or services of our own.

**A failure even if it works**
- Bringing in Horizon's patches is so painful that the fork falls behind.
- Zenith touches or overwrites the worlds of the user's Horizon installation.
- The game contacts Horizon's services without the player asking.
- Zenith starts for an account that does not own the game.
- Publishing Mojang files or third-party shader packs in the repo.

## Fork rules (Claude's decision)

They are what makes syncing viable. They go in `ZENITH.md`.

1. **Do not rename internals.** Crates, paths, internal binaries and `HORIZON_*` variables stay the same. Renaming them would conflict on every sync.
2. **Zenith's code goes in `zenith/`, with its own Cargo workspace** (as `launcher/` does), so Horizon's `Cargo.toml` and `Cargo.lock` stay untouched. In Horizon's files only small hooks marked `// zenith:` fit, with no reformatting.
3. **Look upstream first.** Before fixing something, check with `gh pr list --search` that Horizon is not already doing it.
4. **Generic work is proposed to Horizon.** It reduces our difference. Every outside PR needs the user's approval and follows the agent rules of its `AGENTS.md`.
5. **Its safety rules are inherited:** `tools/check_public_tree.py`, no Mojang files in the repo, and temporary `HORIZON_HOME` / `HORIZON_LAUNCHER_HOME` folders in every test run.

## Phase 1: the base of the fork

**First check, because it decides the rest.** `xtask package` needs `exporters/`, which Horizon compiles against Mojang's jars on its Mac (`.github/workflows/game-build.yml`). Free runners do not have those jars. Plan: reuse the `exporters/` of the archive published with the same nightly. According to `docs/launcher-preparation.md` they are only original Horizon classes, and Zenith does not change their sources. If they cannot be obtained, automatic publishing does not work on free runners and the user gets the alternatives back (their own PC as a runner, or manual publishing).

**Repository**
- Public fork on GitHub as `gitzetazero/zenith`, remote `upstream`, cloned at `C:\Users\<user>\Documents\horizon`.
- Disable the inherited workflows with `gh workflow disable`, without deleting them: they depend on Horizon's runners and secrets, and deleting them would cause conflicts.

**Sync** (`.github/workflows/zenith-sync.yml`, daily, on free GitHub runners)
- Takes the latest `nightly-source/*` tag reachable from `upstream/main`. Those tags are permanent, and nightlies with unmerged PRs are excluded. It does not follow every commit: Horizon makes thousands a week.
- Merges into a temporary branch and runs fmt, clippy and tests.
- Checks that `horizon-client --launch-api` still offers the options the phase 2 bridge uses. That API is experimental and this is the early warning if it changes.
- If everything passes, it pushes to `main` and triggers publishing. If there is a conflict or something fails, it opens a PR with the list of files and neither touches `main` nor publishes.
- Uses a token with workflow permission, because `GITHUB_TOKEN` cannot push Horizon's changes to `.github/workflows`.

**Automatic publishing** (`.github/workflows/zenith-release.yml`)
- Builds the game and the bridge for Windows and Linux, signs, and publishes at most one build a day.
- Accepted limit: nobody opens the game before publishing. Horizon's smoke test covers its commit, not our merge, and a failure that only shows when playing will reach everybody.
- Two cheap brakes: the repo variable `ZENITH_AUTOPUBLISH=off` stops publishing, and the bridge keeps the previous build to go back to.

**Game identity** (smallest possible change)
- Signing: our own key for Zenith's files, verified by the bridge with `crates/horizon-release-sign`. The private half is an Actions secret.
- Visible name: replace "Horizon" with "Zenith" in a single place, when the game's texts are loaded, without editing the language files.
- Game traffic (Claude's decision): an allow list of destinations. Allowed: Microsoft, Xbox and Mojang, and GitHub for the builds. Only when the player asks: the friends relay. Off: telemetry and the "I'm playing" ping (`crates/horizon-telemetry`, `crates/horizon-client/src/heartbeat.rs`) and cosmetics (`HORIZON_COSMETICS=0`, active by default today and tied to the UUID). Environment variables set by the bridge are preferred over changing code.
- `LICENSE`, `THIRD_PARTY_NOTICES.md`, the notice of non-affiliation with Mojang and Microsoft, and the credit to Horizon are kept.

Because Zenith does not distribute the Rust launcher, `launcher/` no longer needs to be touched. This removes the area with the most conflict risk.

**Done when:**
- A real nightly reaches `main` and is published by itself.
- The published archive passes signature verification.

## Phase 2: XMCL as the launcher, with a bridge

**Why it is viable.** Horizon documents how to start it from external launchers ([`docs/api/launch-guide.md`](https://github.com/horizon-rust/horizon/blob/main/docs/api/launch-guide.md), experimental API 0.1.0). XMCL (MIT, Electron) allows a per-instance "prepend command" that is put before the launch command (`prependCommand`, `xmcl-keystone-ui/src/views/BaseSettingJava.vue:157`, `packages/core/launch.ts:701`). XMCL does not have to be modified.

**Sign-in.** XMCL does it, unmodified and with its own registration, as for any other game. Zenith includes and copies no application ID, so this phase needs no approval from Mojang.

**The bridge:** `zenith/bridge/`, a small Rust program (`zenith-bridge.exe`). XMCL downloads Minecraft 26.3 and Java, and runs `zenith-bridge <java> … net.minecraft.client.main.Main --username … --uuid … --accessToken … --gameDir …`. The bridge does not start Java. It does this:
1. **Reads the arguments** from XMCL.
2. **Checks that the account owns the game** (step 0 of Horizon's guide): it refuses if `--userType` is not `msa` and asks the Minecraft services with that token. Offline, ely.by or LittleSkin accounts do not start Zenith.
3. **Installs or updates Zenith** from the Releases, verifying the signature. It keeps the previous build.
4. **Prepares the resources:** `horizon-prepare --check` and, if it fails, `--prepare --complete --progress json` with the files XMCL already downloaded and its Java (`--java-home`). Only for that owner does it download the server jar, which the preparation needs. There is a single prepared folder per Minecraft version. The first time takes minutes, so a progress window must be visible.
5. **Starts the game:** `horizon-client --resources P --game-dir G --home G --width W --height H --session-stdin`. The session goes through standard input, never through the command line. It forwards the output, returns the exit code and writes the crash report path in the format XMCL detects (`Crash report saved to:`).

**First a throwaway test**, with Horizon's current nightly unchanged, before building the real bridge. It must answer:
- whether XMCL accepts the prepend command and exactly which arguments it passes;
- whether it stops marking the instance as "launching" without a Java log;
- whether its shaders tab works on a vanilla instance;
- whether `--minecraft-dir` works with XMCL's folder structure.

**Setup.** `zenith-bridge setup` leaves the bridge ready in a fixed folder and creates the 26.3 "Zenith" instance with the prepend command. If XMCL's instance format does not allow that safely, it stays as three documented manual steps.

**What this phase gives and what it does not**

| What you asked for | With the bridge | With the Rust launcher (phase 6) |
| --- | --- | --- |
| Appearance | XMCL's | Zenith's brand and theme |
| Game options | XMCL instances and in-game settings | FPS presets and Zenith settings in the launcher |
| Mods and shaders | Shaders and resource packs from Modrinth and CurseForge (to be confirmed in the test) | Filtered mods tab, with `.hmod` |
| Diagnostics | Whatever XMCL detects from crashes and logs | Own panel |

**What is lost compared with Horizon's launcher**, unless added later:
- Building native mods (`.hnmod`).
- Porting modpacks.
- Hosting servers.
- Updates with Horizon's list of versions.

Also, XMCL's mods tab will offer Java mods that do not work in Zenith.

**Inherited limit.** XMCL passes the token on the command line to the bridge, as it does for any game. Other programs of that user could read it during startup. It disappears in phase 6.

**Tests.** The argument parser with real XMCL lines, the refusal of accounts that do not own the game or are not `msa`, and the check that the token never appears in the `horizon-client` command.

**Done when:**
- On a clean XMCL profile, with the user's account, Play opens Zenith on an online server.
- An offline account is refused with a clear message.
- A build with a bad signature is refused.
- A 10-minute traffic capture of play shows only allow-listed destinations.
- `%APPDATA%\horizon` and `%APPDATA%\horizon-launcher` stay unchanged (before and after comparison).

## Phase 3: joining servers

**Choosing the version.** Today it is chosen by exact match of the number the server announces (`older_server`, `net.rs:570`). New order:
1. The version chosen by hand for that server, if any.
2. The announced number, if it is one we know how to translate. `ServerVersion::from_protocol` and `ALL` (`crates/horizon-protocol/src/adapter.rs`) are reused.
3. If the server refuses during login (`net.rs:686`) with a message that includes a range such as "1.7.2-26.2": a single retry with the newest version we can translate inside that range.

There is no probing with pings per version: `<test server>` answers yes to all of them and cuts fast pings, so it would add nothing and could trigger protections. Without a range in the message there is no retry, and the error suggests using the selector.

**Changes**
- `older_servers_enabled()` becomes active by default; `HORIZON_OLDER_SERVERS=0` turns it off.
- The server list (`crates/horizon-client/src/multiplayer.rs:261`) uses the same logic, so a server that can be joined is not marked incompatible.
- Per-server selector ("Automatic", "26.2", …) in the edit screen. It is saved in a separate Zenith file indexed by address, so `servers.dat`, which must stay compatible with vanilla, is not altered. Versions from 1.20.2 to 1.21.7 are marked experimental.
- Show "Joining as 26.2" when connecting. If the translation fails, the error says which version was tried.

**Integrating 1.20.2 to 1.21.8.** Today the translator covers 1.21.9 to 26.2. The rest is in [horizon-rust/horizon#8718](https://github.com/horizon-rust/horizon/pull/8718): a draft with conflicts, +18,944 lines, under active development. Its branch is merged into Zenith and the conflicts are resolved. When Horizon merges it, the sync absorbs it. Meanwhile every change in that branch is brought again by hand.

**No Via code is copied.** ViaVersion and ViaFabricPlus are GPL-3; Zenith stays MIT.

**Known limit.** The game is still 26.3 inside. On old servers there can be visible differences, and some anticheats may react.

**Verification**
- Tests of `crates/horizon-protocol` and new selector tests, with the real response of `<test server>` and the captured message as test data.
- A matrix with real vanilla servers locally, in offline mode, one per protocol: 1.20.2, 1.20.4, 1.20.6, 1.21.1, 1.21.3, 1.21.4, 1.21.5, 1.21.6, 1.21.8, 1.21.10, 1.21.11, 26.1.2, 26.2 and 26.3. With `horizon-ctl`: join, move, break and place a block, chat, inventory, change dimension, die and respawn. It is run by hand, not on every sync, and the jars are not uploaded to the repo.
- `<test server>` on automatic and also forcing 26.2 with the selector, because if it joins directly as 26.3 the test does not exercise the translator.

**Done when (Claude's decision):**
- From 1.21.8 to 26.3, the whole matrix passes. It is mandatory.
- From 1.20.2 to 1.21.7, do what is possible and list what fails as a known problem.
- The user joins `<test server>` from XMCL without configuring anything else and plays 30 minutes without being kicked. A kick by anticheat is investigated; if it comes from being 26.3 inside and cannot be fixed, it is documented and the user decides.

## Phase 4: more FPS

- Measure first: `tools/perf_bench.py` on the user's PC, Zenith against the same Horizon nightly.
- Deliverable: "Max FPS", "Balanced" and "Quality" presets over the settings that already exist (`docs/graphics.md`). They are chosen in the game, and in the launcher when phase 6 exists.
- The engine is only touched if the benchmark proves the improvement, and it is proposed to Horizon first, which is already working on this. That way nothing is lost when syncing.
- No percentage is promised before measuring: it depends on the PC, and the starting point is fixed in the first measurement.

## Phase 5: shaders

- Start from the existing code (`crates/horizon-shaders`, `crates/horizon-client/src/game/shader_packs.rs`), do not rewrite.
- First check its real state by running it: the roadmap and `docs/shaders.md` do not fully agree, and there are open failures such as [#9137](https://github.com/horizon-rust/horizon/issues/9137).
- Order: Sildur's or light ones → BSL → Complementary → Bliss and Photon.
- Done per pack: loads without errors on Windows, normal world, Nether and End, a fixed scene captured with `horizon-ctl` and compared by eye with the same pack in Iris on an XMCL Java instance, with no frame-rate drop unacceptable to the user.
- Packs are never uploaded to the repo: they are downloaded locally (each pack's licence). The repo's test data are packs of our own, such as `crates/horizon-shaders/tests/packs/terrain-test`.

## Phase 6: XMCL translated to Rust (Zenith launcher)

**User's decision: complete 1:1 translation.** It is the largest piece of the whole project: about 7.8 MB of TypeScript and Vue in about 1,400 files, not counting tests. Larger than all of Horizon's launcher (3.3 MB). While it lasts, play continues with regular XMCL and the bridge, so nobody waits. It will have its own detailed plan, with mock-ups to approve before building.

**Condition to start (Claude's decision):** having Mojang's approval. Without it, the resulting launcher could not sign in, and it is not worth investing in this before knowing.

**Blockers only the user can resolve**
- **Own application ID.** A new launcher cannot use XMCL's, which is in `xmcl-runtime/user/pluginOfficialUserApi.ts:12`, nor another application's. That value is not translated. The user registers an app in Azure and asks Mojang for approval ("App ID review"; Horizon documents that it takes days to weeks, steps in `docs/microsoft-account.md`). It is worth asking at the start of phase 1, because it depends on nothing.
- **Own CurseForge API key**, for the same reason.
- **If Mojang rejects it:** Zenith keeps working with regular XMCL and the bridge, and this phase is rethought with the user. Nothing else is blocked.

**Claude's decisions, reviewable in this phase's plan**
- **Repository:** `gitzetazero/zenith-launcher`, new, keeping XMCL's MIT licence and copyright. The base is fixed at XMCL's latest version when starting (v0.73.1 today).
- **Interface:** GPUI Fast, the same framework as Horizon's launcher. Vuetify does not exist in Rust, so an equivalent component library is built first. "1:1" means the same screens, flows and options, not pixel equality.
- **Reuse:** XMCL's language files are kept. Before writing anything from scratch, Horizon's launcher (MIT) is looked at, which already solves accounts, downloads and Modrinth in Rust. The phase 2 bridge becomes a library, and the token stops going on the command line.

**The inventory is the contract.** `docs/xmcl-parity.md` lists each of the 52 services, each screen and each `@xmcl/*` package, with its state: pending, ported or excluded. Only the user can approve an exclusion. Exceptions proposed from the start:
- `xmcl-electron-app` and `deskgap-app`, because the Rust program itself replaces them.
- XMCL's telemetry and its update channel, because they are replaced by Zenith's.

**Order, so every stage is usable**
1. Window, components, themes, languages and settings.
2. Microsoft account, instances and starting Zenith. From here it can already replace XMCL to play Zenith.
3. Downloads: Minecraft, Java, Forge, Fabric, Quilt, OptiFine and mirrors, with Java instances.
4. Resources: mods, shaders, resource packs and worlds, Modrinth and CurseForge, and `.hmod`.
5. Modpacks: import and export.
6. The rest, including P2P multiplayer and third-party accounts (Yggdrasil, ely.by, LittleSkin).

What is Zenith's own is added when its equivalent exists: FPS presets, diagnostics, notices of which mods work on each instance, and brand and theme.

**Accounts that do not own the game.** Offline or third-party accounts never start Zenith's engine. Whether they are ported for Java instances is decided with the user when that stage is reached.

**Method.** XMCL's tests (about 2 MB) are translated too and serve as the specification.

**Keeping up with XMCL.** Its code can no longer be merged. A weekly workflow opens an issue with XMCL's new commits since the base (in the last month, 41 commits and 3 versions), and it is decided which to translate.

**Done when:**
- The inventory has nothing pending.
- The translated tests pass.
- Every screen has been compared with XMCL at 1200x720 and 800x400 (its default and minimum window).
- The user uses it instead of XMCL and gives the go-ahead.

## Order and scope

Order (Claude's decision): Phase 1 → 2 → 3 → 4 → 5 → 6. The bridge goes before the translator because it allows playing and testing Zenith with the user's account without waiting for Mojang. The translation goes last because it is the largest, needs Mojang's approval and nothing depends on it: FPS presets are chosen in the game until then.

Approving this plan approves the complete design and allows building phases 1, 2 and 3, **including creating the public fork and publishing builds automatically**. Each of the others will have its detailed plan before starting.

## What the user needs to do

1. **Try `<test server>` again** with Horizon as it is: the server now announces 26.3.
2. **Install XMCL** on this PC and sign in with their account, for the phase 2 throwaway test. It has not been checked whether it is already installed.
3. **Create the sync token** (contents and workflows permissions) and save it as a repo secret, without pasting it in the chat.
4. **Generate the signing key** with the command Claude gives and save it as a secret. The private half never goes to the chat or the repo.
5. **Register the Azure app and ask Mojang for approval**, as soon as possible even if it is for phase 6, because it takes time. Also a CurseForge API key. Claude gives the steps; the accounts are theirs.
6. **Approve every PR to Horizon** and the launcher mock-ups.

## Risks

- **Automatic publishing without opening the game:** the accepted risk with the most impact. If there is a failure, `ZENITH_AUTOPUBLISH=off` is used and the previous build is restored.
- **Size of the 1:1 translation:** several times larger than the rest of what must be built, and it leaves Zenith maintaining a complete launcher on its own. Translating also what is useless in Zenith (such as the loader installers for the Rust engine) adds work with no gain for that engine. The inventory allows excluding parts if the user changes their mind.
- **`exporters/`:** if they cannot be reused, automatic publishing changes (see phase 1).
- **Experimental launch API:** it can change without notice. It is detected in the sync before publishing, and then the bridge has to be adjusted.
- **Dependence on XMCL:** the prepend command and the argument format are XMCL's and can change in its updates, which we do not control until we have the Rust launcher. XMCL also has its own telemetry, with a setting to turn it off (`disableTelemetry`). It should be turned off if it is not to be sent.
- **Horizon's pace:** the most likely conflicts will be in the translator. If the sync gets stuck, the phase in progress is paused and it is handled first, because staying up to date is a goal and not an extra.
- **#8718 unfinished:** it can change a lot or not be merged. If it is abandoned, that code becomes ours and 1.21.8 depends on it. From 1.21.9 to 26.2 does not depend on it.
- **Experimental translator:** turning it on by default can cause kicks. That is why there is a switch, a selector and clear messages.
- **Server rules:** joining with an unofficial client may go against a server's rules. The user should review those of `<test server>` before the 30-minute test.
- **Base version change:** the user already has `26.4-snapshot-6` installed. When Horizon moves to another version, it will arrive as a large patch in the sync, and the bridge must follow the version declared by `horizon-prepare --launch-api`.
- **Legal:** the original repo has open issues about its "clean room" claims ([#1555](https://github.com/horizon-rust/horizon/issues/1555), [#6741](https://github.com/horizon-rust/horizon/issues/6741)). They were not evaluated here and the fork would inherit them from the first public Release. A lawyer should be consulted before announcing it.
- **Name, before announcing it:** `zenith-rust` is free on GitHub, but "Zenith" is already used by other Minecraft projects, including a cheats repo. Review it then.

## Global verification

- `git log` confirms that `main` contains the latest nightly and `git diff <nightly>..main --stat` shows a small difference outside `zenith/`.
- `python3 tools/pr_ready.py` passes on every change. It needs JDK 25, which this PC lacks (Rust 1.99 and Python 3.12 are already there).
- The tests in `ZENITH.md` are run on Windows. Horizon only declares its harnesses on Linux and macOS, so Windows is the gap to watch.
- After each phase, the user tests it on their PC.

## Not yet verified

Nothing has been compiled or run: this comes from reading the repos on GitHub and from pings to the server. Unconfirmed:
- Whether `<test server>` can be joined today as 26.3, because a ping does not prove the login.
- Why the range changed compared with the capture.
- What Velocity answers to a version it does not support. From memory, its maximum version, in which case step 2 of the selector would already solve the captured case.
- Everything the phase 2 throwaway test must answer. Until it is done, viability with XMCL is a deduction from its code, not a fact.
- Where to get `exporters/` from.
- The real build time on GitHub.
- Whether GPUI Fast covers everything XMCL does with Vuetify and Electron. It is validated with the first stage of phase 6.
