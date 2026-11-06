import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(__file__))
import port_upstream
import rename_map
from gitutil import commit, git, new_repo, write

HERE = os.path.dirname(os.path.abspath(__file__))
BASE = {
    "Cargo.toml": '[workspace]\nmembers = [\n    "crates/horizon-core",\n    "crates/horizon-world",\n    "crates/horizon-server",\n]\n',
    "crates/horizon-core/src/lib.rs": "pub fn core() {}\n",
    "crates/horizon-world/src/lib.rs": "use horizon_core::core;\npub fn w() {\n    core();\n}\n",
    "crates/horizon-server/src/main.rs": "fn main() {}\n",
    "launcher/src/main.rs": "fn main() {}\n",
    "README.md": "horizon readme\n",
    "mods/horizon/src/A.java": "class A {}\n",
}


def fork():
    """(upstream repo, Zenith repo, base sha): Zenith is Horizon's base commit in Zenith's names plus its own files."""
    u = new_repo("up")
    for p, t in BASE.items():
        write(u, p, t)
    base = commit(u, "base")
    z = new_repo("zen")
    pats, keep = port_upstream.parse_list(open(os.path.join(HERE, "prune-paths.txt"), encoding="utf-8").read())
    for p, t in BASE.items():
        if not port_upstream.matches(p, pats):
            write(z, rename_map.map_path(p), rename_map.rewrite(t))
    write(z, "README.md", "zenith readme\n")
    write(z, ".gitattributes", "README.md merge=ours\n")
    write(z, "launcher/own.txt", "our launcher\n")
    write(z, "ci/upstream-synced.txt", base + "\n")
    commit(z, "zenith")
    git(z, "remote", "add", "upstream", u)
    git(z, "fetch", "-q", "upstream")
    return u, z, base


def upstream_commit(u, **files):
    for p, t in files.items():
        p = p.replace("__", "/")
        if t is None:
            git(u, "rm", "-q", p)
        else:
            write(u, p, t)
    return commit(u, "horizon news")


def run(z, **kw):
    out = os.path.join(z, "..", os.path.basename(z) + ".out")
    if os.path.exists(out):
        os.remove(out)
    rc = port_upstream.port(z, "upstream/main", no_cargo=True, output=out, **kw)
    return rc, dict(line.split("=", 1) for line in open(out, encoding="utf-8").read().splitlines())


class PortTest(unittest.TestCase):
    def test_names_and_paths_are_translated_and_what_is_not_carried_is_dropped(self):
        u, z, _ = fork()
        up = upstream_commit(u, **{
            "crates__horizon-world__src__lib.rs": "use horizon_core::core;\npub fn w() {\n    core();\n    horizon_core::core();\n}\n",
            "crates__horizon-server__src__main.rs": "fn main() { x(); }\n",
            "launcher__src__main.rs": "fn main() { y(); }\n",
            "README.md": "horizon readme 2\n",
            "mods__horizon__src__B.java": "class B { String s = \"horizon-core\"; }\n",
            "crates__horizon-core__src__new.rs": "// see https://github.com/horizon-rust/horizon/blob/main/crates/horizon-core\n",
        })
        git(z, "fetch", "-q", "upstream")
        rc, out = run(z)
        self.assertEqual((rc, out["outcome"], out["upstream"]), (0, "ported", up))
        read = lambda p: open(os.path.join(z, p), encoding="utf-8").read()  # noqa: E731
        self.assertIn("zenith_core::core();", read("crates/zenith-world/src/lib.rs"))
        self.assertNotIn("horizon_core", read("crates/zenith-world/src/lib.rs"))
        self.assertEqual(read("README.md"), "zenith readme\n")  # Zenith's own file
        self.assertEqual(read("launcher/own.txt"), "our launcher\n")
        self.assertFalse(os.path.exists(os.path.join(z, "launcher/src/main.rs")))  # Horizon's launcher is not carried
        self.assertFalse(os.path.exists(os.path.join(z, "crates/zenith-server")))
        self.assertIn('"zenith-core"', read("java/horizon/src/B.java"))
        self.assertIn("horizon-rust/horizon/blob/main/crates/horizon-core", read("crates/zenith-core/src/new.rs"))  # URL untouched
        self.assertEqual(read("ci/upstream-synced.txt").split()[0], up)
        self.assertEqual(git(z, "status", "--porcelain").stdout.strip(), "")

    def test_up_to_date_changes_nothing(self):
        u, z, _ = fork()
        head = git(z, "rev-parse", "HEAD").stdout
        rc, out = run(z)
        self.assertEqual((rc, out["outcome"]), (0, "up-to-date"))
        self.assertEqual(git(z, "rev-parse", "HEAD").stdout, head)

    def test_only_dropped_paths_still_records_the_sha(self):
        u, z, _ = fork()
        up = upstream_commit(u, **{"launcher__src__main.rs": "fn main() { y(); }\n"})
        git(z, "fetch", "-q", "upstream")
        rc, out = run(z)
        self.assertEqual((rc, out["outcome"]), (0, "ported"))
        self.assertEqual(open(os.path.join(z, "ci/upstream-synced.txt")).read().split()[0], up)

    def test_conflict_with_our_edit_blocks_and_leaves_the_tree_alone(self):
        u, z, _ = fork()
        write(z, "crates/zenith-core/src/lib.rs", "pub fn core() { /* our hook */ }\n")
        commit(z, "ours")
        head = git(z, "rev-parse", "HEAD").stdout
        upstream_commit(u, **{"crates__horizon-core__src__lib.rs": "pub fn core() { new(); }\n"})
        git(z, "fetch", "-q", "upstream")
        rc, out = run(z)
        self.assertEqual((rc, out["outcome"], out["reason"]), (port_upstream.BLOCKED, "blocked", "conflict"))
        self.assertIn("crates/zenith-core/src/lib.rs", out["conflicts"])
        self.assertEqual(git(z, "rev-parse", "HEAD").stdout, head)
        self.assertEqual(git(z, "status", "--porcelain").stdout.strip(), "")
        # the commit for the person who resolves it carries the rejects
        self.assertIn("lib.rs.rej", git(z, "show", "--stat", "--format=", out["commit"]).stdout)

    def test_dirty_worktree_is_refused(self):
        u, z, _ = fork()
        upstream_commit(u, **{"crates__horizon-core__src__lib.rs": "pub fn core() { new(); }\n"})
        git(z, "fetch", "-q", "upstream")
        write(z, "crates/zenith-core/src/lib.rs", "pub fn core() { local(); }\n")
        rc, out = run(z)
        self.assertEqual((rc, out["reason"]), (port_upstream.BLOCKED, "dirty-worktree"))
        self.assertIn("local()", open(os.path.join(z, "crates/zenith-core/src/lib.rs")).read())

    def test_rename_map_keeps_the_formats_and_maps_the_rest(self):
        r = rename_map.rewrite
        self.assertEqual(r("cargo test -p horizon-client --bin horizon-client; target/fast/horizon-client.exe"),
                         "cargo test -p zenith-client --bin zenith; target/fast/zenith.exe")
        self.assertEqual(r('"horizon-mods/a.hmod" horizon-mods = { path = "../horizon-mods" }'),
                         '"horizon-mods/a.hmod" zenith-mods = { path = "../zenith-mods" }')
        self.assertEqual(r("horizon-shaders.toml crates/horizon-shaders"), "horizon-shaders.toml crates/zenith-shaders")
        self.assertEqual(rename_map.map_path("mods/horizon/src/x"), "java/horizon/src/x")
        self.assertEqual(rename_map.map_path("packaging/icons/horizon.ico"), "packaging/icons/zenith.ico")


if __name__ == "__main__":
    unittest.main()
