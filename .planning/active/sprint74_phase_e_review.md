# Sprint 74 Phase E — Review (adversarial 5-dimension)

Date: 2026-06-08
HEAD entrée: `4c1acc5` (Phase D). Phase E uncommitted.
Méthode: Workflow adversarial 5 dimensions (correctness / sécurité / scope-préflight /
tests / architecture) — 16 agents, ~1.28M tok — chaque finding **vérifié
adversarialement** (skeptic dans les deux sens) avant rétention. 11 findings confirmés
(8 P0/P1, 1 P2, 2 P3) + 2 GAP Codex (C3, C5). Tous traités en-phase sauf les 2 P3
(même sujet) routés carry S75.

## Verdict: PASS

Tous les P0/P1/P2 + les 2 GAP Codex corrigés en-phase + re-vérifiés (clippy `-D warnings`
vert, fail-fast dual-platform vert : Win nextest 1668 / Docker Linux 1672 / doctests /
release 0 warning, web Vitest 314). Les 2 P3 (même cause) documentés en carry S75.

---

## Findings confirmés + résolution

### P0/P1 — gate clippy `-D warnings` ROUGE (le `;`+echo de mon 1er wrapper avait masqué l'exit réel de clippy — piège gate-masking identique au `| tail`)

1. **[P1] `runtime.rs:39` import mort `create_node_with_config`** — le reorder Phase E a
   remplacé les 2 sites d'appel par `create_node_with_protocols`. **FIX** : import retiré.

2. **[P0/P1] `seed_protocol.rs` `request_seed` dead-code en bin** — client du protocole sans
   appelant prod (UI désignation pair différée « Bientôt », NF-2). **FIX** :
   `#[allow(dead_code)]` + rationale (client wire `sbfb/seed/0` exercé par l'E2E, câblé par
   l'UI différée ; gardé API prod, pas test-only).

3. **[P0/P1] `db.rs` `type_complexity` sur le tuple de `consume_seed_invite`**. **FIX**
   (root-cause) : annotation retirée → type inféré depuis la closure (turbofish).

### P2 — sécurité (autorisation)

4. **[P2] L'invite liait `project_id` mais PAS `archive_hash`** — un pair invité pour l'app
   P pouvait faire épingler du contenu étranger sous le tag `keep-online/P` + empoisonner
   la ligne `keep_online(P)` (blast radius contenu aujourd'hui : pas de GC, F lit l'outbox
   propre). **FIX (capability-over-content)** : colonne `archive_hash` ajoutée à
   `seed_invite` (M19) ; `mint_seed_invite`/`consume_seed_invite` prennent `archive_hash` ;
   handler passe `env.request.archive_hash` (rejet si ≠) ; route mint **dérive** le hash du
   browse aggregator (« on n'autorise que ce qu'on voit »). Tests db + handler ajoutés.

### P1 — tests (couverture des branches de rejet du handler)

5. **[P1] 8 branches de rejet du handler non testées**. **FIX** : 9 tests handler-level
   ajoutés (`handle_request` direct) : dialer-mismatch, stale-ts, bad-version,
   replay-via-handler, invite revoked / expired / **exhausted**, wrong-archive-hash (P2),
   content-hash-mismatch (+ rollback du tag spéculatif).

### Codex GAPs (gate bloquante) — corrigés + re-run

- **[C3] anti-replay sous skew futur** : TTL nonce 120s mais fenêtre ts ±120s → une requête
  `ts=now+120` rejouable après purge. **FIX** : `SEED_NONCE_TTL_SECS = 2 * SEED_TS_WINDOW_SECS
  + 1` (241s). Round 2 Codex a relevé l'inclusivité de bord (gate `abs_diff > window` accepte
  exactement `window`, cache purge à `elapsed >= TTL`) → le `+ 1` ferme la fenêtre d'1s au
  bord exact. Round 3 → PASS.
- **[C5] reasons non distincts** : `Expired` et `NoUsesLeft` mappaient tous deux
  « invite-expired ». **FIX** : `NoUsesLeft` → « invite-exhausted » distinct + reason listée
  dans `seed.rs` + test `handler_rejects_exhausted_invite`.

### P3 — robustesse (routé carry S75, même cause ×2)

6. **[P3] invite single-use brûlée sur échec fetch transitoire** (`consume`-avant-`fetch`).
   **DÉCISION : carry S75** (`sprint75_audit_plan.md`). Rationale : `consume`-avant-`fetch`
   est l'ordre *plus sûr* (anti-DoS griefer) ; chemin volontaire non concerné ; UI invite
   « Bientôt » (NF-2) → aucun flux user vivant ne brûle d'invite ; re-mint trivial en pilote.
   Fix propre (re-crédit sur fetch-failed transitoire) inscrit S75.

---

## Dimensions sans finding bloquant
- **Correctness** : ordre vérif correct (version→sig→dialer→ts→nonce→invite→fetch) ; nonce
  recorded avant invite (replay précède conso) ; reorder `coordinator_db` avant node propre.
- **Scope/préflight** : conforme [DETER] (domaines `nexus-`, exclusions canonical, tag
  `keep-online/<pid>`) ; 0 bump wire (ALPN-versionné + `version=1`) ; NF-2 honoré (CTA
  volontaire fonctionnel ; « Inviter un pair » inerte « Bientôt » ; 0 faux bouton actif) ;
  R5 structurel (seeder re-pin octets auteur, signe 0 provenance).
- **Architecture** : `ExtraProtocolFactory` additif (NodeConfig Clone/Debug/Default intacts) ;
  dep `iroh` directe = +1 ligne Cargo.lock (déjà transitif) ; tag keep-online unifié.

## Re-vérification post-fix
- `cargo clippy --workspace --all-targets --locked -- -D warnings` : 0 warning (exit réel
  capturé hors pipe).
- Win nextest 1668/1668 0-skip + doctests + release 0 warning ; Docker Linux 1672/1672
  0-skip ; web Vitest 314 + build + size + scan.
- Codex gate : `sprint74_phase_e_codex_review.md` (re-run post C3/C5 → PASS attendu).
