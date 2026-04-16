// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase A keystore micro-benchmarks.
//!
//! Wall-clock budget for the Argon2id + AEAD path:
//! - `derive_kek_prod_64_mib` : **< 5 s** (calibration target 3 s)
//! - `encrypt_decrypt_happy`  : **< 1 ms**
//! - `bench_unlock_total`     : **< 6 s** (KDF + AEAD + dummy
//!   keyring fetch using the `without_os_keyring` path)
//!
//! The absolute numbers land in `docs/rust/PATTERNS.md
//! §T-keystore-bench-reference` at Phase F so the audit layer can
//! flag drift on the next sprint. The raw pattern is lifted from
//! Sprint 19's `pow` bench — sample size 10, 3-second target time,
//! `html_reports` disabled (CI runs headless).

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use nexus_core_rs::keystore::{KdfParams, KeyStore, LocalFileKeyStore};
use tempfile::TempDir;

/// `derive_kek1` is crate-private — the bench re-derives through
/// the public Argon2 API with the same params so the number we
/// publish matches the production code path byte-for-byte. If the
/// production KDF ever drifts (different domain tag, different
/// output length), the primitive tests inside `keystore.rs` break
/// immediately. Keeping this bench inline avoids widening the
/// public surface for a measurement-only helper.
fn bench_derive_kek_prod(c: &mut Criterion) {
    use argon2::{Algorithm, Argon2, Params, Version};
    let params = Params::new(
        KdfParams::production().m_cost_kib,
        KdfParams::production().t_cost,
        KdfParams::production().parallelism,
        Some(32),
    )
    .expect("prod params must be valid");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut group = c.benchmark_group("keystore_prod");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));
    group.bench_function("derive_kek_64_mib", |b| {
        b.iter(|| {
            let mut out = [0u8; 32];
            let salt = [7u8; 16];
            argon2
                .hash_password_into(b"bench-pin", &salt, &mut out)
                .unwrap();
            out
        })
    });
    group.finish();
}

fn bench_encrypt_decrypt_happy(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let store = LocalFileKeyStore::with_params(dir.path(), KdfParams::fast_for_tests())
        .without_os_keyring();
    store.init("1234").expect("init");

    let mut group = c.benchmark_group("keystore_aead");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(5));
    group.bench_function("unlock_fast_params", |b| {
        b.iter(|| {
            let id = store.unlock("1234").expect("unlock");
            // Consume the identity to zero the SecretBox.
            let _ = id.into_secret_bytes();
        })
    });
    group.finish();
}

fn bench_unlock_total(c: &mut Criterion) {
    // Total unlock path with production KDF (no keyring: CI-safe).
    // Runs a single iteration per sample because prod KDF is ~3 s.
    let dir = TempDir::new().unwrap();
    let store = LocalFileKeyStore::new(dir.path()).without_os_keyring();
    store.init("1234").expect("init (~3 s)");

    let mut group = c.benchmark_group("keystore_unlock_total");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(45));
    group.bench_function("unlock_prod_params_no_keyring", |b| {
        b.iter(|| {
            let id = store.unlock("1234").expect("unlock prod");
            let _ = id.into_secret_bytes();
        })
    });
    group.finish();
}

criterion_group!(
    keystore_benches,
    bench_derive_kek_prod,
    bench_encrypt_decrypt_happy,
    bench_unlock_total
);
criterion_main!(keystore_benches);
