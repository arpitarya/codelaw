#!/usr/bin/env python3
"""Auto-landing — SR-ARCHITECTURE decisions 19 and 20: finished work reaches main
without anyone remembering to land it, and what Cowork could not commit,
Claude Code does.

    start  SessionStart      export CLAUDE_SESSION_ID for this session's commands,
                             so `lockmaster start` records whose change it is;
                             land waiting handoffs in the background
    edit   PostToolUse       note each file this session writes in main's checkout
    stop   Stop              land this session's open changes now (they stay open);
                             land the handoffs; tell the agent, once, about
                             a landing held back or edits left in main's checkout
    end    SessionEnd        land and clean up this session's changes, in the
                             background (the session is going away)

lockmaster is found at `$LOCKMASTER_SRC`, in this repository's own `src/`
(lockmaster itself), in a `lockmaster` checkout beside this repository, or as
an installed package. Without it the hook does nothing.
It never fails a session: every error is swallowed and the turn ends as usual.
Python >= 3.9, standard library only.
"""
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def git(args, cwd):
    try:
        r = subprocess.run(["git"] + args, cwd=str(cwd), capture_output=True, text=True)
    except OSError:
        return None
    return r.stdout.strip() if r.returncode == 0 else None


def main_root(cwd):
    """The primary checkout: the first `git worktree list` entry."""
    out = git(["worktree", "list", "--porcelain"], cwd)
    if not out:
        return None
    return Path(out.splitlines()[0][len("worktree "):])


def candidates():
    env = os.environ.get("LOCKMASTER_SRC")
    if env:
        yield Path(env)
    yield ROOT / "src"
    root = main_root(ROOT)
    if root is not None:
        yield root / "src"
        yield root.parent / "lockmaster" / "src"


def find_src():
    """The directory to put on sys.path, "" when lockmaster is installed, None when absent."""
    for c in candidates():
        if (c / "lockmaster" / "land.py").is_file():
            return str(c)
    try:
        import lockmaster  # noqa: F401
        return ""
    except ImportError:
        return None


def load():
    src = find_src()
    if src is None:
        return None, None
    if src:
        sys.path.insert(0, src)
    from lockmaster import land
    from lockmaster.repo import Repo
    return src, (land, Repo)


def command(src):
    return "PYTHONPATH={} python3 -m lockmaster".format(src) if src else "lockmaster"


def detach(src, args, cwd):
    """Run `lockmaster <args>` on its own, outliving this hook; output to the state dir's log."""
    common = git(["rev-parse", "--git-common-dir"], cwd)
    if not common:
        return
    log_dir = (Path(cwd) / common).resolve() / "lockmaster"
    log_dir.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ)
    if src:
        env["PYTHONPATH"] = src + (os.pathsep + env["PYTHONPATH"] if env.get("PYTHONPATH") else "")
    with open(str(log_dir / "auto-land.log"), "a", encoding="utf-8") as log:
        subprocess.Popen([sys.executable, "-m", "lockmaster"] + args, cwd=str(cwd), env=env,
                         stdin=subprocess.DEVNULL, stdout=log, stderr=log, start_new_session=True)


# ---------------------------------------------------------------- the session's own notes

def notes_file(repo, sid):
    safe = "".join(ch if ch.isalnum() or ch in "-_." else "_" for ch in sid)
    return repo.state / "sessions" / (safe + ".json")


def read_notes(repo, sid):
    try:
        return json.loads(notes_file(repo, sid).read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {"edits": []}


def write_notes(repo, sid, notes):
    f = notes_file(repo, sid)
    f.parent.mkdir(parents=True, exist_ok=True)
    tmp = f.with_name(f.name + ".part")
    tmp.write_text(json.dumps(notes, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(str(tmp), str(f))


# ---------------------------------------------------------------- modes

def start(sid, hook):
    env_file = os.environ.get("CLAUDE_ENV_FILE")
    if env_file and sid:
        with open(env_file, "a", encoding="utf-8") as fh:
            fh.write("export CLAUDE_SESSION_ID={}\n".format(sid))
    src = find_src()
    if src is not None:
        detach(src, ["pickup"], hook.get("cwd") or ROOT)
    return None


def edit(sid, hook):
    raw = (hook.get("tool_input") or {}).get("file_path") or (hook.get("tool_input") or {}).get("notebook_path")
    if not raw or not sid:
        return None
    src, mods = load()
    if mods is None:
        return None
    _, Repo = mods
    p = Path(raw)
    if not p.is_absolute():
        p = Path(hook.get("cwd") or os.getcwd()) / p
    repo = Repo(hook.get("cwd") or ROOT)
    root = os.path.realpath(str(repo.root))
    full = os.path.realpath(str(p))
    if not full.startswith(root + os.sep):
        return None
    rel = os.path.relpath(full, root).replace(os.sep, "/")
    if rel.startswith(".claude/worktrees/") or repo.change_holding(full) is not None:
        return None
    notes = read_notes(repo, sid)
    if rel not in notes["edits"]:
        notes["edits"].append(rel)
        write_notes(repo, sid, notes)
    return None


def stop(sid, hook):
    src, mods = load()
    if mods is None:
        return None
    land, Repo = mods
    cwd = hook.get("cwd") or str(ROOT)
    lines, said = [], []

    def out(text):
        lines.append(text)

    def step(fn):
        """Each step on its own: one failing never skips the others."""
        try:
            return fn()
        except Exception as e:  # noqa: BLE001
            print("auto-land.py stop: {}".format(e), file=sys.stderr)
            return None

    step(lambda: land.sync(Repo(cwd), session=sid, cwd=cwd, out=out))
    for item, outcome, note in step(lambda: land.pickup(Repo(cwd), out=out)) or []:
        if outcome == "red" and note:
            said.append("Edits handed over from Cowork ({}) did not land:\n{}\nFix them where they are, in "
                        "main's checkout; the end of the next turn tries again with them.".format(item, note))
    for item, reason in step(lambda: land.held_reports(Repo(cwd), session=sid, cwd=cwd)) or []:
        said.append("lockmaster could not land {} at the end of this turn:\n{}\nFix it in its worktree; it "
                    "lands at the end of the next turn. Nothing on main changed.".format(item, reason))
    if sid:
        reminder = step(lambda: checkout_reminder(land, Repo(cwd), sid, src))
        if reminder:
            said.append(reminder)
    landed = [l for l in lines if l.startswith("landed ") or l.startswith("handoff ") or l.startswith("finished ")]
    result = {}
    if landed:
        result["systemMessage"] = "lockmaster: " + "; ".join(landed)
    if said:
        if hook.get("stop_hook_active"):
            # Already continuing because of a Stop hook: say it, never block twice in a row.
            result["systemMessage"] = "\n\n".join(([result["systemMessage"]] if landed else []) + said)
        else:
            result["decision"] = "block"
            result["reason"] = "\n\n".join(said)
    return result or None


def checkout_reminder(land, repo, sid, src):
    """Once per set of files: the session's own edits in main's checkout that are not on main."""
    notes = read_notes(repo, sid)
    if not notes["edits"]:
        return None
    left = [p for p in land._checkout_dirty(repo, notes["edits"]) if p in notes["edits"]]
    if not left:
        return None
    key = hashlib.sha256("\0".join(sorted(left)).encode()).hexdigest()[:16]
    if notes.get("told") == key:
        return None
    notes["told"] = key
    write_notes(repo, sid, notes)
    return ("This session edited {} in main's checkout and it is not on main. Land it now: "
            "`{} save -m \"<what changed and why>\" {}`".format(", ".join(left), command(src), " ".join(left)))


def end(sid, hook):
    src = find_src()
    if src is None:
        return None
    cwd = hook.get("cwd") or str(ROOT)
    args = ["sync", "--final", "--cwd", cwd] + (["--session", sid] if sid else [])
    detach(src, args, cwd)
    return None


def main(argv):
    mode = argv[0] if argv else ""
    fn = {"start": start, "edit": edit, "stop": stop, "end": end}.get(mode)
    if fn is None:
        print("usage: auto-land.py start|edit|stop|end", file=sys.stderr)
        return 0
    hook = {}
    if not sys.stdin.isatty():
        try:
            hook = json.loads(sys.stdin.read() or "{}")
        except ValueError:
            hook = {}
    sid = hook.get("session_id") or os.environ.get("CLAUDE_SESSION_ID", "")
    try:
        result = fn(sid, hook)
    except Exception as e:  # noqa: BLE001 — never fail the session over a landing
        print("auto-land.py {}: {}".format(mode, e), file=sys.stderr)
        return 0
    if result:
        print(json.dumps(result))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
