import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";

export interface UsageMetric {
  percent: number | null;
  used_bytes: number | null;
  total_bytes: number | null;
  temp_celsius: number | null;
}

/** cpu_percent is per-core style (Activity Monitor convention). */
export interface ProcessEntry {
  name: string;
  cpu_percent: number;
  mem_bytes: number;
}

export interface MetricsSnapshot {
  cpu: UsageMetric;
  ram: UsageMetric;
  disk: UsageMetric;
  gpu: UsageMetric;
  top_cpu: ProcessEntry[];
  top_mem: ProcessEntry[];
  /** Largest installed apps by on-disk size (slow background scan). */
  top_disk: ProcessEntry[];
  timestamp_ms: number;
}

const EMPTY_METRIC: UsageMetric = {
  percent: null,
  used_bytes: null,
  total_bytes: null,
  temp_celsius: null,
};

export const EMPTY_SNAPSHOT: MetricsSnapshot = {
  cpu: EMPTY_METRIC,
  ram: EMPTY_METRIC,
  disk: EMPTY_METRIC,
  gpu: EMPTY_METRIC,
  top_cpu: [],
  top_mem: [],
  top_disk: [],
  timestamp_ms: 0,
};

/** How many samples the sparklines keep (~1 minute at the visible poll rate). */
const HISTORY_LENGTH = 60;

export const metrics = writable<MetricsSnapshot>(EMPTY_SNAPSHOT);
export const history = writable<MetricsSnapshot[]>([]);

function push(snapshot: MetricsSnapshot) {
  metrics.set(snapshot);
  history.update((h) => [...h.slice(-(HISTORY_LENGTH - 1)), snapshot]);
}

/**
 * Fetches the current snapshot for an instant first paint, then subscribes to
 * the backend's live event stream. Returns a cleanup function to unsubscribe.
 *
 * Outside Tauri (plain `vite dev` in a browser) there is no backend, so a
 * demo stream stands in — this keeps the UI developable in a browser tab.
 */
export async function startMetricsStream(): Promise<() => void> {
  if (!("__TAURI_INTERNALS__" in window)) {
    return startDemoStream();
  }

  try {
    const initial = await invoke<MetricsSnapshot>("get_current_metrics");
    push(initial);
  } catch (err) {
    console.error("failed to fetch initial metrics", err);
  }

  let unlisten: UnlistenFn | null = await listen<MetricsSnapshot>(
    "metrics://update",
    (event) => push(event.payload),
  );

  return () => {
    unlisten?.();
    unlisten = null;
  };
}

function startDemoStream(): () => void {
  const GIB = 1024 ** 3;
  const walk = { cpu: 34, ram: 62, disk: 71, gpu: 48 };
  const step = (v: number) => Math.min(98, Math.max(2, v + (Math.random() - 0.5) * 9));

  const sample = (): MetricsSnapshot => {
    walk.cpu = step(walk.cpu);
    walk.ram = step(walk.ram);
    walk.gpu = step(walk.gpu);
    walk.disk = step(walk.disk);
    return {
      cpu: { percent: walk.cpu, used_bytes: null, total_bytes: null, temp_celsius: 52 + walk.cpu / 4 },
      ram: { percent: walk.ram, used_bytes: (walk.ram / 100) * 16 * GIB, total_bytes: 16 * GIB, temp_celsius: null },
      disk: { percent: walk.disk, used_bytes: (walk.disk / 100) * 512 * GIB, total_bytes: 512 * GIB, temp_celsius: null },
      gpu: { percent: walk.gpu, used_bytes: (walk.gpu / 100) * 8 * GIB, total_bytes: null, temp_celsius: 44 + walk.gpu / 3 },
      top_cpu: [
        { name: "WindowServer", cpu_percent: 18 + Math.random() * 30, mem_bytes: 0.9 * GIB },
        { name: "Google Chrome", cpu_percent: 9 + Math.random() * 15, mem_bytes: 1.4 * GIB },
        { name: "node", cpu_percent: 3 + Math.random() * 8, mem_bytes: 0.3 * GIB },
      ],
      top_mem: [
        { name: "Google Chrome", cpu_percent: 12, mem_bytes: (1.3 + Math.random() * 0.4) * GIB },
        { name: "Docker", cpu_percent: 2, mem_bytes: 1.1 * GIB },
        { name: "WindowServer", cpu_percent: 20, mem_bytes: 0.9 * GIB },
      ],
      top_disk: [
        { name: "Xcode", cpu_percent: 0, mem_bytes: 11.8 * GIB },
        { name: "Docker", cpu_percent: 0, mem_bytes: 3.9 * GIB },
        { name: "Google Chrome", cpu_percent: 0, mem_bytes: 2.4 * GIB },
      ],
      timestamp_ms: Date.now(),
    };
  };

  push(sample());
  const id = setInterval(() => push(sample()), 1000);
  return () => clearInterval(id);
}
