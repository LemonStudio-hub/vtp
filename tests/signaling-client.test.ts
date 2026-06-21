/**
 * Signaling Client Tests
 *
 * Tests for the WebSocket signaling client using a mock WebSocket.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { SignalingClient } from '../src/lib/network/signaling-client';
import type { SignalingState } from '../src/lib/network/types';

/**
 * Mock WebSocket for testing.
 *
 * Simulates the WebSocket API without actual network connections.
 * Call `mockServerMessage()` to simulate incoming messages from the server.
 */
class MockWebSocket {
  static instances: MockWebSocket[] = [];

  readyState = 1; // OPEN
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  sentMessages: string[] = [];
  url: string;

  constructor(url: string) {
    this.url = url;
    MockWebSocket.instances.push(this);

    // Simulate async open
    setTimeout(() => {
      this.readyState = 1;
      this.onopen?.(new Event('open'));
    }, 0);
  }

  send(data: string): void {
    this.sentMessages.push(data);
  }

  close(code?: number, reason?: string): void {
    this.readyState = 3; // CLOSED
    this.onclose?.(new CloseEvent('close', { code: code ?? 1000, reason }));
  }

  /** Simulate a message from the signaling server. */
  mockServerMessage(data: Record<string, unknown>): void {
    this.onmessage?.(new MessageEvent('message', { data: JSON.stringify(data) }));
  }

  /** Simulate a connection error. */
  mockError(): void {
    this.onerror?.(new Event('error'));
  }

  /** Get the last sent message parsed as JSON. */
  lastSentMessage(): Record<string, unknown> | null {
    if (this.sentMessages.length === 0) return null;
    return JSON.parse(this.sentMessages[this.sentMessages.length - 1]);
  }
}

// Replace global WebSocket with mock
const originalWebSocket = globalThis.WebSocket;

describe('SignalingClient', () => {
  let client: SignalingClient;

  beforeEach(() => {
    MockWebSocket.instances = [];
    // @ts-expect-error - Mock WebSocket
    globalThis.WebSocket = MockWebSocket;
    // Ensure WebSocket constants are available
    MockWebSocket.CONNECTING = 0;
    MockWebSocket.OPEN = 1;
    MockWebSocket.CLOSING = 2;
    MockWebSocket.CLOSED = 3;
  });

  afterEach(() => {
    globalThis.WebSocket = originalWebSocket;
    client?.disconnect();
  });

  function getLastMock(): MockWebSocket {
    return MockWebSocket.instances[MockWebSocket.instances.length - 1];
  }

  describe('Connection', () => {
    it('should connect to the signaling server', async () => {
      client = new SignalingClient({
        serverUrl: 'ws://localhost:8787',
        roomId: 'test-room'
      });

      const connectPromise = client.connect();
      const mock = getLastMock();

      // Server sends welcome
      mock.mockServerMessage({ type: 'welcome', peerId: 'p-0' });
      // Server sends joined confirmation
      mock.mockServerMessage({ type: 'joined', peerId: 'p-0', peers: [] });

      await connectPromise;

      expect(client.getState()).toBe('connected');
      expect(client.getPeerId()).toBe('p-0');
    });

    it('should build correct WebSocket URL', async () => {
      client = new SignalingClient({
        serverUrl: 'wss://signal.example.com/',
        roomId: 'my-room'
      });

      const connectPromise = client.connect();
      const mock = getLastMock();

      expect(mock.url).toBe('wss://signal.example.com/room/my-room/ws');

      mock.mockServerMessage({ type: 'welcome', peerId: 'p-0' });
      mock.mockServerMessage({ type: 'joined', peerId: 'p-0', peers: [] });

      await connectPromise;
    });

    it('should reject on connection failure', async () => {
      client = new SignalingClient({
        serverUrl: 'ws://invalid-host',
        roomId: 'test',
        maxReconnectAttempts: 0
      });

      const connectPromise = client.connect();
      const mock = getLastMock();

      // Simulate error before welcome
      mock.mockError();
      mock.close(1006);

      await expect(connectPromise).rejects.toThrow();
    });
  });

  describe('Events', () => {
    it('should emit onPeerJoined when a peer joins', async () => {
      client = new SignalingClient({
        serverUrl: 'ws://localhost:8787',
        roomId: 'test-room'
      });

      const peerJoined = vi.fn();
      client.on('onPeerJoined', peerJoined);

      const connectPromise = client.connect();
      const mock = getLastMock();

      mock.mockServerMessage({ type: 'welcome', peerId: 'p-0' });
      mock.mockServerMessage({ type: 'joined', peerId: 'p-0', peers: [] });
      await connectPromise;

      // Simulate another peer joining
      mock.mockServerMessage({ type: 'peer-joined', peerId: 'p-1' });

      expect(peerJoined).toHaveBeenCalledWith('p-1');
    });

    it('should emit onPeerLeft when a peer leaves', async () => {
      client = new SignalingClient({
        serverUrl: 'ws://localhost:8787',
        roomId: 'test-room'
      });

      const peerLeft = vi.fn();
      client.on('onPeerLeft', peerLeft);

      const connectPromise = client.connect();
      const mock = getLastMock();

      mock.mockServerMessage({ type: 'welcome', peerId: 'p-0' });
      mock.mockServerMessage({ type: 'joined', peerId: 'p-0', peers: ['p-1'] });
      await connectPromise;

      mock.mockServerMessage({ type: 'peer-left', peerId: 'p-1' });

      expect(peerLeft).toHaveBeenCalledWith('p-1');
    });

    it('should emit onOffer when receiving an SDP offer', async () => {
      client = new SignalingClient({
        serverUrl: 'ws://localhost:8787',
        roomId: 'test-room'
      });

      const onOffer = vi.fn();
      client.on('onOffer', onOffer);

      const connectPromise = client.connect();
      const mock = getLastMock();

      mock.mockServerMessage({ type: 'welcome', peerId: 'p-0' });
      mock.mockServerMessage({ type: 'joined', peerId: 'p-0', peers: [] });
      await connectPromise;

      const sdp = { type: 'offer' as const, sdp: 'v=0\r\n...' };
      mock.mockServerMessage({ type: 'offer', from: 'p-1', sdp });

      expect(onOffer).toHaveBeenCalledWith('p-1', sdp);
    });

    it('should emit onStateChange on state transitions', async () => {
      client = new SignalingClient({
        serverUrl: 'ws://localhost:8787',
        roomId: 'test-room'
      });

      const states: SignalingState[] = [];
      client.on('onStateChange', (state) => states.push(state));

      const connectPromise = client.connect();
      const mock = getLastMock();

      mock.mockServerMessage({ type: 'welcome', peerId: 'p-0' });
      mock.mockServerMessage({ type: 'joined', peerId: 'p-0', peers: [] });
      await connectPromise;

      expect(states).toContain('connecting');
      expect(states).toContain('connected');
    });
  });

  describe('Message Sending', () => {
    it('should send SDP offers to the server', async () => {
      client = new SignalingClient({
        serverUrl: 'ws://localhost:8787',
        roomId: 'test-room'
      });

      const connectPromise = client.connect();
      const mock = getLastMock();

      mock.mockServerMessage({ type: 'welcome', peerId: 'p-0' });
      mock.mockServerMessage({ type: 'joined', peerId: 'p-0', peers: [] });
      await connectPromise;

      const sdp = { type: 'offer' as const, sdp: 'v=0\r\n...' };
      client.sendOffer('p-1', sdp);

      expect(mock.sentMessages.length).toBeGreaterThan(0);
      const lastMsg = mock.lastSentMessage();
      expect(lastMsg?.type).toBe('offer');
      expect(lastMsg?.target).toBe('p-1');
    });

    it('should send ICE candidates to the server', async () => {
      client = new SignalingClient({
        serverUrl: 'ws://localhost:8787',
        roomId: 'test-room'
      });

      const connectPromise = client.connect();
      const mock = getLastMock();

      mock.mockServerMessage({ type: 'welcome', peerId: 'p-0' });
      mock.mockServerMessage({ type: 'joined', peerId: 'p-0', peers: [] });
      await connectPromise;

      const candidate = { candidate: 'candidate:1 1 UDP 2122252543 ...' };
      client.sendCandidate('p-1', candidate);

      const lastMsg = mock.lastSentMessage();
      expect(lastMsg?.type).toBe('candidate');
      expect(lastMsg?.target).toBe('p-1');
    });
  });

  describe('Disconnect', () => {
    it('should clean up on disconnect', async () => {
      client = new SignalingClient({
        serverUrl: 'ws://localhost:8787',
        roomId: 'test-room'
      });

      const connectPromise = client.connect();
      const mock = getLastMock();

      mock.mockServerMessage({ type: 'welcome', peerId: 'p-0' });
      mock.mockServerMessage({ type: 'joined', peerId: 'p-0', peers: [] });
      await connectPromise;

      client.disconnect();

      expect(client.getState()).toBe('disconnected');
      expect(client.getPeerId()).toBeNull();
    });
  });
});
