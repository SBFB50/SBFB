# Sprint 76 — Audit plan (audit gate de S75, joue en S76 Phase 0)

**Ecrit** : 2026-06-11 (Phase G Sprint 75).
**Sprint audite** : **Sprint 75** (decouverte PULL node-centrique + ancre VPS —
FIX-A re-mint replay, `NodeDirectoryEntry`/`DOMAIN_NODE_DIRECTORY_V1`, durabilite
locator, pull multi-provider, ancre VPS headless, front node-Browse, carries S74).
**Executeur** : session fraiche S76, Phase 0 (Cas A audit gate).
**Produit attendu** : `.planning/active/sprint75_audit_findings.md`
(verdict PASS / CONDITIONAL PASS / FAIL).
**Tip audite** : commit Phase G `feat(daemon)` wrap-up (HEAD au demarrage S76 ;
tip code phases A-F = `4f52bea`).

---

## §0 Mode d'emploi pour la session fraiche S76

**Ordre de lecture impose** (forme une opinion AVANT de lire les self-reports) :

1. Ce fichier (`sprint76_audit_plan.md`) — la feuille de route.
2. Le **diff complet** S75 : `git diff 0e2fb6b..<tip Phase G>` (audit gate S74 →
   tip). Les 7 commits feat : Phase 0 audit `0e2fb6b`, A `479a87c`,
   B `f6637d3`, C `821aa8c`, D `0010450`, E `1486fc9`, F `4f52bea`, G `<tip>`
   (+ chores intercales : kickoff 4 docs `f008433`, preflight A `e3c3fb6`,
   handoffs `9f7de7f`/`41b13e3`/`491b3c8`/`035a4f7`).
3. `sprint75_kickoff.md` §4 (5 verrous anti-recentralisation + test cardinal),
   §5 (D1-D5 gelees), §8 (carries S74), §9 (12 scope cuts).
4. Le code livre, dans l'ordre des tracks ci-dessous.

**A NE PAS lire avant d'avoir forme une opinion** :
`sprint75_verification.md` (self-report — l'agent livreur a ecrit le code ET la
verification ; valeur de confirmation nulle pour un audit independant) et les
`sprint75_phase_*_review.md` (reviews du livreur). Les lire **apres** pour
comparer, pas pour se faire une opinion.

**Format du livrable** : `sprint75_audit_findings.md` (§7 ci-dessous).

**Contexte non-standard a connaitre** :
- **S75 a amende la roadmap v5** (kickoff §12, decision PO) : la decouverte PULL
  est passee AVANT le GPU (bug live : apps invisibles aux pairs frais, PoW > 30
  min rejete au replay verbatim). GPU → S76, sharding → S77. Ne pas compter ce
  re-sequencement comme un drift — il est documente et arbitre.
- **3 preflights PLAN-ADAPT (C, D, F)** — pas des derives agent, des corrections
  factuelles du plan contre le code reel : C « persister node_ids » irrealisable
  (iroh exige le content-hash) → locator `anchors.json {pubkey,ticket,revision}` ;
  D « re-mint ticket » irrealisable cote consommateur (`mint_ticket_for_hash`
  bail sur blob absent) → download bare-hash multi-provider ; F le variant wire
  `reachableviaseeder` du handoff n'a JAMAIS existe → badge Q7 front-compose
  depuis la paire `unreachable + peer_count>0`, `/browse` byte-identique.
  L'audit verifie que chaque PLAN-ADAPT porte une evidence ground-truth, pas
  qu'il aurait fallu suivre le plan d'origine.
- **D3 a touche une Day-0 anterieure** (DEFER SearchManifest D3/s73) : resolu
  par `sprint75_pivot_proposal.md` + sign-off PO AVANT Phase A (R7 kickoff).
  Le juge a clarifie : `NodeDirectoryEntry` est un objet distinct, le DEFER
  SearchManifest TIENT. Verifier la trace, ne pas rebattre.
- **Environnement** : tests iroh-networked sensibles au reseau hote
  (`create_node` hang 90s = env, remede reboot machine, JAMAIS `wsl --shutdown`).
  JAMAIS 2 cargo paralleles, JAMAIS cargo pendant un round Codex. Le compte
  canonique = Docker Linux sbfb-ci ; Docker local est le gate AVANT PUSH
  (`feedback_wsl_before_push`), pas avant commit. JAMAIS git/codex depuis
  `web/` (`.git` imbrique perime).
- **Phase G env-sensible** : l'acceptance survives-VPS-death + C6 E2E exigent
  le cross-machine live (SSH mac 192.168.1.53 + vps 135.181.42.188, systemd).
  Phase G a execute la validation live (unit systemd validee, systemd-analyze
  security 1.7) — l'audit verifie la TRACE consignee, et ne re-exige le re-run
  live que si la trace est absente ou incoherente.

---

## §1 Critere verdict audit S75

| Verdict | Condition |
|---------|-----------|
| **PASS** | 0 P0, 0 P1, >= 1 P2+ documente |
| **CONDITIONAL PASS** | 0 P0, 0 P1 mais conditions a surveiller S76 |
| **FAIL** | >= 1 P0 ou >= 1 P1 non resoluble dans l'audit |

Rigor signal G4 : PASS exige >= 1 P2+ documente. 0 P0/P1 **et** 0 P2+ = CONCERN
(audit trop superficiel). S75 expose au minimum ~12 P2+ candidats (routes depuis
les 6 phase reviews, ci-dessous) — un audit qui n'en confirme aucun est suspect.

---

## §2 Tracks audit S75 (ce que Phase 0 S76 doit verifier)

> **Note de provenance** : les items routes ci-dessous proviennent du parse §4.4
> des **6 phase reviews A-F (ratio present : 6/6, toutes `## Verdict: PASS`)** +
> Codex (18 rounds cumules A-F : 1+4+7+1+2+3) + preflight G. Ce sont des
> **questions a verifier**, pas des findings confirmes. Chaque item porte une
> severite « si faux » — l'audit tranche.

### Track A — Suites verification

Rejouer la fail-fast `sprint75_verification.md §Fail-fast` (24 rows, plan §5).
Attendu :

- **Windows natif** : nextest `--workspace` **1755**, 0 fail 0 skip.
  Progression par phase depuis S74 sortie 1674 : A 1682 (+8) → B 1714 (+32)
  → C 1724 (+10) → D 1735 (+11) → E 1748 (+13) → F 1750 (+2) → G 1755 (+5 :
  `search_clamps_offset_and_query`, `guardrail_trip_sets_rejected_terminal`,
  `guardrail_trip_on_quorum_path_sets_rejected_terminal`,
  `deploy_strips_existing_provenance`, `fork_entry_count_capped`). Verifier la
  decomposition au `nextest list`.
- **Docker Linux canonique** (row 6) : **1759**, 0 fail 0 skip (run final seul,
  sans charge cargo parallele ; les runs sous contention avaient 16 timeouts
  `operator_server` + 1 flake `sigint` = non-fidelite bind-mount 9p, PAS des
  regressions). L'ecart +4 vs Win = tests `#[cfg(unix)]` structurels.
- **Web** : Vitest **367** (331 → 334 Phase C → 367 Phase F), coverage
  87.17/79.01/85.92/88.5 >= 85/85/78/85, size-limit 6/6, tsc 0, lint 0,
  `scan-en-strings.sh` clean.
- **Rows cles a re-executer soi-meme** (pas juste lire la colonne Observed) :
  7 (le bug live re-mint — les 3 tests REELS `replay_remints_own_ticket_to_current_address`
  + `replay_does_not_remint_a_third_party_address` + `replay_keeps_stale_ticket_when_blob_is_gone`
  dans `runtime.rs` ; le nom `stale_announcement_accepted_by_fresh_receiver`
  du plan §5 etait un placeholder kickoff jamais cree, divergence consignee
  verification.md row 7), 9
  (`node_directory_cross_domain_signature_rejected`), 10
  (`boot_repull_restores_remote_catalogs`), 11
  (`fetch_falls_back_to_seeder_when_anchor_offline`), 12
  (`boot_seed_driver_pins_configured_projects`), 13 (lock-3 tripwire grep),
  14 (0 bump wire), 15 (`release_published_searchable_by_name`).
- **0 delta dependances sur tout le sprint** : `git diff 0e2fb6b..<tip> --
  Cargo.lock` vide (verifie au preflight G S1b). Toute ligne non-vide = drift
  non documente (P2).

### Track B — Phase A : re-mint PoW + adresse au replay outbox (D2)

Question centrale : le replay outbox re-tamponne-t-il VRAIMENT PoW **et**
adresse, sans affaiblir la fenetre anti-replay ?

- Verifier `MAX_PROOF_AGE_SECS=1800` inchange (`pow.rs`) ; l'outbox stocke le
  payload **non-wrappe** ; les 3 sites replay + restore boot re-wrappent frais
  et re-mintent `EndpointAddr`/`BlobTicket` depuis `my_endpoint_addr()`.
- **P1 review fermes a re-confirmer** : le test hijack-guard reecrit en
  **2 noeuds reels** (l'ancien etait un faux-vert T1 mono-noeud) ; le test
  outbox migre. Verifier que les assertions sont load-bearing.
- **T6 (P2 route ici)** : le handler `GossipCmd::Outbox` broadcast n'est PAS
  teste en direct (2 noeuds requis). Confirmer que la couverture indirecte
  (replay E2E unit-simule + C6 live) suffit, ou inscrire le gap S76.
- **WS-3/PD-5 (P2 route ici)** : double parse normalize + `my_endpoint_addr()`
  appele **par entree** au replay (efficience — hoister once-per-pass).
  Verifier que la boucle replay sur N apps ne degrade pas le boot (pilote-borne
  acceptable, mais documenter).

### Track C — Phase B : `NodeDirectoryEntry` + domaine + authoring (D1)

Question centrale : le type signe sibling est-il crypto-isole des domaines
existants et l'ingest est-il vraiment gate ?

- `DOMAIN_NODE_DIRECTORY_V1` disjoint de tous les domaines (`canonical.rs`) ;
  `NODE_DIRECTORY_FORMAT_VERSION=1` NOUVEAU, aucun `*_FORMAT_VERSION` existant
  bumpe. Re-executer `node_directory_cross_domain_replay_rejected`.
- `verify_signed_list_ingest` (helper generique, mitigation drift R1/C1) :
  verifier que les DEUX familles (curator + directory) passent par le MEME gate
  subscription/cap/revision/cross-replay — grep tout arm d'ingest qui
  contournerait le helper.
- **Clotures P1 review B a re-confirmer** : revision home resolution ;
  troncature over-cap UTF-8-safe ; test signature cross-domain. Clotures GAPs
  Codex B (4 rounds) : `REVISION_LOCK`, `is_valid_archive_hash` au sign ET au
  verify, drop `ann.node_id==self` (spoof own node_id, 2 couches).
- Authoring `POST /api/daemon/directory/publish` : auth loopback, caps
  appliques a l'authoring (pas seulement a l'ingest), provenance = node keypair.

### Track D — Phase C : ingest annuaire + durabilite locator (D4, primitive 5)

Question centrale : un catalogue distant survit-il REELLEMENT au reboot, et le
floor anti-rollback tient-il a froid ?

- Locator `anchors.json {pubkey, ticket, revision}` re-valide signature +
  revision au re-fetch (honore D4 « fingerprint persiste, index re-fetche ») ;
  floor anti-rollback PERSISTE : RAM presente → dedup strict `>`, RAM vide →
  floor `>=P` (same-revision restaure apres re-pull echoue). Re-executer
  `boot_repull_restores_remote_catalogs` + verifier le floor reboot-durable
  (GAP Codex C R2/R4 — ne pas se contenter du test in-memory).
- `BrowseSource::NodeDirectory` 3e boucle aggregate : node_id = ancre dialable,
  archive_hash = AUTEUR, `repo_url` rempli par F. Subscription-gated : aucune
  entree d'une ancre non abonnee.
- **known_entry_count-overcount (P2 route ici, documente/accepte)** :
  double-compte best-effort assume — verifier qu'AUCUN chemin ne traite ce
  compteur comme verite de joignabilite (content-addressing BLAKE3 + sonde =
  autorite, invariant cardinal S74) et que THREAT_MODEL le couvre.
- **Re-pull-sequential (P2 advisory)** : re-pull boot sequentiel N x 15s/ancre
  — borne pilote OK, mais N ancres = boot lent. Advisory S76, verifier la doc.
- **GC-blob-reaper (P2 route scope cut #3)** : le blob annuaire re-pull n'est
  PAS pinne skip-GC — coherent avec le scope cut #3 (GC reaper post-launch).
  Verifier que c'est documente, pas oublie.
- **Carries S74 concus dedans** : WIRE-1 (`ReleasePublished` +
  `project_name`/`category` additif 0-bump, producteur deploy cable — re-executer
  row 15), WIRE-2 (SeedRegistry keye `(pid,hash)`, route `?archive_hash`
  Some=version-exacte / None=agnostic-STRICT, front passe `entry.archive_hash`),
  DBQ-1 (`set_keep_online` UPSERT COALESCE — None ne NULL plus le hash M18).
- **WIRE-3 + CARRY-3 (verification d'absorption — candidat P2 si tombes)** :
  le kickoff §8 listait 8 P2 « a concevoir dans le pivot » dont WIRE-3
  (croissance feed reprovide = propriete pre-launch) et CARRY-3 (sanitize
  aggregator byzantine). La row 24 du plan §5 ne les liste PAS dans les carries
  CLOSED. Verifier qu'ils sont traites (absorbes par les caps/ingest-gate B ou
  documentes THREAT_MODEL) **ou re-routes explicitement** — silencieusement
  tombes = P2.

### Track E — Phase D : pull multi-provider + SeedRegistry prod + /nodes (D5)

Question centrale : le fallback seeder fonctionne-t-il sans ticket, et le
registre prod est-il borne contre la monopolisation ?

- `fetch_hash_multi`/`fetch_and_pin_multi` : download bare-hash
  `Downloader.download(hash, Vec<EndpointId>)`, ordonne ancre-d'abord (Q5),
  pkarr resout, `MAX_FETCH_PROVIDERS=16` DANS la primitive (pas chez les
  appelants). Re-executer `fetch_falls_back_to_seeder_when_anchor_offline`.
- GAPs R5 S74 fermes : `blob_serve` 4e tier directory-only + `seed_voluntary`
  `SeedFetchPlan Ticket|Multi` avec garde `h==want_hash` — re-confirmer la garde.
- SeedRegistry prod : SEED-1 clamp in-registry, SEED-2 double cap
  eviction-stalest, **normalisation hex lowercase write+read** (sans elle,
  variantes de casse = monopolisation de slots — finding secu review D).
- Route `GET /api/daemon/nodes` : additive, enveloppe `{nodes}`, `/browse`
  **byte-identique** (PLAN-ADAPT D — verifier qu'aucun champ `/browse` n'a bouge).
- **PULL-3 (P2 route ici, D+E)** : cross-tier failover ABSENT — un ticket
  direct mort ne bascule pas sur directory/multi-provider ; idem call-site
  driver E (first-applicable-only). Confirme defere → **doit entrer au plan
  S76** (pas re-fixe en Phase 0).
- **SEED-1-Sybil (P2 route ici)** : pas de sampling anti-Sybil du seeder tail
  (lexicographic crowding du registre borne) — doc inline livree en D ;
  verifier qu'elle est honnete et router la conception S76.
- **Blob-serve-oracle (P3, route Phase G §15)** : un GET blob-serve declenche
  des dials sortants (oracle drive-by + amplification) — verifier que la row
  THREAT_MODEL §15 livree en G existe et qualifie le residu.
- **E2E-blob-R5a (P2, route Phase G acceptance C6)** : rendu complet via
  blob-serve apres pull multi-provider — verifier la trace C6 de Phase G.

### Track F — Phase E : ancre VPS headless + duress (D3)

Question centrale : le driver boot est-il duress-gate et l'ancre reste-t-elle
« MON serveur », jamais un defaut universel ?

- Config `[seed] keep_online_projects` dans daemon-CORE, **defaut VIDE**
  (verrou-3) + clamp lowercase-64hex. **Q3 : PAS de section `[directory]`** —
  l'ancre passe par `default_curators` (UN attention set). Verifier
  `config.rs` defauts vides + `deploy/config.toml.example` `= []`.
- Driver boot one-shot **duress-gate** (P1 review ferme : data root partage
  sous duress) ; resolution `direct > M18 > annuaire` FIGEE,
  first-applicable-only ; 120s/app ; predicat `seed_already_announced` ;
  `boot_driver_handle` abort+join au shutdown (GAP Codex E R1 ferme).
- Re-annonce PRODUCTEUR state-driven `revision>0` via coeur partage
  `build_sign_announce_directory` (carry P2 C ferme ; boot-only, late-join
  residuel documente).
- Route `POST /api/daemon/seed/request` = 1er appelant prod `request_seed`
  cote REQUESTER : **invite M19 TOUJOURS requise** (la claim self-designation
  etait FAUSSE et a ete corrigee — verifier le code, pas la doc), self-guard
  PARSE anti-base32, mint gate-detention 409, echo nonce.
- Unit systemd durcie : `SBFB_HOME`+`NEXUS_GRID_ROOT` epingles (P1 ferme :
  `User=` sans `$HOME` = crash-loop), `AF_NETLINK`, `@system-service` ;
  `sbfb_home` resolu once-at-boot (isolation tests).
- **Duress-gates-preexist (P2 route ici, lot dette duress E+F)** : les gates
  duress FRERES manquent sur `seed_voluntary`, `set_keep_online`,
  `reannounce_seeds_at_boot` (gap PRE-EXISTANT S74, exposition UX accrue par le
  front F). Confirme defere → **doit entrer au plan S76 comme lot** (pas un fix
  Phase 0). Candidat P1 UNIQUEMENT si l'audit trouve un chemin duress qui
  publie/annonce des donnees du vrai data root — sinon P2 lot.
- **Re-drive-ingest (P2 route ici)** : driver one-shot → fenetre morte au
  premier boot si l'annuaire est ingere APRES le passage du driver ; remede
  operateur documente (restart). Router conception re-drive-on-ingest S76.
- **same-key-exemption (P3 advisory)** : doc `seed.rs:111-116` a realigner si
  une exemption same-key est un jour voulue. Sans action, verifier juste que la
  doc n'affirme pas le contraire du code.
- **Systemd-validation (route Phase G acceptance)** : la validation live de
  l'unit sur le VPS reel (systemd-analyze security, QUIC sous seccomp) a ete
  faite en G (score 1.7 consigne) — verifier la trace.

### Track G — Phase F : front node-Browse + verrou-4 + WEB-1

Question centrale : le marquage provenance/derive est-il IMPOSSIBLE a obtenir
depuis une source non-editeur, et le toggle seed est-il fidele a l'etat daemon ?

- **Verrou-4 (lock-4)** : `VerificationDetail` par projectId + prop additive
  `expectedArtifactHash` (avertissement « autre version que celle affichee ») ;
  marqueur « Version derivee » rendu UNIQUEMENT depuis une annonce EDITEUR
  `source === "direct"` avec match exact `(pid, hash)`. **Piege structurel a
  re-verifier** : les boucles curator ET nodedirectory hardcodent
  `is_open_source:false` (`browse.rs:684/803`) — tout code front qui lirait ce
  champ depuis ces sources produirait un FAUX marquage. Row sans annonce
  editeur = AUCUN claim. Re-executer le test lock-4b reecrit (P1 review F
  ferme : fixture nodedirectory reordonnee + scenario decisif PID_ORPHAN
  aucun-badge — verifier que la fixture n'a pas regresse vers l'exclusion morte).
- **Badge Q7 front-compose** : derive de `unreachable + peer_count>0`, gate
  `!!archive_hash` ; AUCUN variant wire nouveau ; `/browse` byte-identique.
- **WEB-1 CLOSED** : cle loopback `self_pin_enabled` 3-etats — precedence
  echo > intent > defaut-ON ; deploy n'ecrit PAS de row M18 ; replay outbox ne
  filtre que les OFF explicites (self_seeding seul = FAUX OFF app fraiche) ;
  echos reset sur PAIRE `pid:hash` (GAP Codex R1, pattern « adjust state during
  render » — setState dans un effect = erreur lint). Verifier la reconciliation
  du toggle contre l'etat daemon reel.
- AddAnchorDialog = route subscribe EXISTANTE (Q3/DQ3), placeholder inerte
  (verrou-3 : aucune ancre pre-remplie) ; cold-start gate sur subscriptions
  CONNUES-vides (GAP Codex R2) ; `callDaemon` remonte `{"error"}` non-2xx ;
  Zod `/nodes` `.strict()` sur l'ENVELOPPE seule.
- **Discriminateur curator-vs-ancre (P2 route ici)** : les lignes « en
  attente » de `/nodes` ne distinguent pas un curateur d'une ancre —
  `listCurators().entries` le permettrait SANS changement wire. Router
  conception S76.
- **NITs F (advisory, sans action — verifier qu'ils sont documentes)** :
  semantique 404 (« no source for requested version » vs pid inconnu) ;
  asymetrie 400/404 arete hash-sans-ticket ; fidelite fixtures URL provenance ;
  `truncateHex` duplique x4 ; test addAnchor range dans `describe("listNodes")`.

### Track H — Phase G : carries hygiene + wrap-up + acceptance

Question centrale : les 4 fixes carries sont-ils reels (pas des tests qui
re-prouvent l'existant) et l'acceptance survives-VPS-death est-elle demontree ?

- **CARRY-5** : clamp `offset` + borne longueur `q` dans `search_handler`
  (`http.rs`) — le clamp `limit.min(100)` PRE-EXISTAIT (preflight G S2) : le fix
  et le test doivent cibler offset+q, pas re-prouver limit. Enveloppe reponse
  `{results,total,took_ms}` INCHANGEE (consommateur Zod `.strict()` Phase E S73).
- **CARRY-2** : trip guardrail → `Rejected` terminal via helper partage
  `reject_result_on_guardrail_trip` sur les DEUX chemins (HTTP +
  `validator_loop` gossip). L'ancien comportement (`validator_loop.rs:82-90`
  warn + return sans statut = zombie) doit etre mort. Verifier que le residu
  quorum `task_results` (texte rejete en table d'accumulation, carry S73 B.2)
  est documente.
- **PULL-1** : `strip_zip_member` AVANT inject provenance dans
  `finalize_deploy` — le chemin fork→redeploy (zip blob-reconstruit portant la
  `provenance.json` de l'auteur d'origine) ne produit plus de double membre.
  Invariant renforce : provenance = TOUJOURS l'auteur local re-signant (R5/verrou-4).
- **FORK-1** : `MAX_ARCHIVE_ENTRIES=4096` entry-cap dans `fork.rs::extract_zip`
  (les caps bytes existaient ; le vecteur flood-de-petites-entrees est ferme).
- **THREAT_MODEL §15.1** : rows S75 (route directory pull, blob-serve oracle
  — cf. Track E, `/nodes`, SEED-1/2, fresh-flood, driver boot + route requester
  E, exposition front F) + edits compagnons §7/§2/§4 (process §16 — le
  preflight G les annonce ; absents = P2 doc).
- **Acceptance survives-VPS-death** (test cardinal kickoff §4) : trace
  consignee horodatee — (a) aucune decouverte hard-cablee sur le node_id VPS
  (binaire `default_curators=[]`), (b) une autre ancre est premiere-classe,
  (c) apps joignables tant qu'un detenteur BLAKE3 repond. + **C6 E2E**
  (E2E-blob-R5a route ici) + validation systemd live (Track F).
- **LT-2 ARME** : dry-run Radicle PRIVE fait (trace G) ; le flip publie reste
  decision PO — verifier la trace, ne pas exiger le flip.
- **META-1** : regle PATTERNS GAP-carry ecrite ; PATTERNS §P59 (rust) + §P37
  (shell) presents et fideles au code.

### Track I — Wire 0-bump + pre-launch policy (transverse)

- Grep `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` : TOUS a 1, AUCUN bump sur
  tout le sprint (`CURATOR_LIST`, `KEY_ROTATION`, `POW`, `SEED`, `TASK`,
  `PIN_FILE`, `FEED`) ; `NODE_DIRECTORY_FORMAT_VERSION=1` est NOUVEAU (additif,
  son propre domaine — pattern S74 SeedRequest, conforme policy).
- Aucun decoder legacy/multi-version introduit ; les `#[serde(default)]`
  nouveaux portent un rationale runtime-tolerance (pas compat historique).
- `/browse` byte-identique apres D et F (le badge Q7 est front-compose) ;
  `/nodes` et `/seed/request` sont des routes ADDITIVES.
- Day-0 D1-D5 preservees ; le seul contact Day-0 (D3 vs DEFER SearchManifest)
  est passe par pivot_proposal + sign-off PO (cf. §0).

### Track J — 5 verrous anti-recentralisation (transverse)

Rejouer la checklist kickoff §4 contre le code final :
1. **Lock-1** : zero champ cible/hote au publish (grep front publish + API).
2. **Lock-2** : node-Browse additif, `known_browse_entries` honnete (sur-ensemble,
   jamais substitution de la grille curator).
3. **Lock-3 (TRIPWIRE DESIGN-CONFLICT)** : grep `135.181.42.188` + tout node_id
   hard-code dans `crates/` hors tests = ZERO ; `default_curators` ET
   `keep_online_projects` compiles VIDES (`config.rs`) ; AddAnchorDialog
   placeholder inerte. Un hit = **P0/P1 selon portee**.
4. **Lock-4** : provenance/signature toujours AUTEUR (cf. Track G) — le seeder
   n'est jamais rendu comme autorite.
5. **Lock-5** : suggestion declenchee par l'etat observe, jamais poussee au
   publish ; le pull d'une ancre est un choix explicite (confidentialite
   default-OFF : aucune requete user ne quitte la machine silencieusement).

### Track G1 presence (P1 bloquant si absent)

Verifier que `sprint75_design_review.md` existe dans `active/` (ou migre
`archive/` au S76 Phase 0) avec scoring G1 : **D1 OK, D2 OK, D3 reserve
(adjust : pivot_proposal + sign-off PO), D4 OK, D5 OK**. Present sur sprint
feature non-trivial = OK. Absent = **P1** (gate bypasse). Present sans scoring
= P2. Verifier aussi `sprint75_pivot_proposal.md` (la piece du sign-off D3).

### Track HARDENING drift (P2 informatif)

S75 ajoute des surfaces : nouveau domaine signe (`DOMAIN_NODE_DIRECTORY_V1`),
ingest annuaire gossip, route requester seed, route `/nodes`, driver headless
boot, unit systemd durcie. Comparer `HARDENING_ROADMAP.md §3` vs livre :
- THREAT_MODEL §15/§15.1 couvre les nouvelles surfaces (cf. Track H). **Correction
  S76 Phase B (B7, LOOPBACK-TIERS-STALE) : ce plan affirmait a tort que LOOPBACK
  trust tiers couvrait deja les routes additives — `/api/daemon/{nodes,seed,
  seed-count,keep-online,search}`, `/api/daemon/seed/request`, `/api/daemon/
  directory/publish` etaient ABSENTES de `LOOPBACK_ENDPOINTS_TRUST_TIERS.md §3`
  (drift cumule 2 sprints S74+S75). S76 B7 les a inscrites (toutes T0 ; T1
  candidats `/seed/request` + `/directory/publish` = actions signantes).**
- Pour chaque item prescrit non livre : scope-cut justifie kickoff §9 ou blocker
  documente, sinon P2 (drift). Informative, pas bloquante. Drift cumule 3+
  sprints sans justification → remonter le signal HARDENING_ROADMAP lui-meme.

### Track K — Meta-process

- **G8 7/7** : preflights presents avec verdicts reels A SCOPE-CUT-CONSISTENT,
  B EXECUTE, C PLAN-ADAPT, D PLAN-ADAPT, E SCOPE-CUT-CONSISTENT, F PLAN-ADAPT,
  G SCOPE-CUT-CONSISTENT (0 DESIGN-CONFLICT — le tripwire lock-3 n'a jamais
  declenche). Chaque PLAN-ADAPT porte une evidence ground-truth (cf. §0).
- **Reviews 6/6 + G** : `sprint75_phase_{a..f}_review.md` toutes
  `## Verdict: PASS` (format exact) ; review G au commit wrap-up. P1 fermes
  in-phase : A 2, B 3, E 2, F 1 (0 P1 en C/D) — verifier les clotures citees
  Tracks B/C/F/G.
- **Codex 6/6 + G, artefacts BRUTS** (sortie `codex exec -o`, jamais resumes) :
  A 1 round, B 4, C 7, D 1, E 2 (R1 28 CONFIRMED + 1 GAP), F 3 (R1 17C+1GAP →
  R2 20C+2GAP → R3 21 CONFIRMED PASS). Verifier que chaque PARTIEL/GAP est
  reconcilie dans la review et reporte au body. Lecon process a noter : Codex
  ne connait pas les frontieres de phase (les expliciter au prompt) et peut
  flagger sa propre sortie precedente (scoper au code).
- **Commit discipline** : 7 feat A-G bodies 9 sections `##` exactes + chores
  planning ; verifier les SHA (§0). Memory mise a jour apres chaque feat.
- **Absorbes en aval (PAS des items ouverts)** : re-announce-producer (C→E,
  CLOSED E) ; seed_voluntary-collision + NodeSummary-coupling (D→F, CLOSED F :
  archive_hash discriminateur + Zod `.strict()` enveloppe). Verifier les
  clotures, ne pas les re-router.

---

## §3 Carries re-routes vers S76 (a traiter ou re-router au kickoff S76)

| Carry | Source | Pourquoi differe | Piste connue |
|---|---|---|---|
| **Lot duress freres** | E+F (gap S74) | Pre-existant, hors-scope phase ; exposition accrue par le front F | Gates duress sur `seed_voluntary`, `set_keep_online`, `reannounce_seeds_at_boot` — lot dette coherent, pas 3 rustines |
| **PULL-3 cross-tier failover** | D+E | Conception failover (ticket mort → directory → multi-provider), pas un patch | Chaine de fallback ordonnee + call-site driver E |
| **Sampling anti-Sybil seeder tail** | D | Conception (le registre borne est crowdable lexicographiquement) | Sampling aleatoire du tail au-dela des N premiers |
| **Re-drive-on-ingest** | E | Driver one-shot = fenetre morte 1er boot ; remede operateur documente | Re-driver a l'ingest d'un annuaire couvrant un project_id configure |
| **Discriminateur curator-vs-ancre** | F | Lignes en-attente `/nodes` ambigues ; faisable sans wire | `listCurators().entries` distingue les deux familles |
| **T6 GossipCmd::Outbox direct** | A | 2 noeuds requis en test | Harness 2-noeuds (pattern hijack-guard A) |
| **WS-3/PD-5 hoisting** | A | Efficience, non-bloquant pilote | Hoister normalize + `my_endpoint_addr()` once-per-pass |
| **same-key seed.rs doc** | E | Advisory P3 | Realigner doc si exemption voulue un jour |
| **NITs F** (404, 400/404, fixtures, truncateHex x4, addAnchor grouping) | F | Sans action (advisory) | Opportuniste si une phase touche la zone |
| **SeedAnnounced ne converge pas cross-noeud** | G (constat acceptance live) | `peer_count:0` sur Windows ET Mac ~10 min apres le pin VPS (`.git/S75_PHASE_G_ACCEPTANCE.md`) ; hypothese : asymetrie d'abonnement feed (personne ne suit le feed du seeder). Best-effort par design MAIS un registre toujours-vide affaiblit le dial-set multi-provider (aggrave PULL-3) | Investiguer la propagation feed du `SeedAnnounced` (sync doc cross-swarm) ; lier a PULL-3 |
| **Annuaire du seeder n'annonce pas ce qu'il seede** | G (constat acceptance live) | `directory/publish` du VPS = `catalog_len:0` (own/direct-only) — un pair frais dont la seule ancre est le seeder ne peut pas DECOUVRIR l'app servie. Conforme verrou-4 (seeder != editeur), mais a arbitrer : section « seeded » distincte non-autoritaire dans `NodeDirectoryEntry` ? | Question design PO (pas un bug) — peser contre verrou-4 et le modele F-Droid |
| **Surface UX-ARRIVAL (B11, COUVERTE S76-B)** | post-S75 hotfix `e980d7e` | Registre observed RAM + split arrivee (`/browse` mes-sources vs « Decouvert ») + `/nodes` observed catalog-backed — surface inscrite tardivement, jamais routee a un track d'audit | **Couverte** : observed borne+rate-limite (tests cap/TTL/rate-limit/self-guard + `observed_capture_is_availability_only` S76-B), `from_subscribed` CATALOG-BACKED (`browse_views_derives_from_subscribed`), discriminateur curateur/ancre (B6, `Nodes.test.tsx`). THREAT_MODEL §15.1 rows observed + spoof « Tes sources ». Auditer comme TRACK COUVERT, pas comme gap |

**Statut S76 Phase B (dette reservee, `fix(daemon+shell)`)** : carries §3 traites
par la phase dette — **Lot duress freres** (B1, `seed_voluntary` + `set_keep_online`
duress no-op, THREAT_MODEL row L→Nil), **PULL-3** (B3, chaine cross-tier
`build_seed_fetch_chain` ticket→directory→multi), **CARRY-3** (B2, downgrade
`is_open_source` a l'ingress aggregator), **Discriminateur curator-vs-ancre** (B6),
**T6 GossipCmd::Outbox direct** (B4, test consommateur DB), **WS-3/PD-5 hoisting**
(B5, `my_endpoint_addr()` once-per-pass), **LOOPBACK-TIERS** (B7), **blob-serve
bearer** (B8 doc), **bridge allowlist parite** (B10), **UX-ARRIVAL** (B11).
**NON traites (reconduits S77, exemption nommee)** : Sampling anti-Sybil seeder
tail (dependance interne sharding), Re-drive-on-ingest, same-key seed.rs doc P3,
SeedAnnounced non-converge + annuaire-du-seeder (questions design PO, hors dette
mecanique). L'audit S76 verifie les CLOTURES citees, ne re-route pas les lignes
couvertes.

**Externes inchanges (a reporter tels quels)** : P2-A-1 rand (exemption
upstream), P2-AUDIT-2 iroh pre-release transitives (pin 0.98), T-NN+2 iframe
Rust-wasm (PATTERNS §P34), P3-OS-1 `operator_server` OR duplique, LT-3/LT-4/LT-7
hors-sprint. **LT-2** : trigger ARME + dry-run prive FAIT — verifier la trace
(Track H), le flip publie = decision PO. Verifier qu'aucun n'atteint 3 reports
sans exemption (escalade G7).

---

## §4 S76 Objective — GPU partage cross-machine (contexte, hors audit)

Apres l'audit S75, S76 ouvre (roadmap v5 amendee §12 kickoff S75) le **GPU
partage cross-machine** (ex-S75, decale par le pivot decouverte). La decouverte
PULL livree par S75 est le prerequis (on ne partage pas du GPU entre noeuds
qu'on ne decouvre pas). **Pre-requis a inscrire au plan S76** (carries §3) :
le lot duress freres (exposition front F), PULL-3 si le dispatch GPU s'appuie
sur le fetch multi-provider, et le sampling anti-Sybil si la selection de
workers lit le SeedRegistry.

---

## §5 Out of scope pour l'audit (NE PAS rebattre)

L'audit S75 **audite**, il ne re-concoit pas. Ne pas rebattre :
- **D1-D5 gelees** (kickoff §5) : substrat `NodeDirectoryEntry` sibling (D1),
  FIX-A re-mint d'abord + fenetre 1800s inchangee (D2), VPS 2 roles bornes +
  sign-off PO (D3), durabilite locator + re-pull boot (D4), wire additif 0-bump
  + multi-provider (D5). Les 3 PLAN-ADAPT (C/D/F) sont des adaptations
  d'implementation DANS le cadre D1-D5, deja arbitres.
- **Le DEFER SearchManifest (D3/s73)** — TIENT, confirme par pivot_proposal +
  sign-off PO. Ne pas exiger SearchManifest.
- **Les 12 scope cuts** (kickoff §9) : SearchManifest (#1), Tantivy (#2), GC
  reaper (#3), recherche federee (#4), approbation pair seed (#5),
  mobile/Electron (#6), migration wire (#7), GPU→S76 (#8), sharding→S77 (#9),
  kudos-threshold tuning (#10), multi-ancre UX avancee (#11), Bloom/Merkle (#12).
- **Pre-launch policy** : pas de bump `*_VERSION` tant que rien n'est pousse ;
  canonical librement editable ; ne PAS exiger de migration wire.
- **Le pin iroh 0.98** et les arbitrages PO (amendement roadmap, sign-off D3,
  choix Q3 default_curators).
- Re-corriger un P2/P3 deja documente (router vers S76+ phases, pas le
  re-implementer en Phase 0).

---

## §6 Verdict global attendu

- **PASS** : 0 P0, 0 P1 → S76 Phase A demarre direct. **Scenario attendu** —
  les ~12 P2 candidats sont des conceptions deferees (duress lot, PULL-3,
  anti-Sybil, re-drive) ou des residus documentes (over-count, re-pull
  sequentiel), non bloquants ; les invariants headline (bug live ferme row 7 ;
  durabilite row 10 ; lock-3 tripwire vierge ; verrou-4 marquage
  editeur-seulement ; survives-VPS-death trace) tiennent ; G1 present.
- **CONDITIONAL PASS** : 1-3 P1 fixables → S76 Phase A bloque tant que les
  `fix(sprint75): ...` ne sont pas landed. **Candidats P1 a trancher** :
  (1) Track G — un chemin front lit `is_open_source` des boucles
  curator/nodedirectory hardcodees (`browse.rs:684/803`) → faux marquage
  derive ; (2) Track J — un node_id/ancre hard-code a glisse (lock-3) ;
  (3) Track F — un chemin duress publie/annonce des donnees du vrai data root
  malgre le gate driver ; (4) Track H — l'acceptance survives-VPS-death ou le
  run Docker canonique n'a pas de trace reelle (self-report non re-jouable).
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle.

---

## §7 Livrable final attendu

`sprint75_audit_findings.md` (pattern Sprint 6/7), sections :
1. **Auditeur** — id session, duree.
2. **Tip audite** — SHA master pris comme base (tip code A-F `4f52bea` + G).
3. **Verdict global** — PASS / CONDITIONAL PASS / FAIL.
4. **Une section par track (A-K + G1 + HARDENING)** avec verdict (PASS /
   CONCERN / FAIL) + findings.
5. **Findings list sorted by severity** — table P0 → P3.
6. **Commits fix attendus** — si CONDITIONAL PASS, liste `fix(sprint75): ...`
   prealable au kickoff S76.
7. **P2 a logger en tech debt** — items vers `PATTERNS.md` sans code change.
8. **P3 laisses sans action** — nits ignores.
9. **Notes on audit completeness** — ce qui n'a pas ete couvert et pourquoi
   (notamment si l'env bloque le live cross-machine : le consigner, auditer la
   trace Phase G a la place).

**Critere SMART** : la fail-fast `verification.md` rejoue verte (Windows
nextest 0 fail + Docker Linux canonique 0 fail + web 367+/coverage/size) +
lock-3 tripwire vierge + 0 P0/P1 non resolu = S76 kickoff debloque (GPU partage
cross-machine).

**Exit Gate** : l'audit S75 est complet quand `sprint75_audit_findings.md`
porte un verdict avec >= 1 P2+ (G4), couvre toutes les tracks, ingere le diff
complet S75 (Phases 0 + A-G), confirme les invariants headline (FIX-A ferme le
bug live ; durabilite locator reboot-prouvee ; lock-3/lock-4 sains ;
survives-VPS-death trace), et tranche les 4 candidats P1 du §6.
