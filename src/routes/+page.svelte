<script lang="ts">
  import Dashboard from '$components/Dashboard.svelte';
  import { workerStore } from '$stores/worker';
  import { onMount, onDestroy } from 'svelte';

  let worker: Worker | null = null;

  onMount(() => {
    worker = new Worker(new URL('$lib/worker/index.ts', import.meta.url), {
      type: 'module'
    });

    workerStore.set(worker);
  });

  onDestroy(() => {
    if (worker) {
      worker.terminate();
    }
  });
</script>

<svelte:head>
  <title>VTP Node - Verifiable Time Proof</title>
</svelte:head>

<main>
  <Dashboard />
</main>

<style>
  main {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }
</style>
