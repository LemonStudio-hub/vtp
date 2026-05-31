<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';

  let mounted = false;

  onMount(() => {
    mounted = true;
  });
</script>

{#if mounted}
  <slot />
{/if}

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family:
      -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans',
      'Helvetica Neue', sans-serif;
    background-color: #1a1a2e;
    color: #e6e6e6;
  }
</style>
