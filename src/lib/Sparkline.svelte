<script lang="ts">
  /** Percent history, oldest first; nulls (sensor gaps) break the line. */
  export let values: (number | null)[] = [];
  /** Severity of the metric the sparkline belongs to; tints the line. */
  export let level: "low" | "medium" | "high" | "very-high" | "unavailable" = "low";

  const W = 120;
  const H = 28;
  const PAD = 2;

  // Unique per instance so the gradient defs of the four cards don't collide.
  const uid = `s${Math.random().toString(36).slice(2, 8)}`;

  $: tone = level === "very-high" ? "danger" : level === "high" ? "warn" : "ok";

  $: points = values.map((v, i) => ({
    x: values.length > 1 ? (i / (values.length - 1)) * W : W,
    y: v === null ? null : H - PAD - (Math.min(Math.max(v, 0), 100) / 100) * (H - PAD * 2),
  }));

  // One path per contiguous non-null run.
  $: linePath = points
    .map((p, i) => {
      if (p.y === null) return "";
      const prev = i > 0 ? points[i - 1] : null;
      const cmd = prev && prev.y !== null ? "L" : "M";
      return `${cmd} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`;
    })
    .join(" ");

  // Area wash under the last contiguous run, closed to the baseline.
  $: areaPath = (() => {
    let start = points.length - 1;
    while (start > 0 && points[start - 1].y !== null) start -= 1;
    const run = points.slice(start).filter((p) => p.y !== null);
    if (run.length < 2) return "";
    const line = run.map((p, i) => `${i === 0 ? "M" : "L"} ${p.x.toFixed(1)} ${p.y!.toFixed(1)}`).join(" ");
    return `${line} L ${run[run.length - 1].x.toFixed(1)} ${H} L ${run[0].x.toFixed(1)} ${H} Z`;
  })();

  $: hasData = points.some((p) => p.y !== null);

  // Latest non-null point; the "now" dot rides here. Positioned with CSS
  // percentages (not inside the stretched SVG) so it stays a true circle.
  $: last = (() => {
    for (let i = points.length - 1; i >= 0; i -= 1) {
      if (points[i].y !== null) return points[i];
    }
    return null;
  })();
</script>

{#if hasData}
  <div class="spark-wrap" data-tone={tone}>
    <svg class="spark" viewBox="0 0 {W} {H}" preserveAspectRatio="none" aria-hidden="true">
      <defs>
        <!-- stop-color comes from CSS below: some WebKit builds don't
             re-resolve currentColor inside <defs> when `color` changes. -->
        <linearGradient id="{uid}-fade" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-opacity="0.22" />
          <stop offset="100%" stop-opacity="0.02" />
        </linearGradient>
      </defs>
      {#if areaPath}
        <path class="spark-area" d={areaPath} fill="url(#{uid}-fade)" />
      {/if}
      <path class="spark-line" d={linePath} fill="none" />
    </svg>
    {#if last && last.y !== null}
      <span
        class="spark-dot"
        style="left: {((last.x / W) * 100).toFixed(2)}%; top: {((last.y / H) * 100).toFixed(2)}%"
      ></span>
    {/if}
  </div>
{/if}

<style>
  .spark-wrap {
    position: relative;
    width: 100%;
    height: 28px;
    color: var(--ok);
    transition: color 400ms ease;
  }

  .spark-wrap[data-tone="warn"] {
    color: var(--warn);
  }

  .spark-wrap[data-tone="danger"] {
    color: var(--danger);
  }

  .spark-wrap stop {
    stop-color: var(--ok);
  }

  .spark-wrap[data-tone="warn"] stop {
    stop-color: var(--warn);
  }

  .spark-wrap[data-tone="danger"] stop {
    stop-color: var(--danger);
  }

  .spark {
    display: block;
    width: 100%;
    height: 100%;
    opacity: 0.9;
    overflow: visible;
  }

  .spark-line {
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linejoin: round;
    stroke-linecap: round;
    vector-effect: non-scaling-stroke;
  }

  .spark-dot {
    position: absolute;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: currentColor;
    transform: translate(-50%, -50%);
    box-shadow: 0 0 6px currentColor;
    pointer-events: none;
  }
</style>
