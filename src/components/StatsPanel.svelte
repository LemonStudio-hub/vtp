<!--
  StatsPanel Component

  Displays real-time statistics for VDF computation, including:
  - Real-time speed (steps/sec) with micro-trend bars
  - Cumulative steps
  - Uptime
  - Draw count
  - Luck index
  - Memory usage
  - Peak speed

  Data source:
  Subscribes to workerState via Svelte Store for real-time data.
  Uses utility functions from $utils for formatting.
-->

<script lang="ts">
  import { workerState } from '$stores/worker';
  import { isVisible } from '$stores/visibility';
  import { formatNumber, formatSpeed, formatTime, formatBytes } from '$utils';

  /** Animated values for smooth transitions */
  let displaySpeed = 0;
  let displayStep = 0;
  let displayUptime = 0;

  /**
   * Reactive animation updates.
   * Smoothly interpolates displayed values towards actual values.
   * Only animates when the tab is visible to conserve resources.
   */
  $: {
    if ($isVisible) {
      animateValue('speed', $workerState.speed);
      animateValue('step', $workerState.currentStep);
      animateValue('uptime', $workerState.uptime);
    } else {
      displaySpeed = $workerState.speed;
      displayStep = $workerState.currentStep;
      displayUptime = $workerState.uptime;
    }
  }

  /** Micro-trend bars from recent speed history */
  $: trendBars = $workerState.speedHistory.slice(-12);

  function animateValue(key: string, target: number) {
    const current = key === 'speed' ? displaySpeed : key === 'step' ? displayStep : displayUptime;
    const diff = target - current;

    if (Math.abs(diff) < 1) {
      if (key === 'speed') displaySpeed = target;
      else if (key === 'step') displayStep = target;
      else displayUptime = target;
      return;
    }

    const step = diff * 0.1;
    requestAnimationFrame(() => {
      if (key === 'speed') displaySpeed += step;
      else if (key === 'step') displayStep += step;
      else displayUptime += step;
    });
  }

  function getLuckColor(percent: number): string {
    if (percent >= 120) return 'luck-excellent';
    if (percent >= 100) return 'luck-good';
    if (percent >= 80) return 'luck-normal';
    return 'luck-low';
  }

  /** Normalize trend bars to [0, 1] range */
  function normalizeTrend(values: number[]): number[] {
    if (values.length === 0) return [];
    const max = Math.max(...values) || 1;
    return values.map((v) => v / max);
  }
</script>

<div class="stats-grid">
  <!-- Speed with trend -->
  <div class="stat-item stat-wide">
    <div class="stat-header">
      <span class="stat-icon">⚡</span>
      <span class="label">Speed</span>
    </div>
    <div class="stat-body">
      <span class="value speed-value">
        {formatSpeed(displaySpeed)}
        <span class="unit">steps/s</span>
      </span>
      <div class="trend-bars">
        {#each normalizeTrend(trendBars) as bar}
          <div class="trend-bar" style="height: {Math.max(bar * 100, 4)}%"></div>
        {/each}
      </div>
    </div>
  </div>

  <!-- Total Steps -->
  <div class="stat-item">
    <div class="stat-header">
      <span class="stat-icon">📊</span>
      <span class="label">Total Steps</span>
    </div>
    <span class="value">{formatNumber(displayStep)}</span>
  </div>

  <!-- Uptime -->
  <div class="stat-item">
    <div class="stat-header">
      <span class="stat-icon">⏱</span>
      <span class="label">Uptime</span>
    </div>
    <span class="value mono">{formatTime(displayUptime)}</span>
  </div>

  <!-- Draws -->
  <div class="stat-item">
    <div class="stat-header">
      <span class="stat-icon">🎯</span>
      <span class="label">Draws</span>
    </div>
    <span class="value">{formatNumber($workerState.winnerCount)}</span>
  </div>

  <!-- Luck Index -->
  <div class="stat-item">
    <div class="stat-header">
      <span class="stat-icon">🍀</span>
      <span class="label">Luck Index</span>
    </div>
    <span class="value {getLuckColor($workerState.luckPercentage)}">
      {$workerState.luckPercentage.toFixed(0)}%
      {#if $workerState.luckPercentage > 100}
        <span class="luck-badge">Lucky</span>
      {/if}
    </span>
  </div>

  <!-- Memory -->
  <div class="stat-item">
    <div class="stat-header">
      <span class="stat-icon">💾</span>
      <span class="label">Memory</span>
    </div>
    <span class="value" class:muted={$workerState.memoryUsage === 0}>
      {$workerState.memoryUsage > 0 ? formatBytes($workerState.memoryUsage) : 'N/A'}
    </span>
  </div>

  <!-- Peak Speed -->
  <div class="stat-item">
    <div class="stat-header">
      <span class="stat-icon">🚀</span>
      <span class="label">Peak Speed</span>
    </div>
    <span class="value">
      {formatSpeed($workerState.peakSpeed)}
      <span class="unit">steps/s</span>
    </span>
  </div>
</div>

<style>
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: var(--space-md);
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
    padding: 0.875rem;
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    transition: all 0.3s ease;
  }

  .stat-item:hover {
    background: var(--color-surface-hover);
    border-color: var(--color-border-accent);
    transform: translateY(-2px);
  }

  .stat-wide {
    grid-column: span 1;
  }

  .stat-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .stat-icon {
    font-size: 0.875rem;
  }

  .label {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .stat-body {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: var(--space-sm);
  }

  .value {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-accent-green);
    font-family: var(--font-mono);
    line-height: 1;
  }

  .value.mono {
    font-size: 1.25rem;
  }

  .value.muted {
    color: var(--color-text-muted);
    font-weight: 400;
    font-size: 1.25rem;
  }

  .unit {
    font-size: 0.625rem;
    color: var(--color-text-muted);
    margin-left: 0.25rem;
  }

  .speed-value {
    display: flex;
    align-items: baseline;
  }

  /* Trend bars */
  .trend-bars {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 28px;
    flex-shrink: 0;
  }

  .trend-bar {
    width: 4px;
    min-height: 2px;
    background: var(--color-accent-green);
    border-radius: 1px;
    opacity: 0.6;
    transition: height 0.3s ease;
  }

  .trend-bar:last-child {
    opacity: 1;
  }

  /* Luck colors */
  .luck-excellent {
    color: #10b981;
  }
  .luck-good {
    color: var(--color-accent-green);
  }
  .luck-normal {
    color: var(--color-accent-amber);
  }
  .luck-low {
    color: var(--color-accent-red);
  }

  .luck-badge {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    font-size: 0.5625rem;
    font-weight: 600;
    background: rgba(0, 255, 136, 0.15);
    color: var(--color-accent-green);
    border-radius: 4px;
    margin-left: 0.375rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    vertical-align: middle;
  }

  @media (max-width: 768px) {
    .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    .value {
      font-size: 1.125rem;
    }

    .stat-wide {
      grid-column: span 1;
    }
  }
</style>
