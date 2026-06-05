<!--
  StatsPanel Component

  Displays real-time statistics for VDF computation, including:
  - Real-time speed (steps/sec)
  - Cumulative steps
  - Uptime
  - Draw count
  - Luck index

  Data source:
  Subscribes to workerState via Svelte Store for real-time data

  Usage example:
  ```svelte
  <StatsPanel />
  ```
-->

<script lang="ts">
  import { workerState } from '$stores/worker';

  /** Animated values for smooth transitions */
  let displaySpeed = 0;
  let displayStep = 0;
  let displayUptime = 0;

  /**
   * Reactive animation updates
   *
   * Smoothly interpolates displayed values towards actual values
   */
  $: {
    animateValue('speed', $workerState.speed);
    animateValue('step', $workerState.currentStep);
    animateValue('uptime', $workerState.uptime);
  }

  /**
   * Animate a numeric value
   *
   * Smoothly transitions from current displayed value to target value
   *
   * @param key - The value key to animate
   * @param target - The target value
   */
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

  /**
   * Format a number with locale-specific thousands separators
   *
   * @param num - The number to format
   * @returns Formatted string
   *
   * @example
   * formatNumber(1234567) // "1,234,567"
   */
  function formatNumber(num: number): string {
    return Math.floor(num).toLocaleString();
  }

  /**
   * Format speed value to human-readable format
   *
   * Automatically selects appropriate unit (M, K, or raw).
   *
   * @param speed - Speed value in steps/second
   * @returns Formatted string
   *
   * @example
   * formatSpeed(1500000) // "1.5M"
   * formatSpeed(1500) // "1.5K"
   * formatSpeed(500) // "500"
   */
  function formatSpeed(speed: number): string {
    if (speed >= 1000000) {
      return `${(speed / 1000000).toFixed(1)}M`;
    } else if (speed >= 1000) {
      return `${(speed / 1000).toFixed(1)}K`;
    }
    return speed.toFixed(0);
  }

  /**
   * Format time in seconds to HH:MM:SS format
   *
   * @param seconds - Time in seconds
   * @returns Formatted time string
   *
   * @example
   * formatTime(3661) // "01:01:01"
   */
  function formatTime(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }

  /**
   * Get luck color based on percentage
   *
   * Returns appropriate color class based on luck value
   */
  function getLuckColor(percent: number): string {
    if (percent >= 120) return 'luck-excellent';
    if (percent >= 100) return 'luck-good';
    if (percent >= 80) return 'luck-normal';
    return 'luck-low';
  }
</script>

<!-- Stats panel grid layout -->
<div class="stats-grid">
  <!-- Real-time speed -->
  <div class="stat-item">
    <div class="stat-header">
      <span class="stat-icon">⚡</span>
      <span class="label">Speed</span>
    </div>
    <span class="value speed-value">
      {formatSpeed(displaySpeed)}
      <span class="unit">steps/s</span>
    </span>
  </div>

  <!-- Cumulative steps -->
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

  <!-- Draw count -->
  <div class="stat-item">
    <div class="stat-header">
      <span class="stat-icon">🎯</span>
      <span class="label">Draws</span>
    </div>
    <span class="value">{formatNumber($workerState.winnerCount)}</span>
  </div>

  <!-- Luck index -->
  <div class="stat-item luck-stat">
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
</div>

<style>
  /* Stats grid layout */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 1.25rem;
  }

  /* Stat item container */
  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.02);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    transition: all 0.3s ease;
  }

  .stat-item:hover {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(0, 255, 136, 0.1);
    transform: translateY(-2px);
  }

  /* Stat header */
  .stat-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .stat-icon {
    font-size: 1rem;
  }

  /* Label styles */
  .label {
    font-size: 0.75rem;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  /* Value styles */
  .value {
    font-size: 1.75rem;
    font-weight: 700;
    color: #00ff88;
    font-family: 'Courier New', monospace;
    line-height: 1;
  }

  .value.mono {
    font-size: 1.5rem;
  }

  .unit {
    font-size: 0.75rem;
    color: #666;
    margin-left: 0.25rem;
  }

  .speed-value {
    display: flex;
    align-items: baseline;
  }

  /* Luck colors */
  .luck-excellent {
    color: #10b981;
  }

  .luck-good {
    color: #00ff88;
  }

  .luck-normal {
    color: #f59e0b;
  }

  .luck-low {
    color: #ef4444;
  }

  .luck-badge {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    font-size: 0.625rem;
    font-weight: 600;
    background: rgba(0, 255, 136, 0.2);
    color: #00ff88;
    border-radius: 4px;
    margin-left: 0.5rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  @media (max-width: 768px) {
    .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    .value {
      font-size: 1.25rem;
    }
  }
</style>
