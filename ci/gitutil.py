"""Test helper: tiny temp git repos."""
import os
import subprocess
import tempfile


def git(repo, *args, check=True):
    return subprocess.run(["git", "-C", repo, "-c", "user.name=t", "-c", "user.email=t@t", "-c", "core.autocrlf=false",
                           *args], check=check, capture_output=True, text=True, encoding="utf-8")


def new_repo(prefix="zen"):
    d = tempfile.mkdtemp(prefix=prefix)
    git(d, "init", "-q", "-b", "main")
    git(d, "config", "merge.ours.driver", "true")
    return d


def write(repo, rel, text):
    p = os.path.join(repo, rel)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)


def commit(repo, msg="c"):
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", msg, "--allow-empty")
    return git(repo, "rev-parse", "HEAD").stdout.strip()
