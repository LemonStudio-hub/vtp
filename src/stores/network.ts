/**
 * Network State Management Module
 *
 * Manages the VTP network layer state using Svelte stores.
 * Provides reactive state for:
 * - Signaling connection status
 * - Connected peers and their info
 * - Network-wide events (peer join/leave, messages)
 *
 * @example
 * ```typescript
 * import { networkState, networkEvents } from '$stores/network';
 *
 * networkState.subscribe(state => {
 *   console.log(`Peers: ${state.peerCount}, Signaling: ${state.signalingState}`);
 * });
 * ```
 */

import { writable, derived } from 'svelte/store';
import type { SignalingState, PeerInfo, PeerManagerState } from '$lib/network/types';

/**
 * Network Event interface.
 *
 * Represents a network-layer event (peer join/leave, message received).
 */
export interface NetworkEvent {
  /** Event type. */
  type: 'peer-joined' | 'peer-left' | 'message' | 'signaling' | 'error';
  /** Event timestamp (Unix epoch ms). */
  timestamp: number;
  /** Human-readable description. */
  message: string;
  /** Associated peer ID, if any. */
  peerId?: string;
}

/**
 * Network State interface.
 *
 * Aggregate view of the networking layer state, suitable for UI display.
 */
export interface NetworkState {
  /** Signaling server connection state. */
  signalingState: SignalingState;

  /** This node's peer ID (assigned by signaling server). */
  localPeerId: string | null;

  /** Number of connected peers. */
  peerCount: number;

  /** Map of peer ID → PeerInfo for all active peers. */
  peers: Map<string, PeerInfo>;

  /** Whether the network layer is initialized and connected. */
  isConnected: boolean;

  /** Number of IPv6 peers. */
  ipv6PeerCount: number;

  /** Average RTT across all peers (ms, -1 if unknown). */
  averageRtt: number;
}

// ─── Stores ───────────────────────────────────────────────────────

/**
 * Network State Store.
 *
 * Holds the current state of the networking layer.
 * Updated by the PeerManager event handlers in +page.svelte.
 */
export const networkState = writable<NetworkState>({
  signalingState: 'disconnected',
  localPeerId: null,
  peerCount: 0,
  peers: new Map(),
  isConnected: false,
  ipv6PeerCount: 0,
  averageRtt: -1
});

/**
 * Network Event Log Store.
 *
 * Stores network events for display in the UI.
 * Capped at 30 entries; older events are removed.
 */
export const networkEvents = writable<NetworkEvent[]>([]);

// ─── Derived Stores ───────────────────────────────────────────────

/**
 * Whether the signaling server is connected.
 */
export const signalingConnected = derived(
  networkState,
  ($state) => $state.signalingState === 'connected'
);

/**
 * Whether there are any connected peers.
 */
export const hasPeers = derived(networkState, ($state) => $state.peerCount > 0);

/**
 * Connection quality indicator.
 * 'good' = all peers connected, low RTT
 * 'degraded' = some peers disconnected or high RTT
 * 'disconnected' = no signaling or no peers
 */
export const connectionQuality = derived(networkState, ($state) => {
  if ($state.signalingState !== 'connected' || $state.peerCount === 0) {
    return 'disconnected';
  }

  const connectedPeers = [...$state.peers.values()].filter((p) => p.state === 'connected');
  if (connectedPeers.length < $state.peerCount * 0.5) {
    return 'degraded';
  }

  if ($state.averageRtt > 500) {
    return 'degraded';
  }

  return 'good';
});

// ─── Actions ──────────────────────────────────────────────────────

/**
 * Update the network state from a PeerManagerState.
 *
 * Called by +page.svelte when PeerManager emits state changes.
 */
export function updateNetworkState(managerState: PeerManagerState): void {
  const peers = new Map(managerState.peers);
  const connectedPeers = [...peers.values()].filter((p) => p.state === 'connected');
  const ipv6Peers = connectedPeers.filter((p) => p.isIpv6);
  const rtts = connectedPeers.filter((p) => p.rtt > 0).map((p) => p.rtt);
  const avgRtt = rtts.length > 0 ? rtts.reduce((a, b) => a + b, 0) / rtts.length : -1;

  networkState.set({
    signalingState: managerState.signaling,
    localPeerId: managerState.localPeerId,
    peerCount: managerState.peerCount,
    peers,
    isConnected: managerState.signaling === 'connected' && managerState.peerCount > 0,
    ipv6PeerCount: ipv6Peers.length,
    averageRtt: Math.round(avgRtt)
  });
}

/**
 * Add a network event to the event log.
 *
 * Automatically prepends the event and caps the list at 30 entries.
 */
export function addNetworkEvent(event: Omit<NetworkEvent, 'timestamp'>): void {
  networkEvents.update((current) => [{ ...event, timestamp: Date.now() }, ...current.slice(0, 29)]);
}

/**
 * Reset all network state to initial values.
 */
export function resetNetworkState(): void {
  networkState.set({
    signalingState: 'disconnected',
    localPeerId: null,
    peerCount: 0,
    peers: new Map(),
    isConnected: false,
    ipv6PeerCount: 0,
    averageRtt: -1
  });
  networkEvents.set([]);
}
