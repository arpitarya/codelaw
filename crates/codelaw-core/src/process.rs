//! Stopping a child process and everything it started (SR-ARCHITECTURE
//! decisions 18 and 19). The one copy: the adapter runner and the external
//! fixer both spawn their child as the leader of its own process group and
//! call [`kill_group`] at a timeout or after it exits.

use std::process::Child;

/// Stop `child` and everything it started. On Unix the child leads its own
/// process group (spawned with `CommandExt::process_group(0)`), so the whole
/// group is killed, not just the immediate child; elsewhere only the child is.
pub fn kill_group(child: &mut Child) {
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
