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
| Where it lives | this fork, and from the next import `launcher/` of `gitzetazero/zenith` ([import instructions](../tools/zenith/import-instructions.md)) |

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
  Kept: window size, fullscreen, monitor, shared `options.txt`, game language sync, quick launch, quick play of a server or a world (what `bridge/src/args.rs` forwards).
- **Removed features**: the XMCL AI Agent (service, tools, settings, pages), Minecraft Together / P2P multiplayer (WebRTC peers, ICE/signaling, `PeerService`, friends), the XMCL account service, Bedrock, the Java mods and blueprints pages,
  modpack import/export/publish, remote Java server hosting, the Modrinth publish dialogs, the community translation service, remote flights, and `node-datachannel`. Telemetry is **hard-off**: no exporter, no key, no setting (`xmcl-runtime/infra/plugins/telemetry.ts` only answers the renderer's IPC with no-ops;
  `xmcl-keystone-ui/src/telemetry.ts` starts disabled and nothing enables it).
- **Zenith Market** (`/market`, replaces the modpack store): see below.
- **Discord presence**: "Playing Zenith", the `zenith-logo` asset and a button to the Zenith repository (`xmcl-runtime/zenith/presence.ts`). Disabled silently without an application id.
- **About page** (`SettingAbout.vue`): author and maintainer `gitzetazero`, the Zenith repository, version, MIT licence, one credits paragraph (XMCL by Voxelum / ci010, MIT; the engine is a fork of Horizon).
- **Identity**: product name `Zenith Launcher`, app id `dev.zenith.launcher`, deep-link scheme `zenith-launcher://`, AppUserModelId `dev.zenith.launcher`, window titles, tray, the brand mark of `branding` (a Z lit by a four-point star), default theme colors from the palette (`docs/branding/palette.json`).
- **Packaging**: Windows x64 only: a per-user NSIS installer `Zenith-Launcher-Setup-x64.exe` and a portable folder zip `Zenith-Launcher-Portable-x64.zip` (the unpacked app, made by the release workflow; `xmcl-electron-app/build/electron-builder.config.ts`). Each release stamps `<XMCL base>-zenith0.1.<run>` into `xmcl-electron-app/package.json` (`tools/zenith/stamp-version.mjs`), so Settings > About names the build rather than XMCL's unchanged version. The Linux and macOS targets stay in the tree untouched and unbuilt.
- **Zenith instance** (`xmcl-electron-app/main/pluginZenith.ts`, `xmcl-runtime/zenith/instance.ts`): the Zenith bridge ships as an extra resource. On startup, once the data folder is known, the bridge shim is installed (and refreshed after updates) and, if no instance named
  `Zenith` exists and the marker `zenith-instance.json` is absent, the `Zenith` instance is created. Delete the marker to recreate it.

## Zenith Market

Page `Zenith Market` (sidebar, `/market`; `/store` redirects). Code: `xmcl-runtime/zenith/market/` (service `ZenithMarketService`, contract in `xmcl-runtime-api/src/services/ZenithMarketService.ts`), UI `Market.vue`, `MarketEntry.vue`, `MarketProject.vue`.

| Source | What | Notes |
|---|---|---|
| Zenith registry | `https://horizon-rust.github.io/horizon-mods/index.json` (schema 1, the same public index Horizon's launcher reads), cached 10 minutes in memory and in `%APPDATA%\zenith-launcher\zenith-market\registry-index.json` | kinds `mod` (`.hmod`), `native-mod` (`.hnmod`), `shader` (`.hshader`) and `shaderpack` (zip), `resourcepack`, `datapack`, `modpack` (`.hpack`); an unknown kind is kept in the index but never shown. https only (`ZENITH_MOD_INDEX` may point to a test index, https only). |
| Modrinth | `https://api.modrinth.com/v2` (no key) | resource packs, data packs (a `mod` project with the `datapack` category), shader packs; Java mods and modpacks only appear with "Show everything", marked Doesn't run. |

Badges, as in Horizon: **Works** (made for the Zenith game version), **Partly** (older resource packs, data packs, Java-format shader packs: the reason is shown), **Doesn't run** (Java mods, registry projects for another game version: never installable).
Native mods are listed with **Requires rebuild** and install into `horizon-native-mods/`, where the game lists them and says that the game it runs was not built with them: this launcher places the file, it does not compile native code. Modpacks (`.hpack`) are listed and not installable yet.

Install (all checked before the file reaches the instance): https, known hosts (`cdn.modrinth.com`, the registry's release assets, a project's own GitHub release assets), a plain file name, the size, the **SHA-256** of the index (**SHA-512** on Modrinth), the index's
takedown list, and the author's **minisign** signature when the project has a key. Destinations in the Zenith instance folder: `horizon-mods/` (mods start turned off: enable them in the game, Options > Mods), `shaderpacks/`, `resourcepacks/`, and
`saves/<world>/datapacks/` (pick a world). Requests carry `User-Agent: Zenith-Launcher/<version> (github.com/gitzetazero/zenith)`; there are no proxies or XMCL endpoints. CurseForge is not used (Horizon's launcher does not offer it yet either).

## What XMCL embedded and what Zenith Launcher does about it

| What XMCL embedded | What Zenith Launcher does | What the maintainer must provide |
|---|---|---|
| Microsoft client ID `1363d629-...` | **Replaced by the public Minecraft client id `00000000402b5328`**, built into the launcher (no repository variable needed). It is a Live Connect v1 application, so the launcher uses the v1 authorization-code flow (see "Microsoft sign-in" below). `ZENITH_MSA_CLIENT_ID` / `zenith-config.json` can still override it. | Nothing: the id is public. |
| Application Insights keys | **Removed.** Telemetry is hard-off. | Nothing. |
| Hosted services `api/ai/signaling/core-api.xmcl.app`, `xmcl.blob.core.windows.net` | **Removed.** Nothing calls them; a guard (`xmcl-runtime/zenith/blockedHosts.ts`) refuses such requests anyway. News are off, flights are never downloaded, mod databases come from GitHub, ely.by uses its bundled list. | Nothing. |
| CurseForge API key | **No key.** `CURSEFORGE_API_KEY` / `curseforgeApiKey` (not used by the Market). | Optional. |
| Modrinth OAuth app and its code exchange at `api.xmcl.app` | **Removed.** Modrinth sign-in is unavailable; the Market uses Modrinth anonymously. | A Modrinth OAuth app **and** a service holding its secret, if sign-in is ever wanted (not planned). |
| LittleSkin OAuth app and its client secret | **Removed.** Optional `ZENITH_LITTLESKIN_CLIENT_ID`. | Optional. |
| Discord application `1075044884400054363` | **Removed.** `ZENITH_DISCORD_CLIENT_ID` (environment, `zenith-config.json` `discordClientId`, or the build variable). Without it the presence is disabled silently. | Create a Discord application named **Zenith** (<https://discord.com/developers/applications>), upload the logo as a Rich Presence art asset named `zenith-logo` (`docs/branding`, `branding/png/logo-tile-512.png`), and set the application id. |
| Updater (`api.xmcl.app/latest`, Voxelum releases) | **Replaced.** It reads `https://api.github.com/repos/gitzetazero/zenith/releases`, takes the newest published release that carries `Zenith-Launcher-Setup-x64.exe`, and links to `https://github.com/gitzetazero/zenith/releases` (no automatic download, no `.asar`). | Publish launcher releases in the Zenith repository. See "Launcher updates" below. |
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
- The exes are **not code-signed** (SmartScreen will warn). The portable download used to be a self-extracting exe that Defender quarantined as `Trojan:Script/Wacatac.H!ml` (a machine-learning false positive); it is now a plain folder zip, and the installer scans clean. The release notes carry the SHA-256 of both files.

## Limits

- Not verified on third-party PCs; builds and unit tests of the touched code run in CI and on a development PC. `.hpack` modpacks cannot be installed from the Market yet, and native mods install as files only: running one needs a game built with it.
- Microsoft sign-in uses the public Minecraft client id, shared with other launchers: Mojang must accept it for this application, and Microsoft could retire it.
- Updates: installed Windows builds self-update (see "Launcher updates"); portable, development and non-Windows builds link to Releases.

### Launcher updates

Releases are dated (`nightly-<date>`), so "is there an update" compares the build's own commit (`ZENITH_COMMIT_BUILD`) with the "Zenith build of commit <sha>" line of the release notes (`xmcl-runtime/zenith/release.ts`). The same commit never updates, and a build or release without a known commit never self-updates (no loops).

- **Installed Windows build (NSIS):** the launcher downloads `Zenith-Launcher-Setup-x64.exe` from the release, to a temp folder, only from `https://github.com/gitzetazero/zenith/releases/download/...` (any other host is refused). It then checks the file's SHA-256 against the `<hash>  <filename>` line in the release notes. No hash, a mismatch or a wrong host means the installer is never run: the update shows an error and Releases stays one click away. "Download updates automatically" (Settings > Update, the existing `autoDownload` setting, on by default) does this in the background, once per launcher session; with it off, the download starts from the update dialog. The restart is always the user's click (`Restart to Install`): the hash is checked again, then the installer is started detached with `/S --updated --force-run` (silent, per user, relaunches the launcher) and the launcher quits. The restart is refused while a game process is running.
- **Portable build** (the folder zip: no `Uninstall *.exe` next to the program, or `PORTABLE_EXECUTABLE_FILE` set), development builds, AppX and non-Windows: the old behaviour. The dialog says the build does not update itself and opens the Releases page.
- **Security model and limits:** HTTPS from GitHub releases of `gitzetazero/zenith` plus the SHA-256 from the notes of the same release. The hash catches a corrupt or truncated download, not a replaced release (whoever can edit the release can edit both). Installers are not code-signed for Windows, so SmartScreen may warn. The game itself is updated separately by the bridge, with its Ed25519 signature.
- **Tested:** unit tests of the pure decisions (`xmcl-runtime/zenith/updateVerify.test.ts`) and typecheck. The download, hash check and silent install on a real installed build were not exercised.

## Microsoft sign-in: the public Minecraft client id

Zenith Launcher signs in with Microsoft's **public Minecraft client id**
`00000000402b5328`, the historical "Sign in to Minecraft" application. It is
not a secret and every client embeds it in the authorization URL. It is **not**
a Microsoft Entra application, so MSAL and the v2 endpoints
(`login.microsoftonline.com/consumers/oauth2/v2.0/*`) cannot use it: Microsoft
answers `unauthorized_client` / `AADSTS700016`. The launcher therefore uses the
Live Connect v1 flow:

1. **Authorization code**:
   `https://login.live.com/oauth20_authorize.srf` with
   `client_id=00000000402b5328`,
   `scope=service::user.auth.xboxlive.com::MBI_SSL`, `response_type=code` and
   `redirect_uri=https://login.live.com/oauth20_desktop.srf`. The login window
   intercepts that Microsoft-hosted redirect and reads `code`; a localhost
   redirect is not registered for this id and is rejected by Microsoft.
2. **Token**: `POST https://login.live.com/oauth20_token.srf` with
   `grant_type=authorization_code`, the code, the same redirect URI and scope.
   The result is a Microsoft access token and a refresh token.
3. **Refresh**: the same token endpoint with `grant_type=refresh_token`. The
   refresh token, account key and expiry live in the OS secret storage under
   `xmcl-oauth-legacy`; they never reach logs, `accounts.json` or the CLI.
4. **Xbox Live, XSTS and Minecraft Services** are unchanged:
   `RpsTicket=d=<Microsoft access token>`, `rp://api.minecraftservices.com/`,
   `login_with_xbox`, `/minecraft/profile`, `/entitlements/mcstore`.

The device-code flow is not offered for this id: Live Connect has no device
endpoint. The browser window is the only interactive route. The built-in id can
still be overridden with `ZENITH_MSA_CLIENT_ID` (environment) or
`zenith-config.json` (`msaClientId`); an override that is an Entra application
keeps using the MSAL/v2 flow.

### Why not `00000000-402b-4cd3-a82b-c45ab2f1d3f7`?

That is the id used by the new Microsoft Store launcher, but Microsoft has
decommissioned it for third-party clients: on 2026-11-06 the v2 authorize and
device-code endpoints answer `unauthorized_client` / `AADSTS700016`, and the v1
endpoints answer `invalid_client`, all with "The client does not exist or is not
enabled for consumers". No flow adaptation can fix that: the failure happens
before an authorization request is made. The public id that still works is the
legacy `00000000402b5328`.

### Limits

- The id is shared with other launchers. Microsoft has tolerated it for years
  but can revoke it at any time; only the maintainer can register a replacement.
- Accounts signed in with the previous Azure app need one fresh sign-in: their
  refresh tokens belong to a different client id and are not reused.
- Mojang's `login_with_xbox` must recognise the application. The legacy id is
  the Minecraft application id, so it is the one the ecosystem uses, but a real
  end-to-end sign-in needs a real Microsoft account and cannot be verified in
  CI. If Mojang rejects it, the launcher shows the 403 ("Invalid app
  registration") from Minecraft Services; approval is Mojang's, through
  <https://aka.ms/AppRegInfo>, and is not something this repository can grant.
- Live Connect v1 is legacy. If Microsoft retires it, this flow stops working.
- The Microsoft token is not a Minecraft token; the Xbox/Minecraft steps and
  their owner gate are unchanged.
