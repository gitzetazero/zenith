# Zenith Launcher: player guide

**Zenith Launcher** is the program that opens Zenith. It is a fork of [XMCL](https://github.com/Voxelum/x-minecraft-launcher) (Voxelum, MIT license) with Zenith branding and the Zenith **bridge** built in. Downloads: <https://github.com/gitzetazero/zenith/releases/latest>. The source is in [`launcher`](launcher); how it differs from XMCL: [`launcher/docs/ZENITH-LAUNCHER.md`](launcher/docs/ZENITH-LAUNCHER.md).

It is an independent project and **not affiliated with XMCL (Voxelum), Mojang or Microsoft**. It is **experimental**: checked with automated tests and the contents of the published files, not by running the installer on third-party PCs.

## Requirements

- Windows 10 or 11 (Windows only for now).
- A Microsoft account that owns **Minecraft: Java Edition**. The bridge checks it with Minecraft's services. Sign in with it once; after that you can also play with an **offline profile** (a name without signing in, for offline-mode servers and LAN) on that PC for 30 days after each sign-in. Without a recent verified owner on the PC, offline, ely.by and LittleSkin accounts are refused.
- Zenith ships no Mojang files: the launcher downloads Minecraft and the bridge prepares Zenith's resources from that download.

## Install

1. From the Releases page, download `Zenith-Launcher-Setup-x64.exe` (or `Zenith-Launcher-Portable-x64.zip`) and compare its SHA-256 with the hashes in the release notes (`Get-FileHash <file> -Algorithm SHA256`). That only detects a corrupt download.
2. Run it. **The build is not code-signed**, so SmartScreen will warn ("Windows protected your PC": More info, Run anyway). The installer is per-user, no administrator.
3. Open "Zenith Launcher". The first time it asks for a data folder; once chosen, the **Zenith** instance is created (once; delete `zenith-instance.json` in the launcher data folder to recreate it).
4. Sign in and press Play on the **Zenith** instance. Microsoft sign-in uses Zenith's own Azure app (public client id `ZENITH_MSA_CLIENT_ID`, set at build time). **Mojang must still approve that app registration**: until it does, Microsoft sign-in may be refused for players.

The first Play downloads Minecraft 26.3 and prepares Zenith's resources (about 30 seconds to a few minutes). Without Java 25, the bridge downloads Mojang's.

**Data:** the launcher keeps its data in `%APPDATA%\zenith-launcher` (never XMCL's folder) and Zenith's in `%APPDATA%\zenith`. **Uninstall:** Windows "Installed apps"; it never touches your data.

**Updates:** the launcher links to the Releases page when a newer version exists. The game itself updates through the bridge, with the signature checked before installing.

## Honest limits

- Not code-signed; not verified on third-party PCs; Windows x64 only.
- No telemetry, and no XMCL-hosted services (XMCL account, news, AI agent, P2P multiplayer relay are unavailable).
