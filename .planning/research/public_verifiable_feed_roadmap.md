# Roadmap — SBFB Public Verifiable Protocol Feed

**Date:** 2026-05-13
**Statut:** valide PO, non engage en sprint
**Estimation:** 6 sprints (5+1 reserve), ~12-16 semaines au rythme actuel
**Prerequis:** S61 audit S60 termine
**Qualification:** 5 sprints = compression controlee avec gate de scission au S2. 6 sprints = estimation responsable pour credibilite publique.
**Source:** analyse factuelle deep par team 5 agents (2026-05-13), corrigee par le PO.

---

## 0. Contexte et diagnostic

### Ce qui existe deja (verifie factuellement)

| Brique | Evidence |
|---|---|
| Deploy-from-repo + SBFB.json + provenance SLSA L1 | `deploy.rs:65-249`, `provenance.rs:28-54` |
| is_open_source derive daemon-side | `http.rs:883-902` (guard), `publish.rs:64-75` (doc) |
| 14 domaines canoniques JCS/RFC8785 + domain separation | `canonical.rs` — task, result, claim, invite, kudos, curator_list, provenance, warrant_canary, pow, duress_ack, age_witness, contributor_attestation, key_rotation, delegation_cert |
| ProjectAnnouncement versione (v, msg_type, node_id) | `publish.rs:31-76`, validation `publish.rs:160-176` |
| Curator lists signees Ed25519 (5 verifications + revocation) | `curator.rs:201-341` |
| Build tasks task_type:"build" + quorum SHA256 | `dispatcher.rs:18`, `validator.rs:113-147`, redundancy >= 3 |
| Kudos ledger hash-chain BLAKE3 + EMA alpha=0.97 | `kudos_ledger.rs:24-145` |
| PoW Hashcash SHA256 publisher+topic bound, escalating | `pow.rs` brique mature, tests couvrants |
| 3 rate-limiters GCRA (worker, storage 10/min, browse 10/min) | `rate_limit.rs`, `storage_limiter.rs`, `browse_limiter.rs` |
| Quarantine SQLite TTL 15min + CLI/HTTP management | `quarantine_queue.rs` |
| Age witness Ed25519 attestation (7j min, 30j witness) | `age_witness.rs` brique mature |
| AppStorage P2P via iroh-docs (MVP hardcode sbfb-ideas) | `storage_api.rs:29` REPLICATED_APPS |
| Multi-node tests automatises (6 tests, 2 MANDATORY E2E) | `multi_daemon.rs` |
| Protocol Explorer MVP (5 sections + live status) | `examples/sbfb-explorer/` |
| Installers Win NSIS + Linux .deb + macOS .dmg | S60 livre |
| 1259 Rust / 258 Vitest / 6/6 size-limit | S60 tag v1.0 |

### Ce qui manque (verifie factuellement)

| Gap | Evidence |
|---|---|
| Feed protocolaire rejouable | `gossip_outbox` = buffer local, pas de log append-only reseau |
| Log des transitions d'etat | Aucun event sourcing. Pas d'event quand Draft→Verified, quorum atteint, source stale |
| Endpoint/CLI verify-release | `verify_provenance()` existe (`provenance.rs:56-86`) mais non expose HTTP/CLI |
| UI "pourquoi ce projet est verifie" | Badge "Verifie" ShieldCheck seulement (`BrowsedProject.tsx:271-279`), aucun detail |
| Sync durable entre noeuds | Gossip = live-only. Pas de catch-up historique, pas de cursor |
| Spec PublicFeedOperation | Recherche narrative dans `p2panda_public_protocol_briques.md`, pas de types/tests/schema |

### Synthese

SBFB sait publier, signer, verifier, browser, builder et limiter certains abus. Mais un tiers ne peut pas encore rejouer une histoire complete du type : ce projet a ete publie depuis ce repo, cette release pointe vers cette provenance, ce build a atteint quorum, ces curators l'ont recommande, cette vue Browse vient de ces evenements verifiables.

---

## 1. Plan en 6 sprints

### Sprint 1 — Spec executable + feed local rejouable

**Objectif fonctionnel :** Un noeud SBFB peut enregistrer les evenements publics du reseau dans un feed local append-only signe, les rejouer depuis zero, et reconstruire une vue Browse/Public Registry.

**Phases :**
- Phase A : Spec executable — types Rust `PublicFeedOperation`, domaine canonique `DOMAIN_FEED_V1`, signature d'operation, schema SQLite, cursor format, regles de replay, vectors de test. Formaliser la spec dans `docs/protocol/PUBLIC_FEED_SPEC.md`. Politique versioning post-v1.0 explicite (chaque break bumpe la version).
- Phase B : Feed local — store append-only SQLite, hash-chain BLAKE3 (pattern kudos_ledger), insertion d'operations, replay depuis zero.
- Phase C : Materialisation + cursor — `PublicRegistryView` materialisee depuis le feed, cursor persiste, reprise apres interruption. Integration minimale avec `BrowseAggregator` comme source supplementaire.
- Phase D : Tests — hash-chain integrity, transitions d'etat (ReleasePublished → SourceBecameStale), Local Draft ne peut pas apparaitre comme Verified Release, corruption detection, cursor restart.

**Operations minimales Sprint 1 :**
- `ReleasePublished` (core — obligatoire)
- `SourceBecameStale` (transition de lifecycle — obligatoire)

Les operations `CuratorVouched` et `BuildQuorumReached` sont definies dans la spec Phase A mais implementees dans un sprint suivant. Sprint 1 se concentre sur le cycle de vie release, pas sur les signaux sociaux/quorum.

**Prerequis :** Recherche p2panda deja faite (65-75% narrative spec dans `p2panda_public_protocol_briques.md`). Reste a convertir en types Rust + tests.

**Patterns reutilisables :** hash-chain kudos_ledger, gossip_outbox schema, BrowseAggregator builder pattern.

---

### Sprint 2 — Sync P2P durable + anti-spam minimal

**Objectif fonctionnel :** Deux ou trois noeuds synchronisent le feed public. Un noeud qui revient apres une periode offline rattrape l'historique. Les operations sont protegees par PoW + rate-limit + quarantine.

**Phases :**
- Phase A : Feed foundation sync — integration iroh-docs comme transport (precedent : AppStorage S58), ticket-based join, subscribe LiveEvent.
- Phase B : Catch-up offline — noeud B offline, noeud A publie N operations, noeud B redemarre et rattrape. Cursor sync persiste.
- Phase C : Multi-daemon E2E — tests 2-3 noeuds, replay idempotent, validation hash-chain apres sync.
- Phase D : Anti-spam minimal — PoW feed (primitive reutilisable, integration hot path feed a faire), rate-limit keyed (author_pubkey, feed_topic) (primitive GCRA reutilisable, instantiation feed a faire), quarantine feed (primitive generique, branchement producteur feed a faire), age witness gate (primitive reutilisable, integration admission feed a faire).

**Gate de scission :** Si Sprint 2 ne prouve PAS offline catch-up + replay idempotent + 2/3 noeuds + anti-spam hot path, alors le Sprint 2 se scinde et le plan passe a 7 sprints. Cette evaluation se fait en review Phase C.

**Risques identifies :**
- Multi-writer consensus sur sequence numbers (iroh-docs gere par timestamp tiebreaker, mais besoin de validation)
- Ordering causal (ReleasePublished avant CuratorVouched sur meme release)
- Network timing jitter (precedent : S55 Phase D jitter ±15s)
- Anti-spam : PoW/rate-limit/quarantine sont generiques mais pas encore branches sur le hot path feed (quarantine payload_json = string generique, mais producteur pas encore wired)

**Mecanismes anti-spam — primitives reutilisables :**
| Mecanisme | Statut primitive | Integration hot path feed |
|---|---|---|
| PoW (publisher+topic generic) | Brique mature, topic/pubkey parametrable | Pas encore branchee sur le feed |
| GCRA rate-limit (keyed generic) | Brique mature, cle quelconque hashable | Instantiation avec cle feed a faire |
| Quarantine (payload_json generique) | Brique mature, payload string generique | Producteur feed pas encore wire |
| Age witness (pure function) | Brique mature, verification standalone | Admission feed pas encore gatee |

**Attention PO :** Les primitives existent et sont generiques, mais aucune n'est encore branchee sur le hot path feed. L'integration est du glue code, mais les tests adversariaux (Sprint 4) et le pilote externe (Sprint 5) reveleront les gaps d'une protection minimale.

---

### Sprint 3 — Verification tiers + UX

**Objectif fonctionnel :** Un utilisateur non-technique peut voir "pourquoi ce projet est verifie" dans l'UI. Un developpeur peut verifier une release via endpoint HTTP ou CLI. Une app iframe peut appeler `verifyRelease()` via le bridge.

**Phases :**
- Phase A : Endpoint HTTP — `GET /api/v1/project/{id}/provenance` expose `verify_provenance()` (pure function existante `provenance.rs:56-86`). Table SQLite pour stocker/retrouver les ProvenanceRecord par project_id.
- Phase B : Bridge methods — `getProvenanceRecord(projectId)`, `verifyRelease(projectId)`, `getPublicFeedCursor()` dans `sbfb-bridge.js`. API client TypeScript + schema Zod.
- Phase C : UI proof-chain — composant `VerificationDetail` (modal/drawer). Badge "Verifie" cliquable → detail : repo URL, commit SHA, artifact hash, provenance signature, build quorum, curators. Vue "pourquoi ce projet est verifie".
- Phase D : Protocol Explorer avance — nouvelle section "Verification & Provenance" dans `examples/sbfb-explorer/`. Demo live de verification.

---

### Sprint 4 — Hardening public cible

**Objectif fonctionnel :** Le feed public resiste aux attaques adversariales. Un nouveau noeud from scratch peut rattraper et verifier le feed complet. Les cas de corruption, spam, mauvais repo, mauvais hash et restart sont couverts par des tests.

**Phases :**
- Phase A : Tests adversariaux feed — replay hors ordre, corruption hash-chain, fork-bomb (1000 operations spam), operations avec mauvais repo/hash, signatures forgees, payloads oversized.
- Phase B : Tests adversariaux crypto — couvrir les 7 primitives crypto × 6 wire formats. Ed25519 forgery, BLAKE3 tampering, PoW difficulty bypass, age witness future timestamp, canonical bytes manipulation.
- Phase C : Nouveau noeud from scratch — scenario complet : daemon neuf, join reseau, sync feed entier, reconstruire Browse, verifier toutes les preuves, valider coherence. Test E2E multi-daemon.
- Phase D : Documentation protocole — `docs/protocol/PUBLIC_FEED_SPEC.md` finalise. Schema d'operations, regles de replay, politique versioning, exemples JSON, guide "comment verifier une release SBFB".

**Couverture actuelle :** tres faible en tests adversariaux (quelques tests de rejet de signature/payload invalide). Ce sprint vise une couverture significative des scenarios adversariaux sur le feed public.

---

### Sprint 5 — Go-live public

**Objectif fonctionnel :** SBFB est deploye publiquement avec attestations de release, tag v1.0 pousse, binaires distribues, et au moins un pilote externe qui utilise le protocole.

**Phases :**
- Phase A : Release pipeline — workflow CI release sur tag (GHA ou Woodpecker). Attestations SLSA (`release-attest.sh`). Binaires signes (launcher + daemon + worker). SHA256 + `.intoto.jsonl`. Tag public pousse vers origin (v1.0 si pas encore pousse, ou tag suivant si v1.0 deja publie a ce stade).
- Phase B : Evidence pack auditeur — scope freeze commit dans `EXTERNAL_AUDIT_SCOPE.md`. Bundle : THREAT_MODEL + BUILDING.md + domaines crypto + crate priority. PGP key pour `security@sbfb.network` publiee. SECURITY.md finalise.
- Phase C : Pilote externe — groupe ferme (2-3 testeurs externes). 3 curators, 2-3 projets publies. Monitoring feed sync, crash reports, feedback UX. Documentation onboarding testeur.
- Phase D : Mirrors + fallback — pkarr relay failover wiring daemon-side. Mirror fallback operationnel. Bootstrap allowlist cleanup (post-v1.0, transition vers age-witness only).

---

### Sprint 6 (reserve) — Hardening post-pilote + RRV optionnel

**Objectif fonctionnel :** Corriger les problemes remontes par le pilote externe. Optionnellement, integrer SearchManifestPublished dans le feed public.

**Ce sprint existe car :**
- Le pilote externe (S5 Phase C) remontera inevitablement des issues non prevues
- L'anti-spam minimal (S2 Phase D) peut necessiter un renforcement apres exposition publique
- La sync P2P durable (S2) peut reveler des edge cases sous charge reelle
- Le gate de scission S2 peut avoir ete declenche, auquel cas ce sprint absorbe le debordement

**Phases :**
- Phase A : Fixes pilote — resolution des issues remontees par les testeurs externes
- Phase B : Anti-spam renforce (si necessaire) — fork-bomb tests publics, politique Sybil/reputation avancee, quotas dynamiques
- Phase C : RRV optionnel — `SearchManifestPublished` dans le feed, recherche verified-only, preuve retournee avec chaque resultat
- Phase D : Audit prep — vendor engagement RFP (Trail of Bits recommande), scope freeze final, documentation gaps

---

## 2. Jalons de credibilite

| Apres | Statut |
|---|---|
| Sprint 1 | Feed local rejouable, spec executable — demo interne |
| Sprint 2 | Sync P2P + anti-spam minimal — demo serieuse |
| Sprint 3 | Verification tiers + UX — MVP protocolaire utilisable |
| Sprint 4 | Hardening adversarial — protocole robuste |
| Sprint 5 | Go-live public + pilote — credible publiquement |
| Sprint 6 | Post-pilote + RRV — protocole mature |

---

## 3. Dependances et risques

### Risque principal : Sprint 2 (sync P2P)

Le seul sprint a haut risque. La sync P2P durable est un probleme fondamentalement different du gossip live. Les precedents (AppStorage S58) montrent que iroh-docs fonctionne pour du single-writer key-value, mais le feed public est multi-writer avec ordering causal.

**Gate de scission :** Evaluation Phase C review. Si offline catch-up + replay idempotent + 2/3 noeuds + anti-spam hot path ne sont pas tous prouves, scission automatique.

### Risque secondaire : Anti-spam feed

"100-120 LOC anti-spam minimal" est credible pour brancher les primitives existantes. Ce n'est pas une protection publique complete. Les tests adversariaux (Sprint 4) et le pilote externe (Sprint 5) reveleront les gaps.

### Dependance : politique versioning post-v1.0

Le tag v1.0 est pose localement mais pas pousse vers origin. Le push du tag fait partie du Sprint 5 (go-live), pas des prerequis. Mais la politique post-v1.0 (chaque break bumpe la version) s'applique au Public Feed des maintenant. Sprint 1 Phase A doit explicitement definir le versioning du feed sous ce regime.

### Dependance : S61 audit S60 termine

Ce plan ne commence pas avant que l'audit S60 (Sprint 61 Phase 0) soit cloture.

---

## 4. Ce que le plan ne couvre PAS

- Interop externe large (clients alternatifs, parsers tiers)
- Audit tiers formel (seulement prep + RFP)
- p2panda-discovery/auth/encryption (espaces prives, post-v1.0)
- SearchManifest/RRV complet (optionnel Sprint 6, sinon post-plan)
- Tor transport (arti, post-plan)
- Runtime isolation VM (post-plan)
- AppStorage P2P multi-app (dehardcode sbfb-ideas, post-plan)

---

## 5. Lien avec documents existants

- Recherche p2panda : `.planning/research/p2panda_public_protocol_briques.md`
- Recherche RRV : `.planning/research/chat_ia_reseau_recherche_reseau_rnd.md`
- Publish model : `docs/architecture/PUBLISH_MODEL.md`
- Self-hosted build : `docs/architecture/SELF_HOSTED_BUILD.md`
- Threat model : `docs/security/THREAT_MODEL.md`
- External audit scope : `docs/security/EXTERNAL_AUDIT_SCOPE.md`
- Hardening roadmap : `docs/security/HARDENING_ROADMAP.md`
- Release gates : `docs/security/RELEASE_GATES.md`
- Carry list S61 : `.planning/active/` (sprint courant)

---

## 6. Decision PO

5 sprints = meilleur plan compresse, avec risque assume et gate de scission.
6 sprints = plan public credible sans compression.
Pour SBFB "protocole open source verifiable publiquement", recommandation officielle : **5+1 reserve, pas "5 garanti"**.

La decouverte "anti-spam feed = phase, pas sprint" est partiellement vraie : les LOC de glue sont faibles, mais la preuve produit/protocole (tests adversariaux, pilote, exposition publique) demande plus qu'un branchement.
