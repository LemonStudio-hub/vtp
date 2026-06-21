//! Performance benchmarks for the Wesolowski VDF implementation.
//!
//! Benchmarks cover:
//! - Single VDF step (class group squaring)
//! - Batch VDF execution at various scales
//! - Discriminant and generator derivation
//! - Wesolowski proof generation and verification
//! - Class group composition and reduction
//! - BigInt arithmetic primitives
//! - Serialization/deserialization

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// Re-export vtp_core types for benchmarking
use vtp_core::vdf::{generate_proof, vdf_step, verify_proof, VdfIterator};

// ============================================================================
// Constants
// ============================================================================

const SEED_32: [u8; 32] = [0u8; 32];
const SEED_ALT: [u8; 32] = [42u8; 32];

// ============================================================================
// VDF Single Step Benchmarks
// ============================================================================

/// Benchmark a single VDF step (class group squaring).
fn bench_vdf_single_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("vdf/single_step");

    group.bench_function("squaring_zero_seed", |b| b.iter(|| black_box(vdf_step(&SEED_32))));

    group.bench_function("squaring_nonzero_seed", |b| b.iter(|| black_box(vdf_step(&SEED_ALT))));

    group.finish();
}

/// Benchmark VDF step determinism (same input → same output).
fn bench_vdf_step_determinism(c: &mut Criterion) {
    c.bench_function("vdf/step_determinism", |b| {
        b.iter(|| {
            let r1 = vdf_step(&SEED_32);
            let r2 = vdf_step(&SEED_32);
            assert_eq!(r1, r2);
            black_box(r1)
        })
    });
}

// ============================================================================
// VDF Iterator Benchmarks
// ============================================================================

/// Benchmark VDF iterator creation.
fn bench_vdf_iterator_creation(c: &mut Criterion) {
    c.bench_function("vdf/iterator_new", |b| {
        b.iter(|| black_box(VdfIterator::new(&SEED_32, 1000)))
    });
}

/// Benchmark VDF batch execution at various scales.
fn bench_vdf_batch_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("vdf/batch_execution");

    for batch_size in [1, 10, 100, 1000, 10000] {
        group.throughput(Throughput::Elements(batch_size));
        group.bench_with_input(BenchmarkId::new("steps", batch_size), &batch_size, |b, &size| {
            b.iter(|| {
                let mut iter = VdfIterator::new(&SEED_32, size);
                let steps = iter.run_batch(size);
                assert_eq!(steps, size);
                black_box(iter.get_state())
            })
        });
    }

    group.finish();
}

/// Benchmark VDF single-step iteration via `next()`.
fn bench_vdf_next(c: &mut Criterion) {
    let mut group = c.benchmark_group("vdf/next");

    for count in [1, 10, 100] {
        group.bench_with_input(BenchmarkId::new("calls", count), &count, |b, &n| {
            b.iter(|| {
                let mut iter = VdfIterator::new(&SEED_32, n);
                for _ in 0..n {
                    iter.next();
                }
                black_box(iter.get_state())
            })
        });
    }

    group.finish();
}

/// Benchmark VDF batch execution with larger total steps.
fn bench_vdf_large_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("vdf/large_batch");
    group.sample_size(10); // Fewer samples for long-running benchmarks

    for total in [1000, 5000, 10000] {
        group.throughput(Throughput::Elements(total));
        group.bench_with_input(BenchmarkId::new("total_steps", total), &total, |b, &total| {
            b.iter(|| {
                let mut iter = VdfIterator::new(&SEED_32, total);
                let steps = iter.run_batch(total);
                assert_eq!(steps, total);
                black_box(iter.get_state())
            })
        });
    }

    group.finish();
}

// ============================================================================
// Discriminant and Generator Derivation
// ============================================================================

/// Benchmark discriminant and generator derivation from seed.
fn bench_discriminant_derivation(c: &mut Criterion) {
    c.bench_function("vdf/derive_discriminant_and_generator", |b| {
        b.iter(|| {
            // This is internal, so we benchmark via VdfIterator::new which calls it
            let iter = VdfIterator::new(&SEED_32, 1);
            black_box(iter.get_state())
        })
    });
}

// ============================================================================
// Proof Generation and Verification
// ============================================================================

/// Benchmark Wesolowski proof generation at various time parameters.
fn bench_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vdf/proof_generation");
    group.sample_size(10);

    for total in [5, 10, 50, 100] {
        // Pre-compute the VDF output
        let mut iter = VdfIterator::new(&SEED_32, total);
        iter.run_batch(total);
        let state = iter.get_state();

        group.bench_with_input(BenchmarkId::new("T", total), &(total, state), |b, &(t, ref s)| {
            b.iter(|| black_box(generate_proof(&SEED_32, s, t)))
        });
    }

    group.finish();
}

/// Benchmark Wesolowski proof verification at various time parameters.
fn bench_proof_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("vdf/proof_verification");

    for total in [5, 10, 50, 100] {
        // Pre-compute VDF output and proof
        let mut iter = VdfIterator::new(&SEED_32, total);
        iter.run_batch(total);
        let state = iter.get_state();
        let proof = generate_proof(&SEED_32, &state, total);

        group.bench_with_input(
            BenchmarkId::new("T", total),
            &(total, state, proof),
            |b, &(t, ref s, ref p)| b.iter(|| black_box(verify_proof(&SEED_32, s, t, p))),
        );
    }

    group.finish();
}

/// Benchmark proof generation + verification round-trip.
fn bench_proof_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("vdf/proof_roundtrip");
    group.sample_size(10);

    for total in [5, 10, 50] {
        group.bench_with_input(BenchmarkId::new("T", total), &total, |b, &t| {
            b.iter(|| {
                let mut iter = VdfIterator::new(&SEED_32, t);
                iter.run_batch(t);
                let state = iter.get_state();
                let proof = generate_proof(&SEED_32, &state, t);
                assert!(verify_proof(&SEED_32, &state, t, &proof));
                black_box(proof)
            })
        });
    }

    group.finish();
}

// ============================================================================
// Class Group Internal Operations
// ============================================================================

/// Benchmark class group composition via vdf_step chaining.
fn bench_class_group_chaining(c: &mut Criterion) {
    let mut group = c.benchmark_group("vdf/class_group_chaining");

    for count in [10, 100, 1000] {
        group.throughput(Throughput::Elements(count));
        group.bench_with_input(BenchmarkId::new("chain_length", count), &count, |b, &n| {
            b.iter(|| {
                let mut state = SEED_32;
                for _ in 0..n {
                    state = vdf_step(&state);
                }
                black_box(state)
            })
        });
    }

    group.finish();
}

// ============================================================================
// Serialization Benchmarks
// ============================================================================

/// Benchmark VDF state serialization.
fn bench_vdf_state_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("vdf/serialization");

    // Create a VDF iterator and advance it
    let mut iter = VdfIterator::new(&SEED_32, 100);
    iter.run_batch(100);

    group.bench_function("get_state", |b| b.iter(|| black_box(iter.get_state())));

    group.finish();
}

// ============================================================================
// Throughput Benchmarks
// ============================================================================

/// Benchmark VDF throughput (steps per second).
fn bench_vdf_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("vdf/throughput");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));

    group.bench_function("steps_per_second_1000", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let mut iter = VdfIterator::new(&SEED_32, 1000);
                iter.run_batch(1000);
                black_box(iter.get_state());
            }
            start.elapsed()
        })
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    vdf_benches,
    bench_vdf_single_step,
    bench_vdf_step_determinism,
    bench_vdf_iterator_creation,
    bench_vdf_batch_execution,
    bench_vdf_next,
    bench_vdf_large_batch,
    bench_discriminant_derivation,
    bench_proof_generation,
    bench_proof_verification,
    bench_proof_roundtrip,
    bench_class_group_chaining,
    bench_vdf_state_serialization,
    bench_vdf_throughput,
);

criterion_main!(vdf_benches);
