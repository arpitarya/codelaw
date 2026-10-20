"""codelaw on PyPI: the Python API over the bundled binary (SR-PACKAGES decision 5).

Each call runs the binary this wheel installed and returns its ``--format json``
report unchanged, so the results are the binary's results (L6 decision 5).
"""

from __future__ import annotations

import json
import os
import site
import subprocess
import sys
import sysconfig
from importlib import metadata
from collections.abc import Sequence
from typing import Any

__all__ = ["binary_path", "check", "fix"]


def binary_path() -> str:
    """The absolute path of the codelaw binary installed with this package."""
    override = os.environ.get("CODELAW_BINARY")
    if override:
        return override
    exe = "codelaw.exe" if sys.platform == "win32" else "codelaw"
    # The file list this distribution installed: never another `codelaw` on the
    # same bin folder (an npm -g shim, say).
    try:
        for f in metadata.files("codelaw") or ():
            if f.name == exe and f.parent.name in ("bin", "Scripts", "scripts"):
                path = str(f.locate())
                if os.path.isfile(path):
                    return path
    except metadata.PackageNotFoundError:
        pass
    schemes = [sysconfig.get_default_scheme()]
    if sys.platform == "win32":
        schemes.append("nt_user")
    elif sys.platform == "darwin" and sys._framework:  # type: ignore[attr-defined]
        schemes.append("osx_framework_user")
    else:
        schemes.append("posix_user")
    for scheme in schemes:
        candidate = os.path.join(sysconfig.get_path("scripts", scheme), exe)
        if os.path.isfile(candidate):
            return candidate
    # pip --target / an unusual layout: <prefix>/bin beside this package's site-packages.
    here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    for parent in (here, os.path.dirname(here), os.path.dirname(os.path.dirname(here))):
        for sub in ("bin", "Scripts"):
            candidate = os.path.join(parent, sub, exe)
            if os.path.isfile(candidate):
                return candidate
    raise FileNotFoundError("codelaw: the binary this package installs is missing; reinstall codelaw")


def environment() -> dict[str, str]:
    """The binary's environment: this interpreter's site-packages, so it finds the
    codelaw-rules-* packs installed beside this package (SR-RULE-PACKS decision 4)."""
    paths = list(site.getsitepackages())
    user = site.getusersitepackages()
    if isinstance(user, str) and os.path.isdir(user):
        paths.append(user)
    env = dict(os.environ)
    env["CODELAW_SITE_PACKAGES"] = os.pathsep.join(paths)
    return env


def _run(args: list[str]) -> dict[str, Any]:
    r = subprocess.run([binary_path(), *args], capture_output=True, text=True, encoding="utf-8", env=environment())
    # 0: clean, 1: diagnostics over the threshold; 2: the run could not start.
    if r.returncode not in (0, 1):
        raise RuntimeError(f"codelaw {args[0]} failed (exit {r.returncode}): {r.stderr.strip()}")
    return json.loads(r.stdout)


def _common(lang: Sequence[str] | None, rule: Sequence[str] | None, threads: int | None, no_cache: bool) -> list[str]:
    a: list[str] = []
    if lang:
        a += ["--lang", ",".join(lang)]
    if rule:
        a += ["--rule", ",".join(rule)]
    if threads is not None:
        a += ["--threads", str(threads)]
    if no_cache:
        a.append("--no-cache")
    return a


def check(
    paths: Sequence[str] = (),
    *,
    lang: Sequence[str] | None = None,
    rule: Sequence[str] | None = None,
    threads: int | None = None,
    no_cache: bool = False,
) -> dict[str, Any]:
    """``codelaw check --format json``; returns the report."""
    return _run(["check", "--format", "json", *_common(lang, rule, threads, no_cache), "--", *paths])


def fix(
    paths: Sequence[str] = (),
    *,
    unsafe: bool = False,
    dry_run: bool = False,
    lang: Sequence[str] | None = None,
    rule: Sequence[str] | None = None,
    threads: int | None = None,
    no_cache: bool = False,
) -> dict[str, Any]:
    """``codelaw fix --format json``; returns the report."""
    a = ["fix", "--format", "json", *_common(lang, rule, threads, no_cache)]
    if unsafe:
        a.append("--unsafe")
    if dry_run:
        a.append("--dry-run")
    return _run([*a, "--", *paths])
