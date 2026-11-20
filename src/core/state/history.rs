use std::collections::VecDeque;

pub struct MetricsHistory {
    pub cpu: Vec<VecDeque<(f64, f64)>>,
    pub mem: VecDeque<(f64, f64)>,
    pub net_rx: VecDeque<(f64, f64)>,
    pub net_tx: VecDeque<(f64, f64)>,
    /// Aggregate disk read rate (bytes/s across all disks).
    pub disk_read: VecDeque<(f64, f64)>,
    /// Aggregate disk write rate (bytes/s across all disks).
    pub disk_write: VecDeque<(f64, f64)>,
    /// 1-minute load average (chart series for `load_avg.one`).
    pub load: VecDeque<(f64, f64)>,
    max_points: usize,
}

impl MetricsHistory {
    pub fn new(max_points: usize) -> Self {
        Self {
            cpu: Vec::new(),
            mem: VecDeque::with_capacity(max_points),
            net_rx: VecDeque::with_capacity(max_points),
            net_tx: VecDeque::with_capacity(max_points),
            disk_read: VecDeque::with_capacity(max_points),
            disk_write: VecDeque::with_capacity(max_points),
            load: VecDeque::with_capacity(max_points),
            max_points,
        }
    }

    #[cfg(test)]
    pub fn set_max_points(&mut self, max: usize) {
        self.max_points = max;
        self.mem.truncate(max);
        self.mem.shrink_to_fit();
        self.net_rx.truncate(max);
        self.net_rx.shrink_to_fit();
        self.net_tx.truncate(max);
        self.net_tx.shrink_to_fit();
        self.disk_read.truncate(max);
        self.disk_read.shrink_to_fit();
        self.disk_write.truncate(max);
        self.disk_write.shrink_to_fit();
        self.load.truncate(max);
        self.load.shrink_to_fit();
        for h in &mut self.cpu {
            h.truncate(max);
            h.shrink_to_fit();
        }
    }

    pub fn push_cpu(&mut self, cpu_id: usize, x: f64, y: f64) {
        if cpu_id >= self.cpu.len() {
            self.cpu
                .resize(cpu_id + 1, VecDeque::with_capacity(self.max_points));
        }
        let h = &mut self.cpu[cpu_id];
        if h.len() >= self.max_points {
            h.pop_front();
        }
        h.push_back((x, y));
    }

    pub fn push_mem(&mut self, x: f64, y: f64) {
        if self.mem.len() >= self.max_points {
            self.mem.pop_front();
        }
        self.mem.push_back((x, y));
    }

    pub fn push_net(&mut self, x: f64, rx: f64, tx: f64) {
        if self.net_rx.len() >= self.max_points {
            self.net_rx.pop_front();
            self.net_tx.pop_front();
        }
        self.net_rx.push_back((x, rx));
        self.net_tx.push_back((x, tx));
    }

    /// Push aggregate disk throughput (bytes/s, summed across disks). Both
    /// series share the same bound as the other histories.
    pub fn push_disk(&mut self, x: f64, read_rate: f64, write_rate: f64) {
        if self.disk_read.len() >= self.max_points {
            self.disk_read.pop_front();
            self.disk_write.pop_front();
        }
        self.disk_read.push_back((x, read_rate));
        self.disk_write.push_back((x, write_rate));
    }

    /// Push the 1-minute load average (`load_avg.one`).
    pub fn push_load(&mut self, x: f64, one: f64) {
        if self.load.len() >= self.max_points {
            self.load.pop_front();
        }
        self.load.push_back((x, one));
    }

    pub fn reset_cpu(&mut self, count: usize) {
        self.cpu = vec![VecDeque::with_capacity(self.max_points); count];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_new() {
        let h = MetricsHistory::new(100);
        assert!(h.cpu.is_empty());
        assert!(h.mem.is_empty());
        assert!(h.net_rx.is_empty());
        assert!(h.net_tx.is_empty());
    }

    #[test]
    fn test_history_push_cpu() {
        let mut h = MetricsHistory::new(3);
        h.push_cpu(0, 1.0, 50.0);
        h.push_cpu(0, 2.0, 60.0);
        assert_eq!(h.cpu[0].len(), 2);
        h.push_cpu(0, 3.0, 70.0);
        h.push_cpu(0, 4.0, 80.0);
        assert_eq!(h.cpu[0].len(), 3);
        assert_eq!(h.cpu[0][0], (2.0, 60.0));
    }

    #[test]
    fn test_history_push_mem() {
        let mut h = MetricsHistory::new(2);
        h.push_mem(1.0, 30.0);
        h.push_mem(2.0, 40.0);
        assert_eq!(h.mem.len(), 2);
        h.push_mem(3.0, 50.0);
        assert_eq!(h.mem.len(), 2);
        assert_eq!(h.mem[0], (2.0, 40.0));
    }

    #[test]
    fn test_history_push_net() {
        let mut h = MetricsHistory::new(5);
        h.push_net(1.0, 100.0, 200.0);
        assert_eq!(h.net_rx.len(), 1);
        assert_eq!(h.net_tx[0], (1.0, 200.0));
    }

    #[test]
    fn test_history_push_disk() {
        let mut h = MetricsHistory::new(2);
        h.push_disk(1.0, 100.0, 50.0);
        h.push_disk(2.0, 120.0, 60.0);
        assert_eq!(h.disk_read.len(), 2);
        assert_eq!(h.disk_write[1], (2.0, 60.0));
        // Bounded like the other histories.
        h.push_disk(3.0, 130.0, 70.0);
        assert_eq!(h.disk_read.len(), 2);
        assert_eq!(h.disk_read[0], (2.0, 120.0));
        assert_eq!(h.disk_write[0], (2.0, 60.0));
    }

    #[test]
    fn test_history_push_load() {
        let mut h = MetricsHistory::new(3);
        h.push_load(1.0, 0.5);
        h.push_load(2.0, 0.8);
        assert_eq!(h.load.len(), 2);
        h.push_load(3.0, 1.1);
        h.push_load(4.0, 1.4);
        assert_eq!(h.load.len(), 3);
        assert_eq!(h.load[0], (2.0, 0.8));
    }

    #[test]
    fn test_history_new_has_ux8_series_empty() {
        let h = MetricsHistory::new(100);
        assert!(h.disk_read.is_empty());
        assert!(h.disk_write.is_empty());
        assert!(h.load.is_empty());
    }

    #[test]
    fn test_history_reset_cpu() {
        let mut h = MetricsHistory::new(10);
        h.push_cpu(0, 1.0, 50.0);
        h.push_cpu(1, 2.0, 60.0);
        assert_eq!(h.cpu.len(), 2);
        h.reset_cpu(4);
        assert_eq!(h.cpu.len(), 4);
        assert!(h.cpu[0].is_empty());
    }

    #[test]
    fn test_history_auto_resize_cpu() {
        let mut h = MetricsHistory::new(10);
        h.push_cpu(5, 1.0, 99.0);
        assert_eq!(h.cpu.len(), 6);
        assert_eq!(h.cpu[5][0], (1.0, 99.0));
    }

    #[test]
    fn test_history_set_max_points() {
        let mut h = MetricsHistory::new(10);
        for i in 0..20 {
            h.push_mem(i as f64, i as f64);
            h.push_disk(i as f64, i as f64, i as f64);
            h.push_load(i as f64, i as f64);
        }
        assert_eq!(h.mem.len(), 10);
        assert_eq!(h.disk_read.len(), 10);
        assert_eq!(h.disk_write.len(), 10);
        assert_eq!(h.load.len(), 10);
        h.set_max_points(5);
        assert_eq!(h.mem.len(), 5);
        assert_eq!(h.disk_read.len(), 5);
        assert_eq!(h.disk_write.len(), 5);
        assert_eq!(h.load.len(), 5);
    }
}
