/**
 * VTP Signaling Worker
 *
 * Cloudflare Worker entry point that routes incoming requests to the
 * appropriate Durable Object instance.
 *
 * Routing:
 *   GET /room/:roomId/ws      → WebSocket upgrade to SignalingRoom DO
 *   GET /room/:roomId/health   → DO health check
 *   GET /room/:roomId/stats    → DO statistics
 *   GET /health                → Worker-level health check
 *   *                          → 404
 *
 * The room ID from the URL path is used as the Durable Object name,
 * ensuring all connections for the same room are routed to the same
 * DO instance (Cloudflare guarantees single-instance-per-name).
 */

export { SignalingRoom } from './signaling-do';

interface Env {
  SIGNALING_ROOM: DurableObjectNamespace;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // Worker-level health check
    if (url.pathname === '/health') {
      return Response.json({
        status: 'ok',
        service: 'vtp-signaling',
        timestamp: Date.now()
      });
    }

    // Parse room ID from URL: /room/:roomId/...
    const roomMatch = url.pathname.match(/^\/room\/([^/]+)(\/.*)?$/);
    if (!roomMatch) {
      return Response.json(
        { error: 'Not found', hint: 'Use /room/:roomId/ws for WebSocket connections' },
        { status: 404 }
      );
    }

    const roomId = roomMatch[1];
    const subPath = roomMatch[2] || '/';

    // Validate room ID (alphanumeric + hyphens, 1-64 chars)
    if (!/^[a-zA-Z0-9-]{1,64}$/.test(roomId)) {
      return Response.json(
        { error: 'Invalid room ID', hint: 'Use alphanumeric characters and hyphens, 1-64 chars' },
        { status: 400 }
      );
    }

    // Get the Durable Object instance for this room
    const doId = env.SIGNALING_ROOM.idFromName(roomId);
    const stub = env.SIGNALING_ROOM.get(doId);

    // Forward the request to the DO
    const doUrl = new URL(request.url);
    doUrl.pathname = subPath;

    return stub.fetch(new Request(doUrl.toString(), request));
  }
};
