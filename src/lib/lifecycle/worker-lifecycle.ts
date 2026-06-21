/**
 * Worker Lifecycle Module
 *
 * Encapsulates Web Worker instantiation, message routing, and teardown.
 * Extracted from +page.svelte for independent testability.
 */

import { workerStore, workerState, addEvent, pushSpeedHistory } from '$stores/worker';
import type {
  WorkerResponse,
  HeartbeatMessage,
  ProgressMessage,
  WinnerMessage,
  ErrorMessage,
  StartedMessage,
  FinishedMessage
} from '$lib/worker/types';
import type { StatePersistence } from '$lib/background';
import type { WorkerWatchdog } from '$lib/background';

export interface WorkerLifecycleOptions {
  watchdog: WorkerWatchdog | null;
  persistence: StatePersistence | null;
  onSystemWake: (duration: number) => void;
}

export interface WorkerLifecycleHandle {
  worker: Worker;
  destroy: () => void;
}

/**
 * Create and wire up a Web Worker with message routing.
 *
 * Handles all Worker→Main thread message dispatch:
 * - heartbeat → watchdog
 * - progress → stores + persistence
 * - started/winner/error/finished/stopped → stores + events
 */
export function createWorkerLifecycle(options: WorkerLifecycleOptions): WorkerLifecycleHandle {
  const { watchdog, persistence, onSystemWake } = options;

  const worker = new Worker(new URL('$lib/worker/index.ts', import.meta.url), {
    type: 'module'
  });

  function handleMessage(event: MessageEvent<WorkerResponse>) {
    const data = event.data;

    switch (data.type) {
      case 'heartbeat': {
        const hb = data as HeartbeatMessage;
        watchdog?.recordHeartbeat(hb.timestamp, hb.status);
        if (hb.drift && hb.drift > 5000) {
          onSystemWake(hb.drift);
        }
        break;
      }

      case 'progress': {
        const msg = data as ProgressMessage;
        pushSpeedHistory(msg.speed);
        workerState.update((s) => ({
          ...s,
          currentStep: msg.step,
          memoryUsage: msg.memoryUsage || 0
        }));
        persistence?.updateState({
          stepCount: msg.step,
          speed: msg.speed,
          wasRunning: true,
          wasPaused: false
        });
        break;
      }

      case 'started': {
        const msg = data as StartedMessage;
        workerState.update((s) => ({
          ...s,
          isRunning: true,
          isPaused: false,
          publicKey: msg.publicKey
        }));
        addEvent({ type: 'info', message: 'VDF computation started' });
        break;
      }

      case 'winner': {
        const msg = data as WinnerMessage;
        workerState.update((s) => ({
          ...s,
          winnerCount: s.winnerCount + 1
        }));
        addEvent({ type: 'winner', message: `Winner found at step ${msg.step}` });
        break;
      }

      case 'error': {
        const msg = data as ErrorMessage;
        addEvent({ type: 'error', message: `[${msg.code}] ${msg.message}` });
        break;
      }

      case 'finished': {
        const msg = data as FinishedMessage;
        workerState.update((s) => ({
          ...s,
          isRunning: false,
          currentStep: msg.step
        }));
        addEvent({ type: 'info', message: `VDF computation finished at step ${msg.step}` });
        break;
      }

      case 'stopped': {
        workerState.update((s) => ({
          ...s,
          isRunning: false,
          isPaused: false
        }));
        break;
      }
    }
  }

  worker.onmessage = handleMessage;
  workerStore.set(worker);

  return {
    worker,
    destroy() {
      worker.terminate();
    }
  };
}
