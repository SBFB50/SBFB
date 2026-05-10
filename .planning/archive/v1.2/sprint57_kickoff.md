# Sprint 57 — Kickoff (Protocol Explorer + Ideas Hub + MANDATORY carries)

**Ecrit** : 2026-05-09 (post-audit gate S56 PASS `e7a9b93`).
**Type** : **sprint impair** — pas de phase dette obligatoire
(§6.2.1 Regle 1). 2 items 3/3 MANDATORY a traiter Phase A.
**Tip master d'entree** : `e7a9b93`.
**Phase 0 audit Sprint 56** : **DEJA JOUE** — `e7a9b93` PASS
(0 P0, 0 P1, 1 P2, 2 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-09 (0j). 5 fichiers
  security avec triggers_revalidate. 0 trigger actif pertinent pour
  le theme S57. HARDENING_ROADMAP frais (S56). Pas de pre-research
  supplementaire.

- **iroh 0.98 multi-node test patterns** : le crate nexus-test-harness
  fournit deja un `TestHarness` pour spawn de daemon. Les tests E2E
  existants (cross_daemon_blob.rs depuis S35) valident le pattern
  spawn + port isolation. governor rate-limiter et gossip sont
  testes unitairement. L'E2E multi-noeuds est une extension du
  pattern existant (2 daemons, gossip bidirectionnel).

- **ROADMAP_COMMITMENTS check** :
  - LT-7 self-hosted build : Tier 1+2 DONE (S55). Tier 3 S57+.
    **Condition PRE-V1.0 OBLIGATOIRE** : Tier 1+2 satisfait.
    Tier 3 (N builders, auto-deploy) reste S58+.
  - LT-1 Kudos-v2 : trigger Gini > 0.70. Pas de donnees. Latent.
  - LT-2..LT-5 latents. LT-6 RESOLVED S32.
  - 0 condition declenchee.

- **Verified deploy** (S14) : le chemin clone → Keyoxide → zip →
  provenance.json est implemente mais jamais teste E2E avec une
  vraie app tierce. S57 cree les premieres apps mais les deploie
  manuellement en zip local via blob-serve. Le deploy verifie
  E2E depuis repo = test S58.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 56 CLOSED + audit PASS (`e7a9b93`). Bridge postMessage
etendu a 9 methodes (4 existantes + 5 S56). Gossip resilient
(outbox persistent + rate-limit per-peer). CI operationnel
(Woodpecker ci.sbfb.world + GHA). LT-7 Tier 1+2 livres.

**Etat technique (tip `e7a9b93`)** :
- Workspace clean, edition 2024, Rust 1.94
- Bridge 9 methodes : task_submit, storage_get, storage_set,
  pii_redact, storage_list, storage_delete, identity_pubkey,
  node_status, browse_list
- Storage backend : in-memory HashMap (storage_api.rs, S56 Phase C)
  Carry P2 : persistence SQLite
- Outbox gossip persistent SQLite (coordinator.db M6)
- Browse rate-limit governor GCRA per-peer (10 req/min)
- CoordinatorDb : 6 migrations SQLite WAL
- Blob-serve daemon : sert des archives zip dans iframe sandbox
- Verified deploy : implemente S14, jamais teste E2E avec app
  reelle
- 21 occurrences cfg(unix) dans 11 fichiers Rust
- nexus-test-harness : TestHarness + cross_daemon_blob.rs (S35)

**Carries entrants S57** :

| Item | Compteur | Source |
|---|---|---|
| P2-S54-windows-test-cfg-unix | **3/3 MANDATORY** | S54 Phase B |
| P2-S54-test-E2E-multi-noeuds | **3/3 MANDATORY** | S54 Phase C |
| P2-A-1 rand blocker upstream | 15+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-JITTER-SCOPE | 2/3 | S55 Phase D |
| P2-INVITE-U16-WIRE | 2/3 | S55 Phase D |
| P2-RETAIN-RECENT | 1/3 | S56 audit |
| P2-STORAGE-SQLITE | 1/3 | S56 Phase C |

### §1.2 Ancrage roadmap

S56 a livre le bridge + gossip resilience. S57 est le sprint
qui pose les premieres apps sur le reseau — la preuve vivante
que SBFB fonctionne.

Roadmap pre-v1.0 (decision utilisateur 2026-05-07) :
- **S56** : gossip resilience + bridge extensions + dette pair ✓
- **S57** : Protocol Explorer MVP + Ideas Hub MVP ← **ici**
- **S58** : stabilisation + verified deploy E2E + tag v1.0

### §1.3 Compteurs tests entree (tip `e7a9b93`)

| Suite | Count |
|---|---|
| Rust nextest | 1227 |
| Rust doctests | 6 passed, 1 ignored |
| Vitest | 256 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 6/6 |
| **Total** | **~1489** |

**Post-S57 attendu** : ~1510+ (E2E multi-noeuds tests + windows
cfg tests + app bridge integration tests Vitest).

### §1.4 Pre-launch protocol policy (rappel)

Phase A touche les tests et la CI, pas de changement wire format.
Phases B-C creent des apps HTML/JS qui utilisent le bridge
existant — pas de nouveau wire format P2P. Phase C ajoute une
migration M7 pour la persistence storage (table interne, pas de
wire format). Aucun `*_FORMAT_VERSION` a changer.

---

## §2 Goal

Sprint 57 deploie les 2 premieres apps SBFB (Protocol Explorer +
Ideas Hub) et ferme les 2 derniers MANDATORY carries, prouvant
que le reseau fonctionne de bout en bout.
**Critere SMART : 24+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up. 2 apps fonctionnelles dans
iframe sandbox. E2E multi-noeuds test vert. cfg(unix) gap
documente et couvert CI.**

---

## §3 Phase 0 — Audit gate S56

**DEJA JOUE** : commit `e7a9b93` PASS
(0 P0, 0 P1, 1 P2, 2 P3).
Audit findings dans `.planning/archive/v1.2/sprint56_audit_findings.md`.
8 carries documentes pour S57 (cf. §1.1 ci-dessus).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — Apps dans examples/ du monorepo, HTML/CSS/JS pur

**Retenu** : les 2 apps sont creees dans `examples/sbfb-explorer/`
et `examples/sbfb-ideas/` du monorepo. Pure HTML/CSS/JS sans
framework (pas de React, Preact ou Alpine.js). Chaque app est un
dossier avec `index.html` + `style.css` + `app.js` + assets SVG.
Le SDK `sbfb-bridge.js` est copie depuis `web/public/`.

En S57, les apps sont testees localement via blob-serve (zip
manuel ou script). Le deploy verifie E2E depuis repos Git separes
est reporte a S58 (stabilisation pre-tag v1.0).

**Rejete** :
- Repos Git separes : complexite supplementaire (creation repos,
  CI distincte, verified deploy E2E non teste). Premature pour le
  MVP. Separation S58.
- Micro-framework (Preact, Alpine.js) : overhead inutile pour
  du contenu majoritairement statique (Protocol Explorer) ou un
  CRUD simple (Ideas Hub). HTML pur + quelques `document.querySelector`
  suffisent. < 500KB zip cible.
- React/Vue : exclu par design (apps SBFB doivent etre legeres,
  le shell React est un client parmi d'autres).

**Implications code** : `examples/sbfb-explorer/` (NEW),
`examples/sbfb-ideas/` (NEW), `web/public/sbfb-bridge.js` (copie).

### D2 — MANDATORY windows-test : audit cfg + doc CI cross-platform

**Retenu** : auditer les 21 occurrences `cfg(unix)` dans 11
fichiers Rust. Pour chaque occurrence, verifier que le code est
correctement gate et que les tests associes sont gated ou ont un
equivalent Windows. Documenter le resultat dans `docs/rust/PATTERNS.md`
section §P46. Verifier que le job Windows GHA existant (rust-ci.yml
`windows-latest` matrix) couvre correctement les tests gates.
Documenter pourquoi certains tests sont platform-specific.

L'objectif n'est pas d'ajouter des implementations Windows pour
toutes les features unix-only (UDS, /proc, libc::kill) — c'est
de s'assurer que le CI couvre les 2 plateformes et que les gaps
sont documentes.

**Rejete** :
- Implementer des equivalents Windows pour chaque cfg(unix) :
  hors-scope. UDS n'a pas d'equivalent direct Windows (named
  pipes deja implementes). /proc n'existe pas sur Windows. Les
  features unix-only restent unix-only, correctement gatees.
- Ignorer le probleme : a 3/3 MANDATORY, pas une option.
- CI Windows Woodpecker : Woodpecker est Linux-only. GHA
  supporte Windows et a deja un job windows-latest dans
  rust-ci.yml.

**Implications code** : `docs/rust/PATTERNS.md` (§P46),
`.github/workflows/` (Windows job si manquant), 0-5 fichiers
Rust (ajout cfg(test) gates si manquants).

### D3 — MANDATORY E2E multi-noeuds : 2 daemons localhost

**Retenu** : test d'integration dans `nexus-test-harness` qui
spawn 2 instances de shell-daemon sur des ports differents avec
des data directories distincts. Verification : gossip discover
mutuel + message exchange. Le test utilise le pattern existant
`TestHarness` enrichi de helpers multi-node.

Les 2 daemons utilisent localhost avec des ports aleatoires
(port 0 allocation OS). Pas de Docker ni VPS — le test tourne
sur la machine dev et en CI. Timeout 30s pour le handshake
gossip. Le test verifie :
1. Node A demarre, publie un message gossip
2. Node B demarre, decouvre Node A via bootstrap
3. Node B recoit le message gossip de Node A
4. (optionnel) Node B publie, Node A recoit

**Rejete** :
- Docker multi-container : overhead CI significatif, complexite
  Docker-in-Docker dans Woodpecker. Les 2 daemons sur localhost
  suffisent pour prouver le gossip.
- Test VPS-to-VPS : infrastructure externe, flaky par nature.
  Le test cross-machine est deja valide manuellement (LAN
  Win-Mac, WAN dev-VPS Helsinki). L'E2E automatise est localhost.
- Mock gossip : ne prouve rien. Le point est de tester le vrai
  gossip iroh entre 2 noeuds reels.

**Implications code** : `crates/nexus-test-harness/` (helpers
multi-node), `crates/nexus-test-harness/tests/` (test E2E).

### D4 — Storage persistence SQLite (migration M7)

**Retenu** : persister le HashMap `AppStorage` dans une table
SQLite `app_storage(app_name TEXT, key TEXT, value TEXT, PRIMARY KEY
(app_name, key))` dans coordinator.db (migration M7). Le storage
est charge au boot et ecrit a chaque mutation (set/delete).
Le HashMap in-memory reste le cache de lecture pour la performance.

Motivation : Ideas Hub stocke les idees et votes dans le
storage. Sans persistence, tout est perdu au restart du daemon.
C'est un deal-breaker UX pour l'app.

**Rejete** :
- Fichier JSON per-app : pas de transactions, corruption possible
  sur crash. SQLite est deja dans le workspace (rusqlite).
- Rester in-memory : inacceptable pour Ideas Hub (perte de
  donnees au restart). Protocol Explorer peut vivre sans (F1/F2
  statiques) mais c'est incoherent d'avoir 2 backends.
- iroh-docs comme backend : overhead P2P pour du stockage local.
  iroh-docs est pour la replication reseau, pas le stockage
  per-node.

**Implications code** : `crates/nexus-coordinator-rs/src/db.rs`
(migration M7 + helpers), `crates/nexus-shell-daemon/src/storage_api.rs`
(load at boot + write-through).

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ⚠️, D3 ✅, D4 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 4).

D2 ⚠️ (GHA Windows job deja existant) : le reviewer note que
rust-ci.yml a deja un job `windows-latest` dans la matrice test
(ligne 120). Le D2 mentionne "ajouter un job Windows a GHA si
absent" — formulation trompeuse. Le scope Phase A est un **audit
des gaps cfg et documentation**, pas de la creation d'infra CI.
Le job Windows existe depuis avant S57. Correction inline :
supprimer "ajouter un job" → clarifier que Phase A verifie que
le job existant couvre les tests correctement gates.

---

## §5 Plan Phase outline A..E

### Phase A — MANDATORY carries (windows-test + E2E multi-noeuds)

**But** : fermer les 2 items 3/3 MANDATORY.
CLOSE P2-S54-windows-test-cfg-unix + P2-S54-test-E2E-multi-noeuds.

- Audit 21 cfg(unix) / 12 cfg(windows) dans 11 fichiers
- Documenter dans PATTERNS.md §P46 la strategie cross-platform
- Verifier/ajouter gating cfg(test) sur tests platform-specific
- GHA Windows job verification
- TestHarness multi-node helpers
- Test E2E : 2 daemons gossip discover + message exchange
- Commit : `feat(sprint57): Sprint 57 Phase A — MANDATORY carries
  windows-test + E2E multi-noeuds`

### Phase B — Storage persistence SQLite (M7)

**But** : persister le storage des apps dans coordinator.db pour
que les donnees survivent au restart du daemon. Prerequis Ideas Hub.

- Migration M7 : table app_storage dans coordinator.db
- Helpers DB : load_all_storage() + upsert_storage() + delete_storage()
- storage_api.rs : load at boot + write-through sur set/delete
- 4 tests Rust (persistence, overwrite, delete, boot load)
- Commit : `feat(sprint57): Sprint 57 Phase B — storage persistence
  SQLite M7`

### Phase C — Protocol Explorer MVP

**But** : premiere app SBFB fonctionnelle dans l'iframe sandbox.

- `examples/sbfb-explorer/index.html` : structure + navigation
- `examples/sbfb-explorer/style.css` : CSS minimal (dark theme)
- `examples/sbfb-explorer/app.js` : bridge integration F3 live
  status (node_status + browse_list + identity_pubkey)
- Contenu F1 : 5 sections protocole (architecture, cycle app,
  cycle tache, securite, philosophie)
- Contenu F2 : liens vers le code source
- sbfb-bridge.js copie depuis web/public/
- Commit : `feat(sprint57): Sprint 57 Phase C — Protocol Explorer
  MVP (sbfb-explorer)`

### Phase D — Ideas Hub MVP

**But** : deuxieme app SBFB. Consomme la persistence Phase B.

- `examples/sbfb-ideas/index.html` : formulaire + liste + vote
- `examples/sbfb-ideas/style.css` : CSS minimal (dark theme)
- `examples/sbfb-ideas/app.js` : bridge CRUD (storage_set +
  storage_list + storage_delete + identity_pubkey)
- F1 : proposer idee (titre + description, auteur = identity)
- F2 : voter (1 vote/identite/idee, toggle upvote)
- Commit : `feat(sprint57): Sprint 57 Phase D — Ideas Hub MVP
  (sbfb-ideas)`

### Phase E — Wrap-up + verification + audit plan S58

**But** : cloturer le sprint.

- CLAUDE.md : update S57 CLOSED, carries S58
- HARDENING_ROADMAP : update last_validated S57
- verification.md : 26+ fail-fast rows
- sprint58_audit_plan.md : 7+ tracks
- Commit : `chore(sprint57): Phase E — wrap-up + verification +
  audit plan S58`

---

## §6 Items carry/dette

### Carries confirmes S57

- [phase A] **P2-S54-windows-test-cfg-unix** 3/3 MANDATORY :
  **ADRESSE Phase A** → CLOSE attendu.
- [phase A] **P2-S54-test-E2E-multi-noeuds** 3/3 MANDATORY :
  **ADRESSE Phase A** → CLOSE attendu.
- [phase C] **P2-STORAGE-SQLITE** 1/3 :
  **ADRESSE Phase C** → CLOSE attendu.
- [carry] **P2-A-1** rand blocker upstream 15+/3 : exemption externe.
- [carry] **P2-AUDIT-2** iroh transitives : herite pin 0.98.
- [carry] **P2-JITTER-SCOPE** 2/3 : carry S58.
- [carry] **P2-INVITE-U16-WIRE** 2/3 : carry S58.
- [carry] **P2-RETAIN-RECENT** 1/3 : carry S58.

### Carries residuels post-S57

| Item | Compteur S58 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 16+/3 | exemption |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-JITTER-SCOPE | 3/3 **MANDATORY** | S55 Phase D |
| P2-INVITE-U16-WIRE | 3/3 **MANDATORY** | S55 Phase D |
| P2-RETAIN-RECENT | 2/3 | S56 audit |

**Attention S58 pair** : JITTER-SCOPE et INVITE-U16-WIRE passent
a 3/3 MANDATORY. S58 est pair → phase dette obligatoire
(§6.2.1 Regle 1) + 2 items MANDATORY.

---

## §7 Scope cuts

1. **LT-7 Tier 3** (N builders, auto-deploy) — S58+
2. **Verified deploy E2E from repos Git separes** — S58
3. **Protocol Explorer F3 avance** (gossip stats, latence peers) — S58
4. **Protocol Explorer F4** (tutoriel interactif) — S58
5. **Ideas Hub F3** (lier repos Git) — S58
6. **Ideas Hub F4** (groupes de travail) — post-v1.0
7. **Ideas Hub F5** (integration reseau, gossip notifications) — post-v1.0
8. **Kudos-weighted voting** (Ideas Hub votes × reputation) — S58
9. **AppStorage replication P2P** (iroh-docs sync entre noeuds) — post-v1.0
10. **Rate-limit retain_recent housekeeping** — S58 (P2-RETAIN-RECENT 2/3)
11. **P2-JITTER-SCOPE test integration** — S58 (3/3 MANDATORY)
12. **P2-INVITE-U16-WIRE doc post-v1.0** — S58 (3/3 MANDATORY)
13. **LT-1 Kudos-v2 fairness reform** — S58+

---

## §8 Tracabilite scope (S56 → S57)

| S56 scope cut | S57 disposition |
|---|---|
| LT-7 Tier 3 (N builders, auto-deploy) | Scope cut reporte S58+ |
| Protocol Explorer MVP | **Phase B** |
| Ideas Hub MVP | **Phase C** |
| P2-JITTER-SCOPE test integration | Scope cut reporte S58 (3/3 MANDATORY) |
| P2-INVITE-U16-WIRE doc post-v1.0 | Scope cut reporte S58 (3/3 MANDATORY) |
| LT-1 Kudos-v2 fairness reform | Scope cut S58+ |
| E2E multi-noeuds automatise | **Phase A** (3/3 MANDATORY) |
| Windows test cfg(unix) CI | **Phase A** (3/3 MANDATORY) |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | E2E multi-noeuds flaky (gossip timing-dependent) | Medium | Medium | Timeout 30s + retry logic + skip CI si flaky (documenter). Le test peut etre `#[ignore]` avec annotation en CI GHA. |
| R2 | Protocol Explorer trop ambitieux en contenu | Medium | Low | MVP = 5 pages HTML essentielles. Le contenu est extensible S58. |
| R3 | Ideas Hub storage schema evolue | Low | Low | Pre-launch policy : pas de compat, on redefinit. Schema v1 simple. |
| R4 | Migration M7 conflicte avec M6 outbox | Low | Low | Migrations additives independantes. rusqlite_migration gere l'ordre. |
| R5 | Windows CI GHA consomme minutes Actions | Low | Medium | Job Windows execute uniquement nextest (pas de build release). Limite a cargo test --workspace. |

---

## §10 Audit gate pattern — rappel

Phase 0 S56 jouee (PASS `e7a9b93`). Phase D produira
sprint58_audit_plan.md pour la session fraiche S58.

---

## §11 Checkpoint de validation

1. **D1** : Apps dans examples/ du monorepo, HTML/JS pur ?
   → oui (precedent hello-world-app, pas de framework necessaire)
2. **D2** : Windows test = audit cfg + doc, pas d'impl Windows ?
   → oui (les features unix-only restent unix-only, l'objectif
   est la couverture CI et la documentation)
3. **D3** : E2E multi-noeuds = 2 daemons localhost ?
   → oui (pattern TestHarness existant, pas besoin de Docker/VPS)
4. **D4** : Storage persistence = SQLite M7 dans coordinator.db ?
   → oui (consistent avec outbox M6, rusqlite deja dans workspace)

**Re-decoupage Phase B/C/D (post-Phase A)** : la Phase B
initiale (Protocol Explorer) et Phase C initiale (Ideas Hub +
storage) ont ete re-decoupees en 3 phases pour respecter la
discipline de commit atomique. Le backend Rust (storage M7) est
separe des apps HTML/JS, et chaque app a sa propre phase.
