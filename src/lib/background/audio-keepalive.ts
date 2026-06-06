/**
 * AudioContext Keep-Alive Manager
 *
 * Uses a silent AudioContext with an oscillator connected to a zero-volume
 * GainNode to prevent browsers from throttling background tab timers.
 *
 * Why this works (2026 best practices):
 * - Browsers (especially Chrome) deprioritize tabs that appear "idle"
 * - An active AudioContext signals to the browser that the tab is doing
 *   meaningful work, preventing aggressive timer throttling
 * - A silent oscillator (gain=0) achieves this without audible side effects
 * - This is a progressive enhancement: if AudioContext is unavailable,
 *   the app falls back to other keep-alive strategies
 *
 * Browser compatibility:
 * - Chrome/Edge: Full support, prevents background timer throttling
 * - Firefox: AudioContext works but throttling behavior differs
 * - Safari: Requires user gesture to create AudioContext; handled via resume()
 * - Mobile browsers: May be overridden by OS-level power management
 *
 * Fallback chain:
 * AudioContext -> Web Worker (already in place) -> setInterval with visibility awareness
 *
 * @module audio-keepalive
 */

export interface AudioKeepAliveOptions {
  /** Whether to auto-resume on visibility change (default: true) */
  autoResume?: boolean;
}

/**
 * Manages a silent AudioContext to prevent browser background throttling.
 *
 * @example
 * ```typescript
 * const keepAlive = new AudioKeepAlive();
 * keepAlive.start();
 *
 * // Later:
 * keepAlive.stop();
 * ```
 */
export class AudioKeepAlive {
  private audioCtx: AudioContext | null = null;
  private oscillator: OscillatorNode | null = null;
  private gainNode: GainNode | null = null;
  private started = false;
  private options: Required<AudioKeepAliveOptions>;
  private visibilityHandler: (() => void) | null = null;

  constructor(options: AudioKeepAliveOptions = {}) {
    this.options = {
      autoResume: options.autoResume ?? true
    };
  }

  /** Whether the AudioContext keep-alive is currently active */
  get isActive(): boolean {
    return this.started && this.audioCtx?.state === 'running';
  }

  /**
   * Start the silent AudioContext.
   *
   * Creates an OscillatorNode → GainNode(gain=0) → destination chain.
   * The oscillator produces a signal that is completely silenced by the
   * zero-gain node, but the active AudioContext prevents throttling.
   *
   * @returns true if successfully started, false if not supported
   */
  start(): boolean {
    if (this.started) return true;

    try {
      const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
      if (!AudioCtx) {
        console.warn('[AudioKeepAlive] AudioContext not supported');
        return false;
      }

      this.audioCtx = new AudioCtx();

      // Create silent oscillator chain
      this.oscillator = this.audioCtx.createOscillator();
      this.gainNode = this.audioCtx.createGain();

      // Zero volume - completely inaudible
      this.gainNode.gain.value = 0;

      // Connect: oscillator → gain (silent) → speakers
      this.oscillator.connect(this.gainNode);
      this.gainNode.connect(this.audioCtx.destination);

      // Start oscillating
      this.oscillator.start();

      this.started = true;

      // Auto-resume on visibility change (handles Safari's auto-suspend)
      if (this.options.autoResume) {
        this.visibilityHandler = () => {
          if (document.visibilityState === 'visible' && this.audioCtx?.state === 'suspended') {
            this.audioCtx.resume();
          }
        };
        document.addEventListener('visibilitychange', this.visibilityHandler);
      }

      // Handle Safari's requirement for user gesture
      if (this.audioCtx.state === 'suspended') {
        this.resumeOnUserGesture();
      }

      return true;
    } catch (err) {
      console.warn('[AudioKeepAlive] Failed to start:', err);
      return false;
    }
  }

  /**
   * Safari requires a user gesture to resume AudioContext.
   * We listen for the first user interaction and resume automatically.
   */
  private resumeOnUserGesture(): void {
    const resumeHandler = () => {
      if (this.audioCtx?.state === 'suspended') {
        this.audioCtx.resume();
      }
      // Remove after first interaction
      document.removeEventListener('click', resumeHandler);
      document.removeEventListener('keydown', resumeHandler);
      document.removeEventListener('touchstart', resumeHandler);
    };

    document.addEventListener('click', resumeHandler, { once: false });
    document.addEventListener('keydown', resumeHandler, { once: false });
    document.addEventListener('touchstart', resumeHandler, { once: false });
  }

  /**
   * Stop the AudioContext and release resources.
   */
  stop(): void {
    if (!this.started) return;

    try {
      this.oscillator?.stop();
      this.oscillator?.disconnect();
      this.gainNode?.disconnect();
      this.audioCtx?.close();
    } catch {
      // Ignore errors during cleanup
    }

    this.oscillator = null;
    this.gainNode = null;
    this.audioCtx = null;
    this.started = false;

    if (this.visibilityHandler) {
      document.removeEventListener('visibilitychange', this.visibilityHandler);
      this.visibilityHandler = null;
    }
  }
}
