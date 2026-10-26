#!/usr/bin/env python3
"""Assemble and check every codelaw package (SR-PACKAGES decisions 3-9; L6).

The release publishes exactly what this script assembles, and nothing is
compiled here: the eight binaries and the eight engine wheels come from the
build jobs in `.github/workflows/ci.yml`.

    package.py prepare-wheel                 copy LICENSE and README into packaging/python for maturin
    package.py extract WHEEL OUTDIR          take the binary out of a maturin wheel (the npm copy)
    package.py assemble --bins B --wheels W --out dist
                                             build the 25 npm tarballs and the 9 pack wheels; copy the 8 engine wheels
    package.py check dist                    the release checks, against what was assembled
    package.py sync-dev-core                 copy the dev-server plugins' shared source into each package (W-49)
    package.py names                         every name codelaw publishes (SR-PACKAGES §1)

`--bins` holds one folder per platform key (`linux-x64-gnu/codelaw`, ...),
`--wheels` the maturin wheels. Python 3.9 syntax (work/MACHINE.md).
"""
from __future__ import annotations

import argparse
import base64
import concurrent.futures
import gzip
import hashlib
import io
import json
import pathlib
import re
import shutil
import subprocess
import sys
import tarfile
import zipfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import version as versionmod  # noqa: E402

ROOT = versionmod.ROOT
LICENCE = "MIT"
LICENCE_FILES = ("LICENSE",)
SCHEMA = ROOT / "crates" / "codelaw-config" / "codelaw.schema.json"  # shipped in npm codelaw (SR-PACKAGES decision 3)
REPO = "https://github.com/arpitarya/codelaw"
PACK_TABLE = ROOT / "crates" / "codelaw-rules" / "src" / "pack.rs"

# The npm-only dev-server plugins (W-49, SR-PACKAGES decision 3a). Each is one folder
# under packaging/npm/ holding index.js, index.d.ts and README.md; the shared
# `dev-core/core.js` (and, for Next, the webpack plugin) is copied in by
# `sync-dev-core`, so each package has no dependency (L6 decision 4).
DEV_PLUGINS = {
    "vite-plugin": ("@codelaw/vite-plugin", "codelaw's findings in Vite's overlay and terminal; fails vite build on an error"),
    "webpack-plugin": ("@codelaw/webpack-plugin", "codelaw's findings as webpack and Rspack errors and warnings; fails the build on an error"),
    "next": ("@codelaw/next", "withCodelaw(nextConfig): codelaw's findings in next dev, and next build fails on an error"),
}
DEV_SHARED = {  # destination (relative to packaging/npm) -> source
    "vite-plugin/core.js": "dev-core/core.js",
    "webpack-plugin/core.js": "dev-core/core.js",
    "next/core.js": "dev-core/core.js",
    "next/plugin.js": "webpack-plugin/index.js",
}
NPM_DIR = ROOT / "packaging" / "npm"


def sync_dev_core(check: bool = False) -> list[str]:
    """Copy the shared dev-plugin sources into place; with `check`, only report the copies that differ."""
    stale = []
    for dst, src in DEV_SHARED.items():
        want = (NPM_DIR / src).read_bytes()
        d = NPM_DIR / dst
        if not d.is_file() or d.read_bytes() != want:
            stale.append(f"packaging/npm/{dst} differs from packaging/npm/{src}")
            if not check:
                d.write_bytes(want)
    return stale


def read_packs() -> tuple[tuple[str, ...], tuple[str, ...]]:
    """Which first-party pack is published on which registry: read from
    FIRST_PARTY_PACKS in pack.rs, the one list (L6 decision 11; the
    decider's ruling of 2026-10-24), so the engine and the release never
    disagree. Returns (on npm, on PyPI), in the table's order."""
    text = PACK_TABLE.read_text(encoding="utf-8")
    body = text[text.index("pub const FIRST_PARTY_PACKS"):]
    body = body[:body.index("];")]
    npm: list[str] = []
    pypi: list[str] = []
    for kind, short in re.findall(r'\b(both|npm_only|pypi_only)\("([a-z-]+)"', body):
        if kind in ("both", "npm_only"):
            npm.append(short)
        if kind in ("both", "pypi_only"):
            pypi.append(short)
    folders = sorted(p.parent.name for p in (ROOT / "packs").glob("*/codelaw-pack.toml"))
    if not npm or sorted(set(npm) | set(pypi)) != folders:
        sys.exit(f"✗ FIRST_PARTY_PACKS in {PACK_TABLE.relative_to(ROOT)} names {sorted(set(npm) | set(pypi))}, packs/ holds {folders}")
    return tuple(npm), tuple(pypi)


NPM_PACKS, PYPI_PACKS = read_packs()
PACKS = tuple(dict.fromkeys(NPM_PACKS + PYPI_PACKS))

# key -> (npm os, npm cpu, npm libc or None, Rust target, wheel platform-tag pattern)
TARGETS = {
    "darwin-arm64": ("darwin", "arm64", None, "aarch64-apple-darwin", r"macosx_\d+_\d+_arm64"),
    "darwin-x64": ("darwin", "x64", None, "x86_64-apple-darwin", r"macosx_\d+_\d+_x86_64"),
    "linux-x64-gnu": ("linux", "x64", "glibc", "x86_64-unknown-linux-gnu", r"manylinux_2_17_x86_64(\.manylinux2014_x86_64)?"),
    "linux-arm64-gnu": ("linux", "arm64", "glibc", "aarch64-unknown-linux-gnu", r"manylinux_2_17_aarch64(\.manylinux2014_aarch64)?"),
    "linux-x64-musl": ("linux", "x64", "musl", "x86_64-unknown-linux-musl", r"musllinux_1_2_x86_64"),
    "linux-arm64-musl": ("linux", "arm64", "musl", "aarch64-unknown-linux-musl", r"musllinux_1_2_aarch64"),
    "win32-x64": ("win32", "x64", None, "x86_64-pc-windows-msvc", r"win_amd64"),
    "win32-arm64": ("win32", "arm64", None, "aarch64-pc-windows-msvc", r"win_arm64"),
}
ZIP_DATE = (1980, 1, 1, 0, 0, 0)
# npm's own fixed mtime for tarball entries (1985-10-26T08:15:00Z).
NPM_MTIME = 499162500


def fail(msg: str) -> None:
    print("✗ " + msg, file=sys.stderr)
    sys.exit(1)


def exe(key: str) -> str:
    return "codelaw.exe" if key.startswith("win32") else "codelaw"


def pep440(v: str) -> str:
    """Cargo's `0.2.0-beta.1` is PyPI's `0.2.0b1` (maturin converts the same way)."""
    m = re.match(r"^(\d+\.\d+\.\d+)(?:-(alpha|beta|rc)\.(\d+))?$", v)
    if not m:
        fail(f"version {v} has no PyPI form (use -alpha.N, -beta.N or -rc.N)")
    core, pre, n = m.groups()
    return core + ({"alpha": "a", "beta": "b", "rc": "rc"}[pre] + n if pre else "")


def pack_meta(name: str) -> dict:
    text = (ROOT / "packs" / name / "codelaw-pack.toml").read_text(encoding="utf-8")
    m = re.search(r'(?m)^description = "([^"]*)"', text)
    return {"description": m.group(1) if m else f"codelaw's {name} rules"}


def write_json(path: pathlib.Path, obj: dict) -> None:
    path.write_text(json.dumps(obj, indent=2) + "\n", encoding="utf-8")


def copy_licences(dest: pathlib.Path) -> None:
    for f in LICENCE_FILES:
        shutil.copyfile(ROOT / f, dest / f)


def base_manifest(name: str, v: str, description: str) -> dict:
    return {
        "name": name,
        "version": v,
        "description": description,
        "license": LICENCE,
        "repository": {"type": "git", "url": f"git+{REPO}.git"},
        "homepage": REPO,
    }


def pack_manifest(name: str, v: str) -> dict:
    """A rule pack's package.json (decision 9)."""
    m = base_manifest(f"@codelaw/rules-{name}", v, pack_meta(name)["description"])
    m["keywords"] = ["codelaw", "codelaw-pack", "lint-rules"]
    return m


# ── prepare-wheel / extract ────────────────────────────────────────────────


def prepare_wheel() -> None:
    dest = ROOT / "packaging" / "python"
    copy_licences(dest)
    shutil.copyfile(ROOT / "packaging" / "README.md", dest / "README.md")
    print("✔ packaging/python: LICENSE, README.md")


def wheel_binary(wheel: pathlib.Path) -> tuple[str, bytes]:
    with zipfile.ZipFile(wheel) as z:
        hits = [n for n in z.namelist() if re.search(r"\.data/scripts/codelaw(\.exe)?$", n)]
        if len(hits) != 1:
            fail(f"{wheel.name}: expected one codelaw binary under .data/scripts/, found {hits}")
        return hits[0], z.read(hits[0])


def extract(wheel: pathlib.Path, outdir: pathlib.Path) -> None:
    name, data = wheel_binary(wheel)
    outdir.mkdir(parents=True, exist_ok=True)
    target = outdir / pathlib.PurePosixPath(name).name
    target.write_bytes(data)
    target.chmod(0o755)
    print(f"✔ {target} ({len(data)} bytes) from {wheel.name}")


# ── assemble ───────────────────────────────────────────────────────────────


def npm_pack(pkg_dir: pathlib.Path, dest: pathlib.Path) -> None:
    npm = shutil.which("npm") or fail("npm is not on PATH")
    subprocess.run([npm, "pack", "--pack-destination", str(dest), "--silent"], cwd=pkg_dir, check=True, stdout=subprocess.DEVNULL)


def stage_pack(name: str, stage: pathlib.Path) -> pathlib.Path:
    """`packs/<name>/` copied to `stage/rules-<name>/` with the licence files: what
    every registry's copy of the pack holds (decision 9). The one pack builder —
    `assemble` and `scripts/build-packs.py` both stage through here (W-27)."""
    d = stage / f"rules-{name}"
    shutil.copytree(ROOT / "packs" / name, d, ignore=shutil.ignore_patterns(".*"))
    copy_licences(d)
    return d


def pack_tarball(name: str, v: str, src: pathlib.Path, dest: pathlib.Path) -> pathlib.Path:
    """`@codelaw/rules-<name>` as an npm tarball: `package/` is the staged folder plus
    its package.json. Written here, not by `npm pack`, so it is byte-for-byte the
    same wherever it is built: fixed mtimes and modes, sorted entries, gzip mtime 0."""
    entries = [(f.relative_to(src).as_posix(), f.read_bytes()) for f in src.rglob("*") if f.is_file()]
    entries.append(("package.json", (json.dumps(pack_manifest(name, v), indent=2) + "\n").encode("utf-8")))
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w", format=tarfile.PAX_FORMAT) as tar:
        for rel, data in sorted(entries):
            info = tarfile.TarInfo("package/" + rel)
            info.size = len(data)
            info.mtime = NPM_MTIME
            info.mode = 0o644
            tar.addfile(info, io.BytesIO(data))
    # npm names a scoped package's tarball without the `@` and with `-` for `/`.
    out = dest / f"codelaw-rules-{name}-{v}.tgz"
    with open(out, "wb") as raw, gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as gz:
        gz.write(buf.getvalue())
    return out


def pack_module(name: str) -> str:
    """The Python package a pack's wheel installs, and its wheel's file-name
    distribution: `security-js` is `codelaw_rules_security_js` (the engine's
    FirstPartyPack::module; PEP 427 escapes `-` in a wheel name)."""
    return "codelaw_rules_" + name.replace("-", "_")


def wheel_for_pack(name: str, v: str, src: pathlib.Path, dest: pathlib.Path) -> pathlib.Path:
    """One pure py3-none-any wheel: package `codelaw_rules_<name>` holds the pack folder."""
    dist = pack_module(name)
    pv = pep440(v)
    info = f"{dist}-{pv}.dist-info"
    readme = (src / "README.md").read_text(encoding="utf-8")
    metadata = "\n".join([
        "Metadata-Version: 2.4",
        f"Name: codelaw-rules-{name}",
        f"Version: {pv}",
        f"Summary: {pack_meta(name)['description']}",
        f"License-Expression: {LICENCE}",
        *[f"License-File: {f}" for f in LICENCE_FILES],
        f"Project-URL: Repository, {REPO}",
        "Requires-Python: >=3.11",
        "Description-Content-Type: text/markdown",
        "",
        readme,
    ])
    wheel_txt = "Wheel-Version: 1.0\nGenerator: codelaw scripts/package.py\nRoot-Is-Purelib: true\nTag: py3-none-any\n"
    entries: list[tuple[str, bytes]] = []
    for f in sorted(p for p in src.rglob("*") if p.is_file()):
        entries.append((f"{dist}/{f.relative_to(src).as_posix()}", f.read_bytes()))
    entries.append((f"{info}/METADATA", metadata.encode()))
    entries.append((f"{info}/WHEEL", wheel_txt.encode()))
    for f in LICENCE_FILES:
        entries.append((f"{info}/licenses/{f}", (ROOT / f).read_bytes()))
    record = io.StringIO()
    for path, data in entries:
        digest = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()
        record.write(f"{path},sha256={digest},{len(data)}\n")
    record.write(f"{info}/RECORD,,\n")
    entries.append((f"{info}/RECORD", record.getvalue().encode()))
    out = dest / f"{dist}-{pv}-py3-none-any.whl"
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        for path, data in entries:
            zi = zipfile.ZipInfo(path, ZIP_DATE)
            zi.external_attr = 0o644 << 16
            zi.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(zi, data)
    return out


def assemble(bins: pathlib.Path, wheels: pathlib.Path, out: pathlib.Path) -> None:
    bins, wheels, out = bins.resolve(), wheels.resolve(), out.resolve()  # npm pack runs in each package folder
    v = versionmod.read_version()
    if out.exists():
        shutil.rmtree(out)
    stage, npm_out, pypi_out = out / "stage", out / "npm", out / "pypi"
    for d in (stage, npm_out, pypi_out):
        d.mkdir(parents=True)
    to_pack: list[pathlib.Path] = []

    # @codelaw/cli-<key> ×8: the binary and nothing else (decision 4)
    got = sorted(p.name for p in bins.iterdir() if p.is_dir()) if bins.is_dir() else []
    if got != sorted(TARGETS):
        fail(f"--bins must hold exactly the eight platform folders; missing {sorted(set(TARGETS) - set(got))}, extra {sorted(set(got) - set(TARGETS))}")
    for key, (os_, cpu, libc, triple, _) in TARGETS.items():
        d = stage / f"cli-{key}"
        d.mkdir()
        b = bins / key / exe(key)
        if not b.is_file():
            fail(f"{b} is missing")
        shutil.copyfile(b, d / exe(key))
        (d / exe(key)).chmod(0o755)
        copy_licences(d)
        (d / "README.md").write_text(f"# @codelaw/cli-{key}\n\nThe codelaw binary for `{triple}`. Install `codelaw`, not this package — npm picks this one for you.\n", encoding="utf-8")
        m = base_manifest(f"@codelaw/cli-{key}", v, f"The codelaw binary for {key}")
        m.update({"os": [os_], "cpu": [cpu], "files": [exe(key), *LICENCE_FILES], "preferUnplugged": True})
        if libc:
            m["libc"] = [libc]
        write_json(d / "package.json", m)
        to_pack.append(d)

    # npm codelaw: launcher, API, types; optionalDependencies only (decision 3)
    d = stage / "codelaw"
    shutil.copytree(ROOT / "packaging" / "npm" / "codelaw", d)
    shutil.copyfile(ROOT / "packaging" / "README.md", d / "README.md")
    shutil.copyfile(SCHEMA, d / "codelaw.schema.json")  # the config schema (SR-CONFIG decision 6)
    copy_licences(d)
    m = base_manifest("codelaw", v, "A linter and formatter whose rules are Markdown files")
    m.update({
        "type": "module",
        "bin": {"codelaw": "bin/codelaw.js"},
        "main": "index.js",
        "types": "index.d.ts",
        "exports": {".": {"types": "./index.d.ts", "default": "./index.js"}, "./package.json": "./package.json", "./codelaw.schema.json": "./codelaw.schema.json"},
        "files": ["bin/", "index.js", "index.d.ts", "codelaw.schema.json", *LICENCE_FILES],
        "engines": {"node": ">=22"},
        "keywords": ["linter", "formatter", "eslint", "ruff", "prettier"],
        "optionalDependencies": {f"@codelaw/cli-{k}": v for k in TARGETS},
    })
    write_json(d / "package.json", m)
    to_pack.append(d)

    # the dev-server plugins, npm only, no dependencies (decision 3a)
    stale = sync_dev_core(check=True)
    if stale:
        fail("run `python3 scripts/package.py sync-dev-core`:\n  " + "\n  ".join(stale))
    for folder, (pname, desc) in DEV_PLUGINS.items():
        d = stage / folder
        shutil.copytree(NPM_DIR / folder, d)
        copy_licences(d)
        m = base_manifest(pname, v, desc)
        m.update({
            "type": "module",
            "main": "index.js",
            "types": "index.d.ts",
            "exports": {".": {"types": "./index.d.ts", "default": "./index.js"}, "./package.json": "./package.json"},
            "files": [f.name for f in sorted(d.iterdir()) if f.suffix in (".js", ".ts", ".md")] + list(LICENCE_FILES),
            "engines": {"node": ">=22"},
            "keywords": ["codelaw", "linter", "dev-server"],
        })
        write_json(d / "package.json", m)
        to_pack.append(d)

    # the rule packs, each on the registries pack.rs names, the same files
    # on both where it is on both (decision 9, L6 decision 11)
    for name in PACKS:
        pd = stage_pack(name, stage)
        if name in PYPI_PACKS:
            wheel_for_pack(name, v, pd, pypi_out)
        if name in NPM_PACKS:
            pack_tarball(name, v, pd, npm_out)
    with concurrent.futures.ThreadPoolExecutor(8) as pool:
        list(pool.map(lambda d: npm_pack(d, npm_out), to_pack))

    # PyPI codelaw ×8: maturin's wheels, as built (decision 5)
    engine = sorted(wheels.glob("codelaw-*.whl")) if wheels.is_dir() else []
    for w in engine:
        shutil.copyfile(w, pypi_out / w.name)

    lines = []
    for f in sorted(list(npm_out.iterdir()) + list(pypi_out.iterdir())):
        lines.append(f"{hashlib.sha256(f.read_bytes()).hexdigest()}  {f.parent.name}/{f.name}")
    (out / "SHA256SUMS").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"✔ assembled {len(list(npm_out.iterdir()))} npm tarballs and {len(list(pypi_out.iterdir()))} wheels at {v} in {out}")


# ── check ──────────────────────────────────────────────────────────────────


def tar_files(t: pathlib.Path) -> dict[str, bytes]:
    with tarfile.open(t, "r:gz") as tf:
        return {m.name[len("package/"):]: tf.extractfile(m).read() for m in tf.getmembers() if m.isfile()}


def check(out: pathlib.Path) -> None:
    v = versionmod.read_version()
    pv = pep440(v)
    bad: list[str] = []

    # npm: exactly the SR-PACKAGES names (B-020), one version, zero dependencies
    npm: dict[str, dict[str, bytes]] = {}
    for t in sorted((out / "npm").glob("*.tgz")):
        files = tar_files(t)
        man = json.loads(files["package.json"])
        npm[man["name"]] = files
        if man["version"] != v:
            bad.append(f"{t.name}: version {man['version']} != {v}")
        if man.get("license") != LICENCE:
            bad.append(f"{t.name}: license {man.get('license')} != {LICENCE}")
        for f in LICENCE_FILES + ("README.md",):
            if f not in files:
                bad.append(f"{t.name}: has no {f}")
        for field in ("dependencies", "peerDependencies", "bundleDependencies", "bundledDependencies"):
            if man.get(field):
                bad.append(f"{t.name}: has {field} — every package has zero runtime dependencies (L6 decision 4)")
        if man["name"] != "codelaw" and man.get("optionalDependencies"):
            bad.append(f"{t.name}: only codelaw may have optionalDependencies")
        if "scripts" in man:
            bad.append(f"{t.name}: has lifecycle scripts")
    want_npm = {"codelaw"} | {f"@codelaw/cli-{k}" for k in TARGETS} | {f"@codelaw/rules-{n}" for n in NPM_PACKS} | {n for n, _ in DEV_PLUGINS.values()}
    bad.extend(sync_dev_core(check=True))
    if set(npm) != want_npm:
        bad.append(f"npm names: missing {sorted(want_npm - set(npm))}, not allowed {sorted(set(npm) - want_npm)}")
    if "codelaw" in npm and npm["codelaw"].get("codelaw.schema.json") != SCHEMA.read_bytes():
        bad.append(f"codelaw: codelaw.schema.json is missing or differs from {SCHEMA.relative_to(ROOT)} (SR-CONFIG decision 6)")
    if "codelaw" in npm:
        opt = json.loads(npm["codelaw"]["package.json"]).get("optionalDependencies", {})
        if opt != {f"@codelaw/cli-{k}": v for k in TARGETS}:
            bad.append(f"codelaw optionalDependencies are not exactly the eight binaries at {v}: {opt}")

    # PyPI: codelaw ×8 platform wheels and a pure codelaw-rules-* wheel per PyPI pack, nothing else, no sdist
    seen_tags: dict[str, str] = {}
    pypi_packs: dict[str, dict[str, bytes]] = {}
    for w in sorted((out / "pypi").iterdir()):
        m = re.match(r"^(codelaw(?:_rules_[a-z]+)?)-([^-]+)-([^-]+)-([^-]+)-([^-]+)\.whl$", w.name)
        if not m:
            bad.append(f"pypi: {w.name} is not a wheel codelaw publishes")
            continue
        dist, wv, py, abi, plat = m.groups()
        if wv != pv:
            bad.append(f"{w.name}: version {wv} != {pv}")
        with zipfile.ZipFile(w) as z:
            meta = z.read(f"{dist}-{wv}.dist-info/METADATA").decode()
            names = z.namelist()
            if re.search(r"(?m)^Requires-Dist:", meta):
                bad.append(f"{w.name}: has Requires-Dist — zero runtime dependencies")
            if f"License-Expression: {LICENCE}" not in meta:
                bad.append(f"{w.name}: METADATA has no License-Expression: {LICENCE}")
            for f in LICENCE_FILES:
                if not any(n.endswith(f"/licenses/{f}") for n in names):
                    bad.append(f"{w.name}: has no dist-info/licenses/{f}")
            if dist == "codelaw":
                key = next((k for k, t in TARGETS.items() if re.fullmatch(t[4], plat)), None)
                if key is None:
                    bad.append(f"{w.name}: platform {plat} is not one of the eight")
                    continue
                if key in seen_tags:
                    bad.append(f"{w.name}: a second wheel for {key}")
                seen_tags[key] = w.name
                _, data = wheel_binary(w)
                cli = npm.get(f"@codelaw/cli-{key}", {}).get(exe(key))
                if cli is not None and cli != data:
                    bad.append(f"{w.name}: its binary differs from @codelaw/cli-{key}'s — one binary per platform (L6 decision 2)")
                for f in ("codelaw/__init__.py", "codelaw/__main__.py", "codelaw/py.typed"):
                    if f not in names:
                        bad.append(f"{w.name}: has no {f}")
            else:
                if (py, abi, plat) != ("py3", "none", "any"):
                    bad.append(f"{w.name}: a pack wheel is py3-none-any")
                short = next((p for p in PYPI_PACKS if pack_module(p) == dist), dist[len("codelaw_rules_"):])
                prefix = f"{dist}/"
                pypi_packs[short] = {n[len(prefix):]: z.read(n) for n in names if n.startswith(prefix) and not n.endswith("/")}
    if sorted(seen_tags) != sorted(TARGETS):
        bad.append(f"PyPI codelaw wheels: missing {sorted(set(TARGETS) - set(seen_tags))}")
    if sorted(pypi_packs) != sorted(PYPI_PACKS):
        bad.append(f"PyPI packs: missing {sorted(set(PYPI_PACKS) - set(pypi_packs))}, not allowed {sorted(set(pypi_packs) - set(PYPI_PACKS))}")

    # pack parity: every published copy is packs/<name>/ (+ LICENSE), so a
    # pack on both registries holds the same files on both, byte for byte
    # (L6 decision 11, SR-RULE-PACKS decision 3)
    for name in PACKS:
        src = ROOT / "packs" / name
        tree = {p.relative_to(src).as_posix() for p in src.rglob("*") if p.is_file()}
        copies = {}
        if name in NPM_PACKS:
            copies["npm"] = {k: x for k, x in npm.get(f"@codelaw/rules-{name}", {}).items() if k != "package.json"}
        if name in PYPI_PACKS:
            copies["PyPI"] = pypi_packs.get(name, {})
        for reg, files in copies.items():
            if set(files) - set(LICENCE_FILES) != tree - set(LICENCE_FILES):
                bad.append(f"rules-{name}: the {reg} copy is not packs/{name}/ (+ LICENSE)")
        if len(copies) == 2 and copies["npm"] != copies["PyPI"]:
            a, b = copies["npm"], copies["PyPI"]
            diff = sorted(set(a) ^ set(b)) or sorted(k for k in a if a[k] != b.get(k))
            bad.append(f"rules-{name}: npm and PyPI differ ({len(diff)} files, e.g. {diff[:3]})")

    # SHA256SUMS: exactly the files that will be uploaded, each with its digest (W-18)
    sums = out / "SHA256SUMS"
    listed: dict[str, str] = {}
    if not sums.is_file():
        bad.append("SHA256SUMS is missing")
    else:
        for line in sums.read_text(encoding="utf-8").splitlines():
            m = re.fullmatch(r"([0-9a-f]{64})  ((?:npm|pypi)/[^/]+)", line)
            if not m:
                bad.append(f"SHA256SUMS: malformed line {line!r}")
                continue
            listed[m.group(2)] = m.group(1)
    present = {f"{d}/{f.name}" for d in ("npm", "pypi") if (out / d).is_dir() for f in (out / d).iterdir() if f.is_file()}
    if sums.is_file() and set(listed) != present:
        bad.append(f"SHA256SUMS lists {sorted(set(listed) - present)[:3]} not in dist, and omits {sorted(present - set(listed))[:3]}")
    for rel, digest in listed.items():
        f = out / rel
        if f.is_file() and hashlib.sha256(f.read_bytes()).hexdigest() != digest:
            bad.append(f"{rel}: its sha256 is not the one SHA256SUMS lists")

    if bad:
        fail(f"{len(bad)} release check(s) failed:\n  " + "\n  ".join(bad))
    print(f"✔ {len(npm)} npm packages and {len(seen_tags) + len(pypi_packs)} wheels at {v}: only the SR-PACKAGES names, one version, "
          "zero dependencies, licensed, one binary per platform, each pack its folder on every registry it is on, SHA256SUMS true")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("prepare-wheel")
    e = sub.add_parser("extract")
    e.add_argument("wheel", type=pathlib.Path)
    e.add_argument("outdir", type=pathlib.Path)
    a = sub.add_parser("assemble")
    a.add_argument("--bins", type=pathlib.Path, required=True)
    a.add_argument("--wheels", type=pathlib.Path, required=True)
    a.add_argument("--out", type=pathlib.Path, required=True)
    c = sub.add_parser("check")
    c.add_argument("out", type=pathlib.Path)
    sub.add_parser("sync-dev-core")
    sub.add_parser("names", help="every name codelaw publishes, one `registry name` per line")
    args = ap.parse_args()
    if args.cmd == "prepare-wheel":
        prepare_wheel()
    elif args.cmd == "extract":
        extract(args.wheel, args.outdir)
    elif args.cmd == "assemble":
        assemble(args.bins, args.wheels, args.out)
    elif args.cmd == "check":
        check(args.out)
    elif args.cmd == "sync-dev-core":
        sync_dev_core()
    else:
        for n in ["codelaw", *[f"@codelaw/cli-{k}" for k in TARGETS], *[f"@codelaw/rules-{x}" for x in NPM_PACKS], *[n for n, _ in DEV_PLUGINS.values()]]:
            print(f"npm {n}")
        for n in ["codelaw", *[f"codelaw-rules-{x}" for x in PYPI_PACKS]]:
            print(f"pypi {n}")


if __name__ == "__main__":
    main()
