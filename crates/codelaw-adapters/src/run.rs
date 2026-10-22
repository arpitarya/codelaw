//! Finding and running an installed adapter (SR-ARCHITECTURE decisions 19
//! and 28): the program is looked up, never downloaded; it runs with the
//! project root as its working directory, its output drained as it runs,
//! and is stopped at the timeout with everything it started.

use crate::manifest::Manifest;
use semver::Version;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// The names a program may have on disk.
fn candidates(binary: &str) -> Vec<String> {
    if cfg!(windows) { vec![format!("{binary}.exe"), format!("{binary}.cmd"), binary.to_owned()] } else { vec![binary.to_owned()] }
}

/// The program for `manifest`: the project's `node_modules/.bin`, then its
/// `.venv/bin` (`Scripts` on Windows), then `PATH`. `None` when it is not
/// installed.
pub fn locate(manifest: &Manifest, root: &Path, path_env: Option<&std::ffi::OsStr>) -> Option<PathBuf> {
    let venv = if cfg!(windows) { "Scripts" } else { "bin" };
    let mut dirs = vec![root.join("node_modules").join(".bin"), root.join(".venv").join(venv)];
    if let Some(p) = path_env {
        dirs.extend(std::env::split_paths(p));
    }
    for dir in dirs {
        for name in candidates(&manifest.binary) {
            let file = dir.join(&name);
            if is_program(&file) {
                return Some(file);
            }
        }
    }
    None
}

#[cfg(unix)]
fn is_program(file: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(file).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_program(file: &Path) -> bool {
    file.is_file()
}

/// What a finished program printed.
pub struct Ran {
    pub stdout: String,
    pub stderr: String,
    pub status: std::process::ExitStatus,
}

/// How long a program's output may stay open after it exits: a process it
/// started and left running can hold the pipe.
const CLOSE_GRACE: Duration = Duration::from_millis(500);

fn drain(mut pipe: impl Read + Send + 'static) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    rx
}

/// Stop the program and everything it started: on Unix it leads its own
/// process group, and the group is killed.
fn kill_all(child: &mut Child) {
    #[cfg(unix)]
    if let Ok(pid) = libc::pid_t::try_from(child.id()) {
        // SAFETY: `kill` takes plain integers and has no memory effects; a
        // group that is already gone is ESRCH, which is ignored.
        unsafe {
            libc::kill(-pid, libc::SIGKILL);
        }
    }
    let _ = child.kill();
}

/// Run `program args…` in `cwd`. A non-zero exit is not an error here: the
/// linters exit 1 when they find something; the caller reads the output.
pub fn run(program: &Path, args: &[String], cwd: &Path, timeout: Duration) -> Result<Ran, String> {
    let shown = program.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    // A `.cmd` shim runs through `cmd /C` (Windows).
    let mut cmd = if program.extension().is_some_and(|e| e.eq_ignore_ascii_case("cmd")) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(program);
        c
    } else {
        Command::new(program)
    };
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start `{shown}`: {e}"))?;
    let stdout = drain(child.stdout.take().expect("piped"));
    let stderr = drain(child.stderr.take().expect("piped"));
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= timeout => {
                kill_all(&mut child);
                let _ = child.wait();
                return Err(format!("`{shown}` timed out after {timeout:?}"));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(e) => {
                kill_all(&mut child);
                let _ = child.wait();
                return Err(format!("`{shown}` failed: {e}"));
            }
        }
    };
    kill_all(&mut child);
    let text = |rx: mpsc::Receiver<Vec<u8>>| rx.recv_timeout(CLOSE_GRACE).ok().map(|b| String::from_utf8_lossy(&b).into_owned());
    let Some(stdout) = text(stdout) else {
        return Err(format!("`{shown}` exited but a process it started kept its output open"));
    };
    let stderr = text(stderr).unwrap_or_default();
    Ok(Ran { stdout, stderr, status })
}

/// The last non-empty line a program wrote to stderr, for a message.
pub fn stderr_tail(stderr: &str) -> String {
    stderr.lines().rev().find(|l| !l.trim().is_empty()).map(|l| format!(": {}", l.trim())).unwrap_or_default()
}

/// The first `x.y.z` in what `--version` printed: `ruff 0.16.9`,
/// `Version: 1.89.0`, `html-validate-11.17.0`, `17.16.0`.
pub fn parse_version(text: &str) -> Option<Version> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'.') {
            let end = text[i..].find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'))).map_or(text.len(), |n| i + n);
            let token = text[i..end].trim_end_matches(['.', '-', '+']);
            if let Ok(v) = Version::parse(token) {
                return Some(v);
            }
        }
        i += 1;
    }
    None
}

/// The installed program's version, as `--version` reports it.
pub fn version(manifest: &Manifest, program: &Path, cwd: &Path) -> Result<Version, String> {
    let ran = run(program, &manifest.version_args, cwd, Duration::from_secs(10))?;
    parse_version(&ran.stdout)
        .or_else(|| parse_version(&ran.stderr))
        .ok_or_else(|| format!("`{} {}` printed no version{}", manifest.binary, manifest.version_args.join(" "), stderr_tail(&ran.stderr)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_are_read_from_each_tool_banner() {
        for (text, want) in [
            ("ruff 0.16.9\n", "0.16.9"),
            ("Version: 1.89.0\n", "1.89.0"),
            ("html-validate-11.17.0\n", "11.17.0"),
            ("17.16.0\n", "17.16.0"),
            ("tool 2.0.0-rc.1 (abc)\n", "2.0.0-rc.1"),
        ] {
            assert_eq!(parse_version(text).map(|v| v.to_string()).as_deref(), Some(want), "{text}");
        }
        assert_eq!(parse_version("no version here"), None);
    }

    #[test]
    fn a_missing_program_is_not_located() {
        let m = crate::manifest("ruff").unwrap().clone();
        let mut m = m;
        m.binary = "codelaw-no-such-adapter-binary".into();
        assert_eq!(locate(&m, Path::new("."), std::env::var_os("PATH").as_deref()), None);
    }
}
