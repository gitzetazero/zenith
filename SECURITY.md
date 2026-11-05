# Security

## How to report a vulnerability

**Do not open a public issue and do not publish details.**

Use GitHub's private advisory: in this repository, the **Security** tab, then **Report a vulnerability**
(<https://github.com/gitzetazero/zenith/security/advisories/new>). Only the maintainers see it.

Say what you found, how to reproduce it and which version (nightly) you were using. We answer as soon as we can; this is a small project and we do not promise deadlines.

## What is in scope

- The **bridge** (`zenith-bridge`) and the Zenith Launcher installer.
- The **signed updates**: the verification of signatures, hashes and the build stamp.
- The account's **ownership check** (that only people who own Minecraft: Java Edition can play).
- **Zenith's changes to the game client**, especially the server version translator and the version selector.

## What is out of scope

- Bugs in the project we come from, [Horizon](https://github.com/horizon-rust/horizon), that Zenith does not change: report them there.
- Bugs in XMCL, Minecraft, Mojang or Microsoft: report them to their owners.
- The installer not being code-signed for Windows, or SmartScreen warning: this is known and stated in the README.

## Supported versions

Only the latest `nightly` published in [Releases](https://github.com/gitzetazero/zenith/releases/tag/nightly).
