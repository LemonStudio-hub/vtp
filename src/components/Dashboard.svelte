<!--
  Dashboard Component

  Main dashboard interface for the VTP node, responsible for:
  1. Displaying VDF computation progress
  2. Showing real-time statistics
  3. Providing control buttons (start/pause/resume)
  4. Displaying the event log

  Component structure:
  - IdentityBadge: Node identity indicator
  - VDFCanvas: VDF progress visualization
  - StatsPanel: Real-time statistics panel
  - EventLog: Event log
  - Control button area

  Usage example:
  ```svelte
  <Dashboard />
  ```
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { workerStore, workerState } from '$stores/worker';
  import StatsPanel from './StatsPanel.svelte';
  import EventLog from './EventLog.svelte';
  import IdentityBadge from './IdentityBadge.svelte';
  import VDFCanvas from './VDFCanvas.svelte';

  /** Reference to the Web Worker instance */
  let worker: Worker | null = null;

  /** Store subscription cancellation function */
  let unsubscribe: () => void;

  /** Animation trigger for staggered entrance */
  let mounted = false;

  /**
   * Subscribe to the Worker Store when the component mounts
   *
   * Obtains a reference to the Worker instance for sending control commands
   */
  onMount(() => {
    unsubscribe = workerStore.subscribe((w) => {
      worker = w;
    });

    // Trigger staggered entrance animations
    setTimeout(() => {
      mounted = true;
    }, 200);
  });

  /**
   * Unsubscribe when the component is destroyed
   *
   * Prevents memory leaks
   */
  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });

  /**
   * Handle the start button click
   *
   * Sends a start command to the Worker with VDF configuration parameters
   * and updates the store with totalSteps for progress calculation.
   */
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

  /**
   * Handle the pause button click
   *
   * Sends a pause command to the Worker and updates the store
   */
  function handlePause() {
    if (worker) {
      workerState.update((s) => ({ ...s, isPaused: true }));
      worker.postMessage({ type: 'pause' });
    }
  }

  /**
   * Handle the resume button click
   *
   * Sends a resume command to the Worker and updates the store
   */
  function handleResume() {
    if (worker) {
      workerState.update((s) => ({ ...s, isPaused: false }));
      worker.postMessage({ type: 'resume' });
    }
  }
</script>

<!-- Main dashboard layout -->
<div class="dashboard" class:mounted>
  <!-- Header: identity indicator and title -->
  <header class="header-animate">
    <IdentityBadge />
    <div class="title-container">
      <h1>VTP Node</h1>
      <span class="subtitle">Verifiable Time Proof</span>
    </div>
    <div class="status-indicator" class:running={$workerState.isRunning}>
      <span class="status-dot"></span>
      <span class="status-text">{$workerState.isRunning ? 'Computing' : 'Ready'}</span>
    </div>
  </header>

  <!-- Main content grid -->
  <div class="content-grid">
    <!-- Left column: VDF visualization and stats -->
    <div class="left-column">
      <!-- VDF progress visualization area -->
      <section class="vdf-section glass-card">
        <VDFCanvas />
      </section>

      <!-- Statistics panel -->
      <section class="stats-section glass-card">
        <StatsPanel />
      </section>
    </div>

    <!-- Right column: controls and event log -->
    <div class="right-column">
      <!-- Control button area -->
      <section class="controls glass-card">
        <h3 class="section-title">Controls</h3>
        <div class="button-group">
          <!-- Start button: only enabled when not running -->
          <button class="btn btn-start" on:click={handleStart} disabled={$workerState.isRunning}>
            <span class="btn-icon">▶</span>
            <span>Start</span>
          </button>

          <!-- Pause button: only enabled when running and not already paused -->
          <button
            class="btn btn-pause"
            on:click={handlePause}
            disabled={!$workerState.isRunning || $workerState.isPaused}
          >
            <span class="btn-icon">⏸</span>
            <span>Pause</span>
          </button>

          <!-- Resume button: only enabled when paused -->
          <button class="btn btn-resume" on:click={handleResume} disabled={!$workerState.isPaused}>
            <span class="btn-icon">▶</span>
            <span>Resume</span>
          </button>
        </div>
      </section>

      <!-- Event log area -->
      <section class="events-section glass-card">
        <EventLog />
      </section>
    </div>
  </div>
</div>

<style>
  /* Dashboard main container */
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    opacity: 0;
    transform: translateY(30px);
    transition: all 0.6s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .dashboard.mounted {
    opacity: 1;
    transform: translateY(0);
  }

  /* Header area */
  .header-animate {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 1.5rem 2rem;
    background: rgba(22, 33, 62, 0.6);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(0, 255, 136, 0.1);
    border-radius: 16px;
    opacity: 0;
    transform: translateX(-20px);
    animation: slideIn 0.5s ease forwards;
    animation-delay: 0.3s;
  }

  @keyframes slideIn {
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .title-container {
    flex: 1;
  }

  /* Title styles */
  h1 {
    margin: 0;
    font-size: 2rem;
    font-weight: 700;
    background: linear-gradient(135deg, #00ff88 0%, #00cc6a 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    line-height: 1.2;
  }

  .subtitle {
    font-size: 0.875rem;
    color: #666;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  /* Status indicator */
  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 20px;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #666;
    transition: all 0.3s ease;
  }

  .status-indicator.running .status-dot {
    background: #00ff88;
    box-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
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
    font-size: 0.875rem;
    color: #888;
    font-family: 'Courier New', monospace;
  }

  .status-indicator.running .status-text {
    color: #00ff88;
  }

  /* Content grid layout */
  .content-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
  }

  @media (max-width: 1024px) {
    .content-grid {
      grid-template-columns: 1fr;
    }
  }

  .left-column,
  .right-column {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  /* Glass card effect */
  .glass-card {
    background: rgba(22, 33, 62, 0.6);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(0, 255, 136, 0.1);
    border-radius: 16px;
    padding: 1.5rem;
    opacity: 0;
    transform: translateY(20px);
    animation: fadeInUp 0.6s ease forwards;
  }

  .glass-card:nth-child(1) {
    animation-delay: 0.4s;
  }
  .glass-card:nth-child(2) {
    animation-delay: 0.5s;
  }
  .glass-card:nth-child(3) {
    animation-delay: 0.6s;
  }
  .glass-card:nth-child(4) {
    animation-delay: 0.7s;
  }

  @keyframes fadeInUp {
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* Section title */
  .section-title {
    margin: 0 0 1rem 0;
    font-size: 0.875rem;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  /* VDF visualization area */
  .vdf-section {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 400px;
  }

  /* Statistics panel area */
  .stats-section {
    background: rgba(22, 33, 62, 0.6);
  }

  /* Control button container */
  .controls {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .button-group {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
  }

  /* Generic button styles */
  .btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.875rem 1.5rem;
    font-size: 0.9375rem;
    font-weight: 600;
    border: none;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    flex: 1;
    min-width: 120px;
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
    font-size: 1rem;
  }

  /* Start button */
  .btn-start:not(:disabled) {
    background: linear-gradient(135deg, #00ff88 0%, #00cc6a 100%);
    color: #0a0a1a;
    box-shadow: 0 4px 15px rgba(0, 255, 136, 0.3);
  }

  .btn-start:not(:disabled):hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(0, 255, 136, 0.4);
  }

  .btn-start:not(:disabled):active {
    transform: translateY(0);
  }

  /* Pause button */
  .btn-pause:not(:disabled) {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    color: #0a0a1a;
    box-shadow: 0 4px 15px rgba(245, 158, 11, 0.3);
  }

  .btn-pause:not(:disabled):hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(245, 158, 11, 0.4);
  }

  /* Resume button */
  .btn-resume:not(:disabled) {
    background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
    color: #fff;
    box-shadow: 0 4px 15px rgba(99, 102, 241, 0.3);
  }

  .btn-resume:not(:disabled):hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(99, 102, 241, 0.4);
  }

  /* Disabled button styles */
  .btn:disabled {
    background: rgba(255, 255, 255, 0.05);
    color: #444;
    cursor: not-allowed;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  /* Event log area */
  .events-section {
    flex: 1;
    max-height: 400px;
    overflow: hidden;
  }

  @media (max-width: 768px) {
    .header-animate {
      flex-direction: column;
      text-align: center;
    }

    .title-container {
      display: flex;
      flex-direction: column;
      align-items: center;
    }

    .button-group {
      flex-direction: column;
    }

    .btn {
      width: 100%;
    }
  }
</style>
