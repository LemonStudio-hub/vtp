/**
 * Lifecycle Module
 *
 * Barrel export for the three lifecycle subsystems extracted from +page.svelte.
 */

export { createWorkerLifecycle } from './worker-lifecycle';
export type { WorkerLifecycleOptions, WorkerLifecycleHandle } from './worker-lifecycle';

export { createBackgroundSystem } from './background-lifecycle';
export type { BackgroundLifecycleOptions, BackgroundLifecycleHandle } from './background-lifecycle';

export { createNetworkLifecycle } from './network-lifecycle';
export type { NetworkLifecycleOptions, NetworkLifecycleHandle } from './network-lifecycle';
