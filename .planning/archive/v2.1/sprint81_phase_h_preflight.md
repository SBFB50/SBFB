# Preflight S81 Phase H — Migration LIVE ancre VPS

## Verdict: PLAN-ADAPT

Le chemin de données est **TRANCHÉ = in-place auto-migration** (prouvé Phase F sur COPIE
du store VPS réel, `70dd845`) et ce chemin ne contredit AUCUNE Day-0 gelée (iroh strictement
seul, pins exacts, `node_id` préservé). Aucun DESIGN-CONFLICT. Mais la Phase H n'est PAS
`EXECUTE` tel qu'écrite : le RUNBOOK planifié porte cinq divergences opérationnelles à corriger
AVANT le flip — dont une **réfutation** (le rollback documenté est incomplet, R2 REFUTED) et une
étape que le plan/kickoff sous-classent (le tar per-nœud, seul filet universel de la crash-window,
dégradé à « recommandé à bas coût » kickoff:58-59 alors qu'il est load-bearing). Ce sont des
adaptations de runbook (delta tests H = 0, 0 code neuf), pas une re-conception. D'où PLAN-ADAPT.

## Rationale du verdict

Signaux des 5 scans : S1a PLAN-ADAPT, S1b EXECUTE, S2 EXECUTE, S3 PLAN-ADAPT, S4 PLAN-ADAPT.
Vérifs adversariales : R1 UNCERTAIN, **R2 REFUTED**, R3 UNCERTAIN, R4 CONFIRMED, R5 CONFIRMED.

La substance technique est **sûre** (R4 CONFIRMED : migration in-place redb 2→4 crash-safe via
garde + tar, idempotente, prouvée sur COPIE du store VPS réel ; R5 CONFIRMED : gate de convergence
observable/mesurable, exercé live sous le lock 1.0.1 exact). Ce qui reste PLAN-ADAPT n'est pas le
« quoi » mais le « comment opérationnel écrit » :

1. **Rollback incomplet (R2 REFUTED, P1)** — `STORE_MIGRATION_OPS.md:29-30` = « restaurer le tar OU
   renommer le backup », sans **re-déploiement du binaire 0.98**. La migration docs.redb est
   AUTOMATIQUE à l'ouverture et one-way : restaurer le tar puis rebooter sous 1.0.1 **re-migre** et
   rejoue le flip raté. Rollback correct = 2 gestes (restore tar + redeploy 0.98).
2. **Portée snapshot sous-spécifiée (P1)** — plan H `sprint81_plan.md:336` énumère « `docs.redb` +
   `blobs/` » et laisse tomber `node_key` (l'identité !), `coordinator.db*` et
   `.sbfb/directory_revision.json`. Lu littéralement, ce périmètre **casse le `node_id`**.
   (`STORE_MIGRATION_OPS.md:22-23` règle 1 est correct — c'est le plan H qui diverge.)
3. **Tar dégradé (P1)** — `kickoff:58-59` classe le tar « recommandé à bas coût » alors que
   `plan:335` le MANDATE. Seul filet universel de la crash-window + seul rollback one-way → NON-SKIPPABLE.
4. **Sémantique convergence (P2)** — « après CHAQUE nœud » (`plan:339`) inatteignable au 1er flip
   (partitionné). À scinder LOCAL par-nœud + CROSS-nœud dès le 2e nœud 1.0.
5. **Rationale VPS-dernier (P3)** — « fenêtre bornée » ne vient PAS de l'ordre (toute paire 0.98/1.0
   totalement partitionnée quel que soit l'ordre) mais de SAME-DAY + C4/C5-aucun-tiers.

Aucun point n'exige de code — ils enrichissent le runbook, livrable central de H.

## S1a OSS prior-art (findings + evidence fichier:ligne)

RUNBOOK CONFORME à l'état de l'art : flotte de 3 nœuds auto-possédés sans tiers + wire
non-rétrocompat = **FLAG-DAY coordonné**, pas rolling mixte. Fondations déjà PROUVÉES.

- **H-S1a-01 (info)** — Identité structurellement sûre : `<root>/node_key` 32 octets
  (`runtime.rs:129-151`) → `SecretKey::from_bytes` (`node.rs:391-393`) → `NodeId` Ed25519
  déterministe ; `node_key` HORS redb. `node.rs:550-567` vert sous 1.0.1.
- **H-S1a-02 (P2)** — Résidu : plan/F n'assertent que le byte-identique du FICHIER, PAS la dérivation
  cross-version. `node.rs:551` = INTRA-1.0.1. Fermer : charger `node_key` 0.98 (fixture F) sous 1.0.1,
  asserter `endpoint.id()` == `node_id` 0.98.
- **H-S1a-03 (P2)** — `load_or_generate_node_key` régénère **warn-only** si len≠32 (`runtime.rs:139`) :
  tar tronqué → nouveau `node_id` + simple warn. Capturer `node_id` avant snapshot, asserter au boot.
- **H-S1a-04 (P2)** — « convergence après CHAQUE nœud » inatteignable au 1er flip. Scinder (a) LOCALE
  + (b) cross-nœud dès le 2e nœud 1.0.
- **H-S1a-05 (P3)** — Rationale VPS-dernier sur-vendu ; vrais bénéfices = downtime seeder minimal +
  partenaires dev+Mac déjà en 1.0 + caveat Linux `rename(2)` éprouvé en dernier.
- **H-S1a-06 (P2)** — Portée snapshot : unité déclare DEUX roots (`NEXUS_GRID_ROOT` +
  `SBFB_HOME=.sbfb`, `service:36-49`) ; `directory_revision.json` sous `.sbfb`. Expliciter DEUX roots
  + checklist survivants.
- **H-S1a-07 (info)** — Blobs 0.100→0.103 no-op + redb 2→4 atomique CONFIRMÉS par F sur store VPS réel
  (copie) : 0 wipe, 0 perte M18.
- **H-S1a-08 (P3)** — Gel publish/ingest = **discipline opérateur** (aucun gate code) ; le runbook doit
  le DIRE, pas suggérer un mécanisme inexistant.

## S1b Deps/CVE

**Rien ne bloque le flip (0 P0/P1).**

- **S1b-1 (info)** — Pins iroh EXACTS effectifs manifest+lock, reproductibles `--locked`
  (`Cargo.toml:48-51`, `release-attest.sh:76`).
- **S1b-2/3 (info)** — Advisories carriés inatteignables sur VPS Linux défaut : quick-xml = macOS plist
  (dead x86_64-linux) ; hickory-0.24 gated DNS-fallback `enabled: false` (`dns_fallback.rs:127`).
- **S1b-4 (P2)** — `ed25519-dalek 3.0.0-rc.0` = duplicate interne iroh, PAS un CVE (`bans=warn`,
  carry S82). **NE PAS rouvrir le gate Phase G ni flip `deny.toml [bans]`.**
- **S1b-5 (P2)** — Ambiguïté build-chain GHA ubuntu-latest vs Docker `rust:1.94`. **Nommer le binaire
  du VPS** + sha256 + intoto dans le RUNBOOK.
- **S1b-6 (info)** — Yank-watch RC : CI rouge (`yanked=deny`) mais build `--locked` intact. Ajouter
  `cargo deny check` pré-flip.

## S2 Décisions historiques (cohérence Day-0 / C1..C10)

**Le runbook ne contredit AUCUNE décision gelée (S2 = EXECUTE).**

- **S2-1** — In-place cohérent : C4/C5 assoupli ÉLARGIT la latitude sans imposer le wipe ; l'in-place
  RÉSISTE (F : 0 wipe, 0 perte M18) → wipe inutile, in-place strictement plus sûr.
- **S2-2** — `node_id` intrinsèque à l'in-place ; re-install stock INTERDIT (D3 cond.5/R1).
- **S2-3** — C5 self-heal-neutralisé DÉJÀ satisfait par code committé (A2 fail-loud ×2 + garde F
  `refuse_recreate_on_interrupted_migration`, `runtime.rs:2574`). **Ne pas re-planifier en H.**
- **S2-4** — Gate R2 SATISFAIT (F=PASS `70dd845`) ; gate 25/08 MOOT ; 15/09 garde ~2 mois runway.
- **S2-5 (P3)** — Wipe éventuel SCOPÉ docs/blobs, JAMAIS `node_key`/`anchors`/`coordinator.db`.
- **S2-6 (P3)** — Re-décision Topologie A-vs-B (avant 25/08) orthogonale ; **ne pas la figer en H**.
- **Snapshot Mac RÉSOLU** : PRIS 2026-07-08 (`STORE_MIGRATION_OPS.md:24-27`). Caveat : vérif Mac
  n'a pas couvert `.sbfb/directory_revision.json`.

## S3 Threat model (menaces couvertes / gaps P0-P3)

**0 P0/P1 sécurité. Gap = COMPLÉTUDE DOC (P2).**

- **S3-F1 (P2)** — Aucune entrée pour l'OPÉRATION de flip : §15.4 scopée « E2/E3/F » (mécanisme).
  Produire **§15.5** flip LIVE (résiduels LOW sous C4/C5).
- **S3-F2 (P2)** — Tar VPS PAS encore pris (Win+Mac PRIS). Caveat Linux `rename`-clobber → tar (pas
  rename) = rollback VPS.
- **S3-F3 (P3)** — Partition totale : couverte + LOW (seuls NOS 3 nœuds, 0 perte données).
- **S3-F4 (info)** — Crash mid-migration + rollback=tar : COUVERT (garde + 2 tests + §15.4).
- **S3-F5 (info)** — Duress / plan B : COUVERT (§15.4 E3, IROH_SELFHOST_OPS, gates C8).
- **S3-F6 (P3)** — Régression `node_id` : gate en place, recouvrable sous C4/C5 ; plier au §15.5.
- **S3-F7 (info)** — Gel = discipline anti-split-brain, pas frontière de sécurité.

## S4 Wire format / store on-disk (invariants préservés)

**Tous les invariants de persistance structurellement préservés (S4 = PLAN-ADAPT).**

- **S4-1 (P1)** — Tar per-nœud = SEUL rollback one-way + seul filet crash-window (recreate silencieux
  non-attrapé par A2 sans backup). À élever load-bearing.
- **S4-2 (P2)** — « VPS DERNIER » = ordre risk-minimizing RECOMMANDÉ ; vrais invariants de correction =
  same-day + gel + `node_key` jamais régénéré + gate F-PASS + tar per-nœud.
- **S4-3 (info)** — `node_id` préservé par construction (`node_key` HORS redb/blobs).
- **S4-4 (info)** — `anchors.json` + floor révision + `directory_revision.json` HORS store → survivent.
- **S4-5 (info)** — `BlobTicket` byte-stable 0.100↔0.103 ; constantes wire SBFB
  (`FEED_FORMAT_VERSION=1`, tous `*_FORMAT_VERSION=1`) UNTOUCHED (F git-diff = 0).
- **S4-6 (P2)** — R4 « partition totale » = propriété UPSTREAM non vérifiable depuis SBFB ; à confirmer
  empiriquement au flip + logger dans l'artefact T2.

## Chemin de données TRANCHÉ (in-place vs wipe+re-pull) + justification

**TRANCHÉ = IN-PLACE auto-migration** (redb 2→4 automatique à l'ouverture + blobs 0.103 no-op).

1. **Prouvé sur store VPS RÉEL** (Phase F `70dd845`, `store_migration.rs:313-526`) : blobs no-op même
   dirty, docs migre + backup sibling + 0 orphan, second open idempotent, DEUX namespaces M8 survivent,
   `node_key` byte-identique, tickets `anchors.json` re-parsent.
2. **Crash-safe** : temp-write + rename + persist ; chaque fenêtre soit self-healing, soit
   fail-loud-gardée, soit succès ; `rename(2)` atomique ext4/xfs (VPS Ubuntu 24.04).
3. **Strictement supérieur au wipe** sous C4/C5 (préserve pins M18) ; fallback wipe inutile.
4. **Day-0 tenues** : iroh seul, pins exacts, `node_id` préservé.

Condition load-bearing : le garde self-heal `runtime.rs:2574` reste **ACTIF** (non supprimé) au flip.
« Neutralisé » (C5) ≠ supprimé.

## RUNBOOK opérationnel de-risqué

Livrable Phase H (delta tests = 0, 0 code neuf). Same-day, UNE session.

**Phase 0 — Préparation** : (1) nommer binaire VPS + sha256 + intoto ; ne PAS utiliser `deploy/deploy.sh`
(cible/service divergents) → `scp` `/usr/local/bin/nexus-shell-daemon` + `systemctl restart`. (2) conserver
binaire 0.98 côte-à-côte. (3) `cargo deny check` sur le commit. (4) capturer `node_id` de référence par
nœud. (5) capturer baseline `{archive_hash, sha256(index.html)}` par app.

**Phase 1 — Snapshot tar (NON-SKIPPABLE, 3 nœuds)** : (6) daemon ARRÊTÉ (WAL déchiré sinon,
`db.rs:363`). (7) tar root COMPLET (pas « docs.redb + blobs/ ») + checklist survivants : `node_key`,
`coordinator.db`(+wal/shm), `blobs/`, `docs.redb`, `anchors.json`, `subscriptions.json`,
`.sbfb/directory_revision.json` ; traiter DEUX roots. (8) vérifier restaurabilité (extract jetable,
`node_key` 32 octets + `directory_revision.json`).

**Phase 2 — Flip par nœud (dev Win → Mac → VPS DERNIER)** : (9) gel publish/ingest = discipline.
(10) par nœud : deploy 1.0.1 → restart (service inchangé `start --headless`) → 1er boot : **0 crash-loop**
+ `docs.redb` migré (backup sibling présent) + **`node_id` INCHANGÉ** (asserter == référence, sinon
STOP + restore tar + redeploy 0.98) + feed/ides/pins M18 intacts. (11) convergence DEUX niveaux :
(a) SANTÉ LOCALE (browse local + blob-serve sha256 == baseline) ; (b) CROSS-nœud dès le 2e nœud 1.0
(dev↔Mac browse `status=reachable` + blob-serve sha256 byte-identique = couple E3), re-vérif après VPS ;
scripter en harness committé (contrat JSON b3) ; NE PAS invoquer b3 (= compute, carry WAN S77 orthogonal).
(12) rationale ordre sans sur-vendre (downtime seeder minimal + partenaires 1.0 vérifiés + caveat Linux
en dernier ; bornage = same-day + C4/C5).

**Phase 3 — Post-flip** : (13) re-annonce + re-pull boot (floor re-appliqué). (14) re-armer self-heal =
supprimer `docs.redb.backup-redb-v2-tuples` après convergence saine. (15) écrire §15.5 threat-model.
(16) time-box + déclencheur rollback si check (a)/(b) échoue AVANT de toucher le VPS.

## Rollback (procédure restore tar exacte)

DEUX gestes (restore-tar seul re-migre — R2 REFUTED) : (1) `systemctl stop`. (2) restaurer le tar du
root complet. (3) **RE-DÉPLOYER binaire 0.98** (Phase 0 #2). (4) restart → vérif 0 crash-loop +
`node_id` == référence + apps servies. (5) **VPS = TAR, PAS rename du backup** (caveat Linux
`rename`-clobber). (6) crash avant rename → aucun `.backup` → seul le tar récupère.

## Prérequis GO / NO-GO (checklist bloquante avant le flip)

Voir le champ `go_no_go`. GO uniquement si tout vert ; NO-GO/STOP si divergence `node_id`, crash-loop,
échec convergence dev↔Mac (2e flip), `node_key` absent/tronqué, ou migration interrompue sans backup ni tar.

## Risques résiduels + mitigations (table des 5 vérifications adversariales)

| # | Risque | Verdict | Résidu load-bearing | Mitigation runbook |
|---|--------|---------|---------------------|--------------------|
| R1 | Fenêtre bornée + gel + VPS-dernier suffisent contre perte irréversible | **UNCERTAIN** | Attribution causale erronée ; vrai triplet = tar + in-place F-PASS + node_key | Tar NON-SKIPPABLE ; bornage = same-day+C4/C5 ; gel = discipline ; scinder convergence ; time-box + assert node_id |
| R2 | Rollback restore-tar correct et complet | **REFUTED** | Binaire 0.98 non re-déployé ; snapshot plan:336 omet node_key/coordinator.db/.sbfb ; tar à chaud WAL | Rollback 2 gestes ; tar root COMPLET + checklist ; daemon ARRÊTÉ ; vérif restaurabilité |
| R3 | node_id/node_key VPS préservés | **UNCERTAIN** | node_key préservé code-confirmé ; dérivation cross-version non prouvable offline ; régen warn-only len≠32 | Assert empirique node_id ; vérif len 32 ; interdire SBFB_IDENTITY_SECRET_HEX ; pas deploy.sh |
| R4 | In-place redb 2→4 SAFE, self-heal ne détruit jamais silencieusement | **CONFIRMED** | « Atomique » = crash-safe via garde+tar ; sûreté conditionnelle à la discipline tar | Tar per-nœud load-bearing ; tar=rollback VPS ; re-armer self-heal ; assert node_id ; copy-dry-run Mac/Win |
| R5 | Gate convergence « après CHAQUE nœud » observable/mesurable | **CONFIRMED** | Primitives exercées live (E3/E2 PASS 1.0.1) ; « chaque nœud » inatteignable au 1er flip ; check non committé | Harness browse+sha256 NOMMÉ ; scinder LOCAL/CROSS ; ne pas confondre avec b3 compute |

Aucun résidu ne rend l'in-place non-sûr ni ne heurte une Day-0 — obligations de runbook (delta tests H=0).

## Ce qui est operator-gated

L'agent produit/de-risque le RUNBOOK mais NE PEUT PAS exécuter le flip. Strictement opérateur :
**SSH VPS Hetzner** (scp + systemctl restart) ; **accès physique aux 3 machines** (tar daemon-arrêté +
1ers boots) ; **fenêtre same-day UNE session** (gel, ordre, time-box) ; **décisions PO orthogonales**
(re-décision Topologie A-vs-B avant 25/08, activation plan B gate 15/09, escalade S75 boot-SEED audit
gate S81) ; **choix chaîne de build du binaire VPS** + attestation.

L'agent PEUT préparer en amont (0 code, delta tests 0) : harness convergence browse+sha256 committé,
§15.5 threat-model, correction `STORE_MIGRATION_OPS.md` (rollback 2 gestes + portée snapshot),
enrichissement plan H — tous docs/acceptance, pas du code runtime.