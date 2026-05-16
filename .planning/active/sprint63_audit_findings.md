# Sprint 63 — Audit findings

**Auditeur** : session fraiche Claude Opus 4.6, 2026-05-16.
**Tip audite** : `07a1d02` (HEAD, 4 commits post-tip reference `7198ae5` —
2 fix(examples) + 2 chore(planning) post-S63 Phase D).
**Audit plan** : `sprint64_audit_plan.md` (6 tracks).
**Timebox** : ~25 min.

---

## Verdict global : PASS

- 0 P0
- 0 P1
- 2 P2 (rigor signal G4 satisfait)
- 1 P3

---

## Track 1 — Provenance pipeline integrity : PASS

| Check | Evidence |
|---|---|
| M12 schema | `db.rs:178-194` — CREATE TABLE provenance_records, 9 colonnes, UNIQUE(project_id, artifact_hash), index idx_prov_project. Schema identique kickoff D1. |
| Insert timing | `deploy.rs:222-234` — insert APRES blob store (commentaire explicite fix `5f6a77d`). Non-fatal (debug! log). |
| Endpoint 200 | `http.rs:1728-1743` — record JSON + verified (live Ed25519) + provenance_hash. |
| Endpoint 404 | `http.rs:1745-1748` — "no provenance record for this project". |
| Hash linkage | `deploy.rs:222` provenance_blake3_hex → `deploy.rs:245` announcement.with_provenance_hash(). Chaine intacte. |
| Bridge relay | `useBridge.ts:324-357` — 3 dispatch cases, encodeURIComponent(pid) pour path safety. authFetch authentifie. |
| Auth tier | Endpoint sous `authed_routes` (`http.rs:356-357` dans bloc 256-419, middleware auth_required ligne 419). |

---

## Track 2 — MANDATORY 3/3 resolution : PASS

| Check | Evidence |
|---|---|
| IMAGE-DEP closed | `nexus-launcher/Cargo.toml:21` : `png = "0.18"`. `cargo tree -p nexus-launcher | grep image` → 0 match. Crate `image` completement elimine. |
| tray.rs adaptation | `tray.rs:14-27` : `png::Decoder::new()` → `read_info()` → `next_frame()` → `Icon::from_rgba()`. API native sans abstraction. |
| PLAYWRIGHT closed | `global-setup.ts:1-125` : spawn `nexus-shell-daemon --config <path> init` + `start`. Zero reference Python. Binary lookup: env var → release → debug → PATH fallback. Health poll 500ms avec timeout 30s. |
| global-teardown | State file `.playwright-state.json` avec PID pour kill propre. |

---

## Track 3 — UI proof-chain coherence : PASS

| Check | Evidence |
|---|---|
| 7 champs | `VerificationDetail.tsx:181-243` : repo_url (lien cliquable), commit_sha (tronque+copie), artifact_hash, signature, node_id, timestamp, schema_version. |
| Lazy fetch | `VerificationDetail.tsx:82-87` : useEffect declenche par `open`, pas au mount page. Race-condition guard via fetchIdRef. |
| Verify live | `VerificationDetail.tsx:101-107` : bouton "Reverifier" refetch + re-render. |
| Hash mismatch | `VerificationDetail.tsx:118-121` : compare provenance_hash retourne vs annonce reseau, warning amber si divergent. |
| Badge cliquable | `BrowsedProject.tsx:273-282` : `<button onClick={() => setVerifyOpen(true)}>`. Conditionne par `entry.provenance_hash`. |
| shadcn Dialog | `VerificationDetail.tsx:124` : import depuis `@/components/ui/dialog`. Composant standard. |

---

## Track 4 — Protocol Explorer demo : PASS

| Check | Evidence |
|---|---|
| Section verification | `examples/sbfb-explorer/app.js:174-235` : select projet → verifyRelease() → render resultat. |
| Interactive demo | `app.js:187-188` : `bridge.verifyRelease(projectId)` via bridge postMessage. |
| Hash mismatch detect | `app.js:190-194` : compare `data.provenance_hash` vs `announceHash` du select option. |
| escapeHtml | `app.js:237-239` : remplace `&`, `<`, `>`. |
| escapeAttr | `app.js:242-244` : remplace `&`, `"`, `<`, `>`. Single quote non echappee (P2 documente). |
| XSS via noms projet | `app.js:221-225` : innerHTML construit avec escapeHtml(value) + escapeAttr(title). Safe pour double-quoted attributes. |

---

## Track 5 — Process compliance : PASS

| Check | Evidence |
|---|---|
| 4 preflights G8 | `sprint63_phase_{A,B,C,D}_preflight.md` presents (25-26 lignes chacun). 4x verdict EXECUTE confirme par grep. |
| 4 reviews PASS | `sprint63_phase_{A,B,C,D}_review.md` presents. Verdicts PASS confirmes par grep. |
| Design review G1 | `sprint63_design_review.md` present. Scoring D1⚠️ D2✅ D3✅ D4⚠️ D5✅. |
| Commit discipline | 4 feat commits pattern correct (`feat(scope): Sprint 63 Phase X — titre`). 2 fix commits (`fix(feed): ...`). Bodies riches avec delta tests cumules + scope cuts. |
| Scope cuts | verification.md §4 : 9/10 non touches, 1 (Protocol Explorer) livre Phase D. 0 scope creep. |
| Bridge SDK sync | SHA256 identique web/public/ = explorer/ = ideas/ (`7051AD4A`). |

---

## Track 6 — Carries S64 : PASS

Audit plan §6 documente 16+ carries avec compteurs. Verification :

- F1 P2-VERSION-NOT-STORED (3/3 MANDATORY) : confirme, `provenance_records` n'a pas de colonne `version`.
- F5 P2-IROH-INFRA-TIMEOUT (3/3 MANDATORY) : confirme, tests SBFB_INTEGRATION gate non modifie S63.
- P2-EXPLORER-ESCAPE-SINGLE-QUOTE (1/3) : confirme presence dans code (app.js:242-244).
- Autres carries (P2-PROVENANCE-404-BRIDGE, P2-BADGE-WORDING-PREMATURE, etc.) : documentes, non touches S63.

---

## Findings

### P2 — VITEST-DELTA-COMMIT-BODY-DRIFT

Phase C commit `272523c` body annonce "Vitest: 258 -> 264 (+6: 3
bridge dispatch + 3 VerificationDetail)" mais livraison reelle = +7
(258→265, incluant +1 hash mismatch test). Discrepance corrigee dans
le commit Phase D ("Phase C +3 bridge dispatch + 3 VerificationDetail
+ 1 hash mismatch"). verification.md §2 a le bon compte. Impact nul
(verification.md autoritatif) mais discipline commit body incomplete.

**Evidence** : `git log --format="%b" 272523c | grep "Vitest"` →
"258 -> 264 (+6)" vs verification.md ligne 39 "258 → 265 (+7)".

### P2 — EXPLORER-ESCAPE-SINGLE-QUOTE (re-confirme)

`examples/sbfb-explorer/app.js:242-244` — escapeAttr echappe `&`,
`"`, `<`, `>` mais pas `'` (single quote). Contextually safe (tous
les attributs sont double-quoted dans le template) mais incomplet
en defense-in-depth. Deja documente comme carry dans audit plan §6.

**Evidence** : `app.js:244` →
`return str.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");`

### P3 — PHASE-C-BODY-MINOR-UNDERCOUNT

Nit : le commit body Phase C dit "+6 Vitest" (omet le hash mismatch
test). Auto-corrige Phase D. Aucune action requise.

---

## Compteurs verifies

| Suite | Attendu (audit plan §3) | Observe | Match |
|---|---|---|---|
| Rust nextest | 1305 | 1305 pass (0 skip) | ✓ |
| Vitest | 265 | 265 pass | ✓ |
| size-limit | 6/6 | 6/6 | ✓ |
| Total | ~1576 | ~1576 | ✓ |

---

## Commits fix attendus

Aucun — verdict PASS, 0 P0, 0 P1.

---

## P2 a logger en carry S64

- P2-VITEST-DELTA-COMMIT-BODY-DRIFT : nit process, aucune action code.
  Carry optionnel (le planner S64 peut ignorer).
- P2-EXPLORER-ESCAPE-SINGLE-QUOTE : deja carry (1/3 → planner S64+).

---

## Notes on audit completeness

- 6/6 tracks auditees exhaustivement.
- Tests rejoues live (Rust nextest + Vitest + size-limit) — tous verts.
- Pas de Playwright specs rejouees (setup operationnel confirme par
  verification.md mais specs elles-memes stale — carry
  P2-PLAYWRIGHT-SPECS-STALE documente).
- Security : provenance endpoint sous auth_required (verified
  http.rs:256-419 authed_routes block). Bridge dispatch utilise
  encodeURIComponent. SQL parametrise (rusqlite params![]). 0 unsafe
  nouveau. 0 secret expose.
