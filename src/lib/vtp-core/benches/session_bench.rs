//! Performance benchmarks for the VTP Session state machine and integration tests.
//!
//! Benchmarks cover:
//! - Session creation
//! - Batch execution with lottery checks
//! - Checkpoint data serialization
//! - Winner detection
//! - End-to-end session lifecycle
//! - Utility functions (hashing, encoding, random generation)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vtp_core::session::{BatchResult, Session};
use vtp_core::utils::{bytes_to_hex, generate_random_bytes, hash_bytes, hex_to_bytes};

// ============================================================================
// Constants
// ============================================================================

const SEED: [u8; 32] = [0u8; 32];
const TAU: [u8; 32] = [0xFFu8; 32]; // High threshold = likely winner
const TAU_LOW: [u8; 32] = [0x00u8; 32]; // Low threshold = unlikely winner

// ============================================================================
// Session Creation Benchmarks
// ============================================================================

/// Benchmark session creation.
fn bench_session_creation(c: &mut Criterion) {
    c.bench_function("session/new", |b| {
        b.iter(|| black_box(Session::new(&SEED, 1000, 100, &TAU, 100)))
    });
}

/// Benchmark session creation with various total steps.
fn bench_session_creation_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("session/new/sizes");

    for total in [100, 1000, 10000, 100000] {
        group.bench_with_input(BenchmarkId::new("total", total), &total, |b, &t| {
            b.iter(|| black_box(Session::new(&SEED, t, 100, &TAU, 100)))
        });
    }

    group.finish();
}

// ============================================================================
// Batch Execution Benchmarks
// ============================================================================

/// Benchmark single batch execution (no winner check).
fn bench_session_batch_no_winner(c: &mut Criterion) {
    let mut group = c.benchmark_group("session/batch/no_winner");

    for batch_size in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("batch_size", batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    let mut session = Session::new(&SEED, 100000, 10000, &TAU_LOW, 10000);
                    let result = session.run_batch(size);
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark batch execution with checkpoint steps.
fn bench_session_batch_with_checkpoints(c: &mut Criterion) {
    let mut group = c.benchmark_group("session/batch/with_checkpoints");

    for batch_size in [100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("batch_size", batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    // k=10 means every 10 steps is a checkpoint
                    let mut session = Session::new(&SEED, 100000, 10, &TAU_LOW, 10);
                    let result = session.run_batch(size);
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark session batch execution throughput.
fn bench_session_batch_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("session/batch/throughput");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));

    group.bench_function("steps_per_second", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let mut session = Session::new(&SEED, 100000, 10000, &TAU_LOW, 10000);
                session.run_batch(1000);
            }
            start.elapsed()
        })
    });

    group.finish();
}

// ============================================================================
// Winner Detection Benchmarks
// ============================================================================

/// Benchmark winner detection with high threshold (likely winner).
fn bench_session_winner_detection(c: &mut Criterion) {
    c.bench_function("session/winner_detection", |b| {
        b.iter(|| {
            // Very high threshold = almost certain winner
            let mut session = Session::new(&SEED, 1000, 1, &TAU, 1);
            let result = session.run_batch(1000);
            black_box(result)
        })
    });
}

/// Benchmark winner verification.
fn bench_session_verify_winner(c: &mut Criterion) {
    // Create a session and find a winner first
    let mut session = Session::new(&SEED, 1000, 1, &TAU, 1);

    // Run until we find a winner
    let mut winner_step = 0u64;
    let mut winner_proof = Vec::new();
    for _ in 0..1000 {
        if let BatchResult::Winner(w) = session.run_batch(1) {
            winner_step = w.step;
            winner_proof = w.proof();
            break;
        }
    }

    if winner_step > 0 {
        c.bench_function("session/verify_winner", |b| {
            b.iter(|| black_box(session.verify_winner(winner_step, &winner_proof)))
        });
    }
}

// ============================================================================
// Checkpoint Benchmarks
// ============================================================================

/// Benchmark checkpoint data serialization.
fn bench_session_checkpoint(c: &mut Criterion) {
    let mut group = c.benchmark_group("session/checkpoint");

    // Create and advance a session
    let mut session = Session::new(&SEED, 10000, 100, &TAU_LOW, 100);
    session.run_batch(5000);

    group.bench_function("get_checkpoint_data", |b| {
        b.iter(|| black_box(session.get_checkpoint_data()))
    });

    group.bench_function("state_access", |b| b.iter(|| black_box(session.state())));

    group.bench_function("public_key_access", |b| b.iter(|| black_box(session.public_key())));

    group.finish();
}

// ============================================================================
// Session Lifecycle Benchmarks
// ============================================================================

/// Benchmark complete session lifecycle (create → run → finish).
fn bench_session_lifecycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("session/lifecycle");
    group.sample_size(10);

    for total in [100, 500, 1000] {
        group.bench_with_input(BenchmarkId::new("total_steps", total), &total, |b, &t| {
            b.iter(|| {
                let mut session = Session::new(&SEED, t, 10000, &TAU_LOW, 10000);
                loop {
                    let result = session.run_batch(1000);
                    match result {
                        BatchResult::Finished => break,
                        BatchResult::Error(_) => break,
                        _ => continue,
                    }
                }
                black_box(session.state())
            })
        });
    }

    group.finish();
}

/// Benchmark pause/resume operations.
fn bench_session_pause_resume(c: &mut Criterion) {
    c.bench_function("session/pause_resume", |b| {
        b.iter(|| {
            let mut session = Session::new(&SEED, 10000, 100, &TAU_LOW, 100);
            session.run_batch(100);
            session.pause();
            let _ = session.run_batch(100); // Should return Progress without computing
            session.resume();
            session.run_batch(100);
            black_box(session.state())
        })
    });
}

// ============================================================================
// Utility Function Benchmarks
// ============================================================================

/// Benchmark SHA-256 hashing.
fn bench_hash_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("utils/hash_bytes");

    for size in [32, 256, 1024, 4096, 10240] {
        let data = vec![0xABu8; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("bytes", size), &data, |b, d| {
            b.iter(|| black_box(hash_bytes(d)))
        });
    }

    group.finish();
}

/// Benchmark hex encoding.
fn bench_bytes_to_hex(c: &mut Criterion) {
    let mut group = c.benchmark_group("utils/bytes_to_hex");

    for size in [16, 32, 64, 256, 1024] {
        let data = vec![0xABu8; size];
        group.bench_with_input(BenchmarkId::new("bytes", size), &data, |b, d| {
            b.iter(|| black_box(bytes_to_hex(d)))
        });
    }

    group.finish();
}

/// Benchmark hex decoding.
fn bench_hex_to_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("utils/hex_to_bytes");

    for size in [16, 32, 64, 256, 1024] {
        let hex = "ab".repeat(size);
        group.bench_with_input(BenchmarkId::new("hex_chars", size * 2), &hex, |b, h| {
            b.iter(|| black_box(hex_to_bytes(h).unwrap()))
        });
    }

    group.finish();
}

/// Benchmark random byte generation.
fn bench_generate_random_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("utils/generate_random_bytes");

    for size in [16u32, 32, 64, 256, 1024] {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("bytes", size as usize), &size, |b, &s| {
            b.iter(|| black_box(generate_random_bytes(s)))
        });
    }

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    session_benches,
    bench_session_creation,
    bench_session_creation_sizes,
    bench_session_batch_no_winner,
    bench_session_batch_with_checkpoints,
    bench_session_batch_throughput,
    bench_session_winner_detection,
    bench_session_verify_winner,
    bench_session_checkpoint,
    bench_session_lifecycle,
    bench_session_pause_resume,
    bench_hash_bytes,
    bench_bytes_to_hex,
    bench_hex_to_bytes,
    bench_generate_random_bytes,
);

criterion_main!(session_benches);
