/**
 * SignalingRoom Durable Object
 *
 * Manages a single signaling room — a logical grouping of nodes that
 * are attempting to establish WebRTC connections with each other.
 *
 * Design principles:
 * - **Stateless**: no persistent storage; all state lives in-memory
 *   for the lifetime of the WebSocket connections.
 * - **Transparent relay**: the DO does not inspect or transform SDP/ICE
 *   payloads — it simply routes them from sender to target.
 * - **Automatic cleanup**: peers are removed on WebSocket close/error,
 *   and remaining peers are notified.
 *
 * Each Durable Object instance is identified by a room ID (derived from
 * the URL path). Cloudflare guarantees all connections for the same room
 * ID are routed to the same DO instance.
 */

import type {
  ClientMessage,
  ServerMessage,
  JoinMessage,
  OfferMessage,
  AnswerMessage,
  CandidateMessage
} from './types';

/** Metadata for a connected peer. */
interface PeerState {
  /** Unique peer ID assigned by the DO. */
  id: string;
  /** The WebSocket connection to this peer. */
  ws: WebSocket;
  /** Room ID this peer has joined (null if not yet joined). */
  roomId: string | null;
}

export class SignalingRoom {
  /** All connected peers, keyed by peer ID. */
  private peers = new Map<string, PeerState>();
  /** Room membership: roomId → Set of peer IDs. */
  private rooms = new Map<string, Set<string>>();
  /** Monotonic counter for generating unique peer IDs. */
  private nextPeerId = 0;

  constructor(
    private ctx: DurableObjectState,
    private env: Env
  ) {}

  /**
   * Handle incoming HTTP requests.
   *
   * - GET /ws → upgrade to WebSocket
   * - GET /health → health check
   * - GET /stats → room statistics
   */
  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname === '/health') {
      return new Response('ok', { status: 200 });
    }

    if (url.pathname === '/stats') {
      const stats = {
        peers: this.peers.size,
        rooms: this.rooms.size,
        roomDetails: Object.fromEntries(
          [...this.rooms.entries()].map(([id, members]) => [id, members.size])
        )
      };
      return Response.json(stats);
    }

    // WebSocket upgrade
    if (request.headers.get('Upgrade') === 'websocket') {
      const pair = new WebSocketPair();
      const [client, server] = Object.values(pair);

      this.handleWebSocket(server);

      return new Response(null, { status: 101, webSocket: client });
    }

    return new Response('Expected WebSocket upgrade', { status: 426 });
  }

  /**
   * Register a new WebSocket connection.
   *
   * Assigns a peer ID and sets up message/close/error handlers.
   */
  private handleWebSocket(ws: WebSocket): void {
    const peerId = this.generatePeerId();
    const peer: PeerState = { id: peerId, ws, roomId: null };
    this.peers.set(peerId, peer);

    ws.accept();

    // Send welcome with assigned peer ID
    this.send(ws, { type: 'welcome', peerId });

    ws.addEventListener('message', (event) => {
      try {
        const msg: ClientMessage = JSON.parse(event.data as string);
        this.handleMessage(peerId, msg);
      } catch {
        this.send(ws, {
          type: 'error',
          code: 'INVALID_MESSAGE',
          message: 'Failed to parse message as JSON'
        });
      }
    });

    ws.addEventListener('close', () => {
      this.handleDisconnect(peerId);
    });

    ws.addEventListener('error', () => {
      this.handleDisconnect(peerId);
    });
  }

  /**
   * Route an incoming client message to the appropriate handler.
   */
  private handleMessage(fromId: string, msg: ClientMessage): void {
    switch (msg.type) {
      case 'join':
        this.handleJoin(fromId, msg);
        break;
      case 'offer':
        this.handleRelay(fromId, msg as OfferMessage);
        break;
      case 'answer':
        this.handleRelay(fromId, msg as AnswerMessage);
        break;
      case 'candidate':
        this.handleRelay(fromId, msg as CandidateMessage);
        break;
      case 'leave':
        this.handleDisconnect(fromId);
        break;
    }
  }

  /**
   * Handle a peer joining a room.
   *
   * 1. Remove peer from any previous room
   * 2. Add to the requested room
   * 3. Notify existing room members
   * 4. Send the joiner a list of current members
   */
  private handleJoin(fromId: string, msg: JoinMessage): void {
    const peer = this.peers.get(fromId);
    if (!peer) return;

    // Leave previous room if any
    if (peer.roomId) {
      this.leaveRoom(fromId, peer.roomId);
    }

    const { roomId } = msg;
    peer.roomId = roomId;

    // Get or create room
    let room = this.rooms.get(roomId);
    if (!room) {
      room = new Set();
      this.rooms.set(roomId, room);
    }

    // Notify existing members about the new peer
    for (const existingId of room) {
      const existing = this.peers.get(existingId);
      if (existing) {
        this.send(existing.ws, { type: 'peer-joined', peerId: fromId });
      }
    }

    // Add peer to room
    room.add(fromId);

    // Send joiner the current peer list
    this.send(peer.ws, {
      type: 'joined',
      peerId: fromId,
      peers: [...room].filter((id) => id !== fromId)
    });
  }

  /**
   * Relay an SDP offer, SDP answer, or ICE candidate to the target peer.
   *
   * Validates that both sender and target exist and are in the same room
   * before forwarding.
   */
  private handleRelay(fromId: string, msg: OfferMessage | AnswerMessage | CandidateMessage): void {
    const sender = this.peers.get(fromId);
    const target = this.peers.get(msg.target);

    if (!sender || !target) {
      if (sender) {
        this.send(sender.ws, {
          type: 'error',
          code: 'PEER_NOT_FOUND',
          message: `Target peer ${msg.target} not found`
        });
      }
      return;
    }

    // Verify both are in the same room
    if (sender.roomId !== target.roomId || !sender.roomId) {
      this.send(sender.ws, {
        type: 'error',
        code: 'ROOM_MISMATCH',
        message: 'Sender and target are not in the same room'
      });
      return;
    }

    // Relay the message with sender info
    const relayMsg: ServerMessage = {
      type: msg.type,
      from: fromId,
      ...('sdp' in msg ? { sdp: msg.sdp } : {}),
      ...('candidate' in msg ? { candidate: msg.candidate } : {})
    } as ServerMessage;

    this.send(target.ws, relayMsg);
  }

  /**
   * Handle peer disconnection (close, error, or explicit leave).
   *
   * 1. Remove from room and notify remaining members
   * 2. Clean up empty rooms
   * 3. Remove from peer registry
   */
  private handleDisconnect(peerId: string): void {
    const peer = this.peers.get(peerId);
    if (!peer) return;

    if (peer.roomId) {
      this.leaveRoom(peerId, peer.roomId);
    }

    this.peers.delete(peerId);
  }

  /**
   * Remove a peer from a room and notify remaining members.
   * Cleans up empty rooms.
   */
  private leaveRoom(peerId: string, roomId: string): void {
    const room = this.rooms.get(roomId);
    if (!room) return;

    room.delete(peerId);

    // Notify remaining members
    for (const remainingId of room) {
      const remaining = this.peers.get(remainingId);
      if (remaining) {
        this.send(remaining.ws, { type: 'peer-left', peerId });
      }
    }

    // Clean up empty rooms
    if (room.size === 0) {
      this.rooms.delete(roomId);
    }
  }

  /**
   * Send a JSON message over a WebSocket.
   */
  private send(ws: WebSocket, msg: ServerMessage): void {
    try {
      ws.send(JSON.stringify(msg));
    } catch {
      // Connection may have been closed; ignore
    }
  }

  /**
   * Generate a unique peer ID for this DO instance.
   * Format: "p-<hex>" where hex is a monotonically increasing counter.
   */
  private generatePeerId(): string {
    return `p-${(this.nextPeerId++).toString(16)}`;
  }
}

/** Cloudflare environment bindings. */
interface Env {
  SIGNALING_ROOM: DurableObjectNamespace;
}
