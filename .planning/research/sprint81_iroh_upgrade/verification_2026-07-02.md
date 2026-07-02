# Vérification ultracode du staging S81 — 2026-07-02

> Statut : note de vérification, hors sprint. Produit par le Workflow `wf_8ef303fb-526`
> (7 agents, ~672k tokens, 104 outils : 4 ancrages [staging intégral, surface iroh repo,
> état iroh web au 02/07, process/runway] + audit phase-par-phase + attaque adversariale
> + synthèse). Le staging du 2026-06-27 N'EST PAS réécrit par cette note : elle est le
> REGISTRE D'AMENDEMENTS à appliquer à l'activation (prérequis #6 : re-timestamper au jour J).

## Verdict

**VALIDE AVEC AMENDEMENTS** — squelette structurellement solide (scope transport-only,
bisectabilité, migration sur copie, T1/T2 conformes README §4), mais **périmé sur 4 points
et troué sur 3** après 5 jours. Le plan passe de 10 phases (0+A→I) à **12 (0+A+A2+A3+B→I)**.
Ne pas re-concevoir. Le premier risque de crever le 30/09 n'est pas la technique iroh,
c'est **le pattern d'insertion d'arcs off-sprint** dans la file d'attente.

## Hypothèses du 27/06 re-vérifiées au 02/07

| Hypothèse | Statut | Réalité au 02/07 |
|---|---|---|
| « 1.0.0 seule stable, re-pin sur 1re 1.0.x » (C3) | **PÉRIMÉE** | iroh **1.0.1** publiée le 2026-06-29 → pinner directement `=1.0.1` |
| EOL relais N0 2026-09-30 | CONFIRMÉE | 0.9x ET 1.0.0-rcX → 30/09 ; 0.35x → 31/12. Aucun report. Le sunset ne nomme QUE les relais (survie dns.iroh.link pré-1.0 non garantie → veille) |
| Quatuor 1.0 + docs 0.101.0 + gossip 0.101.0 + blobs 0.103.0 | CONFIRMÉE | Publiés le même jour (2026-06-15), deps croisées vérifiées crates.io — **le bloqueur skew GuardianDB est LEVÉ upstream** |
| Pins Cargo inchangés malgré l'arbre sale | CONFIRMÉE | 89 fichiers sales 100 % orthogonaux (sbfb-factory + front) ; `git status crates/nexus-*` VIDE |
| Self-heal = runtime.rs:2515-2528 | **PARTIELLE** | **DEUX sites, pas un** : `boot_storage_namespace` :2456-2549 (recreate :2518, seul couvert) ET le miroir `boot_feed_namespace` :2555-2633 (recreate :2606) — même chemin destructeur |
| Bug materializer wf4 (Phase A) | CONFIRMÉE | Confirmé au code, 0 fix intermédiaire (~41 commits tous Factory/front/i18n) |
| « Feature défaut redb-v2-migration » (C4) | **PÉRIMÉE** | **Ce flag N'EXISTE PAS** — migration AUTOMATIQUE à l'ouverture (iroh-docs PR #105) ; saut réel redb ^2.6.3→^4.1. La fixture Phase F VALIDE l'auto-migration, elle n'active rien |
| MSRV « plancher 1.91 à vérifier, 1.95 interdit sans preuve » (C6) | **PÉRIMÉE (tranchée)** | `rust_version=1.91` confirmé crates.io pour les 5 crates → toolchain 1.94 SUFFIT, image CI inchangée. Décision résiduelle : bump rust-version DÉCLARÉE Cargo.toml:24 1.85→1.91 |
| URLs pkarr/relais peuvent changer (R3) | CONFIRMÉE (fait) | Les relais publics ont basculé vers la flotte « v1.0 stable » dans la release 1.0.0 → check nommé survie URL obligatoire |
| ed25519-dalek -rc dans l'arbre iroh (C7) | PARTIELLE | Ni confirmé ni infirmé au 02/07 → le gate cargo tree -d flip-or-carry reste le bon dispositif |
| Rig T2 dispo → RIG-ABSENT illégitime | **PARTIELLE** | VPS+Mac joignables SSH MAIS **Mac sans Ollama** + blocker WAN task-delivery S77 jamais fermé + **aucun b3 PASS complet n'a jamais existé** → palier quorum conditionnel (C10) |
| « T1 convergence bloquant CI chaque push » | **PARTIELLE** | Ni Woodpecker ni GHA ne posent `SBFB_INTEGRATION=1` → les 5 tests multi_daemon relay-gated **early-return verts EN SILENCE depuis toujours** — baseline relais jamais mesurée en CI |
| iroh 1.0 sans audit tiers | CONFIRMÉE | R-iroh-audit P0 inchangé, upgrade ≠ Gate 1/3, pilote reste fermé |
| Rename Node→Endpoint absorbé, travail = docs+redb | PARTIELLE | Vrai, MAIS 0.98→1.0 supprime encore : `Connection::to_info()`→`weak_handle()`, `PathWatcher/PathInfo`→`paths()/PathList+PathEvent #[non_exhaustive]`, `Incoming::local_ip`→`local_addr`, ClientBuilder `query_param`→`auth_token` (liste canonique rc.0 → re-ancrer préflight Phase E) |
| Wire-freeze 1.0 réduit le churn | PARTIELLE | Vrai pour iroh core SEULEMENT — **iroh-docs (toute la convergence SBFB) reste pré-1.0**, wire re-cassé 2× en 6 semaines → pin exact `=0.101.0` + trigger de veille |
| sbfb-ides vs sbfb-ideas (incohérence interne staging) | PARTIELLE | Jamais tranchée — à résoudre au code au préflight Phase F |

## Plan amendé (12 phases)

- **0** — Audit gate S80 + activation sur tip propre (baseline figée POST arc committé + S80 I/J + dual-platform + push ; C1..C7 re-confirmés jour J).
- **A** — Fix convergence materializer wf4, 0-bump AVANT bump (fold après verify_chain + tri topo prev_hash + tie-break + garde monotone ; verify_entry vérifie prev_hash).
- **A2 (NOUVELLE)** — Self-heal root-cause **×2**, 0-bump : `boot_storage_namespace` ET le miroir `boot_feed_namespace` (Err→fail-fast diagnostiquable, Ok(None) seul recrée).
- **A3 (NOUVELLE)** — Baseline transport LIVE 0.98 : artefact JSON b3 par palier committé + run Win `SBFB_INTEGRATION=1` archivé + Ollama sur le Mac (ou arbitrage C10) + copie store VPS rapatriée.
- **B** — Bump `=1.0.1` + compagnons exacts (+ iroh-tickets/iroh-metrics si relogement) + fix CaTlsConfig + MSRV-confirmation (1.91 tranché).
- **C** — iroh-docs deep (types iroh-base + wire EntrySignature→Signature + DocTicket stabilité string + zombies legacy supprimés).
- **D** — iroh-blobs 0.103 cascade + redb4 blobs + ExtraProtocolFactory.
- **E** — Surfaces fragiles re-cert (liste canonique rc.0 : weak_handle/paths/PathEvent...) + **PLAN B PRÉ-PROVISIONNÉ** (relais self-hosted wire-compat 0.98 + pkarr self-hosted + acceptance zéro-n0). Split E' si le portage shard dépasse le mécanique.
- **F** — Migration on-disk redb 2→4 sur COPIE (préflight = lire le CODE upstream #105 : atomicité, crash mid-migration ; garde self-heal ×2 ; trancher sbfb-ides/ideas).
- **G** — CI/MSRV déclarée 1.91/convergence crypto (cargo tree -d flip-or-carry) + docs sécurité + trigger veille iroh-docs 0.102+.
- **H** — Migration LIVE : tar snapshot sur les **3 nœuds**, **fenêtre d'incompatibilité BORNÉE** (flip same-day en UNE session + gel publish/ingest + convergence vérifiée après CHAQUE nœud — les flottes relais 0.98/1.0 diffèrent : partition possiblement totale pendant la fenêtre), re-annonce post-flip.
- **I** — Wrap-up + T1 libellé corrigé (hermétique-CI vs relay-gated-local ; job `SBFB_INTEGRATION=1` nightly/manuel OU couverture T2-live actée) + T2 par paliers selon C10 + amendement roadmap (arbitrage slot S82 rendu BLOQUANT).

## Décisions : C1/C2/C7 confirmées · C3 amendée (=1.0.1) · C4 amendée (auto-migration #105, pas de feature) · C5 étendue (miroir feed) · C6 tranchée (1.91) · **+3 NOUVELLES à ratifier PO** :

- **C8** — Plan B relais/discovery PRÉ-PROVISIONNÉ obligatoire (Phase E, 2-4 j) + 3 gates calendaires ÉCRITS dans le kickoff : **01/08** (corps S81 pas ouvert → provisionner immédiatement), **25/08** (Phase F pas PASS → basculer la flotte sur le plan B), **15/09** (Phase H pas faite → plan B ACTIF, 2 semaines de vérification zéro-n0).
- **C9** — Arbitrage slot S82 SUR-RÉSERVÉ (4 prétendants : fondation Viewer, sharding live ex-S78, workflow-engine 12 phases, dette docs-contract) — rendu BLOQUANT en Phase I.
- **C10** — Sort du palier quorum T2 : fixer le blocker WAN task-delivery en 0-bump avant bump (extension bornée) OU re-scoper le quorum hors T2 transport avec statut hérité BLOCK{WAN-delivery-carry-S77} tracé.

## Runway (honnête)

90 jours au 02/07. File d'attente : arc 89 fichiers (~2-3 j) + S80 I/J (~2-4 j) + audit gate (~1 j) → ouverture réaliste **10-12/07** ; S81 12 phases ≈ 2-3 semaines → DONE fin juillet/mi-août = **marge ~2× le chemin critique**. Conditions : ne RIEN insérer avant S81 (S82 workflow-engine reste DERRIÈRE), pas de nouvel arc off-sprint après S80, veille hebdo blog/status n0 dès maintenant. Le point dur = **Phase H (VPS migré LIVE) avant le 30/09**, pas le merge.

## Prérequis d'activation (checklist 8 points)

1. Arbre 100 % propre (arc 89 fichiers committé + review/Codex groupés rejoués). 2. S80 clos (I/J). 3. Dual-platform vert + tip POUSSÉ. 4. Audit gate S80 PASS. 5. C1..C10 confirmés PO. 6. Re-check crates.io jour J (1.0.2 ? audit tiers ?). 7. Réconciliation sprint80_kickoff/audit_plan (S81=iroh, Viewer→S82+). 8. SEULEMENT ENSUITE `git mv` vers active/ (avant = casse la détection A/B/C/D du bootstrap).

## Registre des mises à jour à appliquer aux fichiers stagés (à l'activation)

kickoff+plan : =1.0.1 + veille 1.0.2 ; supprimer « feature redb-v2-migration » partout → « auto-migration #105 » ; ajouter le miroir self-heal :2606 partout où :2515 est cité ; insérer A2+A3 ; Phase E re-ancrée liste rc.0 ; Phase B deps iroh-tickets/metrics + irpc 0.14→0.17 ; Phase G rust-version 1.85→1.91 + trigger veille docs ; Phase H fenêtre bornée + tar ×3 ; Phase I libellé T1 corrigé + paliers T2 C10 ; README checklist 8 coches + 3 gates calendaires ; trancher sbfb-ides/ideas ; aligner MEMORY (repère :2617 → :2606/:2518 réels).
