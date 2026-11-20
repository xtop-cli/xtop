//! GPU probe via the `nvidia-smi` CLI, shared by Linux and Windows.

use xtop_plugin_api::model::GpuInfo;

/// Query NVIDIA GPUs through `nvidia-smi` when available.
pub fn read_gpu_info_nvidia_smi() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,memory.total,memory.used,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 5 {
                    let name = parts[0].to_string();
                    let usage = parts[1].parse::<f64>().unwrap_or(0.0);
                    let mem_total = parts[2].parse::<u64>().unwrap_or(0) * 1024 * 1024;
                    let mem_used = parts[3].parse::<u64>().unwrap_or(0) * 1024 * 1024;
                    let temp = parts[4].parse::<f32>().unwrap_or(0.0);
                    gpus.push(GpuInfo {
                        name,
                        usage,
                        temperature: temp,
                        memory_total: mem_total,
                        memory_used: mem_used,
                    });
                }
            }
        }
    }
    gpus
}
