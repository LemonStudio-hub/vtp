<!--
  ProgressHero Component

  An enhanced hero display combining a responsive progress ring with
  key computation metrics. Replaces the full VDFCanvas particle
  animation with a focused, data-rich progress indicator.

  Features:
  - Responsive Canvas progress ring (scales to container, max 300px)
  - Percentage, current step, and ETA display
  - Ambient glow and animated endpoint dot
  - Three key metrics row: Speed, Steps, ETA
  - Throttled redraws via animationBudget store
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { workerState, progress } from '$stores/worker';
  import { animationBudget } from '$stores/visibility';
  import { formatSpeed, formatNumber, formatTime, calculateETA } from '$utils';
  import { setupCanvas, createAnimationLoop } from '$lib/canvas';
  import type { CanvasSetupHandle, AnimationLoopHandle } from '$lib/canvas';

  /** Canvas element reference */
  let canvas: HTMLCanvasElement;

  /** Container element for ResizeObserver */
  let container: HTMLDivElement;

  /** Canvas setup handle (ctx, sizeCanvas, destroy) */
  let canvasHandle: CanvasSetupHandle | null = null;

  /** Animation loop handle */
  let loop: AnimationLoopHandle | null = null;

  /** Time counter for ambient animations */
  let time = 0;

  /** Cached context reference */
  let ctx: CanvasRenderingContext2D | null = null;

  /** ETA calculation */
  $: eta = calculateETA($workerState.currentStep, $workerState.totalSteps, $workerState.speed);

  /** Update animation budget from store */
  $: loop?.setBudget($animationBudget);

  /** Start animation when running */
  $: if (ctx && $workerState.isRunning) {
    loop?.start();
  }

  /** Stop animation when not running */
  $: if (!$workerState.isRunning && loop) {
    loop.stop();
    draw();
  }

  onMount(() => {
    if (!canvas || !container) return;

    canvasHandle = setupCanvas({
      canvas,
      container,
      onResize: draw
    });
    ctx = canvasHandle.ctx;

    loop = createAnimationLoop({
      budget: $animationBudget,
      onFrame(dt) {
        time += dt;
        draw();
      }
    });

    // Draw initial idle state
    draw();
  });

  onDestroy(() => {
    loop?.destroy();
    canvasHandle?.destroy();
  });

  function draw() {
    if (!ctx || !canvas) return;
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.width / dpr;
    const h = canvas.height / dpr;
    const cx = w / 2;
    const cy = h / 2;
    const radius = Math.min(w, h) * 0.38;
    const lineWidth = Math.max(6, radius * 0.08);

    ctx.clearRect(0, 0, w, h);

    const prog = $progress;
    const isRunning = $workerState.isRunning;

    // Ambient glow
    const glowR = radius + 30 + Math.sin(time * 2) * 8;
    const glow = ctx.createRadialGradient(cx, cy, 0, cx, cy, glowR);
    if (isRunning) {
      glow.addColorStop(0, 'rgba(0, 255, 136, 0.06)');
      glow.addColorStop(0.6, 'rgba(0, 255, 136, 0.02)');
      glow.addColorStop(1, 'transparent');
    } else {
      glow.addColorStop(0, 'rgba(100, 100, 100, 0.03)');
      glow.addColorStop(1, 'transparent');
    }
    ctx.fillStyle = glow;
    ctx.fillRect(0, 0, w, h);

    // Background ring
    ctx.beginPath();
    ctx.arc(cx, cy, radius, 0, Math.PI * 2);
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.06)';
    ctx.lineWidth = lineWidth;
    ctx.stroke();

    // Tick marks
    for (let i = 0; i < 60; i++) {
      const angle = (i / 60) * Math.PI * 2 - Math.PI / 2;
      const innerR = i % 5 === 0 ? radius - lineWidth - 6 : radius - lineWidth - 3;
      const outerR = radius - lineWidth;
      ctx.beginPath();
      ctx.moveTo(cx + Math.cos(angle) * innerR, cy + Math.sin(angle) * innerR);
      ctx.lineTo(cx + Math.cos(angle) * outerR, cy + Math.sin(angle) * outerR);
      ctx.strokeStyle = i % 5 === 0 ? 'rgba(255, 255, 255, 0.1)' : 'rgba(255, 255, 255, 0.03)';
      ctx.lineWidth = i % 5 === 0 ? 1.5 : 0.5;
      ctx.stroke();
    }

    // Progress arc
    if (prog > 0) {
      const startAngle = -Math.PI / 2;
      const endAngle = startAngle + Math.PI * 2 * prog;

      // Glow behind arc
      ctx.save();
      ctx.shadowBlur = 15;
      ctx.shadowColor = 'rgba(0, 255, 136, 0.4)';
      ctx.beginPath();
      ctx.arc(cx, cy, radius, startAngle, endAngle);
      ctx.strokeStyle = 'rgba(0, 255, 136, 0.15)';
      ctx.lineWidth = lineWidth + 6;
      ctx.lineCap = 'round';
      ctx.stroke();
      ctx.restore();

      // Gradient arc
      const gradient = ctx.createConicGradient(startAngle, cx, cy);
      gradient.addColorStop(0, '#00ff88');
      gradient.addColorStop(prog, '#00cc6a');
      gradient.addColorStop(1, 'transparent');

      ctx.beginPath();
      ctx.arc(cx, cy, radius, startAngle, endAngle);
      ctx.strokeStyle = gradient;
      ctx.lineWidth = lineWidth;
      ctx.lineCap = 'round';
      ctx.stroke();

      // Endpoint dot
      const ex = cx + Math.cos(endAngle) * radius;
      const ey = cy + Math.sin(endAngle) * radius;

      ctx.beginPath();
      ctx.arc(ex, ey, 5, 0, Math.PI * 2);
      ctx.fillStyle = 'rgba(0, 255, 136, 0.3)';
      ctx.fill();

      ctx.beginPath();
      ctx.arc(ex, ey, 3, 0, Math.PI * 2);
      ctx.fillStyle = '#00ff88';
      ctx.fill();
    }

    // Center text: percentage
    const pctText = `${(prog * 100).toFixed(1)}%`;
    ctx.font = `bold ${Math.max(16, radius * 0.28)}px var(--font-mono), monospace`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillStyle = isRunning ? '#00ff88' : '#555';
    ctx.fillText(pctText, cx, cy - radius * 0.08);

    // Center text: step count
    ctx.font = `${Math.max(10, radius * 0.11)}px var(--font-mono), monospace`;
    ctx.fillStyle = 'rgba(255, 255, 255, 0.3)';
    const stepText = `${formatNumber($workerState.currentStep)} / ${formatNumber($workerState.totalSteps)}`;
    ctx.fillText(stepText, cx, cy + radius * 0.18);
  }
</script>

<div class="progress-hero">
  <div class="ring-container" bind:this={container}>
    <canvas bind:this={canvas}></canvas>
  </div>

  <div class="metrics-row">
    <div class="metric">
      <span class="metric-icon">⚡</span>
      <div class="metric-content">
        <span class="metric-value">{formatSpeed($workerState.speed)}</span>
        <span class="metric-label">steps/s</span>
      </div>
    </div>

    <div class="metric">
      <span class="metric-icon">📊</span>
      <div class="metric-content">
        <span class="metric-value">{formatNumber($workerState.currentStep)}</span>
        <span class="metric-label">steps</span>
      </div>
    </div>

    <div class="metric">
      <span class="metric-icon">⏱</span>
      <div class="metric-content">
        <span class="metric-value">{eta >= 0 ? formatTime(eta) : '--:--:--'}</span>
        <span class="metric-label">remaining</span>
      </div>
    </div>
  </div>
</div>

<style>
  .progress-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-lg);
    width: 100%;
  }

  .ring-container {
    width: 100%;
    max-width: 300px;
    aspect-ratio: 1;
    position: relative;
  }

  .ring-container canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

  .metrics-row {
    display: flex;
    gap: var(--space-xl);
    flex-wrap: wrap;
    justify-content: center;
  }

  .metric {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    transition: all 0.2s ease;
  }

  .metric:hover {
    background: var(--color-surface-hover);
    border-color: var(--color-border-accent);
  }

  .metric-icon {
    font-size: 1rem;
  }

  .metric-content {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .metric-value {
    font-family: var(--font-mono);
    font-size: 1rem;
    font-weight: 700;
    color: var(--color-accent-green);
    line-height: 1.1;
  }

  .metric-label {
    font-size: 0.625rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  @media (max-width: 480px) {
    .metrics-row {
      gap: var(--space-sm);
    }

    .metric {
      padding: 0.375rem 0.75rem;
    }

    .metric-value {
      font-size: 0.875rem;
    }
  }
</style>
