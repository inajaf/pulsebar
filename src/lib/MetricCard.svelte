<script lang="ts">
  import { tweened } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import Gauge from "./Gauge.svelte";
  import Sparkline from "./Sparkline.svelte";
  import type { ProcessEntry, UsageMetric } from "./stores/metrics";

  export let label: string;
  export let icon: "cpu" | "gpu" | "ram" | "disk";
  export let metric: UsageMetric;
  export let history: (number | null)[] = [];
  export let showBytes = false;
  /** Top processes to list under the gauge; `procMode` picks the value shown. */
  export let processes: ProcessEntry[] = [];
  export let procMode: "cpu" | "mem" = "cpu";
  /** Shown where a process list would be when the platform can't provide one. */
  export let procNote = "";

  function procValue(p: ProcessEntry): string {
    return procMode === "cpu" ? `${p.cpu_percent.toFixed(0)}%` : fmtBytes(p.mem_bytes);
  }

  // Same duration/easing as the gauge so number and arc move together.
  // JS tweens don't see the CSS reduced-motion override, so gate manually.
  const reducedMotion =
    typeof window !== "undefined" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const display = tweened(0, { duration: reducedMotion ? 0 : 600, easing: cubicOut });
  $: display.set(metric.percent ?? 0);

  type Level = "low" | "medium" | "high" | "very-high" | "unavailable";

  $: level = (
    metric.percent === null
      ? "unavailable"
      : metric.percent >= 99
        ? "very-high"
        : metric.percent >= 85
          ? "high"
          : metric.percent >= 60
            ? "medium"
            : "low"
  ) as Level;

  // Temperature severity mirrors the usage ramp: amber when hot, red when
  // critical. Thresholds are conservative for consumer silicon.
  $: tempLevel =
    metric.temp_celsius === null
      ? null
      : metric.temp_celsius >= 90
        ? "hot"
        : metric.temp_celsius >= 75
          ? "warm"
          : "normal";

  function fmtBytes(n: number | null): string {
    if (n === null) return "--";
    const gb = n / 1024 ** 3;
    return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(n / 1024 ** 2).toFixed(0)} MB`;
  }

  const ICON_PATHS: Record<string, string> = {
    cpu: "M9 2v2M15 2v2M9 20v2M15 20v2M2 9h2M2 15h2M20 9h2M20 15h2M6 6h12v12H6zM9.5 9.5h5v5h-5z",
    gpu: "M2 7h20v10H4a2 2 0 0 1-2-2V7zM2 7V5m5 12v2m10-2v2M14.5 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0zM18 9.5h1.5",
    ram: "M3 7h18v8H3zM3 15v2m4-2v2m4-2v2m4-2v2m4-2v2M6.5 10v2m3.5-2v2m3.5-2v2m3.5-2v2",
    disk: "M12 3c-4.4 0-8 1.3-8 3v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6c0-1.7-3.6-3-8-3zM4 6c0 1.7 3.6 3 8 3s8-1.3 8-3M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3",
  };
</script>

<article class="card" data-level={level}>
  <header class="card-header">
    <span class="card-id">
      <svg class="card-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d={ICON_PATHS[icon]} />
      </svg>
      <span class="card-label">{label}</span>
    </span>
    {#if metric.temp_celsius !== null}
      <span class="pill" data-temp={tempLevel}>{Math.round(metric.temp_celsius)}°C</span>
    {/if}
  </header>

  <div class="gauge-wrap">
    <Gauge value={metric.percent} />
    <div class="gauge-center">
      {#if metric.percent !== null}
        <span class="value">{Math.round($display)}<small>%</small></span>
        {#if level === "very-high"}
          <span class="status critical">▲ critical</span>
        {:else if level === "high"}
          <span class="status high">▲ high</span>
        {/if}
      {:else}
        <span class="value muted">--</span>
        <span class="status none">no sensor</span>
      {/if}
    </div>
  </div>

  {#if processes.length > 0}
    <ul class="proc-list">
      {#each processes as p}
        <li class="proc-row">
          <span class="proc-name">{p.name}</span>
          <span class="proc-leader" aria-hidden="true"></span>
          <span class="proc-val">{procValue(p)}</span>
        </li>
      {/each}
    </ul>
  {:else if procNote}
    <div class="proc-note">{procNote}</div>
  {/if}

  <footer class="card-foot">
    {#if showBytes && metric.used_bytes !== null}
      {#if metric.total_bytes !== null}
        <span class="bytes">{fmtBytes(metric.used_bytes)} <em>/ {fmtBytes(metric.total_bytes)}</em></span>
      {:else}
        <span class="bytes">{fmtBytes(metric.used_bytes)} <em>in use</em></span>
      {/if}
    {/if}
    <Sparkline values={history} {level} />
  </footer>
</article>

<style>
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 12px 10px;
    background: var(--surface);
    backdrop-filter: var(--glass);
    -webkit-backdrop-filter: var(--glass);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    transition: border-color 400ms ease, box-shadow 400ms ease, transform 200ms ease;
    overflow: hidden;
  }

  /* Subtle lift on hover — the HUD invites a closer look. */
  .card:hover {
    border-color: var(--border-strong);
    transform: translateY(-1px);
  }

  /* Glass edge: a hairline highlight along the card's top. */
  .card::before {
    content: "";
    position: absolute;
    top: 0;
    left: 10%;
    right: 10%;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--border-strong), transparent);
    pointer-events: none;
  }

  .card[data-level="very-high"] {
    border-color: var(--danger-soft);
    box-shadow: var(--shadow), var(--glow-critical);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 20px;
  }

  .card-id {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 700;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .card-icon {
    width: 15px;
    height: 15px;
    opacity: 0.85;
  }

  .gauge-wrap {
    position: relative;
    display: grid;
    place-items: center;
    flex: 1;
  }

  .gauge-center {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1px;
  }

  .value {
    font-size: 23px;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.01em;
    /* Digits keep a fixed width so the number doesn't wobble mid-tween. */
    font-variant-numeric: tabular-nums;
  }

  .value small {
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    margin-left: 1px;
  }

  .value.muted {
    color: var(--muted);
  }

  .status {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .status.critical {
    color: var(--danger);
  }

  .status.high {
    color: var(--warn);
  }

  .status.none {
    color: var(--muted);
    font-style: italic;
    text-transform: none;
    letter-spacing: 0.02em;
  }

  .proc-list {
    list-style: none;
    margin: 0;
    padding: 4px 0 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
    border-top: 1px solid var(--border);
  }

  .proc-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: 1.3;
  }

  .proc-name {
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  /* Dotted leader ties each name to its value, table-of-contents style. */
  .proc-leader {
    flex: 1;
    min-width: 8px;
    border-bottom: 1px dotted var(--border-strong);
    transform: translateY(-3px);
  }

  .proc-val {
    color: var(--text);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .proc-note {
    padding-top: 4px;
    border-top: 1px solid var(--border);
    font-size: 9.5px;
    font-style: italic;
    color: var(--muted);
    line-height: 1.35;
  }

  .card-foot {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-height: 44px;
    justify-content: flex-end;
  }

  .bytes {
    font-family: var(--font-mono);
    font-size: 10px;
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }

  .bytes em {
    font-style: normal;
    color: var(--muted);
  }

  .pill {
    font-family: var(--font-mono);
    font-size: 10px;
    font-variant-numeric: tabular-nums;
    color: var(--muted);
    background: var(--surface-strong);
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 1px 7px;
    white-space: nowrap;
    transition: color 400ms ease, background 400ms ease, border-color 400ms ease;
  }

  .pill[data-temp="warm"] {
    color: var(--warn);
    background: var(--warn-soft);
    border-color: transparent;
  }

  .pill[data-temp="hot"] {
    color: var(--danger);
    background: var(--danger-soft);
    border-color: transparent;
  }
</style>
