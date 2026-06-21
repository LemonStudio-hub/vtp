export class Session {
  constructor(
    seed: Uint8Array,
    total: bigint,
    k: bigint,
    tau: Uint8Array,
    checkpointInterval: bigint
  );
  run_batch(max_steps: bigint): BatchResult;
  readonly state: SessionState;
  readonly public_key: Uint8Array;
  pause(): void;
  resume(): void;
  is_paused(): boolean;
  get_checkpoint_data(): Uint8Array;
  verify_winner(step: bigint, proof: Uint8Array): boolean;
  free(): void;
}

export class SessionState {
  readonly step: number;
  readonly total: number;
  readonly is_active: boolean;
  readonly is_paused: boolean;
  readonly error_count: number;
}

export type BatchResult = 'Progress' | 'Winner' | 'Finished' | 'Error';

export function generate_keypair(): VrfKeypair;
export function prove(secret_key: Uint8Array, alpha: Uint8Array): Uint8Array;
export function proof_to_hash(proof: Uint8Array): Uint8Array;
export function verify(public_key: Uint8Array, alpha: Uint8Array, proof: Uint8Array): boolean;
export function hash_bytes(data: Uint8Array): Uint8Array;
export function bytes_to_hex(bytes: Uint8Array): string;
export function hex_to_bytes(hex: string): Uint8Array;
export function generate_random_bytes(length: number): Uint8Array;
export function ed25519_sign(secret_key: Uint8Array, data: Uint8Array): Uint8Array;
export function ed25519_verify(
  public_key: Uint8Array,
  data: Uint8Array,
  signature: Uint8Array
): boolean;

export class VrfKeypair {
  readonly public_key: Uint8Array;
  readonly secret_key: Uint8Array;
}
