//! Performance benchmarks for the ECVRF-EDWARDS25519-SHA512-TAI implementation.
//!
//! Benchmarks cover:
//! - Keypair generation
//! - Proof generation (prove)
//! - Proof verification (verify)
//! - Proof to hash extraction
//! - End-to-end prove + verify round-trip
//! - Various message sizes
//! - Determinism verification overhead

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vtp_core::vrf::{generate_keypair, proof_to_hash, prove, verify};

// ============================================================================
// Constants
// ============================================================================

const MESSAGE_SMALL: &[u8] = b"benchmark";
const MESSAGE_MEDIUM: &[u8; 1024] = &[0xABu8; 1024];
const MESSAGE_LARGE: &[u8; 10240] = &[0xCDu8; 10240];

// ============================================================================
// Keypair Generation Benchmarks
// ============================================================================

/// Benchmark VRF keypair generation.
fn bench_keypair_generation(c: &mut Criterion) {
    c.bench_function("vrf/generate_keypair", |b| {
        b.iter(|| {
            let kp = generate_keypair();
            black_box((kp.public_key(), kp.secret_key()))
        })
    });
}

/// Benchmark keypair generation with key extraction.
fn bench_keypair_generation_with_extraction(c: &mut Criterion) {
    c.bench_function("vrf/generate_keypair_full", |b| {
        b.iter(|| {
            let kp = generate_keypair();
            let pk = kp.public_key();
            let sk = kp.secret_key();
            assert_eq!(pk.len(), 32);
            assert_eq!(sk.len(), 32);
            black_box((pk, sk))
        })
    });
}

// ============================================================================
// Proof Generation Benchmarks
// ============================================================================

/// Benchmark VRF proof generation with small message.
fn bench_prove_small(c: &mut Criterion) {
    let kp = generate_keypair();
    let sk = kp.secret_key();

    c.bench_function("vrf/prove/small_message", |b| {
        b.iter(|| black_box(prove(&sk, MESSAGE_SMALL)))
    });
}

/// Benchmark VRF proof generation with medium message (1 KB).
fn bench_prove_medium(c: &mut Criterion) {
    let kp = generate_keypair();
    let sk = kp.secret_key();

    c.bench_function("vrf/prove/medium_message_1kb", |b| {
        b.iter(|| black_box(prove(&sk, MESSAGE_MEDIUM)))
    });
}

/// Benchmark VRF proof generation with large message (10 KB).
fn bench_prove_large(c: &mut Criterion) {
    let kp = generate_keypair();
    let sk = kp.secret_key();

    c.bench_function("vrf/prove/large_message_10kb", |b| {
        b.iter(|| black_box(prove(&sk, MESSAGE_LARGE)))
    });
}

/// Benchmark VRF proof generation across message sizes.
fn bench_prove_message_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("vrf/prove/message_sizes");

    let kp = generate_keypair();
    let sk = kp.secret_key();

    for size in [0, 32, 256, 1024, 4096, 10240] {
        let message = vec![0xABu8; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("bytes", size), &message, |b, msg| {
            b.iter(|| black_box(prove(&sk, msg)))
        });
    }

    group.finish();
}

// ============================================================================
// Proof Verification Benchmarks
// ============================================================================

/// Benchmark VRF proof verification with small message.
fn bench_verify_small(c: &mut Criterion) {
    let kp = generate_keypair();
    let pk = kp.public_key();
    let sk = kp.secret_key();
    let proof = prove(&sk, MESSAGE_SMALL);

    c.bench_function("vrf/verify/small_message", |b| {
        b.iter(|| black_box(verify(&pk, MESSAGE_SMALL, &proof)))
    });
}

/// Benchmark VRF proof verification with medium message.
fn bench_verify_medium(c: &mut Criterion) {
    let kp = generate_keypair();
    let pk = kp.public_key();
    let sk = kp.secret_key();
    let proof = prove(&sk, MESSAGE_MEDIUM);

    c.bench_function("vrf/verify/medium_message_1kb", |b| {
        b.iter(|| black_box(verify(&pk, MESSAGE_MEDIUM, &proof)))
    });
}

/// Benchmark VRF proof verification with large message.
fn bench_verify_large(c: &mut Criterion) {
    let kp = generate_keypair();
    let pk = kp.public_key();
    let sk = kp.secret_key();
    let proof = prove(&sk, MESSAGE_LARGE);

    c.bench_function("vrf/verify/large_message_10kb", |b| {
        b.iter(|| black_box(verify(&pk, MESSAGE_LARGE, &proof)))
    });
}

/// Benchmark VRF proof verification across message sizes.
fn bench_verify_message_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("vrf/verify/message_sizes");

    let kp = generate_keypair();
    let pk = kp.public_key();
    let sk = kp.secret_key();

    for size in [0, 32, 256, 1024, 4096, 10240] {
        let message = vec![0xABu8; size];
        let proof = prove(&sk, &message);
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("bytes", size),
            &(message, proof),
            |b, (msg, p)| b.iter(|| black_box(verify(&pk, msg, p))),
        );
    }

    group.finish();
}

// ============================================================================
// Proof to Hash Benchmarks
// ============================================================================

/// Benchmark VRF proof_to_hash extraction.
fn bench_proof_to_hash(c: &mut Criterion) {
    let kp = generate_keypair();
    let sk = kp.secret_key();
    let proof = prove(&sk, MESSAGE_SMALL);

    c.bench_function("vrf/proof_to_hash", |b| b.iter(|| black_box(proof_to_hash(&proof))));
}

// ============================================================================
// Round-Trip Benchmarks
// ============================================================================

/// Benchmark full VRF prove + verify round-trip.
fn bench_prove_verify_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("vrf/roundtrip");

    let kp = generate_keypair();
    let pk = kp.public_key();
    let sk = kp.secret_key();

    for size in [32, 256, 1024] {
        let message = vec![0xABu8; size];
        group.bench_with_input(BenchmarkId::new("message_bytes", size), &message, |b, msg| {
            b.iter(|| {
                let proof = prove(&sk, msg);
                let valid = verify(&pk, msg, &proof);
                assert!(valid);
                black_box(proof)
            })
        });
    }

    group.finish();
}

/// Benchmark full round-trip including proof_to_hash.
fn bench_full_roundtrip(c: &mut Criterion) {
    let kp = generate_keypair();
    let pk = kp.public_key();
    let sk = kp.secret_key();

    c.bench_function("vrf/full_roundtrip", |b| {
        b.iter(|| {
            let proof = prove(&sk, MESSAGE_SMALL);
            let valid = verify(&pk, MESSAGE_SMALL, &proof);
            assert!(valid);
            let hash = proof_to_hash(&proof);
            assert_eq!(hash.len(), 32);
            black_box(hash)
        })
    });
}

// ============================================================================
// Determinism Benchmarks
// ============================================================================

/// Benchmark determinism check (prove twice, compare).
fn bench_determinism_check(c: &mut Criterion) {
    let kp = generate_keypair();
    let sk = kp.secret_key();

    c.bench_function("vrf/determinism_check", |b| {
        b.iter(|| {
            let p1 = prove(&sk, MESSAGE_SMALL);
            let p2 = prove(&sk, MESSAGE_SMALL);
            assert_eq!(p1, p2);
            black_box(p1)
        })
    });
}

// ============================================================================
// Throughput Benchmarks
// ============================================================================

/// Benchmark VRF proof generation throughput.
fn bench_prove_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("vrf/throughput");
    group.sample_size(50);
    group.measurement_time(std::time::Duration::from_secs(10));

    let kp = generate_keypair();
    let sk = kp.secret_key();

    group.bench_function("proofs_per_second", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let proof = prove(&sk, MESSAGE_SMALL);
                black_box(proof);
            }
            start.elapsed()
        })
    });

    group.finish();
}

/// Benchmark VRF verification throughput.
fn bench_verify_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("vrf/verify_throughput");
    group.sample_size(50);
    group.measurement_time(std::time::Duration::from_secs(10));

    let kp = generate_keypair();
    let pk = kp.public_key();
    let sk = kp.secret_key();
    let proof = prove(&sk, MESSAGE_SMALL);

    group.bench_function("verifications_per_second", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let valid = verify(&pk, MESSAGE_SMALL, &proof);
                assert!(valid);
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
    vrf_benches,
    bench_keypair_generation,
    bench_keypair_generation_with_extraction,
    bench_prove_small,
    bench_prove_medium,
    bench_prove_large,
    bench_prove_message_sizes,
    bench_verify_small,
    bench_verify_medium,
    bench_verify_large,
    bench_verify_message_sizes,
    bench_proof_to_hash,
    bench_prove_verify_roundtrip,
    bench_full_roundtrip,
    bench_determinism_check,
    bench_prove_throughput,
    bench_verify_throughput,
);

criterion_main!(vrf_benches);
