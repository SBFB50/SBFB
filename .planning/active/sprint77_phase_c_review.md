# Sprint 77 — Review Phase C : shard wire primitives + run proof

## Verdict: PASS

> Review Workflow ultracode (`wf_c61d813a-83f`) : fan-out 4 dimensions +
> vérification adversariale (3 findings confirmés réels) + synthèse.
> Aucun P0/P1. Les 4 P2 + 2 P3 de couverture de tests ont été **fermés
> en-phase** (cf. §Résolution en-phase). Verdict promu PASS après Codex
> (cf. §Codex reconciliation).

## Résumé (3-5 lignes)
Phase C livre EXACTEMENT le scope §6.2 du plan : 4 primitives wire
(`ShardAssignment`/`ShardPlan` non signés, `ShardedSessionManifest` signé
`DOMAIN_SHARD_PLAN_V1`, `RunProof` signé `DOMAIN_RUN_PROOF_V1`), en mirror
fidèle ligne-par-ligne du patron `compute_group.rs` (Phase B). Crypto
correcte (contre-exemples de forgerie rejetés), 0 bump wire confirmé sur le
diff réel (`--numstat` = 0 deletion / 4 fichiers, 2 `*_FORMAT_VERSION`=1, 2
`DOMAIN_*` strictement additifs), no-float confirmé, `#[serde(default)]`
borné aux 2 champs optionnels documentés, slot N0 `activation_fingerprint`
= `[0u8;32]` réservé avec doc-note d'auto-attestation honnête. **Aucun
P0/P1.** Les findings P2/P3 = dette de couverture sur des branches d'erreur
défensives correctes, désormais fermée en-phase.

## Dimension 1 — Crypto correctness — **PASS**
`ShardedSessionManifestEntry::sign` et `RunProofEntry::sign` suivent l'ordre
exact du patron `compute_group.rs` : (1) identité==keypair sinon
`Err(Crypto)`, (2) `check_*_caps`, (3) `canonical_bytes(&payload, DOMAIN)`,
(4) `keypair.sign`. `verify_signature` suit : (1) version, (2) caps AVANT
hash, (3) attribution redondante, (4) `crypto::verify`. La signature couvre
le **payload seul** : `canonical_bytes` est appelé sur `&self.manifest` /
`&self.proof` uniquement — l'enveloppe (`signature` BigArray + identité
redondante) n'entre jamais dans la pré-image (prouvé par
`manifest_verify_rejects_tampered_payload`). Caps DoS bornés au sign ET au
verify, avant hash, avec rejet verify-side prouvé sur signature nulle
(rejet AVANT crypto). Séparation de domaine correcte :
`DOMAIN_SHARD_PLAN_V1` ≠ `DOMAIN_RUN_PROOF_V1` ≠ `DOMAIN_COMPUTE_GROUP_V1` ;
séparateur `0x00` dans `canonical_bytes` → collision de préfixe impossible ;
anti-replay prouvé par `cross_domain_signature_rejected`.
`is_pipeline_contiguous` est un invariant structurel non vérifié au verify
par design (doc-noté : le check stateful « couvre exactement `[0..L)` » vit
dans le scheduler Phase D). Aucune faille crypto réelle.

## Dimension 2 — Scope / conformité plan+preflight — **PASS**
Les 4 primitives §6.2 sont livrées en mirror exact. Les 5 corrections-clés
du preflight PLAN-ADAPT sont toutes appliquées : (1) `RunMetrics`
tout-entiers (`decode_milli_tokens_per_sec:u64`, `network_rx/tx_bytes:u64`) ;
(2) digests `[u8;32]` partout (jamais `String "blake3:..."`) ; (3)
`version:u16` + enveloppe `*Entry` (pas `schema_version` à plat) ; (4) slot
N0 `activation_fingerprint`=`[0u8;32]` forcé par `RunProof::new`, doc-note
honnête alignée sur `task.rs::logprobs_hash` ; (5) `Eq` dérivable partout.
**0 bump wire** : `SHARD_PLAN_FORMAT_VERSION`/`RUN_PROOF_FORMAT_VERSION`=1
net-new, aucun des 11 `*_FORMAT_VERSION` existants touché, 2 `DOMAIN_*`
strictement additifs. **Aucun scope creep** : pas de scheduler, pas
d'exécution, pas de vérif réelle du fingerprint (différés D/F/G).
Corrélation `group_id` manifest↔`ComputeGroup` doc-notée comme invariant
stateful non-câblé en C (Phase D/J).

## Dimension 3 — Couverture de tests — **CONCERN → fermé en-phase**
15 `#[test]` initiaux couvrant le cœur du patron. La review a identifié 6
branches d'erreur défensives **correctes** mais non exercées (4 P2 + 2 P3) :
version-mismatch au verify (manifest + run_proof), caps `session_id`
(manifest + run_proof), cap `shard_hashes` (par-assignment itératif),
asymétrie des tests négatifs RunProof (`sign_rejects_wrong_signer` +
`verify_rejects_tampered_{payload,signature}` manquants), direction
cross-domain RunProof. Aucune n'était P0/P1 (code correct, mirror fidèle),
mais ce sont de vraies branches qu'un refactor casserait sans rougir aucun
test — exactement le scénario que vise la gate-of-testability. **Décision
(directive ultra-complet / no band-aid) : fermées en-phase**, pas en carry.

## Dimension 4 — Conventions / patterns / wire-format — **PASS**
0 magic number : 7 constantes nommées doc-commentées avec rationale chiffré
+ miroir `compute_group`. Domaines énumérés = enums fermés
(`ShardRole`/`KvCachePolicy` `rename_all="snake_case"` — rôle/policy inconnu
= erreur de désérialisation au boundary signé, conforme
`feedback_named_constants`). Rich rustdoc + AGPL header + module-doc calé sur
`compute_group`. re-exports `lib.rs` complets et triés. Doc-comment des 2
`DOMAIN_*` énumère la disjonction cross-famille + phrase canon « purely
additive, 0-bump … S74 `DOMAIN_SEED_REQUEST_V1` pattern ». `--numstat` = 0
deletion (ajout pur). Unique `#[allow(clippy::too_many_arguments)]`
(`ShardedSessionManifest::new`, 8 args) justifié ; aucun `unwrap` en prod ;
`#[serde(default)]` borné à `fallback_node` + `activation_fingerprint`,
jamais sur version/identité.

## Findings confirmés (tous fermés en-phase)

| severity | titre | fichier:ligne | statut |
|---|---|---|---|
| P2 | Branche version-mismatch au verify non testée (manifest + run_proof) | shard_plan.rs | FERMÉ : `manifest_verify_rejects_wrong_version` + `run_proof_verify_rejects_wrong_version` (sig valide sur version inconnue → seul le gate version rejette) |
| P2 | Caps `session_id` non exercés (manifest + run_proof) | shard_plan.rs | FERMÉ : `manifest_rejects_oversized_session_id` + `run_proof_rejects_oversized_session_id` (sign + verify-side) |
| P2 | Asymétrie RunProof : `sign_rejects_wrong_signer` + `verify_rejects_tampered_{payload,signature}` manquants | shard_plan.rs | FERMÉ : 3 tests ajoutés, mirror des jumeaux manifest |
| P3 | Cap `shard_hashes` (par-assignment itératif) non exercé | shard_plan.rs | FERMÉ : `manifest_rejects_oversized_shard_hashes` (sign + verify-side) |
| P3 | Direction cross-domain RunProof non testée | shard_plan.rs | FERMÉ : `run_proof_cross_domain_signature_rejected` |
| P3 | Scope-cut `network_profile`/`security` (draft §10.1) non tracé | shard_plan.rs | Documenté au commit body §Scope cuts (différé cohérent, non exigé §6.2) |

**Aucun P0/P1.** Les 6 branches sont désormais couvertes (9 tests ajoutés).

## Résolution en-phase
9 tests défensifs ajoutés après la review (mirror des jumeaux du patron) :
`manifest_verify_rejects_wrong_version`, `run_proof_verify_rejects_wrong_version`,
`manifest_rejects_oversized_session_id`, `run_proof_rejects_oversized_session_id`,
`manifest_rejects_oversized_shard_hashes`, `run_proof_sign_rejects_wrong_signer`,
`run_proof_verify_rejects_tampered_payload`, `run_proof_verify_rejects_tampered_signature`,
`run_proof_cross_domain_signature_rejected`. Delta tests : **15 → 24**.

## Délta tests
`shard_plan.rs` net-new = **24 `#[test]`** (tous grep-résolus à de vraies
fn). Fichier NET-NEW (0 deletion). `canonical.rs` (+2 DOMAIN_*), `lib.rs`
(+mod +re-exports) : additif pur, 0 test.

## Carry T-NN+3 (statut)
**NON absorbé — correct.** Phase C duplique le boilerplate
`sign`/`verify_signature`/`check_*_caps` une 5e/6e fois sans introduire de
trait `SignedEnvelope` transverse (conforme preflight §180 + directive
anti-band-aid : un refactor transverse élargirait le blast radius sur 4+
modules crypto stables). `canonical.rs` touché UNIQUEMENT pour 2 constantes
`DOMAIN_*` additives. **Carry P2 T-NN+3 documenté au commit body** (5e/6e
copie atteinte, candidat sprint de dette dédié).

## Note pour Codex (points à revérifier en externe)
1. Anti-replay cross-domaine (pré-image diffère par préfixe domaine + `0x00`).
2. Caps AVANT hash aux deux bouts (sign + verify).
3. Signature couvre le payload seul (`canonical_bytes` jamais sur l'enveloppe).
4. No-float exhaustif (`grep f32|f64` = doc-comment seul).
5. 0 bump wire (`git diff --cached` : 11 `*_FORMAT_VERSION` intacts, 2 DOMAIN_* additifs).
6. Les 9 tests défensifs ajoutés couvrent bien les 6 branches listées.

## Codex reconciliation
Codex GPT 5.5 (`codex exec`, output brut dans `sprint77_phase_c_codex_review.md`)
a tourné **4 rounds**, convergence en fermant un item réel décroissant à
chaque tour (aucun GAP P0/P1 sur aucun round) :
- **Round 1** : 6 CONFIRME / 0 GAP / 1 PARTIEL — caps verify-side `group_id`
  + `session_id` (manifest) testés sign-side seul. → FERMÉ : forge-enveloppe
  verify-side ajoutée aux 2 tests.
- **Round 2** : 6 / 0 / 1 PARTIEL — les asserts verify-side de cap (signature
  nulle + `is_err()`) passeraient aussi si le cap était absent (échec dû à la
  signature). → FERMÉ : les 6 asserts verify-side de cap assertent désormais
  l'erreur de cap spécifique (`.contains("exceeds")`), prouvant que le cap
  fire AVANT la crypto.
- **Round 3** : 6 / 0 / 1 PARTIEL — `DOMAIN_SHARD_PLAN_V1`/`DOMAIN_RUN_PROOF_V1`
  absents du `pub use canonical::{...}` racine (les 19 autres DOMAIN_* y sont,
  dont `DOMAIN_COMPUTE_GROUP_V1` de Phase B). → FERMÉ : 2 domaines ajoutés au
  re-export racine.
- **Round 4** : **7 CONFIRME / 0 GAP / 0 PARTIEL — CLEAN.**

Suites relancées après chaque correction (fmt + clippy + nextest ciblé) ;
gate final complet (Win workspace + Docker canonique) vert. Le fichier
`sprint77_phase_c_codex_review.md` est l'output BRUT du round 4 (non réécrit).
Aucun GAP P0/P1 jamais soulevé ; les 3 PARTIEL successifs étaient de la
rigueur de test / cohérence sur du code confirmé correct, fermés en-phase
(directive ultra-complet / no band-aid).
