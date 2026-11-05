#!/usr/bin/env python3
"""The logic of .github/workflows/zenith-sync.yml, runnable locally.

  sync.py merge   --upstream-ref REF [--output FILE] [--no-cargo]
        Merges REF into the current branch with a real merge commit (never a rebase). Prunes (prune.py), lets
        merge=ours and Cargo.toml/Cargo.lock be resolved, then runs the intact check. Prints key=value lines
        (also appended to --output): outcome=up-to-date|merged|blocked, reason, commit, upstream, short, conflicts.
        Exit 0 for up-to-date and merged, 10 for blocked. A blocked run leaves the branch as it was.
  sync.py checks  [--no-cargo]                     the build/test gate; prints failed=<step> and exits 1 on failure
  sync.py report  --reason R --commit SHA [--conflicts LIST] [--dry-run]
        Pushes SHA as branch sync/<date>-<short>, opens or updates its PR and opens or updates ONE issue labelled `sync`
  sync.py resolved [--dry-run]                     closes the open `sync` issue after a successful sync
  sync.py release --upstream SHA [--pick-json J] [--dry-run]
        Dispatches zenith-release only when SHA is covered by the newest Horizon nightly; else says it is pending
"""
import argparse
import datetime
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import check_zenith_intact  # noqa: E402
import prune  # noqa: E402

BLOCKED = 10
ISSUE_TITLE = "Sync blocked: conflict with upstream"
LABEL = "sync"


def run(cmd, cwd=".", check=True):
    return subprocess.run(cmd, cwd=cwd, check=check, capture_output=True, text=True, encoding="utf-8")


def git(repo, *args, check=True):
    return run(["git", "-c", "core.autocrlf=false", *args], cwd=repo, check=check)


def rev(repo, ref):
    return git(repo, "rev-parse", ref).stdout.strip()


def emit(pairs, output):
    for k, v in pairs.items():
        line = f"{k}={v}"
        print(line)
        if output:
            with open(output, "a", encoding="utf-8", newline="\n") as f:
                f.write(line + "\n")


def unmerged(repo):
    return sorted(set(git(repo, "diff", "--name-only", "--diff-filter=U").stdout.split("\n")) - {""})


def resolve_generated(repo):
    """Cargo.toml: take Horizon's (prune takes our crates out again). Cargo.lock: take Horizon's (cargo rewrites it)."""
    for f in ("Cargo.toml", "Cargo.lock"):
        if f in unmerged(repo):
            git(repo, "checkout", "--theirs", "--", f)
            git(repo, "add", f)


def regenerate_lock(repo):
    run(["cargo", "metadata", "--format-version", "1"], cwd=repo)  # no --locked: drops what pruning removed
    git(repo, "add", "Cargo.lock")


def merge(repo, ref, no_cargo=False, output=None):
    upstream = rev(repo, ref)
    base = {"upstream": upstream, "short": upstream[:7]}
    if git(repo, "merge-base", "--is-ancestor", upstream, "HEAD", check=False).returncode == 0:
        emit({"outcome": "up-to-date", **base}, output)
        return 0
    head = rev(repo, "HEAD")
    if git(repo, "status", "--porcelain", "--untracked-files=no").stdout.strip():
        # the blocked paths below `reset --hard`: never run on a tree with work in it
        emit({"outcome": "blocked", "reason": "dirty-worktree", "commit": upstream, "conflicts": "uncommitted changes", **base}, output)
        return BLOCKED
    git(repo, "config", "merge.ours.driver", "true")  # README.md & co: merge=ours in .gitattributes
    git(repo, "config", "user.name", "zenith-sync")
    git(repo, "config", "user.email", "zenith-sync@users.noreply.github.com")
    started = git(repo, "merge", "--no-commit", "--no-ff", upstream, check=False)
    if git(repo, "rev-parse", "-q", "--verify", "MERGE_HEAD", check=False).returncode != 0:
        # git refused to start the merge (dirty tree, no identity...): never commit the tree as a fake "merge"
        git(repo, "reset", "--hard", head)
        why = (started.stderr or started.stdout).strip().replace(chr(10), " ")[:300]
        emit({"outcome": "blocked", "reason": "merge-not-started", "commit": upstream, "conflicts": why, **base}, output)
        return BLOCKED
    resolve_generated(repo)
    prune.prune(repo, os.path.join(HERE, "prune-paths.txt"))  # also removes pruned files the merge brought in
    left = unmerged(repo)
    if left:
        git(repo, "merge", "--abort", check=False)
        git(repo, "reset", "--hard", head)
        emit({"outcome": "blocked", "reason": "conflict", "commit": upstream, "conflicts": ",".join(left), **base}, output)
        return BLOCKED
    if not no_cargo:
        regenerate_lock(repo)
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", f"Merge horizon-rust/horizon main ({base['short']}) into main")
    merged = rev(repo, "HEAD")
    if git(repo, "merge-base", "--is-ancestor", upstream, merged, check=False).returncode != 0:
        git(repo, "reset", "--hard", head)
        emit({"outcome": "blocked", "reason": "upstream-not-merged", "commit": merged, **base}, output)
        return BLOCKED
    with open(os.path.join(repo, "zenith/ci/zenith-manifest.json"), encoding="utf-8") as f:
        bad = check_zenith_intact.check(repo, json.load(f))
    if bad:
        git(repo, "reset", "--hard", head)
        emit({"outcome": "blocked", "reason": "intact", "commit": merged, "conflicts": " | ".join(bad), **base}, output)
        return BLOCKED
    emit({"outcome": "merged", "commit": merged, **base}, output)
    return 0


def steps(no_cargo):
    py = sys.executable
    s = [("zenith/ci unit tests", [py, "-m", "unittest", "discover", "-s", "zenith/ci"]),
         ("check_public_tree", [py, "tools/check_public_tree.py"]),
         ("check_links", [py, "zenith/ci/check_links.py"]),
         ("check_zenith_intact", [py, "zenith/ci/check_zenith_intact.py"])]
    if not no_cargo:
        bridge = ["--manifest-path", "zenith/bridge/Cargo.toml"]
        exe = ".exe" if os.name == "nt" else ""
        s += [("cargo fmt", ["cargo", "fmt", "--all", "--check"]),
              ("cargo clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"]),
              ("cargo test protocol", ["cargo", "test", "-p", "horizon-protocol", "--locked"]),
              ("bridge cargo fmt", ["cargo", "fmt", *bridge, "--check"]),
              ("bridge cargo clippy", ["cargo", "clippy", *bridge, "--all-targets", "--locked", "--", "-D", "warnings"]),
              ("bridge cargo test", ["cargo", "test", *bridge, "--locked"]),
              ("cargo build client", ["cargo", "build", "-p", "horizon-client", "--locked"]),
              ("launch API check", [py, "zenith/ci/check_launch_api.py", f"target/debug/horizon-client{exe}"])]
    return s


def checks(repo, no_cargo):
    for name, cmd in steps(no_cargo):
        print(f"::group::{name}", flush=True)
        r = subprocess.run(cmd, cwd=repo)
        print("::endgroup::", flush=True)
        if r.returncode != 0:
            print(f"failed={name}")
            return 1
    print("failed=")
    return 0


def today():
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")


class Gh:
    """gh with the sync PAT first and GITHUB_TOKEN as a fallback (the PAT may lack issues/PR scope)."""

    def __init__(self, dry):
        self.dry = dry
        self.tokens = [t for t in (os.environ.get("GH_TOKEN"), os.environ.get("GITHUB_TOKEN")) if t] or [None]

    def __call__(self, *args, check=True):
        cmd = ["gh", *args]
        if self.dry:
            print("DRY-RUN:", " ".join(cmd))
            return ""
        last = None
        for t in self.tokens:
            env = dict(os.environ, **({"GH_TOKEN": t} if t else {}))
            last = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", env=env)
            if last.returncode == 0:
                return last.stdout
        if check:
            raise SystemExit(f"gh {' '.join(args[:3])} failed: {last.stderr.strip()}")
        return ""


def report(repo, reason, commit, conflicts, dry):
    short = os.environ.get("UPSTREAM_SHA", commit)[:7]
    branch = f"sync/{today()}-{short}"
    assert branch.startswith("sync/")  # the only branch this script ever force-pushes
    gh = Gh(dry)
    what = ("Horizon's main as it is: merge it into main and resolve the conflicts" if reason == "conflict"
            else "the merge that failed")
    lines = [f"Merging Horizon's `main` ({short}) into Zenith's `main` was blocked: **{reason}**.", ""]
    if conflicts:
        lines += ["Details:", ""] + [f"- `{c}`" for c in conflicts] + [""]
    lines += [f"Branch `{branch}` holds {what}.",
              "`main` is untouched and stays behind Horizon until this is resolved (ZENITH.md, section Sync).", ""]
    tmp = os.path.join(os.environ.get("RUNNER_TEMP", os.environ.get("TEMP", "/tmp")), "sync-body.md")
    with open(tmp, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    plain = f"https://github.com/{os.environ.get('GITHUB_REPOSITORY', '')}.git"
    if dry:
        print(f"DRY-RUN: git push -f {plain} {commit}:refs/heads/{branch}")
    else:
        token = os.environ.get("GH_TOKEN") or ""
        url = plain.replace("https://", f"https://x-access-token:{token}@") if token else plain
        git(repo, "push", "-f", url, f"{commit}:refs/heads/{branch}")
    title = f"sync: Horizon main {short} ({reason})"
    prs = json.loads(gh("pr", "list", "--head", branch, "--state", "open", "--json", "number") or "[]")
    if prs:
        gh("pr", "edit", str(prs[0]["number"]), "--title", title, "--body-file", tmp)
    else:
        gh("pr", "create", "--base", "main", "--head", branch, "--title", title, "--body-file", tmp)
    labels = json.loads(gh("label", "list", "--json", "name", "--limit", "200") or "[]")
    if not dry and LABEL not in [x["name"] for x in labels]:
        gh("label", "create", LABEL, "--color", "d93f0b", "--description", "Zenith/Horizon sync is blocked")
    issues = json.loads(gh("issue", "list", "--label", LABEL, "--state", "open", "--json", "number,title") or "[]")
    mine = [i for i in issues if i["title"] == ISSUE_TITLE]
    if mine:
        gh("issue", "comment", str(mine[0]["number"]), "--body-file", tmp)
    else:
        gh("issue", "create", "--title", ISSUE_TITLE, "--label", LABEL, "--body-file", tmp)
    return 0


def resolved(dry):
    gh = Gh(dry)
    issues = json.loads(gh("issue", "list", "--label", LABEL, "--state", "open", "--json", "number,title") or "[]")
    for i in issues:
        if i["title"] == ISSUE_TITLE:
            gh("issue", "close", str(i["number"]), "--comment", "Sync is current again: closing.")
    return 0


def covered(repo, upstream, tag):
    return git(repo, "merge-base", "--is-ancestor", upstream, f"{tag}^{{commit}}", check=False).returncode == 0


def release(repo, upstream, pick_json, dry):
    if pick_json:
        pick = json.loads(pick_json)
    else:
        pick = json.loads(run([sys.executable, os.path.join(HERE, "pick_nightly.py"), "--ref", upstream], cwd=repo).stdout)
    if not pick:
        print("release pending: no Horizon nightly found")
        return 0
    tag, rel = pick["tag"], pick["release"]
    if not covered(repo, upstream, tag):
        print(f"release pending: waiting for the next Horizon nightly ({upstream[:7]} is newer than {tag})")
        return 0
    day = today()
    if not dry and subprocess.run(["gh", "release", "view", f"nightly-{day}"], capture_output=True).returncode == 0:
        print(f"release not needed: nightly-{day} is already published")
        return 0
    print(f"release: {upstream[:7]} is covered by {tag}; dispatching zenith-release (exporters from {rel})")
    Gh(dry)("workflow", "run", "zenith-release.yml", "-f", f"release={rel}", "-f", f"date={day}")
    return 0


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    m = sub.add_parser("merge")
    m.add_argument("--upstream-ref", default="upstream/main")
    m.add_argument("--output")
    m.add_argument("--no-cargo", action="store_true")
    c = sub.add_parser("checks")
    c.add_argument("--no-cargo", action="store_true")
    r = sub.add_parser("report")
    r.add_argument("--reason", required=True)
    r.add_argument("--commit", required=True)
    r.add_argument("--conflicts", default="")
    r.add_argument("--dry-run", action="store_true")
    s = sub.add_parser("resolved")
    s.add_argument("--dry-run", action="store_true")
    e = sub.add_parser("release")
    e.add_argument("--upstream", required=True)
    e.add_argument("--pick-json")
    e.add_argument("--dry-run", action="store_true")
    for p in (m, c, r, s, e):
        p.add_argument("--repo", default=".")
    a = ap.parse_args()
    repo = os.path.abspath(a.repo)
    if a.cmd == "merge":
        return merge(repo, a.upstream_ref, a.no_cargo, a.output)
    if a.cmd == "checks":
        return checks(repo, a.no_cargo)
    if a.cmd == "report":
        conflicts = [x for x in a.conflicts.replace(" | ", ",").split(",") if x]
        return report(repo, a.reason, a.commit, conflicts, a.dry_run)
    if a.cmd == "resolved":
        return resolved(a.dry_run)
    return release(repo, a.upstream, a.pick_json, a.dry_run)


if __name__ == "__main__":
    sys.exit(main())
