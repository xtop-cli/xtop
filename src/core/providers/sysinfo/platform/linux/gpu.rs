//! Linux GPU probe from `/sys/class/drm` (used when `nvidia-smi` is absent).

use std::fs;
use std::path::Path;

use xtop_plugin_api::model::GpuInfo;

/// Extra GPU detection from `/sys/class/drm`.
pub fn read_gpu_info_from_sysfs() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/drm/") {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.starts_with("card") && !fname.contains('-') {
                let base = entry.path();
                let dev = base.join("device");
                let gpu_name = fs::read_to_string(dev.join("product_name"))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| fname.clone());
                let mem_total = fs::read_to_string(dev.join("mem_info_vram_total"))
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok())
                    .unwrap_or(0);
                let mem_used = fs::read_to_string(dev.join("mem_info_vram_used"))
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok())
                    .unwrap_or(0);
                let temp = find_hwmon_temp(&base.join("device"), "gpu").unwrap_or(0.0);
                gpus.push(GpuInfo {
                    name: gpu_name,
                    usage: 0.0,
                    temperature: temp,
                    memory_total: mem_total,
                    memory_used: mem_used,
                });
            }
        }
    }
    gpus
}

/// First hwmon temperature (in celsius) whose label matches `label_filter`.
fn find_hwmon_temp(device_path: &Path, label_filter: &str) -> Option<f32> {
    let hwmon = device_path.join("hwmon");
    if hwmon.exists() {
        if let Ok(entries) = fs::read_dir(&hwmon) {
            for entry in entries.flatten() {
                let hwmon_dir = entry.path();
                if let Ok(labels) = fs::read_to_string(hwmon_dir.join("temp1_label")) {
                    if labels.trim().to_lowercase().contains(label_filter) {
                        if let Ok(input) = fs::read_to_string(hwmon_dir.join("temp1_input")) {
                            if let Ok(millideg) = input.trim().parse::<f32>() {
                                return Some(millideg / 1000.0);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
