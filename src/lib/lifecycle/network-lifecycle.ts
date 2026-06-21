/**
 * Network Lifecycle Module
 *
 * Encapsulates initialization and teardown of the P2P networking layer:
 * - Protobuf codec initialization
 * - PeerManager creation and event wiring
 * - Signaling server connection
 *
 * Extracted from +page.svelte for independent testability.
 */

import { PeerManager, initCodec } from '$lib/network';
import type { DecodedMessage, PeerManagerState } from '$lib/network/types';
import type { CryptoProvider } from '$lib/consensus/crypto-provider';
import { updateNetworkState, addNetworkEvent, resetNetworkState } from '$stores/network';

export interface NetworkLifecycleOptions {
  crypto: CryptoProvider;
}

export interface NetworkLifecycleHandle {
  destroy: () => void;
}

/**
 * Initialize the P2P networking layer.
 *
 * Sets up the PeerManager with a default configuration. The signaling
 * server URL is read from the environment or defaults to localhost.
 */
export async function createNetworkLifecycle(
  options: NetworkLifecycleOptions
): Promise<NetworkLifecycleHandle> {
  let peerManager: PeerManager | null = null;

  try {
    // Initialize the Protobuf codec
    await initCodec();

    // Read signaling config from environment or use defaults
    const signalingUrl = import.meta.env.VITE_SIGNALING_URL || 'ws://localhost:8787';
    const roomId = import.meta.env.VITE_ROOM_ID || 'vtp-default';

    // Generate a temporary keypair for signing
    // In production, this would be derived from the VRF keypair
    const keypair = {
      publicKey: new Uint8Array(32),
      secretKey: new Uint8Array(32)
    };
    crypto.getRandomValues(keypair.publicKey);
    crypto.getRandomValues(keypair.secretKey);

    peerManager = new PeerManager({
      signaling: {
        serverUrl: signalingUrl,
        roomId,
        reconnectIntervalMs: 3000,
        maxReconnectAttempts: 5
      },
      keypair,
      crypto: options.crypto,
      preferIpv6: true
    });

    // Wire up PeerManager events
    peerManager.on('onStateChange', (state: PeerManagerState) => {
      updateNetworkState(state);
    });

    peerManager.on('onPeerConnected', (peerId: string) => {
      addNetworkEvent({
        type: 'peer-joined',
        message: 'Peer connected',
        peerId
      });
    });

    peerManager.on('onPeerDisconnected', (peerId: string) => {
      addNetworkEvent({
        type: 'peer-left',
        message: 'Peer disconnected',
        peerId
      });
    });

    peerManager.on('onMessage', (from: string, msg: DecodedMessage) => {
      addNetworkEvent({
        type: 'message',
        message: `Received ${msg.type}`,
        peerId: from
      });
    });

    // Connect to signaling server
    await peerManager.connect();

    addNetworkEvent({
      type: 'signaling',
      message: 'Connected to signaling server'
    });
  } catch (err) {
    console.warn('[Network] Failed to initialize networking:', err);
    addNetworkEvent({
      type: 'error',
      message: `Network init failed: ${err instanceof Error ? err.message : 'Unknown error'}`
    });
  }

  return {
    destroy() {
      peerManager?.disconnect();
      peerManager = null;
      resetNetworkState();
    }
  };
}
