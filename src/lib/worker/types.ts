/**
 * Worker Communication Type Definitions
 *
 * Defines the message types for communication between the Web Worker
 * and the main thread. Uses TypeScript interfaces to ensure type safety.
 *
 * Message flow:
 * - Main thread → Worker: WorkerMessage (control commands)
 * - Worker → Main thread: WorkerResponse (status updates)
 *
 * @example
 * ```typescript
 * import type { WorkerMessage, ProgressMessage } from './types';
 *
 * // Send a control command
 * const startMsg: WorkerMessage = {
 *   type: 'start',
 *   seed: new Uint8Array(32),
 *   total: 1000000,
 *   k: 1000,
 *   tau: new Uint8Array(32),
 *   checkpointInterval: 100000
 * };
 * worker.postMessage(startMsg);
 *
 * // Listen for progress updates
 * worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
 *   if (event.data.type === 'progress') {
 *     const msg = event.data as ProgressMessage;
 *     console.log(`Step: ${msg.step}, Speed: ${msg.speed}`);
 *   }
 * };
 * ```
 *
 * @module types
 */

/**
 * Worker Message Interface
 *
 * Control messages sent from the main thread to the Worker.
 * Supported command types: start / pause / resume / stop
 */
export interface WorkerMessage {
  /** Command type */
  type: string;

  /** VDF computation seed, minimum 32 bytes */
  seed?: Uint8Array;

  /** Total number of VDF steps to compute */
  total?: number;

  /** VRF draw interval */
  k?: number;

  /** VRF threshold, 32 bytes */
  tau?: Uint8Array;

  /** Checkpoint interval */
  checkpointInterval?: number;

  /** Maximum steps per batch */
  maxSteps?: number;
}

/**
 * Progress Message Interface
 *
 * Periodically sent by the Worker to the main thread as a progress report.
 * Sent once per second, containing the current step count, speed, and
 * memory usage.
 */
export interface ProgressMessage {
  /** Message type */
  type: 'progress';

  /** Current number of VDF steps completed */
  step: number;

  /** Current computation speed (steps per second) */
  speed: number;

  /** Current memory usage (bytes) */
  memoryUsage: number;
}

/**
 * Winner Message Interface
 *
 * Sent to the main thread when a winning draw is discovered.
 * Contains the winning step number and the VRF proof.
 */
export interface WinnerMessage {
  /** Message type */
  type: 'winner';

  /** Winning step number */
  step: number;

  /** VRF proof */
  proof: Uint8Array;
}

/**
 * Heartbeat Message Interface
 *
 * Sent periodically to keep the connection with the main thread active.
 * Used to monitor the Worker's liveness and detect system sleep.
 *
 * The `drift` field helps the main thread detect system sleep: if the
 * actual time between heartbeats significantly exceeds the expected
 * interval, the system likely slept or the tab was frozen.
 */
export interface HeartbeatMessage {
  /** Message type */
  type: 'heartbeat';

  /** Message timestamp (Unix epoch milliseconds) */
  timestamp: number;

  /** Current status: running / paused */
  status: string;

  /**
   * Timing drift in ms since last heartbeat.
   * Helps main thread detect system sleep or timer throttling.
   * Present in enhanced heartbeat mode.
   */
  drift?: number;

  /**
   * Current heartbeat interval in ms.
   * Adapts based on visibility state (faster when visible, slower when hidden).
   */
  interval?: number;
}

/**
 * Error Message Interface
 *
 * Sent to the main thread when an error occurs.
 * Contains an error code, description, and recoverability flag.
 */
export interface ErrorMessage {
  /** Message type */
  type: 'error';

  /** Error code for programmatic handling */
  code: string;

  /** Error description for display purposes */
  message: string;

  /** Whether the error is recoverable */
  recoverable: boolean;
}

/**
 * Started Message Interface
 *
 * Sent to the main thread after the Worker has been successfully initialized.
 * Contains the node's VRF public key.
 */
export interface StartedMessage {
  /** Message type */
  type: 'started';

  /** The node's VRF public key */
  publicKey: Uint8Array;
}

/**
 * Stopped Message Interface
 *
 * Sent to the main thread after the Worker has stopped.
 */
export interface StoppedMessage {
  /** Message type */
  type: 'stopped';
}

/**
 * Finished Message Interface
 *
 * Sent to the main thread after the VDF computation has completed.
 * Contains the final step count.
 */
export interface FinishedMessage {
  /** Message type */
  type: 'finished';

  /** Final step count */
  step: number;
}

/**
 * Worker Response Type Union
 *
 * All possible Worker response message types.
 * Used for type-safe message handling.
 */
export type WorkerResponse =
  | ProgressMessage
  | WinnerMessage
  | HeartbeatMessage
  | ErrorMessage
  | StartedMessage
  | StoppedMessage
  | FinishedMessage;
