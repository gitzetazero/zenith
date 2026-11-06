# Zenith Launcher: what differs from XMCL

Zenith Launcher is a fork of **X Minecraft Launcher (XMCL)** by Voxelum / ci010 (<https://github.com/Voxelum/x-minecraft-launcher>, MIT license).
XMCL's `LICENSE` and copyright are kept. Zenith's changes are concentrated and marked with a `// zenith:` comment, or live in new files
(`xmcl-runtime/zenith/`, `xmcl-electron-app/main/pluginZenith.ts`, `xmcl-keystone-ui/src/views/Market*.vue`, `tools/zenith/`). The app keeps all of XMCL's languages; the
new strings exist in English (default) and Spanish and fall back to English elsewhere.

## Base

| Item | Value |
|---|---|
| Upstream repository | `Voxelum/x-minecraft-launcher` (git remote `upstream`) |
| Base tag | `v0.73.1` |
| Base commit | `aeb524039` (`chore(release): version 0.73.1 (#1825)`) |
| Zenith Launcher version | `0.73.1-zenith.N` (`N` increases with each Zenith release) |
| Where it lives | this fork, and from the next import `zenith/launcher/` of `gitzetazero/zenith` ([import instructions](../tools/zenith/import-instructions.md)) |

## What is different

- **Zenith only.** The launcher installs and plays Zenith, nothing else. Creating an instance (the Add instance dialog, drag and drop, modpack import, commands) always goes through
  `InstanceService.createInstance`, which calls `enforceZenithInstance` (`xmcl-runtime/zenith/only.ts`): it pins the Zenith Minecraft version (`26.3`, the same fallback the bridge uses; a patch version of that line is kept),
  sets `java` to the bridge shim, empties every Java option and **refuses** mod loaders (Forge, NeoForge, Fabric, Quilt), OptiFine, LabyMod, Bedrock and modpack upstreams. The UI only offers a name.
  Existing non-Zenith instances are not supported.
- **Data folder `.minecraftz`.** XMCL's `.minecraft` (Windows/Linux `<appData>/.minecraft`, macOS `<appData>/minecraft`) became `<appData>/.minecraftz` (macOS `minecraftz`) and XMCL's default data root `~/.minecraftx` is now the same folder
  (`xmcl-runtime/zenith/dataFolder.ts`, used by `LauncherApp.minecraftDataPath` and the first-run wizard). The official `.minecraft` is never read or written: there is no import-from-`.minecraft` setting; `migrateMinecraftFile` is not wired to the UI.
  The launcher's own config stays in `%APPDATA%\zenith-launcher`.
- **No Java UI.** Preferred/global Java, memory (min/max/auto), JVM arguments, prepend and pre-execute commands, environment variables, Java version checks, installers, the Java problem dialog and the "Fix Java" flows are gone.
  The bridge is still the instance's `java` (an invisible copy of `zenith-bridge` installed as `<ZENITH_HOME or %APPDATA%\zenith>\java\bin\java.exe` and `javaw.exe`); saving the global settings writes the old Java values back as empty.
  Kept: window size, fullscreen, monitor, shared `options.txt`, game language sync, quick launch, quick play of a server or a world (what `zenith/bridge/src/args.rs` forwards).
- **Removed features**: the XMCL AI Agent (service, tools, settings, pages), Minecraft Together / P2P multiplayer (WebRTC peers, ICE/signaling, `PeerService`, friends), the XMCL account service, Bedrock, the Java mods and blueprints pages,
  modpack import/export/publish, remote Java server hosting, the Modrinth publish dialogs, the community translation service, remote flights, and `node-datachannel`. Telemetry is **hard-off**: no exporter, no key, no setting (`xmcl-runtime/infra/plugins/telemetry.ts` only answers the renderer's IPC with no-ops;
  `xmcl-keystone-ui/src/telemetry.ts` starts disabled and nothing enables it).
- **Zenith Market** (`/market`, replaces the modpack store): see below.
- **Discord presence**: "Playing Zenith", the `zenith-logo` asset and a button to the Zenith repository (`xmcl-runtime/zenith/presence.ts`). Disabled silently without an application id.
- **About page** (`SettingAbout.vue`): author and maintainer `gitzetazero`, the Zenith repository, version, MIT licence, one credits paragraph (XMCL by Voxelum / ci010, MIT; the engine is a fork of Horizon).
- **Identity**: product name `Zenith Launcher`, app id `dev.zenith.launcher`, deep-link scheme `zenith-launcher://`, AppUserModelId `dev.zenith.launcher`, window titles, tray, the brand mark of `zenith/branding` (a Z lit by a four-point star), default theme colors from the palette (`docs/branding/palette.json`).
- **Packaging**: Windows x64 only: a per-user NSIS installer `Zenith-Launcher-Setup-x64.exe` and a portable `Zenith-Launcher-Portable-x64.exe` (`xmcl-electron-app/build/electron-builder.config.ts`). The Linux and macOS targets stay in the tree untouched and unbuilt.
- **Zenith instance** (`xmcl-electron-app/main/pluginZenith.ts`, `xmcl-runtime/zenith/instance.ts`): the Zenith bridge ships as an extra resource. On startup, once the data folder is known, the bridge shim is installed (and refreshed after updates) and, if no instance named
  `Zenith` exists and the marker `zenith-instance.json` is absent, the `Zenith` instance is created. Delete the marker to recreate it.

## Zenith Market

Page `Zenith Market` (sidebar, `/market`; `/store` redirects). Code: `xmcl-runtime/zenith/market/` (service `ZenithMarketService`, contract in `xmcl-runtime-api/src/services/ZenithMarketService.ts`), UI `Market.vue`, `MarketEntry.vue`, `MarketProject.vue`.

| Source | What | Notes |
|---|---|---|
| Zenith registry | `https://horizon-rust.github.io/horizon-mods/index.json` (schema 1, the same public index Horizon's launcher reads), cached 10 minutes in memory and in `%APPDATA%\zenith-launcher\zenith-market\registry-index.json` | kinds `mod` (`.hmod`), `native-mod` (`.hnmod`), `shader` (`.hshader`) and `shaderpack` (zip), `resourcepack`, `datapack`, `modpack` (`.hpack`); an unknown kind is kept in the index but never shown. https only (`ZENITH_MOD_INDEX` may point to a test index, https only). |
| Modrinth | `https://api.modrinth.com/v2` (no key) | resource packs, data packs (a `mod` project with the `datapack` category), shader packs; Java mods and modpacks only appear with "Show everything", marked Doesn't run. |

Badges, as in Horizon: **Works** (made for the Zenith game version), **Partly** (older resource packs, data packs, Java-format shader packs: the reason is shown), **Doesn't run** (Java mods, registry projects for another game version: never installable).
Native mods are listed with **Requires rebuild**: they are compiled into the game, so this launcher does not install them yet. Modpacks (`.hpack`) are listed and not installable yet.

Install (all checked before the file reaches the instance): https, known hosts (`cdn.modrinth.com`, the registry's release assets, a project's own GitHub release assets), a plain file name, the size, the **SHA-256** of the index (**SHA-512** on Modrinth), the index's
takedown list, and the author's **minisign** signature when the project has a key. Destinations in the Zenith instance folder: `horizon-mods/` (mods start turned off: enable them in the game, Options > Mods), `shaderpacks/`, `resourcepacks/`, and
`saves/<world>/datapacks/` (pick a world). Requests carry `User-Agent: Zenith-Launcher/<version> (github.com/gitzetazero/zenith)`; there are no proxies or XMCL endpoints. CurseForge is not used (Horizon's launcher does not offer it yet either).

## What XMCL embedded and what Zenith Launcher does about it

| What XMCL embedded | What Zenith Launcher does | What the maintainer must provide |
|---|---|---|
| Microsoft client ID `1363d629-...` | **Removed.** `ZENITH_MSA_CLIENT_ID` (environment), `zenith-config.json` (`msaClientId`) or the build (repository variable). Without it, Microsoft sign-in shows a message and never contacts Microsoft. | Their own Azure app, registered and approved by Mojang/Microsoft (personal accounts, public client, redirect `http://localhost:25555/auth`). |
| Application Insights keys | **Removed.** Telemetry is hard-off. | Nothing. |
| Hosted services `api/ai/signaling/core-api.xmcl.app`, `xmcl.blob.core.windows.net` | **Removed.** Nothing calls them; a guard (`xmcl-runtime/zenith/blockedHosts.ts`) refuses such requests anyway. News are off, flights are never downloaded, mod databases come from GitHub, ely.by uses its bundled list. | Nothing. |
| CurseForge API key | **No key.** `CURSEFORGE_API_KEY` / `curseforgeApiKey` (not used by the Market). | Optional. |
| Modrinth OAuth app and its code exchange at `api.xmcl.app` | **Removed.** Modrinth sign-in is unavailable; the Market uses Modrinth anonymously. | A Modrinth OAuth app **and** a service holding its secret, if sign-in is ever wanted (not planned). |
| LittleSkin OAuth app and its client secret | **Removed.** Optional `ZENITH_LITTLESKIN_CLIENT_ID`. | Optional. |
| Discord application `1075044884400054363` | **Removed.** `ZENITH_DISCORD_CLIENT_ID` (environment, `zenith-config.json` `discordClientId`, or the build variable). Without it the presence is disabled silently. | Create a Discord application named **Zenith** (<https://discord.com/developers/applications>), upload the logo as a Rich Presence art asset named `zenith-logo` (`docs/branding`, `zenith/branding/png/logo-tile-512.png`), and set the application id. |
| Updater (`api.xmcl.app/latest`, Voxelum releases) | **Replaced.** It reads `https://api.github.com/repos/gitzetazero/zenith/releases`, takes the newest published release that carries `Zenith-Launcher-Setup-x64.exe`, and links to `https://github.com/gitzetazero/zenith/releases` (no automatic download, no `.asar`). | Publish launcher releases in the Zenith repository. |
| Help links `xmcl.app/...` | Point to the Zenith repository. | Nothing. |

Account flows not tied to an XMCL registration (Yggdrasil, ely.by, LittleSkin with password, offline) are untouched in the code, but the Zenith bridge only accepts licensed Microsoft accounts.

### Optional configuration (`%APPDATA%\zenith-launcher\zenith-config.json`)

```json
{ "msaClientId": "", "curseforgeApiKey": "", "modrinthClientId": "", "littleskinClientId": "", "discordClientId": "" }
```

The environment wins over the file, and the file over the build.

## Syncing with XMCL

```bash
git fetch upstream --tags
git merge v0.74.0        # or whichever upstream tag is next
```

Many files were deleted (agent, peer, xmclAccount, bedrock, Java and instance-creation views): git will report modify/delete conflicts for upstream changes to them; keep them deleted. Other places where conflicts will happen:

- `README.md`, `xmcl-electron-app/package.json`, `xmcl-electron-app/build/electron-builder.config.ts`, `xmcl-electron-app/main/definedServices.ts` and `definedPlugins.ts`.
- `xmcl-keystone-ui/locales/*.yaml` (take upstream's side and re-apply the `XMCL` -> `Zenith Launcher` renames; rerun `node xmcl-keystone-ui/scripts/i18n.mjs lint` and remove the keys of the deleted features).
- `xmcl-runtime/instance/InstanceService.ts` (the guard call in `createInstance`), `user/pluginOfficialUserApi.ts`, `infra/plugins/*`, `presence/PresenceService.ts`, `app/LauncherApp.ts`, `setup/pluginSetup.ts`: search for `zenith`.
- `xmcl-keystone-ui`: `windows/main/{App.vue,Context.ts,router.ts}`, `views/{AppAddInstanceDialog,BaseSetting*,Setting*,Home*,AppSideBar*}.vue`, `composables/{launchButton,setting,theme}.ts`.
- The binary icons in `xmcl-electron-app/icons` and `logo.webp`.

After merging: `pnpm install`, `pnpm run check`, `pnpm vitest run xmcl-runtime/zenith`, and `node tools/zenith/verify-asar.mjs <app.asar>` of a build.

## Build and releases

- `.github/workflows/zenith-release.yml` (manual, `windows-latest`, third-party actions pinned by commit SHA, no secrets; only `GITHUB_TOKEN` with `contents: write` at the publish step): installs, downloads and verifies the bridge, typechecks, runs the unit tests of the touched code, builds with electron-builder,
  verifies `app.asar` (`tools/zenith/verify-asar.mjs`: no `xmcl.app`, `1363d629`, `XMCL Agent`, Together, signaling; `.minecraftz` and the product name present), writes `SHA256SUMS` and publishes a **normal release** (not a prerelease, marked latest).
- The bridge is verified by `tools/zenith/fetch-bridge.mjs`: the Ed25519 signature of `SHA256SUMS` from the `nightly` release of `gitzetazero/zenith` against Zenith's release key, then the exe's SHA-256 against that signed list. Any mismatch fails the build.
- XMCL's inherited workflows (`build.yml`, `deploy-*.yml`, `sign-release.yml`, `e2e.yml`, ...) stay in the tree but are **disabled** in this repository (they need secrets or publish XMCL things) and are not imported into the Zenith repository.
- The exes are **not code-signed** (SmartScreen will warn).

## Limits

- Not verified on third-party PCs; builds and unit tests of the touched code run in CI and on a development PC. Native mods and `.hpack` modpacks cannot be installed from the Market yet.
- Microsoft sign-in does not work until an approved Azure app exists and is configured.
- Updates: notice and link to Releases only.
