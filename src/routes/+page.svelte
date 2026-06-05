<!--
  @component +page (Root Page)

  The main entry page of the VTP (Verifiable Time Proof) application.

  This component is responsible for:
  1. Instantiating the Web Worker that performs time-proof computations
     in a background thread to avoid blocking the UI.
  2. Registering the worker instance into the shared `workerStore` so that
     child components (e.g., Dashboard) can communicate with it.
  3. Terminating the worker on component destruction to free resources.
  4. Rendering the Dashboard component as the primary user interface.

  Page structure:
  - <svelte:head> sets the browser tab title for SEO / accessibility.
  - <main> serves as a centered content container for the Dashboard.
-->
<script lang="ts">
  import Dashboard from '$components/Dashboard.svelte';
  import PWAInstall from '$components/PWAInstall.svelte';
  import { workerStore } from '$stores/worker';
  import { onMount, onDestroy } from 'svelte';

  /** Reference to the instantiated Web Worker; null before mount and after destroy. */
  let worker: Worker | null = null;

  /** Flag to trigger fade-in animation after mount */
  let loaded = false;

  /**
   * Lifecycle: runs after the component is first rendered in the DOM.
   * Creates a new Web Worker using Vite's `new Worker(new URL(...))` syntax
   * for module-based workers, then publishes it to the shared store so that
   * child components can send/receive messages.
   */
  onMount(() => {
    worker = new Worker(new URL('$lib/worker/index.ts', import.meta.url), {
      type: 'module'
    });

    // Expose the worker instance to the rest of the app via the store
    workerStore.set(worker);

    // Trigger fade-in animation
    setTimeout(() => {
      loaded = true;
    }, 100);
  });

  /**
   * Lifecycle: runs when the component is destroyed (e.g., on navigation).
   * Terminates the Web Worker to release the background thread and
   * associated memory resources.
   */
  onDestroy(() => {
    if (worker) {
      worker.terminate();
    }
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
