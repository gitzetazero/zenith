#!/usr/bin/env python3
"""Ports Horizon's new commits into Zenith's renamed tree (Zenith is a hard fork: crates, binaries and folders have other names).

  port_upstream.py --upstream-ref upstream/main [--since SHA] [--output FILE] [--no-cargo]

Takes the upstream changes between the last synced SHA (ci/upstream-synced.txt, or --since) and the ref, as one diff, and
  - drops what Zenith does not carry (ci/prune-paths.txt, in Horizon's names), the files Zenith owns although Horizon has them
    (`merge=ours` in .gitattributes), our own folders and Cargo.lock (regenerated);
  - rewrites paths and text with ci/rename_map.py (`horizon_core` -> `zenith_core`, `mods/horizon` -> `java/horizon`, ...);
  - applies it to the work tree, records the new SHA in ci/upstream-synced.txt, regenerates Cargo.lock and commits.
Prints key=value lines (also appended to --output): outcome=up-to-date|ported|blocked, reason, commit, upstream, short, conflicts.
Exit 0 for up-to-date and ported, 10 for blocked (the tree is left as it was). Never pushes: ci/sync.py decides what happens next.
"""
import argparse
import fnmatch
import json
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import check_zenith_intact  # noqa: E402
import rename_map  # noqa: E402

BLOCKED = 10
SYNCED = "ci/upstream-synced.txt"
OWN_DIRS = ("launcher/", "bridge/", "branding/", "ci/")  # Zenith's folders: Horizon has other things under some of these names
GENERATED = ("Cargo.lock",)


def git_bytes(repo, *args, check=True, input=None):
    return subprocess.run(["git", "-c", "core.autocrlf=false", *args], cwd=repo, check=check, capture_output=True, input=input)


def git(repo, *args, check=True):
    r = git_bytes(repo, *args, check=check)
    return subprocess.CompletedProcess(r.args, r.returncode, r.stdout.decode("utf-8", "replace"), r.stderr.decode("utf-8", "replace"))


def rev(repo, ref):
    return git(repo, "rev-parse", ref).stdout.strip()


def emit(pairs, output):
    for k, v in pairs.items():
        line = f"{k}={v}"
        print(line)
        if output:
            with open(output, "a", encoding="utf-8", newline="\n") as f:
                f.write(line + "\n")


def parse_list(text):
    prune, keep = [], []
    for line in text.splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            (keep if line.startswith("!") else prune).append(line.lstrip("!"))
    return prune, keep


def matches(path, pats):
    return any(path.startswith(p) if p.endswith("/") else fnmatch.fnmatchcase(path, p) for p in pats)


def ours_files(repo):
    """Upstream paths that are Zenith's own files (merge=ours in .gitattributes)."""
    p = os.path.join(repo, ".gitattributes")
    if not os.path.isfile(p):
        return set()
    out = set()
    with open(p, encoding="utf-8") as f:
        for line in f:
            parts = line.split()
            if len(parts) >= 2 and "merge=ours" in parts[1:]:
                out.add(parts[0])
    return out


def last_synced(repo):
    with open(os.path.join(repo, SYNCED), encoding="utf-8") as f:
        for line in f:
            if line.strip() and not line.startswith("#"):
                return line.split()[0]
    raise SystemExit(f"{SYNCED} has no SHA")


def dropped(path, pats, keep, ours):
    mapped = rename_map.map_path(path)
    return (matches(path, pats) and not matches(path, keep)) or path in ours or path in GENERATED \
        or mapped.startswith(OWN_DIRS) or mapped == SYNCED


def translate(diff, pats, keep, ours):
    """(patch, kept_files, dropped_files): `diff` (bytes, `git diff --binary -M`) in Zenith's names, without what is not carried."""
    text = diff.decode("utf-8", "surrogateescape")
    out, kept, skipped = [], [], []
    for sec in re.split(r"(?m)^(?=diff --git )", text):
        if not sec.strip():
            continue
        head = sec.split("\n", 1)[0]
        m = re.match(r"diff --git a/(.*) b/(.*)$", head)
        if not m:
            raise ValueError(f"unexpected diff header: {head!r}")
        old, new = m.group(1), m.group(2)
        if dropped(old, pats, keep, ours) or dropped(new, pats, keep, ours):
            skipped.append(new)
            continue
        binary = "GIT binary patch" in sec or re.search(r"(?m)^Binary files ", sec)
        lines, hunk = [], False
        for line in sec.split("\n"):
            if line.startswith("index ") and not hunk:
                continue  # the blob ids are Horizon's; the rewritten text has other ones
            if line.startswith("diff --git "):
                line = f"diff --git a/{rename_map.map_path(old)} b/{rename_map.map_path(new)}"
            elif not hunk and line.startswith(("rename from ", "rename to ", "copy from ", "copy to ")):
                first, second, path = line.split(" ", 2)
                line = f"{first} {second} {rename_map.map_path(path)}"
            elif not hunk and line.startswith(("--- a/", "+++ b/")):
                line = line[:6] + rename_map.map_path(line[6:])
            elif line.startswith("@@"):
                hunk = True
            elif hunk and not binary and line[:1] in ("+", "-", " "):
                line = line[0] + rename_map.rewrite(line[1:])
            lines.append(line)
        out.append("\n".join(lines))
        kept.append(rename_map.map_path(new))
    return "".join(out).encode("utf-8", "surrogateescape"), kept, skipped


def regenerate_lock(repo):
    subprocess.run(["cargo", "metadata", "--format-version", "1"], cwd=repo, check=True, capture_output=True)  # new deps resolved here
    git(repo, "add", "Cargo.lock")


def with_rejects(repo, patch, last, up):
    """A commit with what applies of `patch` plus .rej files for the rest, for the person who resolves the block (the tree is not changed)."""
    head = rev(repo, "HEAD")
    git_bytes(repo, "apply", "--index", "--recount", "--whitespace=nowarn", "--reject", "-", check=False, input=patch)
    git(repo, "add", "-A")
    git(repo, "-c", "user.name=zenith-sync", "-c", "user.email=zenith-sync@users.noreply.github.com", "commit", "-q", "--allow-empty",
        "-m", f"Partial port of horizon-rust/horizon {last[:7]}..{up[:7]}: *.rej files hold what did not apply")
    wip = rev(repo, "HEAD")
    git(repo, "reset", "-q", "--hard", head)
    git(repo, "clean", "-q", "-fd")
    return wip


def port(repo, ref, since=None, no_cargo=False, output=None):
    up = rev(repo, ref)
    base = {"upstream": up, "short": up[:7]}
    last = since or last_synced(repo)
    if up == last:
        emit({"outcome": "up-to-date", **base}, output)
        return 0
    head = rev(repo, "HEAD")
    if git(repo, "status", "--porcelain", "--untracked-files=no").stdout.strip():
        emit({"outcome": "blocked", "reason": "dirty-worktree", "commit": head, "conflicts": "uncommitted changes", **base}, output)
        return BLOCKED

    def blocked(reason, why, commit=head):
        git(repo, "reset", "-q", "--hard", head)
        git(repo, "clean", "-q", "-fd")
        emit({"outcome": "blocked", "reason": reason, "commit": commit, "conflicts": why, **base}, output)
        return BLOCKED

    with open(os.path.join(HERE, "prune-paths.txt"), encoding="utf-8") as f:
        pats, keep = parse_list(f.read())
    diff = git_bytes(repo, "diff", "--binary", "-M", "--no-color", "--no-ext-diff", last, up).stdout
    patch, kept, skipped = translate(diff, pats, keep, ours_files(repo))
    print(f"port: {len(kept)} file(s) to apply, {len(skipped)} dropped (not carried or Zenith's own)")
    if kept:
        done = git_bytes(repo, "apply", "--index", "--recount", "--whitespace=nowarn", "-", check=False, input=patch)
        if done.returncode != 0:
            err = done.stderr.decode("utf-8", "replace")
            files = sorted(set(re.findall(r"(?m)^error: (?:patch failed: )?([^:\n]+?)(?::\d+)?(?::| already| does)", err)))
            return blocked("conflict", ",".join(files) or err.strip().replace("\n", " ")[:300],
                           commit=with_rejects(repo, patch, last, up))
    with open(os.path.join(repo, SYNCED), "w", encoding="utf-8", newline="\n") as f:
        f.write(f"{up}\n# the last horizon-rust/horizon commit ported into this tree (ci/port_upstream.py)\n")
    git(repo, "add", SYNCED)
    if kept and not no_cargo:
        try:
            regenerate_lock(repo)
        except subprocess.CalledProcessError as e:
            return blocked("cargo-metadata", (e.stderr or b"").decode("utf-8", "replace").strip().replace("\n", " ")[:300])
    git(repo, "-c", "user.name=zenith-sync", "-c", "user.email=zenith-sync@users.noreply.github.com", "commit", "-q", "-m",
        f"Port horizon-rust/horizon main {last[:7]}..{up[:7]} (renamed to Zenith's names)\n\n"
        f"{len(kept)} file(s) applied, {len(skipped)} dropped (not carried or Zenith's own).")
    ported = rev(repo, "HEAD")
    manifest = os.path.join(repo, "ci/zenith-manifest.json")
    if os.path.isfile(manifest):
        with open(manifest, encoding="utf-8") as f:
            bad = check_zenith_intact.check(repo, json.load(f))
        if bad:
            return blocked("intact", " | ".join(bad), commit=ported)
    emit({"outcome": "ported", "commit": ported, **base}, output)
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=".")
    ap.add_argument("--upstream-ref", default="upstream/main")
    ap.add_argument("--since")
    ap.add_argument("--output")
    ap.add_argument("--no-cargo", action="store_true")
    a = ap.parse_args()
    return port(os.path.abspath(a.repo), a.upstream_ref, a.since, a.no_cargo, a.output)


if __name__ == "__main__":
    sys.exit(main())
