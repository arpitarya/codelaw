#!/usr/bin/env python3
"""The one version (L6 decision 9; SR-PACKAGES decision 7).

The version lives in ONE file: `Cargo.toml` [workspace.package] `version`.
Every npm and PyPI manifest is generated from it by `scripts/package.py`; the
only other copies are the five `packs/*/codelaw-pack.toml` files, which this
script writes and checks.

    python3 scripts/version.py                     print the version
    python3 scripts/version.py --set 0.2.0         write it everywhere
    python3 scripts/version.py --check             every copy agrees (B-012)
    python3 scripts/version.py --check --tag v0.2.0 --binary target/release/codelaw

Python 3.9 syntax on purpose (work/MACHINE.md): no tomllib, so the few lines
read here are matched by pattern, and a file that stops matching is an error.
"""
import argparse
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
for _s in (sys.stdout, sys.stderr):  # ✔/✗ on a Windows console
    _s.reconfigure(encoding="utf-8")
CARGO = ROOT / "Cargo.toml"
LOCK = ROOT / "Cargo.lock"
PACKS = sorted((ROOT / "packs").glob("*/codelaw-pack.toml"))
SEMVER = re.compile(r"^\d+\.\d+\.\d+(-(alpha|beta|rc)\.\d+)?$")  # a pre-release PyPI can also take (package.pep440)
WORKSPACE_VERSION = re.compile(r'(\[workspace\.package\][^\[]*?\nversion = ")([^"]+)(")', re.S)
PACK_VERSION = re.compile(r'(?m)^(version = ")([^"]+)(")')
PACK_MIN = re.compile(r'(?m)^min_codelaw = "([^"]+)"')


def fail(msg):
    print("✗ " + msg, file=sys.stderr)
    sys.exit(1)


def read_version():
    m = WORKSPACE_VERSION.search(CARGO.read_text(encoding="utf-8"))
    if not m:
        fail("Cargo.toml has no [workspace.package] version")
    return m.group(2)


def key(v):
    """Order versions by their numbers; a pre-release sorts before its release."""
    core, _, pre = v.partition("-")
    return tuple(int(x) for x in core.split(".")) + ((0, pre) if pre else (1, ""))


def workspace_crates():
    return sorted(p.parent.name for p in (ROOT / "crates").glob("*/Cargo.toml"))


def lock_versions():
    """name -> version for the workspace crates in Cargo.lock."""
    text = LOCK.read_text(encoding="utf-8")
    out = {}
    for name, ver in re.findall(r'\[\[package\]\]\nname = "([^"]+)"\nversion = "([^"]+)"', text):
        out.setdefault(name, ver)
    return {c: out.get(c) for c in workspace_crates()}


def set_version(v):
    if not SEMVER.match(v):
        fail(f"`{v}` is not MAJOR.MINOR.PATCH[-alpha|beta|rc.N]")
    text = CARGO.read_text(encoding="utf-8")
    CARGO.write_text(WORKSPACE_VERSION.sub(lambda m: m.group(1) + v + m.group(3), text, count=1), encoding="utf-8")
    for p in PACKS:
        t = p.read_text(encoding="utf-8")
        if not PACK_VERSION.search(t):
            fail(f"{p.relative_to(ROOT)} has no version line")
        p.write_text(PACK_VERSION.sub(lambda m: m.group(1) + v + m.group(3), t, count=1), encoding="utf-8")
    lock = LOCK.read_text(encoding="utf-8")
    for c in workspace_crates():
        lock = re.sub(r'(\[\[package\]\]\nname = "' + re.escape(c) + r'"\nversion = ")[^"]+(")', lambda m: m.group(1) + v + m.group(2), lock)
    LOCK.write_text(lock, encoding="utf-8")
    print(f"✔ version {v}: Cargo.toml, Cargo.lock and {len(PACKS)} pack manifests")


def check(tag=None, binary=None):
    v = read_version()
    if not SEMVER.match(v):
        fail(f"Cargo.toml version `{v}` is not MAJOR.MINOR.PATCH[-alpha|beta|rc.N]")
    bad = []
    for p in PACKS:
        t = p.read_text(encoding="utf-8")
        m = PACK_VERSION.search(t)
        got = m.group(2) if m else None
        if got != v:
            bad.append(f"{p.relative_to(ROOT)}: version {got} != {v}")
        mn = PACK_MIN.search(t)
        if not mn:
            bad.append(f"{p.relative_to(ROOT)}: no min_codelaw")
        elif key(mn.group(1)) > key(v):
            bad.append(f"{p.relative_to(ROOT)}: min_codelaw {mn.group(1)} is newer than the engine {v}")
    for crate, got in lock_versions().items():
        if got != v:
            bad.append(f"Cargo.lock: {crate} {got} != {v} (run `cargo update --workspace`)")
    if tag is not None and tag.lstrip("v") != v:
        bad.append(f"release tag {tag} != version {v} — run `python3 scripts/version.py --set {tag.lstrip('v')}` first")
    if binary is not None:
        out = subprocess.run([binary, "--version"], capture_output=True, text=True).stdout.strip()
        if out != f"codelaw {v}":
            bad.append(f"{binary} --version says `{out}`, not `codelaw {v}`")
    if bad:
        fail("the version copies disagree:\n  " + "\n  ".join(bad))
    print(f"✔ version {v}: Cargo.toml, Cargo.lock, {len(PACKS)} packs" + (", the tag" if tag else "") + (", the binary" if binary else ""))


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--set", metavar="VERSION")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--tag")
    ap.add_argument("--binary")
    a = ap.parse_args()
    if a.set:
        set_version(a.set)
    elif a.check:
        check(a.tag, a.binary)
    else:
        print(read_version())


if __name__ == "__main__":
    main()
