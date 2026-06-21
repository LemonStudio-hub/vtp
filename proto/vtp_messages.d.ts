/* eslint-disable @typescript-eslint/no-unused-vars */
/**
 * Minimal type declarations for the generated protobuf module.
 *
 * The full generated files (vtp_messages.js, vtp_messages.json)
 * are produced by `node proto/generate.js` and are gitignored.
 * This stub provides enough type information for TypeScript/svelte-check
 * without committing the full generated artifacts.
 */
export namespace vtp {
  interface ISignedMessage {
    payload?: Uint8Array | null;
    senderPubkey?: Uint8Array | null;
    signature?: Uint8Array | null;
    timestamp?: number | Long | null;
  }

  class SignedMessage {
    static encode(message: ISignedMessage): { finish(): Uint8Array };
    static decode(data: Uint8Array): SignedMessage;
    public payload: Uint8Array;
    public senderPubkey: Uint8Array;
    public signature: Uint8Array;
    public timestamp: Long;
  }

  class MessageBody {
    static encode(message: Record<string, unknown>): { finish(): Uint8Array };
    static decode(data: Uint8Array): MessageBody;
    public ping?: { nonce: number } | null;
    public pong?: { nonce: number } | null;
    public checkpoint?: Record<string, unknown> | null;
    public winner?: Record<string, unknown> | null;
    public peerInfo?: Record<string, unknown> | null;
    public vdfProgress?: Record<string, unknown> | null;
    public consensusProposal?: Record<string, unknown> | null;
    public consensusVote?: Record<string, unknown> | null;
    public newRound?: Record<string, unknown> | null;
  }
}

type Long = number;
