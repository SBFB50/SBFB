// SPDX-License-Identifier: AGPL-3.0-or-later
//! Criterion bench for the Hashcash PoW solver.
//!
//! Sprint 19 Phase B : run with `cargo bench --bench pow` to
//! measure wall-clock solving time at three difficulty levels.
//! The intent is regression-guarding, not microbenchmarking :
//! Criterion's reports flag a 2x regression on the default
//! difficulty if a future refactor accidentally pessimises the
//! hash loop.
//!
//! Target wall-clock (single core, modern 2026 CPU) :
//!
//! | Difficulty | Expected | Upper bound |
//! |---|---|---|
//! | 2^12 | ~5 ms | 50 ms |
//! | 2^18 (default) | ~100 ms | 500 ms |
//! | 2^20 | ~400 ms | 2000 ms |
//!
//! The 2^20 bench is guarded by a sample_size of 10 so CI does
//! not spend minutes on it.

use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use nexus_core_rs::pow::{HashcashChallenge, solve};

fn bench_solve_at(c: &mut Criterion, difficulty: u32, label: &str, sample_size: usize) {
    let mut group = c.benchmark_group(label);
    group.sample_size(sample_size);
    // Use a fixed issued_at so the canonical bytes don't drift
    // across iterations — the solver restarts nonce=0 each call
    // so the measurement is over the average case for that
    // exact challenge.
    let challenge = HashcashChallenge::new_at([0x11; 32], [0x22; 32], difficulty, 1_700_000_000);
    group.bench_function("solve", |b| {
        b.iter(|| {
            let proof =
                solve(&challenge, Duration::from_secs(30)).expect("solve must find a nonce");
            // Drop the proof so the compiler doesn't optimise
            // away the work.
            std::hint::black_box(proof);
        });
    });
    group.finish();
}

fn bench_solve_12_bits(c: &mut Criterion) {
    bench_solve_at(c, 12, "pow_solve_12_bits", 50);
}

fn bench_solve_18_bits(c: &mut Criterion) {
    bench_solve_at(c, 18, "pow_solve_18_bits_default", 30);
}

fn bench_solve_20_bits(c: &mut Criterion) {
    // Stress bench : sample_size=10 keeps CI under ~30s worst
    // case.
    bench_solve_at(c, 20, "pow_solve_20_bits_stress", 10);
}

criterion_group!(
    benches,
    bench_solve_12_bits,
    bench_solve_18_bits,
    bench_solve_20_bits
);
criterion_main!(benches);
