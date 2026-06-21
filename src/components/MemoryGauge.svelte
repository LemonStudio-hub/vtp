<!--
  MemoryGauge Component

  A vertical bar gauge showing current WASM heap memory usage.

  Features:
  - Color transitions: green (<50%), amber (50-80%), red (>80%)
  - Formatted byte display via formatBytes utility
  - Graceful "N/A" state when memory data is unavailable
-->

<script lang="ts">
  import { workerState } from '$stores/worker';
  import { formatBytes } from '$utils';
  import { clamp } from '$utils';

  /** Reference max memory (Chrome's jsHeapSizeLimit or 512MB fallback) */
  let maxMemory = 512 * 1024 * 1024;

  /** Try to read the real heap limit from performance.memory (Chrome only) */
  if (typeof performance !== 'undefined' && 'memory' in performance) {
    const mem = (performance as unknown as { memory: { jsHeapSizeLimit: number } }).memory;
    if (mem.jsHeapSizeLimit > 0) {
      maxMemory = mem.jsHeapSizeLimit;
    }
  }

  /** Compute fill percentage, clamped to [0, 1] */
  $: fillPercent =
    $workerState.memoryUsage > 0 ? clamp($workerState.memoryUsage / maxMemory, 0, 1) : 0;

  /** Determine color based on usage level */
  $: gaugeColor =
    fillPercent < 0.5
      ? 'var(--color-accent-green)'
      : fillPercent < 0.8
        ? 'var(--color-accent-amber)'
        : 'var(--color-accent-red)';

  /** Whether memory data is available */
  $: hasData = $workerState.memoryUsage > 0;
</script>

<div class="memory-gauge">
  <span class="gauge-label">Memory</span>

  <div class="gauge-track">
    <div
      class="gauge-fill"
      style="height: {fillPercent *
        100}%; background: {gaugeColor}; box-shadow: 0 0 8px {gaugeColor};"
    ></div>
  </div>

  <span class="gauge-value" class:unavailable={!hasData}>
    {hasData ? formatBytes($workerState.memoryUsage) : 'N/A'}
  </span>
</div>

<style>
  .memory-gauge {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-sm);
  }

  .gauge-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .gauge-track {
    width: 24px;
    height: 120px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: var(--radius-full);
    position: relative;
    overflow: hidden;
    border: 1px solid var(--color-border);
  }

  .gauge-fill {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    border-radius: var(--radius-full);
    transition:
      height 0.5s cubic-bezier(0.16, 1, 0.3, 1),
      background 0.5s ease;
  }

  .gauge-value {
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    text-align: center;
  }

  .gauge-value.unavailable {
    color: var(--color-text-muted);
    font-weight: 400;
  }
</style>
