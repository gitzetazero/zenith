"""Horizon -> Zenith names, layout and exceptions. Used once to rename the tree and by port_upstream.py to translate Horizon's patches.

Mapped: crate/package/binary names (`horizon-core` -> `zenith-core`, `horizon_core` -> `zenith_core`, `horizon-ctl`), the game binary
(`horizon-client` as an executable -> `zenith`), packaging icon names and the directory `mods/horizon` -> `java/horizon`.
Kept on purpose (they are formats and protocols, not names): the wire brand "horizon", `horizon.*` lang keys, assets/horizon,
`horizon-mods/` (the game's mod folder and the `overrides/horizon-mods` modpack path), `horizon-shaders.toml`,
`horizon-prepare.json`, the friends protocol labels, Horizon's release asset names and upstream URLs."""
import re

# Horizon's crates (including the ones Zenith does not carry, so patches that mention them translate consistently).
CRATES = """cargo-horizon-mod horizon-anticheat horizon-assets horizon-bench horizon-bots horizon-chat-filter
horizon-client horizon-client-core horizon-client-input horizon-client-lib horizon-client-render horizon-command
horizon-core horizon-cosmetics horizon-dualsense horizon-friends horizon-hdr horizon-items horizon-mcp horizon-mod-api-gen
horizon-mod-deps horizon-mod-registry horizon-mods horizon-narrator horizon-optifine horizon-port-pack horizon-prepare
horizon-protocol horizon-relay horizon-release-sign horizon-server horizon-server-files horizon-shader-api horizon-shaders
horizon-telemetry horizon-telemetry-server horizon-theme horizon-threads horizon-world""".split()
# binaries that are not crates
BINS = ["horizon-ctl", "horizon-shader", "horizon-parity", "horizon-tick-parity", "horizon-geometry-parity",
        "horizon-noise-parity", "horizon-density-parity", "horizon-float-log-parity", "horizon-worldgen-parity"]
MAP = {}
for _n in CRATES + BINS:
    MAP[_n] = _n.replace("horizon", "zenith")
    MAP[_n.replace("-", "_")] = MAP[_n].replace("-", "_")
_NAMES = "|".join(re.escape(k) for k in sorted(MAP, key=len, reverse=True))
_RX = re.compile(r"(?:(?<![A-Za-z0-9_-])|(?<=\\n))(" + _NAMES + r")(?![A-Za-z0-9_])")
_URL = re.compile(r"(https?://|github\.com/|horizon-rust/)\S*$")
# The game executable (a file name, not the package): becomes `zenith`.
_EXE = [(re.compile(r"CARGO_BIN_EXE_horizon-client"), "CARGO_BIN_EXE_zenith"),
        (re.compile(r"((?:fast|release|debug)/)horizon-client"), r"\1zenith"),
        (re.compile(r"horizon-client(?=\{EXE\}|\.exe)"), "zenith"),
        (re.compile(r"(--bin )horizon-client\b"), r"\1zenith"),
        (re.compile(r"(CARGO_BIN_EXE_)(horizon-[a-z-]+)"), lambda m: m.group(1) + MAP.get(m.group(2), m.group(2)))]
_LAYOUT = [(re.compile(r"cargo horizon-mod"), "cargo zenith-mod"),
           (re.compile(r"(?<![\w-])mods/horizon\b"), "java/horizon"),
           (re.compile(r"horizon\.(ico|icns)\b"), r"zenith.\1"),
           (re.compile(r"(apps/)horizon\.(png|svg)"), r"\1zenith.\2"),
           (re.compile(r"horizon(-launcher)?\.desktop"), r"zenith\1.desktop")]
_CRATE_CTX = re.compile(r"(crates/|\.\./|-p |crate `|crate |name = \")$")


def _keep(m, text):
    """True when this match is one of the names that stay Horizon's."""
    word, end = m.group(0), m.end()
    before = text[max(0, m.start() - 12):m.start()]
    if word == "horizon-mods":  # the crate stays a crate; elsewhere it is the game's mod folder
        return not (_CRATE_CTX.search(before) or text.startswith("/index", end) or text.startswith(" = {", end))
    if word == "horizon-shaders" and text.startswith(".toml", end):
        return True
    if word == "horizon-prepare" and text.startswith(".json", end):
        return True
    if word == "horizon-friends" and before.endswith('"') and text[end:end + 1] in (" ", '"', "-"):
        return True
    return word == "horizon-hdr" and text.startswith("-pending", end)


def rewrite(text):
    """Translate Horizon names to Zenith's in `text`, leaving upstream URLs and the kept names alone."""
    for rx, s in _EXE:
        text = rx.sub(s, text)

    def sub(m):
        word = m.group(0)
        if _URL.search(text[max(0, m.start() - 80):m.start()]) or _keep(m, text):
            return word
        return MAP[word]

    text = _RX.sub(sub, text)
    for rx, s in _LAYOUT:
        text = rx.sub(s, text)
    return text


def map_path(path):
    """Where an upstream path lives in Zenith's tree."""
    path = re.sub(r"^mods/horizon(?=/|$)", "java/horizon", path)
    out = []
    for part in path.split("/"):
        if part in ("horizon.ico", "horizon.icns"):
            part = part.replace("horizon", "zenith")
        elif part.startswith("horizon.") and part.rsplit(".", 1)[-1] in ("png", "svg", "desktop"):
            part = part.replace("horizon", "zenith", 1)
        elif part == "horizon-launcher.desktop":
            part = "zenith-launcher.desktop"
        elif part in ("horizon-shaders.toml", "horizon-prepare.json"):
            pass
        else:
            part = _RX.sub(lambda m: MAP[m.group(0)], part)
        out.append(part)
    return "/".join(out)
