"""Horizon -> Zenith names for crates and tools. Used once to rename the tree and by port_upstream.py to translate Horizon's patches.

Only crate/package/binary names are mapped. Kept on purpose: the wire brand "horizon", lang keys `horizon.*`, assets/horizon,
Horizon's release asset names, upstream URLs and the licence text."""
import re

# Horizon's crates (including the ones Zenith prunes, so patches that mention them translate consistently).
CRATES = """cargo-horizon-mod horizon-anticheat horizon-assets horizon-bench horizon-bots horizon-chat-filter
horizon-client horizon-client-core horizon-client-input horizon-client-lib horizon-client-render horizon-command
horizon-core horizon-cosmetics horizon-dualsense horizon-friends horizon-hdr horizon-items horizon-mcp horizon-mod-api-gen
horizon-mod-deps horizon-mod-registry horizon-mods horizon-narrator horizon-optifine horizon-port-pack horizon-prepare
horizon-protocol horizon-relay horizon-release-sign horizon-server horizon-server-files horizon-shader-api horizon-shaders
horizon-telemetry horizon-telemetry-server horizon-theme horizon-threads horizon-world""".split()
EXTRA = ["horizon-ctl"]  # binaries
NAMES = sorted(CRATES + EXTRA, key=len, reverse=True)
MAP = {}
for n in NAMES:
    new = n.replace("horizon", "zenith")
    MAP[n] = new
    MAP[n.replace("-", "_")] = new.replace("-", "_")
_RX = re.compile(r"(?<![A-Za-z0-9_-])(" + "|".join(re.escape(k) for k in sorted(MAP, key=len, reverse=True)) + r")(?![A-Za-z0-9_])")
_URL = re.compile(r"(https?://|github\.com/|horizon-rust/)\S*$")


def rewrite(text):
    """Translate crate/binary names, leaving upstream URLs alone."""
    def sub(m):
        if _URL.search(text[max(0, m.start() - 80):m.start()]):
            return m.group(0)
        return MAP[m.group(1)]
    return _RX.sub(sub, text)


def rewrite_path(path):
    return "/".join(rewrite(p) for p in path.split("/"))
