/**
 * Tests for the network state management store.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
  networkState,
  networkEvents,
  signalingConnected,
  hasPeers,
  connectionQuality,
  updateNetworkState,
  addNetworkEvent,
  resetNetworkState
} from '../src/stores/network';
import type { PeerManagerState, PeerInfo } from '../src/lib/network/types';

function makePeer(overrides: Partial<PeerInfo> = {}): PeerInfo {
  return {
    id: 'peer-1',
    state: 'connected',
    isIpv6: false,
    rtt: 50,
    ...overrides
  };
}

function makeManagerState(overrides: Partial<PeerManagerState> = {}): PeerManagerState {
  return {
    signaling: 'connected',
    localPeerId: 'self-123',
    peerCount: 1,
    peers: new Map([['peer-1', makePeer()]]),
    ...overrides
  };
}

describe('Network Store', () => {
  beforeEach(() => {
    resetNetworkState();
  });

  describe('networkState initial values', () => {
    it('has correct defaults', () => {
      const state = get(networkState);
      expect(state.signalingState).toBe('disconnected');
      expect(state.localPeerId).toBeNull();
      expect(state.peerCount).toBe(0);
      expect(state.peers.size).toBe(0);
      expect(state.isConnected).toBe(false);
      expect(state.ipv6PeerCount).toBe(0);
      expect(state.averageRtt).toBe(-1);
    });
  });

  describe('updateNetworkState', () => {
    it('updates state from PeerManagerState', () => {
      updateNetworkState(makeManagerState());
      const state = get(networkState);
      expect(state.signalingState).toBe('connected');
      expect(state.localPeerId).toBe('self-123');
      expect(state.peerCount).toBe(1);
      expect(state.isConnected).toBe(true);
    });

    it('computes IPv6 peer count', () => {
      const peers = new Map<string, PeerInfo>([
        ['p1', makePeer({ id: 'p1', isIpv6: true })],
        ['p2', makePeer({ id: 'p2', isIpv6: false })],
        ['p3', makePeer({ id: 'p3', isIpv6: true })]
      ]);
      updateNetworkState(makeManagerState({ peers, peerCount: 3 }));
      expect(get(networkState).ipv6PeerCount).toBe(2);
    });

    it('computes average RTT', () => {
      const peers = new Map<string, PeerInfo>([
        ['p1', makePeer({ id: 'p1', rtt: 100 })],
        ['p2', makePeer({ id: 'p2', rtt: 200 })]
      ]);
      updateNetworkState(makeManagerState({ peers, peerCount: 2 }));
      expect(get(networkState).averageRtt).toBe(150);
    });

    it('returns -1 RTT when no peers have RTT', () => {
      const peers = new Map<string, PeerInfo>([['p1', makePeer({ id: 'p1', rtt: -1 })]]);
      updateNetworkState(makeManagerState({ peers, peerCount: 1 }));
      expect(get(networkState).averageRtt).toBe(-1);
    });

    it('isConnected is false when signaling disconnected', () => {
      updateNetworkState(makeManagerState({ signaling: 'disconnected' }));
      expect(get(networkState).isConnected).toBe(false);
    });

    it('isConnected is false when no peers', () => {
      updateNetworkState(makeManagerState({ peerCount: 0, peers: new Map() }));
      expect(get(networkState).isConnected).toBe(false);
    });
  });

  describe('addNetworkEvent', () => {
    it('prepends events with timestamp', () => {
      addNetworkEvent({ type: 'peer-joined', message: 'Peer A joined' });
      addNetworkEvent({ type: 'peer-left', message: 'Peer B left' });

      const events = get(networkEvents);
      expect(events).toHaveLength(2);
      expect(events[0].message).toBe('Peer B left');
      expect(events[1].message).toBe('Peer A joined');
      expect(events[0].timestamp).toBeGreaterThan(0);
    });

    it('caps at 30 events', () => {
      for (let i = 0; i < 35; i++) {
        addNetworkEvent({ type: 'message', message: `msg-${i}` });
      }
      expect(get(networkEvents)).toHaveLength(30);
      // Most recent should be first
      expect(get(networkEvents)[0].message).toBe('msg-34');
    });

    it('includes optional peerId', () => {
      addNetworkEvent({ type: 'peer-joined', message: 'Joined', peerId: 'abc' });
      expect(get(networkEvents)[0].peerId).toBe('abc');
    });
  });

  describe('resetNetworkState', () => {
    it('restores all defaults', () => {
      updateNetworkState(makeManagerState());
      addNetworkEvent({ type: 'message', message: 'test' });

      resetNetworkState();

      const state = get(networkState);
      expect(state.signalingState).toBe('disconnected');
      expect(state.peerCount).toBe(0);
      expect(get(networkEvents)).toHaveLength(0);
    });
  });

  describe('signalingConnected', () => {
    it('is false when disconnected', () => {
      expect(get(signalingConnected)).toBe(false);
    });

    it('is true when connected', () => {
      updateNetworkState(makeManagerState({ signaling: 'connected' }));
      expect(get(signalingConnected)).toBe(true);
    });

    it('is false when connecting', () => {
      updateNetworkState(makeManagerState({ signaling: 'connecting' }));
      expect(get(signalingConnected)).toBe(false);
    });
  });

  describe('hasPeers', () => {
    it('is false when no peers', () => {
      expect(get(hasPeers)).toBe(false);
    });

    it('is true when peers exist', () => {
      updateNetworkState(makeManagerState());
      expect(get(hasPeers)).toBe(true);
    });
  });

  describe('connectionQuality', () => {
    it('is disconnected when no signaling', () => {
      expect(get(connectionQuality)).toBe('disconnected');
    });

    it('is disconnected when no peers', () => {
      updateNetworkState(
        makeManagerState({ signaling: 'connected', peerCount: 0, peers: new Map() })
      );
      expect(get(connectionQuality)).toBe('disconnected');
    });

    it('is good when connected with low RTT', () => {
      const peers = new Map<string, PeerInfo>([['p1', makePeer({ rtt: 50, state: 'connected' })]]);
      updateNetworkState(makeManagerState({ peers, peerCount: 1 }));
      expect(get(connectionQuality)).toBe('good');
    });

    it('is degraded when RTT is high', () => {
      const peers = new Map<string, PeerInfo>([['p1', makePeer({ rtt: 600, state: 'connected' })]]);
      updateNetworkState(makeManagerState({ peers, peerCount: 1 }));
      expect(get(connectionQuality)).toBe('degraded');
    });

    it('is degraded when less than half peers connected', () => {
      const peers = new Map<string, PeerInfo>([
        ['p1', makePeer({ id: 'p1', state: 'connected' })],
        ['p2', makePeer({ id: 'p2', state: 'disconnected' })],
        ['p3', makePeer({ id: 'p3', state: 'disconnected' })]
      ]);
      updateNetworkState(makeManagerState({ peers, peerCount: 3 }));
      expect(get(connectionQuality)).toBe('degraded');
    });
  });
});
