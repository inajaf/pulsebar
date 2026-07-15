<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import MetricCard from "$lib/MetricCard.svelte";
  import { history, metrics, startMetricsStream } from "$lib/stores/metrics";
  import "$lib/styles/tokens.css";
  import "$lib/styles/app.css";

  let stop: (() => void) | null = null;

  onMount(async () => {
    // Inside Tauri the window is transparent + vibrancy-backed; the .native
    // tokens make the UI a dark glass HUD over it.
    if ("__TAURI_INTERNALS__" in window) {
      document.documentElement.classList.add("native");
    }
    stop = await startMetricsStream();
  });

  onDestroy(() => {
    stop?.();
  });

  $: lastUpdated = $metrics.timestamp_ms
    ? new Date($metrics.timestamp_ms).toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      })
    : "--:--:--";

  $: cpuHistory = $history.map((s) => s.cpu.percent);
  $: gpuHistory = $history.map((s) => s.gpu.percent);
  $: ramHistory = $history.map((s) => s.ram.percent);
  $: diskHistory = $history.map((s) => s.disk.percent);
</script>

<div class="dashboard">
  <div class="dashboard-header">
    <div class="header-title">
      <span class="live-dot"></span>
      <h1>System Monitor</h1>
    </div>
    <span class="timestamp">{lastUpdated}</span>
  </div>

  <div class="grid">
    <MetricCard label="CPU" icon="cpu" metric={$metrics.cpu} history={cpuHistory} processes={$metrics.top_cpu} procMode="cpu" />
    <MetricCard
      label="GPU"
      icon="gpu"
      metric={$metrics.gpu}
      history={gpuHistory}
      showBytes
      procNote="per-process GPU stats are private API on macOS"
    />
    <MetricCard label="Memory" icon="ram" metric={$metrics.ram} history={ramHistory} showBytes processes={$metrics.top_mem} procMode="mem" />
    <MetricCard label="Disk" icon="disk" metric={$metrics.disk} history={diskHistory} showBytes processes={$metrics.top_disk} procMode="mem" />
  </div>
</div>
