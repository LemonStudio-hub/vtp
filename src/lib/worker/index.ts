/**
 * VTP Web Worker Main Module
 *
 * Executes VDF computation tasks in a background thread.
 * Communicates with the main thread via postMessage.
 *
 * Key features:
 * 1. Receives control commands from main thread (start/pause/resume/stop)
 * 2. Executes VDF batch computations
 * 3. Periodically reports computation progress
 * 4. Sends heartbeat packets to maintain connection
 * 5. Reports winning events and errors
 *
 * Communication protocol:
 * - Receive: WorkerMessage (contains commands and parameters)
 * - Send: ProgressMessage | WinnerMessage | HeartbeatMessage | ErrorMessage
 *
 * Performance considerations:
 * - Uses time-slicing mechanism to avoid blocking the browser
 * - Batch processing reduces communication overhead
 * - Periodic memory monitoring prevents leaks
 */

/**
 * Import the WASM module type definitions only.
 * The actual WASM module is loaded at runtime via dynamic import.
 */
import type { Session as SessionType } from '../vtp-core/types/vtp_core';

/**
 * Runtime reference to the Session class, obtained after WASM initialization.
 */
let Session: typeof SessionType;

/**
 * Dynamically load and initialize the WASM module.
 * Uses a runtime fetch + eval approach to bypass Vite's import analysis
 * which cannot parse the Rust doc comments in the wasm-pack generated JS.
 */
async function loadWasm(): Promise<void> {
  const response = await fetch('/wasm/vtp_core.js');
  const wasmCode = await response.text();

  // Create a blob URL and import the module
  const blob = new Blob([wasmCode], { type: 'application/javascript' });
  const blobUrl = URL.createObjectURL(blob);
  const wasmModule = await import(/* @vite-ignore */ blobUrl);
  URL.revokeObjectURL(blobUrl);

  // Initialize the WASM binary
  await wasmModule.default();

  // Extract the Session class
  Session = wasmModule.Session;
}

/**
 * Worker Message Interface
 *
 * Control messages sent from main thread to Worker
 */
interface WorkerMessage {
  /** Command type: start/pause/resume/stop */
  type: string;

  /** VDF computation seed, minimum 32 bytes */
  seed?: Uint8Array;

  /** Total VDF steps target */
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
 * Periodic progress report sent from Worker to main thread
 */
interface ProgressMessage {
  type: 'progress';

  /** Current completed VDF steps */
  step: number;

  /** Current computation speed (steps/sec) */
  speed: number;

  /** Current memory usage (bytes) */
  memoryUsage: number;
}

/**
 * Batch Result Type
 *
 * Corresponds to BatchResult enum in Rust
 */
type BatchResultType = 'Progress' | 'Winner' | 'Finished' | 'Error';

/**
 * Winner Message Interface
 *
 * Sent to main thread when a winning draw is discovered
 */
interface WinnerMessage {
  type: 'winner';

  /** Winning step number */
  step: number;

  /** VRF proof */
  proof: Uint8Array;
}

/**
 * Heartbeat Message Interface
 *
 * Periodically sent to main thread to keep connection alive
 */
interface HeartbeatMessage {
  type: 'heartbeat';

  /** Message timestamp */
  timestamp: number;

  /** Current status: running/paused */
  status: string;

  /** Timing drift in ms since last heartbeat */
  drift?: number;

  /** Current heartbeat interval in ms */
  interval?: number;
}

/**
 * Error Message Interface
 *
 * Sent to main thread when an error occurs
 */
interface ErrorMessage {
  type: 'error';

  /** Error code for programmatic handling */
  code: string;

  /** Error description for display */
  message: string;

  /** Whether the error is recoverable */
  recoverable: boolean;
}

// ==================== State Variables ====================

/** Current VDF session instance */
let session: SessionType | null = null;

/** Whether currently running */
let isRunning = false;

/** Whether paused */
let isPaused = false;

/** Computation start time */
let startTime = 0;

/** Last report time */
let lastReportTime = 0;

/** Current step count */
let stepCount = 0;

/** Current computation speed */
let speed = 0;

/** Heartbeat timer */
let heartbeatInterval: ReturnType<typeof setInterval> | null = null;

/** Timestamp of the last heartbeat sent */
let lastHeartbeatTime = 0;

/**
 * Current heartbeat interval in ms.
 * Adaptive: uses shorter interval when tab is likely visible (10s)
 * and longer interval when likely hidden (30s) to conserve resources.
 * The main thread can send 'setHeartbeatMode' to explicitly control this.
 */
let heartbeatIntervalMs = 10000;

// ==================== Message Handling ====================

/**
 * Handle messages from main thread
 *
 * Dispatches to corresponding handler functions based on message type.
 * Supported commands: start/pause/resume/stop
 *
 * @param event - Message event containing WorkerMessage data
 */
self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
  const { type, ...params } = event.data;

  switch (type) {
    case 'start':
      await handleStart(params);
      break;
    case 'pause':
      handlePause();
      break;
    case 'resume':
      handleResume();
      break;
    case 'stop':
      handleStop();
      break;
    case 'setHeartbeatMode':
      handleSetHeartbeatMode(params as { visible?: boolean });
      break;
    default:
      sendError('UNKNOWN_COMMAND', `Unknown command: ${type}`, false);
  }
};

// ==================== Command Handlers ====================

/**
 * Handle start command
 *
 * Initializes VDF session and begins computation.
 *
 * Workflow:
 * 1. Validate parameter completeness
 * 2. Clean up old session
 * 3. Create new Session instance
 * 4. Initialize state variables
 * 5. Start heartbeat
 * 6. Begin main loop
 *
 * @param params - WorkerMessage containing VDF configuration parameters
 */
async function handleStart(params: Omit<WorkerMessage, 'type'>) {
  try {
    const { seed, total, k, tau, checkpointInterval } = params;

    // Validate required parameters
    if (!seed || !total || !k || !tau || !checkpointInterval) {
      sendError('INVALID_PARAMS', 'Missing required parameters', false);
      return;
    }

    // Load and initialize WASM module if not already done
    if (!Session) {
      await loadWasm();
    }

    // Clean up old session
    if (session) {
      session.free();
    }

    // Create new session
    session = new Session(seed, total, k, tau, checkpointInterval);
    isRunning = true;
    isPaused = false;
    startTime = Date.now();
    lastReportTime = startTime;
    stepCount = 0;

    // Notify main thread that session has started
    self.postMessage({
      type: 'started',
      publicKey: session.public_key()
    });

    // Start heartbeat and main loop
    startHeartbeat();
    runMainLoop();
  } catch (error) {
    sendError('INIT_FAILED', `Failed to initialize session: ${error}`, true);
  }
}

/**
 * Handle pause command
 *
 * Pauses VDF computation.
 * Main loop stops executing but session state is preserved.
 */
function handlePause() {
  if (session && isRunning) {
    session.pause();
    isPaused = true;
    stopHeartbeat();
  }
}

/**
 * Handle resume command
 *
 * Resumes VDF computation.
 * Continues from pause point, restarts heartbeat and main loop.
 */
function handleResume() {
  if (session && isRunning && isPaused) {
    session.resume();
    isPaused = false;
    startHeartbeat();
    runMainLoop();
  }
}

/**
 * Handle stop command
 *
 * Stops VDF computation and cleans up resources.
 */
function handleStop() {
  isRunning = false;
  isPaused = false;
  stopHeartbeat();

  if (session) {
    session.free();
    session = null;
  }

  self.postMessage({ type: 'stopped' });
}

// ==================== Core Computation Loop ====================

/**
 * Main computation loop
 *
 * Continuously executes VDF batch computations until session completes or is paused/stopped.
 *
 * Performance optimization strategies:
 * 1. Batch processing: Execute BATCH_SIZE steps per iteration to reduce function call overhead
 * 2. Time-slicing: Each iteration不超过 TIME_SLICE_MS to avoid blocking the browser
 * 3. Periodic reporting: Report progress every REPORT_INTERVAL_MS milliseconds
 * 4. Error recovery: Catch exceptions and retry with delay
 *
 * Constants:
 * - BATCH_SIZE = 1000: Steps per batch
 * - TIME_SLICE_MS = 50: Time slice length (milliseconds)
 * - REPORT_INTERVAL_MS = 1000: Progress report interval (milliseconds)
 */
async function runMainLoop() {
  const BATCH_SIZE = 1000;
  const TIME_SLICE_MS = 50;
  const REPORT_INTERVAL_MS = 1000;

  while (isRunning && !isPaused) {
    const loopStart = performance.now();

    try {
      if (!session) break;

      // Execute batch VDF computation
      const result = session.run_batch(BATCH_SIZE) as BatchResultType;
      stepCount = session.state().step;

      // Calculate and report progress
      const now = Date.now();
      const elapsed = (now - lastReportTime) / 1000;

      if (elapsed >= REPORT_INTERVAL_MS / 1000) {
        // Calculate current speed (steps/sec)
        speed = (stepCount - (stepCount - BATCH_SIZE)) / elapsed;

        const progressMsg: ProgressMessage = {
          type: 'progress',
          step: stepCount,
          speed: speed,
          memoryUsage: getMemoryUsage()
        };
        self.postMessage(progressMsg);

        lastReportTime = now;
      }

      // Handle winner event
      if (result === 'Winner') {
        const proof = session.get_checkpoint_data();
        const winnerMsg: WinnerMessage = {
          type: 'winner',
          step: stepCount,
          proof: new Uint8Array(proof)
        };
        self.postMessage(winnerMsg);
      }

      // Handle completion event
      if (result === 'Finished') {
        isRunning = false;
        self.postMessage({ type: 'finished', step: stepCount });
        break;
      }

      // Handle error event
      if (result === 'Error') {
        sendError('VDF_ERROR', 'VDF computation error occurred', true);
      }

      // Time-slicing: wait for remaining time if computation took less than slice
      const elapsed_ms = performance.now() - loopStart;
      if (elapsed_ms < TIME_SLICE_MS) {
        await sleep(TIME_SLICE_MS - elapsed_ms);
      }
    } catch (error) {
      sendError('COMPUTATION_ERROR', `Error during computation: ${error}`, true);
      // Delay retry after error
      await sleep(1000);
    }
  }
}

// ==================== Helper Functions ====================

/**
 * Send error message to main thread
 *
 * @param code - Error code for programmatic handling
 * @param message - Error description for display
 * @param recoverable - Whether recoverable, affects frontend handling strategy
 */
function sendError(code: string, message: string, recoverable: boolean) {
  const errorMsg: ErrorMessage = {
    type: 'error',
    code,
    message,
    recoverable
  };
  self.postMessage(errorMsg);
}

/**
 * Get current memory usage
 *
 * Uses performance.memory API to get JavaScript heap memory usage.
 *
 * @returns Memory usage in bytes, or 0 if not supported
 *
 * Compatibility notes:
 * - performance.memory is only available in Chrome/Edge browsers
 * - Firefox and Safari do not support this API
 * - Uses type assertion to avoid TypeScript errors
 */
function getMemoryUsage(): number {
  // Check if performance.memory API is supported
  // Use type assertion because TypeScript doesn't include this property by default
  const perf = performance as any;
  if (perf.memory && typeof perf.memory.usedJSHeapSize === 'number') {
    return perf.memory.usedJSHeapSize;
  }
  return 0;
}

/**
 * Start heartbeat timer
 *
 * Sends heartbeat messages at adaptive intervals to keep connection with
 * main thread alive. Includes drift detection for system sleep identification.
 *
 * Interval adapts based on estimated visibility:
 * - Visible (active): 10 seconds - responsive monitoring
 * - Hidden (background): 30 seconds - conserve resources
 *
 * Each heartbeat includes a `drift` field showing how much time actually
 * elapsed since the last heartbeat. Large drift values indicate the system
 * slept or the browser throttled the Worker's timers.
 */
function startHeartbeat() {
  stopHeartbeat();
  lastHeartbeatTime = Date.now();

  heartbeatInterval = setInterval(() => {
    const now = Date.now();
    const drift = now - lastHeartbeatTime - heartbeatIntervalMs;

    const heartbeatMsg: HeartbeatMessage = {
      type: 'heartbeat',
      timestamp: now,
      status: isPaused ? 'paused' : 'running',
      drift,
      interval: heartbeatIntervalMs
    };
    self.postMessage(heartbeatMsg);

    lastHeartbeatTime = now;
  }, heartbeatIntervalMs);
}

/**
 * Stop heartbeat timer
 *
 * Clears heartbeat timer and resets reference.
 */
function stopHeartbeat() {
  if (heartbeatInterval) {
    clearInterval(heartbeatInterval);
    heartbeatInterval = null;
  }
}

/**
 * Handle setHeartbeatMode command from main thread.
 *
 * Adjusts the heartbeat interval based on tab visibility.
 * Called by the main thread when visibility changes.
 *
 * @param params - Object with `visible` boolean flag
 */
function handleSetHeartbeatMode(params: { visible?: boolean }) {
  const newInterval = params.visible ? 10000 : 30000;

  if (newInterval !== heartbeatIntervalMs) {
    heartbeatIntervalMs = newInterval;

    // Restart heartbeat with new interval if currently running
    if (isRunning && !isPaused) {
      startHeartbeat();
    }
  }
}

/**
 * Async sleep function
 *
 * Returns a Promise that resolves after specified milliseconds.
 * Used for time-slicing control and error retry delays.
 *
 * @param ms - Sleep duration in milliseconds
 * @returns Promise<void>
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
