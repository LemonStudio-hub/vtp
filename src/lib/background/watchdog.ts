/**
 * Worker Watchdog
 *
 * Monitors Web Worker health via heartbeat messages and provides
 * automatic recovery when the worker becomes unresponsive.
 *
 * Key capabilities:
 * 1. Heartbeat monitoring with configurable timeout
 * 2. Missed heartbeat tracking and escalation
 * 3. Automatic worker restart on failure
 * 4. Integration with visibility state (relaxed monitoring when hidden)
 *
 * Design rationale:
 * - Workers can become unresponsive when the system sleeps or the browser
 *   discards the tab. The watchdog detects this via missed heartbeats.
 * - When the tab is hidden, we use relaxed thresholds to account for
 *   browser timer throttling on the Worker's setInterval.
 * - On wake from sleep, we immediately check if the worker is still alive
 *   instead of waiting for the next scheduled heartbeat.
 *
 * @module watchdog
 */

export interface WatchdogOptions {
  /** Heartbeat timeout in ms before considering worker unresponsive (default: 30000) */
  heartbeatTimeout?: number;
  /** Relaxed heartbeat timeout when tab is hidden (default: 60000) */
  hiddenTimeout?: number;
  /** Max consecutive missed heartbeats before restart (default: 3) */
  maxMissedBeats?: number;
  /** Callback when worker is detected as unresponsive */
  onUnresponsive?: () => void;
  /** Callback when worker is restarted */
  onRestart?: () => void;
  /** Callback when heartbeat is received */
  onHeartbeat?: (timestamp: number, status: string) => void;
}

interface WatchdogState {
  lastHeartbeat: number;
  missedBeats: number;
  isMonitoring: boolean;
}

/**
 * Monitors Web Worker health and provides automatic recovery.
 *
 * @example
 * ```typescript
 * const watchdog = new WorkerWatchdog({
 *   heartbeatTimeout: 30000,
 *   maxMissedBeats: 3,
 *   onUnresponsive: () => console.warn('Worker unresponsive!'),
 * });
 *
 * // When worker sends a heartbeat:
 * watchdog.recordHeartbeat(Date.now(), 'running');
 *
 * // Start monitoring:
 * watchdog.start();
 *
 * // Cleanup:
 * watchdog.destroy();
 * ```
 */
export class WorkerWatchdog {
  private options: Required<WatchdogOptions>;
  private state: WatchdogState;
  private checkInterval: ReturnType<typeof setInterval> | null = null;
  private isHidden = false;
  private destroyed = false;

  private visibilityHandler: (() => void) | null = null;

  constructor(options: WatchdogOptions = {}) {
    this.options = {
      heartbeatTimeout: options.heartbeatTimeout ?? 30000,
      hiddenTimeout: options.hiddenTimeout ?? 60000,
      maxMissedBeats: options.maxMissedBeats ?? 3,
      onUnresponsive: options.onUnresponsive ?? (() => {}),
      onRestart: options.onRestart ?? (() => {}),
      onHeartbeat: options.onHeartbeat ?? (() => {})
    };

    this.state = {
      lastHeartbeat: Date.now(),
      missedBeats: 0,
      isMonitoring: false
    };
  }

  /** Current number of consecutive missed heartbeats */
  get missedBeats(): number {
    return this.state.missedBeats;
  }

  /** Whether the watchdog is currently monitoring */
  get isMonitoring(): boolean {
    return this.state.isMonitoring;
  }

  /**
   * Record a heartbeat from the worker.
   * Resets the missed beat counter.
   */
  recordHeartbeat(timestamp: number, status: string): void {
    this.state.lastHeartbeat = timestamp;
    this.state.missedBeats = 0;
    this.options.onHeartbeat(timestamp, status);
  }

  /**
   * Start monitoring the worker.
   * Checks heartbeat status every 5 seconds.
   */
  start(): void {
    if (this.state.isMonitoring) return;

    this.state.isMonitoring = true;
    this.state.lastHeartbeat = Date.now();
    this.state.missedBeats = 0;

    // Listen for visibility changes to adjust timeout thresholds
    this.visibilityHandler = () => {
      this.isHidden = document.visibilityState === 'hidden';
    };
    document.addEventListener('visibilitychange', this.visibilityHandler);
    this.isHidden = document.visibilityState === 'hidden';

    // Periodic health check
    this.checkInterval = setInterval(() => {
      this.checkHealth();
    }, 5000);
  }

  /**
   * Stop monitoring.
   */
  stop(): void {
    this.state.isMonitoring = false;

    if (this.checkInterval !== null) {
      clearInterval(this.checkInterval);
      this.checkInterval = null;
    }

    if (this.visibilityHandler) {
      document.removeEventListener('visibilitychange', this.visibilityHandler);
      this.visibilityHandler = null;
    }
  }

  /**
   * Perform an immediate health check.
   * Useful when waking from sleep to verify worker status without waiting
   * for the next scheduled check.
   */
  checkNow(): void {
    if (this.state.isMonitoring) {
      this.checkHealth();
    }
  }

  /**
   * Check worker health based on time since last heartbeat.
   */
  private checkHealth(): void {
    const now = Date.now();
    const elapsed = now - this.state.lastHeartbeat;
    const threshold = this.isHidden ? this.options.hiddenTimeout : this.options.heartbeatTimeout;

    if (elapsed > threshold) {
      this.state.missedBeats++;

      if (this.state.missedBeats >= this.options.maxMissedBeats) {
        // Worker is considered unresponsive - trigger restart
        this.options.onUnresponsive();
        this.options.onRestart();
        // Reset state after restart
        this.state.lastHeartbeat = Date.now();
        this.state.missedBeats = 0;
      } else {
        this.options.onUnresponsive();
      }
    }
  }

  /**
   * Clean up all resources.
   */
  destroy(): void {
    if (this.destroyed) return;
    this.destroyed = true;
    this.stop();
  }
}
