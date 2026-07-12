# Sprint 81 — Design Review Board (G1)

> **STATUT : ACTIVÉ le 2026-07-02 — les deux conditions du board sont jouées.**
> (1) Phase 0 = audit gate S80 : **CONDITIONAL PASS → PASS effectif** (findings
> `dcc3eea`, fix P1 `2c85b28`) ; (2) arbitrages C1..C7 **confirmés par le PO** à
> l'activation, avec **UNE décision CONTRE la reco évaluée ici : C1 — le sharding
> est INCLUS au T2** (bi-axe, Phases I/J ajoutées ; le scoring D4 ci-dessous, qui
> évaluait la reco « sharding hors T2 », est SUPERSEDÉ sur ce point par le bloc
> « Décisions PO à l'activation » du kickoff, qui fait autorité) ; C4/C5 assoupli
> (« il n'y a personne sur le réseau ») ; C2/C3/C6/C7 = recos confirmées.
> **Le verdict board flippe donc CONDITIONAL → PASS** (avec la réserve D4 notée) —
> le scoring ci-dessous est conservé tel quel comme artefact historique du board
> (évaluation indépendante du 2026-06-27, non réécrite a posteriori).

> **Méthode (ultracode).** Kickoff S81 orchestré en Workflow : agents recherche D1..D8 →
> G1 design review board (perspective indépendante, scoring 0-5 par décision) + lentille
> adversariale (sceptique technique). **Pas de rubber-stamp** : les claims du sceptique
> ont été **re-vérifiés au code ce jour (2026-06-27)** et les claims réfutés **retirés** ;
> les contradictions inter-cartes sont tranchées. Le finding le plus dangereux du sceptique
> **tient** : le self-heal `runtime.rs:2515-2528` n'est PAS un backstop (cf. §Lentille
> adversariale + D3). **Sprint** : S81 — **Upgrade iroh 0.98 → 1.0 + sweep deps** (insertion
> roadmap non-planifiée, forcing-function relais N0 sunset 2026-09-30).

## Verdict board : CONDITIONAL (DRAFT — conditions intégrées, à confirmer à l'ouverture)

**3/8 PASS** (D5/D7/D8) + **5 CONDITIONAL** (D1/D2/D3/D4/D6) + **0 CONFLICT**. Aucune
décision ne contredit une Day-0 gelée (l'upgrade 1.0 est anticipé par « évaluer upgrade
1.0 ») ni la pre-launch policy (wire SBFB `JCS`/`DOMAIN_*_V1`/`FEED_FORMAT_VERSION` NON
bumpés). Les conditions G1 sont **intégrées comme tranchées dans le dossier**, mais — S80
non clos — le lot **reste CONDITIONAL** : il ne flippera à PASS qu'après (1) Phase 0 = audit
gate S80 PASS, (2) confirmation PO des arbitrages C1..C7. Aucun obstacle de conception
n'empêche d'ouvrir S81 sur cette base ; le risque est **opérationnel** (migration live
one-way), pas décisionnel.

## Scoring par décision

| Décision | Score | Verdict | Note |
|---|:--:|---|---|
| **D1** Version cible (GA `.0` vs patch `1.0.x`) | 4/5 | CONDITIONAL | Reco synthétisée (`=1.0.0` + re-pin sur 1re `1.0.x` avant push live) résout la reco wf3 « viser la dernière 1.0.x » **insatisfiable** (0 patch publiée à date) ; runway ~3 mois tranche `wait`-vs-`proceed`. Quatuor confirmé `Cargo.toml:37-41`. |
| **D2** Relais / discovery post-EOL N0 | 4/5 | CONDITIONAL | `presets::N0` déjà câblé (`node.rs:318`) + escape-hatch `RelayMode::Custom` (`node.rs:329,348`). Condition **BLOQUANTE** : survie de l'URL pkarr `pkarr_resolver.rs:54` (`dns.iroh.link/pkarr`) — sinon discovery casse **silencieusement**. |
| **D3** Migration données on-disk (redb 2→4, stores LIVE) | 4/5 | CONDITIONAL | **RISQUE PRINCIPAL.** Hybride durci ; **self-heal `runtime.rs:2515-2528` n'est PAS un backstop** (vérifié : `create_doc()` id NEUF + écrase M8 **sans `import_ticket`** = perte silencieuse). 7 conditions cumulatives. |
| **D4** Stratégie test / convergence (T1 hermétique + T2 LIVE) | 4/5 | CONDITIONAL | Gate testabilité (README §4) respecté. **RIG-ABSENT illégitime sur l'axe transport** (rig confirmé dispo) ; axe sharding (`shard.rs` RTT/PathId, rig GPU chroniquement absent) **hors T2 → S82**. |
| **D5** Scope du fix materializer (wf4) | 4/5 | PASS | Conflit inter-cartes arbitré **in-sprint, Phase A, AVANT le bump** : baseline 0.98 verte = bisectabilité (`feed_materializer.rs:54-58,95-101` + `public_feed.rs:588` confirmés au code). Commit propre distinct du bump. |
| **D6** MSRV + sweep deps feuilles | 3/5 | CONDITIONAL | Contradiction MSRV tranchée **empiriquement** (`cargo +1.94 build` Docker) ; **P2-AUDIT-2 NON fermé** par l'upgrade (RC échangé contre RC) → gate `cargo tree -d` ou carry **P2-AUDIT-2-RESIDUEL**. |
| **D7** Carries + roadmap (séquencement) | 4/5 | PASS | iroh **STRICTEMENT SEUL** (bisectable) ; blast-radius corrigé = **3 crates déclarent iroh** (re-scan call-sites côté daemon) ; clôture P2-AUDIT-2 **gatée par D6** ; amender roadmap_v5. |
| **D8** R-iroh-audit / posture release | 5/5 | PASS | **R-iroh-audit P0 INCHANGÉ** ; upgrade **≠ Gate 1/Gate 3**, ne débloque PAS le pilote public. Wire-freeze 1.0 = churn désérialisation réduit (neutre-à-positif), jamais durcissement de confiance. |

## Détail par décision (perspective indépendante)

- **D1 — Version cible (4/5 CONDITIONAL).** La reco wf3 « viser la dernière `1.0.x` » est
  **factuellement insatisfiable** au 2026-06-27 : `1.0.0` (12 j) est la **seule** stable, **aucune
  `1.0.x` patch n'existe**. La reco est **réécrite** : coder sur `iroh = "=1.0.0"` maintenant
  (pin exact) + **re-pin OBLIGATOIRE** sur la 1re `1.0.x` publiée AVANT le push live ; si aucune
  patch au code-freeze → soak documenté + veille RustSEC sur `1.0.0` (interdiction de pousser la
  `.0` brute si une patch existe). La tension `wait`(carte 5) vs `proceed`(carte 3) est tranchée
  par le runway ~3 mois (sunset relais N0 0.9x **2026-09-30**). Quatuor confirmé au code,
  point unique `Cargo.toml:37-41` → `iroh 1.0.0` / `iroh-docs 0.101.0` / `iroh-gossip 0.101.0` /
  `iroh-blobs 0.103.0`. NON contradictoire avec Day-0 (« évaluer upgrade 1.0 » anticipe). Résidu :
  risque `.0`-fraîche réel → pin exact + re-pin conditionnel.
- **D2 — Relais / discovery post-EOL N0 (4/5 CONDITIONAL).** Reco (c) `presets::N0` par défaut
  (mis à jour <24 h après release par n0) + relais iroh self-hosted **optionnel** pour l'ancre VPS
  comme résilience = correcte et déjà outillée : `node.rs:318` `Endpoint::builder(presets::N0)` +
  escape-hatch `SBFB_CUSTOM_RELAYS`/`relays.json` → `RelayMode::Custom` (`node.rs:329,348`,
  `relay_config.rs:17-20`). L'upgrade **résout** le forcing-function (ligne 1.0 supportée until EOL
  vs 0.9x coupée 2026-09-30). **Condition BLOQUANTE** : vérifier explicitement la survie de l'URL
  pkarr `pkarr_resolver.rs:54` (`dns.iroh.link/pkarr`) sous 1.0 — le blog n0 avertit « wire-breaking
  relay changes get new URLs » → sinon discovery casse **silencieusement** (pas de crash), ancre
  injoignable. Check nommé, jamais plié dans « recompile ».
- **D3 — Migration données on-disk redb 2→4 (4/5 CONDITIONAL).** RISQUE PRINCIPAL du sprint, traité
  avec rigueur. Reco hybride durcie (c) : `docs.redb` = migration **IN-PLACE impérative** (saut
  0.98→0.101 **DIRECT**, jamais 0.99/0.100 contre l'ancien store) ; coordinator SQLite (pins
  `keep_online` M18, `public_feed`) **INDÉPENDANT** donc intact ; blobs content-addressed
  re-dérivables mais **testés en staging**, pas un pari. **Point dur corrigé par le sceptique,
  vérifié au code** : le self-heal `runtime.rs:2515-2528` n'est PAS un backstop — la branche `None`
  appelle `create_doc()` (namespace id **NEUF**) + `set_storage_namespace` qui écrase la ligne M8
  **SANS `import_ticket`** de l'ancien ticket → orpheline les entries sbfb-ides répliquées + casse
  les `DocTicket` persistés, en `warn`-only silencieux. **7 conditions cumulatives** : (1) saut direct ;
  (2) wipe docs **INTERDIT** (toléré blobs uniquement, filet re-pull) ; (3) tar snapshot de
  `NEXUS_GRID_ROOT` AVANT 1er boot 0.101 (migration **ONE-WAY**, rollback = restore tar) ; (4)
  validation sur **COPIE** du store VPS peuplé AVANT flip ; (5) ancre VPS in-place gardant
  `node_key`/`node_id` (re-install stock S75 **INTERDIT** : régénérerait l'identité → casse les
  locators abonnés) ; (6) fixture redb 2→4 dans T1 ; **(7, ajout sceptique) self-heal `runtime.rs:2515`
  neutralisé/gardé pendant la migration**. À tracer : la pre-launch policy « wire modifiable
  librement » ne couvre **PAS** ce store on-disk déjà déployé.
- **D4 — Stratégie test / convergence (4/5 CONDITIONAL).** Énoncé conforme au gate de testabilité
  **NON-NÉGOCIABLE** (README §4). T1 = convergence **hermétique in-process** (`multi_daemon`
  loopback/`MemoryLookup`) BLOQUANT sur **Win natif + CI Linux**, **JAMAIS Docker-on-Windows**
  (`multi_daemon` env-bloqué, `create_node` hang). T2 = acceptance JSON committé
  (`PASS`/`BLOCK{diagnosis}`/`RIG-ABSENT`) sur rig **réel** VPS Hetzner + dev Win + Mac M2. **Verdict
  splitté (correction sceptique)** : l'axe **transport** existe et PEUT atteindre PASS (S75
  survives-VPS-death = LIVE PASS, LAN Win↔Mac validé) → **`RIG-ABSENT` illégitime** sur cet axe (seul
  un rig génuinement HS le justifie) ; l'axe **sharding** (`shard.rs` RTT/PathId multipath noq +
  orchestrateur in-vivo, rig GPU 5080+M2 **chroniquement absent** S76 DIFFERE / S77 RIG-ABSENT /
  orchestrateur reporté S78) est **hors T2 de S81** → reporté S82. On ne re-joue PAS une acceptance
  (S77 b3_shard) qui n'a **jamais** passé. S81 prouve doc-sync/gossip/blobs/seed/annuaire + recompile
  shard **handshake** (pas le RTT live).
- **D5 — Scope fix materializer wf4 (4/5 PASS).** CONFLIT inter-cartes arbitré : carte 1 D8 →
  « sprint séparé postérieur » ; cartes 3/4 → IN S81, **Phase A, AVANT le bump**. Le board **ADOPTE
  cartes 3/4**. Bug confirmé au code : `feed_materializer.rs:54-58` écrase `latest_release_hash` sans
  garde monotone ; `materialize_full` `:95-101` folde sans `verify_chain` ; `verify_entry`
  (`public_feed.rs:588`) sans check `prev_hash`. Convergence-critique et **0-bump** (logique
  coordinator SQLite, **indépendante d'iroh**) ; le gate de convergence cross-machine de S81 le
  **révèle de toute façon** ; le corriger AVANT le bump établit une **baseline 0.98 verte** → préserve
  la **bisectabilité** (un échec post-bump = iroh, pas le materializer) — plus fort que le sprint
  séparé de carte 1. Conforme Day-0 « root cause, no band-aid ». **Discipline imposée** : commit propre
  dédié ; fold APRÈS `verify_chain` + tri topo `prev_hash` + tie-break `(timestamp,author,hash)` +
  garde monotone `apply()` + `verify_entry` check `prev_hash`. Refuser tout mélange avec le commit de
  migration.
- **D6 — MSRV + sweep deps feuilles (3/5 CONDITIONAL).** CONTRADICTION inter-cartes tranchée
  **empiriquement** : carte 4 réclame un bump `1.94→1.95` **inconditionnel** ; cartes 1/3 démontrent
  que le plancher réel = `1.91` (iroh-docs 0.101) **DÉJÀ franchi** par le Docker canonique `rust:1.94`
  (`rust-version` déclarée `1.85`, `Cargo.toml:24`). Le board tranche pour cartes 1/3 : **NE PAS
  bumper 1.95 sans preuve cargo** qu'une feuille l'exige (`cargo +1.94 build` Docker AVANT budget).
  **P2-AUDIT-2 n'est PAS pleinement résolu par l'upgrade** : iroh 1.0 épingle encore ed25519-dalek sur
  un `-rc` (RC échangé contre RC), tandis que le lock 0.98 actuel tire déjà un fouillis **dupliqué**
  (`3.0.0-pre.6` + `3.0.0-rc.4`, sha2 `0.11.0-rc`, der/pkcs8/spki `0.8.0-rc`). Crypto SBFB critique sur
  `ed25519-dalek 2.x` stable (`Cargo.toml:58`), **isolée et inchangée**. Gate de convergence =
  `cargo tree -d` post-bump montre **un seul** arbre ed25519-dalek + **0** `*-pre`/`*-rc` dupliqués →
  **SI converge**, flipper `deny.toml:107` `multiple-versions warn→deny` ; **SINON** lever
  **P2-AUDIT-2-RESIDUEL** et **NE PAS** annoncer P2-AUDIT-2 CLOSED. Vérifier aussi que le `2.x` SBFB ne
  s'effondre PAS sur l'arbre RC d'iroh.
- **D7 — Carries + roadmap, séquencement (4/5 PASS).** Routage cohérent avec la discipline scope-cuts
  stricts. **iroh STRICTEMENT SEUL** (bisectable, directive séquencement #7) ; GuardianDB/dette
  **séparés et postérieurs**. Reroutés S82 : Viewer fondation (`tools/factory-ui` jeté S80) + Aperçu
  scellé/Proof Card ; 8 P2/11 P3 docs-contract → **sprint dette NOMMÉ distinct** ; 2 P1 in-vivo
  (sharding RIG-ABSENT S77, app-authoring S79) restent **standing** ; l'orchestrateur sharding DOIT
  séquencer **APRÈS S81** (`shard.rs` re-vérifié sous 1.0 en Phase E — sinon travail jeté). **Correction
  factuelle adoptée** : blast-radius = **3 crates déclarent iroh** — `nexus-core-rs/Cargo.toml:19-22`
  (les 4), `nexus-shell-daemon/Cargo.toml:78,84` (iroh-blobs + iroh, **PAS wrapper-only** —
  `seed_protocol` impl `ProtocolHandler`), `nexus-shell-daemon-core/Cargo.toml:179,186` (dev-deps) ;
  bump VERSION reste un point unique mais **call-sites API à re-scanner côté daemon**. Condition :
  amender explicitement `roadmap_v5` (insertion S81-iroh non-planifiée + Viewer→S82) ; clôture
  P2-AUDIT-2 **GATÉE par D6**.
- **D8 — R-iroh-audit / posture release (5/5 PASS).** Décision sans ambiguïté, pleinement conforme aux
  Day-0 gelées (« pilote fermé 2-3 personnes, R-iroh-audit P0 → pas public »). Toutes les cartes
  convergent : iroh 1.0 n'a reçu **AUCUN audit tiers public** → **R-iroh-audit P0 INCHANGÉ**, l'upgrade
  **NE franchit PAS Gate 1/Gate 3** et **NE débloque PAS** le pilote public ; c'est une maintenance
  **forcing-function-driven** (continuité transport avant cutoff 2026-09-30), pas une levée de zone
  rouge. Le **wire-freeze 1.0** réduit le churn de la surface de désérialisation (THREAT_MODEL menace E)
  = **neutre-à-positif**, jamais un durcissement de confiance. Condition (documentaire, non bloquante) :
  libellé explicite « upgrade = patch-train sécurité + wire-freeze, **PAS** levée R-iroh-audit » dans
  kickoff + commit body ; amender `THREAT_MODEL.md:22,128,195` + `EXTERNAL_AUDIT_SCOPE.md §2.4/§2.7`
  (versions 0.97/0.99→1.0.0, note R-iroh-audit reconfirmée **verbatim**) + `HARDENING_ROADMAP.md:5`
  (trigger iroh **FIRED**, bump `last_validated`) ; rejouer la checklist `cargo tree` §2.7 avant tout
  envoi RFP vendor.

## Lentille adversariale — constats et résolution

> Le sceptique a contesté 4 claims load-bearing. **3 réfutations tiennent et sont intégrées** ;
> 1 claim « SAUVE » est réécrit en « UNVERIFIED-high-risk ». Tous re-vérifiés au code ce jour.

- **[Technique] Self-heal présenté comme « filet » (CONFIRMED_ISSUE/CRITIQUE — claim carte 1 D2
  RÉFUTÉ).** Carte 1 recommandait « garder le self-heal en filet ». **DANGEREUX et REJETÉ** :
  `runtime.rs:2515-2528` branche `None` → `create_doc()` (id NEUF) + écrase M8 **sans `import_ticket`**
  → perte silencieuse, pas un backstop. → **Résolu (D3 condition 7)** : self-heal **neutralisé/gardé**
  pendant la fenêtre de migration (un échec de migration doit **crasher diagnostiquablement**, pas
  perdre en silence) ; conservé pour le seul cas légitime (DB importée d'un autre data-dir).
- **[Technique] « Viser la dernière 1.0.x » insatisfiable (CONFIRMED_ISSUE/MAJOR — reco wf3
  RÉÉCRITE).** 0 patch `1.0.x` publiée au 2026-06-27. → **Résolu (D1)** : `=1.0.0` + re-pin
  conditionnel + soak/veille RustSEC documentés.
- **[Technique] P2-AUDIT-2 NON fermé par l'upgrade (CONFIRMED_ISSUE/MAJOR — claim « upgrade ferme
  P2-AUDIT-2 » RÉFUTÉ).** iroh 1.0 épingle encore ed25519-dalek sur un `-rc`. → **Résolu (D6/C7)** :
  ne PAS pré-annoncer CLOSED ; gate `cargo tree -d` → flip `deny.toml` OU carry **P2-AUDIT-2-RESIDUEL**.
- **[Technique] `shard.rs` « SAUVE » (NEEDS_REWRITE/MAJOR — claim adouci).** Le sceptique réfute
  « SAUVE » : API 1.0 post-cutoff, non-hermétique, rig absent. → **Réécrit en UNVERIFIED-high-risk
  (R5)** : re-cert **compile + handshake seulement** en S81 (Phase E, `shard.rs:60-63,171-181,299-327`) ;
  re-cert **LIVE multipath → S82** ; jamais claim « stable verbatim ».
- **[Technique] Bug materializer (CONFIRMED — vérifié au code).** `feed_materializer.rs:54-58`
  écrasement inconditionnel, `:95-101` fold non vérifié, `public_feed.rs:588` pas de `prev_hash`. →
  **Résolu (D5)** : Phase A in-sprint avant le bump, 0-bump, commit propre.
- **[Scope] Axe sharding mélangé au T2 (NEEDS_PO/MAJOR).** Le DONE non-PROVISIONAL en 1 sprint est à
  risque sur l'axe shard, exactement comme S77. → **Résolu (C1/C4/D4)** : scoper le DONE sur l'axe
  **TRANSPORT-convergence** uniquement ; axe sharding **explicitement hors T2** → S82.
- **[Scope] MSRV 1.95 inconditionnel (CONTRADICTION inter-cartes).** Carte 4 vs cartes 1/3
  (plancher 1.91 déjà franchi). → **Résolu (C6/D6)** : empirique, 1.94 sauf preuve feuille.
- **[Scope] Blast-radius sous-estimé (CORRECTION factuelle).** Pas seulement `nexus-core-rs` : **3
  crates** déclarent iroh (call-sites daemon `ProtocolHandler` seed). → **Intégré (D7)** : re-scan
  call-sites sur les 3 crates ; bump VERSION reste point unique.
- **[Scope] Faux signal de durcissement (NO_ISSUE/MINOR).** Risque de présenter l'upgrade comme
  amélioration sécurité → relâche garde pilote. → **Résolu (D8/R9)** : libellé « upgrade ≠ Gate 1,
  R-iroh-audit P0 inchangé, pilote reste ferme » obligatoire dans kickoff + commit body + docs.
- **[Scope] Tentation de bundler (NO_ISSUE/MINOR).** Viewer/dette/GuardianDB/materializer-dans-le-bump
  → bisectabilité détruite. → **Résolu (D7/R10)** : iroh STRICTEMENT SEUL ; materializer Phase A commit
  séparé AVANT bump ; tout le reste rerouté.

## Arbitrages PO load-bearing (C1..C7 — recos à confirmer AVANT le 1er Edit)

> S80 non clos → ces arbitrages ne sont **pas** tranchés ; ils sont intégrés comme **recommandations**.
> Le PO les confirme à l'ouverture réelle de S81 (après Phase 0 = audit gate S80). L'ordre de
> structuration suit le dossier (C1 = le plus structurant).

1. **C1 — Scope « non-PROVISIONAL DONE » réaliste (le plus structurant).** *Reco : scoper le DONE sur
   l'axe TRANSPORT-convergence uniquement* (doc-sync/gossip/blobs/seed/annuaire) ; **sortir l'axe
   SHARDING du T2** (re-cert live shard → S82). Sans ce cadrage, S81 finit PROVISIONAL sur l'axe shard
   comme S77.
2. **C2 — Scope du fix materializer.** *Reco : Phase A in-sprint (cartes 3/4), AVANT le bump, commit
   propre séparé* (vs carte 1 « sprint séparé postérieur »). Baseline 0.98 verte = bisectabilité.
   **Jamais dans le commit de bump.**
3. **C3 — Version cible.** *Reco : `iroh = "=1.0.0"` maintenant + re-pin OBLIGATOIRE sur la 1re `1.0.x`
   AVANT le push live* ; si aucune patch au code-freeze → soak + veille RustSEC documentés.
4. **C4 — Stratégie données-live.** *Reco : migration IN-PLACE impérative `docs.redb` (saut direct),
   validée sur COPIE du store VPS peuplé AVANT flip ; self-heal neutralisé ; blobs in-place avec test
   staging + filet wipe re-fetchables ; tar snapshot avant 1er boot 0.101 ; ancre VPS migrée EN DERNIER,
   gardant `node_key`/`node_id` (re-install stock S75 INTERDIT).*
5. **C5 — Le self-heal n'est PAS un backstop (correction critique du sceptique, vérifiée au code).**
   *Reco : garder le self-heal pour le cas légitime (DB importée d'un autre data-dir) mais le
   DÉSACTIVER/garder pendant la fenêtre de migration redb* (sinon un échec déclenche une perte
   silencieuse au lieu d'un crash diagnostiquable).
6. **C6 — MSRV (contradiction inter-cartes).** *Reco : vérifier empiriquement (`cargo +1.94 build`
   Docker) AVANT de budgéter ; NE PAS bumper 1.95 sans preuve cargo qu'une feuille l'exige.*
7. **C7 — Clôture P2-AUDIT-2.** *Reco : ne PAS pré-annoncer CLOSED ; gate `cargo tree -d` → si convergent
   (un seul arbre ed25519-dalek + 0 `*-pre`/`*-rc`), flipper `deny.toml:107` ; sinon carry
   P2-AUDIT-2-RESIDUEL.* Vérifier aussi que le `ed25519-dalek 2.x` SBFB ne s'effondre PAS sur l'arbre RC.

## Réconciliation main-thread (vérification indépendante des faits load-bearing)

Avant de figer ce DRAFT, les faits porteurs ont été re-vérifiés au code (2026-06-27) :
- **Pin iroh actuel** : `Cargo.toml:37-41` = iroh/iroh-docs/iroh-gossip `0.98`, iroh-blobs `0.100` ;
  `Cargo.toml:24` rust-version `1.85` ; `Cargo.toml:58` ed25519-dalek `2.1`. CONFIRMÉ → bump en point
  unique workspace.
- **Surface de déclaration iroh = 3 crates** : `nexus-core-rs/Cargo.toml:19-22` (les 4) +
  `nexus-shell-daemon/Cargo.toml:78,84` (iroh-blobs + iroh, **PAS wrapper-only** — `seed_protocol`
  impl `ProtocolHandler`) + `nexus-shell-daemon-core/Cargo.toml:179,186` (dev-deps). CONFIRMÉ →
  re-scan call-sites côté daemon obligatoire (D7).
- **Self-heal destructeur** : `runtime.rs:2515-2528` branche `None` → `create_doc()` (id NEUF) +
  `set_storage_namespace` écrase M8 **sans `import_ticket`**, en `warn`-only. CONFIRMÉ → ce n'est PAS
  un backstop (D3/C5).
- **Bug materializer** : `feed_materializer.rs:54-58` (écrasement inconditionnel `latest_release_hash`),
  `:95-101` (fold non précédé de `verify_chain`), `public_feed.rs:588-591` (pas de check `prev_hash`).
  CONFIRMÉ → fix wf4 en Phase A avant le bump (D5).

## Statut

**CONDITIONAL (DRAFT de staging)** — le lot de décisions est solide (3 PASS forts + 5 CONDITIONAL aux
conditions intégrées, 0 CONFLICT) et **aucune décision ne contredit une Day-0 gelée ni la pre-launch
policy**. Le flip vers **PASS effectif** est conditionné à : (1) **Phase 0 = audit gate S80 PASS**
(jouée à l'ouverture réelle), (2) **confirmation PO des arbitrages C1..C7**, (3) figement de la liste
exacte des carries entrants et de la baseline de tests à la clôture S80. Les conditions G1 BLOQUANTES
(survie URL pkarr D2 ; 7 conditions migration D3 ; T2 LIVE transport D4 ; materializer Phase A avant
bump D5 ; gate `cargo tree -d` D6 ; iroh-seul + roadmap D7 ; libellé posture D8) sont **non
négociables** et reportées dans le plan de phases. **NE PAS ouvrir le corps S81 (A→I) tant que S80
n'est pas clos et Phase 0 PASS.**