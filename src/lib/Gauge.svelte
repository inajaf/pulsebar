<script lang="ts">
  import { tweened } from "svelte/motion";
  import { cubicOut } from "svelte/easing";

  export let value: number | null = null;
  export let size = 104;
  export let strokeWidth = 8;

  // Unique per instance so the four gauges' gradient defs don't collide.
  const uid = `g${Math.random().toString(36).slice(2, 8)}`;

  const SWEEP = 270; // degrees of visible arc
  const START = 135; // arc begins at bottom-left

  // JS tweens don't see the CSS reduced-motion override, so gate manually.
  const reducedMotion =
    typeof window !== "undefined" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const display = tweened(0, { duration: reducedMotion ? 0 : 600, easing: cubicOut });
  $: display.set(value === null ? 0 : Math.min(Math.max(value, 0), 100));

  $: level =
    value === null
      ? "unavailable"
      : value >= 99
        ? "very-high"
        : value >= 85
          ? "high"
          : value >= 60
            ? "medium"
            : "low";

  $: gradient = level === "very-high" ? "danger" : level === "high" ? "warn" : "ok";

  $: center = size / 2;
  // Inset leaves room for the ticks and the tip marker's glow.
  $: radius = (size - strokeWidth) / 2 - 5;

  function polar(angleDeg: number, r: number, c: number) {
    const a = (angleDeg * Math.PI) / 180;
    return { x: c + r * Math.cos(a), y: c + r * Math.sin(a) };
  }

  // 270° arc path; pathLength="100" lets dasharray speak in percent.
  $: arcPath = (() => {
    const from = polar(START, radius, center);
    const to = polar(START + SWEEP, radius, center);
    return `M ${from.x} ${from.y} A ${radius} ${radius} 0 1 1 ${to.x} ${to.y}`;
  })();

  // Tip marker rides the end of the filled arc.
  $: tip = polar(START + (SWEEP * $display) / 100, radius, center);

  // Hairline ticks at 0 / 25 / 50 / 75 / 100, just outside the track.
  $: ticks = [0, 25, 50, 75, 100].map((t) => {
    const angle = START + (SWEEP * t) / 100;
    const inner = polar(angle, radius + strokeWidth / 2 + 2, center);
    const outer = polar(angle, radius + strokeWidth / 2 + 5, center);
    return { x1: inner.x, y1: inner.y, x2: outer.x, y2: outer.y };
  });
</script>

<!-- Decorative: the numeric value next to the gauge carries the data. -->
<svg width={size} height={size} viewBox="0 0 {size} {size}" class="gauge" data-level={level} aria-hidden="true">
  <defs>
    <linearGradient id="{uid}-ok" x1="0%" y1="100%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="var(--ok-2)" />
      <stop offset="100%" stop-color="var(--ok)" />
    </linearGradient>
    <linearGradient id="{uid}-warn" x1="0%" y1="100%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="var(--warn-2)" />
      <stop offset="100%" stop-color="var(--warn)" />
    </linearGradient>
    <linearGradient id="{uid}-danger" x1="0%" y1="100%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="var(--danger-2)" />
      <stop offset="100%" stop-color="var(--danger)" />
    </linearGradient>
    <!-- Dashed overlay carves both track and fill into HUD-style segments. -->
    <mask id="{uid}-seg">
      <path
        d={arcPath}
        pathLength="100"
        stroke="#fff"
        stroke-width={strokeWidth + 3}
        fill="none"
        stroke-dasharray="1.1 0.55"
        stroke-dashoffset="0.55"
      />
    </mask>
  </defs>

  {#each ticks as t}
    <line class="gauge-tick" x1={t.x1} y1={t.y1} x2={t.x2} y2={t.y2} />
  {/each}

  <g mask="url(#{uid}-seg)">
    <path
      class="gauge-track"
      d={arcPath}
      pathLength="100"
      stroke-width={strokeWidth}
      fill="none"
    />

    {#if value !== null}
      <path
        class="gauge-fill"
        d={arcPath}
        pathLength="100"
        stroke-width={strokeWidth}
        fill="none"
        stroke-dasharray="{$display} 100"
        stroke="url(#{uid}-{gradient})"
      />
    {/if}
  </g>

  {#if value !== null}
    <circle class="gauge-tip" cx={tip.x} cy={tip.y} r="2.5" />
  {/if}
</svg>

<style>
  .gauge-track {
    stroke: var(--surface-strong);
    transition: stroke 400ms ease;
  }

  .gauge-tick {
    stroke: var(--border-strong);
    stroke-width: 1;
  }

  .gauge-fill {
    transition: filter 400ms ease;
  }

  .gauge-tip {
    fill: var(--text);
    transition: filter 400ms ease;
  }

  /* Meter rule: the unfilled track is a faint step of the active ramp,
     so state reads across the whole arc, not just the filled part. */
  .gauge[data-level="low"] .gauge-track,
  .gauge[data-level="medium"] .gauge-track {
    stroke: var(--ok-soft);
  }
  .gauge[data-level="high"] .gauge-track {
    stroke: var(--warn-soft);
  }
  .gauge[data-level="very-high"] .gauge-track {
    stroke: var(--danger-soft);
  }

  .gauge[data-level="low"] .gauge-fill,
  .gauge[data-level="medium"] .gauge-fill {
    filter: drop-shadow(0 0 4px var(--ok-soft));
  }

  .gauge[data-level="high"] .gauge-fill {
    filter: drop-shadow(0 0 5px var(--warn-soft));
  }

  .gauge[data-level="very-high"] .gauge-fill {
    filter: drop-shadow(0 0 7px var(--danger));
  }

  .gauge[data-level="very-high"] .gauge-tip {
    filter: drop-shadow(0 0 4px var(--danger));
  }

  /* Segmentation already textures the arc; unavailable just goes inert. */
  .gauge[data-level="unavailable"] .gauge-track {
    stroke: var(--border-strong);
  }

  @media (prefers-reduced-motion: reduce) {
    .gauge-fill,
    .gauge-track,
    .gauge-tip {
      transition: none;
    }
  }
</style>
