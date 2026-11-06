"""Fails when a zenith-client no longer offers what the Zenith bridge relies on (Launch API 0.1)."""
import json
import subprocess
import sys

NEEDED = ["--resources", "--game-dir", "--home", "--width", "--height", "--session-stdin",
          "--quickPlayMultiplayer", "--quickPlaySingleplayer"]


def problems(api):
    """The reasons `api` (the parsed `--launch-api` JSON) is not enough; empty when it is."""
    found = []
    if not str(api.get("launch_api", "")).startswith("0.1"):
        found.append(f"launch_api is {api.get('launch_api')!r}, expected 0.1.x")
    have = {o.get("name") for o in api.get("options", []) if isinstance(o, dict)}
    found += [f"option {o} is missing" for o in NEEDED if o not in have]
    return found


def main(argv):
    if len(argv) != 2:
        print("usage: check_launch_api.py <zenith-client>", file=sys.stderr)
        return 2
    out = subprocess.run([argv[1], "--launch-api"], capture_output=True, text=True, check=True).stdout
    found = problems(json.loads(out))
    for p in found:
        print(f"launch API: {p}", file=sys.stderr)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
