# S65-S75 Cross-Cutting Research — Dependances, Risques, Sequencage

**Date :** 2026-05-18
**Confiance globale :** HIGH (analyse exhaustive codebase + recherches S65/S66/S67 deja produites + verification iroh ecosystem)
**Scope :** 11 sprints, 3 arcs, 20 carry items, zones rouges, complexite, sequencage

---

## 1. Carry Items S64->S65 — Verification exhaustive dans le code

### 1.1 P2-FEED-INSERT-NO-AUTH-TIER (3/3 MANDATORY S65)

**Status : VRAI GAP -- CONFIRME**

**Verification :** `crates/nexus-shell-daemon/src/feed_sync.rs:445-487` -- la fonction `feed_insert()` accepte un `Json<FeedInsertRequest>` et insere directement dans le feed via `public_feed::insert_feed_operation()` sans aucune verification du auth tier du caller. Ni `auth_required()` middleware verification (qui est sur la route), ni `PeerCredsVerified` extension check, ni aucun tier-based gating.

**Risque :** Un caller authentifie (bearer token valide) mais de tier AUTO (T0) peut inserer des operations feed. La route `/api/daemon/feed/insert` est montee a `http.rs:347` avec le middleware auth standard, mais aucun tier differentiation n'est faite. N'importe quel process avec le bearer token peut injecter des operations feed.

**Complexite fix :** ~30-50 LOC. Ajouter un check `CONFIRM_PROMPT` (T1) ou au minimum verifier que le caller est le noeud local (pas un peer distant via proxy). Le trust tier system de `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` place cette route en T0 actuellement, mais elle devrait etre au moins T1.

**Distribution :** OBLIGATOIRE S65 Phase A (pre-requis : tout le sprint repose sur le contrat public de confiance).

### 1.2 P2-PROVENANCE-404-BRIDGE (2/3)

**Status : VRAI GAP -- CONFIRME**

**Verification :** Le bridge method `provenance_get` retourne le `ProvenanceRecord` quand il existe, mais ne distingue pas entre "projet inexistant" (404) et "projet sans provenance" (pas de record). Le code `VerificationDetail.tsx` gere un `error` generique mais ne peut pas afficher un message UX specifique.

**Complexite fix :** ~20-30 LOC. Retourner un code distinct dans la reponse bridge pour differencier "project not found" vs "no provenance record".

**Distribution :** Absorbable par S65 (contrat public -- ameliore l'UX de verification) ou S68 (proof pack).

### 1.3 P2-BADGE-WORDING-PREMATURE (pre-existant S14)

**Status : VRAI GAP -- CONFIRME -- DIRECTEMENT LIE A S65**

**Verification :** `Browse.tsx:258-259` et `BrowsedProject.tsx:280-281` affichent "Verifie" + ShieldCheck quand `entry.provenance_hash` existe. AUCUNE verification live n'est faite -- le badge presuppose que l'existence d'un hash = verification reussie. La recherche S65 documente 2 cas CRITIQUES dans l'inventaire exhaustif.

**Complexite fix :** ~80-120 LOC. Remplacer le badge statique par un badge conditionnel : (1) appel lazy au endpoint provenance_verify, (2) 3 etats : "verifie" (signature ok), "non verifie" (pas de provenance), "en attente" (loading). C'est le coeur meme du sprint S65 "contrat public".

**Distribution :** OBLIGATOIRE S65 -- c'est litteralement le sujet du sprint.

### 1.4 P2-COMMIT-TITLE-FORMAT (2/3)

**Status : VRAI GAP -- PROCESS**

**Verification :** Le pattern `feat(scope): Sprint N Phase X -- titre` est documente dans `docs/claude/README.md` mais pas de validation automatique. Les commits S64 respectent le format mais rien ne l'enforce.

**Complexite fix :** ~20 LOC (hook pre-commit regex).

**Distribution :** Absorbable par n'importe quel sprint. Recommendation : S65 Phase dette pair.

### 1.5 P2-REVIEW-ORDER (2/3)

**Status : VRAI GAP -- PROCESS**

**Verification :** L'ordre de review (preflight -> code -> review) n'est pas formellement documente dans README.md de maniere exhaustive.

**Complexite fix :** ~10 LOC doc amendment.

**Distribution :** Absorbable par S65 Phase dette pair.

### 1.6 P2-PYTHON-BLOCK-EXEMPTION (2/3)

**Status : POTENTIELLEMENT OBSOLETE**

**Verification :** Le projet est 100% Rust depuis S50-S51. Il n'y a plus de code Python dans le build path. Le SKILL.md reference des blocs Python mais le projet n'en a plus. L'exemption est toujours pertinente si des tests ou scripts Python existent dans le futur, mais actuellement c'est du zombie process.

**Complexite fix :** 5 LOC SKILL.md amendment.

**Distribution :** S65 dette pair ou reclassification en "resolved by pivot S50".

### 1.7 P2-EXPLORER-ESCAPE-SINGLE-QUOTE (2/3)

**Status : VRAI GAP -- CONFIRME**

**Verification :** `examples/sbfb-explorer/app.js:242-245` -- `escapeAttr()` echappe `&`, `"`, `<`, `>` mais PAS `'` (single quote). Un attribut HTML construit avec des single quotes serait vulnerable a l'injection.

**Complexite fix :** 1 LOC (ajouter `.replace(/'/g, "&#39;")`).

**Distribution :** S65 dette pair.

### 1.8 P2-PLAYWRIGHT-SPECS-STALE (2/3)

**Status : VRAI GAP -- CONFIRME**

**Verification :** 12 fichiers `web/tests/gov-*.spec.ts` referent a l'app "gov" qui n'existe plus depuis le pivot Rust S50. Le premier fichier (`gov-dashboard.spec.ts`) reference "Sprint 8 Phase B" et `govdata.db`. Il n'y a plus de code Python coordinator, donc ces specs ne peuvent pas passer.

**Complexite fix :** Suppression pure (12 fichiers). Si Playwright tests doivent etre re-ecrits pour les pages actuelles (Browse, BrowsedProject, Deploy, etc.), c'est un effort plus important (~200-400 LOC).

**Distribution :** S65 dette pair (suppression des zombies) + S69 pilote (re-ecriture des specs pour les pages actuelles).

### 1.9 P2-VERIFY-LOCAL-KEY-ONLY (2/3)

**Status : VRAI GAP -- CONFIRME**

**Verification :** `verify_entry()` dans `public_feed.rs:420-452` verifie la signature Ed25519 avec la pubkey **contenue dans l'entree elle-meme** (`entry.author_pubkey`). Pour un noeud local, c'est suffisant. Pour la verification cross-node, il faudrait resoudre la pubkey via pkarr pour confirmer que le `author_pubkey` correspond bien au node_id revendique.

**Complexite fix :** ~50-80 LOC. Ajout d'un resolver pkarr dans le path de verification remote.

**Distribution :** Necessaire avant S69 (pilote externe). Recommendation : S67 ou S68.

### 1.10 P2-COVERAGE-DEPLOY-E2E (2/3)

**Status : VRAI GAP -- CONFIRME**

**Verification :** Aucun test E2E `deploy_roundtrip` n'existe dans le workspace. `deploy.rs` est teste unitairement mais le cycle complet (deploy from repo -> archive -> provenance -> publish -> verify) n'est pas couvert en E2E.

**Complexite fix :** ~100-150 LOC test E2E dans `multi_daemon.rs` ou nouveau fichier.

**Distribution :** S68 (proof pack -- le test E2E deploy roundtrip EST le proof pack).

### 1.11 P2-FEED-JOIN-HANDLE-LEAK (1/3)

**Status : VRAI GAP -- CONFIRME**

**Verification :** `feed_sync.rs:616` -- `tokio::spawn(async move { ... })` lance une task qui traite le live stream sans JoinHandle stocke ni shutdown channel. Quand le daemon shutdown, cette task est abandonnee (pas rejointe). Le subscribe task natif (boot-time, `feed_sync.rs:299`) a un shutdown channel via `tokio::sync::watch::Receiver<bool>`, mais le join handler n'en a pas.

**Complexite fix :** ~40-60 LOC. Stocker le JoinHandle dans l'etat, passer un shutdown receiver, rejoindre a la fermeture.

**Distribution :** S66 (durabilite -- directement lie au lifecycle du daemon).

### 1.12 P2-VERIFY-ENTRY-VERSION-GUARD (1/3)

**Status : VRAI GAP -- CONFIRME**

**Verification :** `verify_entry()` dans `public_feed.rs:420-452` ne verifie PAS le champ `version`. Le champ est present dans `FeedEntry.version` et dans `FeedEntryCanonical.version`, mais `verify_entry()` ne fait aucun check. Pre-launch (v1.0 tag pose localement mais pas pousse), `FEED_FORMAT_VERSION == 1` est la seule valeur valide.

**Complexite fix :** ~5 LOC. Ajouter `if entry.version != FEED_FORMAT_VERSION { return Err(...) }` en tete de `verify_entry()`.

**Distribution :** OBLIGATOIRE avant go-live. Recommendation : S65 Phase A (en meme temps que FEED-INSERT-NO-AUTH-TIER).

### 1.13 P2-ORPHAN-REPUBLISH-RECOVERY (1/3)

**Status : VRAI GAP -- CONFIRME**

**Verification :** Si un publish echoue apres insertion DB mais avant propagation iroh-docs (crash, timeout iroh, erreur reseau), l'entree est dans la DB SQLite mais pas dans le namespace iroh-docs. Le rollback tail-safe (S64 Phase B fix) empeche la suppression si l'entree est chainee. Il n'y a pas de mecanisme de republication DB->iroh-docs pour ces orphelins.

**Complexite fix :** ~60-100 LOC. Startup scan des entries DB non presentes dans iroh-docs + republish.

**Distribution :** S66 (durabilite -- crash recovery).

### 1.14 P2-A-1 rand blocker upstream

**Status : EXEMPTION EXTERNE -- INCHANGE**

**Verification :** `Cargo.toml:58` pin `rand = "0.8"`. rand 0.9 n'est pas encore stable. Pas d'action possible.

**Distribution :** Monitoring. Resolu quand rand 0.9 est stable + iroh l'adopte.

### 1.15 P2-AUDIT-2 pre-release transitives iroh

**Status : EXEMPTION EXTERNE -- MAIS SITUATION CHANGE**

**Verification :** iroh 1.0.0-rc.0 a ete publie le 11 mai 2026. Le MSRV est passe a Rust 1.91. Les transitives pre-release heritees du pin 0.98 vont persister tant que SBFB reste sur 0.98. L'upgrade vers 1.0 resoudrait ce carry mais introduirait des breaking changes significatifs (PathWatcher -> PathList, ConnectionInfo -> WeakConnectionHandle, reexports elimines).

**Risque de S66 :** L'upgrade iroh 1.0 pourrait etre necessaire pour la durabilite si des bugs de persistence iroh-docs sont fixes uniquement dans la branche 1.0.

**Distribution :** Decision point S66. Soit upgrade iroh 1.0 (effort ~1-2 phases), soit rester sur 0.98 avec les transitives. Recommendation : rester sur 0.98 pour S65-S68, evaluer upgrade pour S69 (pilote).

### 1.16 P2-G-1 exe lock intermittent

**Status : MONITORING -- INCHANGE**

**Verification :** Bug intermittent de verrouillage de l'executable lors du build. Non reproduit 3 fois consecutivement. Probablement lie a Windows antivirus ou cargo target lock contention.

**Distribution :** Monitoring continu. Pas d'action sprint.

### 1.17 T-NN+2 iframe Rust-wasm

**Status : HORS SCOPE ROADMAP S65-S75**

**Verification :** Le carry concerne le remplacement de l'iframe PII SDK (onnxruntime-web) par une solution Rust-wasm. Les triggers (tract opset 19, ort wasm32-browser, gline-rs wasm-bindgen) ne sont toujours pas actifs. La PII SDK iframe fonctionne.

**Distribution :** Hors scope S65-S75. Reclassification possible post-Factory (S75+).

### 1.18 LT-2 Radicle sortie cap G7

**Status : TRIGGER PENDING -- tag v1.0 pose localement mais pas pousse**

**Verification :** Le trigger est le push du tag v1.0 vers origin. Le tag est pose localement (`cf1100b` ou anterieur). La doc de reference est `docs/release/MIRROR_FALLBACK.md`. Radicle est un fallback mirror pour la distribution du repo, pas du protocole.

**Distribution :** Trigger au push du tag v1.0. Si le tag est pousse pendant S65, le dry-run Radicle peut etre fait en S66 ou S67.

### 1.19 LT-5 redundancy persistence

**Status : VRAI GAP -- MAIS CONTEXTE CHANGE**

**Verification :** `docs/release/ROADMAP_COMMITMENTS.md:251-274` -- `RedundancyDispatcher` existe dans l'ancien code Python coordinator (`redundancy.py`) mais n'est instancie nulle part en production. Depuis le pivot Rust S50, le coordinator Python n'existe plus. Le `RedundancyDispatcher` Rust n'a pas ete re-implemente. Le trigger est "premier deploiement multi-worker" ou "tag v1.0 go-live".

**Impact S65-S75 :** Necessaire avant S69 (pilote externe) si des workers tiers participent. Si le pilote est "ferme" (2-3 testeurs sans workers), peut etre differe.

**Distribution :** S69 si workers tiers, sinon post-S75.

### 1.20 LT-7 self-hosted build

**Status : TIER 1+2 DONE, TIER 3 VALIDATED**

**Verification :** S55 a livre Tier 1+2. S60 a valide Tier 3 P2P infra. Worker quorum E2E est carry post-tag. La brique est fonctionnelle mais le quorum multi-worker E2E n'est pas teste.

**Distribution :** Le quorum E2E peut etre absorbe par S69 (pilote externe) ou S73 (Factory).

---

## 2. Graphe de dependances inter-sprints

### 2.1 Dependances explicites

```
S65 Contrat Public
  |---> S67 Gouvernance (vocabulaire de confiance = fondation gouvernance)
  |---> S68 Proof Pack (la taxonomie definit quoi prouver)
  |---> S71 RRV Proof Cards (les niveaux de preuve S65 = le schema des proof cards)

S66 Durabilite
  |---> S69 Pilote (un daemon qui perd ses donnees au restart est inutilisable en pilote)
  |---> S70 RRV LocalOnly (l'indexation locale necessite un feed qui survit au restart)

S67 Gouvernance
  |---> S72 SearchManifest (qui decide quel curator trust un manifest ?)

S68 Proof Pack
  |---> S69 Pilote (le proof pack EST le livrable du pilote)

S69 Pilote
  |---> S70 (le feedback pilote informe le design RRV)
  |     NOTE : S70 peut DEMARRER en parallele du pilote

S70 RRV LocalOnly
  |---> S71 Proof Cards (les resultats RRV portent les proof labels)
  |---> S75 Babel (RRV trouve les composants Babel)

S71 RRV Proof Cards
  |---> S72 SearchManifest (les proof cards enrichissent les manifests)

S73 Templates
  |---> S74 Broker/Sandbox (les templates sont utilises dans le broker)
  |---> S75 Babel (le template Babel sort de Factory)
```

### 2.2 Dependances cachees identifiees

**D-HIDDEN-1 : S65 -> S66 (auth tier avant persistence)**
Le fix FEED-INSERT-NO-AUTH-TIER (S65) doit etre fait AVANT que le feed devienne persistent (S66). Sinon, des operations non-autorisees seraient persistees indefiniment.

**D-HIDDEN-2 : S66 -> S72 (persistence avant SearchManifest P2P)**
Les SearchManifests doivent survivre aux restarts. Si le store est volatil, les manifests recus du reseau sont perdus.

**D-HIDDEN-3 : iroh 1.0 upgrade -> S66/S69**
iroh 1.0.0-rc.0 est sorti le 11 mai 2026. Si SBFB reste sur 0.98 et que n0 arrete de maintenir la branche 0.98 apres la sortie de 1.0, les bugs iroh-docs/iroh-blobs decouverts pendant le pilote n'auront pas de fix upstream. Decision point critique.

**D-HIDDEN-4 : S65 badge fix -> S73 Factory UX**
Factory affichera des apps avec des badges de confiance. Le vocabulaire clarifie en S65 doit etre utilise dans Factory, sinon regression du wording.

**D-HIDDEN-5 : S67 CuratorVouched feed op -> S72 SearchManifest**
`CuratorVouched` est defini dans la spec mais pas implemente. Il doit etre implemente pour que la gouvernance soit effective dans les manifests.

**D-HIDDEN-6 : wasmtime CVEs -> S74 Broker/Sandbox**
Le broker execute du code dans un sandbox. Si WASM/wasmtime est utilise pour l'isolation, les 12 CVEs avril 2026 doivent etre adresses (pin >= 43.0.1).

### 2.3 Graphe ASCII complet

```
                    S65 Contrat Public
                   / | \         \
                  /  |  \         \
         S66 Durabilite  \    S67 Gouvernance
           |     \        \       |
           |      \     S68 Proof Pack
           |       \        |
           |    S69 Pilote Ferme
           |        |
       S70 RRV Local  (peut chevaucher fin S69)
           |
       S71 Proof Cards
           |
       S72 SearchManifest
                              S73 Templates (peut demarrer // S70-S71)
                                  |
                              S74 Broker/Sandbox
                                  |
                              S75 Babel Dogfood
```

---

## 3. Zones rouges et risques transversaux

### 3.1 R-iroh-audit P0

**Etat :** iroh est un protocole reseau non audite par un tiers. Aucun audit de securite public de la pile iroh n'a ete publie. n0 computer n'a pas annonce d'audit en cours.

**Impact roadmap :**
- **S69 (pilote) :** Risque ELEVE. Le pilote expose le protocole iroh a des testeurs externes. Si une vuln iroh est decouverte pendant le pilote, c'est une interruption critique.
- **S72 (SearchManifest) :** Risque MOYEN. Le SearchManifest transitent via iroh-gossip/iroh-docs.
- **Mitigation :** SBFB ne peut pas auditer iroh. Le EXTERNAL_AUDIT_SCOPE.md (Sprint 28) documente iroh comme "upstream trust assumption". Le pilote doit etre ferme (amis, pas public) pour limiter l'exposition.

**Bloquant S69 ?** NON si le pilote est ferme. OUI si le pilote est ouvert au public.

### 3.2 R-wasmtime-cve P0

**Etat :** 12 CVEs avril 2026 dont 2 Critical (CVSS 9.0 sandbox escape). wasmtime n'est PAS une dependance directe de SBFB (pas dans Cargo.toml).

**Impact roadmap :**
- **S73-S75 (Factory) :** Si Factory utilise wasmtime pour l'isolation des workspaces, pin >= 43.0.1 obligatoire.
- **Architecture Factory :** La decision est : (a) wasmtime pour isolation WASM, (b) processus sandbox OS-level (nsjail/landlock/AppContainer), (c) iframe-only (pattern actuel). Le choix (c) evite completement le risque wasmtime.

**Recommendation :** Factory S73-S74 ne devrait PAS introduire wasmtime. L'isolation iframe + CSP actuelle est suffisante. Le broker execute des commandes OS (git clone, npm install, build), pas du WASM arbitraire. L'isolation doit etre OS-level (processus + filesystem sandbox), pas WASM.

### 3.3 R-libcrux-hax P2

**Etat :** libcrux (ML-KEM implementation) et hax (Rust formal verification) sont des dependances potentielles post-quantum. SBFB ne les utilise pas actuellement.

**Impact roadmap :** AUCUN sur S65-S75. Post-quantum crypto est hors scope (v2+, cf. threat model H2).

### 3.4 R-pyodide-escape

**Etat :** Pyodide permet l'execution Python dans le browser. Babel S75 utiliserait Pyodide pour les apps NLLB/traduction.

**Impact roadmap :**
- **S75 (Babel) :** L'app Babel s'execute dans une iframe sandbox `allow-scripts` sans `allow-same-origin`, avec CSP `connect-src 'none'`. Meme si Pyodide a une vuln, l'iframe ne peut pas communiquer avec le daemon (connect-src none) ni acceder aux cookies/localStorage du shell (opaque origin).
- **Risque residuel :** Pyodide pourrait exploiter un bug browser (WebAssembly engine) pour echapper a l'iframe. Risque accepte (meme surface que tout contenu web).

**Recommendation :** Acceptable pour S75. Pas de mitigation supplementaire necessaire au-dela de l'iframe sandbox.

### 3.5 iroh 0.98 pin — impact sur durabilite S66

**Etat CRITIQUE :** iroh 1.0.0-rc.0 est sorti le 11 mai 2026 (il y a 7 jours). La branche 0.98 n'aura probablement plus de maintenance active une fois iroh 1.0 stable sort (quelques semaines/mois). Les transitives pre-release (carry P2-AUDIT-2) ne seront resolues que par upgrade.

**Impact S66 :**
- iroh-docs 0.98 avec `Docs::persistent(path)` fonctionne pour la persistance locale. La recherche S66 confirme que le gap est dans runtime.rs (pas de data_dir passe a NodeConfig), pas dans iroh-docs lui-meme.
- iroh-blobs 0.100 `FsStore` est disponible pour la persistance blobs. Le commentaire `node.rs:23` ("Sprint 4 will add") est un TODO vieux de 60+ sprints.
- **Conclusion :** S66 peut se faire sur iroh 0.98 sans probleme. L'upgrade iroh 1.0 est un sprint dedie (effort ~2-3 phases avec breaking changes), a planifier apres S69 pilote ou en reserve.

**Decision point :** Si le pilote S69 revele des bugs iroh-docs/iroh-blobs fixes uniquement en 1.0, l'upgrade devient bloquant. Probabilite : FAIBLE (les bugs de persistence sont dans le code SBFB, pas dans iroh).

---

## 4. Estimation de complexite par sprint

### S65 — Contrat Public
| Dimension | Estimation |
|-----------|------------|
| Phases | A-D (4 phases) |
| Crates Rust | nexus-shell-daemon (feed_insert auth), nexus-coordinator-rs (verify_entry version guard) |
| Frontend | Browse.tsx, BrowsedProject.tsx, VerificationDetail.tsx (~3 composants) |
| Nouveaux composants | TrustBadge.tsx (nouveau), TrustTaxonomy.md (doc) |
| Risque technique | 2/5 (refactor UI + 2 fixes securite, pas de nouvelle archi) |
| Dependance externe | Aucune |
| Carry absorbes | FEED-INSERT-NO-AUTH-TIER (MANDATORY), BADGE-WORDING-PREMATURE, VERIFY-ENTRY-VERSION-GUARD, COMMIT-TITLE-FORMAT, REVIEW-ORDER, PYTHON-BLOCK-EXEMPTION, EXPLORER-ESCAPE-SINGLE-QUOTE |
| Delta tests estime | +8-12 Rust, +5-10 Vitest |

### S66 — Durabilite
| Dimension | Estimation |
|-----------|------------|
| Phases | A-E (5 phases, sprint technique lourd) |
| Crates Rust | nexus-core-rs (NodeConfig data_dir, FsStore), nexus-shell-daemon (runtime.rs boot sequence, feed_sync shutdown) |
| Frontend | Aucun changement |
| Nouveaux composants | Aucun (recablage interne) |
| Risque technique | 4/5 (persistence iroh-docs = changement fondamental du lifecycle daemon) |
| Dependance externe | iroh-blobs FsStore feature verification |
| Carry absorbes | FEED-JOIN-HANDLE-LEAK, ORPHAN-REPUBLISH-RECOVERY |
| Delta tests estime | +15-25 Rust (persistence, crash recovery, restart) |

### S67 — Gouvernance
| Dimension | Estimation |
|-----------|------------|
| Phases | A-D (4 phases) |
| Crates Rust | nexus-coordinator-rs (CuratorVouched feed op), nexus-shell-daemon-core (curator runtime enrichi) |
| Frontend | Curators.tsx (refonte), nouveau CuratorDetail.tsx |
| Nouveaux composants | CuratorVouched operation dans PublicFeedOperation enum |
| Risque technique | 3/5 (nouvelle operation feed + UI) |
| Dependance externe | Aucune |
| Carry absorbes | Aucun directement |
| Delta tests estime | +10-15 Rust, +5-8 Vitest |

### S68 — Proof Pack Release
| Dimension | Estimation |
|-----------|------------|
| Phases | A-D (4 phases) |
| Crates Rust | nexus-coordinator-rs (provenance, deploy E2E), nexus-shell-daemon (release endpoint) |
| Frontend | Nouveau ProofPack.tsx ou enrichissement VerificationDetail |
| Nouveaux composants | release-attest.sh (CI), SECURITY.md (doc) |
| Risque technique | 2/5 (assemblage de briques existantes) |
| Dependance externe | CI pipeline (GHA ou Woodpecker) |
| Carry absorbes | COVERAGE-DEPLOY-E2E, PROVENANCE-404-BRIDGE |
| Delta tests estime | +8-12 Rust, +3-5 Vitest |

### S69 — Pilote Ferme
| Dimension | Estimation |
|-----------|------------|
| Phases | A-E (5 phases, sprint operationnel lourd) |
| Crates Rust | Fixes uniquement (bug reports pilote) |
| Frontend | Onboarding flow, monitoring dashboard minimal |
| Nouveaux composants | Onboarding guide externe, monitoring scripts |
| Risque technique | 5/5 (premiere exposition a des utilisateurs reels) |
| Dependance externe | 2-3 testeurs externes, infrastructure VPS/LAN |
| Carry absorbes | VERIFY-LOCAL-KEY-ONLY, PLAYWRIGHT-SPECS-STALE (re-ecriture) |
| Delta tests estime | +5-10 Rust (fixes), +10-15 Playwright (re-ecriture) |

### S70 — RRV LocalOnly
| Dimension | Estimation |
|-----------|------------|
| Phases | A-D (4 phases) |
| Crates Rust | Nouveau crate ou module dans nexus-coordinator-rs (index local) |
| Frontend | Nouveau SearchBar.tsx, SearchResults.tsx |
| Nouveaux composants | Index local SQLite FTS5, query parser |
| Risque technique | 3/5 (FTS5 = bien connu, mais design UX recherche = iterable) |
| Dependance externe | SQLite FTS5 (deja dans rusqlite) |
| Carry absorbes | Aucun |
| Delta tests estime | +12-18 Rust, +8-12 Vitest |

### S71 — RRV Proof Cards
| Dimension | Estimation |
|-----------|------------|
| Phases | A-C (3 phases, sprint plus court) |
| Crates Rust | Extension du module search |
| Frontend | ProofCard.tsx (composant de resultat enrichi) |
| Nouveaux composants | ProofLabel schema dans les resultats |
| Risque technique | 2/5 (enrichissement UI, pas de nouvelle archi) |
| Dependance externe | Aucune |
| Carry absorbes | Aucun |
| Delta tests estime | +5-8 Rust, +5-8 Vitest |

### S72 — SearchManifest Opt-In
| Dimension | Estimation |
|-----------|------------|
| Phases | A-D (4 phases) |
| Crates Rust | nexus-coordinator-rs (SearchManifestPublished feed op), nexus-shell-daemon-core (discovery gossip) |
| Frontend | SearchSettings.tsx (opt-in toggle) |
| Nouveaux composants | SearchManifestPublished operation, manifest schema |
| Risque technique | 4/5 (nouvelle operation feed P2P, discovery, trust boundary) |
| Dependance externe | Aucune |
| Carry absorbes | Aucun |
| Delta tests estime | +15-20 Rust, +5-8 Vitest |

### S73 — Code Factory Templates
| Dimension | Estimation |
|-----------|------------|
| Phases | A-D (4 phases) |
| Crates Rust | Nouveau module templates (git init, scaffolding) |
| Frontend | FactoryWizard.tsx, TemplateSelector.tsx |
| Nouveaux composants | Template registry, project scaffolding |
| Risque technique | 2/5 (git init + file copy, pas de nouvelle crypto) |
| Dependance externe | Templates repo structure |
| Carry absorbes | Aucun |
| Delta tests estime | +10-15 Rust, +8-12 Vitest |

### S74 — Code Factory Broker/Sandbox
| Dimension | Estimation |
|-----------|------------|
| Phases | A-E (5 phases, sprint technique lourd) |
| Crates Rust | Nouveau crate nexus-factory-broker ou module (processus sandbox, IPC) |
| Frontend | FactoryConsole.tsx (output streaming) |
| Nouveaux composants | Broker IPC, sandbox wrapper (OS-level, pas wasmtime) |
| Risque technique | 4/5 (isolation OS = complexe, multi-platform) |
| Dependance externe | OS sandbox APIs (Windows AppContainer, Linux landlock/nsjail) |
| Carry absorbes | Aucun |
| Delta tests estime | +15-25 Rust, +5-8 Vitest |

### S75 — Babel Dogfood / Domain Packs
| Dimension | Estimation |
|-----------|------------|
| Phases | A-E (5 phases, sprint applicatif) |
| Crates Rust | Minimal (Babel = app externe utilisant le protocole) |
| Frontend | Babel app (repo separe), Factory integration |
| Nouveaux composants | Babel template, domain pack schema |
| Risque technique | 3/5 (premiere vraie app = integration test du protocole entier) |
| Dependance externe | NLLB-200 model, Pyodide, Gutenberg data |
| Carry absorbes | Aucun |
| Delta tests estime | +5-10 Rust (protocol tests), Babel tests dans son repo |

---

## 5. Sequencage optimal

### 5.1 Chemin critique

Le chemin critique est la plus longue chaine de dependances :

```
S65 -> S66 -> S69 -> S70 -> S71 -> S72
```

**6 sprints en serie = ~12-16 semaines** au rythme actuel (2 semaines/sprint).

L'arc 3 (S73-S75) n'est PAS sur le chemin critique car il peut demarrer en parallele de S70-S72.

### 5.2 Parallelisations possibles

| Periode | Sprint principal | Sprint parallele possible |
|---------|------------------|--------------------------|
| Semaines 1-2 | S65 Contrat Public | -- |
| Semaines 3-4 | S66 Durabilite | -- |
| Semaines 5-6 | S67 Gouvernance | -- |
| Semaines 7-8 | S68 Proof Pack | -- |
| Semaines 9-10 | S69 Pilote | S73 Templates (si dev separe) |
| Semaines 11-12 | S70 RRV Local | S73 Templates (continuation) |
| Semaines 13-14 | S71 Proof Cards | S74 Broker/Sandbox (si dev separe) |
| Semaines 15-16 | S72 SearchManifest | S74 Broker/Sandbox (continuation) |
| Semaines 17-18 | S75 Babel Dogfood | -- |

**MAIS** : SBFB est un projet solo-maintainer (pattern OpenBSD, cf. vision_model.md). La parallelisation n'est possible que si un contributeur externe prend l'arc 3. En pratique, les sprints sont sequentiels.

**Sequencage realiste solo-maintainer :**

```
S65 (2 sem) -> S66 (2 sem) -> S67 (2 sem) -> S68 (1.5 sem)
-> S69 (2 sem + 1-2 sem pilote actif) -> S70 (2 sem)
-> S71 (1.5 sem) -> S72 (2 sem) -> S73 (2 sem) -> S74 (2 sem)
-> S75 (2 sem)
```

**Total : ~22-24 semaines = ~5.5-6 mois.**

### 5.3 Gates entre arcs

**Gate 1 : Arc 1 -> Arc 2 (apres S69)**
- **Condition :** Le pilote ferme est operationnel, les bugs critiques sont fixes.
- **Go/no-go :** Si le pilote revele des problemes fondamentaux (persistence, sync P2P, crash), les fixes absorbent le sprint reserve (non dans la roadmap S65-S75, mais herite de l'ancien S6 reserve).
- **Decision :** PO evalue le feedback pilote. Si > 5 bugs P0/P1, S70 est reporte.

**Gate 2 : Arc 2 -> Arc 3 (apres S72)**
- **Condition :** SearchManifest fonctionne opt-in, RRV local trouve des briques.
- **Go/no-go :** Si RRV local est insuffisant pour Factory (pas assez de briques indexees), Factory demarre sans RRV (templates statiques uniquement).
- **Decision :** PO evalue si RRV@dev est fonctionnel. Si oui, Factory l'integre. Sinon, Factory est standalone.

### 5.4 Points de decision go/no-go

| Sprint | Point de decision | Consequence no-go |
|--------|-------------------|-------------------|
| S65 fin | Taxonomie de confiance coherente ? | Reporter S67 gouvernance |
| S66 fin | Daemon survit 10 restarts sans perte ? | Sprint fix supplementaire |
| S68 fin | Proof pack complet et reproductible ? | Reporter pilote |
| S69 Phase C | Pilote 48h sans P0 ? | Sprint fix, re-pilote |
| S70 fin | FTS5 index >= 100 briques ? | Factory sans RRV |
| S72 fin | Manifest P2P sync stable 3 noeuds ? | SearchManifest reste local |
| S74 fin | Sandbox isole sans escape ? | Babel sans sandbox broker |

---

## 6. Distribution carry items par sprint

### S65 — OBLIGATOIRE
| Item | Raison |
|------|--------|
| P2-FEED-INSERT-NO-AUTH-TIER | 3/3 MANDATORY, securite feed |
| P2-VERIFY-ENTRY-VERSION-GUARD | Pre-go-live, 5 LOC |
| P2-BADGE-WORDING-PREMATURE | Coeur du sprint S65 |

### S65 — Dette pair
| Item | Raison |
|------|--------|
| P2-COMMIT-TITLE-FORMAT | Process fix, 20 LOC |
| P2-REVIEW-ORDER | Process fix, 10 LOC |
| P2-PYTHON-BLOCK-EXEMPTION | Reclassification resolved |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE | 1 LOC fix |

### S66 — Absorbe naturellement
| Item | Raison |
|------|--------|
| P2-FEED-JOIN-HANDLE-LEAK | Shutdown lifecycle = sujet S66 |
| P2-ORPHAN-REPUBLISH-RECOVERY | Crash recovery = sujet S66 |

### S68 — Absorbe naturellement
| Item | Raison |
|------|--------|
| P2-PROVENANCE-404-BRIDGE | UX verification = proof pack |
| P2-COVERAGE-DEPLOY-E2E | Deploy roundtrip E2E = proof pack |

### S69 — Absorbe naturellement
| Item | Raison |
|------|--------|
| P2-VERIFY-LOCAL-KEY-ONLY | Cross-node verification = pilote |
| P2-PLAYWRIGHT-SPECS-STALE | Re-ecriture specs = pilote QA |

### Monitoring continu (pas de sprint specifique)
| Item | Raison |
|------|--------|
| P2-A-1 rand blocker | Upstream, pas d'action |
| P2-AUDIT-2 iroh transitives | Decision point S66 |
| P2-G-1 exe lock | Monitoring, pas reproductible |

### Hors scope S65-S75
| Item | Raison |
|------|--------|
| T-NN+2 iframe Rust-wasm | Triggers non actifs |
| LT-5 redundancy persistence | Post-S75 sauf si pilote S69 l'exige |
| LT-7 quorum E2E | Post-S75 sauf si pilote S69 l'exige |

### Trigger-dependent
| Item | Raison |
|------|--------|
| LT-2 Radicle | Trigger = push tag v1.0. S66 ou S67 si pousse |

---

## 7. Risques de sequencage

### 7.1 Que se passe-t-il si S69 (pilote) revele des problemes fondamentaux ?

**Scenario :** Le pilote revele que la sync P2P perd des entrees, que le feed se corrompt apres 24h, ou que les restarts cassent l'etat.

**Impact :** Sprint(s) fix supplementaire(s) entre S69 et S70. L'arc 2 (RRV) est retarde de 2-4 semaines.

**Mitigation :** 
- S66 (durabilite) est specifiquement concu pour prevenir ce scenario
- Le pilote est FERME (2-3 amis, pas public) pour limiter la surface des issues
- Le test E2E nouveau noeud (S64 Phase D) couvre deja le scenario de base

**Probabilite :** MOYENNE (30%). Les bugs de persistence (gap S66 #1 : iroh-docs en memoire) sont connus et seront fixes. Les bugs imprevus sont les interactions multi-noeud en conditions reelles.

**Plan B :** S70 absorbe les fixes si < 3 bugs P0. Sinon, sprint fix dedie.

### 7.2 Que se passe-t-il si iroh 0.98 pose des problemes de durabilite (S66) ?

**Scenario :** `Docs::persistent(path)` sur iroh-docs 0.98 a un bug (corruption redb, perte d'entrees apres restart, performance degradee).

**Impact :** Necessaire d'upgrader iroh 1.0 (effort ~2-3 phases avec breaking changes API). S66 se transforme en sprint iroh upgrade + durabilite = 6 phases.

**Mitigation :**
- Le test `persistent_data_dir_reboots_with_same_doc_and_author()` existe deja dans `node.rs` et passe sur iroh 0.98. La persistence iroh-docs fonctionne en unit test.
- Le vrai risque est la persistence sous charge (multi-writer, beaucoup d'entrees, interruptions). Ceci sera decouvert en S66 Phase tests, pas en S69 pilote (trop tard).

**Probabilite :** FAIBLE (15%). La persistence iroh-docs est une feature mature (redb backend) qui fonctionne en tests unitaires.

**Plan B :** Si persistence buggy, fallback = stocker TOUTES les entrees dans SQLite (deja fait pour le feed) et utiliser iroh-docs uniquement comme transport P2P. Resilient.

### 7.3 Le RRV (S70-72) est-il vraiment necessaire avant Factory (S73-75) ?

**Reponse : NON.** La recherche `sbfb_project_factory_rrv_oss_research.md` dit explicitement :

> "Ne pas attendre le RRV complet. [...] Project Factory n'est pas une app iframe toute-puissante. Project Factory est une UI + broker local + workspace sandbox + index @dev."

**Implications :**
- S73 (Templates) peut demarrer SANS S70-S72. Les templates sont des structures de fichiers, pas des resultats de recherche.
- S74 (Broker/Sandbox) peut demarrer SANS S70-S72. Le broker execute des commandes OS, pas des queries RRV.
- S75 (Babel) BENEFICIE de RRV mais n'en a pas BESOIN. Babel peut etre construit a partir d'un template statique + config manuelle.

**Recommendation :** Si le rythme le permet, faire S70 avant S73. Sinon, S73 peut chevaucher S70-S71 sans blocage.

### 7.4 Peut-on faire S73 (templates) en parallele de S70 ?

**Reponse : OUI, avec des reserves.**

**Argument pour :**
- S73 ne depend PAS de S70. Les templates sont des structures de fichiers statiques.
- S73 est faible risque (2/5) et ne touche pas les memes crates que S70.
- Cela raccourcirait la roadmap totale de ~2 semaines.

**Argument contre :**
- Solo maintainer = pas de parallelisation reelle. Un sprint a la fois.
- S73 en parallele de S70 = context switch couteux.
- Le feedback du pilote S69 pourrait changer les priorites de S73.

**Recommendation :** Sequentiel. S73 apres S72 (ou apres S71 si S72 est differe).

---

## 8. Resume des decisions critiques

### 8.1 iroh 0.98 vs 1.0

**Decision :** Rester sur 0.98 pour S65-S69. Evaluer upgrade pour S70+ apres que 1.0 stable sorte.

**Rationale :** L'upgrade 1.0 est un effort ~2-3 phases (MSRV 1.91, API PathWatcher/ConnectionInfo, reexports). Le faire pendant l'arc 1 (credibilite publique) serait une distraction. Le faire apres le pilote permet de profiter de la stabilisation 1.0.

### 8.2 wasmtime vs OS sandbox pour Factory

**Decision :** OS sandbox (processus + filesystem isolation), pas wasmtime.

**Rationale :** Factory execute des commandes OS (git, npm, cargo), pas du code WASM. wasmtime introduirait 12 CVEs + surface d'attaque + complexite. L'iframe sandbox + CSP actuelle gere deja l'isolation des apps. Le broker a besoin d'isoler des processus OS, pas des modules WASM.

### 8.3 Pilote ferme vs ouvert

**Decision :** Ferme (2-3 amis/collegues).

**Rationale :** R-iroh-audit P0 rend le pilote public irresponsable sans audit tiers. Le pilote ferme donne le feedback necessaire sans exposer des inconnus a des risques de securite non audites.

### 8.4 Sequencage Arc 2 / Arc 3

**Decision :** Sequentiel, Arc 2 avant Arc 3. Sauf si feedback pilote S69 rend Arc 2 non prioritaire, auquel cas Arc 3 passe devant.

**Rationale :** RRV enrichit Factory mais n'est pas bloquant. Le go/no-go apres S69 pilote decidera.

---

## 9. Calendrier previsionnel

| Semaine | Sprint | Arc | Theme |
|---------|--------|-----|-------|
| 1-2 | S65 | 1 | Contrat Public + 7 carry items |
| 3-4 | S66 | 1 | Durabilite (persistence + crash recovery) |
| 5-6 | S67 | 1 | Gouvernance (CuratorVouched + UI) |
| 7-8 | S68 | 1 | Proof Pack (release pipeline + evidence) |
| 9-11 | S69 | 1 | Pilote Ferme (deploy + feedback) |
| -- | **GATE 1** | -- | Go/no-go Arc 2 |
| 12-13 | S70 | 2 | RRV LocalOnly (FTS5 index) |
| 14-15 | S71 | 2 | Proof Cards (resultats enrichis) |
| 16-17 | S72 | 2 | SearchManifest (P2P discovery) |
| -- | **GATE 2** | -- | Go/no-go Arc 3 |
| 18-19 | S73 | 3 | Templates (scaffolding) |
| 20-21 | S74 | 3 | Broker/Sandbox (isolation OS) |
| 22-24 | S75 | 3 | Babel Dogfood (premiere app) |

**Total : ~24 semaines = ~6 mois (mai 2026 -> novembre 2026).**

**Contingence :** +2-4 semaines pour fixes pilote, iroh upgrade eventuel, ou gates echouees.

---

## 10. Tests delta projete cumule

| Sprint | Rust entry | Rust exit | Vitest exit | Total |
|--------|-----------|-----------|-------------|-------|
| S65 | 1326 | ~1338 | ~275 | ~1619 |
| S66 | ~1338 | ~1363 | ~275 | ~1644 |
| S67 | ~1363 | ~1378 | ~283 | ~1667 |
| S68 | ~1378 | ~1390 | ~288 | ~1684 |
| S69 | ~1390 | ~1400 | ~303 | ~1709 |
| S70 | ~1400 | ~1418 | ~315 | ~1739 |
| S71 | ~1418 | ~1426 | ~323 | ~1755 |
| S72 | ~1426 | ~1446 | ~331 | ~1783 |
| S73 | ~1446 | ~1461 | ~343 | ~1810 |
| S74 | ~1461 | ~1486 | ~351 | ~1843 |
| S75 | ~1486 | ~1496 | ~361 | ~1863 |

**Projection S75 : ~1863 tests totaux** (vs 1597 actuels, +266 net).

---

## 11. Synthese carry items — tableau decision final

| # | Item | Compteur | Status code | Sprint cible | Complexite |
|---|------|----------|-------------|--------------|------------|
| 1 | P2-FEED-INSERT-NO-AUTH-TIER | 3/3 | VRAI GAP | **S65 MANDATORY** | 30-50 LOC |
| 2 | P2-VERIFY-ENTRY-VERSION-GUARD | 1/3 | VRAI GAP | **S65** | 5 LOC |
| 3 | P2-BADGE-WORDING-PREMATURE | pre-S14 | VRAI GAP | **S65** (coeur sprint) | 80-120 LOC |
| 4 | P2-COMMIT-TITLE-FORMAT | 2/3 | PROCESS | S65 dette | 20 LOC |
| 5 | P2-REVIEW-ORDER | 2/3 | PROCESS | S65 dette | 10 LOC |
| 6 | P2-PYTHON-BLOCK-EXEMPTION | 2/3 | QUASI-OBSOLETE | S65 reclassification | 5 LOC |
| 7 | P2-EXPLORER-ESCAPE-SINGLE-QUOTE | 2/3 | VRAI GAP | S65 dette | 1 LOC |
| 8 | P2-FEED-JOIN-HANDLE-LEAK | 1/3 | VRAI GAP | S66 | 40-60 LOC |
| 9 | P2-ORPHAN-REPUBLISH-RECOVERY | 1/3 | VRAI GAP | S66 | 60-100 LOC |
| 10 | P2-PROVENANCE-404-BRIDGE | 2/3 | VRAI GAP | S68 | 20-30 LOC |
| 11 | P2-COVERAGE-DEPLOY-E2E | 2/3 | VRAI GAP | S68 | 100-150 LOC |
| 12 | P2-VERIFY-LOCAL-KEY-ONLY | 2/3 | VRAI GAP | S69 | 50-80 LOC |
| 13 | P2-PLAYWRIGHT-SPECS-STALE | 2/3 | VRAI GAP | S65 (suppr) + S69 (re-ecriture) | 12 fichiers suppr + 200-400 LOC |
| 14 | P2-A-1 rand blocker | ext | UPSTREAM | Monitoring | -- |
| 15 | P2-AUDIT-2 iroh transitives | ext | UPSTREAM | Decision S66 | -- |
| 16 | P2-G-1 exe lock | monitoring | INTERMITTENT | Monitoring | -- |
| 17 | T-NN+2 iframe Rust-wasm | hors cap | HORS SCOPE | Post-S75 | -- |
| 18 | LT-2 Radicle | trigger | TRIGGER v1.0 push | S66-S67 | -- |
| 19 | LT-5 redundancy persistence | latent | VRAI GAP | Post-S75 (sauf S69) | -- |
| 20 | LT-7 self-hosted build | latent | T1+2 DONE | Post-S75 (sauf S69) | -- |

---

## 12. Recommandation finale

**L'ordre propose (S65->S75) est le bon.** Les dependances cachees confirment que :

1. S65 (vocabulaire de confiance) est le socle de tout. AUCUN sprint ne devrait le preceder.
2. S66 (durabilite) est le prerequis technique le plus critique. Un daemon qui perd ses donnees au restart rend tout le reste inutile.
3. S67-S68 sont inter-dependants mais S67 -> S68 est le bon ordre (la gouvernance informe le proof pack).
4. S69 est le sprint le plus risque (premiere exposition externe) et doit etre blinde par les 4 sprints precedents.
5. L'arc 2 (S70-S72) peut etre raccourci si S70 montre que RRV n'est pas necessaire pour Factory.
6. L'arc 3 (S73-S75) est le plus independant et pourrait demarrer plus tot si un contributeur externe rejoint.

**Le risque principal n'est pas technique, c'est operationnel :** le pilote S69 est le premier contact avec des utilisateurs reels. Tout ce qui precede doit etre solide. L'iroh 0.98 pin est le second risque (maintenabilite a moyen terme), mais il est gerable en restant sur 0.98 pour l'arc 1 et en evaluant l'upgrade pour l'arc 2.
