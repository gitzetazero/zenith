# Throwaway test with XMCL (static form)

Task 5 of the phases 1 to 3 plan. XMCL's interface was not driven: nobody could click on the machine and there was no approval to control the screen. What can be done without it was done: reading XMCL's code, running its argument generator and its way of launching processes against the real JSON of version 26.3, and testing Zenith's real nightly. Everything that really needs the interface is in the section "What could NOT be verified".

- XMCL: `Voxelum/x-minecraft-launcher`, commit `aeb5240` (main branch on 2026-11-05), XMCL 0.73.1, `@xmcl/core` 2.16.3.
- Zenith: `nightly` Release of `gitzetazero/zenith` ("Nightly 2026-11-05 · b933efa").
- Everything downloaded and generated lives outside the repository, in `%TEMP%\zenith-spike`.
- Fixture: `bridge/tests/fixtures/xmcl-args.txt`.

## Summary of what matters

1. **Blocking for the plan.** On Windows, with a prepend command XMCL launches with `shell: true`, that is, through `cmd.exe`, whose limit is 8191 characters. The Minecraft 26.3 command is about 6900 characters plus 82 times the length of XMCL's data root. With the default root (`%APPDATA%\xmcl`, about 35 to 40 characters) the command is about 9500 to 10 000 characters and **`cmd.exe` rejects it with "The command line is too long" without ever running the bridge**. It only works with a root of about 19 characters or less (for example `C:\xmcl`). Measured, not assumed.
2. **The bridge must be a Windows-subsystem executable** (`#![windows_subsystem = "windows"]`). With `shell: true` and `detached: true`, a console executable loses its standard output: XMCL does not receive it and never sees `LWJGL Version: ` or `Crash report saved to:`. Measured.
3. **Two assumptions of task 6 are false for 26.3.** There is no `--userType`, and `--width/--height` do not appear with `-1`. Also `--clientId` and `--xuid` arrive as literal text, unsubstituted (`${clientid}`, `${auth_xuid}`).

## 1. The exact arguments XMCL passes

### How they are generated

`xmcl-runtime/launch/LaunchService.ts`, `#generateOptions()` (lines 192 to 290) builds the options and `launch()` calls `launch(op)` of `@xmcl/core`, which in turn uses `generateArguments` (`packages/core/launch.ts:770`). What XMCL passes:

| `@xmcl/core` option | Value |
| --- | --- |
| `gamePath` | the instance folder, `<root>/instances/<name>` (it is also the `cwd`) |
| `resourcePath` | XMCL's data root (`getPath()`) |
| `javaPath` | `javaw.exe` next to the chosen Java, if it exists and can be executed; otherwise `java.exe` |
| `gameProfile` | the account's `{ name, id }` |
| `accessToken` | the account's token (if there is none, a random hexadecimal) |
| `userType`, `properties`, `features` | **not passed**: `userType` stays `msa` but the 26.3 template does not use it; `features` stays empty |
| `launcherName`, `launcherBrand` | `XMCL (0.73.1)` |
| `extraJVMArgs`, `extraMCArgs` | the instance's or the global `vmOptions` and `mcOptions` |
| `server` | `{ ip, port }` if the instance has `server` |
| `quickPlayMultiplayer` | `host[:port]` if there is `server` |
| `quickPlaySingleplayer` | `options.world` if requested |
| `resolution` | `inst.resolution ?? globalResolution` |
| `useHashAssetsIndex` | `true` |
| `prependCommand` | the string, split with `normalizeCommandLine` |
| `extraExecOption` | `{ shell: prepend && prepend.length > 0, detached: true, cwd: gamePath, env }` |
| `prechecks` | `[]` (`@xmcl/core`'s own is not used; the checks are XMCL middlewares) |

### How it was run

`@xmcl/core` was bundled with esbuild (with an empty stand-in for `@xmcl/unzip`, which is only used to extract natives) and `generateArguments` and `launch` were called with the same options. To see what the bridge really receives, a test executable was compiled (`zenith-bridge.exe` in `…\Jose Ñ\bridge dir\`, a path with spaces and an Ñ) that dumps `GetCommandLineW` and `std::env::args()` to a file. The version JSON was a copy of the real 26.3 (read only) and the paths pointed to temporary folders.

Main command:

```
node run.cjs        # generateArguments in 10 scenarios
node real2.cjs      # the real core.launch() (spawn with shell: true, detached: true) against the dump executable
```

### argv layout

With a prepend command, the bridge's `argv` is: the bridge's path, `javaw.exe`, the JVM arguments, the main class, the game arguments, extras. The prepend command goes **first and does not replace Java**: Java becomes the first argument. `javaw.exe` is substituted when it exists (XMCL always does it on Windows if the file is there). So the plan is right that `argv[0]` (after removing the bridge's name) is Java, but it is `javaw.exe`.

The fixture is exactly what the bridge receives, without its own path (43 arguments, one per line). Order:

```
 1  <root>\jre\java-runtime-epsilon\bin\javaw.exe          (backslashes)
 2  -XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump
 3  -XX:StackShadowPages=32
 4  --enable-native-access=ALL-UNNAMED
 5  --add-exports
 6  java.base/jdk.internal.misc=ALL-UNNAMED
 7  -Djava.library.path=<root>/versions/26.3/26.3-natives/java          (forward slashes)
 8  -Djna.tmpdir=…/26.3-natives/jna
 9  -Dorg.lwjgl.system.SharedLibraryExtractPath=…/26.3-natives/lwjgl
10  -Dio.netty.native.workdir=…/26.3-natives/netty
11  -Dminecraft.launcher.brand=XMCL (0.73.1)            (a single argument, with a space)
12  -Dminecraft.launcher.version=XMCL (0.73.1)
13  -cp
14  <82 jars separated by ';'>…;<root>/versions/26.3/26.3.jar
15  -DlibraryDirectory=<root>/libraries
16-22 -Xmx2G -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20
    -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=32M     (DEFAULT_EXTRA_JVM_ARGS)
23  net.minecraft.client.main.Main
24-25  --username <name>
26-27  --version 26.3
28-29  --gameDir <root>/instances/<instance>
30-31  --assetsDir <root>/assets
32-33  --assetIndex e787e8afd149a4cefa89d341b91f5aed8f5d93a0   (the index's sha1, not "34")
34-35  --uuid <uuid without dashes>
36-37  --accessToken <token>
38-39  --clientId ${clientid}                  <- literal text
40-41  --xuid ${auth_xuid}                     <- literal text
42-43  --versionType release
```

Details measured with the 26.3 JSON:

- **There is no `--userType`.** The 26.3 `arguments.game` template does not contain it. The task 6 assumption ("reject if `--userType` is not `msa`") cannot be met. The only thing that tells an account without ownership apart is the result of asking Minecraft's services. For ely.by and LittleSkin XMCL adds `-javaagent:<jar>=<server>` and `-Dauthlibinjector.side=client` to the JVM arguments (code in `#generateOptions` and `generateArguments`), which does allow rejecting them before the network.
- **`--clientId ${clientid}` and `--xuid ${auth_xuid}`** come out literal, because XMCL does not define those variables and `format()` leaves the key when there is no value. The bridge must treat any value that starts with `${` as absent (xuid `None`).
- **`--width` and `--height` do not exist by default.** The template conditions them on the `has_custom_resolution` feature, which XMCL does not enable, and XMCL only adds them at the end when there is a `resolution`. No `-1` is seen. Measured:
  - without a resolution: no `--width` or `--height`;
  - `resolution: {}` (empty object): neither;
  - `{ width: 854, height: 480 }`: at the end `--height 480 --width 854` (**height first**);
  - `{ fullscreen: true }`: at the end `--fullscreen`.
- **Server.** With `server` on the instance the two come out together and in this order, after the others: `--quickPlayMultiplayer host[:port]`, `--server host` and `--port N` (no `--port` if there is no port). A world gives `--quickPlaySingleplayer <folder>` (with spaces in one piece: `New World`). `--quickPlayPath` is not used.
- **Extras.** `mcOptions` go after `--versionType release`, before quickPlay/server/resolution.
- `--demo` only if there is no account.
- **Paths.** A mix of separators: `javaw.exe` comes with `\`; everything else (`--gameDir`, `--assetsDir`, `-cp`, natives) with `/`. The process `cwd` is `--gameDir`.

### Minecraft root and layout on disk

`--assetsDir` is always `<root>/assets`, so **the root is the parent of `--assetsDir`**. Under that root XMCL uses the classic `MinecraftFolder` layout: `versions/26.3/26.3.json`, `versions/26.3/26.3.jar` (the last element of `-cp`), `libraries/…` (Maven) and `assets/indexes`, `assets/objects`. The game lives in `<root>/instances/<name>` (`--gameDir`), not in the root. It is the same layout that Horizon's launcher uses (`zenith-prepare --minecraft-dir` accepts it as it is, and the preparation below was done with it).

Two XMCL details that affect the bridge:

- `--assetIndex` is the index's sha1. XMCL installs `assets/indexes/<sha1>.json` and creates an alias `assets/indexes/34.json` with a `link` operation (`packages/installer/assets.ts:104`). The alias could not be checked on disk without installing the game: if it did not exist, `--minecraft-dir` would not find the index and `--assets-dir` would have to be passed with the index by its name.
- Every instance carries junctions (`libraries/`, `versions/`) to the root, created by `pluginLaunchPrecheck.ts`. They do not change what the bridge receives.

### Measured variants

With the same experiment, only the ends change. The last part of argv in each case:

```
Server + port:       … --versionType release --quickPlayMultiplayer play.example.invalid:25566 --server play.example.invalid --port 25566
Server without port: … --versionType release --quickPlayMultiplayer play.example.invalid --server play.example.invalid
World:               … --versionType release --quickPlaySingleplayer New World
Resolution:          … --versionType release --height 480 --width 854
Fullscreen:          … --versionType release --fullscreen
Extras:              … --versionType release --foo "bar baz"
```

## 2. Prepend command: quoting, splitting and execution

### Splitting (`xmcl-runtime/launch/utils/cmd.ts`)

`splitCommandLine` uses `/[^\s"']+|"([^"]*)"|'([^']*)'/g`. Measured rules:

| Field value | Result |
| --- | --- |
| `"C:\path with spaces\zenith-bridge.exe"` (double quotes) | a single piece, without quotes. Works. |
| `'C:\path with spaces\zenith-bridge.exe'` (single quotes) | the same |
| `C:\Users\Jose Ñ\bridge dir\zenith-bridge.exe` (no quotes) | **split into 3 pieces** (`…\Jose`, `Ñ\bridge`, `dir\zenith-bridge.exe`) and fails: `""C:\…\Jose"" is not recognized as an internal or external command` |
| empty quotes `""` | not treated as an empty string (the `match[1]` branch is false and the text `""` is added) |

The Ñ and other non-ASCII characters pass intact both ways (checked with the dump: the bridge receives `Jose Ñ` correctly).

(Superseded: see section 10) **For `setup` (task 10): `prependCommand` must carry the bridge's path in double quotes.** The bridge's own arguments can be added after it.

### Execution (`@xmcl/core` `launch()`)

With a non-empty `prependCommand`, `extraExecOption.shell` is `true`. `launch()` then wraps **each** argument in double quotes (`args.map(a => `"${a}"`)`) and calls `spawn(args[0], args.slice(1), { shell: true, detached: true, cwd, env })`. On Windows Node runs `cmd.exe /d /s /c "<everything together>"`. Measured consequences:

- **`cmd.exe`'s 8191-character limit.** This is the most serious. The real `launch()` was run with the 26.3 command, varying only the root's length (bridge path of 70 characters):

  | Root length | Command line | Result |
  | --- | --- | --- |
  | 5 (`C:\zx`) | 6960 | the bridge runs |
  | 19 | 8122 | the bridge runs |
  | 20 | 8204 | **"The command line is too long."**, the bridge does not start, code 1 |
  | 22 | 8370 | the same |
  | 36 (a normal temporary folder) | ~11 000 | the same |

  The line grows 82 characters for each root character (82 appearances of the root in the libraries and natives path) and 1 for each character of the bridge path. XMCL's default root (`%APPDATA%\xmcl`) is about 35 to 40 characters, so it **fails**. Without a prepend command there is no `shell` and no such limit (the `CreateProcess` limit is 32 767). No measure against this was found in XMCL (`grep` for "too long", "8191", "E2BIG": nothing).
- Inside `cmd.exe`, **`%VAR%` is expanded**: an argument `100%ZXTEST%` arrived as `100EXPANDED`. A `"` inside an argument is lost (`q"uote` arrived as `quote`). `&` between quotes passes fine. An instance name or path with `%…%` or `"` is corrupted before it reaches the bridge.
- **`process.pid` is `cmd.exe`'s, not the bridge's.** A non-forced `kill()` (`process.kill()`) only ends `cmd.exe` and leaves the bridge and the game alive. `kill(…, force)` does use `taskkill /pid … /f /t` (the whole tree) (`LaunchService.ts:733`). The behaviour of the interface button depends on which one is used (not verified).
- `detached: true` plus `shell: true` make `cmd.exe` be created without a console.

### The bridge's standard output (needs the Windows subsystem)

Measured with `child_process.spawn` and the same options as XMCL (`shell: true`, `detached: true`):

| Executable | stdout the parent sees |
| --- | --- |
| console, `shell` + `detached` (**XMCL's case**) | `""` (empty, **lost**) |
| `windows` subsystem, `shell` + `detached` | `LWJGL Version: zenith` |
| console, `shell` without `detached` | `LWJGL Version: zenith` |
| console, without `shell`, with `detached` | `LWJGL Version: zenith` |

That is: with the combination XMCL uses, a console bridge cannot talk to XMCL. The probable cause is that `cmd.exe` has no console (`DETACHED_PROCESS`), a child console program gets a new console and its output goes there, not to the pipe (an inference; the window was not observed). With `#![windows_subsystem = "windows"]` the output arrives. **Decision:** the bridge uses `#![windows_subsystem = "windows"]`, launches `zenith-client` with `CREATE_NO_WINDOW` and pipes, and forwards stdout and stderr. The preparation in its own console (`CREATE_NEW_CONSOLE`, task 9) does not have this problem because it is a separate process with its own console.

## 3. Instance: where it lives and what it needs

- **Data root:** `%APPDATA%\xmcl\root` is a text file with the path (`xmcl-runtime/app/LauncherApp.ts:348`). If it does not exist, it is the first start and the interface asks for the path (`bootstrap`). If reading fails, it uses `%APPDATA%\xmcl`. `LAUNCHER_NAME` is `xmcl`.
- **Managed instances:** `<root>/instances/<name>/instance.json`. On start, `InstanceService` reads the whole `instances/` folder (`readdirEnsured`) and also `%APPDATA%\xmcl\instances.json` (list of external instances, groups and selection). A new folder with `instance.json` is detected **only when XMCL starts** (they are loaded once at startup); XMCL must be restarted. If the `name` field does not match the folder name, XMCL **renames the folder** to `filenamify(name)` (`InstanceService.ts`, function `loadInstance`).
- **Schema** (`packages/instance/instance.ts`, zod 4): all fields have a default or are optional. `InstanceSchema.parse` and `createInstance` (bundled with esbuild) were run with the minimal JSON `{ "name": "Zenith", "runtime": { "minecraft": "26.3" }, "prependCommand": "\"C:\\path\\zenith-bridge.exe\"" }` and it was accepted, filling in `edition: "java"`, the rest of `runtime` empty, `version: ""`, dates at 0, etc. If `runtime.minecraft` is missing, XMCL puts the latest version. Which fits the plan: `name`, `runtime.minecraft` and `prependCommand` are enough.
- **Per-instance `prependCommand`:** the `prependCommand` field (string, optional) of `instance.json`; it is edited in the instance's Java setting (`BaseSettingJava.vue:157`). `inst.prependCommand ?? globalPrependCommand` is used (`xmcl-runtime-api/src/util/launch.ts:140`); the global one (`globalPrependCommand`, default `''`) applies to every instance that does not define its own.
- **Java:** `java` is optional in the schema; if missing, the interface chooses or installs the Java that the version JSON asks for (`javaVersion: java-runtime-epsilon`, 25). This could not be checked.
- **What XMCL downloads and the bridge needs:** everything listed before, under `<root>/versions`, `libraries`, `assets`, plus `jre/<component>/bin` for the managed Java.

## 4. Detecting "started" and failures (`createMinecraftProcessWatcher`, `packages/core/launch.ts:524`)

Only **stdout** is watched (stderr takes no part in detection). For each chunk of output, in this `else if` order:

1. Contains `---- Minecraft Crash Report ----`: saves the chunk as a report.
2. Contains `Crash report saved to:`: the path is what follows `Crash report saved to: #@!@# ` (exactly `len("Crash report saved to: #@!@# ")` characters are skipped from the start of the marker). Then `LaunchService` cuts at the last `.txt`.
3. Contains `Crash report saved to ` (no colon): the path is what follows.
4. If not yet ready and it contains any of `Missing metadata in pack`, `Registering resource reload listener`, `Reloading ResourceManager`, `LWJGL Version: `, `OpenAL initialized.`: emits `minecraft-window-ready` (only once).
5. If not yet ready and it contains ` Preparing level `: the same.
6. Contains `Failed to start the minecraft server`: saves the chunk as a report.

Exit: the process's `exit`. It is marked as a failure if the code is not 0, or there is a report, or there is a report path. If there is a report and the code is 0, it changes it to 1. **The checks are per output chunk and by substring**; the normal thing is to print each line in a separate write. The bridge's exact text must be `Crash report saved to: #@!@# <path>` (the form without `#@!@#` is also accepted in branch 3, but branch 2 wins if there is a colon).

stderr accumulates in `errorLogs` and is emitted as `minecraft-stderr`; it is only shown in the failure dialog and in the log. None of the above could be checked in the interface (what it shows with code 1, etc.).

Risks for a program that is not Java:

- **There is no Java check on the prepend command.** The `java-validation` middleware (`pluginLaunchPrecheck.ts`) validates the instance's Java (`input.java`), not the prepend command. If the Java is valid, the bridge does not get in the way. It was checked that `launch()` uses `prechecks: []`.
- `process.unref()` is only so XMCL can exit without waiting; it changes nothing for the bridge.
- After `spawn`, if `process.pid` is not a number, XMCL waits 1 s for the `error` event. With `cmd.exe` there is always a `pid`, so a bridge that does not exist or fails **does not produce an "invalid Java" error**: it shows up as an exit with code 1 and `cmd.exe`'s message on stderr.
- XMCL decodes the output by guessing the encoding (`guessEncodingByBuffer`): the bridge should write UTF-8 and complete lines.
- `stdin` stays open as a pipe (XMCL never writes or closes it); the bridge must not read from it.
- There is a language middleware (`sync-language`) that may write `options.txt`, only if the user turns it on.

## 5. The nightly's Launch API

Download (`gh release download nightly -R gitzetazero/zenith`), signature check with the key in `bridge/zenith-release-keys.txt`, and unpacking into its own folder:

```
$ cargo run -q -p zenith-release-sign -- verify --tag nightly --key "<key>" <folder with SHA256SUMS, SHA256SUMS.sig, zip and tar.gz>
nightly: 2 files match SHA256SUMS            (exit code 0)
```

(The Linux `.tar.gz` must be there too: `SHA256SUMS` lists it and verification fails if it is missing; it is not optional.) The zip weighs 36.9 MB, 100 MB unpacked, and **it has a root folder `horizon-windows-x64/`** with `zenith-client.exe`, `zenith-prepare.exe`, `zenith-ctl.exe`, `research-lock.json`, `exporters/`, `mods/`, `tools/`, `crates/`, `LICENSE` and `PLAY.txt`.

`zenith-client.exe --launch-api` (code 0):

```json
{ "crash_report_line": "Horizon: crash report saved to ", "exit_codes": [0,1,2,101],
  "horizon": "Nightly 2026-11-05 · b933efa", "launch_api": "0.1.0", "minecraft": "26.3",
  "options": [ "--resources DIR", "--game-dir DIR", "--home DIR", "--player-dir DIR", "--width N", "--height N",
               "--fullscreen", "--session-file PATH", "--session-stdin", "--quickPlaySingleplayer WORLD",
               "--quickPlayMultiplayer HOST[:PORT]", "--launch-api" ] }
```

`zenith-prepare.exe --launch-api` (code 0):

```json
{ "launch_api": "0.1.0", "horizon": "nightly-2026-11-05", "minecraft": "26.3", "java_major": 25,
  "schema": 7, "output_version": 3,
  "options": [ "--prepare", "--server-jar PATH", "--minecraft-dir PATH", "--version-json PATH", "--client-jar PATH",
               "--libraries-dir PATH", "--assets-dir PATH", "--java-home PATH", "--root PATH", "--output PATH",
               "--check", "--complete", "--progress json", "--launch-api" ],
  "exit_codes": [0,1,2] }
```

Notes: the two `horizon` fields have a different format (`Nightly 2026-11-05 · b933efa` versus `nightly-2026-11-05`), so they cannot be compared with each other. `--root PATH` exists in `zenith-prepare` and is not in the guide; see the note about `.local` below.

## 6. Preparation and smoke test

### Preparation (read only on the Horizon launcher's Minecraft folder)

```
zenith-prepare.exe --prepare --server-jar <…>\server-jars\26.3.jar \
    --minecraft-dir <…>\horizon-launcher\minecraft --java-home "C:\Program Files\Eclipse Adoptium\jdk-25.0.5.8-hotspot" \
    --output %TEMP%\zenith-spike\prepared --complete --progress json
```

- Exit code 0 in 31 s (the program says "Done in 26.6 s"). 72 JSON lines on stdout, stderr empty, no errors. It ended with `{"fraction": 1.000}`.
- Relevant messages: it imported the 26.3 client from the Minecraft folder, "Checked the game files in 1.4 s", exported block states, compiled the exporters (client and server, about 3 s each), copied assets and sounds (4962 files, 377.3 MB).
- **It downloaded nothing**: the preparation only asked for local files (checked from the output; the network was not measured with another tool).
- Size of the prepared folder: **595.3 MB, 28 178 files**. Also:
  - the client's first start added `horizon-data.bin` (120 MB) inside that folder (see below), so the folder must be writable and ends up at about 715 MB;
  - the preparation wrote **398.7 MB (1947 files) in `<build>\.local\minecraft\26.3`**, inside the unpacked build's folder, not in `--output`. `--root PATH` already exists to move this; it was not tested. If the bridge keeps builds in a folder that must not be modified, `--root` has to be passed.
- `zenith-prepare.exe --check --output …\prepared`: code 0, no output.

### Start (smoke)

Variables `HORIZON_HOME`, `HORIZON_NO_AUDIO=1`, `HORIZON_NO_HEARTBEAT=1`, `HORIZON_TELEMETRY_URL=off`; arguments `--resources <prepared> --game-dir <game> --home <home> --width 640 --height 360 --session-stdin` and the fake session `{"name":"ZenithTest","uuid":"…","access_token":"FAKE","user_type":"msa"}` on standard input.

- Still alive at 25 s. **Empty stdout**; everything goes to stderr, as text, starting with `Horizon Nightly 2026-11-05 · b933efa`.
- `logs/latest.log` (in the `--home` folder): Vulkan renderer on the discrete GPU, `HDR output HDR10 (PQ)`, `core-ready after 876 ms`, `loading screen shown after 1739 ms`, `world renderer ready after 1935 ms`, `first frame after 83 ms`, `resources loaded in 4.9s; 35723 block states, 1658 items, 67 biomes`, `rest-ready after 4866 ms`. The only error line: `Failed to retrieve profile key pair: InvalidSession`, expected with a fake token.
- **There is no line that says "title screen"**: it reached "loading screen shown", "world renderer ready" and "rest-ready", and the process stayed alive. The window could not be seen (no screen control). It is consistent with having reached the title, not a proof.
- At 17 s it wrote `horizon-data.bin` (120 MB) in the prepared folder (`wrote horizon-data.bin in 17451 ms`, thread `Prepared data file`). Ending the process before that point, or leaving the folder read-only, has an untested effect.
- It was ended by force (`taskkill /T /F`); the exit code the interpreter collected (143) is not the client's. The behaviour when the window is closed was not tested.
- Exit code for rejected options: `--width abc` and `--bogus` give code 2 and a message (`Horizon: --width: invalid digit found in string`), as the guide says.
- It created in `--game-dir`: `config/`, `options.txt` and `horizon-graphics.txt`.

## 7. What could NOT be verified without XMCL's interface

- That XMCL's interface accepts a hand-written `instance.json`. It was only checked, with the bundled zod schema, that the minimal JSON parses without error, and from the code that `InstanceService` loads every folder in `instances/` at startup. It was not checked that it appears in the list, that the play button works or that XMCL does not rewrite it unexpectedly.
- The play button and the whole path from the interface: `instanceLaunch.ts`, interface-side prechecks, the download of Java 25 and of 26.3, sign-in. Only what the launch service does once it has the options was reproduced.
- That XMCL stops marking the instance as "starting" when it sees `LWJGL Version: zenith`, and that it detects `Crash report saved to: #@!@# <path>`: only the logic (section 4), not the interface's reaction.
- What XMCL shows when the bridge exits with 1 and writes to stderr.
- That no console window is seen with a console executable, and that none is seen with the Windows subsystem either (only standard output was measured; a window is not observed without a screen).
- That XMCL's shaders tab works on a vanilla instance and writes to `shaderpacks/` (the code creates `shaderpacks/` in the instance with `ensureDir`, `InstanceService.ts:~380`, but the flow was not tested).
- Asset indexes with aliases (`34.json` next to `<sha1>.json`): only the code was read; the game was not installed with XMCL.
- Whether `zenith-prepare --minecraft-dir <XMCL root>` works with a real XMCL installation: it was tested with Horizon's launcher layout, which is the same according to the code, but not with files installed by XMCL.
- The interface's "Stop" behaviour (`kill` versus `forceKill` and child processes).
- Offline, ely.by or LittleSkin accounts: only the code was read.
- Any network measurement (that `zenith-client` does not open unexpected connections) and the test on an online server: that is task 10.

## 8. Assumptions of tasks 6 to 10 that the evidence contradicts

| Task | The plan assumes | What was measured | Correction |
| --- | --- | --- | --- |
| 6 | `--width -1 --height -1` appear and mean "no size" (`minus_one_size_is_none`) | They never appear by default. With a resolution: `--height H --width W` (height first). With fullscreen: `--fullscreen`. | The test must be "without `--width`/`--height` it is `None`". Tolerate `-1` in case it arrives. Add `fullscreen: bool` to `LaunchArgs` and to `client_cmd` (`--fullscreen` is in the Launch API). |
| 6 | The bridge rejects if `--userType` is not `msa` (spec, phase 2, point 2) | `--userType` does not exist in 26.3. | Remove that check. Keep only the network ownership check (offline accounts' token: 32 random hex, the services answer 401, it is `NotOwner`). Also reject if there is an authlib-injector `-javaagent:` in the JVM arguments. The session's `user_type` is always `msa`. |
| 6 | `--xuid` is optional and either comes or does not | It always comes, as the literal text `${auth_xuid}` (and `--clientId ${clientid}`). | Treat any value that starts with `${` as absent. `xuid` is `None`. |
| 6 | `--server a.b --port 25566` and `quickPlayMultiplayer` are alternatives | With a server on the instance XMCL passes **both** at once, in the order `--quickPlayMultiplayer`, `--server`, `--port`. | Give priority to `--quickPlayMultiplayer`. The other two only if it is missing (in case an old version of XMCL or Minecraft uses them alone). |
| 6 | `argv[0]` is Java | Yes, but it is `javaw.exe`, not `java.exe`, and with a mixed path (`\` in Java, `/` in the rest). | `--java-home` = parent of the parent of `bin` (valid for `javaw.exe`). Path tests with mixed `\` and `/`. |
| 6 | `minecraft_root()` is the parent of `assets_dir` | Correct. | No change. Note that `--assetIndex` is a sha1, not the index's name. |
| 6 | `parse` ignores what it does not know | True, and convenient: there are 40 JVM arguments and the classpath. | No change. |
| 9 | `run_game` must force or avoid consoles with `CREATE_NO_WINDOW` (task 5 asked whether `windows_subsystem` was needed) | `#![windows_subsystem = "windows"]` is needed **in the bridge**; with a console the standard output is lost and XMCL never sees `LWJGL Version: zenith`. | `main.rs` with `#![windows_subsystem = "windows"]` on Windows. Errors go to stderr and, with the Windows subsystem, also to `bridge.log` (stderr will not be seen in any console). |
| 9 | `prepare_cmd` does not use `--root` | The preparation writes about 400 MB in `<build>\.local` if it is not given `--root`. | Pass `--root <home>/prepared-root/<version>` (or whatever folder is decided) so nothing is written inside the installed build; check exactly what it does first. |
| 9 | The prepared folder is read-only after preparation | The client writes `horizon-data.bin` (120 MB) in it on the first start. | It must be writable. A first start also takes about 17 s in the background. |
| 8 | `install` extracts the zip in `builds/<v>` and `current()` is that folder | The zip has a root folder `horizon-windows-x64/`. | The executable is in `builds/<v>/horizon-windows-x64/zenith-client.exe`; or the root folder is dropped when extracting. Also check that `enclosed_name()` tolerates that structure. |
| 8 | `SHA256SUMS` lists what the bridge downloads | `SHA256SUMS` also lists the Linux `.tar.gz`; `verify` requires having every listed file. | The bridge verifies with `check` on the specific file, not with the folder `verify` command. |
| 10 (Superseded: see section 10) | `setup` writes `instance.json` and works with any root | **With the default root, `cmd.exe` rejects the command (section 2).** | Decision pending before task 10 (see below). |
| 10 | A path with spaces needs quotes (`quotes_path_with_spaces`) | Confirmed: double quotes. Without them it is split into pieces. | Already planned. Write `"<path>"` in `prependCommand`. The test must use a path with Ñ. |
| 10 | XMCL loads the instance as soon as it is written | It is loaded when XMCL starts; it must be restarted (already planned in the README). | No change, but `setup` must warn if XMCL is running. |

### Decision pending because of the `cmd.exe` limit finding

(Superseded: see section 10; the bridge as the instance's "Java" was chosen.)

It is not resolved here. Options, from cheapest to most expensive, none tested with a real XMCL:

1. Require a short data root (about 19 characters or less for 26.3; for example `C:\xmcl`), chosen at XMCL's first start, and have `zenith-bridge setup` check it by reading `%APPDATA%\xmcl\root` and explain it. The margin depends on the bridge's path (each extra character subtracts one) and on the number of libraries of the version; `setup` must generate the command with the same rules as XMCL and check that it does not exceed 8191 characters, instead of trusting a fixed number.
2. A short directory junction (`mklink /J`) pointing at the real root, writing that short path in `%APPDATA%\xmcl\root`. It would have to be checked that XMCL respects it and does not resolve the link.
3. Ask XMCL not to use `shell: true` on Windows when the prepend command is an executable (a change in `@xmcl/core`/`LaunchService`). It contradicts the design's "XMCL must not be modified".
4. Drop `prependCommand` for this phase and move on to phase 6 (own launcher).

Until it is decided, task 10 should not be written as it stands. See section 10: a fourth way (the bridge as the instance's `java`, without `cmd.exe`) measured afterwards, which also corrects the 19-character limit (it was measured with a 10-character token).

## 9. Side effects of this test (to be honest)

- The folder `C:\zx` was created (three subfolders and a copy of the version JSON, ~45 KB) as a short test root, and could not be deleted from this session (the deletion was blocked by a safety check). It has to be deleted by hand: `rmdir /s /q C:\zx`.
- When testing rejected options (`--width abc`, `--bogus`) **without** defining `HORIZON_HOME`, `zenith-client` wrote its log in `%APPDATA%\horizon\logs` (the default): it rotated the previous `latest.log` to `2026-11-04-4.log.gz`, created `2026-11-05-1.log.gz` and rewrote `latest.log`. This goes against the rule of not touching `%APPDATA%\horizon*` and **leaves the file list of `%APPDATA%\horizon` different from before the test**; the comparison in step 6 of task 10 must start from a profile that is noted down again. Nothing else was modified (the previous log's content was kept compressed; `%APPDATA%\horizon-launcher` had no change). No attempt was made to undo it, so as not to write more there.
- All downloads, unpackings, the prepared folder, the build folder and the XMCL clone are in `%TEMP%\zenith-spike`. They are not deleted so they can be reused in later tasks.

## 10. Alternative: the bridge as the instance's "java" (without cmd.exe)

The finding that motivates this: with `prependCommand` XMCL uses `shell: true` (cmd.exe, limit 8191) and passes the **real** Minecraft token, a JWT that can be over a thousand characters. The idea: do not use `prependCommand` and put in the `java` field of `instance.json` a copy of the bridge that pretends to be Java. Then XMCL launches `<java> -Xmx… -cp … net.minecraft.client.main.Main <args>` **without a shell** and the `CreateProcess` limit (32 767) rules.

Static test, like the previous one: no clicks in the interface. The code of the same XMCL clone (`aeb5240`, read only) was read and its real functions (bundled with esbuild) were run against a fake "java". Environment: Windows 11 (10.0.26300), Node v24.22.1. Everything in a temporary folder; `%APPDATA%\horizon*`, `%APPDATA%\xmcl`, `C:\xmcl` and `%APPDATA%\zenith` were not touched. It is reproduced with `bridge/tests/fixtures/java-shim-spike/` (see "How to reproduce it").

### Verdict

**Viable with caveats.** XMCL's mechanism (Java resolution, validation, the switch to `javaw.exe`, launching without a shell, reading stdout) works with an executable that answers `-version`, and the command line leaves about 21 000 characters of margin. Things that need XMCL's interface remain open (last section) and two design consequences that have nothing to do with the limit: XMCL no longer downloads a Java 25 for the bridge, and other instances could choose the shim as their Java. The design with `prependCommand` is **in serious doubt**: with `C:\xmcl` it fails as soon as the token exceeds about 1090 characters, and the real length could not be measured.

### How XMCL decides, with code evidence

**No shell, except with a prepend command.** `LaunchService.#generateOptions` (`xmcl-runtime/launch/LaunchService.ts:228`): `shell: prepend && prepend.length > 0`. `@xmcl/core`'s `launch()` (`packages/core/launch.ts:688-698`) only quotes the arguments if `extraExecOption.shell` is true, and then does `spawn(args[0], args.slice(1), { cwd, detached: true, env })`. `inst.prependCommand ?? globalPrependCommand` (`xmcl-runtime-api/src/util/launch.ts:140`) uses the global only if the instance's is `undefined`, and `normalizeCommandLine('')` returns `''` (false), so **an explicit `"prependCommand": ""` on the instance also overrides a global prepend command**; the schema accepts the empty string (`packages/instance/instance.ts:273`). Measured below: no shell.

**What XMCL runs to find out what "Java" is** (`packages/installer/java.ts`, `resolveJavaWithDiagnostic`): it checks that the file exists and runs `execFile(<path>, ['-version'], { windowsHide: true })`. It reads the version from **stderr and, failing that, from stdout** (`parseJavaVersionOutput`), **without looking at the exit code** (it only keeps it for diagnostics). `parseJavaVersion` requires the text to contain `openjdk`, `java version` or `java_version` as a word, and a number. It does not read `<home>/release` except to fill in an invalid record after failing (`JavaService.ts:170`). It does not use `-XshowSettings:properties` (grep with no results) and does not look at the file name, nor the vendor, nor whether it is in `bin/` under a home. Measured with `parseJavaVersion`:

| Output | Result |
| --- | --- |
| `openjdk version "25"` | `{version:"25", majorVersion:25}` |
| `java version "25"` | the same |
| 3 Temurin lines with `"25.0.5"` | `{version:"25.0.5", majorVersion:25}` |
| `25.0.5`, `version "25.0.5"`, a banner of our own or nothing | `undefined` (not Java) |

`resolveJava` on an executable that only answers `-version` (`fakejava.rs`) gives `25.0.5 (major 25)` **both from a console and from a Windows-subsystem build** (with stdout and stderr on pipes, the Windows-subsystem one also writes). A foreign binary (`whoami.exe`) gives `undefined`: XMCL **rejects the unknown**; it only accepts what speaks "java". A non-existent file gives `ENOENT`.

**Optional extra.** `JavaService.resolveJava` calls `getJavaArch` (`detectJVMArch.ts`): `execFile(<java>, ['JavaProxy','-jvm_info'], { cwd: <XMCL data> })` and expects JSON with `java_arch` (`"64"` becomes `x64`). If it fails it only warns and leaves the architecture undefined (cosmetic: it shows in the Java list). The shim answers `{"java_arch":"64"}`.

**How much version XMCL requires.** The `java-validation` middleware (`pluginLaunchPrecheck.ts:99-149`) **does not compare versions**: it only requires `resolveJava` to return something. The comparison is in the interface: `getAutoOrManuallJava` (`xmcl-runtime-api/src/util/java.ts`) does it against the version JSON's requirement (`javaVersion.majorVersion`; for 26.3: `java-runtime-epsilon`, **25**, rule `>=25`) and `getInstanceJavaIssue` (`instanceJavaDiagnose.ts`) opens the "incompatible" dialog if it is not met. The real selection functions were run (`validate.cjs`; the last point reproduces `getInstanceJavaIssue` by hand):

```
shim 25 (java.exe)         resolveJava: 25.0.5 (major 25) | arch: 64 | quality: 0 (Matched)  issue: null
shim 25 (javaw.exe)        resolveJava: 25.0.5 (major 25) | arch: 64 | quality: 0 (Matched)  issue: null
shim that says 21          resolveJava: 21.0.5 (major 21) | arch: 64 | quality: 2            issue: incompatible
foreign binary (whoami.exe) resolveJava: undefined                                           issue: invalid
non-existent path          X_OK: ENOENT, resolveJava: undefined                              issue: invalid
```

With XMCL's Java list empty (`all` empty) the result is `invalid` **even with a real Java**, because `getAutoOrManuallJava` returns `noJava` without looking at the field. It is transient: `resolveJava` adds the Java to the list and the interface recomputes (`watch` in `instanceJava.ts`). That refresh could not be seen without the interface.

### The switch to `javaw.exe` and the shim's names

`#generateOptions` (lines 205-206) computes `join(dirname(javaPath), 'javaw.exe')` and, if it exists and `access(X_OK)` (on Windows: that it exists), launches **that** instead of the field, whatever the field's name. In contrast `java-validation` and the interface probe `options.java`, the field's. Therefore:

- **`java.exe`** must exist: it is what is probed with `-version` and what goes in the `java` field.
- **`javaw.exe`** must be **in the same folder**: it is what receives the game's arguments. It can be the same binary: it was tested with a console `java.exe` + a Windows-subsystem `javaw.exe` (A1), only a console `java.exe` (A2) and both Windows-subsystem (A3). In all three it launches and XMCL sees the output. Without `javaw.exe`, `java.exe` is launched.

### What the shim receives and how its output is treated

The real `core.launch()` of `@xmcl/core` was run with the same options as XMCL (`shell: false`, `detached: true`, `cwd`, `env`), a fake token of 1200 characters and the real `createMinecraftProcessWatcher` (the one `LaunchService` uses), against the shim (`e2e.cjs`, summarised):

```
A1 java.exe console + javaw.exe winsub  launched=jre\bin\javaw.exe   ready=true code=0 stdout="LWJGL Version: zenith" stderr="stderr line from shim" argc=43 cwdOk=true pidEqualsNodes=true
A2 only java.exe (console)              launched=jre2\bin\java.exe   ready=true code=0 stdout="LWJGL Version: zenith" …
A3 java.exe and javaw.exe winsub        launched=jre3\bin\javaw.exe  ready=true code=0 stdout="LWJGL Version: zenith" …
```

- **Standard output arrives in all three cases**, also with the console executable, unlike the `shell: true` case in section 2. `minecraft-window-ready` fires with `LWJGL Version: `; stderr arrives separately.
- **The shim's `argv`:** `argv[0]` is the path of `javaw.exe` that XMCL passes and then come the 42 arguments of the `xmcl-args.txt` fixture (which has `javaw.exe` as its first line, 43 in total). After normalising the root they are identical, except the uuid and the token, which in the fixture were filler (`#34`, `#36`). Compared with `prependCommand` **there is no bridge path before**: `argv[1]` is already `-XX:HeapDumpPath=…`.
- **`process.pid` is the shim's** (the `PID` that the shim dumps matches Node's `pid`). There is no `cmd.exe` in between, so XMCL's Stop (`process.kill()`) already reaches the bridge; whether the game's tree dies with it depends on the bridge's job object (`d628dd28a`).
- **Without a shell there is no expansion or loss of characters.** With the path `…/Jose Ñ/java dir/bin/javaw.exe` and extra arguments `100%ZXTEST%`, `a&b`, `q"uote` and one with a tab, the shim received exactly `["--x","100%ZXTEST%","--y","a&b","--z","q\"uote","--w","tab\there"]` and the path with Ñ and spaces intact. (With `cmd.exe`, section 2, `%…%` was expanded and the quotes were lost.)
- A `-version` takes about 12.6 ms (average of 20, Windows subsystem).

### Measured line length (the reason for this section)

The same 26.3 command (82 jars) with `core.generateArguments`, a fake token of variable length (a real JWT could not be obtained), data root `C:\Users\Some Long User Name\AppData\Roaming\xmcl` (49 characters) and Java at `C:\Users\Some Long User Name\AppData\Roaming\zenith\java\bin\javaw.exe`. "Without shell" is the real shim's `GetCommandLineW`, corrected to the long Java path (+2 for the quotes libuv puts on a path with spaces; an uncertainty of about 3 characters). "With prepend + shell" is the length of the command `cmd.exe` receives (bridge path of about 70 characters, as in the previous test):

| Token | Without shell (CreateProcess, limit 32 767) | Margin | With prepend + shell (cmd.exe, limit 8191) |
| --- | --- | --- | --- |
| 10 | 10 444 | 22 323 | 10 531 |
| 700 | 11 134 | 21 633 | 11 221 |
| **1200** | **11 634** | **21 133** | **11 721** |
| 1500 | 11 934 | 20 833 | 12 021 |

- Real `cmd.exe`, long root, token 1200, with prepend: **code 1, "The command line is too long.", the program did not run.**
- Without a shell it fits with room to spare. The line grows 82 characters per root character, so a root of about 300 characters would be needed to reach 32 767 (before that `MAX_PATH` would weigh in elsewhere).
- **It corrects section 2.** The "about 19 characters" limit was measured with a 10-character token. With the recommended short root and the current design:

  ```
  prepend+shell, root C:\xmcl (7), token 10:   command=7108, code 0, shim ran: true
  prepend+shell, root C:\xmcl (7), token 1200: command=8298, code 1, shim ran: false
  ```

  The token counts character by character: with `C:\xmcl` the `prependCommand` design only works with tokens of up to about 1090 characters (8191 − 7108 + 10). The real token's length is unknown here.

### What happens with XMCL's interface and settings (read, not run)

- **XMCL does not replace our `java` field by itself.** The interface uses it as it is if assigned (`getAutoOrManuallJava`: "User assigned java path, revalidate it"; `instanceVersionInstall.ts:44-47`: with `inst.java` defined it neither installs nor looks for another; `InstallCoordinator.ts:150-166`: "A pinned instance Java wins when it resolves"). The only automatic replacement is in the middleware and **only if the path hangs from `<root>/jre/<component>`** (`getManagedJavaComponent`): if `resolveJava` fails, XMCL removes that Java and **reinstalls Mojang's**. **The shim must not be under `<XMCL root>/jre`.** Outside it, if it fails, the error is `launchNoProperJava`: visible and with no replacement.
- **The global Java** (`globalJava`) only breaks ties in the **automatic choice** when `instance.java` is empty (`selectJavaByPreference`, parameter `preferredPath`); it does not override an assigned field. There is no XMCL setting to change.
- **Manual replacements** of the field (done by the user): the button to use the recommended Java in the Java problem dialog (`HomeLaunchStatusDialog.vue:192-196`), the instance's Java page and the agent tools (`tools.ts:1450-1456`). If the shim answered `-version` wrongly, that dialog would offer exactly to replace it.
- **Java list.** The shim will appear in XMCL's Java list (cache in `%APPDATA%\xmcl\java.json`, revalidated on every start with `-version`: `refreshLocalJava`), as 25.0.5 `x64`. XMCL only searches known paths (Mojang, Oracle, OpenJDK, Adoptium, Zulu, `JAVA_HOME`, registry, `where java`, `<root>/jre/*`); a folder of our own such as `%APPDATA%\zenith\java` is not discovered by itself, it gets in through the instance.
- **Risk measured with the selection function:** an instance **without its own Java** of another version (for example 1.21.4, which asks for `>=21`) may choose the shim for being "Java 25" if it is earlier in the list:

  ```
  list [shim25, real21] -> chooses S/bin/java.exe     list [real21, shim25] -> chooses R/bin/java.exe
  ```

  It is avoided by making the shim's game mode **refuse** anything that is not its Minecraft version (exit ≠ 0 with a clear message) or delegate to a real Java. Not tested in the interface.
- **Java download.** With the field assigned and resolvable, XMCL **does not download** `java-runtime-epsilon` (see above). Consequence: the bridge no longer receives a Java 25 in `argv` for `zenith-prepare --java-home`; it would have to provide or locate its own. This is not a flaw of the shim but of the design.

### Exact conditions of the alternative

1. A folder of our own **outside `<XMCL root>/jre`**, for example `%APPDATA%\zenith\java\bin\`, with `java.exe` and `javaw.exe` (two independent copies; no hard links, because of the `ino` that `refreshLocalJava` uses to avoid duplicates). Optional: `..\release` with `JAVA_VERSION="25.0.5"` (only read after a failure).
2. `instance.json`: `java` with the absolute path of `…\bin\java.exe` (in JSON the backslashes are doubled) and `"prependCommand": ""` (it also overrides the global one; `shell` stays false).
3. A single Windows-subsystem executable is valid for both names (measured). The mode is decided by the file name (`java`/`javaw`) or by `argv[1]` (`-version`, `JavaProxy`).
4. Answers to the probes: fast, code 0, no network or writes.
   - `-version` (only argument): on stderr (or stdout) `openjdk version "25.0.5" <date> LTS` and the two runtime lines; the major must be **greater than or equal to the JSON's `javaVersion.majorVersion`** (25 for 26.3).
   - `JavaProxy -jvm_info`: `{"java_arch":"64"}` on stdout (optional).
   - Any other invocation is a game launch.
5. Game mode: read `argv` (fixture), start Zenith, write on **stdout** `LWJGL Version: …` when ready and `Crash report saved to: #@!@# <path>` if it fails; stderr accumulates for the error dialog; exit with the game's code.
6. Windows locks a running `javaw.exe`: the copy can only be replaced with the game closed. It has to be decided who restores it after a bridge update (`setup`, or the `-version` mode itself if no game is running).

### What could NOT be verified without the interface

- That XMCL loads a hand-written instance with `java` and `prependCommand: ""` and does not rewrite them (for example when opening the instance's Java page and saving).
- How the Java page, the global Java list and the Play button look with the shim (version, architecture, absence of warnings), and the refresh after the transient state of an empty list.
- That there is no console window or flicker with the shim (console or Windows subsystem); only standard output was measured.
- What XMCL shows when the shim exits with a code ≠ 0, and its reaction to `Crash report saved to:`.
- The real Minecraft token: its length was not obtained (nobody can sign in here); the 1200 characters are an assumption. The conclusion about `cmd.exe` depends on it; the shim's does not (21 000 characters of margin).
- The Stop button (normal and forced `kill`) and the process tree.
- That another instance does not choose the shim in practice and that no normal path offers to replace it.
- That a version of XMCL other than `aeb5240` (0.73.1) validates Java the same way.
- Antivirus or SmartScreen facing a `java.exe` that is not from a known vendor.

### How to reproduce it

```
sh bridge/tests/fixtures/java-shim-spike/build.sh <XMCL clone, read only> <work folder> <26.3.json>
# needs rustc, node 24 and esbuild (ESBUILD_DIR=<path to node_modules/esbuild> if it is not installed)
```

It compiles `fakejava.rs` (console, Windows subsystem and the "says Java 21" variant), bundles with esbuild `@xmcl/core`, `packages/installer/java.ts` and `xmcl-runtime-api/src/util/java.ts` from the clone and runs `e2e.cjs` (launch, lengths, `cmd.exe`), `validate.cjs` (resolution, quality, selection in other instances) and `probe.cjs`. It only writes in the work folder.
