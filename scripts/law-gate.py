#!/usr/bin/env python3
"""law-gate — refuse a release whose engine is not complete.

L7: codelaw is a linter AND a formatter, and neither ships without the other.
L6 decision 3: every `codelaw` package carries every language, the formatter,
the MCP server and the LSP. A binary missing any of `format`, `mcp` or `lsp`
must never reach a registry (SR-PACKAGES decision 11).

    python3 scripts/law-gate.py dist/             # the binary for this host, from dist/npm
    python3 scripts/law-gate.py --binary PATH     # a binary you already have
    python3 scripts/law-gate.py dist/ --report    # say what is missing, exit 0 (ci.yml)

Run by publish.yml's gate on every release, and by the decider before the
one-time hand publish (RELEASING.md). Exit 1 when the engine is incomplete.
"""
import argparse
import os
import pathlib
import platform
import subprocess
import sys
import tarfile
import tempfile

# command -> the law that requires it, and the work item that builds it
REQUIRED = {
    "format": ("L7; L6 decision 3", "W-07"),
    "mcp": ("L6 decision 3", "W-08"),
    "lsp": ("L6 decision 3", "W-09"),
}


def host_key() -> str:
    os_ = {"darwin": "darwin", "linux": "linux", "win32": "win32"}.get(sys.platform)
    cpu = {"x86_64": "x64", "amd64": "x64", "arm64": "arm64", "aarch64": "arm64"}.get(platform.machine().lower())
    if not os_ or not cpu:
        sys.exit(f"law-gate: no codelaw binary is built for {sys.platform}/{platform.machine()}")
    return f"{os_}-{cpu}-gnu" if os_ == "linux" else f"{os_}-{cpu}"


def binary_from_dist(dist: pathlib.Path, into: pathlib.Path) -> pathlib.Path:
    key = host_key()
    tgz = sorted((dist / "npm").glob(f"codelaw-cli-{key}-*.tgz"))
    if len(tgz) != 1:
        sys.exit(f"law-gate: expected one {dist}/npm/codelaw-cli-{key}-*.tgz, found {len(tgz)}")
    name = "package/codelaw.exe" if key.startswith("win32") else "package/codelaw"
    with tarfile.open(tgz[0], "r:gz") as tf:
        data = tf.extractfile(tf.getmember(name)).read()
    b = into / pathlib.Path(name).name
    b.write_bytes(data)
    b.chmod(0o755)
    return b


def missing_commands(binary: pathlib.Path) -> list[str]:
    out = []
    for cmd in REQUIRED:
        r = subprocess.run([str(binary), cmd, "--help"], capture_output=True, timeout=30)
        if r.returncode != 0:
            out.append(cmd)
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("dist", nargs="?", type=pathlib.Path, help="the assembled dist/ folder")
    ap.add_argument("--binary", type=pathlib.Path, help="check this binary instead of dist/")
    ap.add_argument("--report", action="store_true", help="report only; always exit 0")
    a = ap.parse_args()
    if not a.binary and not a.dist:
        ap.error("give dist/ or --binary")

    with tempfile.TemporaryDirectory() as tmp:
        binary = a.binary or binary_from_dist(a.dist, pathlib.Path(tmp))
        missing = missing_commands(binary)

    if not missing:
        print("law-gate: the engine is complete — format, mcp and lsp are present (L6 decision 3, L7)")
        return
    lines = [f"  - no `codelaw {c}` — required by {REQUIRED[c][0]}; built by {REQUIRED[c][1]}" for c in missing]
    msg = "the engine is incomplete, so it may not be released:\n" + "\n".join(lines)
    if a.report:
        if "GITHUB_ACTIONS" in os.environ:
            print("::warning title=law-gate::" + msg.replace("\n", "%0A"))
        else:
            print("law-gate (report): " + msg)
        return
    print("law-gate: refusing to release — " + msg, file=sys.stderr)
    sys.exit(1)


if __name__ == "__main__":
    main()
