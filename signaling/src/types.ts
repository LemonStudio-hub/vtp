/**
 * Signaling Protocol Types
 *
 * Message types exchanged between browser nodes and the Durable Object
 * over WebSocket during the WebRTC signaling phase.
 *
 * Flow:
 * 1. Node connects via WebSocket → DO assigns peerId, sends "welcome"
 * 2. Node sends "join" with roomId → DO adds to room, notifies peers
 * 3. Nodes exchange SDP offers/answers and ICE candidates through the DO
 * 4. Once WebRTC DataChannel is established, signaling WebSocket is optional
 */

// ─── Client → DO messages ─────────────────────────────────────────

/** Request to join a signaling room. */
export interface JoinMessage {
  type: 'join';
  /** Target room ID (typically the VDF challenge ID). */
  roomId: string;
}

/** SDP offer relay to a specific peer. */
export interface OfferMessage {
  type: 'offer';
  /** Target peer ID. */
  target: string;
  /** SDP offer. */
  sdp: RTCSessionDescriptionInit;
}

/** SDP answer relay to a specific peer. */
export interface AnswerMessage {
  type: 'answer';
  /** Target peer ID. */
  target: string;
  /** SDP answer. */
  sdp: RTCSessionDescriptionInit;
}

/** ICE candidate relay to a specific peer. */
export interface CandidateMessage {
  type: 'candidate';
  /** Target peer ID. */
  target: string;
  /** ICE candidate. */
  candidate: RTCIceCandidateInit;
}

/** Graceful leave notification. */
export interface LeaveMessage {
  type: 'leave';
}

export type ClientMessage =
  | JoinMessage
  | OfferMessage
  | AnswerMessage
  | CandidateMessage
  | LeaveMessage;

// ─── DO → Client messages ─────────────────────────────────────────

/** Sent immediately after WebSocket open — assigns peer identity. */
export interface WelcomeMessage {
  type: 'welcome';
  /** Assigned peer ID (unique within the DO instance). */
  peerId: string;
}

/** Confirmation of room join, includes current members. */
export interface JoinedMessage {
  type: 'joined';
  /** This node's peer ID. */
  peerId: string;
  /** Peer IDs already in the room. */
  peers: string[];
}

/** Broadcast when a new peer joins the room. */
export interface PeerJoinedMessage {
  type: 'peer-joined';
  /** The new peer's ID. */
  peerId: string;
}

/** Broadcast when a peer leaves the room. */
export interface PeerLeftMessage {
  type: 'peer-left';
  /** The departing peer's ID. */
  peerId: string;
}

/** Relayed SDP offer from another peer. */
export interface RelayedOfferMessage {
  type: 'offer';
  /** Sender peer ID. */
  from: string;
  /** SDP offer. */
  sdp: RTCSessionDescriptionInit;
}

/** Relayed SDP answer from another peer. */
export interface RelayedAnswerMessage {
  type: 'answer';
  /** Sender peer ID. */
  from: string;
  /** SDP answer. */
  sdp: RTCSessionDescriptionInit;
}

/** Relayed ICE candidate from another peer. */
export interface RelayedCandidateMessage {
  type: 'candidate';
  /** Sender peer ID. */
  from: string;
  /** ICE candidate. */
  candidate: RTCIceCandidateInit;
}

/** Error notification. */
export interface ErrorMessage {
  type: 'error';
  /** Machine-readable error code. */
  code: string;
  /** Human-readable description. */
  message: string;
}

export type ServerMessage =
  | WelcomeMessage
  | JoinedMessage
  | PeerJoinedMessage
  | PeerLeftMessage
  | RelayedOfferMessage
  | RelayedAnswerMessage
  | RelayedCandidateMessage
  | ErrorMessage;
