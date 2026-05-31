export class Session {
  constructor(
    seed: Uint8Array,
    total: number,
    k: number,
    tau: Uint8Array,
    checkpointInterval: number
  );
  run_batch(max_steps: number): string;
  state(): {
    step: number;
    total: number;
    is_active: boolean;
    is_paused: boolean;
    error_count: number;
  };
  public_key(): Uint8Array;
  pause(): void;
  resume(): void;
  is_paused(): boolean;
  get_checkpoint_data(): Uint8Array;
  verify_winner(step: number, proof: Uint8Array): boolean;
  free(): void;
}

export type BatchResult = 'Progress' | 'Winner' | 'Finished' | 'Error';
