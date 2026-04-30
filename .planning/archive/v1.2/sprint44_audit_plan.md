# Sprint 44 — Audit plan (Sprint 43 post-mortem)

**Auditeur** : session fraiche (pas la session qui a code S43).
**Tip d'entree** : `0ec0458` (S43 Phase C, dernier feat commit).
**Documents source** : `sprint43_kickoff.md` (D1..D3) +
`sprint43_plan.md` (§Phase A, §Phase B, §Phase C) +
`sprint43_verification.md` (28/28 fail-fast).

---

## Mode d'emploi

Lire dans l'ordre : (1) ce fichier, (2) sprint43_plan.md,
(3) sprint43_kickoff.md §D1..D3. Ne PAS lire le code source
avant d'avoir parcoure les tracks ci-dessous et forme une
opinion. Timebox : 2-3h. Livrable : `sprint43_audit_findings.md`.

## Track A — MANDATORY batch (Phase A)

- [ ] A-1 : conn() pub(crate) — verifier que le mot-cle est bien
  `pub(crate)` et que tous les appelants compilent.
- [ ] A-2 : persist error — verifier tracing::warn! sur les 2
  sites (canary_registry.rs:158,168).
- [ ] A-3 : Mutex consolidation — verifier que ReloadState struct
  contient les 3 champs et que maybe_reload/reload_policy/reload_set
  utilisent self.reload au lieu des anciens champs.
- [ ] A-4 : rerun hash — verifier blake3::hash au lieu de
  DefaultHasher. Test simple_hash_deterministic present.
- [ ] A-5 : MintRequest::new() — verifier constructor existe et
  que mk_req dans les tests l'utilise.
- [ ] A-6 : URL single-quote — grep confirme 0 instance.
- [ ] A-7 : LOC kickoff — plans S43 ne contiennent pas d'estimation
  LOC prospective.

## Track B — Files + consent API (Phase B)

- [ ] B-1 : consent.rs — verifier 4 routes (get/set/whitelist add-remove),
  validation node_id hex 64, atomic write (tmp+rename), threat notes
  per level 1-4.
- [ ] B-2 : files.rs — verifier CAS SHA-256, 3 routes (upload/manifest/stream),
  validation sha256 hex 64, MAX_UPLOAD_BYTES 50MB.
- [ ] B-3 : header injection fix — verifier sanitisation CRLF+quotes
  sur original_name et content_type dans upload_file.
- [ ] B-4 : routes enregistrees dans http.rs — 7 routes consent+files.
- [ ] B-5 : sha2 dep — verifier ajout dans Cargo.toml daemon.

## Track C — Canary + contributor API (Phase C)

- [ ] C-1 : canary_api.rs — 2 handlers (inject-rate, observed-divergence).
  CanaryInputManager access via Option dans state (503 si None).
- [ ] C-2 : contributor_api.rs — 3 handlers (verify, list, envelope).
  Utilise ContributorRegistry via CoordinatorDb.
- [ ] C-3 : proxy supprime — proxy_contributor_verify et
  is_64_lowercase_hex supprimes. Test stale bad_gateway supprime.
- [ ] C-4 : canary_input.rs — Debug impl + set_inject_rate() +
  recent_divergences() delegates.
- [ ] C-5 : #[allow(dead_code)] — coord_http_client et
  coord_base_url marques, raison documentee (proxy supprime).

## Track D — Process / meta

- [ ] D-1 : G8 preflights Phase A + B + C — verifier coherence
  (3/3 EXECUTE, 0 DESIGN-CONFLICT).
- [ ] D-2 : scope cuts 6/6 — verifier aucun viole (diff --stat).
- [ ] D-3 : 7/7 MANDATORY items resolus — verifier dans le diff.

## Track E — Doc coherence

- [ ] E-1 : HARDENING_ROADMAP compteurs — verifier 1111 Rust / ~2114 total
- [ ] E-2 : CLAUDE.md etat actuel — verifier S43 CLOSED + carries S44
- [ ] E-3 : SPRINT_LOG.md — verifier row S43 presente
- [ ] E-4 : Phase review files present : 3/3 (A + B + C)
- [ ] E-5 : Phase preflight files present : 3/3 (A + B + C)

---

## Carries S44

| Item | Compteur | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 9+/3 | exemption blocker externe |
| P2-AUDIT-2 pre-release transitives iroh | herite | pin 0.98 |
| P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 | 5/3 | exemption dep S45 |
| P2-REVIEW-A-1-S42 ChainResult mutations target | 2/3 | Phase A review |
| P2-REVIEW-B-1-S42 pow_keypair identity doc | 2/3 | Phase B review |
| P2-REVIEW-B-1-S43 coord dead_code cleanup | 1/3 | Phase C review |
| P3-REVIEW-A-2-S42 babel-scraper untracked | 2/3 | Phase A review |
| P3-REVIEW-C-1-S42 list_apps aggregate probe | 2/3 | Phase C review |
| P3-AUDIT-A-1-S42 couverture RNG rate>1 | 2/3 | S42 audit |
| P3-AUDIT-C-1-S42 Debug vs serde | 2/3 | S42 audit |
| P3-AUDIT-C-2-S42 pagination limit/offset | 2/3 | S42 audit |
| P3-REVIEW-B-1-S43 tests HTTP integration | 1/3 | Phase B review |
| P3-REVIEW-C-1-S43 prefix route /api/contributor sans /v1/ | 1/3 | Phase C review |
| P3-REVIEW-A-1-S43 TOCTOU canary reload | 1/3 | Phase A review |

**Resolus S43** : P2-REVIEW-A-1-S41 conn() pub,
P3-REVIEW-A-2-S39 LOC kickoff, P3-REVIEW-B-2-S39 persist error,
P3-AUDIT-A-1-S39 URL single-quote, P3-REVIEW-B-1-S40 Manager Mutex,
P3-REVIEW-C-1-S40 rerun hash, P3-REVIEW-B-1-S41 MintRequest.

**Note S44 pair** : S44 est pair → phase dette obligatoire
(§6.2.1 Regle 1). Items P2 a 2/3 approchent MANDATORY (3/3 au
prochain carry).

---

## Verdict global attendu

- PASS : 0 P0, 0 P1 → S44 Phase A demarre direct
- CONDITIONAL PASS : 1-3 P1 → fix(sprint43): ... avant S44 Phase A
- FAIL : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- D1..D3 gelees du kickoff (ne pas rebattre)
- Pin iroh 0.98 (Day 0 #3)
- Scope cuts S43 (decision sprint, pas audit)

## Livrable attendu

`sprint43_audit_findings.md` avec : verdict global, section par
track, findings P0→P3, commits fix attendus si CONDITIONAL PASS.
