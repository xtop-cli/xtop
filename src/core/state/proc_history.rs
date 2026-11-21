//! Bounded per-process CPU history for small braille sparks (UX9.1).
//!
//! Fed once per tick from the visible process list (the same sorted +
//! capped list that rides in the snapshot): every process in that list gets
//! one sample per tick. Storage stays flat:
//!
//! - at most `max_pids` pids are tracked — the kernel's process cap
//!   (`XTOP_MAX_PROCESSES`, default 200 — the same env var and default the
//!   sysinfo provider reads) plus a margin for churn; when the cap is hit
//!   the *oldest* tracked pid is evicted first (FIFO of first-seen pids);
//! - each pid keeps at most `max_samples` (30) samples; older ones drop off
//!   the front, so the series is oldest → newest;
//! - pids that leave the visible list keep their history until evicted by
//!   the pid cap, so a process that flickers out of the top list keeps its
//!   spark for a while.
//!
//! The `WidgetState::process_cpu_history` implementation hands out owned
//! copies; a pid that was never seen (or was evicted) yields an empty
//! series, and renderers draw nothing for it.

use std::collections::HashMap;
use std::collections::VecDeque;

/// Default `XTOP_MAX_PROCESSES` cap, mirroring the sysinfo provider.
const DEFAULT_MAX_PROCESSES: usize = 200;
/// Churn margin added to the process cap for the tracked-pid bound.
const PID_MARGIN: usize = 40;
/// Samples kept per pid (oldest dropped first).
const MAX_SAMPLES: usize = 30;

pub struct ProcessCpuHistory {
    samples: HashMap<u32, VecDeque<f64>>,
    /// First-seen pid order, for FIFO eviction of the oldest tracked pid.
    order: VecDeque<u32>,
    max_pids: usize,
    max_samples: usize,
}

impl Default for ProcessCpuHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessCpuHistory {
    /// Tracked-pid cap = `XTOP_MAX_PROCESSES` (default 200) + margin.
    pub fn new() -> Self {
        let max_pids = std::env::var("XTOP_MAX_PROCESSES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_PROCESSES)
            .max(1)
            .saturating_add(PID_MARGIN);
        Self::new_with_limits(max_pids, MAX_SAMPLES)
    }

    /// Bounds are injectable (tests pin the eviction behavior directly).
    fn new_with_limits(max_pids: usize, max_samples: usize) -> Self {
        Self {
            samples: HashMap::new(),
            order: VecDeque::new(),
            max_pids,
            max_samples,
        }
    }

    /// Record one CPU-usage sample for a pid (called once per tick for every
    /// process in the visible list).
    pub fn push(&mut self, pid: u32, cpu_usage: f64) {
        if let Some(ring) = self.samples.get_mut(&pid) {
            if ring.len() >= self.max_samples {
                ring.pop_front();
            }
            ring.push_back(cpu_usage);
            return;
        }
        if self.order.len() >= self.max_pids {
            if let Some(evicted) = self.order.pop_front() {
                self.samples.remove(&evicted);
            }
        }
        self.order.push_back(pid);
        let mut ring = VecDeque::with_capacity(self.max_samples.min(8));
        ring.push_back(cpu_usage);
        self.samples.insert(pid, ring);
    }

    /// Owned copy of the samples for a pid, oldest → newest. Empty for
    /// unknown/evicted pids.
    pub fn history(&self, pid: u32) -> Vec<f64> {
        self.samples
            .get(&pid)
            .map(|ring| ring.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Number of tracked pids.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.order.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_returns_oldest_to_newest() {
        let mut h = ProcessCpuHistory::new();
        h.push(10, 1.0);
        h.push(10, 2.0);
        h.push(10, 3.0);
        assert_eq!(h.history(10), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn per_pid_samples_are_bounded_to_the_window() {
        let mut h = ProcessCpuHistory::new_with_limits(100, 4);
        for i in 0..10 {
            h.push(7, i as f64);
        }
        assert_eq!(h.history(7), vec![6.0, 7.0, 8.0, 9.0]);
    }

    #[test]
    fn pid_cap_evicts_the_oldest_tracked_pid() {
        let mut h = ProcessCpuHistory::new_with_limits(3, 30);
        h.push(1, 1.0);
        h.push(2, 2.0);
        h.push(3, 3.0);
        assert_eq!(h.len(), 3);
        // Fourth distinct pid evicts pid 1 (FIFO of first-seen pids).
        h.push(4, 4.0);
        assert_eq!(h.len(), 3);
        assert!(h.history(1).is_empty(), "oldest pid evicted first");
        assert_eq!(h.history(2), vec![2.0]);
        assert_eq!(h.history(4), vec![4.0]);
    }

    #[test]
    fn existing_pids_do_not_count_against_the_cap() {
        let mut h = ProcessCpuHistory::new_with_limits(2, 30);
        h.push(1, 1.0);
        h.push(2, 2.0);
        // Re-pushing existing pids never evicts.
        h.push(1, 1.5);
        h.push(2, 2.5);
        assert_eq!(h.len(), 2);
        assert_eq!(h.history(1), vec![1.0, 1.5]);
        assert_eq!(h.history(2), vec![2.0, 2.5]);
    }

    #[test]
    fn evicted_pid_reseeds_a_fresh_series() {
        let mut h = ProcessCpuHistory::new_with_limits(1, 30);
        h.push(1, 1.0);
        h.push(2, 2.0); // evicts pid 1
        assert!(h.history(1).is_empty());
        h.push(1, 99.0);
        assert_eq!(h.history(1), vec![99.0]);
    }

    #[test]
    fn unknown_pid_returns_empty() {
        let h = ProcessCpuHistory::new();
        assert!(h.history(424_242).is_empty());
    }
}
