<!--
  @component +page (Root Page)

  The main entry page of the VTP (Verifiable Time Proof) application.

  This component is a thin composition root that wires together three
  lifecycle subsystems and renders the Dashboard:
  1. Worker Lifecycle - Web Worker management and message routing
  2. Background Lifecycle - Keep-alive, watchdog, and persistence
  3. Network Lifecycle - P2P networking via WebRTC

  All business logic lives in the lifecycle modules under $lib/lifecycle/.
-->
<script lang="ts">
  import Dashboard from '$components/Dashboard.svelte';
  import PWAInstall from '$components/PWAInstall.svelte';
  import {
    createWorkerLifecycle,
    createBackgroundSystem,
    createNetworkLifecycle
  } from '$lib/lifecycle';
  import { getCryptoProvider } from '$lib/wasm-loader';
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { workerState } from '$stores/worker';

  /** Flag to trigger fade-in animation after mount */
  let loaded = false;

  /** Cleanup function composed from all lifecycle handles */
  let destroy: (() => void) | null = null;

  /**
   * Lifecycle: runs after the component is first rendered in the DOM.
   * Composes the three lifecycle subsystems and wires them together.
   */
  onMount(async () => {
    // Create a temporary worker first (needed by background system)
    const worker = new Worker(new URL('$lib/worker/index.ts', import.meta.url), {
      type: 'module'
    });

    // Helper to get current state snapshot for persistence
    function getCurrentSnapshot() {
      const s = get(workerState);
      return {
        timestamp: Date.now(),
        stepCount: s.currentStep,
        totalSteps: s.totalSteps,
        speed: s.speed,
        uptime: s.uptime,
        winnerCount: s.winnerCount,
        wasRunning: s.isRunning,
        wasPaused: s.isPaused
      };
    }

    // System wake handler (shared between background and worker lifecycles)
    function onSystemWake(_duration: number) {
      bg?.watchdog?.checkNow();
      bg?.persistence?.updateState(getCurrentSnapshot());
    }

    // Initialize background system first (provides watchdog + persistence)
    const bg = await createBackgroundSystem({ worker, onSystemWake });

    // Initialize worker lifecycle (routes messages to stores + watchdog)
    const workerLife = createWorkerLifecycle({
      watchdog: bg.watchdog,
      persistence: bg.persistence,
      onSystemWake
    });

    // Load WASM crypto provider for signing/verification
    const crypto = await getCryptoProvider();

    // Initialize network layer
    const network = await createNetworkLifecycle({ crypto });

    // Compose cleanup
    destroy = () => {
      network.destroy();
      bg.destroy();
      workerLife.destroy();
    };

    // Trigger fade-in animation
    setTimeout(() => {
      loaded = true;
    }, 100);
  });

  /**
   * Lifecycle: runs when the component is destroyed.
   * Cleans up all subsystems in reverse order.
   */
  onDestroy(() => {
    destroy?.();
  });
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
