/**
 * Peer Manager
 *
 * Orchestrates the full networking lifecycle:
 * 1. Connects to the signaling server (SignalingClient)
 * 2. Initiates WebRTC connections to new peers (PeerConnection)
 * 3. Manages multiple concurrent peer connections
 * 4. Routes incoming messages through the MessageCodec
 * 5. Broadcasts messages to all connected peers
 *
 * Polite peer model: when two peers discover each other, the peer with
 * the lexicographically smaller ID creates the offer. This prevents
 * glare (both peers sending offers simultaneously).
 *
 * @example
 * ```typescript
 * const manager = new PeerManager({
 *   signaling: { serverUrl: 'wss://signal.example.com', roomId: 'abc' },
 *   preferIpv6: true,
 *   keypair: { publicKey: ..., secretKey: ... },
 * });
 *
 * manager.on('onMessage', (from, msg) => {
 *   console.log(`Received ${msg.type} from ${from}`);
 * });
 *
 * await manager.connect();
 * ```
 */

import { SignalingClient } from './signaling-client';
import { PeerConnection } from './peer-connection';
import { MessageCodec, initCodec } from './message-codec';
import type { CryptoProvider } from '$lib/consensus/crypto-provider';
import type {
  SignalingConfig,
  PeerConnectionConfig,
  PeerInfo,
  PeerManagerEvents,
  PeerManagerState,
  MessagePayload,
  SigningKeyPair
} from './types';

/** Configuration for the PeerManager. */
export interface PeerManagerConfig {
  /** Signaling server configuration. */
  signaling: SignalingConfig;
  /** Ed25519 keypair for message signing. */
  keypair: SigningKeyPair;
  /** Cryptographic operations provider for signing/verification. */
  crypto: CryptoProvider;
  /** Whether to prefer IPv6 (default: true). */
  preferIpv6?: boolean;
  /** ICE servers override. */
  iceServers?: RTCIceServer[];
  /** DataChannel label (default: "vtp-consensus"). */
  channelLabel?: string;
}

export class PeerManager {
  private signaling: SignalingClient;
  private codec: MessageCodec;
  private peers = new Map<string, PeerConnection>();
  private localPeerId: string | null = null;
  private events: Partial<PeerManagerEvents> = {};
  private config: PeerManagerConfig;
  private codecReady = false;

  constructor(config: PeerManagerConfig) {
    this.config = config;

    // Initialize signaling client
    this.signaling = new SignalingClient(config.signaling);

    // Initialize message codec
    this.codec = new MessageCodec(config.keypair, config.crypto);

    // Wire up signaling events
    this.setupSignalingEvents();
  }

  // ─── Public API ──────────────────────────────────────────────────

  /** Register event handlers. */
  on<K extends keyof PeerManagerEvents>(event: K, handler: PeerManagerEvents[K]): void {
    this.events[event] = handler;
  }

  /** Remove an event handler. */
  off<K extends keyof PeerManagerEvents>(event: K): void {
    delete this.events[event];
  }

  /**
   * Connect to the signaling server and start accepting peer connections.
   *
   * Initializes the Protobuf codec, then connects to signaling.
   */
  async connect(): Promise<void> {
    // Initialize Protobuf codec
    if (!this.codecReady) {
      await initCodec();
      this.codecReady = true;
    }

    // Connect to signaling server
    await this.signaling.connect();
    this.localPeerId = this.signaling.getPeerId();
  }

  /** Disconnect from signaling and close all peer connections. */
  disconnect(): void {
    this.signaling.disconnect();

    for (const [, peer] of this.peers) {
      peer.close();
    }
    this.peers.clear();

    this.localPeerId = null;
    this.emitStateChange();
  }

  /**
   * Broadcast a message to all connected peers.
   *
   * @param payload - The message payload to broadcast.
   * @returns Number of peers the message was sent to.
   */
  broadcast(payload: MessagePayload): number {
    const encoded = this.codec.encode(payload);
    let sent = 0;

    for (const [, peer] of this.peers) {
      if (peer.send(encoded)) {
        sent++;
      }
    }

    return sent;
  }

  /**
   * Send a message to a specific peer.
   *
   * @param peerId - Target peer ID.
   * @param payload - The message payload.
   * @returns true if sent successfully.
   */
  sendTo(peerId: string, payload: MessagePayload): boolean {
    const peer = this.peers.get(peerId);
    if (!peer) return false;

    const encoded = this.codec.encode(payload);
    return peer.send(encoded);
  }

  /** Get the current peer manager state. */
  getState(): PeerManagerState {
    const peers = new Map<string, PeerInfo>();
    for (const [id, peer] of this.peers) {
      peers.set(id, peer.getInfo());
    }

    return {
      signaling: this.signaling.getState(),
      peerCount: this.peers.size,
      peers,
      localPeerId: this.localPeerId
    };
  }

  /** Get info about a specific peer. */
  getPeer(peerId: string): PeerInfo | null {
    return this.peers.get(peerId)?.getInfo() ?? null;
  }

  /** Get the number of connected peers. */
  getPeerCount(): number {
    return this.peers.size;
  }

  // ─── Private: Signaling Events ──────────────────────────────────

  private setupSignalingEvents(): void {
    this.signaling.on('onStateChange', (_state) => {
      this.emitStateChange();
    });

    this.signaling.on('onPeerJoined', (peerId) => {
      this.handlePeerJoined(peerId);
    });

    this.signaling.on('onPeerLeft', (peerId) => {
      this.handlePeerLeft(peerId);
    });

    this.signaling.on('onOffer', (from, sdp) => {
      this.handleOffer(from, sdp);
    });

    this.signaling.on('onAnswer', (from, sdp) => {
      this.handleAnswer(from, sdp);
    });

    this.signaling.on('onCandidate', (from, candidate) => {
      this.handleCandidate(from, candidate);
    });
  }

  // ─── Private: WebRTC Handshake ──────────────────────────────────

  /**
   * Handle a new peer joining the room.
   *
   * Uses the polite peer model: the peer with the smaller ID creates
   * the offer to prevent glare.
   */
  private handlePeerJoined(peerId: string): void {
    if (this.peers.has(peerId)) return;

    // Polite peer: smaller ID creates the offer
    if (this.localPeerId && this.localPeerId < peerId) {
      this.createOfferToPeer(peerId);
    }
    // Otherwise, wait for the remote peer's offer
  }

  /** Handle a peer leaving the room. */
  private handlePeerLeft(peerId: string): void {
    const peer = this.peers.get(peerId);
    if (peer) {
      peer.close();
      this.peers.delete(peerId);
      this.events.onPeerDisconnected?.(peerId);
      this.emitStateChange();
    }
  }

  /** Create and send an SDP offer to a peer. */
  private async createOfferToPeer(peerId: string): Promise<void> {
    const peer = this.createPeerConnection(peerId);

    try {
      const offer = await peer.createOffer();
      this.signaling.sendOffer(peerId, offer);
    } catch (err) {
      console.error(`[PeerManager] Failed to create offer for ${peerId}:`, err);
      this.removePeer(peerId);
    }
  }

  /** Handle an incoming SDP offer. */
  private async handleOffer(from: string, sdp: RTCSessionDescriptionInit): Promise<void> {
    let peer = this.peers.get(from);
    if (!peer) {
      peer = this.createPeerConnection(from);
    }

    try {
      const answer = await peer.handleOffer(sdp);
      this.signaling.sendAnswer(from, answer);
    } catch (err) {
      console.error(`[PeerManager] Failed to handle offer from ${from}:`, err);
      this.removePeer(from);
    }
  }

  /** Handle an incoming SDP answer. */
  private async handleAnswer(from: string, sdp: RTCSessionDescriptionInit): Promise<void> {
    const peer = this.peers.get(from);
    if (!peer) {
      console.warn(`[PeerManager] Received answer from unknown peer: ${from}`);
      return;
    }

    try {
      await peer.handleAnswer(sdp);
    } catch (err) {
      console.error(`[PeerManager] Failed to handle answer from ${from}:`, err);
      this.removePeer(from);
    }
  }

  /** Handle an incoming ICE candidate. */
  private async handleCandidate(from: string, candidate: RTCIceCandidateInit): Promise<void> {
    const peer = this.peers.get(from);
    if (!peer) {
      console.warn(`[PeerManager] Received candidate from unknown peer: ${from}`);
      return;
    }

    await peer.addIceCandidate(candidate);
  }

  // ─── Private: PeerConnection Management ─────────────────────────

  /** Create a new PeerConnection and wire up events. */
  private createPeerConnection(peerId: string): PeerConnection {
    const peerConfig: PeerConnectionConfig = {
      preferIpv6: this.config.preferIpv6 ?? true,
      channelLabel: this.config.channelLabel ?? 'vtp-consensus',
      ...(this.config.iceServers ? { iceServers: this.config.iceServers } : {})
    };

    const peer = new PeerConnection(peerId, peerConfig);

    // Wire up events
    peer.on('onStateChange', (state) => {
      this.emitStateChange();

      if (state === 'connected') {
        this.events.onPeerConnected?.(peerId, peer.getInfo());
      } else if (state === 'disconnected' || state === 'failed' || state === 'closed') {
        if (state !== 'disconnected') {
          // Only remove on failed/closed; disconnected may recover
          this.events.onPeerDisconnected?.(peerId);
        }
      }
    });

    peer.on('onCandidate', (candidate) => {
      this.signaling.sendCandidate(peerId, candidate);
    });

    peer.on('onMessage', (data) => {
      this.handlePeerMessage(peerId, data);
    });

    this.peers.set(peerId, peer);
    this.emitStateChange();

    return peer;
  }

  /** Remove a peer connection. */
  private removePeer(peerId: string): void {
    const peer = this.peers.get(peerId);
    if (peer) {
      peer.close();
      this.peers.delete(peerId);
      this.emitStateChange();
    }
  }

  // ─── Private: Message Handling ───────────────────────────────────

  /** Handle a binary message received from a peer. */
  private handlePeerMessage(from: string, data: ArrayBuffer): void {
    const decoded = this.codec.decode(data);
    if (!decoded) {
      console.warn(`[PeerManager] Failed to decode message from ${from}`);
      return;
    }

    // Handle ping/pong internally
    if (decoded.type === 'ping') {
      this.sendTo(from, { type: 'pong', nonce: (decoded.payload as { nonce: number }).nonce });
      return;
    }

    // Forward all other messages to the application
    this.events.onMessage?.(from, decoded);
  }

  // ─── Private: State ─────────────────────────────────────────────

  private emitStateChange(): void {
    this.events.onStateChange?.(this.getState());
  }
}
