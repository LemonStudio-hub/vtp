<!--
  PWAInstall Component

  Provides PWA installation functionality, including:
  - Listening for browser install prompt events
  - Displaying a custom install button
  - Handling the user's installation choice

  Features:
  - Automatic detection of PWA install criteria
  - Custom install prompt UI
  - Handling of installation success/failure events
  - Reactive show/hide behaviour

  Browser compatibility:
  - Chrome 67+
  - Edge 79+
  - Firefox not supported (must be manually added to home screen)

  Usage example:
  ```svelte
  <PWAInstall />
  ```
-->

<script lang="ts">
  import { onMount } from 'svelte';

  /**
   * Deferred install prompt event
   *
   * The browser fires the beforeinstallprompt event when the PWA
   * meets the installation criteria. We prevent the default behaviour
   * and store the event reference for later invocation.
   */
  let deferredPrompt: any = null;

  /**
   * Whether to show the install button
   *
   * The install button is shown when deferredPrompt is available
   */
  let showInstallButton = false;

  /**
   * Register event listeners when the component mounts
   *
   * Listens for two key events:
   * 1. beforeinstallprompt: the browser is ready to show the install prompt
   * 2. appinstalled: the application was installed successfully
   */
  onMount(() => {
    /**
     * Listen for the beforeinstallprompt event
     *
     * Fired when the browser detects that the PWA meets the install criteria.
     * We suppress the default install prompt and show a custom button instead.
     */
    window.addEventListener('beforeinstallprompt', (e) => {
      // Suppress the default install prompt
      e.preventDefault();

      // Store the event reference for later use
      deferredPrompt = e;

      // Show the custom install button
      showInstallButton = true;
    });

    /**
     * Listen for the appinstalled event
     *
     * Fired when the PWA has been installed successfully.
     * Resets the state and hides the install button.
     */
    window.addEventListener('appinstalled', () => {
      // Hide the install button
      showInstallButton = false;

      // Clean up the event reference
      deferredPrompt = null;
    });
  });

  /**
   * Handle the install button click
   *
   * Triggers the browser's install prompt so the user can choose
   * whether to install the application.
   *
   * Workflow:
   * 1. Check whether an install prompt is available
   * 2. Show the browser's installation dialog
   * 3. Update the state based on the user's choice
   */
  async function handleInstall() {
    if (!deferredPrompt) return;

    // Show the browser's installation dialog
    deferredPrompt.prompt();

    // Wait for the user's choice
    const { outcome } = await deferredPrompt.userChoice;

    // Update state based on the user's choice
    if (outcome === 'accepted') {
      showInstallButton = false;
    }

    // Clean up the event reference
    deferredPrompt = null;
  }
</script>

<!-- Conditionally render the install button -->
{#if showInstallButton}
  <button class="install-button" on:click={handleInstall}>
    <span class="install-icon">📲</span>
    <span class="install-text">Install App</span>
  </button>
{/if}

<style>
  /* Install button styles */
  .install-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.875rem 1.5rem;
    background: linear-gradient(135deg, #00ff88 0%, #00cc6a 100%);
    color: #0a0a1a;
    border: none;
    border-radius: 12px;
    font-size: 0.9375rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    box-shadow: 0 4px 15px rgba(0, 255, 136, 0.3);
    animation: slideUp 0.5s ease forwards;
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* Button hover effect */
  .install-button:hover {
    transform: translateY(-3px);
    box-shadow: 0 8px 25px rgba(0, 255, 136, 0.4);
  }

  /* Button active/press effect */
  .install-button:active {
    transform: translateY(-1px);
    box-shadow: 0 4px 15px rgba(0, 255, 136, 0.3);
  }

  .install-icon {
    font-size: 1.125rem;
  }

  .install-text {
    letter-spacing: 0.02em;
  }
</style>
