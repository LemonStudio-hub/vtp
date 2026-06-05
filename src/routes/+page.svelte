<!--
  @component +page (Root Page)

  The main entry page of the VTP (Verifiable Time Proof) application.

  This component is responsible for:
  1. Instantiating the Web Worker that performs time-proof computations
     in a background thread to avoid blocking the UI.
  2. Registering the worker instance into the shared `workerStore` so that
     child components (e.g., Dashboard) can communicate with it.
  3. Initializing the background keep-alive system:
     - VisibilityManager: detects tab visibility and system sleep
     - AudioKeepAlive: silent AudioContext prevents timer throttling
     - WorkerWatchdog: monitors worker health via heartbeat
     - StatePersistence: periodic IndexedDB snapshots for recovery
  4. Terminating the worker and cleaning up keep-alive on destroy.
  5. Rendering the Dashboard component as the primary user interface.

  Page structure:
  - <svelte:head> sets the browser tab title for SEO / accessibility.
  - <main> serves as a centered content container for the Dashboard.
-->
<script lang="ts">
  import Dashboard from '$components/Dashboard.svelte';
  import PWAInstall from '$components/PWAInstall.svelte';
  import { workerStore, workerState, addEvent } from '$stores/worker';
  import {
    isVisible,
    audioKeepAliveActive,
    workerHealthy,
    missedHeartbeats,
    signalWake
  } from '$stores/visibility';
  import {
    VisibilityManager,
    AudioKeepAlive,
    WorkerWatchdog,
    StatePersistence
  } from '$lib/background';
  import type {
    WorkerResponse,
    HeartbeatMessage,
    ProgressMessage,
    WinnerMessage,
    ErrorMessage,
    StartedMessage,
    FinishedMessage
  } from '$lib/worker/types';
  import { onMount, onDestroy } from 'svelte';

  /** Reference to the instantiated Web Worker; null before mount and after destroy. */
  let worker: Worker | null = null;

  /** Flag to trigger fade-in animation after mount */
  let loaded = false;

  /** Background keep-alive managers */
  let visibilityManager: VisibilityManager | null = null;
  let audioKeepAlive: AudioKeepAlive | null = null;
  let watchdog: WorkerWatchdog | null = null;
  let persistence: StatePersistence | null = null;

  /**
   * Lifecycle: runs after the component is first rendered in the DOM.
   * Creates a new Web Worker using Vite's `new Worker(new URL(...))` syntax
   * for module-based workers, then publishes it to the shared store so that
   * child components can send/receive messages.
   *
   * Also initializes the background keep-alive system.
   */
  onMount(() => {
    worker = new Worker(new URL('$lib/worker/index.ts', import.meta.url), {
      type: 'module'
    });

    // Set up Worker message routing (including heartbeat interception)
    worker.onmessage = handleWorkerMessage;

    // Expose the worker instance to the rest of the app via the store
    workerStore.set(worker);

    // Initialize background keep-alive system
    initBackgroundSystem();

    // Trigger fade-in animation
    setTimeout(() => {
      loaded = true;
    }, 100);
  });

  /**
   * Lifecycle: runs when the component is destroyed (e.g., on navigation).
   * Terminates the Web Worker and cleans up all keep-alive resources.
   */
  onDestroy(() => {
    destroyBackgroundSystem();

    if (worker) {
      worker.terminate();
    }
  });

  /**
   * Handle messages from the Web Worker.
   * Routes heartbeat messages to the watchdog and updates stores.
   */
  function handleWorkerMessage(event: MessageEvent<WorkerResponse>) {
    const data = event.data;

    switch (data.type) {
      case 'heartbeat': {
        const hb = data as HeartbeatMessage;
        watchdog?.recordHeartbeat(hb.timestamp, hb.status);
        // If heartbeat drift is very large, system likely slept
        if (hb.drift && hb.drift > 5000) {
          onSystemWake(hb.drift);
        }
        break;
      }

      case 'progress': {
        const msg = data as ProgressMessage;
        workerState.update((s) => ({
          ...s,
          currentStep: msg.step,
          speed: msg.speed
        }));
        // Update persistence with latest state
        persistence?.updateState({
          stepCount: msg.step,
          speed: msg.speed,
          wasRunning: true,
          wasPaused: false
        });
        break;
      }

      case 'started': {
        const msg = data as StartedMessage;
        workerState.update((s) => ({
          ...s,
          isRunning: true,
          isPaused: false,
          publicKey: msg.publicKey
        }));
        addEvent({ type: 'info', message: 'VDF computation started' });
        break;
      }

      case 'winner': {
        const msg = data as WinnerMessage;
        workerState.update((s) => ({
          ...s,
          winnerCount: s.winnerCount + 1
        }));
        addEvent({ type: 'winner', message: `Winner found at step ${msg.step}` });
        break;
      }

      case 'error': {
        const msg = data as ErrorMessage;
        addEvent({ type: 'error', message: `[${msg.code}] ${msg.message}` });
        break;
      }

      case 'finished': {
        const msg = data as FinishedMessage;
        workerState.update((s) => ({
          ...s,
          isRunning: false,
          currentStep: msg.step
        }));
        addEvent({ type: 'info', message: `VDF computation finished at step ${msg.step}` });
        break;
      }

      case 'stopped': {
        workerState.update((s) => ({
          ...s,
          isRunning: false,
          isPaused: false
        }));
        break;
      }
    }
  }

  /**
   * Initialize all background keep-alive managers.
   *
   * Strategy layering (progressive enhancement):
   * 1. VisibilityManager - detects tab state (always active)
   * 2. AudioKeepAlive - prevents timer throttling (best-effort)
   * 3. WorkerWatchdog - monitors worker health (always active when running)
   * 4. StatePersistence - saves state for recovery (best-effort)
   */
  async function initBackgroundSystem() {
    // 1. Visibility Manager
    visibilityManager = new VisibilityManager({
      onVisibilityChange: ({ state }) => {
        isVisible.set(state === 'visible');

        // Tell the Worker to adjust its heartbeat interval
        if (worker) {
          worker.postMessage({
            type: 'setHeartbeatMode',
            visible: state === 'visible'
          });
        }

        // Save state snapshot when tab becomes hidden
        if (state === 'hidden' && persistence) {
          persistence.updateState(getCurrentSnapshot());
          persistence.saveSnapshot(getCurrentSnapshot());
        }
      },
      onWake: ({ duration, isSystemSleep }) => {
        signalWake();

        if (isSystemSleep && duration > 10000) {
          onSystemWake(duration);
        }
      }
    });

    // 2. AudioContext Keep-Alive
    audioKeepAlive = new AudioKeepAlive({ autoResume: true });
    const audioStarted = audioKeepAlive.start();
    audioKeepAliveActive.set(audioStarted);

    // 3. Worker Watchdog
    watchdog = new WorkerWatchdog({
      heartbeatTimeout: 30000,
      hiddenTimeout: 60000,
      maxMissedBeats: 3,
      onUnresponsive: () => {
        workerHealthy.set(false);
        missedHeartbeats.update((n) => n + 1);
      },
      onRestart: () => {
        console.warn('[Watchdog] Worker unresponsive, triggering recovery');
        handleWorkerRecovery();
      },
      onHeartbeat: () => {
        workerHealthy.set(true);
        missedHeartbeats.set(0);
      }
    });
    watchdog.start();

    // 4. State Persistence
    persistence = new StatePersistence();
    await persistence.init();
    persistence.startPeriodicSnapshots(30000);

    // Check for recovery snapshot on page load
    const snapshot = await persistence.restoreSnapshot();
    if (snapshot?.wasRunning && !snapshot.wasPaused) {
      // Found recovery snapshot at step ${snapshot.stepCount}
    }
  }

  /**
   * Handle system wake event.
   * Verifies worker health and triggers recovery if needed.
   */
  function onSystemWake(_duration: number) {
    // System wake detected after ${Math.round(_duration / 1000)}s

    // Immediately check worker health
    watchdog?.checkNow();

    // Update persistence with current state
    persistence?.updateState(getCurrentSnapshot());
  }

  /**
   * Handle worker recovery when watchdog detects unresponsiveness.
   */
  function handleWorkerRecovery() {
    if (persistence) {
      persistence.updateState(getCurrentSnapshot());
      persistence.saveSnapshot(getCurrentSnapshot());
    }
  }

  /**
   * Get current computation state for persistence.
   */
  function getCurrentSnapshot() {
    const defaultSnapshot = {
      timestamp: Date.now(),
      stepCount: 0,
      totalSteps: 0,
      speed: 0,
      uptime: 0,
      winnerCount: 0,
      wasRunning: false,
      wasPaused: false
    };

    let snapshot = defaultSnapshot;
    const unsub = workerState.subscribe((s) => {
      snapshot = {
        timestamp: Date.now(),
        stepCount: s.currentStep,
        totalSteps: s.totalSteps,
        speed: s.speed,
        uptime: s.uptime,
        winnerCount: s.winnerCount,
        wasRunning: s.isRunning,
        wasPaused: s.isPaused
      };
    });
    unsub();
    return snapshot;
  }

  /**
   * Clean up all background keep-alive resources.
   */
  function destroyBackgroundSystem() {
    visibilityManager?.destroy();
    visibilityManager = null;

    audioKeepAlive?.stop();
    audioKeepAlive = null;
    audioKeepAliveActive.set(false);

    watchdog?.destroy();
    watchdog = null;

    persistence?.destroy();
    persistence = null;
  }
</script>

<!-- Set the page title displayed in the browser tab -->
<svelte:head>
  <title>VTP Node - Verifiable Time Proof</title>
</svelte:head>

<!-- Primary content container: centered layout with max-width constraint -->
<main class:loaded>
  <!-- Floating PWA install button -->
  <div class="pwa-container">
    <PWAInstall />
  </div>

  <!-- Dashboard renders all UI controls, progress display, and event log -->
  <Dashboard />
</main>

<style>
  /* Center the main content and add responsive padding */
  main {
    position: relative;
    z-index: 1;
    max-width: 1400px;
    margin: 0 auto;
    padding: 2rem;
    opacity: 0;
    transform: translateY(20px);
    transition: all 0.8s cubic-bezier(0.16, 1, 0.3, 1);
  }

  main.loaded {
    opacity: 1;
    transform: translateY(0);
  }

  .pwa-container {
    position: fixed;
    bottom: 2rem;
    right: 2rem;
    z-index: 1000;
  }

  @media (max-width: 768px) {
    main {
      padding: 1rem;
    }
  }
</style>
