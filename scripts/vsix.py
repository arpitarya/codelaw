#!/usr/bin/env python3
"""vsix — the editor extension's eight platform packages (SR-PACKAGES decision 6; W-95).

`codelaw.codelaw-vscode` ships one `.vsix` per platform, each with that platform's
`bin/codelaw` beside the compiled extension (SR-EDITOR decision 4, step 4 of
the binary lookup). A `.vsix` is a zip in the Open Packaging Conventions
layout: `extension.vsixmanifest`, `[Content_Types].xml` and the extension's
files under `extension/`. This writes it with the standard library, as
`scripts/package.py` writes the npm tarballs without `npm pack` — `@vscode/vsce`
is not a ratified dependency (L11), and its tree is not L2-clean.

    vsix.py build --bins B --out dist    one .vsix per platform into dist/vscode/, and its SHA256SUMS
    vsix.py check dist                   the checks on what was built (--bins B: the binary is that build's)

Before `build`: `npm ci && npm test` in `packaging/vscode` (it reads the
compiled `out/src/` and the runtime packages of `node_modules/`, the ones
`package-lock.json` does not mark `dev`). `--bins` is the folder of `ci.yml`'s
build jobs, one per platform key (`linux-x64-gnu/codelaw`, ...).

The output is `dist/vscode/`, **never** `dist/npm` or `dist/pypi`: publish.yml
uploads those two folders whole, and the extension is published only after
the decider has created the publisher and the tokens (W-95, RELEASING.md).

Python 3.9 syntax (work/MACHINE.md); the standard library only.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import struct
import sys
import zipfile
from xml.sax.saxutils import escape

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import package as pkg  # noqa: E402
import version as versionmod  # noqa: E402

ROOT = versionmod.ROOT
EXT = ROOT / "packaging" / "vscode"

# codelaw's platform key -> VS Code's target platform (`vsce package --target`).
# Alpine is VS Code's name for the musl builds.
TARGETS = {
    "darwin-arm64": "darwin-arm64",
    "darwin-x64": "darwin-x64",
    "linux-x64-gnu": "linux-x64",
    "linux-arm64-gnu": "linux-arm64",
    "linux-x64-musl": "alpine-x64",
    "linux-arm64-musl": "alpine-arm64",
    "win32-x64": "win32-x64",
    "win32-arm64": "win32-arm64",
}
assert set(TARGETS) == set(pkg.TARGETS), "vsix.py TARGETS and package.py TARGETS name different platforms"

# The extension's own files, as installed (`extension/<path>`).
OWN = ("package.json", "README.md", "icon.png")
SKIP_IN_MODULES = (".d.ts", ".map", ".ts")  # types and source maps: not run
CONTENT_TYPES = {  # any other extension is application/octet-stream, as vsce writes it
    ".js": "application/javascript",
    ".json": "application/json",
    ".md": "text/markdown",
    ".txt": "text/plain",
    ".png": "image/png",
    ".vsixmanifest": "text/xml",
}
MIN_ICON = 128  # the Marketplace's smallest icon


def fail(msg: str) -> None:
    print("✗ " + msg, file=sys.stderr)
    sys.exit(1)


def manifest() -> dict:
    return json.loads((EXT / "package.json").read_text(encoding="utf-8"))


def runtime_modules() -> list[pathlib.Path]:
    """Every file of the runtime packages `package-lock.json` names (not `dev`)."""
    lock = json.loads((EXT / "package-lock.json").read_text(encoding="utf-8"))
    files: list[pathlib.Path] = []
    for key, meta in sorted(lock["packages"].items()):
        if not key or meta.get("dev"):
            continue
        d = EXT / key
        if not d.is_dir():
            fail(f"{key} is a runtime package and is not installed: run `npm ci` in packaging/vscode first")
        for f in sorted(d.rglob("*")):
            rel = f.relative_to(d)
            if f.is_file() and not f.name.endswith(SKIP_IN_MODULES) and not (rel.parts[0] == "node_modules"):
                files.append(f)
    return files


def extension_files() -> dict[str, bytes]:
    """`extension/<path>` -> bytes, for everything but the binary."""
    m = manifest()
    main = EXT / m["main"]
    if not main.is_file():
        fail(f"{m['main']} is missing: run `npm ci && npm test` in packaging/vscode first")
    out = {f"extension/{n}": (EXT / n).read_bytes() for n in OWN}
    out["extension/LICENSE.txt"] = (ROOT / "LICENSE").read_bytes()
    for f in sorted((EXT / "out" / "src").rglob("*.js")):
        out["extension/" + f.relative_to(EXT).as_posix()] = f.read_bytes()
    for f in runtime_modules():
        out["extension/" + f.relative_to(EXT).as_posix()] = f.read_bytes()
    return out


def vsixmanifest(m: dict, target: str) -> str:
    """The `extension.vsixmanifest` the Marketplace and Open VSX read, as `vsce` writes it."""
    repo = m["repository"]["url"]
    links = {
        "Source": repo,
        "Getstarted": repo,
        "GitHub": repo,
        "Support": m["bugs"]["url"],
        "Learn": m["homepage"],
    }
    props = [
        ("Microsoft.VisualStudio.Code.Engine", m["engines"]["vscode"]),
        ("Microsoft.VisualStudio.Code.ExtensionDependencies", ""),
        ("Microsoft.VisualStudio.Code.ExtensionPack", ""),
        ("Microsoft.VisualStudio.Code.ExtensionKind", "workspace"),
        ("Microsoft.VisualStudio.Code.LocalizedLanguages", ""),
        ("Microsoft.VisualStudio.Code.ExecutesCode", "true"),
        *[(f"Microsoft.VisualStudio.Services.Links.{k}", v) for k, v in links.items()],
        ("Microsoft.VisualStudio.Services.Branding.Color", m["galleryBanner"]["color"]),
        ("Microsoft.VisualStudio.Services.Branding.Theme", m["galleryBanner"]["theme"]),
        ("Microsoft.VisualStudio.Services.GitHubFlavoredMarkdown", "true"),
        ("Microsoft.VisualStudio.Services.Content.Pricing", m.get("pricing", "Free")),
    ]
    flags = "Public Preview" if m.get("preview") else "Public"
    a = lambda s: escape(s, {'"': "&quot;"})  # noqa: E731 — an attribute value
    return "\n".join([
        '<?xml version="1.0" encoding="utf-8"?>',
        '<PackageManifest Version="2.0.0" xmlns="http://schemas.microsoft.com/developer/vsx-schema/2011" '
        'xmlns:d="http://schemas.microsoft.com/developer/vsx-schema-design/2011">',
        "  <Metadata>",
        f'    <Identity Language="en-US" Id="{a(m["name"])}" Version="{a(m["version"])}" Publisher="{a(m["publisher"])}" TargetPlatform="{target}"/>',
        f"    <DisplayName>{escape(m['displayName'])}</DisplayName>",
        f'    <Description xml:space="preserve">{escape(m["description"])}</Description>',
        f"    <Tags>{escape(','.join(m.get('keywords', [])))}</Tags>",
        f"    <Categories>{escape(','.join(m['categories']))}</Categories>",
        f"    <GalleryFlags>{flags}</GalleryFlags>",
        "    <Properties>",
        *[f'      <Property Id="{k}" Value="{a(v)}"/>' for k, v in props],
        "    </Properties>",
        "    <License>extension/LICENSE.txt</License>",
        f"    <Icon>extension/{m['icon']}</Icon>",
        "  </Metadata>",
        "  <Installation>",
        '    <InstallationTarget Id="Microsoft.VisualStudio.Code"/>',
        "  </Installation>",
        "  <Dependencies/>",
        "  <Assets>",
        '    <Asset Type="Microsoft.VisualStudio.Code.Manifest" Path="extension/package.json" Addressable="true"/>',
        '    <Asset Type="Microsoft.VisualStudio.Services.Content.Details" Path="extension/README.md" Addressable="true"/>',
        '    <Asset Type="Microsoft.VisualStudio.Services.Content.License" Path="extension/LICENSE.txt" Addressable="true"/>',
        f'    <Asset Type="Microsoft.VisualStudio.Services.Icons.Default" Path="extension/{a(m["icon"])}" Addressable="true"/>',
        "  </Assets>",
        "</PackageManifest>",
        "",
    ])


def content_types(names: list[str]) -> str:
    """`[Content_Types].xml`: a Default per extension, an Override per file with none (the binary on Unix)."""
    exts = sorted({pathlib.PurePosixPath(n).suffix.lower() for n in names} - {""})
    bare = sorted(n for n in names if not pathlib.PurePosixPath(n).suffix)
    return "\n".join([
        '<?xml version="1.0" encoding="utf-8"?>',
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">',
        *[f'  <Default Extension="{e}" ContentType="{CONTENT_TYPES.get(e, "application/octet-stream")}"/>' for e in exts],
        *[f'  <Override PartName="/{escape(n)}" ContentType="application/octet-stream"/>' for n in bare],
        "</Types>",
        "",
    ])


def write_zip(path: pathlib.Path, entries: dict[str, bytes], executable: set[str]) -> None:
    """Deterministic: sorted names, a fixed date, the mode in the external attributes."""
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for name in sorted(entries):
            zi = zipfile.ZipInfo(name, pkg.ZIP_DATE)
            zi.compress_type = zipfile.ZIP_DEFLATED
            zi.external_attr = (0o100755 if name in executable else 0o100644) << 16
            z.writestr(zi, entries[name])


def vsix_name(m: dict, target: str) -> str:
    return f"{m['name']}-{target}-{m['version']}.vsix"  # vsce's own file name


def build(bins: pathlib.Path, out: pathlib.Path) -> None:
    m = manifest()
    v = versionmod.read_version()
    if m["version"] != v:
        fail(f"packaging/vscode/package.json is {m['version']}, the version is {v}: run scripts/version.py --set {v}")
    base = extension_files()
    dest = out / "vscode"
    dest.mkdir(parents=True, exist_ok=True)
    sums = []
    for key, target in TARGETS.items():
        exe = pkg.exe(key)
        b = bins / key / exe
        if not b.is_file():
            fail(f"no binary for {key} at {b}")
        entries = dict(base)
        entries[f"extension/bin/{exe}"] = b.read_bytes()
        entries["extension.vsixmanifest"] = vsixmanifest(m, target).encode()
        entries["[Content_Types].xml"] = content_types(list(entries)).encode()
        f = dest / vsix_name(m, target)
        write_zip(f, entries, {f"extension/bin/{exe}"})
        sums.append(f"{hashlib.sha256(f.read_bytes()).hexdigest()}  {f.name}")
        print(f"→ {f.relative_to(out)} ({f.stat().st_size // 1024} KiB)")
    (dest / "SHA256SUMS").write_text("\n".join(sums) + "\n", encoding="utf-8")
    print(f"✔ {len(TARGETS)} .vsix files of {m['publisher']}.{m['name']} {v} in {dest}")


def png_size(data: bytes) -> tuple[int, int]:
    if data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
        return (0, 0)
    return struct.unpack(">II", data[16:24])


def check(out: pathlib.Path, bins: pathlib.Path | None, release: bool) -> None:
    m = manifest()
    v = versionmod.read_version()
    dest = out / "vscode"
    bad: list[str] = []
    if release and "-" in v:
        bad.append(f"{v}: the Marketplace takes major.minor.patch only (a pre-release is a flag, not a version)")
    for field in ("publisher", "name", "displayName", "description", "version", "license", "icon", "repository", "engines", "categories"):
        if not m.get(field):
            bad.append(f"package.json has no `{field}` (the Marketplace needs it)")
    readme = (EXT / "README.md").read_text(encoding="utf-8")
    for link in re.findall(r"\]\(([^)]+)\)", readme):
        if not re.match(r"^(https?:|#|mailto:)", link):
            bad.append(f"README.md links to `{link}`: a relative link breaks on the Marketplace page")
    want = {vsix_name(m, t) for t in TARGETS.values()}
    have = {f.name for f in dest.glob("*.vsix")}
    if want != have:
        bad.append(f"dist/vscode holds {sorted(have)}, not the {len(want)} platform packages {sorted(want)}")
    listed = {}
    sums = dest / "SHA256SUMS"
    for line in (sums.read_text(encoding="utf-8").splitlines() if sums.is_file() else []):
        h, n = line.split("  ", 1)
        listed[n] = h
    for key, target in TARGETS.items():
        f = dest / vsix_name(m, target)
        if not f.is_file():
            continue
        if listed.get(f.name) != hashlib.sha256(f.read_bytes()).hexdigest():
            bad.append(f"{f.name}: SHA256SUMS does not hold its digest")
        with zipfile.ZipFile(f) as z:
            names = set(z.namelist())
            exe = f"extension/bin/{pkg.exe(key)}"
            need = {"extension.vsixmanifest", "[Content_Types].xml", "extension/package.json", "extension/README.md",
                    "extension/LICENSE.txt", f"extension/{m['icon']}", "extension/" + m["main"].lstrip("./"), exe}
            for n in sorted(need - names):
                bad.append(f"{f.name}: no {n}")
            other = [n for n in names if n.startswith("extension/bin/") and n != exe]
            if other:
                bad.append(f"{f.name}: a binary of another platform: {other}")
            if any(n.startswith(("extension/src/", "extension/test/", "extension/out/test/")) for n in names):
                bad.append(f"{f.name}: holds the extension's sources or tests")
            if any("/node_modules/" in n and re.search(r"/(typescript|@types)/", n) for n in names):
                bad.append(f"{f.name}: holds a dev dependency")
            if exe in names:
                if (z.getinfo(exe).external_attr >> 16) & 0o111 == 0:
                    bad.append(f"{f.name}: {exe} is not executable")
                if bins is not None and z.read(exe) != (bins / key / pkg.exe(key)).read_bytes():
                    bad.append(f"{f.name}: {exe} is not the {key} build's binary")
            if "extension.vsixmanifest" in names:
                x = z.read("extension.vsixmanifest").decode()
                ident = f'Id="{m["name"]}" Version="{v}" Publisher="{m["publisher"]}" TargetPlatform="{target}"'
                if ident not in x:
                    bad.append(f"{f.name}: the manifest's identity is not {ident}")
            if "extension/package.json" in names and json.loads(z.read("extension/package.json"))["version"] != v:
                bad.append(f"{f.name}: package.json is not at {v}")
            if "extension/LICENSE.txt" in names and z.read("extension/LICENSE.txt") != (ROOT / "LICENSE").read_bytes():
                bad.append(f"{f.name}: LICENSE.txt is not the repository's LICENSE")
            icon = f"extension/{m['icon']}"
            if icon in names:
                w, h = png_size(z.read(icon))
                if min(w, h) < MIN_ICON:
                    bad.append(f"{f.name}: the icon is {w}x{h}; the Marketplace wants a PNG of at least {MIN_ICON}x{MIN_ICON}")
    if bad:
        fail(f"{len(bad)} extension check(s) failed:\n  " + "\n  ".join(bad))
    print(f"✔ {len(TARGETS)} .vsix files of {m['publisher']}.{m['name']} {v}: one per platform, each its own binary"
          + (" (the build's)" if bins else "") + ", the manifest's identity, licence, icon, no sources or dev dependencies, SHA256SUMS true")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("build")
    b.add_argument("--bins", type=pathlib.Path, required=True)
    b.add_argument("--out", type=pathlib.Path, required=True)
    c = sub.add_parser("check")
    c.add_argument("out", type=pathlib.Path)
    c.add_argument("--bins", type=pathlib.Path)
    c.add_argument("--release", action="store_true", help="also refuse what the Marketplace refuses at upload (a pre-release version)")
    args = ap.parse_args()
    if args.cmd == "build":
        build(args.bins, args.out)
    else:
        check(args.out, args.bins, args.release)


if __name__ == "__main__":
    main()
