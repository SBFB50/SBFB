# Sprint 77 — Phase N review

> Produit par un Workflow ultracode (5 dimensions + 2 vérificateurs adversariaux
> + synthèse, 8 agents Opus 4.8 1M, 645k tokens). Run `wf_6f5182e7-cc9`.
> Verdict initial **CONCERN** (1 P1) → corrigé → Codex 4 rounds CLEAN →
> **PASS**, cf. §Réconciliation + §Codex reconciliation.

## Verdict: PASS

Review Workflow initial **CONCERN** (1 P1 + 2 P2) → tous corrigés ; Codex GPT 5.5
4 rounds → **CLEAN**. Détail §Réconciliation + §Codex reconciliation.

## Verdict (Workflow, initial): CONCERN
Review clean on lift-fidelity / contiguity-threat-honesty / scope / patterns, but
a CONFIRMED P1 — a false source-anchored cardinality claim ("closed 3-method
enum") that the source-ref-check silently passes — needs fixing before commit;
the defect is fully contained to 3 doc strings.

| Dimension | Verdict |
|---|---|
| VERBATIM-LIFT FIDELITY + example-runs-under-nextest | PASS |
| SOURCE-REF-CHECK correctness + robustness | CONCERN (P1) |
| CONTIGUITY-vs-COVERAGE + THREAT + HONESTY | PASS |
| PLAN §20.3 SCOPE CONFORMANCE + PO decisions | PASS |
| PATTERNS + llms.txt CONVENTION + quality | PASS |
| Adversarial: agent-consumable layer | CONCERN (P1 confirmed) |
| Adversarial: completeness | PASS |

## Findings

### P1 — False "closed 3-method" bridge cardinality claim (CONFIRMED by dimension + adversarial)
The docs asserted `web/src/bridge/protocol.ts:BridgeMethodSchema` is a "closed
3-method enum" / "three-method whitelist". Ground truth: the enum has **16
members** (`task_submit, storage_get, storage_set, pii_redact, storage_list,
storage_delete, identity_pubkey, node_status, browse_list, storage_version,
provenance_get, provenance_verify, feed_cursor_get, search, proof_card_get,
task_result`); `web/public/sbfb-bridge.js` exposes all 16. The load-bearing PO #4
point (NO shard method) is TRUE — but the "3" is stale Sprint-13 lore that
misleads an agent about the real bridge surface. The source-ref-check passed
silently because it validates only that the symbol resolves, never a cardinality
claim about it. Locations: `WIRING_SPEC.md`, `llms.txt`, `examples/bridge_gap.md`.
Note: `HOW_TO_WIRE.md:19` (Phase M) correctly says "une whitelist de méthodes"
(no number) — the error is Phase-N-introduced (it propagated from the preflight
S2 scan, which read only protocol.ts:20-23).

### P2 — Honesty-gate not grep-enforced on either llms.txt (CONFIRMED)
Plan §20.4 requires PROVISIONAL/S78 grep-enforced "dans M ET N"; preflight
watch-item 8 says extend the honesty-gate to WIRING_SPEC + llms.txt. The original
`require_marker` calls pinned the markers on `WIRING_SPEC.md` ONLY; a regression
stripping PROVISIONAL/S78 from either llms.txt passed CI silently (confirmed by a
marker-strip regression test → still EXIT 0).

### P2 — Source-ref symbol resolution is substring-only (latent, ACCEPTED tradeoff)
`grep -qF "$sym" "$path"` has no word boundary / definition lead-token — a
deliberate BusyBox-safety choice (no `grep -P`, no `\b`) so the gate runs on the
`bash:5` CI image. A renamed symbol whose old name lingers in a comment would
silently resolve. Did NOT trigger on any current anchor (all 51 resolve to real
definitions). Accepted as a documented BusyBox tradeoff.

### P3 — `## Optional` H2 carried a parenthetical suffix (cosmetic) — FIXED
A strict llmstxt.org parser keys the skippable section on exact `Optional`.

### P3 — Drift-guard example not covered by a rustfmt gate (cosmetic, ACCEPTED)
`docs/sharding/examples/sign_verify.rs` is outside `cargo fmt --all`'s reach; API
drift IS still caught by compile via `include!` (the load-bearing guard). Adding
`rustfmt --check` to the doc-lint is REJECTED: the doc-lint runs on the `bash:5`
CI image, which has no Rust toolchain — coupling it would break the gate.

## Adversarial verification result
Both verifiers converged with the dimensions. CONFIRMED: the P1 (independent read
of protocol.ts = 16 members + live doc-lint exit 0 despite the false claim) and
the P2 honesty-gate gap (live marker-strip regression). Independently re-verified
PASS: verbatim lift byte-identical (`sign_verify.rs` vs `shard_plan.rs:602-700`),
2 tests pass under nextest via `include!`, contiguity≠coverage NOT regressed,
confidentiality not overclaimed (`project_shard_session` emits only
session_id+member_count), both preflight anchor-traps avoided. No new P0/P1.

## Scope cuts respected? (plan §20.3 livrables all present)
Yes — (a) docs/sharding/llms.txt index (Truth-Stack + PROVISIONAL/S78 + Not
evidenced + links + Optional); (b) root llms.txt sharding-only (PO #3 banner);
(c) WIRING_SPEC.md 5 fixed-order sections; (d) examples sign_verify.rs (verbatim,
runs under nextest) + observe.curl.md (member_count-only) + bridge_gap.md
(PROPOSED/GAP-not-shipped, PO #4). Contiguity≠coverage NOT reintroduced.

## Invariants
0 functional Rust (only the include! harness + lifted example) · 0 wire bump
(FORMAT_VERSION stay 1) · 0 new dep (no Cargo.toml/lock change) ·
consumed-never-authoritative (no PASS/GREEN/verdict language; grep = 0) ·
source-ref validates rank-1 only (`crates|docs|web|scripts`, `.planning/active/`
NOT captured → no dangling anchor after archive). Honesty grep-enforced: NOW full
(WIRING_SPEC + both llms.txt) after the P2 fix.

---

## Réconciliation (Claude) — CONCERN → PASS-PENDING

Tous les items bloquants/recommandés du Workflow sont traités :

- **P1 (3-method) — CORRIGÉ** dans les 3 fichiers : `WIRING_SPEC.md`
  (« closed enum of app-facing methods, none of which is a shard method »),
  `llms.txt` (idem), `examples/bridge_gap.md` (liste maintenant les 16 méthodes
  réelles, pointeur canonique vers `protocol.ts`, supprime « exactly the three
  methods above »). Grep anti-régression : **0** occurrence résiduelle de
  `3-method`/`three-method` dans `docs/sharding/` + `llms.txt`.
- **P2 (honesty-gate llms.txt) — CORRIGÉ** : ajout de
  `require_marker "$SHARD_LLMS" "PROVISIONAL"`, `… "S78"`, et
  `require_marker "$ROOT_LLMS" "sharding subsystem only"`. Test négatif :
  strip PROVISIONAL → `check-sharding-docs.sh` **EXIT 1** ; restore → **EXIT 0**.
  Ce lock défend aussi contre tout futur flip « shipped/LIVE-AND-DONE » de la
  bannière (incident observé pendant la phase).
- **P3 (Optional H2) — CORRIGÉ** : `## Optional` exact + qualificatif en note.
- **P2 substring-only + P3 rustfmt — ACCEPTÉS** comme tradeoffs documentés
  (contrainte BusyBox `bash:5`, garde de drift = compile via `include!`).

Re-vérification post-fix : `bash scripts/check-sharding-docs.sh` **clean**
(links + anchors + honesty + french-body + source-ref) ; suites Rust inchangées
(le fix ne touche que 3 `.md` + 1 `.sh`, aucun `.rs`). nextest example harness
toujours **2/2** (Win + Docker 1.94).

## Codex reconciliation

Codex GPT 5.5 (`codex exec -s read-only`, sandbox read-only) — **4 rounds** jusqu'à
CLEAN. Artefacts bruts (jamais réécrits) : `sprint77_phase_n_codex_review.md`
(= round 4, verdict sur le code committé) + trail `_round1/_round2/_round3`.

- **Round 1 → 3 gaps** : (a) faux « closed 3-method enum » du bridge (réalité 16
  méthodes, `protocol.ts:20-49`) — corrigé dans WIRING_SPEC + llms.txt +
  bridge_gap.md (propriété drift-proof « none is a shard method » + liste réelle) ;
  (b) root llms.txt débordait du scope sharding-only (section « Project
  orientation ») — retirée (banner prose conservé) ; (c) bridge_gap.md
  sur-engageait « frozen for S78 » + payload concret — adouci en placeholder
  illustratif non-contractuel.
- **Round 2 → 1 P2** : caps SIGN + OBSERVE non ancrés `path:Symbol` — ancrés
  (`SESSION_ID_MAX`/`SHARD_GROUP_ID_MAX`/`SHARD_HASHES_MAX`, OBSERVE
  `shard_session_response`/`project_shard_session`).
- **Round 3 → 2 items** : (a) ancre manquante pour l'ordering
  `is_member`-avant-`accept_bi` (claim sécurité) — ancrée
  (`shard.rs:is_member` + `shard.rs:accept_bi`) ; (b) le source-ref-check valide
  les refs présentes mais n'exige pas qu'une clause porteuse AIT une ref —
  fermé en **classe** par un **required-anchor check** (allowlist de 11 symboles
  porteurs qui DOIVENT apparaître dans WIRING_SPEC ; test négatif : strip → EXIT 1).
- **Round 4 → OVERALL CLEAN** (0 P0/P1/P2 ; 8/8 livrables CLEAN). Note Codex : son
  sandbox ne peut exécuter le script (pas de WSL) — exécuté localement (vert) +
  suites Rust Win/Docker vertes.

Suites relancées après chaque correction (doc-lint clean ; les fixes round 1-4 ne
touchent aucun `.rs` fonctionnel, donc nextest/clippy/fmt Rust inchangés et verts).

**Verdict final : PASS.** Prêt à commit.
