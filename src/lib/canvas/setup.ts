/**
 * Canvas Setup Utility
 *
 * Shared canvas initialization boilerplate extracted from ProgressHero,
 * SpeedChart, and other canvas-based components.
 *
 * Handles:
 * - 2D context acquisition
 * - ResizeObserver with debounce for responsive sizing
 * - DevicePixelRatio-aware canvas scaling
 * - Cleanup on destroy
 */

export interface CanvasSetupOptions {
  /** The canvas element to set up */
  canvas: HTMLCanvasElement;
  /** The container element to observe for resize */
  container: HTMLElement;
  /** Called after canvas is resized (use this to redraw) */
  onResize?: () => void;
}

export interface CanvasSetupHandle {
  /** The 2D rendering context */
  ctx: CanvasRenderingContext2D;
  /** Recalculate canvas size to match container (also calls onResize) */
  sizeCanvas: () => void;
  /** Clean up ResizeObserver and timers */
  destroy: () => void;
}

/**
 * Set up a canvas element with responsive sizing and DPR awareness.
 *
 * @returns A handle with the context, sizeCanvas function, and destroy cleanup.
 * @throws If the canvas 2D context cannot be obtained.
 */
export function setupCanvas(options: CanvasSetupOptions): CanvasSetupHandle {
  const { canvas, container, onResize } = options;

  const ctx = canvas.getContext('2d');
  if (!ctx) {
    throw new Error('Failed to get 2D context from canvas');
  }

  let resizeTimer: ReturnType<typeof setTimeout> | null = null;

  function sizeCanvas() {
    const rect = container.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    canvas.style.width = `${rect.width}px`;
    canvas.style.height = `${rect.height}px`;
    ctx!.setTransform(dpr, 0, 0, dpr, 0, 0);
    onResize?.();
  }

  const resizeObserver = new ResizeObserver(() => {
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(sizeCanvas, 100);
  });
  resizeObserver.observe(container);

  // Initial sizing
  sizeCanvas();

  return {
    ctx,
    sizeCanvas,
    destroy() {
      resizeObserver.disconnect();
      if (resizeTimer) {
        clearTimeout(resizeTimer);
        resizeTimer = null;
      }
    }
  };
}
