<!--
  Dashboard Component

  Main dashboard interface for the VTP node, redesigned with a 3-section layout:

  Section 1 — Hero area: IdentityBadge + ProgressHero side by side
  Section 2 — Metrics grid: StatsPanel cards + MemoryGauge + SpeedChart
  Section 3 — Detail panels: SystemHealth + Controls + EventLog

  Component structure:
  - IdentityBadge: Node identity indicator
  - ProgressHero: Enhanced progress ring with key metrics
  - StatsPanel: Real-time statistics grid
  - MemoryGauge: WASM memory usage indicator
  - SpeedChart: Speed history sparkline
  - SystemHealth: Background subsystem status
  - EventLog: Event log with filtering
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { workerStore, workerState } from '$stores/worker';
  import StatsPanel from './StatsPanel.svelte';
  import EventLog from './EventLog.svelte';
  import IdentityBadge from './IdentityBadge.svelte';
  import ProgressHero from './ProgressHero.svelte';
  import SpeedChart from './SpeedChart.svelte';
  import MemoryGauge from './MemoryGauge.svelte';
  import SystemHealth from './SystemHealth.svelte';
  import NetworkStatus from './NetworkStatus.svelte';

  /** Reference to the Web Worker instance */
  let worker: Worker | null = null;

  /** Store subscription cancellation function */
  let unsubscribe: () => void;

  /** Animation trigger for staggered entrance */
  let mounted = false;

  onMount(() => {
    unsubscribe = workerStore.subscribe((w) => {
      worker = w;
    });
    setTimeout(() => {
      mounted = true;
    }, 200);
  });

  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });

  function handleStart() {
    if (worker) {
      const totalSteps = 1000000;
      workerState.update((s) => ({ ...s, totalSteps }));
      worker.postMessage({
        type: 'start',
        seed: new Uint8Array(32),
        total: totalSteps,
        k: 1000,
        tau: new Uint8Array(32),
        checkpointInterval: 100000
      });
    }
  }

  function handlePause() {
    if (worker) {
      workerState.update((s) => ({ ...s, isPaused: true }));
      worker.postMessage({ type: 'pause' });
    }
  }

  function handleResume() {
    if (worker) {
      workerState.update((s) => ({ ...s, isPaused: false }));
      worker.postMessage({ type: 'resume' });
    }
  }
</script>

<div class="dashboard" class:mounted>
  <!-- Section 1: Hero area -->
  <section class="hero-section glass-card">
    <div class="hero-left">
      <IdentityBadge />
    </div>
    <div class="hero-center">
      <ProgressHero />
    </div>
    <div class="hero-status">
      <div class="status-indicator" class:running={$workerState.isRunning}>
        <span class="status-dot"></span>
        <span class="status-text"
          >{$workerState.isRunning
            ? $workerState.isPaused
              ? 'Paused'
              : 'Computing'
            : 'Ready'}</span
        >
      </div>
    </div>
  </section>

  <!-- Section 2: Metrics grid -->
  <section class="metrics-section">
    <div class="stats-area glass-card">
      <StatsPanel />
    </div>
    <div class="memory-area glass-card">
      <MemoryGauge />
    </div>
    <div class="chart-area glass-card">
      <SpeedChart />
    </div>
  </section>

  <!-- Section 3: Detail panels -->
  <section class="detail-section">
    <div class="detail-left">
      <!-- System health -->
      <div class="health-area glass-card">
        <SystemHealth />
      </div>

      <!-- Network status -->
      <div class="network-area glass-card">
        <NetworkStatus />
      </div>

      <!-- Controls -->
      <div class="controls-area glass-card">
        <h3 class="section-title">Controls</h3>
        <div class="button-group">
          <button class="btn btn-start" on:click={handleStart} disabled={$workerState.isRunning}>
            <span class="btn-icon">▶</span>
            <span>Start</span>
          </button>
          <button
            class="btn btn-pause"
            on:click={handlePause}
            disabled={!$workerState.isRunning || $workerState.isPaused}
          >
            <span class="btn-icon">⏸</span>
            <span>Pause</span>
          </button>
          <button class="btn btn-resume" on:click={handleResume} disabled={!$workerState.isPaused}>
            <span class="btn-icon">▶</span>
            <span>Resume</span>
          </button>
        </div>
      </div>
    </div>

    <div class="detail-right glass-card">
      <EventLog />
    </div>
  </section>
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
    opacity: 0;
    transform: translateY(30px);
    transition: all 0.6s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .dashboard.mounted {
    opacity: 1;
    transform: translateY(0);
  }

  /* ── Glass card base ── */
  :global(.glass-card) {
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    border: var(--glass-border);
    border-radius: var(--radius-lg);
    padding: var(--space-lg);
    opacity: 0;
    transform: translateY(20px);
    animation: fadeInUp 0.6s ease forwards;
  }

  :global(.glass-card:nth-child(1)) {
    animation-delay: 0.3s;
  }
  :global(.glass-card:nth-child(2)) {
    animation-delay: 0.4s;
  }
  :global(.glass-card:nth-child(3)) {
    animation-delay: 0.5s;
  }
  :global(.glass-card:nth-child(4)) {
    animation-delay: 0.6s;
  }

  @keyframes fadeInUp {
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* ── Section 1: Hero ── */
  .hero-section {
    display: flex;
    align-items: center;
    gap: var(--space-xl);
    animation-delay: 0.2s;
  }

  .hero-left {
    flex-shrink: 0;
  }

  .hero-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }

  .hero-status {
    flex-shrink: 0;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-text-muted);
    transition: all 0.3s ease;
  }

  .status-indicator.running .status-dot {
    background: var(--color-accent-green);
    box-shadow: 0 0 10px var(--color-accent-green-glow);
    animation: statusPulse 1.5s ease-in-out infinite;
  }

  @keyframes statusPulse {
    0%,
    100% {
      transform: scale(1);
      opacity: 1;
    }
    50% {
      transform: scale(1.2);
      opacity: 0.8;
    }
  }

  .status-text {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    font-family: var(--font-mono);
    letter-spacing: 0.02em;
  }

  .status-indicator.running .status-text {
    color: var(--color-accent-green);
  }

  /* ── Section 2: Metrics ── */
  .metrics-section {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: var(--space-xl);
    align-items: stretch;
  }

  .stats-area {
    min-width: 0;
  }

  .memory-area {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-lg) var(--space-md);
  }

  .chart-area {
    min-width: 280px;
    max-width: 360px;
  }

  /* ── Section 3: Details ── */
  .detail-section {
    display: grid;
    grid-template-columns: 1fr 1.2fr;
    gap: var(--space-xl);
  }

  .detail-left {
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  .controls-area {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .section-title {
    margin: 0;
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .button-group {
    display: flex;
    gap: var(--space-md);
    flex-wrap: wrap;
  }

  .btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    padding: 0.75rem 1.25rem;
    font-size: 0.875rem;
    font-weight: 600;
    font-family: var(--font-sans);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    flex: 1;
    min-width: 100px;
    position: relative;
    overflow: hidden;
  }

  .btn::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
    transition: left 0.5s ease;
  }

  .btn:not(:disabled):hover::before {
    left: 100%;
  }

  .btn-icon {
    font-size: 0.875rem;
  }

  .btn-start:not(:disabled) {
    background: linear-gradient(135deg, #00ff88 0%, #00cc6a 100%);
    color: #0a0a1a;
    box-shadow: 0 4px 15px rgba(0, 255, 136, 0.3);
  }

  .btn-start:not(:disabled):hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(0, 255, 136, 0.4);
  }

  .btn-pause:not(:disabled) {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    color: #0a0a1a;
    box-shadow: 0 4px 15px rgba(245, 158, 11, 0.3);
  }

  .btn-pause:not(:disabled):hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(245, 158, 11, 0.4);
  }

  .btn-resume:not(:disabled) {
    background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
    color: #fff;
    box-shadow: 0 4px 15px rgba(99, 102, 241, 0.3);
  }

  .btn-resume:not(:disabled):hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(99, 102, 241, 0.4);
  }

  .btn:disabled {
    background: rgba(255, 255, 255, 0.04);
    color: var(--color-text-muted);
    cursor: not-allowed;
    border: 1px solid var(--color-border);
  }

  .detail-right {
    max-height: 480px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* ── Responsive ── */
  @media (max-width: 1024px) {
    .metrics-section {
      grid-template-columns: 1fr auto;
    }

    .chart-area {
      grid-column: 1 / -1;
      min-width: 0;
      max-width: none;
    }

    .detail-section {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 768px) {
    .hero-section {
      flex-direction: column;
      text-align: center;
    }

    .metrics-section {
      grid-template-columns: 1fr;
    }

    .memory-area {
      padding: var(--space-md);
    }

    .button-group {
      flex-direction: column;
    }

    .btn {
      width: 100%;
    }
  }
</style>
