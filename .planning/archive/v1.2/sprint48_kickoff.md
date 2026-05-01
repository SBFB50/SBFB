# Sprint 48 — Kickoff (dette pair carries resolution batch)

**Ecrit** : 2026-05-01 (post-audit gate S47 PASS `3d14068`).
**Type** : **sprint pair** — phase dette obligatoire
(§6.2.1 Regle 1).
**Tip master d'entree** : `3d14068`.
**Phase 0 audit Sprint 47** : **DEJA JOUE** — `3d14068` PASS
(0 P0, 0 P1, 0 new P2, 1 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-01 (S46 meme jour).
  0 trigger actif. 11 triggers surveilles : iroh > 0.98, wasmtime
  LTS bump, arti-client > 0.41, Tor PoW hspow, NIST PQC FIPS,
  NVIDIA H100 CCM, frost-ed25519 > 3.0, RFC 9591, openai-agents >
  0.7.0, MCP spec, microsoft/sudo — aucun realise. Pas de
  pre-research.

- **Technologies S48** : aucune nouvelle dep externe. Sprint dette
  pair + carries batch. Memes patterns Rust que S47. Seul ajout
  potentiel : feature gate `test-support` (zero dep externe).

- **ROADMAP_COMMITMENTS check** : LT-1 reclassifie pre-v1.0
  cible S50+. LT-2..LT-5 latents (tag v1.0 non pose). LT-6
  RESOLVED. 0 condition declenchee.

- **HARDENING_ROADMAP §3** : pas de ligne S48 prescrite. Pas de
  drift a documenter.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 47 CLOSED + audit PASS. S47 a livre :
- Phase A : 3 items S45 2/3 CLOSED (diagnostic Err test, invite
  ID collision fix node_id prefix, 7 Python dead code modules
  supprimes)
- Phase B : 9 integration tests 5 routes (deploy/apps/auth)
- Phase C : 7 happy path tests consent/files + 3 deprecated
  aliases supprimes

**9 carries S48** documentes dans `sprint47_audit_findings.md` :
- 3 items a 2/3 (TOCTOU canary, kudos SQL, app-specific schema)
- 4 items NEW S47 a 1/3 (execute_batch_raw, invite format,
  BlobsClient fragility, set_var)
- 2 items exemption permanente (rand upstream, iroh transitives)

S48 pair → **phase dette obligatoire** (§6.2.1 Regle 1). Les 3
items a 2/3 deviennent MANDATORY S49 si non resolus. 2 d'entre
eux sont resolvables dans ce sprint (TOCTOU, kudos SQL). Le 3e
(app-specific schema) a une exemption valide (dep App Runtime
Migration).

### §1.2 Ancrage HARDENING_ROADMAP

Pas de ligne S48 prescrite. 0 trigger actif. Pas d'item drift
a documenter.

### §1.3 Compteurs tests entree (tip `3d14068`)

| Suite | Count |
|---|---|
| Rust nextest | 1185 |
| Rust doctests | 6 passed, 1 ignored |
| SDK pytest | 195 |
| Coord pytest | 264 + 17 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 5/5 |
| **Total** | **~1936** |

### §1.4 Pre-launch protocol policy (rappel)

`*_FORMAT_VERSION` restent a 1. S48 ne touche pas de wire format
canonical. Pas de tolerant decoder multi-version. Sprint dette +
carries — pas d'impact wire.

---

## §2 Goal

Resoudre les items dette 2/3 avant escalade MANDATORY S49
(TOCTOU canary + kudos SQL pagination), fermer les 4 carries
NEW S47 (execute_batch_raw pub, invite format test, set_var
process-wide, deploy BlobsClient reclassification), et
documenter l'exemption app-specific schema drift.
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase C wrap-up.**

---

## §3 Phase 0 — Audit gate S47

**DEJA JOUE** : commit `3d14068` PASS (0 P0, 0 P1, 0 new P2,
1 P3). Audit findings dans
`.planning/active/sprint47_audit_findings.md`. 9 carries
documentes pour S48 (cf. §6 ci-dessous).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — TOCTOU canary reload : mutex-hold-across-read

**Retenu** : modifier `reload_policy()` et `reload_set()` dans
`canary_input.rs` pour garder le verrou `reload` pendant la
lecture du fichier (`read_to_string`). Actuellement le lock est
drop avant le read (lignes 514-515 et 541-542), creant une
fenetre ou le fichier peut etre modifie entre le check mtime et
la lecture. Le fix consiste a lire le fichier sous lock. Le
verrou est detenu ~1ms max (lecture fichier local). Pas de
contention observable — un seul thread appelle `maybe_reload()`
par cycle dispatch.

**Rejete** :
- ArcSwap : ajoute une dep externe pour un pattern qui se resout
  par un hold de lock trivial. Disproportionne pour un singleton
  a reload rare.
- Read-then-compare : lit le fichier d'abord puis compare le
  mtime sous lock. Correct mais inverse la semantique du mutex et
  ajoute des lectures fichier inutiles quand le mtime n'a pas
  change.
- Statu quo : l'item atteint 2/3, fenetre microseconde mais le
  pattern est faux par construction (race verifiable en theorie).

**Implications code** : `canary_input.rs` (reordonnancement lock
scope dans `reload_policy()` et `reload_set()`).

### D2 — kudos SQL pagination : total_count avant skip/take

**Retenu** : capturer `let total_count = all_entries.len()` dans
`kudos_api.rs` AVANT d'appliquer `.skip(offset).take(limit)`.
Ajouter `total_count` au JSON de reponse. Le frontend
`KudosTab.tsx` affiche `total_count` au lieu de `count` pour le
nombre d'entrees. Pas de query SQL supplementaire —
`list_kudos_entries()` ramene deja toutes les lignes en memoire.

**Rejete** :
- SQL COUNT(*) separe : double query inutile puisque toutes les
  lignes sont deja en memoire cote Rust.
- Garder count=page_size : UX incorrecte — utilisateur avec 150
  entrees voit "100 entree(s)" au lieu de "150 entree(s)".
- Pagination SQL-side (LIMIT/OFFSET SQL) : refactoring plus large
  qui touche la couche DB. Correcte a long terme mais
  disproportionnee pour le fix UX immediat.

**Implications code** : `kudos_api.rs` (total_count capture),
`KudosTab.tsx` (affichage total_count).

### D3 — execute_batch_raw : feature gate test-support

**Retenu** : ajouter `[features] test-support = []` dans
`nexus-coordinator-rs/Cargo.toml`. Gater `execute_batch_raw` avec
`#[cfg(any(test, feature = "test-support"))]` pour qu'il soit
visible en mode test du crate ET quand la feature est activee.
Activer `test-support` dans `nexus-shell-daemon/Cargo.toml
[dev-dependencies]`. Cela rend la methode inaccessible aux
consommateurs normaux tout en preservant le test cross-crate
`diagnostic_fairness_returns_500_on_corrupted_db`.

**Rejete** :
- `pub(crate)` simple : casse le test dans nexus-shell-daemon
  (autre crate) qui appelle `db.execute_batch_raw(...)`.
- `conn()` expose : `conn()` est deja `pub(crate)`, pas
  accessible depuis nexus-shell-daemon non plus.
- Copier le SQL dans le test : fragile, le SQL "DROP TABLE kudos"
  est un detail d'implementation de db.rs.
- Laisser pub + #[doc(hidden)] : le carry existe parce que pub
  est trop permissif.

**Implications code** : `nexus-coordinator-rs/Cargo.toml` (feature),
`db.rs` (cfg gate), `nexus-shell-daemon/Cargo.toml` (dev-dep
feature).

### D4 — set_var : refactor sbfb_home dans DaemonHttpState

**Retenu** : ajouter `sbfb_home: Option<PathBuf>` dans
`DaemonHttpState`. Modifier `consent.rs` et `files.rs` pour
utiliser `state.sbfb_home.clone()` quand `Some`, sinon fallback
sur `std::env::var("SBFB_HOME")` puis `~/.sbfb/`. Les tests
passent le path via `mk_state()` enrichi — les 7 appels
`std::env::set_var("SBFB_HOME", ...)` dans http.rs sont
supprimes. L'implementation runtime (daemon reel) passe
`sbfb_home: None` → le fallback env var / home dir est preserve.

**Rejete** :
- temp_env crate : wrapper cosmétique autour de set_var, ne
  resout pas le probleme fondamental (process-wide mutation). Le
  risque d'UB reste si cargo test est utilise au lieu de nextest.
- unsafe set_var avec commentaire : accepte le risque au lieu de
  l'eliminer. Rust 1.81+ flag set_var comme unsafe — le code
  devra ajouter des unsafe blocks a terme.
- Statu quo : fonctionne aujourd'hui avec nextest (process-per-
  test) mais fragile si le mode d'execution change.

**Implications code** : `http.rs` (DaemonHttpState + mk_state +
7 tests), `consent.rs` (sbfb_home param), `files.rs` (sbfb_home
param), `auth.rs` (4 set_var a evaluer).

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 4, 0 ❌).

D3 ⚠️ (cfg(test) cross-crate non documente) : le reviewer
signale que le rationale "Rejete pub(crate)" ne mentionne pas
explicitement que `#[cfg(test)]` en Rust est per-crate —
invisible depuis un autre crate meme en mode test. C'est la
raison technique fondamentale de la feature gate. Decision :
acknowledge — le detail est correct et ajoute a la comprehension
du choix. Le plan §B.1 documente le mecanisme complet.

---

## §5 Plan Phase outline A..C

### Phase A — dette pair 2/3 items resolution

**But** : resoudre les 2 items 2/3 resolvables (TOCTOU canary +
kudos SQL pagination) et documenter l'exemption du 3e (app-
specific schema drift). Sprint pair → phase dette obligatoire.
- P3-AUDIT-B-4-S45 TOCTOU canary reload : mutex hold-across-read
  dans `canary_input.rs`
- P2-REVIEW-B-1-S46 kudos SQL pagination : total_count +
  frontend fix `KudosTab.tsx`
- P2-REVIEW-C-1-S46 app-specific schema drift : exemption
  documentee (dep App Runtime Migration multi-sprint, bloqueur
  externe clair)
- Commit : `feat(sprint48): Sprint 48 Phase A — dette pair
  TOCTOU canary fix + kudos total_count + schema drift exemption`

### Phase B — S47 carries batch

**But** : fermer les carries NEW S47 par fix ou reclassification
documentee.
- P2-REVIEW-A-1-S47 execute_batch_raw : feature gate
  test-support
- P2-REVIEW-A-2-S47 invite format test : assertions pattern dans
  test existant
- P2-REVIEW-C-1-S47 set_var : refactor sbfb_home dans
  DaemonHttpState
- P2-REVIEW-B-1-S47 deploy BlobsClient fragility :
  reclassification documentee (risque inherent a mk_state(),
  partage par 50+ tests, fix = refactoring majeur test infra,
  accepte pre-v1.0)
- Commit : `feat(sprint48): Sprint 48 Phase B — S47 carries
  batch execute_batch_raw gate + invite test + sbfb_home refactor`

### Phase C — Wrap-up

---

## §6 Items carry/dette

### Resolus S48 (plan)

- [plan] **P3-AUDIT-B-4-S45** TOCTOU canary reload **2/3** :
  Phase A (mutex hold-across-read)
- [plan] **P2-REVIEW-B-1-S46** kudos SQL pagination **2/3** :
  Phase A (total_count + frontend)
- [plan] **P2-REVIEW-C-1-S46** app-specific schema drift **2/3** :
  Phase A (exemption documentee — dep App Runtime Migration.
  Bloqueur : routes /app/* dependent de Python coordinator
  AppContext/commands/state. Exemption valide : dependance
  sequentielle interne multi-sprint. L'item ne peut pas atteindre
  3/3 tant que le bloqueur existe. Reclassifie hors compteur
  carry actif.)
- [plan] **P2-REVIEW-A-1-S47** execute_batch_raw pub **1/3** :
  Phase B (feature gate)
- [plan] **P2-REVIEW-A-2-S47** invite format test **1/3** :
  Phase B (assertions)
- [plan] **P2-REVIEW-C-1-S47** set_var process-wide **1/3** :
  Phase B (sbfb_home refactor)
- [plan] **P2-REVIEW-B-1-S47** deploy BlobsClient fragility
  **1/3** : Phase B (reclassification documentee — risque
  inherent a mk_state() partage par 50+ tests, fix = refactoring
  majeur test infra, accepte pre-v1.0. Supprime du compteur
  carry.)

### Carries confirmes S49

- [carry] P2-A-1 rand blocker upstream 11+/3 : exemption blocker
  externe (rand 0.8.x getrandom Tier 3 Windows Arm64 — non resolu
  upstream). Justification renouvelee : pas de release rand 0.9
  ni fix getrandom upstream.
- [carry] P2-AUDIT-2 pre-release transitives iroh : herite pin
  0.98 (Day 0 #3)

---

## §7 Scope cuts

1. **events.py SSE streaming** — S49+ (dep AppEvents bus Rust)
2. **App runtime migration Rust** (AppContext, commands, state) —
   S49+ (multi-sprint, coordinator.py encore necessaire)
3. **MCP server migration Rust** — S49+ (dep runtime apps)
4. **PyO3 bindings removal** — S49+ (dep runtime apps portees)
5. **Suppression complete packages/nexus-coordinator/** — S49+
   (dep runtime apps Rust)
6. **CI/VPS/v1.0** — S49+
7. **Kudos debit/stake** — interdit (Day 0 #7)
8. **Pagination SQL-side LIMIT/OFFSET** — S49+ (D2 retient le fix
   Rust-side total_count, la migration SQL-level est un
   refactoring futur)
9. **Test infra mk_state() refactoring** — S49+ (50+ tests
   impactes, D4 refactoring scoped a sbfb_home uniquement)
10. **auth.rs set_var cleanup** — evaluer Phase B (4 set_var
    dans auth.rs avec save/restore pattern, scope extension si
    simple)

---

## §8 Tracabilite scope (S47 → S48)

| S47 scope cut | S48 disposition |
|---|---|
| events.py SSE streaming — S48+ | Scope cut reporte S49+ |
| App runtime migration Rust — S48+ | Scope cut reporte S49+ |
| MCP server migration Rust — S48+ | Scope cut reporte S49+ |
| PyO3 bindings removal — S48+ | Scope cut reporte S49+ |
| Suppression complete coordinator Python — S48+ | Scope cut reporte S49+ |
| CI/VPS/v1.0 — S49+ | Scope cut reporte S49+ |
| Kudos debit/stake — interdit | Day 0 #7 |
| deploy_from_repo happy path test — hors scope | Inchange |
| kudos SQL pagination runtime fix — S48 | **Phase A** (D2) |
| app-specific schema drift fix — S48+ | **Phase A** (exemption) |
| TOCTOU canary reload fix — S48 | **Phase A** (D1) |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | TOCTOU fix degrade la latence dispatch (lock tenu pendant read) | Low | Low | Read ~1ms local file, un seul caller par cycle, pas de contention |
| R2 | total_count frontend casse Zod schema validation | Low | Medium | KudosListSchema a deja `count: z.number()`, on ajoute `total_count: z.number()` — schema additif |
| R3 | feature gate test-support active accidentellement en prod | Low | Medium | Feature off par defaut, seulement dans [dev-dependencies] — cargo ne l'active pas en release |
| R4 | sbfb_home refactor casse les handlers consent/files en mode daemon reel | Medium | High | sbfb_home=None → fallback identique au comportement actuel, test daemon reel pre-commit |
| R5 | auth.rs set_var save/restore pattern plus complexe que prevu | Medium | Low | Phase B scope extension conditionnelle — si complexe, carry S49 |

---

## §10 Audit gate pattern — rappel

Phase 0 S47 jouee (PASS `3d14068`). Phase C produira
sprint49_audit_plan.md pour la session fraiche S49.

---

## §11 Checkpoint de validation

1. **D1** : mutex hold-across-read vs ArcSwap ?
   → mutex (pas de dep, fenetre eliminee, hold ~1ms)
2. **D2** : total_count Rust-side vs SQL COUNT(*) ?
   → Rust-side (toutes lignes deja en memoire, pas de double query)
3. **D3** : feature gate vs pub(crate) ?
   → feature gate (preserve le test cross-crate)
4. **D4** : sbfb_home dans state vs temp_env crate ?
   → state (elimine le probleme, pas de dep)
