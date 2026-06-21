<!--
  SpeedChart Component

  A pure Canvas-based sparkline showing computation speed over the last
  60 data points. No external charting library required.

  Features:
  - Responsive via ResizeObserver with devicePixelRatio scaling
  - Green gradient fill area with line stroke
  - Glowing endpoint dot at the latest data point
  - Auto-scaling Y-axis with 10% headroom
  - Empty state when no data is available
  - Throttled redraws via animationBudget store
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { workerState } from '$stores/worker';
  import { animationBudget } from '$stores/visibility';
  import { setupCanvas } from '$lib/canvas';
  import type { CanvasSetupHandle } from '$lib/canvas';

  /** Reference to the canvas element */
  let canvas: HTMLCanvasElement;

  /** Container element for ResizeObserver */
  let container: HTMLDivElement;

  /** Canvas setup handle */
  let canvasHandle: CanvasSetupHandle | null = null;

  /** Cached context reference */
  let ctx: CanvasRenderingContext2D | null = null;

  /** Current animation budget from visibility store */
  let currentBudget = 16;

  /** Last redraw timestamp */
  let lastDrawTime = 0;

  $: currentBudget = $animationBudget;

  /** Redraw chart when speedHistory changes */
  $: if (ctx && $workerState.speedHistory.length >= 0) {
    scheduleDraw();
  }

  onMount(() => {
    if (!canvas || !container) return;

    canvasHandle = setupCanvas({
      canvas,
      container,
      onResize: draw
    });
    ctx = canvasHandle.ctx;
  });

  onDestroy(() => {
    canvasHandle?.destroy();
  });

  function scheduleDraw() {
    const now = performance.now();
    if (now - lastDrawTime < currentBudget) return;
    lastDrawTime = now;
    draw();
  }

  function draw() {
    if (!ctx || !canvas) return;
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.width / dpr;
    const h = canvas.height / dpr;
    const data = $workerState.speedHistory;

    ctx.clearRect(0, 0, w, h);

    // Padding
    const pad = { top: 8, right: 16, bottom: 24, left: 16 };
    const chartW = w - pad.left - pad.right;
    const chartH = h - pad.top - pad.bottom;

    if (data.length < 2) {
      // Empty state
      ctx.fillStyle = 'rgba(255, 255, 255, 0.15)';
      ctx.font = '12px var(--font-mono), monospace';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText('Waiting for data…', w / 2, h / 2);
      return;
    }

    const maxVal = Math.max(...data) * 1.1 || 1;
    const points: [number, number][] = data.map((v, i) => [
      pad.left + (i / (data.length - 1)) * chartW,
      pad.top + chartH - (v / maxVal) * chartH
    ]);

    // Gradient fill
    const gradient = ctx.createLinearGradient(0, pad.top, 0, pad.top + chartH);
    gradient.addColorStop(0, 'rgba(0, 255, 136, 0.25)');
    gradient.addColorStop(1, 'rgba(0, 255, 136, 0.02)');

    ctx.beginPath();
    ctx.moveTo(points[0][0], pad.top + chartH);
    for (const [x, y] of points) {
      ctx.lineTo(x, y);
    }
    ctx.lineTo(points[points.length - 1][0], pad.top + chartH);
    ctx.closePath();
    ctx.fillStyle = gradient;
    ctx.fill();

    // Line stroke
    ctx.beginPath();
    ctx.moveTo(points[0][0], points[0][1]);
    for (let i = 1; i < points.length; i++) {
      ctx.lineTo(points[i][0], points[i][1]);
    }
    ctx.strokeStyle = '#00ff88';
    ctx.lineWidth = 2;
    ctx.lineJoin = 'round';
    ctx.stroke();

    // Endpoint dot with glow
    const [ex, ey] = points[points.length - 1];
    ctx.beginPath();
    ctx.arc(ex, ey, 6, 0, Math.PI * 2);
    ctx.fillStyle = 'rgba(0, 255, 136, 0.3)';
    ctx.fill();

    ctx.beginPath();
    ctx.arc(ex, ey, 3, 0, Math.PI * 2);
    ctx.fillStyle = '#00ff88';
    ctx.fill();

    // Y-axis label (max)
    ctx.fillStyle = 'rgba(255, 255, 255, 0.2)';
    ctx.font = '10px var(--font-mono), monospace';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'top';
    const maxLabel =
      maxVal >= 1000000
        ? `${(maxVal / 1000000).toFixed(1)}M`
        : maxVal >= 1000
          ? `${(maxVal / 1000).toFixed(0)}K`
          : maxVal.toFixed(0);
    ctx.fillText(maxLabel, w - 4, pad.top + 2);
  }
</script>

<div class="speed-chart" bind:this={container}>
  <div class="chart-header">
    <span class="chart-label">Speed History</span>
    <span class="chart-value">{$workerState.speedHistory.length} samples</span>
  </div>
  <div class="chart-body">
    <canvas bind:this={canvas}></canvas>
  </div>
</div>

<style>
  .speed-chart {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    height: 100%;
  }

  .chart-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .chart-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .chart-value {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .chart-body {
    flex: 1;
    min-height: 140px;
    position: relative;
  }

  .chart-body canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
</style>
