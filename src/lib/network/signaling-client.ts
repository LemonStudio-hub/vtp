/**
 * Signaling Client
 *
 * Manages the WebSocket connection to a Cloudflare Durable Object
 * signaling server. Handles:
 * - Connection lifecycle (connect, reconnect, disconnect)
 * - Room join/leave
 * - SDP offer/answer relay
 * - ICE candidate relay
 *
 * The client is intentionally simple — it does not manage WebRTC
 * connections itself. It exposes events that the PeerManager listens
 * to for initiating WebRTC handshakes.
 *
 * @example
 * ```typescript
 * const client = new SignalingClient({
 *   serverUrl: 'wss://signal.example.com',
 *   roomId: 'challenge-abc123',
 * });
 *
 * client.on('onPeerJoined', (peerId) => {
 *   // Initiate WebRTC connection to this peer
 * });
 *
 * await client.connect();
 * ```
 */

import type { SignalingConfig, SignalingState, SignalingEvents } from './types';

/** Default configuration values. */
const DEFAULTS = {
  reconnectIntervalMs: 3000,
  maxReconnectAttempts: 10
};

export class SignalingClient {
  private config: Required<SignalingConfig>;
  private ws: WebSocket | null = null;
  private state: SignalingState = 'disconnected';
  private peerId: string | null = null;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private events: Partial<SignalingEvents> = {};

  constructor(config: SignalingConfig) {
    this.config = { ...DEFAULTS, ...config };
  }

  // ─── Public API ──────────────────────────────────────────────────

  /** Register event handlers. */
  on<K extends keyof SignalingEvents>(event: K, handler: SignalingEvents[K]): void {
    this.events[event] = handler;
  }

  /** Remove an event handler. */
  off<K extends keyof SignalingEvents>(event: K): void {
    delete this.events[event];
  }

  /** Get the assigned peer ID (available after connect). */
  getPeerId(): string | null {
    return this.peerId;
  }

  /** Get the current signaling state. */
  getState(): SignalingState {
    return this.state;
  }

  /**
   * Connect to the signaling server and join the configured room.
   *
   * Returns a promise that resolves when the WebSocket is open and
   * the room join is confirmed, or rejects on connection failure.
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.state === 'connected') {
        resolve();
        return;
      }

      this.setState('connecting');
      this.reconnectAttempts = 0;

      try {
        const wsUrl = this.buildWebSocketUrl();
        this.ws = new WebSocket(wsUrl);

        // Resolve on successful room join (handled in onMessage)
        let resolved = false;

        this.ws.onopen = () => {
          this.reconnectAttempts = 0;
          // WebSocket is open; waiting for 'welcome' + 'joined' from server
        };

        this.ws.onmessage = (event) => {
          try {
            const msg = JSON.parse(event.data);
            this.handleMessage(msg);

            // Resolve connect() after room join is confirmed
            if (!resolved && msg.type === 'joined') {
              resolved = true;
              resolve();
            }
          } catch {
            console.warn('[SignalingClient] Failed to parse message:', event.data);
          }
        };

        this.ws.onclose = (event) => {
          this.ws = null;
          this.peerId = null;

          if (this.state === 'connecting' && !resolved) {
            reject(new Error(`WebSocket closed during connect: ${event.code}`));
          }

          if (this.state !== 'disconnected') {
            this.scheduleReconnect();
          }
        };

        this.ws.onerror = () => {
          if (this.state === 'connecting' && !resolved) {
            reject(new Error('WebSocket connection failed'));
          }
        };
      } catch (err) {
        this.setState('failed');
        reject(err);
      }
    });
  }

  /** Disconnect from the signaling server. */
  disconnect(): void {
    this.clearReconnectTimer();

    if (this.ws) {
      this.setState('disconnected');
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }

    this.peerId = null;
  }

  /**
   * Send an SDP offer to a specific peer via the signaling server.
   */
  sendOffer(target: string, sdp: RTCSessionDescriptionInit): void {
    this.send({ type: 'offer', target, sdp });
  }

  /**
   * Send an SDP answer to a specific peer via the signaling server.
   */
  sendAnswer(target: string, sdp: RTCSessionDescriptionInit): void {
    this.send({ type: 'answer', target, sdp });
  }

  /**
   * Send an ICE candidate to a specific peer via the signaling server.
   */
  sendCandidate(target: string, candidate: RTCIceCandidateInit): void {
    this.send({ type: 'candidate', target, candidate });
  }

  // ─── Private ─────────────────────────────────────────────────────

  /** Build the WebSocket URL from config. */
  private buildWebSocketUrl(): string {
    const base = this.config.serverUrl.replace(/\/+$/, '');
    return `${base}/room/${this.config.roomId}/ws`;
  }

  /** Handle an incoming message from the signaling server. */
  private handleMessage(msg: Record<string, unknown>): void {
    switch (msg.type) {
      case 'welcome':
        this.peerId = msg.peerId as string;
        this.setState('connected');
        break;

      case 'joined':
        // Already handled in connect() promise
        break;

      case 'peer-joined':
        this.events.onPeerJoined?.(msg.peerId as string);
        break;

      case 'peer-left':
        this.events.onPeerLeft?.(msg.peerId as string);
        break;

      case 'offer':
        this.events.onOffer?.(msg.from as string, msg.sdp as RTCSessionDescriptionInit);
        break;

      case 'answer':
        this.events.onAnswer?.(msg.from as string, msg.sdp as RTCSessionDescriptionInit);
        break;

      case 'candidate':
        this.events.onCandidate?.(msg.from as string, msg.candidate as RTCIceCandidateInit);
        break;

      case 'error':
        console.warn(`[SignalingClient] Server error: ${msg.code} - ${msg.message}`);
        this.events.onError?.(msg.code as string, msg.message as string);
        break;
    }
  }

  /** Send a JSON message over the WebSocket. */
  private send(msg: Record<string, unknown>): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.warn('[SignalingClient] Cannot send — WebSocket not open');
      return;
    }
    this.ws.send(JSON.stringify(msg));
  }

  /** Update signaling state and notify listener. */
  private setState(state: SignalingState): void {
    if (this.state === state) return;
    this.state = state;
    this.events.onStateChange?.(state);
  }

  /** Schedule a reconnection attempt. */
  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      this.setState('failed');
      return;
    }

    this.setState('reconnecting');
    this.reconnectAttempts++;

    const delay = this.config.reconnectIntervalMs * Math.min(this.reconnectAttempts, 5);
    this.reconnectTimer = setTimeout(() => {
      this.connect().catch(() => {
        // Will retry via onclose → scheduleReconnect
      });
    }, delay);
  }

  /** Clear any pending reconnect timer. */
  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }
}
