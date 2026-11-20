//! Cross-platform implementation of [`SystemDataProvider`] for the kernel.
//!
//! Real-time data comes from the `sysinfo` crate; only the OS-specific gaps
//! (governors, batteries, interface IPs, thread counts, extra GPUs) are
//! delegated to [`super::platform`].

use super::platform::shared::read_gpu_info_nvidia_smi;
use super::platform::{
    read_batteries, read_core_temps, read_cpu_governor, read_gpu_info_from_sysfs,
    read_interface_ips, read_mount_options, read_process_user_id, read_thread_count, RaplPower,
};
use std::collections::HashMap;
use std::time::Instant;
use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, Pid, ProcessRefreshKind,
    RefreshKind, Signal, System,
};
use xtop_plugin_api::model::*;
use xtop_plugin_api::SystemDataProvider;

pub const DEFAULT_MAX_PROCESSES: usize = 200;

pub struct SysinfoProvider {
    sys: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    prev_disk_read: HashMap<String, u64>,
    prev_disk_write: HashMap<String, u64>,
    prev_net_rx: HashMap<String, u64>,
    prev_net_tx: HashMap<String, u64>,
    last_refresh: Instant,
    cached_sys_info: SystemInfo,
    max_processes: usize,
    /// Linux RAPL energy sampler (UX9.1); no-op on other platforms. Sampled
    /// in `refresh_all`, so `SystemInfo::package_power_w` is refreshed once
    /// per tick at the same cadence as the other metrics.
    rapl_power: RaplPower,
}

impl Default for SysinfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SysinfoProvider {
    pub fn new() -> Self {
        // The snapshot is capped to the top-CPU processes to bound per-tick
        // work; users can raise the cap via XTOP_MAX_PROCESSES.
        let max_processes = std::env::var("XTOP_MAX_PROCESSES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_PROCESSES)
            .max(1);
        let sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        let info = SystemInfo {
            hostname: System::host_name().unwrap_or_default(),
            os_version: System::long_os_version().unwrap_or_default(),
            kernel: System::kernel_version().unwrap_or_default(),
            desktop_env: std::env::var("XDG_CURRENT_DESKTOP")
                .or_else(|_| std::env::var("DESKTOP_SESSION"))
                .unwrap_or_default(),
            shell: std::env::var("SHELL")
                .or_else(|_| std::env::var("ComSpec"))
                .unwrap_or_default(),
            // CPU model/brand from sysinfo's first logical core: a library
            // fact on every platform sysinfo covers; empty brand -> None.
            cpu_model: sys.cpus().first().and_then(|c| {
                let brand = c.brand().trim().to_string();
                if brand.is_empty() {
                    None
                } else {
                    Some(brand)
                }
            }),
            // Sampled per tick in `refresh_all` (RAPL deltas, Linux only).
            package_power_w: None,
        };
        Self {
            sys,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            prev_disk_read: HashMap::new(),
            prev_disk_write: HashMap::new(),
            prev_net_rx: HashMap::new(),
            prev_net_tx: HashMap::new(),
            last_refresh: Instant::now(),
            cached_sys_info: info,
            max_processes,
            rapl_power: RaplPower::new(),
        }
    }
}

impl SystemDataProvider for SysinfoProvider {
    fn refresh_all(&mut self) {
        self.sys.refresh_all();
        self.disks.refresh(true);
        self.networks.refresh(true);
        self.components.refresh(true);

        // Record rate baselines *after* this refresh: next sample computes
        // bytes/s as the delta since this point over `last_refresh`.
        for (name, n) in self.networks.iter() {
            self.prev_net_rx.insert(name.clone(), n.received());
            self.prev_net_tx.insert(name.clone(), n.transmitted());
        }
        for d in self.disks.iter() {
            let usage = d.usage();
            let key = d.mount_point().to_string_lossy().to_string();
            self.prev_disk_read.insert(key.clone(), usage.read_bytes);
            self.prev_disk_write.insert(key, usage.written_bytes);
        }

        // Package power (UX9.1): sample the RAPL energy deltas once per
        // refresh so `snapshot()` and `system_info()` both see the fresh
        // value. `None` (no readable source, first baseline sample) is
        // stored as-is — the probe never fabricates a reading, and a value
        // that becomes unreadable drops back to `None`.
        self.cached_sys_info.package_power_w = self.rapl_power.sample();

        self.last_refresh = Instant::now();
    }

    fn snapshot(&self) -> SystemSnapshot {
        // Per-core temperatures (Linux coretemp probe; `None` elsewhere or
        // when the sensors do not map onto the logical cores — UX8.3).
        let core_temps = read_core_temps(self.sys.cpus().len());
        let cpus: Vec<CpuInfo> = self
            .sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, c)| CpuInfo {
                name: c.name().to_string(),
                usage: c.cpu_usage() as f64,
                cpu_id: i,
                frequency: c.frequency(),
                governor: read_cpu_governor(i),
                temp_c: core_temps.get(i).copied().flatten(),
            })
            .collect();

        let total_mem = self.sys.total_memory();
        let used_mem = self.sys.used_memory();
        let memory = MemoryInfo {
            total: total_mem,
            used: used_mem,
            available: self.sys.available_memory(),
            free: self.sys.free_memory(),
            percent: if total_mem > 0 {
                (used_mem as f64 / total_mem as f64) * 100.0
            } else {
                0.0
            },
        };

        let total_swap = self.sys.total_swap();
        let used_swap = self.sys.used_swap();
        let swap = SwapInfo {
            total: total_swap,
            used: used_swap,
            free: self.sys.free_swap(),
            percent: if total_swap > 0 {
                (used_swap as f64 / total_swap as f64) * 100.0
            } else {
                0.0
            },
        };

        let mount_info = read_mount_options();

        let disks: Vec<DiskInfo> = self
            .disks
            .iter()
            .map(|d| {
                let mp = d.mount_point().to_string_lossy().to_string();
                let total = d.total_space();
                let available = d.available_space();
                let used = total - available;
                let opts = mount_info.get(&mp).cloned().unwrap_or_default();
                DiskInfo {
                    mount_point: mp,
                    total_space: total,
                    available_space: available,
                    used_space: used,
                    percent: if total > 0 {
                        (used as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    },
                    file_system: d.file_system().to_string_lossy().to_string(),
                    mount_options: opts,
                }
            })
            .collect();

        let iface_ips = read_interface_ips();

        let networks: Vec<NetworkInfo> = self
            .networks
            .iter()
            .map(|(name, n)| {
                let rx = n.received();
                let tx = n.transmitted();
                let rx_speed = if let Some(prev) = self.prev_net_rx.get(name) {
                    let elapsed = self.last_refresh.elapsed().as_secs_f64();
                    if elapsed > 0.0 {
                        (rx.saturating_sub(*prev)) as f64 / elapsed
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let tx_speed = if let Some(prev) = self.prev_net_tx.get(name) {
                    let elapsed = self.last_refresh.elapsed().as_secs_f64();
                    if elapsed > 0.0 {
                        (tx.saturating_sub(*prev)) as f64 / elapsed
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                NetworkInfo {
                    name: name.clone(),
                    received: rx,
                    transmitted: tx,
                    rx_speed,
                    tx_speed,
                    ip: iface_ips.get(name).cloned().unwrap_or_default(),
                }
            })
            .collect();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let uptime = System::uptime();
        // sysinfo reports process start relative to boot; expose it as epoch
        // seconds so consumers (widgets, plugins) compare against one clock.
        let boot_epoch = now.saturating_sub(uptime);

        let mut procs: Vec<ProcessInfo> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, p)| {
                let start_raw = p.start_time();
                let start = if start_raw > 0 {
                    boot_epoch.saturating_add(start_raw)
                } else {
                    0
                };
                let run = now.saturating_sub(start);
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: p.name().to_string_lossy().to_string(),
                    cpu_usage: p.cpu_usage() as f64,
                    memory: p.memory(),
                    // Platform probe: the numeric uid string on unix; the
                    // SID's numeric RID on Windows (see windows.rs).
                    user_id: read_process_user_id(p),
                    state: format!("{:?}", p.status()),
                    cmd: p
                        .cmd()
                        .first()
                        .map(|c| c.to_string_lossy().to_string())
                        .unwrap_or_default(),

                    // P0
                    exe_path: p.exe().map(|e| e.to_string_lossy().to_string()),
                    parent_pid: p.parent().map(|ppid| ppid.as_u32()),
                    cmd_full: p
                        .cmd()
                        .iter()
                        .map(|c| c.to_string_lossy().to_string())
                        .collect(),

                    // P1
                    start_time: start,
                    run_time: run,
                    effective_user_id: p.effective_user_id().map(|u| u.to_string()),
                    group_id: p.group_id().map(|g| g.to_string()),
                    cwd: p.cwd().map(|c| c.to_string_lossy().to_string()),
                    thread_count: read_thread_count(p.pid()),

                    // P2
                    open_files: p.open_files().unwrap_or(0) as u64,
                    open_files_limit: p.open_files_limit().unwrap_or(0) as u64,
                    disk_total_read_bytes: p.disk_usage().total_read_bytes,
                    disk_total_write_bytes: p.disk_usage().total_written_bytes,
                    environ: p
                        .environ()
                        .iter()
                        .map(|e| e.to_string_lossy().to_string())
                        .collect(),
                    session_id: p.session_id().map(|s| s.as_u32()),
                }
            })
            .collect();
        procs.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        procs.truncate(self.max_processes);

        let mut max_temp = 0.0f32;
        for component in &self.components {
            if let Some(temp) = component.temperature() {
                if temp > max_temp {
                    max_temp = temp;
                }
            }
        }

        let load = System::load_average();

        SystemSnapshot {
            cpu_temp: max_temp as f64,
            cpus,
            memory,
            swap,
            disks,
            networks,
            processes: procs,
            load_avg: LoadAvg {
                one: load.one,
                five: load.five,
                fifteen: load.fifteen,
            },
            uptime,
            disk_io: self.disk_io_inner(),
            batteries: read_batteries(),
            gpus: read_gpu_info(),
            sys_info: self.cached_sys_info.clone(),
        }
    }

    fn disk_io(&self) -> Vec<DiskIOInfo> {
        self.disk_io_inner()
    }

    fn system_info(&self) -> SystemInfo {
        self.cached_sys_info.clone()
    }

    fn kill_process(&self, pid: u32) -> bool {
        if let Some(process) = self.sys.process(Pid::from(pid as usize)) {
            // Only allow killing processes owned by the same user (safety check)
            let current_uid = self
                .sys
                .process(sysinfo::get_current_pid().unwrap_or(Pid::from(0)))
                .and_then(|p| p.user_id());
            let target_uid = process.user_id();
            match (current_uid, target_uid) {
                (Some(current), Some(target)) if current == target => {
                    process.kill_with(Signal::Term).unwrap_or(false)
                }
                (Some(_), Some(_)) => false,
                _ => process.kill_with(Signal::Term).unwrap_or(false),
            }
        } else {
            false
        }
    }

    fn batteries(&self) -> Vec<BatteryInfo> {
        read_batteries()
    }

    fn gpu_info(&self) -> Vec<GpuInfo> {
        read_gpu_info()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl SysinfoProvider {
    fn disk_io_inner(&self) -> Vec<DiskIOInfo> {
        self.disks
            .iter()
            .map(|d| {
                let name = d.mount_point().to_string_lossy().to_string();
                let usage = d.usage();
                let read_bytes = usage.read_bytes;
                let write_bytes = usage.written_bytes;
                let (read_speed, write_speed) = if let (Some(prev_r), Some(prev_w)) = (
                    self.prev_disk_read.get(&name),
                    self.prev_disk_write.get(&name),
                ) {
                    let elapsed = self.last_refresh.elapsed().as_secs_f64();
                    if elapsed > 0.0 {
                        (
                            (read_bytes.saturating_sub(*prev_r)) as f64 / elapsed,
                            (write_bytes.saturating_sub(*prev_w)) as f64 / elapsed,
                        )
                    } else {
                        (0.0, 0.0)
                    }
                } else {
                    (0.0, 0.0)
                };
                DiskIOInfo {
                    name,
                    read_bytes,
                    write_bytes,
                    read_speed,
                    write_speed,
                }
            })
            .collect()
    }
}

fn read_gpu_info() -> Vec<GpuInfo> {
    // nvidia-smi is shared by Linux and Windows; the remaining platforms get
    // their own fallback probe under platform/.
    let mut gpus = read_gpu_info_nvidia_smi();

    // Fallback: platform-specific detection (e.g. /sys/class/drm on Linux).
    if gpus.is_empty() {
        gpus.extend(read_gpu_info_from_sysfs());
    }

    gpus
}
