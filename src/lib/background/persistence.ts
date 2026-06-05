/**
 * State Persistence Manager
 *
 * Provides IndexedDB-based state snapshots for computation recovery.
 *
 * When a browser tab is discarded (Chrome Memory Saver) or the system
 * sleeps for an extended period, in-memory state is lost. This module
 * periodically snapshots