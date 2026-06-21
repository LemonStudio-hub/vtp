/**
 * Canvas Animation Loop Utility
 *
 * Provides a budget-based animation loop with requestAnimationFrame.
 * Extracted from ProgressHero and SpeedChart where the same pattern
 * is repeated.
 */

export interface AnimationLoopOptions {
  /** Minimum milliseconds between frames (from animationBudget store) */
  budget: number;
  /** Called on each frame with delta time in seconds */
  onFrame: (dt: number) => void;
}

export interface AnimationLoopHandle {
  /** Start the animation loop */
  start: () => void;
  /** Stop the animation loop */
  stop: () => void;
  /** Clean up (stops the loop) */
  destroy: () => void;
  /** Update the budget (e.g., when animationBudget store changes) */
  setBudget: (ms: number) => void;
}

/**
 * Create a budget-throttled animation loop.
 *
 * The loop uses requestAnimationFrame but skips frames that arrive
 * before the budget interval has elapsed. This prevents excessive
 * redraws when the tab is visible (16ms budget) and reduces work
 * when hidden (1000ms budget).
 */
export function createAnimationLoop(options: AnimationLoopOptions): AnimationLoopHandle {
  let { budget } = options;
  const { onFrame } = options;
  let animFrame = 0;
  let lastFrame = 0;
  let running = false;

  function tick() {
    if (!running) return;

    const now = performance.now();
    const elapsed = now - lastFrame;

    if (elapsed < budget) {
      animFrame = requestAnimationFrame(tick);
      return;
    }

    lastFrame = now;
    const dt = Math.min(elapsed / 1000, 0.1); // Cap at 100ms to avoid spiral
    onFrame(dt);

    animFrame = requestAnimationFrame(tick);
  }

  return {
    start() {
      if (running) return;
      running = true;
      lastFrame = performance.now();
      animFrame = requestAnimationFrame(tick);
    },
    stop() {
      running = false;
      if (animFrame) {
        cancelAnimationFrame(animFrame);
        animFrame = 0;
      }
    },
    destroy() {
      this.stop();
    },
    setBudget(ms: number) {
      budget = ms;
    }
  };
}
