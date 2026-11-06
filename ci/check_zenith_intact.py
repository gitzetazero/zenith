#!/usr/bin/env python3
"""Checks that a port of Horizon's work did not silently drop Zenith's own work.

Compares the work tree with ci/zenith-manifest.json:
  (a) every Zenith-own file (a file Horizon does not have) still exists;
  (b) no Horizon file has fewer `zenith:` markers than before;
  (c) the key identifiers (IDENTIFIERS below) are still in their files.
Usage: check_zenith_intact.py [--repo DIR] [--manifest FILE]        check (exit 1 and list what is missing)
       check_zenith_intact.py --update [--upstream-ref REF]         rewrite the manifest from the work tree
`--update` is a deliberate act (run it when you add or remove Zenith work on purpose), never part of a sync.
"""
import argparse
import json
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import rename_map  # noqa: E402
MARKER = re.compile(r"(//|#|<!--)\s*zenith:")
# Files Horizon also has but that are ours (kept by merge=ours): they must stay.
EXTRA_FILES = ["README.md", "CONTRIBUTING.md", ".gitattributes", "LICENSE", "THIRD_PARTY_NOTICES.md"]
# (file, text that must stay in it)
IDENTIFIERS = [
    ("crates/zenith-client/src/client_lib.rs", "pub mod zenith_versions"),
    ("crates/zenith-client-core/src/lib.rs", "pub mod zenith_brand"),
    ("crates/zenith-client/src/net.rs", 'is_none_or(|v| v != "0")'),  # older servers on by default
    ("crates/zenith-client/src/net.rs", "pub fn older_servers_enabled"),
    ("crates/zenith-cosmetics/src/lib.rs", "pub const ON_BY_DEFAULT: bool = false;"),
    ("crates/zenith-telemetry/src/lib.rs", "fn the_endpoint_is_off_unless_set"),  # no endpoint by default
    (".gitattributes", "README.md merge=ours"),
    ("Cargo.toml", '"crates/zenith-client"'),
    ("Cargo.toml", '"crates/zenith-client-lib"'),
    ("Cargo.toml", '"crates/zenith-protocol"'),
    ("Cargo.toml", '"crates/zenith-prepare"'),
    ("Cargo.toml", '"xtask"'),
]


def git(repo, *args):
    return subprocess.run(["git", "-C", repo, *args], check=True, capture_output=True, text=True,
                          encoding="utf-8").stdout


def tracked(repo):
    return [p for p in git(repo, "ls-files", "-z").split("\0") if p]


def read(repo, rel):
    with open(os.path.join(repo, rel), encoding="utf-8", errors="replace") as f:
        return f.read()


def markers(repo, files):
    out = {}
    for p in files:
        if not os.path.isfile(os.path.join(repo, p)) or p.startswith(("docs/superpowers/", ".superpowers/")):
            continue
        n = sum(1 for line in read(repo, p).splitlines() if MARKER.search(line))
        if n:
            out[p] = n
    return out


def build(repo, upstream_ref):
    ours = set(tracked(repo))
    # against Horizon's tree at the last port, in Zenith's names: what it does not have is ours
    if upstream_ref is None:
        with open(os.path.join(repo, "ci/upstream-synced.txt"), encoding="utf-8") as f:
            upstream_ref = f.readline().split()[0]
    theirs = {rename_map.map_path(p) for p in git(repo, "ls-tree", "-r", "--name-only", upstream_ref).split("\n") if p}
    own = sorted(p for p in ours - theirs if os.path.isfile(os.path.join(repo, p)))
    return {"own_files": sorted(set(own) | set(EXTRA_FILES)),
            "markers": markers(repo, sorted(ours - set(own)))}


def check(repo, manifest):
    bad = []
    for p in manifest["own_files"]:
        if not os.path.isfile(os.path.join(repo, p)):
            bad.append(f"own file is gone: {p}")
    for p, n in manifest["markers"].items():
        have = markers(repo, [p]).get(p, 0)
        if have < n:
            bad.append(f"zenith: markers fell from {n} to {have} in {p}")
    for p, needle in IDENTIFIERS:
        if not os.path.isfile(os.path.join(repo, p)) or needle not in read(repo, p):
            bad.append(f"identifier missing in {p}: {needle}")
    return bad


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=".")
    ap.add_argument("--manifest")
    ap.add_argument("--update", action="store_true")
    ap.add_argument("--upstream-ref")
    a = ap.parse_args()
    repo = os.path.abspath(a.repo)
    path = a.manifest or os.path.join(repo, "ci/zenith-manifest.json")
    if a.update:
        m = build(repo, a.upstream_ref)
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            json.dump(m, f, indent=1, sort_keys=True)
            f.write("\n")
        print(f"manifest: {len(m['own_files'])} own files, {sum(m['markers'].values())} markers in {len(m['markers'])} files")
        return 0
    bad = check(repo, json.load(open(path, encoding="utf-8")))
    for b in bad:
        print("NOT INTACT:", b)
    if not bad:
        print("zenith intact")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
