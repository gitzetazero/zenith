#!/usr/bin/env python3
"""Checks that every relative link and image in Zenith's own markdown files points at a file that exists.
Usage: check_links.py [--repo DIR]. Exit 1 listing each broken link."""
import argparse
import glob
import os
import re
import sys

OWN = ["README.md", "docs/ZENITH.md", "CONTRIBUTING.md", "SECURITY.md", "CODE_OF_CONDUCT.md",
       "bridge/**/*.md", "launcher/**/*.md", "branding/**/*.md", "docs/PLAYER-GUIDE.md", "docs/SERVERS.md", "docs/GRAPHICS-SETTINGS.md", ".github/ISSUE_TEMPLATE/*.md", ".github/PULL_REQUEST_TEMPLATE/*.md",
       "docs/superpowers/**/*.md"]
LINK = re.compile(r'\]\(\s*<?([^)\s>]+)|<img[^>]*\ssrc="([^"]+)"|<a[^>]*\shref="([^"]+)"')
SKIP = ("http://", "https://", "mailto:", "#", "data:")
IGNORED_DIRS = ("node_modules", "target")


def files(repo):
    seen = set()
    for pat in OWN:
        for f in glob.glob(os.path.join(repo, pat), recursive=True):
            rel = os.path.relpath(f, repo)
            if not any(d in rel.split(os.sep) for d in IGNORED_DIRS) and os.path.isfile(f):
                seen.add(rel)
    return sorted(seen)


def broken(repo):
    bad = []
    for rel in files(repo):
        fenced = False
        for n, line in enumerate(open(os.path.join(repo, rel), encoding="utf-8", errors="replace"), 1):
            if line.lstrip().startswith("```"):
                fenced = not fenced
            if fenced:
                continue
            for m in LINK.finditer(line):
                target = next(g for g in m.groups() if g)
                if target.startswith(SKIP):
                    continue
                path = target.split("#")[0].split("?")[0]
                if not path:
                    continue
                base = repo if path.startswith("/") else os.path.dirname(os.path.join(repo, rel))
                if not os.path.exists(os.path.normpath(os.path.join(base, path.lstrip("/")))):
                    bad.append(f"{rel}:{n}: {target}")
    return bad


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=".")
    bad = broken(os.path.abspath(ap.parse_args().repo))
    for b in bad:
        print("broken link:", b)
    if not bad:
        print("links ok")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
