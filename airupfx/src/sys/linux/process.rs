pub use super::super::unix::process::*;

/// Makes current process become a subreaper.
pub fn become_subreaper() -> std::io::Result<()> {
    let result = unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER) };
    if result.is_negative() {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
