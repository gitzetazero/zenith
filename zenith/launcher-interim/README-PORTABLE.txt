Zenith Launcher (powered by XMCL) - portable version
====================================================

Unzip this folder wherever you like (a short path is best) and open
"Zenith Launcher.cmd". It is the same as running "zenith-bridge.exe start".

First time
  1. Open "Zenith Launcher.cmd". XMCL asks for the data folder: choose it.
  2. Close XMCL completely and open "Zenith Launcher.cmd" again: this creates
     the "Zenith" instance.
  3. In XMCL sign in with your Microsoft account (it must own Minecraft:
     Java Edition), choose the "Zenith" instance and press Play.

What is and is not portable
  - XMCL has no portable mode: its data (the file %APPDATA%\xmcl\root and the
    data folder you choose) and Zenith's (%APPDATA%\zenith) are kept in your
    Windows profile even if this folder is on a USB stick. Deleting this
    folder does not delete them.
  - If you move this folder, open "Zenith Launcher.cmd" again: the instance
    is rebuilt pointing at the new location.

Updates
  - The game (Zenith) updates itself when you press Play, after its signature
    is verified.
  - XMCL updates with its own updater, which only replaces
    resources\app.asar. To renew the bridge, download a new package.

Zenith Launcher is not affiliated with XMCL, Mojang or Microsoft. The package
contains unmodified XMCL (MIT licence, in the licenses folder, which includes
NOTICE.txt). It is not code-signed for Windows: SmartScreen may warn you.
