/**
 * VTP Network Layer
 *
 * Peer-to-peer networking for VTP nodes, consisting of:
 *
 * - **SignalingClient**: WebSocket client for Cloudflare Durable Object signaling
 * - **PeerConnection**: RTCPeerConnection wrapper with IPv6-prioritized ICE
 * - **PeerManager**: Multi-peer orchestrator (signaling + WebRTC + codec)
 * - **MessageCodec**: Protobuf encode/decode with Ed25519 signatures
 *
 * Architecture:
 * ```
 *   Signaling (WebSocket)          Transport (WebRTC DataChannel)
 *   ┌─────────────────┐            ┌──────────────────────────┐
 *   │  SignalingClient │            │  PeerConnection (×N)     │
 *   │  ─ CF DO relay   │  ──────►  │  ─ IPv6-prioritized ICE  │
 *   │  ─ SDP/ICE       │  setup    │  ─ Binary DataChannel    │
 *   └─────────────────┘            └──────────────────────────┘
 *              ▲                              │
 *              │                              ▼
 *   ┌─────────────────┐            ┌──────────────────────────┐
 *   │   PeerManager    │◄──────────│   MessageCodec           │
 *   │   ─ Lifecycle    │  events   │   ─ Protobuf encode      │
 *   │   ─ Routing      │           │   ─ Ed25519 sign/verify  │
 *   └─────────────────┘            └──────────────────────────┘
 * ```
 *
 * Usage:
 * ```typescript
 * import { PeerManager, initCodec } from '$lib/network';
 *
 * await initCodec();
 *
 * const manager = new PeerManager({
 *   signaling: { serverUrl: 'wss://signal.example.com', roomId: 'abc' },
 *   keypair: { publicKey: myPubkey, secretKey: mySecretkey },
 *   preferIpv6: true,
 * });
 *
 * manager.on('onMessage', (from, msg) => { ... });
 * await manager.connect();
 * ```
 */

// Core classes
export { SignalingClient } from './signaling-client';
export { PeerConnection } from './peer-connection';
export { PeerManager } from './peer-manager';
export { MessageCodec, initCodec } from './message-codec';

// Message helpers
export {
  createPing,
  createPong,
  createCheckpoint,
  createWinner,
  createPeerDiscovery,
  createVdfProgress,
  createConsensusProposal,
  createConsensusVote,
  createNewRound
} from './message-codec';

// Types
export type {
  SignalingConfig,
  SignalingState,
  SignalingEvents,
  PeerConnectionConfig,
  PeerState,
  PeerInfo,
  PeerManagerEvents,
  PeerManagerState,
  DecodedMessage,
  MessageType,
  MessagePayload,
  PingPayload,
  PongPayload,
  CheckpointPayload,
  WinnerPayload,
  PeerDiscoveryPayload,
  VdfProgressPayload,
  ConsensusProposalPayload,
  ConsensusVotePayload,
  NewRoundPayload,
  SigningKeyPair
} from './types';

export type { PeerConnectionEvents } from './peer-connection';
export type { PeerManagerConfig } from './peer-manager';
