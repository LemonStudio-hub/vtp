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
-->

<script lang="ts">
  import { workerState } from '$stores/worker';

  /** Reference to the Canvas element */
  let canvas: HTMLCanvasElement;

  /** Canvas 2D rendering context */
  let ctx: CanvasRenderingContext2D | null = null;

  /**
   * Reactive identity pattern update.
   * Draws the identity pattern when canvas and public key are available.
   */
  $: if (canvas && $workerState.publicKey) {
    drawIdentity();
  }

  /**
   * Draw the identity pattern.
   * Generates a unique Identicon from the public key hash.
   */
  function drawIdentity() {
    if (!canvas) return;
    ctx = canvas.getContext('2d');
    if (!ctx) return;

    const size = 64;
    canvas.width = size;
    canvas.height = size;

    const hash = simpleHash($workerState.publicKey);
    const hue = hash % 360;

    // Background gradient
    const gradient = ctx.createLinearGradient(0, 0, size, size);
    gradient.addColorStop(0, `hsl(${hue}, 70%, 50%)`);
    gradient.addColorStop(1, `hsl(${(hue + 30) % 360}, 70%, 40%)`);
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, size, size);

    // Pattern blocks
    ctx.fillStyle = `hsla(${(hue + 180) % 360}, 70%, 50%, 0.8)`;
    for (let i = 0; i < 4; i++) {
      for (let j = 0; j < 4; j++) {
        if ((hash >> (i * 4 + j)) & 1) {
          ctx.fillRect(i * 16 + 1, j * 16 + 1, 14, 14);
        }
      }
    }

    // Depth overlay
    const overlay = ctx.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2);
    overlay.addColorStop(0, 'rgba(255, 255, 255, 0.1)');
    overlay.addColorStop(1, 'rgba(0, 0, 0, 0.1)');
    ctx.fillStyle = overlay;
    ctx.fillRect(0, 0, size, size);
  }

  /**
   * Simple DJB2 hash function for generating the identicon seed.
   */
  function simpleHash(data: Uint8Array | null): number {
    if (!data) return 0;
    let hash = 0;
    for (let i = 0; i < Math.min(data.length, 16); i++) {
      hash = ((hash << 5) - hash + data[i]) | 0;
    }
    return Math.abs(hash);
  }
</script>

<div class="identity-badge" class:running={$workerState.isRunning}>
  <div class="avatar-container">
    <canvas bind:this={canvas} class="avatar"></canvas>
    <div class="avatar-glow"></div>
  </div>

  <div class="node-info">
    <span class="node-id">Node {$workerState.nodeId || '---'}</span>
    <div class="status-container">
      <span class="status-dot" class:active={$workerState.isRunning}></span>
      <span class="status" class:running={$workerState.isRunning}>
        {$workerState.isRunning ? 'Running' : 'Idle'}
      </span>
    </div>
  </div>
</div>

<style>
  .identity-badge {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .avatar-container {
    position: relative;
    width: 56px;
    height: 56px;
  }

  .avatar {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    border: 2px solid var(--color-border);
    transition: all 0.3s ease;
  }

  .identity-badge.running .avatar {
    border-color: rgba(0, 255, 136, 0.3);
  }

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
    0%,
    100% {
      box-shadow: 0 0 20px rgba(0, 255, 136, 0.2);
    }
    50% {
      box-shadow: 0 0 30px rgba(0, 255, 136, 0.4);
    }
  }

  .node-info {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .node-id {
    font-family: var(--font-mono);
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--color-text-primary);
    letter-spacing: 0.02em;
  }

  .status-container {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-text-muted);
    transition: all 0.3s ease;
  }

  .status-dot.active {
    background: var(--color-accent-green);
    box-shadow: 0 0 8px var(--color-accent-green-glow);
  }

  .status {
    font-size: 0.8125rem;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
    letter-spacing: 0.02em;
  }

  .status.running {
    color: var(--color-accent-green);
  }
</style>
