#!/usr/bin/env python3
"""build-packs — the five rule packs as npm tarballs and PyPI wheels.

SR-PACKAGES decision 9 and SR-RULE-PACKS decision 3: each `packs/<name>/` is
built into `@codelaw/rules-<name>` (an npm tarball whose `package/` is the
folder) and `codelaw-rules-<name>` (one pure `py3-none-any` wheel whose
`codelaw_rules_<name>/` is the folder). Both hold the same files, byte for
byte; only the registry's own metadata (`package.json`, `*.dist-info/`) is
added. The version comes from the one file, the workspace `Cargo.toml`
(SR-LAW-6 decision 9).

    python3 scripts/build-packs.py                    # build into dist/packs/
    python3 scripts/build-packs.py --verify CODELAW   # and install both ways, compare
    python3 scripts/build-packs.py --release          # refuse to build without a LICENSE

Stdlib only, Python 3.9 syntax (work/MACHINE.md). Nothing is published.
"""

import argparse
import base64
import gzip
import hashlib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PACKS = ["js", "html", "css", "python", "data"]
# npm's own fixed mtime for tarball entries (1985-10-26T08:15:00Z).
NPM_MTIME = 499162500
ZIP_DATE = (1980, 1, 1, 0, 0, 0)
REQUIRES_PYTHON = ">=3.11"  # SR-LAW-6 decision 8


def fail(msg):
    print("build-packs: " + msg, file=sys.stderr)
    sys.exit(1)


def workspace_version():
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    section = re.search(r"^\[workspace\.package\]\s*$(.*?)(?=^\[|\Z)", text, re.M | re.S)
    match = section and re.search(r'^version\s*=\s*"([^"]+)"', section.group(1), re.M)
    if not match:
        fail("no `version` under [workspace.package] in Cargo.toml")
    return match.group(1)


def manifest_field(text, key):
    match = re.search(r"^" + key + r'\s*=\s*"([^"]*)"', text, re.M)
    return match.group(1) if match else None


def pack_files(pack_dir):
    """Every file of the pack, as (posix path relative to the pack, bytes), sorted."""
    out = []
    for path in sorted(pack_dir.rglob("*")):
        rel = path.relative_to(pack_dir)
        if any(part.startswith(".") for part in rel.parts):
            continue
        if path.is_symlink():
            fail("{}: symlinks are not packed".format(path))
        if path.is_file():
            out.append((rel.as_posix(), path.read_bytes()))
    return out


def read_pack(name, version, release):
    pack_dir = ROOT / "packs" / name
    manifest = (pack_dir / "codelaw-pack.toml").read_text(encoding="utf-8")
    npm_name = "@codelaw/rules-" + name
    if manifest_field(manifest, "name") != npm_name:
        fail("{}: `name` must be {!r}".format(pack_dir / "codelaw-pack.toml", npm_name))
    if manifest_field(manifest, "version") != version:
        fail(
            "{}: `version = {!r}` but Cargo.toml says {!r}; one version for everything (SR-LAW-6 decision 9)".format(
                pack_dir / "codelaw-pack.toml", manifest_field(manifest, "version"), version
            )
        )
    files = pack_files(pack_dir)
    names = {path for path, _ in files}
    licensed = "LICENSE" in names
    if not licensed and release:
        fail("{}: no LICENSE — the project licence is the decider's ruling (SR-RULE-PACKS decision 3)".format(pack_dir))
    return {
        "name": name,
        "npm": npm_name,
        "pypi": "codelaw-rules-" + name,
        "module": "codelaw_rules_" + name,
        "description": manifest_field(manifest, "description") or "",
        "files": files,
        "licensed": licensed,
    }


def licence_id():
    """The SPDX id, read from the line `SPDX-License-Identifier: <id>` in the root LICENSE."""
    path = ROOT / "LICENSE"
    if not path.is_file():
        return None
    match = re.search(r"SPDX-License-Identifier:\s*(\S+)", path.read_text(encoding="utf-8"))
    return match.group(1) if match else None


def package_json(pack, version, spdx):
    data = {
        "name": pack["npm"],
        "version": version,
        "description": pack["description"],
        "keywords": ["codelaw", "codelaw-pack", "lint"],
        "homepage": "https://github.com/arpitarya/codelaw",
        "repository": {"type": "git", "url": "git+https://github.com/arpitarya/codelaw.git", "directory": "packs/" + pack["name"]},
    }
    if spdx:
        data["license"] = spdx
    return (json.dumps(data, indent=2) + "\n").encode("utf-8")


def build_npm(pack, version, spdx, out_dir):
    buf = io.BytesIO()
    entries = list(pack["files"]) + [("package.json", package_json(pack, version, spdx))]
    with tarfile.open(fileobj=buf, mode="w", format=tarfile.PAX_FORMAT) as tar:
        for rel, data in sorted(entries):
            info = tarfile.TarInfo("package/" + rel)
            info.size = len(data)
            info.mtime = NPM_MTIME
            info.mode = 0o644
            tar.addfile(info, io.BytesIO(data))
    # npm names a scoped package's tarball without the `@` and with `-` for `/`.
    path = out_dir / "codelaw-rules-{}-{}.tgz".format(pack["name"], version)
    with open(path, "wb") as raw, gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as gz:
        gz.write(buf.getvalue())
    return path


def record_hash(data):
    digest = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode("ascii")
    return "sha256=" + digest


def build_wheel(pack, version, spdx, out_dir):
    dist_info = "{}-{}.dist-info".format(pack["module"], version)
    metadata = [
        "Metadata-Version: 2.4",
        "Name: " + pack["pypi"],
        "Version: " + version,
        "Summary: " + pack["description"],
        "Home-page: https://github.com/arpitarya/codelaw",
        "Keywords: codelaw,codelaw-pack,lint",
        "Requires-Python: " + REQUIRES_PYTHON,
    ]
    if spdx:
        metadata.append("License-Expression: " + spdx)
        metadata.append("License-File: {}/LICENSE".format(pack["module"]))
    wheel = ["Wheel-Version: 1.0", "Generator: codelaw build-packs", "Root-Is-Purelib: true", "Tag: py3-none-any"]
    entries = [(pack["module"] + "/" + rel, data) for rel, data in pack["files"]]
    entries.append((dist_info + "/METADATA", ("\n".join(metadata) + "\n").encode("utf-8")))
    entries.append((dist_info + "/WHEEL", ("\n".join(wheel) + "\n").encode("utf-8")))
    record_name = dist_info + "/RECORD"
    record = "".join("{},{},{}\n".format(n, record_hash(d), len(d)) for n, d in entries) + record_name + ",,\n"
    entries.append((record_name, record.encode("utf-8")))
    path = out_dir / "{}-{}-py3-none-any.whl".format(pack["module"], version)
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as zf:
        for name, data in entries:
            info = zipfile.ZipInfo(name, ZIP_DATE)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            zf.writestr(info, data)
    return path


def npm_contents(path):
    with tarfile.open(path, "r:gz") as tar:
        return {m.name[len("package/"):]: tar.extractfile(m).read() for m in tar.getmembers() if m.isfile()}


def wheel_contents(path, module):
    with zipfile.ZipFile(path) as zf:
        return {n[len(module) + 1:]: zf.read(n) for n in zf.namelist() if n.startswith(module + "/")}


def check_parity(pack, npm_path, wheel_path):
    """SR-LAW-6 'Pack parity': the two artefacts hold the pack's files, byte for byte."""
    expected = dict(pack["files"])
    npm = npm_contents(npm_path)
    npm.pop("package.json", None)
    wheel = wheel_contents(wheel_path, pack["module"])
    for label, got in (("npm", npm), ("wheel", wheel)):
        if got != expected:
            missing = sorted(set(expected) - set(got))
            extra = sorted(set(got) - set(expected))
            changed = sorted(k for k in set(got) & set(expected) if got[k] != expected[k])
            fail("{} {}: differs from packs/{} (missing {}, extra {}, changed {})".format(
                pack["npm"], label, pack["name"], missing[:3], extra[:3], changed[:3]))


def verify(codelaw, built, version):
    """Install every pack from npm's layout and from a venv's, check the L3 fixtures, compare."""
    presets = []
    for pack, _, _ in built:
        manifest = (ROOT / "packs" / pack["name"] / "codelaw-pack.toml").read_text(encoding="utf-8")
        listed = re.search(r"^presets\s*=\s*\[(.*?)\]", manifest, re.M | re.S)
        presets += [p for p in re.findall(r'"([^"]+)"', listed.group(1)) if p.endswith("/recommended")]
    config = "preview = true\nextends = [{}]\n".format(", ".join('"codelaw:{}"'.format(p) for p in presets))
    env = {k: v for k, v in os.environ.items() if k not in ("VIRTUAL_ENV", "CODELAW_SITE_PACKAGES")}
    outputs = {}
    with tempfile.TemporaryDirectory() as tmp:
        for layout in ("npm", "pypi"):
            project = Path(tmp) / layout
            shutil.copytree(ROOT / "fixtures" / "l3", project / "src", ignore=shutil.ignore_patterns("rules", "README.md"))
            (project / "codelaw.toml").write_text(config, encoding="utf-8")
            for pack, npm_path, wheel_path in built:
                if layout == "npm":
                    dest = project / "node_modules" / "@codelaw" / ("rules-" + pack["name"])
                    for rel, data in npm_contents(npm_path).items():
                        (dest / rel).parent.mkdir(parents=True, exist_ok=True)
                        (dest / rel).write_bytes(data)
                else:
                    with zipfile.ZipFile(wheel_path) as zf:
                        zf.extractall(project / ".venv" / "lib" / "python3.11" / "site-packages")
            run = subprocess.run([codelaw, "check", "--format", "json"], cwd=str(project), env=env, capture_output=True)
            if run.returncode not in (0, 1):
                fail("verify ({}): `codelaw check` exited {}:\n{}".format(layout, run.returncode, run.stderr.decode("utf-8", "replace")))
            outputs[layout] = run.stdout
    if outputs["npm"] != outputs["pypi"]:
        fail("verify: `codelaw check` output differs between the npm and the PyPI installs")
    report = json.loads(outputs["npm"].decode("utf-8"))
    diagnostics = report.get("diagnostics", report) if isinstance(report, dict) else report
    print("verify: {} presets from both layouts, {} diagnostics on fixtures/l3, byte-identical output".format(
        len(presets), len(diagnostics)))


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--out", default=str(ROOT / "dist" / "packs"), help="output folder (default dist/packs)")
    parser.add_argument("--release", action="store_true", help="refuse a pack without LICENSE")
    parser.add_argument("--verify", metavar="CODELAW", help="a codelaw binary: install both ways and compare `check` output")
    args = parser.parse_args()

    version = workspace_version()
    spdx = licence_id()
    if args.release and not spdx:
        fail("no `SPDX-License-Identifier:` line in LICENSE — the project licence is the decider's ruling")
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    built = []
    for name in PACKS:
        pack = read_pack(name, version, args.release)
        npm_path = build_npm(pack, version, spdx, out_dir)
        wheel_path = build_wheel(pack, version, spdx, out_dir)
        check_parity(pack, npm_path, wheel_path)
        built.append((pack, npm_path, wheel_path))
        note = "" if pack["licensed"] and spdx else "  (no licence: not publishable)"
        print("{}@{}: {} files -> {}, {}{}".format(pack["npm"], version, len(pack["files"]), npm_path.name, wheel_path.name, note))
    if args.verify:
        verify(str(Path(args.verify).resolve()), built, version)


if __name__ == "__main__":
    main()
