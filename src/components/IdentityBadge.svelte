<!--
  IdentityBadge Component

  Displays the node identity badge, including:
  - A unique pattern generated from the public key
  - Node ID
  - Running status

  Features:
  - Generates a unique Identicon pattern based on the public key
  - Real-time display of the node running status
  - Reactive updates
  - Animated glow effect when running

  Algorithm description:
  1. Compute a simple hash of the public key
  2. Derive an HSB colour from the hash value
  3. Render a 4x4 grid pattern based on the hash bits

  Usage example:
  ```svelte
  <IdentityBadge />
  ```
-->

<script lang="ts">
  import { workerState } from '$stores/worker';

  /** Reference to the Canvas element */
  let canvas: HTMLCanvasElement;

  /** Canvas 2D rendering context */
  let ctx: CanvasRenderingContext2D | null = null;

  /**
   * Reactive identity pattern update
   *
   * Automatically draws the identity pattern when both the canvas
   * element and the public key are available.
   * Uses Svelte's reactive declaration syntax.
   */
  $: if (canvas && $workerState.publicKey) {
    drawIdentity();
  }

  /**
   * Draw the identity pattern
   *
   * Generates a unique Identicon pattern based on the public key.
   *
   * Algorithm steps:
   * 1. Compute a simple hash of the public key to obtain a numeric value
   * 2. Use hash % 360 as the base hue
   * 3. Fill the background with the base colour
   * 4. For each hash bit, decide whether to draw a complementary-colour block
   * 5. Produce a 4x4 symmetric pattern
   */
  function drawIdentity() {
    if (!canvas) return;

    ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas dimensions
    const size = 64;
    canvas.width = size;
    canvas.height = size;

    // Compute hash value and base hue
    const hash = simpleHash($workerState.publicKey);
    const hue = hash % 360;

    // Draw background with gradient
    const gradient = ctx.createLinearGradient(0, 0, size, size);
    gradient.addColorStop(0, `hsl(${hue}, 70%, 50%)`);
    gradient.addColorStop(1, `hsl(${(hue + 30) % 360}, 70%, 40%)`);
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, size, size);

    // Set complementary colour with transparency for depth
    ctx.fillStyle = `hsla(${(hue + 180) % 360}, 70%, 50%, 0.8)`;

    // Draw the 4x4 grid pattern
    // Each hash bit determines whether to draw a block
    for (let i = 0; i < 4; i++) {
      for (let j = 0; j < 4; j++) {
        // Check the corresponding bit of the hash
        if ((hash >> (i * 4 + j)) & 1) {
          const x = i * 16;
          const y = j * 16;

          // Draw block with slight rounded corners effect
          ctx.fillRect(x + 1, y + 1, 14, 14);
        }
      }
    }

    // Add a subtle overlay for depth
    const overlayGradient = ctx.createRadialGradient(
      size / 2, size / 2, 0,
      size / 2, size / 2, size / 2
    );
    overlayGradient.addColorStop(0, 'rgba(255, 255, 255, 0.1)');
    overlayGradient.addColorStop(1, 'rgba(0, 0, 0, 0.1)');
    ctx.fillStyle = overlayGradient;
    ctx.fillRect(0, 0, size, size);
  }

  /**
   * Simple hash function
   *
   * Computes a simple hash of a byte array and returns a numeric value.
   * Used as a seed for generating the identity pattern.
   *
   * @param data - Byte array to hash (may be null)
   * @returns Hash value (non-negative integer)
   *
   * Algorithm description:
   * - Uses a variant of the classic DJB2 hash algorithm
   * - Only processes the first 16 bytes to avoid excessively long inputs
   * - Uses bitwise operations to ensure a 32-bit integer result
   * - Returns the absolute value to guarantee non-negativity
   */
  function simpleHash(data: Uint8Array | null): number {
    if (!data) return 0;

    let hash = 0;
    // Only process the first 16 bytes
    for (let i = 0; i < Math.min(data.length, 16); i++) {
      // DJB2 hash algorithm: hash = hash * 33 + byte
      hash = ((hash << 5) - hash + data[i]) | 0;
    }
    return Math.abs(hash);
  }
</script>

<!-- Identity badge container -->
<div class="identity-badge" class:running={$workerState.isRunning}>
  <!-- Avatar container with glow effect -->
  <div class="avatar-container">
    <canvas bind:this={canvas} class="avatar"></canvas>
    <div class="avatar-glow"></div>
  </div>

  <!-- Node information -->
  <div class="node-info">
    <!-- Node ID -->
    <span class="node-id">Node {$workerState.nodeId || '---'}</span>

    <!-- Running status -->
    <div class="status-container">
      <span class="status-dot" class:active={$workerState.isRunning}></span>
      <span class="status" class:running={$workerState.isRunning}>
        {$workerState.isRunning ? 'Running' : 'Idle'}
      </span>
    </div>
  </div>
</div>

<style>
  /* Identity badge container */
  .identity-badge {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  /* Avatar container */
  .avatar-container {
    position: relative;
    width: 56px;
    height: 56px;
  }

  /* Identity pattern styles */
  .avatar {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    border: 2px solid rgba(255, 255, 255, 0.1);
    transition: all 0.3s ease;
  }

  .identity-badge.running .avatar {
    border-color: rgba(0, 255, 136, 0.3);
  }

  /* Avatar glow effect */
  .avatar-glow {
    position: absolute;
    top: -4px;
    left: -4px;
    right: -4px;
    bottom: -4px;
    border-radius: 50%;
    background: transparent;
    transition: all 0.3s ease;
    pointer-events: none;
  }

  .identity-badge.running .avatar-glow {
    box-shadow: 0 0 20px rgba(0, 255, 136, 0.3);
    animation: glowPulse 2s ease-in-out infinite;
  }

  @keyframes glowPulse {
    0%, 100% {
      box-shadow: 0 0 20px rgba(0, 255, 136, 0.2);
    }
    50% {
      box-shadow: 0 0 30px rgba(0, 255, 136, 0.4);
    }
  }

  /* Node information container */
  .node-info {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  /* Node ID styles */
  .node-id {
    font-family: 'Courier New', monospace;
    font-size: 0.9375rem;
    font-weight: 600;
    color: #e6e6e6;
    letter-spacing: 0.02em;
  }

  /* Status container */
  .status-container {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  /* Status dot */
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #555;
    transition: all 0.3s ease;
  }

  .status-dot.active {
    background: #00ff88;
    box-shadow: 0 0 8px rgba(0, 255, 136, 0.5);
  }

  /* Status styles */
  .status {
    font-size: 0.8125rem;
    color: #666;
    font-family: 'Courier New', monospace;
    letter-spacing: 0.02em;
  }

  /* Running status styles */
  .status.running {
    color: #00ff88;
  }
</style>
