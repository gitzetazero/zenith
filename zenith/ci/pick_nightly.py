"""Pick the newest Horizon nightly whose source tag is an ancestor of --ref."""
import argparse
import json
import re
import subprocess

TITLE = re.compile(r"^Nightly (\d{4}-\d{2}-\d{2}) \u00b7 ([0-9a-f]{7,40})$")


def parse_title(title):
    m = TITLE.match(title)
    return (m.group(1), m.group(2)) if m else None


def pick(releases, reachable):
    """releases: (tagName, title) pairs, newest first."""
    for tag, title in releases:
        parsed = parse_title(title)
        if parsed and reachable(parsed[1]):
            date, short = parsed
            return {"tag": f"nightly-source/{date}-{short}", "release": tag, "date": date, "short": short}
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ref", required=True)
    ref = ap.parse_args().ref
    out = subprocess.run(
        ["gh", "release", "list", "-R", "horizon-rust/horizon", "--limit", "50", "--json", "tagName,name"],
        check=True, capture_output=True, text=True, encoding="utf-8").stdout
    releases = [(r["tagName"], r["name"]) for r in json.loads(out)
                if r["tagName"] == "nightly" or r["tagName"].startswith("nightly-")]
    by_short = {}
    for tag, title in releases:
        p = parse_title(title)
        if p:
            by_short[p[1]] = f"nightly-source/{p[0]}-{p[1]}"

    def reachable(short):  # tag missing locally also returns non-zero -> skipped
        return subprocess.run(["git", "merge-base", "--is-ancestor", by_short[short], ref],
                              capture_output=True).returncode == 0

    print(json.dumps(pick(releases, reachable)))


if __name__ == "__main__":
    main()
