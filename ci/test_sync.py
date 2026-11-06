import json
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(__file__))
import check_zenith_intact as ci
import sync
from gitutil import commit, git, new_repo, write

HOOK = "line before\nlet x = 1; // zenith: hook\nline after\n"


def setup():
    """A tiny fork: `up` is Horizon (the base), main is Zenith (prunes, a hook, an own file, merge=ours)."""
    r = new_repo()
    write(r, ".gitattributes", "README.md merge=ours\n")
    write(r, "README.md", "horizon readme\n")
    write(r, "crates/engine.rs", "fn engine() {}\n")
    write(r, "crates/hook.rs", "line before\nlet x = 1;\nline after\n")
    write(r, "crates/zenith-server/main.rs", "fn main() {}\n")
    members = "".join(f"    {n},\n" for f, n in ci.IDENTIFIERS if f == "Cargo.toml")
    write(r, "Cargo.toml", "[workspace]\nmembers = [\n" + members + '    "crates/zenith-server",\n]\n')
    needles = {}
    for f, n in ci.IDENTIFIERS:
        needles.setdefault(f, []).append(n)
    for f, ns in needles.items():
        if f not in ("Cargo.toml", ".gitattributes"):
            write(r, f, "\n".join(ns) + "\n")
    for f in ci.EXTRA_FILES:
        if f not in ("README.md", ".gitattributes"):
            write(r, f, "x\n")
    commit(r, "horizon")
    git(r, "branch", "up")
    write(r, "README.md", "zenith readme\n")
    write(r, "crates/hook.rs", HOOK)
    write(r, "zenith/own.txt", "own\n")
    write(r, "ci/zenith-manifest.json", "{}")
    commit(r, "zenith")
    git(r, "rm", "-q", "-r", "crates/zenith-server")
    commit(r, "prune")
    return r


def fake_upstream(r, **files):
    git(r, "checkout", "-q", "up")
    for p, t in files.items():
        if t is None:
            git(r, "rm", "-q", p.replace("__", "/"))
        else:
            write(r, p.replace("__", "/"), t)
    commit(r, "horizon news")
    git(r, "checkout", "-q", "main")


def refresh_manifest(r):
    m = ci.build(r, "up")
    write(r, "ci/zenith-manifest.json", json.dumps(m))
    commit(r, "manifest")


class SyncTest(unittest.TestCase):
    def go(self, r):
        self.out = os.path.join(r, "..", os.path.basename(r) + ".out")
        rc = sync.merge(r, "up", no_cargo=True, output=self.out)
        return rc, dict(l.split("=", 1) for l in open(self.out, encoding="utf-8").read().splitlines())

    def test_up_to_date_is_a_cheap_exit(self):
        r = setup()
        refresh_manifest(r)
        git(r, "merge", "-q", "--no-edit", "up")
        rc, out = self.go(r)
        self.assertEqual((rc, out["outcome"]), (0, "up-to-date"))

    def test_clean_engine_change_and_pruned_and_ours_files_are_auto_resolved(self):
        r = setup()
        refresh_manifest(r)
        fake_upstream(r, **{"crates__engine.rs": "fn engine() { new(); }\n", "crates__horizon-server__main.rs": "fn main() { x(); }\n",
                                                    "README.md": "horizon readme 2\n", "docs__new.md": "new\n"})
        rc, out = self.go(r)
        self.assertEqual((rc, out["outcome"]), (0, "merged"))
        self.assertEqual(len(git(r, "show", "-s", "--format=%P", "HEAD").stdout.split()), 2)  # a real merge commit
        files = git(r, "ls-files").stdout.split()
        self.assertNotIn("crates/zenith-server/main.rs", files)
        self.assertIn("docs/new.md", files)
        self.assertEqual(open(os.path.join(r, "README.md")).read(), "zenith readme\n")
        self.assertNotIn("zenith-server", open(os.path.join(r, "Cargo.toml")).read())

    def test_conflict_around_a_hook_blocks_and_leaves_main_alone(self):
        r = setup()
        refresh_manifest(r)
        before = git(r, "rev-parse", "HEAD").stdout
        fake_upstream(r, **{"crates__hook.rs": "line before\nlet x = 2;\nline after\n"})
        rc, out = self.go(r)
        self.assertEqual((rc, out["outcome"], out["reason"]), (sync.BLOCKED, "blocked", "conflict"))
        self.assertIn("crates/hook.rs", out["conflicts"])
        self.assertEqual(git(r, "rev-parse", "HEAD").stdout, before)
        self.assertEqual(git(r, "status", "--porcelain").stdout.strip(), "")

    def test_clean_merge_that_drops_a_marker_is_blocked_by_the_intact_check(self):
        r = setup()
        refresh_manifest(r)
        git(r, "checkout", "-q", "-b", "up2")  # Horizon "adopts" our tree and then removes our hook cleanly
        write(r, "crates/hook.rs", "line before\nline after\n")
        commit(r, "drop hook")
        git(r, "checkout", "-q", "main")
        before = git(r, "rev-parse", "HEAD").stdout
        rc = sync.merge(r, "up2", no_cargo=True)
        self.assertEqual(rc, sync.BLOCKED)
        self.assertEqual(git(r, "rev-parse", "HEAD").stdout, before)

    def test_dirty_worktree_is_refused_and_left_alone(self):
        r = setup()
        refresh_manifest(r)
        fake_upstream(r, **{"crates__engine.rs": "fn engine() { new(); }\n"})
        write(r, "crates/engine.rs", "fn engine() { local(); }\n")
        rc, out = self.go(r)
        self.assertEqual((rc, out["reason"]), (sync.BLOCKED, "dirty-worktree"))
        self.assertIn("local()", open(os.path.join(r, "crates/engine.rs")).read())

    def test_release_decision(self):
        r = setup()
        up = git(r, "rev-parse", "up").stdout.strip()
        git(r, "tag", "nightly-source/2026-11-05-aaaaaaa", "up")
        pick = json.dumps({"tag": "nightly-source/2026-11-05-aaaaaaa", "release": "nightly"})
        self.assertTrue(sync.covered(r, up, "nightly-source/2026-11-05-aaaaaaa"))
        fake_upstream(r, **{"crates__engine.rs": "fn engine2() {}\n"})
        newer = git(r, "rev-parse", "up").stdout.strip()
        self.assertFalse(sync.covered(r, newer, "nightly-source/2026-11-05-aaaaaaa"))
        self.assertEqual(sync.release(r, newer, pick, True), 0)  # pending: prints, dispatches nothing


if __name__ == "__main__":
    unittest.main()
