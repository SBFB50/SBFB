# Sprint 74 Phase G — Review (wrap-up + T14 coverage + S73/D/E carries)

Base : `66a9409` (Phase F). Phase G = `docs(sprint74)` wrap-up/dette. Review
adversariale double-flux : (1) Workflow 9 agents (5 dimensions → verify
adversarial par finding) ; (2) Codex `gpt` claim-by-claim (19 claims, 2 rounds).
Préflight G8 = EXECUTE (`sprint74_phase_g_preflight.md`).

## Flux 1 — Workflow adversarial (9 agents, 5 dimensions)

Dimensions revues : `b2-quorum-arithmetic`, `is-own-flatten-security`,
`web-xss-error-handling`, `t14-coverage-honesty`, `docs-accuracy`. 4 findings
bruts → verify adversarial (refute-par-défaut) → **3 confirmés, tous traités** :

| Sev (corrigée) | Finding | Fichier | Résolution |
|---|---|---|---|
| P2 | Commentaire malhonnête : prétend `bootstrap.ts` « 100% tested » alors qu'il est 50% funcs / 78.57% stmts / 80% branch (seules les LIGNES = 100%). Mode d'échec « masked honesty » que T14 devait fermer. | `web/vitest.config.ts:53-57` | **FIXÉ** — commentaire réécrit avec l'agrégat mesuré réel (86.91/78.63/85.82/88.23) + caveat honnête bootstrap.ts (no-op `.catch` + gardes window/origin non unit-exercées sous jsdom). |
| P2 | Justification du seuil functions attribue le gap « uniquement à BrowsedProject » — incomplet (bootstrap 50%, schema 80%, FileUploadBlock 90% y contribuent). | `web/vitest.config.ts:59-65` | **FIXÉ** — même bloc réécrit, gap réparti et nommé sur les 4 fichiers, agrégat 85.82% clears 85. |
| P3 | Incohérence cross-doc du décompte des sites `BrowseEntry` (rust §P58.2 « ~25 » vs shell §P36 « ~18 » vs http.rs:1234 « 18 » ; grep réel = 26). Non load-bearing. | `docs/rust/PATTERNS.md`, `docs/shell/PATTERNS.md`, `crates/nexus-shell-daemon/src/http.rs:1234` | **FIXÉ** — les 3 reformulés « every / any BrowseEntry construction site » (nombre fragile retiré). |

**0 finding** confirmé sur `b2-quorum-arithmetic`, `is-own-flatten-security`,
`web-xss-error-handling`, `docs-accuracy` STRIDE : l'arithmétique quorum B.2
(redundancy 2/3/4/5, pas d'off-by-one), le flatten-view `is_own` (node_id reste
`#[serde(skip)]`, `.strict()` accepte la clé, fallback nullish correct,
seed-volontaire distant → `is_own=false`), les gardes XSS `isHttpsUrl` (Browse +
VerificationDetail, aucun schéma non-https n'atteint un `href`), la branche
`query.isError` (AVANT le skeleton), et les claims STRIDE THREAT_MODEL §15
(vérifiés contre seed_registry.rs / seed_protocol.rs / public_feed.rs) sont
**corrects** — aucun changement requis.

## Flux 2 — Codex claim-by-claim (19 claims, 2 rounds)

`sprint74_phase_g_codex_review.md`.

- **Round 1 → GAPS (2)** :
  - GAP 4 (claim, non-code) : les tests `seedCount` ne sont PAS ajoutés par ce
    diff (ils datent de Phase F, ligne 700) ; seuls `is_own` + `triggerPanicWipe`
    le sont. → claim corrigé (le code est sain) ; artefacts (body/SPRINT_LOG)
    disent désormais « is_own + triggerPanicWipe ».
  - GAP 18 (**vrai bug doc**) : rust §P58 « `enabled = 0` is the only explicit
    row » est faux — `finalize_deploy` (deploy.rs:466) et `seed_voluntary`
    écrivent des lignes EXPLICITES `enabled = 1` (avec archive_hash), lues au
    boot par `list_keep_online_enabled` (db.rs:742) pour la re-annonce Phase F.
    → **FIXÉ** : §P58 réécrit (les DEUX états sont explicites ; ligne absente =
    enabled-par-défaut pour le gate rebroadcast, R6 fallback).
- **Round 2 → PASS** (19/19 CONFIRMED après les fixes ci-dessus).

## Verification (achievable fail-fast — voir §Env)

- `cargo fmt --all --check` → 0
- `cargo clippy --workspace --all-targets --locked -- -D warnings` → 0
  (compile TOUT le code Phase G : `BrowseEntryView`, `BrowseListResponse`
  `#[cfg(test)]`, validator B.2)
- `cargo build -p nexus-shell-daemon --release` → 0 warning
- doctests workspace → 0
- Phase G test Rust (`validator::tests::quorum_impossible_before_full_count_rejects_early`) → **PASS**
- crates non-networked (coordinator + events + trace + factory + manifest) → voir `sprint74_verification.md`
- Web : lint 0 / tsc 0 / Vitest **331** / **`test:coverage` GREEN** (86.91 stmts /
  78.63 branch / 85.82 funcs / 88.23 lines ≥ 85/85/78/85, isolé après une race
  `.tmp` causée par un agent review concurrent) / build OK / size 6/6 / scan FR clean

## Env (blocage de session, transparence)

Réseau hôte dégradé → tout test qui monte un nœud iroh (`create_node` →
relay/holepunch) hang 25-90s ; WSL wedgé (`wsl -l -v` hang) → Docker engine 500.
Récupérer exigerait `wsl --shutdown` (interdit, casse Docker) ou un reboot
machine (hors-portée autonome). Les tests iroh-networked sont au MÊME commit base
que Phase F (dual-platform vert 1674 Win / 1678 Linux) et le code Phase G NE
TOUCHE AUCUN chemin iroh (B.2 = SQLite pur ; `is_own` = serialize-only ; le reste
= web + docs). La re-vérification dual-platform complète (Docker canonique + suite
iroh) est différée à la récupération env, AVANT tout push (`feedback_wsl_before_push`:
Docker exigé avant PUSH, pas avant commit ; ces commits restent locaux).

## Verdict: PASS
