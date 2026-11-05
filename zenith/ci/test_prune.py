import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(__file__))
import prune
from gitutil import commit, git, new_repo, write

LIST = os.path.join(os.path.dirname(__file__), "prune-paths.txt")


class PruneTest(unittest.TestCase):
    def repo(self):
        r = new_repo()
        write(r, "crates/horizon-server/src/main.rs", "fn main() {}\n")
        write(r, "benchmarks/a/b.json", "{}\n")
        write(r, "docs/hosting.md", "host\n")
        write(r, "docs/keep.md", "keep\n")
        write(r, ".github/workflows/ci.yml", "a\n")
        write(r, ".github/workflows/zenith-sync.yml", "z\n")
        write(r, "Cargo.toml", '[workspace]\nexclude = ["launcher", "third_party/naga"]\nmembers = [\n    "crates/horizon-client",\n'
                               '    "crates/horizon-server",\n    "xtask",\n]\n')
        commit(r)
        return r

    def test_removes_pruned_and_keeps_the_rest_idempotently(self):
        r = self.repo()
        prune.prune(r, LIST)
        left = git(r, "ls-files").stdout.split()
        self.assertEqual(sorted(left), [".github/workflows/zenith-sync.yml", "Cargo.toml", "docs/keep.md"])
        self.assertEqual(prune.prune(r, LIST), [])  # second run: nothing to do
        toml = open(os.path.join(r, "Cargo.toml")).read()
        self.assertNotIn("horizon-server", toml)
        self.assertNotIn('"launcher"', toml)
        self.assertIn('"third_party/naga"', toml)
        self.assertIn('"crates/horizon-client"', toml)

    def test_upstream_modifies_a_pruned_file_then_merge_conflict_is_resolved(self):
        r = self.repo()
        git(r, "checkout", "-q", "-b", "up")
        write(r, "crates/horizon-server/src/main.rs", "fn main() { println!(); }\n")
        write(r, ".github/workflows/ci.yml", "b\n")
        write(r, "docs/new.md", "new\n")
        commit(r, "upstream")
        git(r, "checkout", "-q", "main")
        prune.prune(r, LIST)
        write(r, "own.txt", "ours\n")
        commit(r, "ours")
        m = git(r, "merge", "--no-commit", "--no-ff", "up", check=False)
        self.assertNotEqual(m.returncode, 0)  # modify/delete conflicts
        self.assertTrue(git(r, "diff", "--name-only", "--diff-filter=U").stdout.strip())
        prune.prune(r, LIST)
        self.assertEqual(git(r, "diff", "--name-only", "--diff-filter=U").stdout.strip(), "")
        git(r, "commit", "-q", "-m", "merge")
        files = git(r, "ls-files").stdout.split()
        self.assertIn("docs/new.md", files)
        self.assertIn("own.txt", files)
        self.assertNotIn("crates/horizon-server/src/main.rs", files)
        self.assertNotIn(".github/workflows/ci.yml", files)

    def test_keep_line_wins(self):
        pats, keep = prune.parse("a/\n!a/x.txt\n")
        self.assertTrue(prune.matches("a/x.txt", pats) and prune.matches("a/x.txt", keep))


if __name__ == "__main__":
    unittest.main()
