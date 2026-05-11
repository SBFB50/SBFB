# Sprint 59 — Kickoff (launcher readiness + verified deploy E2E + LT-1 Kudos-v2)

**Ecrit** : 2026-05-11 (post-audit gate S58 PASS `cd6ef4b` + 2 fixes `fd852ed` `32ce39d`).
**Type** : **sprint impair** — pas de phase dette obligatoire (§6.2.1 Regle 1).
**Tip master d'entree** : `80ec664` (migration S58 → archive/v1.2/).
**Phase 0 audit Sprint 58** : **DEJA JOUE** — `cd6ef4b` PASS
(0 P0, 0 P1, 2 P2, 2 P3). Fixes `fd852ed` + `32ce39d` (Track F
carry count + naming collision).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** : last_validated 2026-05-10 (1j). 5 fichiers
  security avec triggers_revalidate. 0 trigger actif pertinent pour
  le theme S59. HARDENING_ROADMAP frais (S58). Pas de pre-research
  supplementaire.

- **windows-rs context7** (`/microsoft/windows-rs`) : MessageBoxW
  API confirmee — `Win32::UI::WindowsAndMessaging::MessageBoxW(hwnd,
  text, caption, utype)`. `windows-sys` expose les memes FFI sans
  wrappers safe. Raw `extern "system"` FFI est aussi viable (~15 LOC,
  0 dep supplementaire). Le launcher n'a pas de dep `windows`
  actuellement ; le daemon l'utilise pour named_pipe_server.

- **S21 research fairness** (`.planning/research/
  S21_research_fair_allocation_mechanisms.md`, 2026-04-19) : combo
  3 couches recommande — Couche A log-utility `K * log(1 + x/K0)`,
  Couche B DRF dispatcher (multi-ressource), Couche C EMA trust
  decay `alpha^age`. Concepts mathematiques intemporels, pas de
  mise a jour necessaire. Couche B (DRF) necessite infra
  multi-ressource inexistante → defer post-v1.0. Couches A+C
  implementables sur kudos_ledger.rs existant.

- **ROADMAP_COMMITMENTS check (G7 Regle 3)** :
  - LT-1 Kudos-v2 : **RECLASSIFIE pre-v1.0** (decision utilisateur
    2026-04-30). Owner S50, jamais livre S50-S58 (9 sprints).
    Condition ACTIVE. **Doit etre dans le plan S59** — dernier sprint
    feature avant S60 = installer + tag v1.0.
  - LT-2..LT-5 : latent. 0 condition declenchee.
  - LT-6 : RESOLVED S32.
  - LT-7 : Tier 1+2 DONE (S55). Gate pre-v1.0 satisfait. Tier 3
    validation controlee = S60 pre-tag.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 58 CLOSED + audit PASS (`cd6ef4b`). AppStorage P2P
operationnel (iroh-docs namespace + live events + sync E2E).
2 apps SBFB (Protocol Explorer + Ideas Hub) avec storage replique.
CI operationnel (Woodpecker + GHA). Verified deploy backend
complet (deploy.rs 679 LOC, clone → SBFB.json → zip → provenance
→ blob → gossip announce). Launcher fonctionnel (spawn daemon +
browser open + identity + token rotation + NVIDIA check).

**Etat technique (tip `80ec664`)** :
- Workspace clean, edition 2024, Rust 1.95 local / CI pin 1.94
- deploy.rs : `POST /api/v1/deploy-from-repo` complet (clone →
  SBFB.json node_id check → zip → BLAKE3 → Ed25519 provenance →
  contributor attestation → blob store → gossip announce)
- kudos_ledger.rs : credit() flat `amount = tokens_generated`,
  hash chain BLAKE3, verify_chain(), get_project_kudos() raw sums.
  Pas de log-utility, pas de EMA.
- fairness.rs : compute_gini(), compute_top_k_share(),
  compute_churn_rate() — observabilite seule, pas consomme par
  credit flow
- Launcher : spawns daemon, health check TCP, identity init/unlock,
  token rotation 24h, NVIDIA CVE check, auth loopback.
  `#![windows_subsystem = "windows"]` en release (pas de console).
  Pas de MessageBox, pas de seed apps.
- 2 apps exemples dans examples/ (sbfb-explorer, sbfb-ideas) sans
  SBFB.json (pas deployables via deploy-from-repo)
- Storage : dual backend (iroh-docs replique / HashMap+SQLite local).
  storage_join endpoint sans validation REPLICATED_APPS. Pas de
  rate-limit per-author sur storage writes.

**Carries entrants S59** :

| Item | Compteur | Source |
|---|---|---|
| LT-1 Kudos-v2 | **pre-v1.0** | ROADMAP_COMMITMENTS |
| P2-STORAGE-JOIN-VALIDATE | 1/3 → **2/3 si non adresse** | audit S58 |
| P2-STORAGE-ANTISPAM | 1/3 → **2/3 si non adresse** | audit S58 |
| P2-A-1 rand blocker upstream | 19+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |

**Alerte escalade G7** : P2-STORAGE-JOIN-VALIDATE et
P2-STORAGE-ANTISPAM passent 2/3 en sortie S59 si non adresses.
A S60, ils deviennent 3/3 MANDATORY — mauvais timing pendant le
sprint installer/tag. **Decision : les adresser S59 Phase C**
(petits, ~70 LOC total, pattern governor GCRA existant).

### §1.2 Ancrage roadmap

S58 a livre AppStorage P2P (dernier feature bloc avant
stabilisation). S59 ferme les 3 derniers gaps pre-v1.0 :

Roadmap pre-v1.0 (mise a jour 2026-05-11) :
- **S56** : gossip resilience + bridge extensions ✓
- **S57** : Protocol Explorer + Ideas Hub MVPs ✓
- **S58** : AppStorage P2P replication ✓
- **S59** : LT-1 Kudos-v2 + verified deploy E2E + launcher
  readiness + storage carries ← **ici** (early adopter ready)
- **S60** : installer NSIS/WiX + tray + frontend P2P + LT-7 Tier 3
  → tag v1.0 (end user ready)

### §1.3 Compteurs tests entree (tip `80ec664`)

| Suite | Count |
|---|---|
| Rust nextest | 1240 |
| Rust doctests | 6 passed |
| Vitest | 256 |
| Playwright | 42 + 2 fail (env pre-existant) |
| size-limit | 6/6 |
| **Total** | **~1502** |

**Post-S59 attendu** : ~1520+ (kudos formula tests + deploy E2E
test + storage validation tests).

### §1.4 Pre-launch protocol policy (rappel)

LT-1 Kudos-v2 modifie la formule credit() sans toucher
`KUDOS_ENTRY_VERSION` ni le wire format. Les KudosEntry en DB
conservent la meme structure — seule la valeur `amount` change
(log-utility au lieu de flat). Pas de tolerant decoder. Pas de
bump version. Le pre-launch protocol s'applique normalement.

---

## §2 Goal

Sprint 59 rend le produit **early adopter ready** : la boucle
deploy → browse → run fonctionne de bout en bout, le scoring est
equitable (LT-1 pre-v1.0 ferme), le launcher communique les
erreurs, et les 2 carries storage sont resolus avant qu'ils ne
deviennent MANDATORY.
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up. Kudos v2 formula active (log +
EMA). Deploy E2E teste. MessageBox visible sur erreur daemon.**

---

## §3 Phase 0 — Audit gate S58

**DEJA JOUE** : commit `cd6ef4b` PASS (0 P0, 0 P1, 2 P2, 2 P3).
Fixes `fd852ed` + `32ce39d` (carry count + naming collision).
Audit findings dans `.planning/archive/v1.2/sprint58_audit_findings.md`.
5 carries documentes pour S59 (cf. §1.1 ci-dessus).

---

## §4 Decisions Day 0 (D1..D4 gelees)

### D1 — LT-1 Kudos-v2 : log-utility + EMA decay

**Retenu** : implementer les Couches A et C du combo recommande
(research S21 `.planning/research/S21_research_fair_allocation_
mechanisms.md §10`).

**Couche A — Log-utility transform sur credit()** :
```rust
let amount = ((1000.0 * (1.0 + tokens_generated as f64).log2()) as u64).max(1);
```
RTX 5080 produisant 100x plus qu'un Pi 4 ne touche que ~6.6x plus
de kudos (log2(101) / log2(2) ≈ 6.66). Incentive hardware preservee
(volume > compression unitaire), queue coupee.

**Couche C — EMA decay sur effective score** :
```rust
fn effective_score(entries: &[KudosEntry], now_secs: u64) -> u64 {
    let alpha: f64 = 0.97; // half-life ~23 jours a 1 entree/jour
    entries.iter().map(|e| {
        let age_days = (now_secs.saturating_sub(e.created_at)) / 86400;
        (e.amount as f64 * alpha.powi(age_days as i32)) as u64
    }).sum()
}
```
Contributions anciennes decroissent exponentiellement. Un worker
inactif 90j perd ~93% de son score effectif. Empeche la rente
historique.

**get_project_kudos()** retourne les scores effectifs (EMA) au
lieu des sommes brutes. Le hash chain reste sur les montants bruts
(integrite du ledger inchangee). Les metrics fairness.rs
(Gini/top-k) consomment les scores effectifs.

**Rejete** :
- DRF (Dominant Resource Fairness) Couche B : necessite infra
  multi-ressource (VRAM, context_window, tokens/s). Aucune
  collecte de metriques ressources n'existe. Deferred post-v1.0.
- QF (Quadratic Funding) : requiert anti-sybil robuste + matching
  pool externe. Overkill pour v1.0. Log-scaling a les memes
  proprietes anti-concentration avec ~10 LOC.
- Pas de changement (garder v1) : LT-1 est pre-v1.0 obligatoire.
  La formule flat `amount = tokens_generated` produit un Matthew
  effect non borne. Inacceptable pour v1.0 launch.
- Formule multiplicative `tokens × quality × trust` avec log : les
  facteurs quality et trust ne sont pas encore implementes. Ajouter
  3 variables non definies ne resout pas le probleme. Log-utility
  sur le montant brut + EMA temporel suffit.

**Implications code** : `kudos_ledger.rs` (credit + effective_score
+ get_project_kudos), `fairness.rs` (consomme effective scores),
`kudos_api.rs` (API responses changent), tests.

### D2 — Verified deploy E2E : seed SBFB.json + integration test + frontend form

**Retenu** : le backend deploy.rs (679 LOC) est complet. Le gap
est le wiring E2E :

1. **SBFB.json** pour les 2 apps exemples (sbfb-explorer,
   sbfb-ideas). Contient `node_id` placeholder (rempli au premier
   deploy). Structure minimale :
   ```json
   { "node_id": "PLACEHOLDER", "name": "sbfb-explorer" }
   ```
2. **E2E integration test** : gate `SBFB_INTEGRATION=1` (meme
   pattern que test_cross_daemon_storage_sync). Scenario :
   creer repo git local avec index.html + SBFB.json → appeler
   deploy-from-repo → verifier provenance → verifier blob store.
3. **Frontend Deploy page** : composant React dans le shell.
   Formulaire : repo URL + project name + description → POST
   /api/v1/deploy-from-repo. Affiche resultat (hash, provenance).

**Rejete** :
- Keyoxide Ed25519 verification dans SBFB.json : integration
  systeme d'identite externe complexe (Keyoxide API, WKD lookup).
  Le node_id verification actuel suffit (le deployer signe avec
  sa cle daemon). Keyoxide deferred S60/post-v1.0.
- Auto-deploy depuis GitHub webhooks : premature, requiert
  serveur webhook persistant. Deploy manuel via API/UI suffit
  pour early adopters.
- Build depuis le repo (cargo build, npm build) : hors scope.
  Le repo doit contenir le HTML pre-build. Le build self-hosted
  (LT-7) est un flux separe.

**Implications code** : `examples/sbfb-explorer/SBFB.json` (NEW),
`examples/sbfb-ideas/SBFB.json` (NEW), `multi_daemon.rs` ou
`deploy.rs` tests (E2E), `web/src/pages/Deploy.tsx` (NEW),
`web/src/App.tsx` (route).

### D3 — Launcher error UX : MessageBoxW + cross-platform fallback

**Retenu** : afficher une boite de dialogue native quand le daemon
ne demarre pas (spawn failure, port occupe, identity non initialisee).

**Windows** : raw FFI `extern "system"` MessageBoxW, ~15 LOC. Pas
de nouvelle dep (le crate `windows` est dans le workspace mais pas
necessaire pour un appel FFI unique). `MB_ICONERROR | MB_OK` pour
les erreurs fatales.

**Cross-platform fallback** : `eprintln!` + process exit code 1.
Le launcher a deja `#![windows_subsystem = "windows"]` en release
(pas de console visible) → sans MessageBox, l'utilisateur ne voit
rien. Le MessageBox est la seule surface d'erreur.

**Rejete** :
- Crate `msgbox` (crates.io) : 0 commit depuis 2021, pas maintenu.
  Raw FFI = 15 LOC, zero risque supply chain.
- macOS osascript / Linux zenity : cross-platform dialog est S60
  scope. S59 target Windows primary (machine dev).
- Log file seul : invisible a l'utilisateur desktop. Le launcher
  a `windows_subsystem = "windows"` → pas de stderr visible.

**Implications code** : `crates/nexus-launcher/src/main.rs`
(error_msgbox function + appels dans spawn path).

### D4 — Storage carries : validation + rate-limit basique

**Retenu** : resoudre les 2 carries storage avant qu'ils ne
deviennent 3/3 MANDATORY en S60.

**(a) P2-STORAGE-JOIN-VALIDATE** : storage_join endpoint verifie
que l'app est dans la liste REPLICATED_APPS avant d'accepter un
join. Actuellement, n'importe quel nom d'app est accepte.
~20 LOC.

**(b) P2-STORAGE-ANTISPAM** : governor GCRA rate-limit sur les
storage writes, keyed par author (node_id + app_name). Reutilise
le pattern BrowseRequestLimiter (governor 0.10.2, deja dans le
workspace). 10 writes/min/author/app. ~50 LOC.

**Rejete** :
- Validation applicative complete (schema JSON par app) : scope
  trop large, necessite manifest par app. Deferred post-v1.0
  avec AppStorage Phase 2.
- Rate-limit global (pas per-author) : trop restrictif, penalise
  les utilisateurs legitimes.
- Defer S60 : les 2 items deviennent 3/3 MANDATORY pendant le
  sprint installer/tag. Mauvais timing.

**Implications code** : `storage_api.rs` (validation replicated
check), `http.rs` (rate-limit middleware storage endpoints).

### Acknowledged review findings (G1)

Scoring : D1 ⚠️, D2 ⚠️, D3 ⚠️, D4 ⚠️.
Rigor signal G4 satisfait (4 ⚠️ sur 4, 0 ❌).

**D1 ⚠️ (log2 vs ln + alpha divergence)** : log2 est un choix
cosmétique — `log2(x) = ln(x)/ln(2)`, le facteur 1/ln(2) est
absorbé par le K=1000 scale. log2 est informatiquement intuitif
(doublement) vs ln académique. Pas d'impact sur la compression
anti-whale. Sera documenté dans le code (commentaire 1 ligne).
Alpha=0.97 (half-life ~23j) diverge de S21's alpha=0.95 (~14j)
parce que la fréquence de tâches pre-launch est basse (< 1/jour
par worker) — un decay trop agressif pénalise les contributeurs
occasionnels. Nommé comme constante `KUDOS_EMA_ALPHA` pour
ajustement futur. Recherche P2P post-S21 : les concepts (log-
utility, EMA) sont mathématiques/intemporels, pas dépendants d'un
écosystème spécifique. BOINC CreditNew/Filecoin sont des systèmes
à tokenomics, hors modèle SBFB (kudos non-monétaires Day 0 #7).

**D2 ⚠️ (SBFB.json schema + verification chain)** : deploy.rs
définit déjà `SbfbJson { node_id: String }` (ligne 482) comme
struct de facto. Schema doc formelle déferrée S60 avec AppStorage
Phase 2 (manifest per app). Chaîne de vérification complète :
SBFB.json `node_id` → deploy.rs vérifie `== state.node_id` →
daemon signe `provenance.json` Ed25519 → provenance injectée dans
le zip → tout pair peut vérifier. Le node_id n'est pas "juste une
convention" — c'est le binding entre le repo et la clé de
signature. Keyoxide ajouterait une couche d'identité externe
(lien repo → identité sociale), déferrée S60/post-v1.0 comme
documenté. Alternatives intermédiaires (GPG tag, did:key) :
non pertinentes pre-v1.0 (0 user externe, le deployer = le
développeur du réseau).

**D3 ⚠️ (UTF-16 encoding + UB analysis)** : l'implémentation
utilisera `s.encode_utf16().chain(once(0)).collect::<Vec<u16>>()`
— pattern Rust standard pour UTF-16 null-terminated, 0 risque UB.
HWND = `std::ptr::null_mut()` (pas de parent window, MessageBox
standalone). `windows-sys` vs raw FFI : le raw FFI est 3 lignes
d'extern + 10 lignes de wrapper, vs ajouter `windows-sys` dep
avec features `Win32_UI_WindowsAndMessaging` qui importe des
centaines de bindings inutiles. Trade-off = minimal deps pour un
appel unique.

**D4 ⚠️ (REPLICATED_APPS governance + quota justification)** :
REPLICATED_APPS hardcodé = AppStorage Phase 2 scope (S60+,
manifest per app). Pre-v1.0 seule sbfb-ideas est répliquée, la
liste ne change pas. Quota 10/min : calibré sur BrowseRequestLimiter
(proven S56-S58, aucune plainte). Ideas Hub S57 cadence réelle :
~1-3 writes/session (create idea + vote), largement sous 10/min.
Nommé comme constante `STORAGE_WRITES_PER_MINUTE` pour ajustement.
GCRA vs token bucket : GCRA est déjà dans le workspace (governor
0.10.2), éprouvé, 0 dep additionnelle. Token bucket n'apporte
aucun avantage pour ce use case (rate fixe, pas de burst autorisé).

---

## §5 Plan Phase outline A..D

### Phase A — LT-1 Kudos-v2 fairness reform (pre-v1.0)

**But** : fermer LT-1 (9 sprints carry, pre-v1.0 obligatoire).

- credit() log-utility : `floor(1000 * log2(1 + tokens))`
- effective_score() EMA decay alpha=0.97
- get_project_kudos() retourne scores effectifs
- fairness.rs consomme scores effectifs pour Gini/top-k/churn
- kudos_api.rs : API response format inchange (total + contributors)
  mais valeurs sont EMA-weighted
- Tests : formula bounds, EMA decay correctness, chain integrity
  preserved, cross-project independence
- Commit : `feat(sprint59): Sprint 59 Phase A — LT-1 Kudos-v2
  log-utility + EMA fairness reform`

### Phase B — Verified deploy E2E + seed apps

**But** : la boucle deploy → browse fonctionne de bout en bout.

- SBFB.json pour examples/sbfb-explorer/ et examples/sbfb-ideas/
- E2E integration test (gate SBFB_INTEGRATION=1) : local git repo
  → deploy-from-repo → provenance verification → blob stored
- Frontend Deploy page : formulaire React (repo URL, project name,
  description) → POST /api/v1/deploy-from-repo → affiche resultat
- sync-bridge-sdk.sh : ajouter SBFB.json dans la copie
- Commit : `feat(sprint59): Sprint 59 Phase B — Verified deploy
  E2E + seed SBFB.json + Deploy page`

### Phase C — Launcher readiness + storage carries

**But** : le launcher communique les erreurs et les carries storage
sont resolus.

- MessageBoxW raw FFI (cfg(windows)) pour daemon spawn failure
- eprintln fallback (cfg(not(windows)))
- P2-STORAGE-JOIN-VALIDATE : validation is_replicated_app() dans
  storage_join handler avant accept
- P2-STORAGE-ANTISPAM : governor GCRA rate-limit 10 writes/min
  per-author per-app sur storage write endpoints
- Tests : storage join validation (accept replicated, reject other),
  rate-limit (accept within quota, reject excess)
- Commit : `feat(sprint59): Sprint 59 Phase C — Launcher
  MessageBox + storage validation + rate-limit`

### Phase D — Wrap-up + verification + audit plan S60

**But** : cloturer le sprint.

- CLAUDE.md : update S59 CLOSED, carries S60
- HARDENING_ROADMAP : update last_validated S59
- SPRINT_LOG : row S59
- verification.md : 28+ fail-fast rows
- sprint60_audit_plan.md : 7+ tracks
- Memory nexus_grid_pivot.md : update tip + carries
- Commit : `chore(sprint59): Phase D — wrap-up + verification +
  audit plan S60`

---

## §6 Items carry/dette

### Carries confirmes S59

- [Phase A] **LT-1 Kudos-v2** pre-v1.0 :
  **ADRESSE Phase A** → CLOSE attendu.
- [Phase C] **P2-STORAGE-JOIN-VALIDATE** 1/3 → 2/3 :
  **ADRESSE Phase C** → CLOSE attendu.
- [Phase C] **P2-STORAGE-ANTISPAM** 1/3 → 2/3 :
  **ADRESSE Phase C** → CLOSE attendu.
- [carry] **P2-A-1** rand blocker upstream 19+/3 : exemption
  externe. Justification : dep `rand` upstream bloque version
  compatible iroh 0.98. Aucun changement depuis S58.
- [carry] **P2-AUDIT-2** iroh transitives : herite pin 0.98.
  Justification : iroh 0.98 pinne (Day 0 #3), transitives
  non controlables.

### Carries residuels post-S59

| Item | Compteur S60 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 20+/3 | exemption externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |

---

## §7 Scope cuts

1. **AppStorage Phase 2** (namespace per manifest) — S60+
2. **AppStorage Phase 3** (optimisations, purge) — post-v1.0
3. **Kudos-v2 DRF** (Couche B multi-ressource) — post-v1.0
4. **Kudos-weighted voting** — post-v1.0
5. **Keyoxide identity verification in deploy** — S60/post-v1.0
6. **NSIS/WiX installer** — S60
7. **Tray icon** — S60
8. **Frontend P2P distribution** — S60
9. **Protocol Explorer F3** (gossip stats avance) — S60+
10. **Protocol Explorer F4** (tutoriel interactif) — post-v1.0
11. **Ideas Hub F3** (lier repos Git) — S60
12. **Ideas Hub F4-F5** (groupes, integration) — post-v1.0
13. **Ticket Write rotation dynamique** (Option B/C) — post-v1.0
14. **LT-7 Tier 3** validation controlee — S60 pre-tag

---

## §8 Tracabilite scope (S58 → S59)

| S58 scope cut | S59 disposition |
|---|---|
| Verified deploy E2E from repos Git | **Phase B** |
| Protocol Explorer F3 (gossip stats) | Scope cut S60+ |
| Protocol Explorer F4 (tutoriel) | Scope cut post-v1.0 |
| Ideas Hub F3 (lier repos Git) | Scope cut S60 |
| Ideas Hub F4-F5 (groupes, integration) | Scope cut post-v1.0 |
| Kudos-weighted voting | Scope cut post-v1.0 |
| AppStorage Phase 2 (manifest) | Scope cut S60+ |
| AppStorage Phase 3 (optimisations) | Scope cut post-v1.0 |
| LT-1 Kudos-v2 fairness reform | **Phase A** |
| LT-7 Tier 3 (validation controlee) | Scope cut S60 pre-tag |
| Ticket Write rotation dynamique | Scope cut post-v1.0 |
| P2-STORAGE-JOIN-VALIDATE | **Phase C** |
| P2-STORAGE-ANTISPAM | **Phase C** |

---

## §9 Risk register

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Log-utility change les scores existants drastiquement | Medium | Low | Pre-launch (0 user externe). Migration = recalcul conceptuel, pas physique (entries DB inchangees, seul le display change via EMA). |
| R2 | EMA alpha=0.97 trop agressif ou trop doux | Low | Low | Alpha configurable (constante nommee). Ajustable post-launch sans migration. Les metrics Gini/top-k monitoreront l'effet. |
| R3 | Deploy E2E test depend de `git` en PATH | Medium | Medium | Le test est gate SBFB_INTEGRATION=1. git est pre-requis du build executor (LT-7). Pas de regression CI. |
| R4 | Frontend Deploy form expose deploy API a l'utilisateur | Low | Low | L'API est deja exposee (loopback bearer auth). Le form est un wrapper UX. |
| R5 | MessageBoxW bloque le thread launcher | Low | Low | MessageBoxW est modal mais appele uniquement avant exit (erreur fatale). Pas de degradation UX. |
| R6 | Rate-limit storage trop restrictif pour usage normal | Low | Medium | 10 writes/min/author/app = largement suffisant pour Ideas Hub (vote = 1 write). Configurable. |

---

## §10 Audit gate pattern — rappel

Phase 0 S58 jouee (PASS `cd6ef4b` + fixes). Phase D produira
sprint60_audit_plan.md pour la session fraiche S60.

---

## §11 Checkpoint de validation

1. **D1** : LT-1 Kudos-v2 = log-utility + EMA (pas DRF) ?
   → oui (research S21 Couche A+C, DRF hors scope sans infra)
2. **D2** : Verified deploy E2E = seed SBFB.json + test + frontend ?
   → oui (deploy.rs 679 LOC complet, gap = wiring)
3. **D3** : Launcher error UX = raw FFI MessageBoxW ?
   → oui (0 dep, 15 LOC, windows_subsystem = "windows" impose
   une surface d'erreur native)
4. **D4** : Storage carries = validation + rate-limit basique ?
   → oui (evite 3/3 MANDATORY en S60, ~70 LOC total, pattern
   governor existant)
