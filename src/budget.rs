//! Linux RSS sampling for a soft process-tree budget. Checked between
//! instructions and while waiting for solver replies; never prunes a path.
use std::{
    cell::Cell,
    fs,
    path::Path,
    time::{Duration, Instant},
};

pub fn default_mib() -> u64 {
    let available = fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| kib(&s, "MemAvailable:"));
    default_from_kib(available)
}
fn default_from_kib(available: Option<u64>) -> u64 {
    available.map_or(1024, |n| (n / 2048).clamp(1, 8192))
}
fn kib(text: &str, field: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        line.strip_prefix(field)?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}
fn rss(pid: u32) -> Result<u64, String> {
    let text = fs::read_to_string(format!("/proc/{pid}/status"))
        .map_err(|e| format!("cannot measure RSS for process {pid}: {e}"))?;
    if text
        .lines()
        .any(|s| s.starts_with("State:") && (s.contains("Z (zombie)") || s.contains("X (dead)")))
    {
        return Ok(0);
    }
    // Linux omits VmRSS once a child drops its mm during exec/exit,
    // even before its State field becomes zombie. Such a child has no RSS.
    kib(&text, "VmRSS:")
        .or_else(|| (pid != std::process::id()).then_some(0))
        .ok_or_else(|| format!("RSS unavailable for process {pid}"))
}
fn tree(pid: u32, total: &mut u64) -> Result<(), String> {
    *total = total.saturating_add(rss(pid)?);
    let tasks = fs::read_dir(format!("/proc/{pid}/task"))
        .map_err(|e| format!("cannot enumerate child RSS: {e}"))?;
    let mut children = std::collections::BTreeSet::new();
    for task in tasks {
        let path = task.map_err(|e| e.to_string())?.path().join("children");
        match fs::read_to_string(&path) {
            Ok(text) => children.extend(
                text.split_whitespace()
                    .filter_map(|p| p.parse::<u32>().ok()),
            ),
            Err(_) if !path.exists() => {} // thread exited during sampling
            Err(error) => return Err(format!("cannot measure child RSS: {error}")),
        }
    }
    for child in children {
        // A child can exit between the children listing and status read.
        if let Err(error) = tree(child, total)
            && Path::new(&format!("/proc/{child}")).exists()
        {
            return Err(error);
        }
    }
    Ok(())
}
#[derive(Clone)]
pub struct Budget {
    pub limit_mib: u64,
    sampled: Cell<Instant>,
}
impl Budget {
    pub fn new(limit_mib: u64) -> Self {
        Self {
            limit_mib,
            sampled: Cell::new(Instant::now() - Duration::from_secs(1)),
        }
    }
    pub fn check(&self) -> Result<(), String> {
        if self.sampled.get().elapsed() < Duration::from_millis(5) {
            return Ok(());
        }
        self.sampled.set(Instant::now());
        let mut used = 0;
        tree(std::process::id(), &mut used)?;
        if used > self.limit_mib.saturating_mul(1024) {
            return Err(format!(
                "memory budget exceeded ({} MiB); measured process-tree RSS {} MiB",
                self.limit_mib,
                used.div_ceil(1024)
            ));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn available_memory_is_bounded_and_missing_memory_is_conservative() {
        assert_eq!(default_from_kib(Some(32 * 1024 * 1024)), 8192);
        assert_eq!(default_from_kib(Some(4 * 1024 * 1024)), 2048);
        assert_eq!(default_from_kib(None), 1024);
        assert_eq!(default_from_kib(Some(0)), 1);
        assert!(
            Budget::new(0)
                .check()
                .unwrap_err()
                .contains("memory budget exceeded")
        );
    }
}
