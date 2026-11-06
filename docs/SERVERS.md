# Supported servers and versions

Zenith is Minecraft 26.3. To join a server of another version, it translates what the game and the server say to each other. The translation is on by default and you do not have to do anything: Zenith checks which version the server uses and adapts.

> **Important, plainly:** none of the translated versions has been tested against a real server yet. The translator has automated tests of the protocol and of each version, but it has not been tried on real servers. If you join one, it may fail. Tell us what happened in an [issue](https://github.com/gitzetazero/zenith/issues/new/choose).

## Version table

| Server version | Protocol | How you join | Status | Notes |
|---|---|---|---|---|
| 26.3 | 777 | Native | Native (26.3) | Zenith's own version: nothing is translated. |
| 26.2 | 776 | Translated | Translated | Selector: "26.2". |
| 26.1, 26.1.1 and 26.1.2 | 775 | Translated | Translated | The three share a protocol. Selector: "26.1". |
| 1.21.11 | 774 | Translated | Translated | |
| 1.21.9 and 1.21.10 | 773 | Translated | Translated | Selector: "1.21.10". |
| 1.21.7 and 1.21.8 | 772 | Translated | Translated | Selector: "1.21.8". |
| 1.21.6 | 771 | Translated | Translated, experimental | |
| 1.21.5 | 770 | Translated | Translated, experimental | |
| 1.21.4 | 769 | Translated | Translated, experimental | |
| 1.21.2 and 1.21.3 | 768 | Translated | Translated, experimental | Selector: "1.21.3". |
| 1.21 and 1.21.1 | 767 | Translated | Translated, experimental | Selector: "1.21.1". |
| 1.20.5 and 1.20.6 | 766 | Translated | Translated, experimental | Selector: "1.20.5". |
| 1.20.3 and 1.20.4 | 765 | Translated | Translated, experimental | Selector: "1.20.3". |
| 1.20.2 | 764 | Translated | Translated, experimental | The oldest that can be translated. |

"Experimental" is the mark the version selector puts on 1.21.6 and older. The rest are not marked, but they are not tested with real servers either.

## What you CANNOT do

- **Versions older than 1.20.2** (for example 1.8, 1.12 or 1.16). That rules out many classic PvP servers.
- **Bedrock.** It is another game with another protocol.
- **Servers that require mods** on the client: Zenith does not run Java mods.
- **Servers that only accept one specific client** (for example, with an allow-list of clients).
- Servers behind Velocity, BungeeCord, Paper, or with server-side-only mods (Fabric, for example) work if they accept the protocols above, because to the player they are a normal server.

## How Zenith chooses the version

1. **Automatic.** When you click the server, Zenith asks it which version it uses (the same query the server list makes) and picks the matching translator. If the server is already 26.3, nothing is translated.
2. **Your choice.** The *Edit server* screen has a "Version" button. Cycle it to the one you want or leave it on "Automatic". Your choice is saved per address in `zenith-servers.json`, next to your server list. If you fix a version, Zenith no longer asks the server.
3. **One retry.** If the server kicks you with a message that states a version range ("use 1.7.2-26.2", for example), Zenith joins again once with the newest version it can translate inside that range. If that fails too, it stops: there are no loops. It does not retry on local worlds or if you already chose a version.

If the connection drops on a translated server, the error message ends with the version used, for example "(26.2)".

## Turning the translator off

Set the environment variable `HORIZON_OLDER_SERVERS=0` before opening the game. With it you can only join 26.3 servers. It has this name because Zenith keeps the internal names of its base.

## Things you will notice

- The game is 26.3: blocks, items and mobs from versions newer than the server may look different or be missing on old servers, and those from older versions may look as in 26.3.
- Some anticheats may react to a translated client. If you are kicked, it is not your fault: tell us.
- Features that an old server does not have do not work: the translator does not invent them.

## How it was checked

- Yes: automated tests of the protocol and of each translator; the numbers and names in this table come from the code (`crates/zenith-protocol/src/adapter.rs`).
- No: no test against real servers of any version. The test matrix with real servers has not been run yet.
