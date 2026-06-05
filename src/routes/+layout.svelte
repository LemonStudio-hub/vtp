<!--
  @component +layout (Root Layout)

  The root layout component that wraps all page content in the SvelteKit app.

  This component serves two purposes:
  1. **Client-side rendering guard**: Uses a `mounted` flag to ensure child
     content (via <slot />) is only rendered after the component has mounted
     on the client. This prevents server-side rendering (SSR) of components
     that rely on browser-only APIs such as Web Workers.
  2. **Global styles**: Applies a dark-themed base style to the document body,
     including font stack, background color, and text color.

  The `{#if mounted}` conditional acts as a hydration gate — the page content
  is not rendered during SSR, only after the client-side `onMount` fires.
-->
<script lang="ts">
  import { onMount } from 'svelte';

  /**
   * Flag indicating whether the component has mounted on the client.
   * Starts as `false` (during SSR) and is set to `true` in `onMount`,
   * which only runs in the browser.
   */
  let mounted = false;

  /**
   * Lifecycle: runs after the component is mounted in the DOM (client only).
   * Sets `mounted = true` to unconditionally render child content via <slot />,
   * ensuring browser-only APIs (e.g., Web Workers) are available.
   */
  onMount(() => {
    mounted = true;
  });
</script>

<!--
  Hydration gate: child page content is only rendered after client-side mount.
  This prevents SSR errors for components that depend on browser APIs.
-->
{#if mounted}
  <div class="app-container">
    <div class="bg-grid"></div>
    <div class="bg-glow"></div>
    <slot />
  </div>
{/if}

<style>
  /*
   * Global base styles applied to the <body> element.
   * Uses :global() to escape Svelte's scoped CSS and affect the entire page.
   * Sets a dark background (#0a0a1a) with light text (#e6e6e6) and a
   * system-ui font stack for consistent cross-platform typography.
   */
  :global(body) {
    margin: 0;
    padding: 0;
    font-family:
      -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans',
      'Helvetica Neue', sans-serif;
    background-color: #0a0a1a;
    color: #e6e6e6;
    overflow-x: hidden;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(::-webkit-scrollbar) {
    width: 6px;
  }

  :global(::-webkit-scrollbar-track) {
    background: rgba(255, 255, 255, 0.05);
  }

  :global(::-webkit-scrollbar-thumb) {
    background: rgba(0, 255, 136, 0.3);
    border-radius: 3px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: rgba(0, 255, 136, 0.5);
  }

  .app-container {
    position: relative;
    min-height: 100vh;
    overflow: hidden;
  }

  .bg-grid {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-image:
      linear-gradient(rgba(0, 255, 136, 0.03) 1px, transparent 1px),
      linear-gradient(90deg, rgba(0, 255, 136, 0.03) 1px, transparent 1px);
    background-size: 50px 50px;
    pointer-events: none;
    z-index: 0;
  }

  .bg-glow {
    position: fixed;
    top: -50%;
    left: -50%;
    width: 200%;
    height: 200%;
    background:
      radial-gradient(circle at 30% 20%, rgba(0, 255, 136, 0.08) 0%, transparent 40%),
      radial-gradient(circle at 70% 80%, rgba(99, 102, 241, 0.06) 0%, transparent 40%),
      radial-gradient(circle at 50% 50%, rgba(0, 255, 136, 0.04) 0%, transparent 60%);
    animation: bgPulse 15s ease-in-out infinite alternate;
    pointer-events: none;
    z-index: 0;
  }

  @keyframes bgPulse {
    0% {
      transform: translate(0, 0) scale(1);
      opacity: 0.8;
    }
    50% {
      transform: translate(-5%, 5%) scale(1.1);
      opacity: 1;
    }
    100% {
      transform: translate(5%, -5%) scale(1);
      opacity: 0.8;
    }
  }
</style>
