pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod mem;
pub mod processes;
pub mod storage;
pub mod temp;

use crate::state::MetricsSnapshot;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

/// One full sample across all metrics. `sys` is refreshed in place (see
/// `sensors::cpu` for why it must persist across calls).
///
/// The process walk is by far the most expensive sample (~20x the rest
/// combined), so callers gate it with `include_processes` — the lists only
/// matter while the dashboard is visible. When skipped, the lists come back
/// empty and the caller carries the previous ones forward.
pub fn sample(sys: &mut System, include_processes: bool) -> MetricsSnapshot {
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    let cpu_temp = temp::cpu_temp_celsius();
    let (top_cpu, top_mem) = if include_processes {
        processes::sample(sys)
    } else {
        (Vec::new(), Vec::new())
    };

    MetricsSnapshot {
        cpu: cpu::sample(sys, cpu_temp),
        ram: mem::sample(sys),
        disk: disk::sample(),
        gpu: gpu::sample(),
        top_cpu,
        top_mem,
        // Merged from the background scan by the caller.
        top_disk: Vec::new(),
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
    }
}
