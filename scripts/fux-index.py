#!/usr/bin/env python3
"""fux-index — the optional Fux index package beside each rule pack (W-34).

L6 decision 12 and SR-PACKAGES decision 14: every rule pack has a companion
that holds a Fux index of the pack's `rules/` and nothing else —
`@codelaw/rules-<name>-fux-index` on npm for each npm pack and
`codelaw-rules-<name>-fux-index` on PyPI for each PyPI pack, at the pack's
version. Pure data, zero runtime dependencies; Fux reads it, codelaw's engine
never does (L5: codelaw is complete with no Fux).

    fux-index.py assemble --out dist     build every index package into dist/fux-index/
    fux-index.py check dist              the checks on what was assembled
    fux-index.py names                   every name this publishes, `registry name` per line
    fux-index.py latest [--pypi-json F]  `newer <v>` or `current <v>`: PyPI's fux-engine against FUX_VERSION (W-91)
    fux-index.py bump <v>                rewrite FUX_VERSION to <v>, the one line a bump changes (W-91)

**Fux is a build-time tool of codelaw's CI only**, pinned at FUX_VERSION so
the index bytes are reproducible (a Fux release that changes the segment
format rebuilds every index package; moving this pin is the deliberate act).
`assemble` refuses any other `fux`, builds each segment twice in two fresh
folders and refuses a segment whose two builds differ, then packs it with the
builders of `scripts/package.py` (`pack_tarball`, `wheel_for_pack`).

The output is `dist/fux-index/{npm,pypi,SHA256SUMS}`, **never** `dist/npm` or
`dist/pypi`: `publish.yml` uploads those two folders whole, and an index
package is published only after its names are reserved (RELEASING.md).

Python 3.9 syntax (work/MACHINE.md); the standard library only.
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import package  # noqa: E402  the release build's own code (SR-PACKAGES decision 9)

ROOT = package.ROOT
FUX_VERSION = "5.1.0"  # fux-engine, MIT: the one pin the index bytes depend on
SUFFIX = "-fux-index"
MANIFEST = "fux-index.json"
# What a segment holds (fux's `[plugins.index] segment` globs, SR-PKG-INDEX decision 4).
SEGMENT_GLOBS = ("index/*.jsonl", "identifiers.toml")
OUT = "fux-index"
PYPI_JSON = "https://pypi.org/pypi/fux-engine/json"
FUX_LINE = re.compile(r'^FUX_VERSION = "([^"]*)"', re.M)


def fail(msg: str) -> None:
    print("✗ " + msg, file=sys.stderr)
    sys.exit(1)


def description(name: str) -> str:
    return f"A Fux index of codelaw's {name} rules, for fux_search. Optional; install beside codelaw-rules-{name}."


def segment_digest(directory: pathlib.Path) -> str:
    """fux's `plugins.segment_digest`, spelled again so `check` needs no Fux:
    sha256 over RECORD-form lines of the segment's files, sorted, CRLF read as LF."""
    files = sorted({f for g in SEGMENT_GLOBS for f in directory.glob(g) if f.is_file()})
    if not files:
        return ""
    lines = []
    for f in files:
        h = hashlib.sha256(f.read_bytes().replace(b"\r\n", b"\n")).digest()
        lines.append(f"{f.relative_to(directory).as_posix()},sha256={base64.urlsafe_b64encode(h).rstrip(b'=').decode()}")
    return "sha256:" + hashlib.sha256("\n".join(lines).encode("utf-8")).hexdigest()


def run_fux(fux: str, args: list, cwd: pathlib.Path) -> None:
    done = subprocess.run([fux, *args], cwd=cwd, capture_output=True, text=True, encoding="utf-8")
    if done.returncode != 0:
        fail(f"fux {' '.join(args)} (in {cwd}) exited {done.returncode}:\n{done.stdout}{done.stderr}")


def check_fux(fux: str) -> None:
    got = subprocess.run([fux, "--version"], capture_output=True, text=True, encoding="utf-8").stdout.strip()
    if got != f"fux {FUX_VERSION}":
        fail(f"{fux} is `{got}`, this build is pinned to fux {FUX_VERSION} (pip install fux-engine=={FUX_VERSION}); "
             "a different Fux can write a different index, so it never builds one")


def build_segment(fux: str, name: str, v: str, dest: pathlib.Path) -> dict:
    """`packs/<name>/rules/` ingested and packed into `dest`; returns the manifest.
    The scratch repository is the pack folder in miniature, so a rule's id is
    `file:rules/<path>`, a path inside the installed pack (SR-PKG-INDEX decision 11)."""
    on_npm, on_pypi = name in package.NPM_PACKS, name in package.PYPI_PACKS
    npm_name = f"@codelaw/rules-{name}"
    pypi_name = f"codelaw-rules-{name}"
    with tempfile.TemporaryDirectory(prefix="fux-index-") as tmp:
        work = pathlib.Path(tmp)
        shutil.copytree(ROOT / "packs" / name / "rules", work / "rules")
        run_fux(fux, ["setup", "--no-agents", "--no-hooks"], work)
        (work / ".fux" / "sources.yaml").write_text("paths:\n  include:\n    - rules\n", encoding="utf-8")
        run_fux(fux, ["ingest", "--no-progress", "--no-fetch"], work)
        run_fux(fux, ["pack", str(dest), "--content", f"{pypi_name}@{v}", "--npm", npm_name, "--pypi", pypi_name], work)
    # The manifest is outside the digest (SR-PKG-INDEX decision 4), so it may say
    # which registries the docs package is on: a pack on one registry names one.
    mf = dest / MANIFEST
    man = json.loads(mf.read_text(encoding="utf-8"))
    if not on_npm:
        man["content"].pop("npm", None)
    if not on_pypi:
        man["content"].pop("pypi", None)
    mf.write_text(json.dumps(man, indent=2, sort_keys=True, ensure_ascii=False) + "\n", encoding="utf-8")
    return man


def tree(d: pathlib.Path) -> dict:
    return {f.relative_to(d).as_posix(): f.read_bytes() for f in sorted(d.rglob("*")) if f.is_file()}


def readme(name: str, v: str) -> str:
    npm = f"@codelaw/rules-{name}"
    pypi = f"codelaw-rules-{name}"
    on = [r for r, ok in (("npm", name in package.NPM_PACKS), ("PyPI", name in package.PYPI_PACKS)) if ok]
    lines = [
        f"# {npm}{SUFFIX}" if "npm" in on else f"# {pypi}{SUFFIX}",
        "",
        f"An optional [Fux](https://github.com/arpitarya/fux) index of codelaw's `{name}` rules, at {v}.",
        "It is data only (the shards, `identifiers.toml` and `fux-index.json`), has no dependencies and is not",
        "installed by `codelaw init`. codelaw itself never reads it; install it only if your repository uses Fux.",
        "",
        f"It indexes the files of `{npm if 'npm' in on else pypi}` at the same version, so install that pack too.",
        "Pin it in `.fux/config/plugins.toml` with `fux setup --index <PyPI name> --index-npm <npm name>`:",
        "",
    ]
    if "npm" in on:
        lines.append(f"- npm: `{npm}{SUFFIX}`")
    if "PyPI" in on:
        lines.append(f"- PyPI: `{pypi}{SUFFIX}`")
    lines += ["", f"Built with fux {FUX_VERSION} from `packs/{name}/rules/` of the codelaw repository (MIT).", ""]
    return "\n".join(lines)


def assemble(out: pathlib.Path, fux: str) -> None:
    check_fux(fux)
    v = package.versionmod.read_version()
    top = out.resolve() / OUT
    if top.exists():
        shutil.rmtree(top)
    stage, npm_out, pypi_out = top / "stage", top / "npm", top / "pypi"
    for d in (stage, npm_out, pypi_out):
        d.mkdir(parents=True)
    for name in indexed(package.PACKS):
        with tempfile.TemporaryDirectory(prefix="fux-index-out-") as tmp:
            one, two = pathlib.Path(tmp) / "one", pathlib.Path(tmp) / "two"
            build_segment(fux, name, v, one)
            build_segment(fux, name, v, two)
            if tree(one) != tree(two):
                fail(f"{name}: two builds of the same rules gave different segments — the index is not reproducible")
            d = stage / f"rules-{name}{SUFFIX}"
            shutil.copytree(one, d)
        package.copy_licences(d)
        (d / "README.md").write_text(readme(name, v), encoding="utf-8")
        if name in package.PYPI_PACKS:
            package.wheel_for_pack(name, v, d, pypi_out, SUFFIX, description(name))
        if name in package.NPM_PACKS:
            package.pack_tarball(name, v, d, npm_out, SUFFIX, description(name))
        print(f"  {name}: {json.loads((d / MANIFEST).read_text())['docs']} rules, {segment_digest(d)[:19]}…")
    shutil.rmtree(stage)
    lines = [f"{hashlib.sha256(f.read_bytes()).hexdigest()}  {f.parent.name}/{f.name}" for f in sorted(list(npm_out.iterdir()) + list(pypi_out.iterdir()))]
    (top / "SHA256SUMS").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"✔ assembled {len(list(npm_out.iterdir()))} npm tarballs and {len(list(pypi_out.iterdir()))} wheels at {v} in {top}")


def rule_count(name: str) -> int:
    return sum(1 for f in (ROOT / "packs" / name / "rules").rglob("*") if f.is_file())


def indexed(packs):
    """The packs that have an index package: those holding rules. A pack with no
    rule file yet (the four security packs until W-53's rules land) has nothing to
    index — Fux refuses an empty index — and gets its package with its first rule."""
    return tuple(p for p in packs if rule_count(p))


def check(out: pathlib.Path) -> None:
    top = out / OUT
    v = package.versionmod.read_version()
    pv = package.pep440(v)
    bad = []
    npm: dict = {}
    pypi: dict = {}
    for t in sorted((top / "npm").glob("*.tgz")):
        files = package.tar_files(t)
        man = json.loads(files["package.json"])
        npm[man["name"]] = files
        if man["version"] != v:
            bad.append(f"{t.name}: version {man['version']} != {v}")
        if man.get("license") != package.LICENCE:
            bad.append(f"{t.name}: license {man.get('license')} != {package.LICENCE}")
        for f in package.LICENCE_FILES + ("README.md", MANIFEST):
            if f not in files:
                bad.append(f"{t.name}: has no {f}")
        for field in ("dependencies", "peerDependencies", "optionalDependencies", "bundleDependencies", "bundledDependencies", "scripts"):
            if man.get(field):
                bad.append(f"{t.name}: has {field} — an index package is pure data with zero dependencies (L6 decision 12)")
    for w in sorted((top / "pypi").glob("*.whl")):
        m = re.match(r"^(codelaw_rules_[a-z_]+_fux_index)-([^-]+)-py3-none-any\.whl$", w.name)
        if not m:
            bad.append(f"pypi: {w.name} is not an index wheel codelaw publishes")
            continue
        dist, wv = m.groups()
        if wv != pv:
            bad.append(f"{w.name}: version {wv} != {pv}")
        with zipfile.ZipFile(w) as z:
            meta = z.read(f"{dist}-{wv}.dist-info/METADATA").decode()
            names = z.namelist()
            if re.search(r"(?m)^Requires-Dist:", meta):
                bad.append(f"{w.name}: has Requires-Dist — zero runtime dependencies")
            if f"License-Expression: {package.LICENCE}" not in meta:
                bad.append(f"{w.name}: METADATA has no License-Expression: {package.LICENCE}")
            for f in package.LICENCE_FILES:
                if not any(n.endswith(f"/licenses/{f}") for n in names):
                    bad.append(f"{w.name}: has no dist-info/licenses/{f}")
            prefix = dist + "/"
            short = next((p for p in package.PYPI_PACKS if package.pack_module(p) + "_fux_index" == dist), dist)
            pypi[short] = {n[len(prefix):]: z.read(n) for n in names if n.startswith(prefix) and not n.endswith("/")}
    want_npm = {f"@codelaw/rules-{n}{SUFFIX}" for n in indexed(package.NPM_PACKS)}
    if set(npm) != want_npm:
        bad.append(f"npm names: missing {sorted(want_npm - set(npm))}, not allowed {sorted(set(npm) - want_npm)}")
    want_pypi = set(indexed(package.PYPI_PACKS))
    if set(pypi) != want_pypi:
        bad.append(f"PyPI index packages: missing {sorted(want_pypi - set(pypi))}, not allowed {sorted(set(pypi) - want_pypi)}")

    # each segment is what Fux wrote: the digest the manifest states is the one the
    # shards give, the manifest names the pack it indexes, and the rules are all there
    for name in indexed(package.PACKS):
        copies = {}
        if name in package.NPM_PACKS:
            copies["npm"] = {k: x for k, x in npm.get(f"@codelaw/rules-{name}{SUFFIX}", {}).items() if k != "package.json"}
        if name in package.PYPI_PACKS:
            copies["PyPI"] = pypi.get(name, {})
        for reg, files in copies.items():
            if MANIFEST not in files:
                continue
            with tempfile.TemporaryDirectory() as tmp:
                seg = pathlib.Path(tmp)
                for rel, data in files.items():
                    if rel.startswith("index/") or rel == "identifiers.toml":
                        (seg / rel).parent.mkdir(parents=True, exist_ok=True)
                        (seg / rel).write_bytes(data)
                got = segment_digest(seg)
            man = json.loads(files[MANIFEST])
            if man.get("digest") != got:
                bad.append(f"{name} ({reg}): manifest digest {man.get('digest')} is not the shards' {got}")
            if man.get("fux_version") != FUX_VERSION:
                bad.append(f"{name} ({reg}): built by fux {man.get('fux_version')}, this build pins {FUX_VERSION}")
            c = man.get("content", {})
            want = {"version": v}
            if name in package.NPM_PACKS:
                want["npm"] = f"@codelaw/rules-{name}"
            if name in package.PYPI_PACKS:
                want["pypi"] = f"codelaw-rules-{name}"
            if c != want:
                bad.append(f"{name} ({reg}): manifest content {c} is not the pack it indexes {want}")
            if man.get("docs") != rule_count(name):
                bad.append(f"{name} ({reg}): indexes {man.get('docs')} documents, packs/{name}/rules/ holds {rule_count(name)}")
            if not (set(files) - set(package.LICENCE_FILES) - {"README.md"}) >= {"identifiers.toml", MANIFEST}:
                bad.append(f"{name} ({reg}): segment is missing identifiers.toml or the manifest")
            if set(files) - set(package.LICENCE_FILES) - {"README.md", "identifiers.toml", MANIFEST} != {f for f in files if f.startswith("index/") and f.endswith(".jsonl")}:
                bad.append(f"{name} ({reg}): holds files besides the segment, its README and LICENSE")
        if len(copies) == 2 and copies["npm"] != copies["PyPI"]:
            bad.append(f"{name}: the npm and PyPI index packages differ — one segment, both registries")

    sums = top / "SHA256SUMS"
    listed = {}
    if not sums.is_file():
        bad.append("fux-index/SHA256SUMS is missing")
    else:
        for line in sums.read_text(encoding="utf-8").splitlines():
            m = re.fullmatch(r"([0-9a-f]{64})  ((?:npm|pypi)/[^/]+)", line)
            if m:
                listed[m.group(2)] = m.group(1)
            else:
                bad.append(f"SHA256SUMS: malformed line {line!r}")
    present = {f"{d}/{f.name}" for d in ("npm", "pypi") if (top / d).is_dir() for f in (top / d).iterdir() if f.is_file()}
    if sums.is_file() and set(listed) != present:
        bad.append(f"SHA256SUMS lists {sorted(set(listed) - present)[:3]} not present, omits {sorted(present - set(listed))[:3]}")
    for rel, digest in listed.items():
        f = top / rel
        if f.is_file() and hashlib.sha256(f.read_bytes()).hexdigest() != digest:
            bad.append(f"{rel}: its sha256 is not the one SHA256SUMS lists")
    # the release set itself must not carry them: they are published on their own
    for sub in ("npm", "pypi"):
        # `fux?index`: npm's `-fux-index` and a wheel's `_fux_index` (no `**`: Python < 3.13 refuses it)
        for f in (out / sub).glob(f"*{SUFFIX.lstrip('-').replace('-', '?')}*") if (out / sub).is_dir() else []:
            bad.append(f"{sub}/{f.name}: an index package is in the release set — publish.yml would upload it unreserved")
    if bad:
        fail(f"{len(bad)} fux-index check(s) failed:\n  " + "\n  ".join(bad))
    print(f"✔ {len(npm)} npm and {len(pypi)} PyPI index packages at {v}: only the names L6 decision 12 lists, one version, "
          "zero dependencies, licensed, each manifest true to its shards, both registries byte-equal, outside the release set")


def release(v: str) -> tuple:
    """A final release `N(.N)*` as a tuple of ints; anything else (a pre-release,
    a dev or local version) is refused — the bump job never pins one."""
    if not re.fullmatch(r"\d+(\.\d+)*", v):
        fail(f"`{v}` is not a final release (N.N.N); fux-index pins only final releases")
    t = tuple(int(x) for x in v.split("."))
    while len(t) > 1 and t[-1] == 0:
        t = t[:-1]
    return t


def latest(pypi_json) -> None:
    """Compare fux-engine's latest release with FUX_VERSION and print one line:
    `newer <v>` when PyPI has a later release, `current <pin>` otherwise. The
    latest is, in order: $FUX_LATEST (a test, or the workflow's `version`
    input), the JSON file given, or PyPI's JSON API (one read-only GET)."""
    forced = os.environ.get("FUX_LATEST", "").strip()
    want, where = forced, "$FUX_LATEST"
    if not want:
        where = "PyPI"
        if pypi_json:
            raw = pypi_json.read_text(encoding="utf-8")
        else:
            import urllib.request
            with urllib.request.urlopen(PYPI_JSON, timeout=30) as r:
                raw = r.read().decode("utf-8")
        want = json.loads(raw)["info"]["version"]
    # A forced $FUX_LATEST may also move the pin down (a rollback, or a test of the job).
    if (release(want) != release(FUX_VERSION)) if forced else (release(want) > release(FUX_VERSION)):
        print(f"fux-engine {want} ({where}), pinned {FUX_VERSION}: bump", file=sys.stderr)
        print(f"newer {want}")
    else:
        print(f"fux-engine {want} ({where}), pinned {FUX_VERSION}: nothing to do", file=sys.stderr)
        print(f"current {FUX_VERSION}")


def bump(v: str) -> None:
    release(v)
    me = pathlib.Path(__file__).resolve()
    src = me.read_text(encoding="utf-8")
    if len(FUX_LINE.findall(src)) != 1:
        fail(f"{me.name}: expected exactly one `FUX_VERSION = \"…\"` line")
    me.write_text(FUX_LINE.sub(f'FUX_VERSION = "{v}"', src, count=1), encoding="utf-8")
    print(f"✔ FUX_VERSION {FUX_VERSION} → {v}")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("assemble")
    a.add_argument("--out", type=pathlib.Path, required=True)
    a.add_argument("--fux", default=os.environ.get("FUX", "fux"), help="the fux executable (default: fux on PATH)")
    c = sub.add_parser("check")
    c.add_argument("out", type=pathlib.Path)
    sub.add_parser("names")
    lt = sub.add_parser("latest")
    lt.add_argument("--pypi-json", type=pathlib.Path, help="PyPI's JSON for fux-engine, read instead of fetching it")
    b = sub.add_parser("bump")
    b.add_argument("version")
    args = ap.parse_args()
    if args.cmd == "assemble":
        assemble(args.out, shutil.which(args.fux) or fail(f"{args.fux} is not on PATH"))
    elif args.cmd == "check":
        check(args.out)
    elif args.cmd == "latest":
        latest(args.pypi_json)
    elif args.cmd == "bump":
        bump(args.version)
    else:
        for n in indexed(package.NPM_PACKS):
            print(f"npm @codelaw/rules-{n}{SUFFIX}")
        for n in indexed(package.PYPI_PACKS):
            print(f"pypi codelaw-rules-{n}{SUFFIX}")


if __name__ == "__main__":
    main()
