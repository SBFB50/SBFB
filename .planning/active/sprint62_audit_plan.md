# Sprint 62 — Audit plan (auditer Sprint 61)

**Ecrit** : 2026-05-13 (Sprint 61 Phase D wrap-up).
**Sprint a auditer** : Sprint 61 (spec executable + feed local rejouable).
**Tip master Sprint 61** : Phase D commit (ce sprint).

---

## §1 Perimetre de l'audit

Sprint 61 a livre :
- Spec protocole `docs/protocol/PUBLIC_FEED_SPEC.md`
- Types Rust `PublicFeedOperation` + `FeedEntry` + `FeedEntryCanonical`
- Domaine `DOMAIN_FEED_V1` dans canonical.rs
- Migration M9 (public_feed) + M10 (feed_cursor) dans db.rs
- `FeedStore` : insert_feed_operation, replay_all, verify_chain
- `FeedMaterializer` : materialize_full, materialize_verified, materialize_incremental
- `PublicRegistryView` + `ProjectFeedStatus`
- Tests adversariaux : chain tamper, forged signature, orphan stale, cursor restart
- 23 tests Rust ajoutes (1259 → 1282)

---

## §2 Dimensions a auditer

### 2.1 Spec vs Code alignment

- Chaque section de PUBLIC_FEED_SPEC.md doit avoir un equivalent
  dans le code Rust (types, constants, serialization, verification).
- Les test vectors JSON de la spec doivent correspondre aux valeurs
  observees en execution.
- Le `FEED_FORMAT_VERSION` dans le code doit matcher la spec.

### 2.2 Hash-chain integrity

- verify_chain() doit detecter : hash tamper, signature forge,
  prev_hash gap, genesis corruption.
- Le pattern doit etre coherent avec kudos_ledger.rs (meme famille
  BLAKE3 + Ed25519 + JCS).
- Les 2 implementations ne doivent pas diverger sur les primitives.

### 2.3 Migration safety

- M9 et M10 ne doivent pas casser les 8 migrations existantes.
- Le schema M9 doit matcher le kickoff D2 (apres resolution du
  BLOB vs TEXT warning design review).
- cursor table M10 : schema minimal, pas de leak de donnees si
  la DB est inspectee.

### 2.4 Materializer correctness

- materialize_full depuis genesis doit produire le meme resultat
  que materialize_incremental (cursor restart consistency).
- materialize_verified doit echouer sur un feed corrompu.
- materialize_incremental avec cursor hash mismatch doit fallback
  vers full rebuild.

### 2.5 Post-v1.0 policy compliance

- `FEED_FORMAT_VERSION = 1` sous le regime post-v1.0.
- `#[serde(default)]` present sur les champs optionnels pour
  compat ascendante.
- Pas de "pre-launch" tolerant decoder legacy (v1 only).

### 2.6 Scope cuts respect

- 12/12 scope cuts documentes dans kickoff §7.
- Pas de sync P2P, pas d'anti-spam, pas de CuratorVouched,
  pas de HTTP endpoint, pas de bridge provenance, pas de UI.

### 2.7 Carries review

- 6 carries S61 → S62 avec compteurs corrects.
- P2-NSIS-UNINSTALL et P2-IMAGE-DEP a 2/3 → MANDATORY S63 si
  pas resolus S62.
- P2-PLAYWRIGHT-REFACTOR a 2/3 → idem.

---

## §3 Fichiers cles a inspecter

| Fichier | Pourquoi |
|---|---|
| `docs/protocol/PUBLIC_FEED_SPEC.md` | Spec vs code alignment |
| `crates/nexus-coordinator-rs/src/public_feed.rs` | Types + store + chain |
| `crates/nexus-coordinator-rs/src/feed_materializer.rs` | Materializer + cursor |
| `crates/nexus-coordinator-rs/src/db.rs` | Migrations M9+M10 |
| `crates/nexus-core-rs/src/canonical.rs` | DOMAIN_FEED_V1 |
| `.planning/active/sprint61_kickoff.md` | D1..D5 gelees |
| `.planning/active/sprint61_verification.md` | Self-report |

---

## §4 Verdict attendu

L'audit doit produire :
- Findings P0/P1 : action bloquante avant Sprint 62 code.
- Findings P2+ : au moins 1 requis pour rigor G4.
- sprint61_audit_findings.md avec verdict PASS / CONDITIONAL PASS / FAIL.
