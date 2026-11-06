#!/usr/bin/env python3
"""The logic of .github/workflows/zenith-sync.yml, runnable locally.

Zenith is a hard fork (other crate, binary and folder names), so Horizon's work arrives as translated patches
(ci/port_upstream.py), never as a git merge.

  sync.py port    --upstream-ref REF [--output FILE] [--no-cargo]    translate and apply Horizon's new commits (port_upstream.py)
  sync.py checks  [--no-cargo]        the strict gate: build + tests; prints failed=<step> and exits 1 on failure
  sync.py land    [--remote URL]      push HEAD to main (a normal push, never forced); exit 11 when main moved meanwhile
  sync.py report  --reason R --commit SHA [--conflicts LIST] [--dry-run]
        Pushes SHA as branch sync/<date>-<short>, opens or updates its PR and opens or updates ONE issue labelled `sync`
  sync.py resolved [--dry-run]        closes the open `sync` issue and the old sync/* pull requests
  sync.py release --upstream SHA [--pick-json J] [--dry-run]
        Dispatches zenith-release only when SHA is covered by the newest Horizon nightly; else says it is pending
  sync.py run     --upstream-ref REF [--remote URL] [--no-cargo] [--dry-run]
        port -> checks -> land (all green) or report (anything else): the whole weekly job in one command
"""
import argparse
import datetime
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import port_upstream  # noqa: E402

BLOCKED = 10
MOVED = 11  # main moved while the checks ran: nothing is lost, the next run starts again from the new main
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


def steps(no_cargo):
    py = sys.executable
    s = [("ci unit tests", [py, "-m", "unittest", "discover", "-s", "ci"]),
         ("check_public_tree", [py, "tools/check_public_tree.py"]),
         ("check_links", [py, "ci/check_links.py"]),
         ("check_zenith_intact", [py, "ci/check_zenith_intact.py"])]
    if not no_cargo:
        bridge = ["--manifest-path", "bridge/Cargo.toml"]
        exe = ".exe" if os.name == "nt" else ""
        s += [("cargo fmt", ["cargo", "fmt", "--all", "--check"]),
              # the strict part of the gate: every crate and every test target must compile (nothing that fails to build lands)
              ("cargo check workspace", ["cargo", "check", "--workspace", "--tests", "--locked"]),
              ("cargo test protocol", ["cargo", "test", "-p", "zenith-protocol", "--locked"]),
              ("bridge cargo fmt", ["cargo", "fmt", *bridge, "--check"]),
              ("bridge cargo clippy", ["cargo", "clippy", *bridge, "--all-targets", "--locked", "--", "-D", "warnings"]),
              ("bridge cargo test", ["cargo", "test", *bridge, "--locked"]),
              ("cargo build client", ["cargo", "build", "-p", "zenith-client", "--locked"]),
              ("launch API check", [py, "ci/check_launch_api.py", f"target/debug/zenith{exe}"])]
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


def report(repo, reason, commit, conflicts, dry, remote=None):
    short = os.environ.get("UPSTREAM_SHA", commit)[:7]
    branch = f"sync/{today()}-{short}"
    assert branch.startswith("sync/")  # the only branch this script ever force-pushes
    gh = Gh(dry)
    what = ("what applied of the translated patch, with `*.rej` files for what did not: finish it by hand" if reason == "conflict"
            else "the port that failed the gate")
    lines = [f"Porting Horizon's `main` ({short}) into Zenith's `main` was blocked: **{reason}**.", ""]
    if conflicts:
        lines += ["Details:", ""] + [f"- `{c}`" for c in conflicts] + [""]
    lines += [f"Branch `{branch}` holds {what}.",
              "`main` is untouched and stays behind Horizon until this is resolved (docs/ZENITH.md, section Sync with Horizon).", ""]
    tmp = os.path.join(os.environ.get("RUNNER_TEMP", os.environ.get("TEMP", "/tmp")), "sync-body.md")
    with open(tmp, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    plain = remote or f"https://github.com/{os.environ.get('GITHUB_REPOSITORY', '')}.git"
    if dry and not remote:
        print(f"DRY-RUN: git push -f {plain} {commit}:refs/heads/{branch}")
    else:
        token = os.environ.get("GH_TOKEN") or ""
        url = plain.replace("https://", f"https://x-access-token:{token}@") if token and plain.startswith("https://") else plain
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
    # The PR above already carries the report; a repository with Issues turned off must not hide it, so the issue
    # is best effort (the run still ends red).
    issues = json.loads(gh("issue", "list", "--label", LABEL, "--state", "open", "--json", "number,title", check=False) or "[]")
    mine = [i for i in issues if i["title"] == ISSUE_TITLE]
    if mine:
        done = gh("issue", "comment", str(mine[0]["number"]), "--body-file", tmp, check=False)
    else:
        done = gh("issue", "create", "--title", ISSUE_TITLE, "--label", LABEL, "--body-file", tmp, check=False)
    if not dry and not done:
        print("::warning::could not open or update the sync issue (are Issues enabled?); see the pull request")
    return 0


def resolved(dry):
    gh = Gh(dry)
    issues = json.loads(gh("issue", "list", "--label", LABEL, "--state", "open", "--json", "number,title", check=False) or "[]")
    for i in issues:
        if i["title"] == ISSUE_TITLE:
            gh("issue", "close", str(i["number"]), "--comment", "Sync is current again: closing.", check=False)
    prs = json.loads(gh("pr", "list", "--state", "open", "--json", "number,headRefName", check=False) or "[]")
    for p in prs:  # attempts of earlier blocked runs: main now contains Horizon's tip, so they are obsolete
        if p["headRefName"].startswith("sync/"):
            gh("pr", "close", str(p["number"]), "--delete-branch", "--comment", "Sync is current again: closing.", check=False)
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


def land(repo, remote=None):
    """Pushes HEAD to main with a normal push. Refuses a dirty tree and a HEAD that does not contain the remote main (so it can
    never overwrite anything); MOVED when main advanced meanwhile."""
    url = remote or f"https://github.com/{os.environ.get('GITHUB_REPOSITORY', '')}.git"
    token = os.environ.get("GH_TOKEN") or ""
    if token and url.startswith("https://"):
        url = url.replace("https://", f"https://x-access-token:{token}@")
    if git(repo, "status", "--porcelain", "--untracked-files=no").stdout.strip():
        print("land refused: uncommitted changes")
        return BLOCKED
    git(repo, "fetch", "-q", url, "main")
    if git(repo, "merge-base", "--is-ancestor", "FETCH_HEAD", "HEAD", check=False).returncode != 0:
        print("land: main moved while the checks ran; the next scheduled run starts again from the new main")
        return MOVED
    pushed = git(repo, "push", url, "HEAD:refs/heads/main", check=False)  # no --force, ever
    if pushed.returncode != 0:
        print(f"land: push refused ({pushed.stderr.strip()[:200]}); the next scheduled run retries")
        return MOVED
    print(f"landed {rev(repo, 'HEAD')[:7]} on main")
    return 0


def run_all(repo, ref, remote=None, no_cargo=False, dry=False, gate=None):
    """The whole job: port, then the strict gate; all green lands on main and closes the sync issue/PRs; anything else reports."""
    import tempfile
    out = os.path.join(tempfile.mkdtemp(prefix="sync"), "out")
    open(out, "w").close()
    port_upstream.port(repo, ref, no_cargo=no_cargo, output=out)
    with open(out, encoding="utf-8") as f:
        got = dict(line.split("=", 1) for line in f.read().splitlines() if "=" in line)
    if got["outcome"] == "up-to-date":
        return 0
    if got["outcome"] == "blocked":
        report(repo, got["reason"], got["commit"], [c for c in got.get("conflicts", "").replace(" | ", ",").split(",") if c], dry, remote)
        return 1
    ported = got["commit"]
    if (gate or checks)(repo, no_cargo) != 0:
        report(repo, "checks", ported, ["the strict gate failed (see the run log)"], dry, remote)
        return 1
    rc = land(repo, remote)
    if rc == 0:
        resolved(dry)
    return 0 if rc in (0, MOVED) else 1


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    m = sub.add_parser("port")
    m.add_argument("--upstream-ref", default="upstream/main")
    m.add_argument("--output")
    m.add_argument("--no-cargo", action="store_true")
    ld = sub.add_parser("land")
    ld.add_argument("--remote")
    ru = sub.add_parser("run")
    ru.add_argument("--upstream-ref", default="upstream/main")
    ru.add_argument("--remote")
    ru.add_argument("--no-cargo", action="store_true")
    ru.add_argument("--dry-run", action="store_true")
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
    for p in (m, c, r, s, e, ld, ru):
        p.add_argument("--repo", default=".")
    a = ap.parse_args()
    repo = os.path.abspath(a.repo)
    if a.cmd == "port":
        return port_upstream.port(repo, a.upstream_ref, None, a.no_cargo, a.output)
    if a.cmd == "land":
        return land(repo, a.remote)
    if a.cmd == "run":
        return run_all(repo, a.upstream_ref, a.remote, a.no_cargo, a.dry_run)
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
