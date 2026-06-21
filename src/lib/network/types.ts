/**
 * Network Layer Type Definitions
 *
 * Shared types for the VTP networking layer, covering signaling,
 * WebRTC peer connections, and consensus message exchange.
 */

// ─── Signaling ────────────────────────────────────────────────────

/** Configuration for connecting to a signaling server. */
export interface SignalingConfig {
  /** WebSocket URL of the signaling server (e.g., "wss://signal.example.com"). */
  serverUrl: string;
  /** Room ID to join (typically the VDF challenge ID). */
  roomId: string;
  /** Reconnect interval in ms (default: 3000). */
  reconnectIntervalMs?: number;
  /** Max reconnect attempts before giving up (default: 10). */
  maxReconnectAttempts?: number;
}

/** Connection state of the signaling WebSocket. */
export type SignalingState =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'failed';

/** Events emitted by the SignalingClient. */
export interface SignalingEvents {
  onStateChange: (state: SignalingState) => void;
  onPeerJoined: (peerId: string) => void;
  onPeerLeft: (peerId: string) => void;
  onOffer: (from: string, sdp: RTCSessionDescriptionInit) => void;
  onAnswer: (from: string, sdp: RTCSessionDescriptionInit) => void;
  onCandidate: (from: string, candidate: RTCIceCandidateInit) => void;
  onError: (code: string, message: string) => void;
}

// ─── WebRTC ───────────────────────────────────────────────────────

/** Configuration for RTCPeerConnection. */
export interface PeerConnectionConfig {
  /** ICE servers for STUN/TURN. */
  iceServers?: RTCIceServer[];
  /** Whether to prefer IPv6 candidates (default: true). */
  preferIpv6?: boolean;
  /** DataChannel label (default: "vtp-consensus"). */
  channelLabel?: string;
}

/** Connection state of a single peer. */
export type PeerState = 'new' | 'connecting' | 'connected' | 'disconnected' | 'failed' | 'closed';

/** Information about a connected peer. */
export interface PeerInfo {
  /** Remote peer ID assigned by the signaling server. */
  peerId: string;
  /** Current connection state. */
  state: PeerState;
  /** Whether the DataChannel is open and ready. */
  dataChannelOpen: boolean;
  /** Connection establishment timestamp. */
  connectedAt: number | null;
  /** Whether the connection uses IPv6. */
  isIpv6: boolean;
  /** Round-trip time in ms (measured via Ping/Pong, -1 if unknown). */
  rtt: number;
}

// ─── Message Codec ────────────────────────────────────────────────

/** Decoded consensus message (after signature verification). */
export interface DecodedMessage {
  /** Message type discriminator. */
  type: MessageType;
  /** Sender's 32-byte Ed25519 public key. */
  senderPubkey: Uint8Array;
  /** Timestamp (Unix epoch ms). */
  timestamp: number;
  /** Type-specific payload. */
  payload: MessagePayload;
}

/** Consensus message types. */
export type MessageType =
  | 'ping'
  | 'pong'
  | 'checkpoint'
  | 'winner'
  | 'peer-discovery'
  | 'vdf-progress'
  | 'consensus-proposal'
  | 'consensus-vote'
  | 'new-round';

/** Union of all message payloads. */
export type MessagePayload =
  | PingPayload
  | PongPayload
  | CheckpointPayload
  | WinnerPayload
  | PeerDiscoveryPayload
  | VdfProgressPayload
  | ConsensusProposalPayload
  | ConsensusVotePayload
  | NewRoundPayload;

export interface PingPayload {
  type: 'ping';
  nonce: number;
}

export interface PongPayload {
  type: 'pong';
  nonce: number;
}

export interface CheckpointPayload {
  type: 'checkpoint';
  step: number;
  vdfState: Uint8Array;
  vrfProof: Uint8Array;
  senderPubkey: Uint8Array;
}

export interface WinnerPayload {
  type: 'winner';
  step: number;
  vrfProof: Uint8Array;
  senderPubkey: Uint8Array;
}

export interface PeerDiscoveryPayload {
  type: 'peer-discovery';
  nodeId: string;
  publicKey: Uint8Array;
  addresses: string[];
}

export interface VdfProgressPayload {
  type: 'vdf-progress';
  step: number;
  speed: number;
}

export interface ConsensusProposalPayload {
  type: 'consensus-proposal';
  round: number;
  vdfState: Uint8Array;
  vrfProof: Uint8Array;
  blockHash: Uint8Array;
  prevBlockHash: Uint8Array;
  timestamp: number;
}

export interface ConsensusVotePayload {
  type: 'consensus-vote';
  round: number;
  phase: 'prevote' | 'precommit';
  blockHash: Uint8Array | null;
  voterPubkey: Uint8Array;
}

export interface NewRoundPayload {
  type: 'new-round';
  round: number;
  seed: Uint8Array;
  validators: Uint8Array[];
}

/** Key pair for message signing/verification. */
export interface SigningKeyPair {
  /** 32-byte Ed25519 public key. */
  publicKey: Uint8Array;
  /** 32-byte Ed25519 secret key. */
  secretKey: Uint8Array;
}

// ─── Peer Manager ─────────────────────────────────────────────────

/** Events emitted by the PeerManager. */
export interface PeerManagerEvents {
  onPeerConnected: (peerId: string, info: PeerInfo) => void;
  onPeerDisconnected: (peerId: string) => void;
  onMessage: (from: string, message: DecodedMessage) => void;
  onStateChange: (state: PeerManagerState) => void;
}

/** Aggregate state of the peer manager. */
export interface PeerManagerState {
  /** Signaling connection state. */
  signaling: SignalingState;
  /** Number of connected peers. */
  peerCount: number;
  /** Map of peer ID → PeerInfo. */
  peers: Map<string, PeerInfo>;
  /** This node's peer ID (assigned by signaling server). */
  localPeerId: string | null;
}
