/**
 * Page Visibility Manager
 *
 * Detects tab visibility changes, system sleep/wake cycles, and provides
 * a unified "effective visibility" state for the entire application.
 *
 * Key capabilities:
 * 1. Page Visibility API integration (visibilitychange)
 * 2. System sleep/wake detection via timestamp drift analysis
 * 3. Configurable callback system for visibility transitions
 * 4. Wake duration estimation for state recovery
 *
 * Design rationale (2026 best practices):
 * - visibilitychange is the ONLY event that works reliably across all browsers
 *   for detecting tab background transitions (Chrome, Safari, Firefox)
 * - System sleep detection uses performance.now() drift analysis: when the system
 *   sleeps, setTimeout callbacks fire late by exactly the sleep duration
 * - BFCache recovery requires explicit handling (pageshow with persisted=true)
 *
 * @module visibility
 */

export type VisibilityState = 'visible' | 'hidden';

export interface VisibilityChangeEvent {
  state: VisibilityState;
  timestamp: number;
}

export interface WakeEvent {
  /** Duration of the sleep/background period in milliseconds */
  duration: number;
  /** Whether this was a system sleep (vs simple tab switch) */
  isSystemSleep: boolean;
  /** Timestamp when the wake was detected */
  timestamp: number;
}

export interface VisibilityManagerOptions {
  /** Callback when visibility state changes */
  onVisibilityChange?: (event: VisibilityChangeEvent) => void;
  /** Callback when returning from background or system sleep */
  onWake?: (event: WakeEvent) => void;
  /** Threshold in ms to consider a gap as system sleep (default: 5000) */
  sleepThreshold?: number;
}

/**
 * Monitors page visibility and system sleep/wake cycles.
 *
 * @example
 * ```typescript
 * const manager = new VisibilityManager({
 *   onVisibilityChange: ({ state }) => console.log('Visibility:', state),
 *   onWake: ({ duration, isSystemSleep }) => {
 *     console.log(`Woke after ${duration}ms, sleep: ${isSystemSleep}`);
 *   }
 * });
 *
 * // Later, when done:
 * manager.destroy();
 * ```
 */
export class VisibilityManager {
  private options: Required<VisibilityManagerOptions>;
  private lastActiveTime: number;
  private sleepCheckInterval: ReturnType<typeof setInterval> | null = null;
  private currentState: VisibilityState;
  private destroyed = false;

  /** Listeners for cleanup */
  private boundHandleVisibilityChange: () => void;
  private boundHandlePageShow: (e: PageTransitionEvent) => void;

  constructor(options: VisibilityManagerOptions = {}) {
    this.options = {
      onVisibilityChange: options.onVisibilityChange ?? (() => {}),
      onWake: options.onWake ?? (() => {}),
      sleepThreshold: options.sleepThreshold ?? 5000
    };

    this.currentState = document.visibilityState === 'visible' ? 'visible' : 'hidden';
    this.lastActiveTime = performance.now();

    // Bind handlers for cleanup
    this.boundHandleVisibilityChange = this.handleVisibilityChange.bind(this);
    this.boundHandlePageShow = this.handlePageShow.bind(this);

    // Register event listeners
    document.addEventListener('visibilitychange', this.boundHandleVisibilityChange);
    window.addEventListener('pageshow', this.boundHandlePageShow);

    // Start sleep detection polling
    this.startSleepDetection();
  }

  /** Current visibility state */
  get state(): VisibilityState {
    return this.currentState;
  }

  /** Whether the page is currently visible */
  get isVisible(): boolean {
    return this.currentState === 'visible';
  }

  /**
   * Handle visibilitychange event.
   *
   * This is the primary mechanism for detecting tab switches.
   * Covers: tab switch, Alt+Tab, minimize, mobile home button, lock screen.
   */
  private handleVisibilityChange(): void {
    const newState: VisibilityState = document.visibilityState === 'visible' ? 'visible' : 'hidden';
    const previousState = this.currentState;
    this.currentState = newState;

    if (newState === 'visible' && previousState === 'hidden') {
      // Returning from background - detect potential system sleep
      const now = performance.now();
      const gap = now - this.lastActiveTime;

      this.options.onVisibilityChange({ state: 'visible', timestamp: Date.now() });

      if (gap > this.options.sleepThreshold) {
        this.options.onWake({
          duration: gap,
          isSystemSleep: true,
          timestamp: Date.now()
        });
      } else {
        this.options.onWake({
          duration: gap,
          isSystemSleep: false,
          timestamp: Date.now()
        });
      }

      this.lastActiveTime = now;
    } else if (newState === 'hidden') {
      this.lastActiveTime = performance.now();
      this.options.onVisibilityChange({ state: 'hidden', timestamp: Date.now() });
    }
  }

  /**
   * Handle pageshow event for BFCache recovery.
   *
   * When a page is restored from BFCache (back-forward cache),
   * the pageshow event fires with `persisted=true`. In this case,
   * normal visibilitychange may not fire, so we handle it separately.
   */
  private handlePageShow(event: PageTransitionEvent): void {
    if (event.persisted) {
      // Page was restored from BFCache
      this.currentState = 'visible';
      this.lastActiveTime = performance.now();

      this.options.onVisibilityChange({ state: 'visible', timestamp: Date.now() });
      this.options.onWake({
        duration: 0, // Unknown duration from BFCache
        isSystemSleep: false,
        timestamp: Date.now()
      });
    }
  }

  /**
   * Start periodic sleep detection.
   *
   * Uses a low-frequency interval (every 3 seconds) to detect system sleep.
   * If the interval fires much later than expected, the system likely slept.
   * This catches cases that visibilitychange misses (e.g., laptop lid close
   * without explicit tab switch).
   */
  private startSleepDetection(): void {
    let lastCheck = performance.now();

    this.sleepCheckInterval = setInterval(() => {
      const now = performance.now();
      const gap = now - lastCheck;

      // If gap is much larger than the interval (3s), system likely slept
      // We use 2x the interval as the detection threshold
      if (gap > 6000 && this.currentState === 'visible') {
        this.options.onWake({
          duration: gap,
          isSystemSleep: true,
          timestamp: Date.now()
        });
      }

      lastCheck = now;
    }, 3000);
  }

  /**
   * Clean up all event listeners and timers.
   * Must be called when the manager is no longer needed.
   */
  destroy(): void {
    if (this.destroyed) return;
    this.destroyed = true;

    document.removeEventListener('visibilitychange', this.boundHandleVisibilityChange);
    window.removeEventListener('pageshow', this.boundHandlePageShow);

    if (this.sleepCheckInterval !== null) {
      clearInterval(this.sleepCheckInterval);
      this.sleepCheckInterval = null;
    }
  }
}
