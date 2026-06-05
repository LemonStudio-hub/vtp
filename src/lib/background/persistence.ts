/**
 * State Persistence Manager
 *
 * Provides IndexedDB-based state snapshots for computation recovery.
 *
 * When a browser tab is discarded (Chrome Memory Saver) or the system
 * sleeps for an extended period, in-memory state is lost. This module
 * periodically snapshots critical state to IndexedDB so computation
 * can be resumed after recovery.
 *
 * Key features:
 * 1. Periodic state snapshots to IndexedDB
 * 2. Snapshot-on-hide (save when tab becomes hidden)
 * 3. State recovery on page load
 * 4. Automatic cleanup of stale snapshots
 *
 * @module persistence
 */

const DB_NAME = 'vtp-state';
const DB_VERSION = 1;
const STORE_NAME = 'snapshots';
const SNAPSHOT_KEY = 'latest';

export interface ComputationSnapshot {
  /** Timestamp of the snapshot */
  timestamp: number;
  /** Current VDF step count */
  stepCount: number;
  /** Total target steps */
  totalSteps: number;
  /** Computation speed at snapshot time */
  speed: number;
  /** Elapsed uptime in seconds */
  uptime: number;
  /** Winner count */
  winnerCount: number;
  /** Whether computation was running */
  wasRunning: boolean;
  /** Whether computation was paused */
  wasPaused: boolean;
}

/**
 * Manages computation state persistence via IndexedDB.
 *
 * @example
 * ```typescript
 * const persistence = new StatePersistence();
 * await persistence.init();
 *
 * // Save a snapshot
 * await persistence.saveSnapshot({
 *   stepCount: 12345,
 *   totalSteps: 1000000,
 *   wasRunning: true,
 *   wasPaused: false,
 *   // ...
 * });
 *
 * // Restore on page load
 * const snapshot = await persistence.restoreSnapshot();
 * if (snapshot?.wasRunning) {
 *   // Resume computation
 * }
 * ```
 */
export class StatePersistence {
  private db: IDBDatabase | null = null;
  private snapshotInterval: ReturnType<typeof setInterval> | null = null;
  private latestState: Partial<ComputationSnapshot> = {};
  private initialized = false;

  /**
   * Initialize the IndexedDB database.
   * Must be called before any other operations.
   */
  async init(): Promise<boolean> {
    if (this.initialized) return true;

    try {
      this.db = await this.openDB();
      this.initialized = true;
      return true;
    } catch (err) {
      console.warn('[StatePersistence] IndexedDB not available:', err);
      return false;
    }
  }

  /**
   * Update the latest state for periodic snapshots.
   * This does NOT immediately write to IndexedDB.
   */
  updateState(state: Partial<ComputationSnapshot>): void {
    this.latestState = { ...this.latestState, ...state, timestamp: Date.now() };
  }

  /**
   * Start periodic snapshot saving.
   * @param intervalMs Snapshot interval in ms (default: 30000 = 30s)
   */
  startPeriodicSnapshots(intervalMs = 30000): void {
    this.stopPeriodicSnapshots();

    this.snapshotInterval = setInterval(async () => {
      if (this.latestState.wasRunning) {
        await this.saveSnapshot(this.latestState as ComputationSnapshot);
      }
    }, intervalMs);
  }

  /**
   * Stop periodic snapshot saving.
   */
  stopPeriodicSnapshots(): void {
    if (this.snapshotInterval !== null) {
      clearInterval(this.snapshotInterval);
      this.snapshotInterval = null;
    }
  }

  /**
   * Save a snapshot immediately (e.g., when tab becomes hidden).
   */
  async saveSnapshot(snapshot: ComputationSnapshot): Promise<void> {
    if (!this.db) return;

    return new Promise((resolve, reject) => {
      try {
        const tx = this.db!.transaction(STORE_NAME, 'readwrite');
        const store = tx.objectStore(STORE_NAME);
        store.put({ ...snapshot, id: SNAPSHOT_KEY });
        tx.oncomplete = () => resolve();
        tx.onerror = () => reject(tx.error);
      } catch {
        resolve(); // Fail silently
      }
    });
  }

  /**
   * Restore the latest snapshot from IndexedDB.
   * @returns The snapshot or null if none exists
   */
  async restoreSnapshot(): Promise<ComputationSnapshot | null> {
    if (!this.db) return null;

    return new Promise((resolve) => {
      try {
        const tx = this.db!.transaction(STORE_NAME, 'readonly');
        const store = tx.objectStore(STORE_NAME);
        const request = store.get(SNAPSHOT_KEY);

        request.onsuccess = () => {
          const result = request.result;
          if (result) {
            // Check if snapshot is not too old (24 hours)
            const age = Date.now() - result.timestamp;
            if (age < 24 * 60 * 60 * 1000) {
              resolve(result as ComputationSnapshot);
            } else {
              resolve(null);
            }
          } else {
            resolve(null);
          }
        };

        request.onerror = () => resolve(null);
      } catch {
        resolve(null);
      }
    });
  }

  /**
   * Clear all stored snapshots.
   */
  async clear(): Promise<void> {
    if (!this.db) return;

    return new Promise((resolve) => {
      try {
        const tx = this.db!.transaction(STORE_NAME, 'readwrite');
        const store = tx.objectStore(STORE_NAME);
        store.clear();
        tx.oncomplete = () => resolve();
        tx.onerror = () => resolve();
      } catch {
        resolve();
      }
    });
  }

  /**
   * Clean up resources.
   */
  destroy(): void {
    this.stopPeriodicSnapshots();
    this.db?.close();
    this.db = null;
    this.initialized = false;
  }

  /**
   * Open the IndexedDB database.
   */
  private openDB(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          db.createObjectStore(STORE_NAME);
        }
      };

      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }
}
