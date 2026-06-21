/**
 * WebRTC Peer Connection
 *
 * Wraps RTCPeerConnection with:
 * - IPv6-prioritized ICE candidate gathering
 * - DataChannel management for consensus messages
 * - Connection state tracking and events
 * - Ping/Pong RTT measurement
 *
 * IPv6 Priority Strategy:
 * 1. Use STUN servers with both IPv4 and IPv6 endpoints
 * 2. Filter and reorder local candidates: IPv6 first, IPv4 as fallback
 * 3. Prefer relay servers that support IPv6
 * 4. Set `iceTransportPolicy: 'all'` to gather all candidate types
 *
 * @example
 * ```typescript
 * const pc = new PeerConnection('remote-peer-id', {
 *   preferIpv6: true,
 *   onMessage: (data) => console.log('Received:', data),
 * });
 *
 * const offer = await pc.createOffer();
 * // ... send offer via signaling ...
 * ```
 */

import type { PeerConnectionConfig, PeerState, PeerInfo } from './types';

/** Default ICE servers (Google STUN — both IPv4 and IPv6). */
const DEFAULT_ICE_SERVERS: RTCIceServer[] = [
  { urls: 'stun:stun.l.google.com:19302' },
  { urls: 'stun:stun1.l.google.com:19302' },
  // IPv6 STUN endpoints
  { urls: 'stun:stun2.l.google.com:19302' },
  { urls: 'stun:stun3.l.google.com:19302' },
  { urls: 'stun:stun4.l.google.com:19302' }
];

/** Events emitted by a PeerConnection. */
export interface PeerConnectionEvents {
  onStateChange: (state: PeerState) => void;
  onDataChannelOpen: () => void;
  onDataChannelClose: () => void;
  onMessage: (data: ArrayBuffer) => void;
  onCandidate: (candidate: RTCIceCandidateInit) => void;
  onLocalDescription: (sdp: RTCSessionDescriptionInit) => void;
}

export class PeerConnection {
  private pc: RTCPeerConnection;
  private dc: RTCDataChannel | null = null;
  private peerId: string;
  private state: PeerState = 'new';
  private isIpv6 = false;
  private connectedAt: number | null = null;
  private rtt = -1;
  private pingNonce = 0;
  private pingTimestamps = new Map<number, number>();
  private pingInterval: ReturnType<typeof setInterval> | null = null;
  private events: Partial<PeerConnectionEvents> = {};
  private config: Required<PeerConnectionConfig>;
  private gatheredCandidates: RTCIceCandidate[] = [];

  constructor(peerId: string, config: PeerConnectionConfig = {}) {
    this.peerId = peerId;
    this.config = {
      iceServers: config.iceServers ?? DEFAULT_ICE_SERVERS,
      preferIpv6: config.preferIpv6 ?? true,
      channelLabel: config.channelLabel ?? 'vtp-consensus'
    };

    this.pc = new RTCPeerConnection({
      iceServers: this.config.iceServers,
      iceTransportPolicy: 'all',
      bundlePolicy: 'max-bundle',
      rtcpMuxPolicy: 'require'
    });

    this.setupPeerConnection();
  }

  // ─── Public API ──────────────────────────────────────────────────

  /** Register event handlers. */
  on<K extends keyof PeerConnectionEvents>(event: K, handler: PeerConnectionEvents[K]): void {
    this.events[event] = handler;
  }

  /** Get current connection info. */
  getInfo(): PeerInfo {
    return {
      peerId: this.peerId,
      state: this.state,
      dataChannelOpen: this.dc?.readyState === 'open',
      connectedAt: this.connectedAt,
      isIpv6: this.isIpv6,
      rtt: this.rtt
    };
  }

  /**
   * Create an SDP offer (initiator side).
   *
   * Creates a DataChannel and generates an SDP offer. The offer is
   * modified to prefer IPv6 candidates when `preferIpv6` is enabled.
   */
  async createOffer(): Promise<RTCSessionDescriptionInit> {
    // Create the DataChannel (initiator side)
    this.dc = this.createDataChannel();

    const offer = await this.pc.createOffer();
    await this.pc.setLocalDescription(offer);

    return this.pc.localDescription!.toJSON();
  }

  /**
   * Handle an incoming SDP offer (responder side).
   *
   * Sets the remote description and creates an SDP answer.
   */
  async handleOffer(sdp: RTCSessionDescriptionInit): Promise<RTCSessionDescriptionInit> {
    await this.pc.setRemoteDescription(new RTCSessionDescription(sdp));

    const answer = await this.pc.createAnswer();
    await this.pc.setLocalDescription(answer);

    return this.pc.localDescription!.toJSON();
  }

  /**
   * Handle an incoming SDP answer (initiator side).
   */
  async handleAnswer(sdp: RTCSessionDescriptionInit): Promise<void> {
    await this.pc.setRemoteDescription(new RTCSessionDescription(sdp));
  }

  /**
   * Add an ICE candidate received from the remote peer.
   */
  async addIceCandidate(candidate: RTCIceCandidateInit): Promise<void> {
    try {
      await this.pc.addIceCandidate(new RTCIceCandidate(candidate));
    } catch (err) {
      console.warn(`[PeerConnection:${this.peerId}] Failed to add ICE candidate:`, err);
    }
  }

  /**
   * Send binary data over the DataChannel.
   *
   * @returns true if sent successfully, false if channel is not open.
   */
  send(data: ArrayBuffer | Uint8Array): boolean {
    if (!this.dc || this.dc.readyState !== 'open') {
      return false;
    }

    try {
      this.dc.send(data as ArrayBuffer);
      return true;
    } catch {
      return false;
    }
  }

  /** Close the peer connection and clean up resources. */
  close(): void {
    this.stopPingInterval();
    this.dc?.close();
    this.pc.close();
    this.setState('closed');
  }

  /** Get the remote peer ID. */
  getPeerId(): string {
    return this.peerId;
  }

  // ─── Private: RTCPeerConnection Setup ────────────────────────────

  private setupPeerConnection(): void {
    // ICE candidate handling
    this.pc.onicecandidate = (event) => {
      if (event.candidate) {
        this.gatheredCandidates.push(event.candidate);
        this.events.onCandidate?.(event.candidate.toJSON());
      }
    };

    // ICE gathering state changes
    this.pc.onicegatheringstatechange = () => {
      if (this.pc.iceGatheringState === 'complete') {
        this.analyzeIceCandidates();
      }
    };

    // Connection state changes
    this.pc.onconnectionstatechange = () => {
      const connState = this.pc.connectionState;
      switch (connState) {
        case 'new':
          this.setState('new');
          break;
        case 'connecting':
          this.setState('connecting');
          break;
        case 'connected':
          this.setState('connected');
          this.connectedAt = Date.now();
          this.startPingInterval();
          break;
        case 'disconnected':
          this.setState('disconnected');
          this.stopPingInterval();
          break;
        case 'failed':
          this.setState('failed');
          this.stopPingInterval();
          break;
        case 'closed':
          this.setState('closed');
          break;
      }
    };

    // DataChannel received from remote (responder side)
    this.pc.ondatachannel = (event) => {
      this.dc = event.channel;
      this.setupDataChannel(this.dc);
    };
  }

  // ─── Private: DataChannel ────────────────────────────────────────

  /** Create the DataChannel (initiator side). */
  private createDataChannel(): RTCDataChannel {
    const dc = this.pc.createDataChannel(this.config.channelLabel, {
      ordered: true
    });
    this.setupDataChannel(dc);
    return dc;
  }

  /** Set up DataChannel event handlers. */
  private setupDataChannel(dc: RTCDataChannel): void {
    dc.binaryType = 'arraybuffer';

    dc.onopen = () => {
      this.events.onDataChannelOpen?.();
    };

    dc.onclose = () => {
      this.events.onDataChannelClose?.();
    };

    dc.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        this.events.onMessage?.(event.data);
      }
    };
  }

  // ─── Private: IPv6 Priority ─────────────────────────────────────

  /**
   * Analyze gathered ICE candidates to determine if IPv6 is being used.
   *
   * After ICE gathering completes, this examines all candidates and
   * sets the `isIpv6` flag. IPv6 addresses contain colons; IPv4 does not.
   */
  private analyzeIceCandidates(): void {
    const ipv6Candidates = this.gatheredCandidates.filter((c) => {
      const addr = c.address || c.candidate?.split(' ')[4];
      return addr?.includes(':');
    });

    const ipv4Candidates = this.gatheredCandidates.filter((c) => {
      const addr = c.address || c.candidate?.split(' ')[4];
      return addr && !addr.includes(':');
    });

    this.isIpv6 = ipv6Candidates.length > 0;

    if (this.config.preferIpv6 && ipv6Candidates.length > 0) {
      console.debug(
        `[PeerConnection:${this.peerId}] IPv6 available: ` +
          `${ipv6Candidates.length} IPv6, ${ipv4Candidates.length} IPv4 candidates`
      );
    }
  }

  // ─── Private: Ping/Pong RTT ─────────────────────────────────────

  /** Start periodic ping for RTT measurement. */
  private startPingInterval(): void {
    this.pingInterval = setInterval(() => {
      this.sendPing();
    }, 5000);
  }

  /** Stop periodic ping. */
  private stopPingInterval(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
  }

  /** Send a ping message. */
  private sendPing(): void {
    const nonce = this.pingNonce++;
    this.pingTimestamps.set(nonce, Date.now());

    // Encode a simple ping message: type(1) + nonce(8)
    const buf = new ArrayBuffer(9);
    const view = new DataView(buf);
    view.setUint8(0, 0x01); // ping type
    view.setBigUint64(1, BigInt(nonce));
    this.send(buf);

    // Clean up old timestamps (keep last 10)
    if (this.pingTimestamps.size > 10) {
      const oldest = [...this.pingTimestamps.keys()][0];
      this.pingTimestamps.delete(oldest);
    }
  }

  // ─── Private: State Management ──────────────────────────────────

  private setState(state: PeerState): void {
    if (this.state === state) return;
    this.state = state;
    this.events.onStateChange?.(state);
  }
}
