/**
 * Background Lifecycle Module
 *
 * Encapsulates initialization and teardown of the background keep-alive system:
 * - VisibilityManager: detects tab visibility and system sleep
 * - AudioKeepAlive: silent AudioContext prevents timer throttling
 * - WorkerWatchdog: monitors worker health via heartbeat
 * - StatePersistence: periodic IndexedDB snapshots for recovery
 *
 * Extracted from +page.svelte for independent testability.
 */

import {
  isVisible,
  audioKeepAliveActive,
  workerHealthy,
  missedHeartbeats,
  signalWake
} from '$stores/visibility';
import {
  VisibilityManager,
  AudioKeepAlive,
  WorkerWatchdog,
  StatePersistence
} from '$lib/background';
import { get } from 'svelte/store';
import { workerState } from '$stores/worker';

export interface BackgroundLifecycleOptions {
  worker: Worker;
  onSystemWake: (duration: number) => void;
}

export interface BackgroundLifecycleHandle {
  watchdog: WorkerWatchdog;
  persistence: StatePersistence | null;
  destroy: () => void;
}

/** Get current computation state for persistence. */
function getCurrentSnapshot() {
  const s = get(workerState);
  return {
    timestamp: Date.now(),
    stepCount: s.currentStep,
    totalSteps: s.totalSteps,
    speed: s.speed,
    uptime: s.uptime,
    winnerCount: s.winnerCount,
    wasRunning: s.isRunning,
    wasPaused: s.isPaused
  };
}

/**
 * Initialize the background keep-alive system.
 *
 * Strategy layering (progressive enhancement):
 * 1. VisibilityManager - detects tab state (always active)
 * 2. AudioKeepAlive - prevents timer throttling (best-effort)
 * 3. WorkerWatchdog - monitors worker health (always active when running)
 * 4. StatePersistence - saves state for recovery (best-effort)
 */
export async function createBackgroundSystem(
  options: BackgroundLifecycleOptions
): Promise<BackgroundLifecycleHandle> {
  const { worker, onSystemWake } = options;

  let visibilityManager: VisibilityManager | null = null;
  let audioKeepAlive: AudioKeepAlive | null = null;
  let watchdog: WorkerWatchdog | null = null;
  let persistence: StatePersistence | null = null;

  // 1. Visibility Manager
  visibilityManager = new VisibilityManager({
    onVisibilityChange: ({ state }) => {
      isVisible.set(state === 'visible');

      if (worker) {
        worker.postMessage({
          type: 'setHeartbeatMode',
          visible: state === 'visible'
        });
      }

      if (state === 'hidden' && persistence) {
        persistence.updateState(getCurrentSnapshot());
        persistence.saveSnapshot(getCurrentSnapshot());
      }
    },
    onWake: ({ duration, isSystemSleep }) => {
      signalWake();

      if (isSystemSleep && duration > 10000) {
        onSystemWake(duration);
      }
    }
  });

  // 2. AudioContext Keep-Alive
  audioKeepAlive = new AudioKeepAlive({ autoResume: true });
  const audioStarted = audioKeepAlive.start();
  audioKeepAliveActive.set(audioStarted);

  // 3. Worker Watchdog
  watchdog = new WorkerWatchdog({
    heartbeatTimeout: 30000,
    hiddenTimeout: 60000,
    maxMissedBeats: 3,
    onUnresponsive: () => {
      workerHealthy.set(false);
      missedHeartbeats.update((n) => n + 1);
    },
    onRestart: () => {
      console.warn('[Watchdog] Worker unresponsive, triggering recovery');
      if (persistence) {
        persistence.updateState(getCurrentSnapshot());
        persistence.saveSnapshot(getCurrentSnapshot());
      }
    },
    onHeartbeat: () => {
      workerHealthy.set(true);
      missedHeartbeats.set(0);
    }
  });
  watchdog.start();

  // 4. State Persistence
  persistence = new StatePersistence();
  await persistence.init();
  persistence.startPeriodicSnapshots(30000);

  // Check for recovery snapshot on page load
  const snapshot = await persistence.restoreSnapshot();
  if (snapshot?.wasRunning && !snapshot.wasPaused) {
    // Found recovery snapshot at step ${snapshot.stepCount}
  }

  return {
    watchdog,
    persistence,
    destroy() {
      visibilityManager?.destroy();
      visibilityManager = null;

      audioKeepAlive?.stop();
      audioKeepAlive = null;
      audioKeepAliveActive.set(false);

      watchdog?.destroy();
      watchdog = null;

      persistence?.destroy();
      persistence = null;
    }
  };
}
