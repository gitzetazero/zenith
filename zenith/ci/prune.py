#!/usr/bin/env python3
"""Keeps the paths in zenith/ci/prune-paths.txt out of the repository, also in the middle of a merge.

Idempotent. Removes tracked files that match (git rm), resolves modify/delete conflicts on them by deleting,
and takes the matching crates out of the root Cargo.toml workspace lists. Does not commit.
Usage: prune.py [--repo DIR] [--list FILE]   (prints one line per removed path, then a summary)
"""
import argparse
import fnmatch
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def parse(text):
    prune, keep = [], []
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        (keep if line.startswith("!") else prune).append(line.lstrip("!"))
    return prune, keep


def matches(path, pats):
    return any(path.startswith(p) if p.endswith("/") else fnmatch.fnmatchcase(path, p) for p in pats)


def git(repo, *args, check=True):
    return subprocess.run(["git", "-C", repo, *args], check=check, capture_output=True, text=True).stdout


def strip_members(toml, dirs):
    """Removes `"crates/x"` entries (members or exclude) of the pruned directories from Cargo.toml text."""
    for d in dirs:
        d = re.escape(d.rstrip("/"))
        toml = re.sub(rf'^[ \t]*"{d}",?[ \t]*\r?\n', "", toml, flags=re.M)  # one entry per line
        toml = re.sub(rf'"{d}",\s*', "", toml)  # inline list, not last
        toml = re.sub(rf',\s*"{d}"', "", toml)  # inline list, last
    return toml


def prune(repo, list_file):
    with open(list_file, encoding="utf-8") as f:
        pats, keep = parse(f.read())
    tracked = [p for p in git(repo, "ls-files", "-z").split("\0") if p]  # unmerged entries are listed too
    doomed = sorted({p for p in tracked if matches(p, pats) and not matches(p, keep)})
    for i in range(0, len(doomed), 200):
        git(repo, "rm", "-q", "-r", "-f", "--ignore-unmatch", "--", *doomed[i:i + 200])
    for p in doomed:  # a conflicted file may linger in the work tree
        fp = os.path.join(repo, p)
        if os.path.exists(fp):
            os.remove(fp)
    cargo = os.path.join(repo, "Cargo.toml")
    if os.path.exists(cargo):
        with open(cargo, encoding="utf-8", newline="") as f:
            old = f.read()
        new = strip_members(old, [p for p in pats if p.endswith("/")])
        if new != old:
            with open(cargo, "w", encoding="utf-8", newline="") as f:
                f.write(new)
            git(repo, "add", "Cargo.toml", check=False)
            print("Cargo.toml: removed pruned workspace entries")
    return doomed


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=".")
    ap.add_argument("--list", default=os.path.join(HERE, "prune-paths.txt"))
    a = ap.parse_args()
    doomed = prune(os.path.abspath(a.repo), a.list)
    print(f"prune: {len(doomed)} path(s) removed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
