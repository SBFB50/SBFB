# Sprint 75 — Audit findings (audit gate de S75, joue en S76 Phase 0)

Pattern Sprint 6/7. Verdict d'entree de S76 Phase 0.

---

## 1. Auditeur

- **Session** : session fraiche S76, Phase 0 (Cas A audit gate).
- **Methode** : orchestration workflow multi-agents anti-anchoring. Opinion
  formee depuis le **code livre + `git show`/`diff`/`log`** (base
  `0e2fb6b` → tip `38c5578`) AVANT toute lecture des self-reports
  (`sprint75_verification.md`, `sprint75_phase_*_review.md`), conformement au
  §0 du plan d'audit (ordre de lecture impose).
- **Tracks** : 13 pistes (B/C/D/E/F/G/H/I/J/K + HARDENING + UX-ARRIVAL + E2E)
  + G1 presence, chacune confiee a un agent independant ; 4 candidats P1
  passes au crible d'un agent skeptic dedie (refute/confirme avec evidence
  file:line).
- **Self-reports** : lus APRES formation d'opinion, pour COMPARER. Les ecarts
  self-report vs audit sont signales explicitement en §3 et §4 (le plus net :
  le self-report classe la fuite duress en dette M routee S76 ; l'audit la
  re-classe P1 bloquant).

---

## 2. Tip audite

- **Base** : `0e2fb6b` (audit gate S74 PASS, Phase 0 S75).
- **Tip code S75 (7 phases A-G)** : `8b53c38` (Phase G wrap-up ; tip code
  A-F = `4f52bea`).
- **Tip reel de l'arbre au moment de l'audit** : `38c5578`. La fenetre
  `0e2fb6b..38c5578` inclut, EN PLUS des 7 phases S75, le mini-cycle
  hors-sprint **UX-ARRIVAL** (`e980d7e`) + 3 hotfixes Cas D
  (`fdb4fb1` dedup grille /browse, `173426e` reconcile keep-online CTA,
  `38c5578` auto-prune outbox stale). Ces surfaces post-S75 sont DANS la plage
  d'audit et ont ete couvertes (tracks UX-ARRIVAL + I + J + HARDENING), bien
  que livrees hors du cycle review/Codex standard (process lean assume PO,
  substitue par verif live cross-machine PC/Mac/VPS).
- **Commits feat** : Phase 0 `0e2fb6b`, A `479a87c`, B `f6637d3`,
  C `821aa8c`, D `0010450`, E `1486fc9`, F `4f52bea`, G `8b53c38`
  (+ chores : kickoff `f008433`, preflight A `e3c3fb6`, handoffs).

---

## 3. Verdict global

### CONDITIONAL PASS

**Justification.** Le coeur du pivot decouverte PULL (re-mint PoW+adresse au
replay, `NodeDirectoryEntry`/`DOMAIN_NODE_DIRECTORY_V1` crypto-isole, durabilite
locator reboot-prouvee, pull multi-provider, ancre VPS headless, front
node-Browse verrou-4) est **SAIN** : 0 P0, machinerie crypto/wire correcte,
0 bump wire, 0 delta dependance, lock-3 tripwire VIERGE, verrou-4 marquage
editeur-seulement confirme, acceptance survives-VPS-death tracee et corroboree
par tests deterministes. Les 3 invariants headline du §6 tiennent (bug live
ferme ; durabilite reboot-prouvee ; lock-3/lock-4 sains).

**MAIS un P1 bloquant est confirme** (skeptic #3 CONFIRMED, deux chemins) :
le mode duress echange l'identite (keypair leurre) mais PARTAGE le data root
reel. Deux chemins de boot publient/diffusent des donnees du VRAI data root
**signees sous la cle leurre, automatiquement a chaque boot, zero interaction
utilisateur** — exactement le critere d'escalade P1 fixe par le plan d'audit
(§6 : « un chemin duress qui publie/annonce des donnees du vrai data root »).
C'est une **omission de garde** sur des chemins pre-existants (S74 Phase F pour
`reannounce_seeds_at_boot`, S66 Phase C pour la republication feed), pas une
contrainte d'architecture : le frere NEUF `run_boot_seed_driver` (Phase E)
garde explicitement ce risque exact sur la MEME sequence boot. L'asymetrie
(chemin S75 garde, freres non gardes) prouve l'oubli.

Verdict **CONDITIONAL PASS** plutot que FAIL : le P1 est isole, sans impact sur
l'integrite content-addressed (BLAKE3 reste l'autorite), et fixable par un
short-circuit duress + 1 test par chemin (miroir de `boot_seed_driver_noop_in_duress`).
S76 Phase A est **bloque** tant que les `fix(sprint75): ...` de §6 ne sont pas
landed.

**Ecart self-report.** `sprint75_verification.md` §8 et `THREAT_MODEL.md:883`
documentent HONNETEMENT le gap duress des freres, mais le classent en dette M
routee S76. L'audit re-qualifie le chemin AUTOMATIQUE-au-boot `reannounce_seeds_at_boot`
en **P1**, et identifie un DEUXIEME chemin (republication feed S66
`runtime.rs:769-795` + orphan recovery `799-854`) qui n'apparait NI dans
THREAT_MODEL NI dans `sprint75_phase_e_review.md` — plus large que le residu
documente, a fermer dans le meme lot.

Compte : **0 P0, 1 P1, 14 P2, 6 P3.**

---

## 4. Verdict par track

### Track B — Phase A re-mint PoW + adresse replay (`479a87c`) — PASS

Le fix du bug live de decouverte est correct et complet. `MAX_PROOF_AGE_SECS=1800`
INCHANGE (le re-mint rend la fenetre correcte sans la supprimer) ; const-assert
compile-time `SESSION_WINDOW(900) < MAX_PROOF_AGE_SECS(1800)` LOAD-BEARING contre
regression. L'outbox persiste le payload NON-WRAPPE ; les 4 sites broadcast
(browse_request, NeighborUp, GossipCmd::Outbox live, periodic republish) routent
tous par `remint_and_wrap_for_replay` (re-mint BlobTicket via `my_endpoint_addr()`
courant + re-stampe PoW frais) ; le boot restore re-ingere localement sans
broadcast. Garde anti-hijack `ann.node_id == node.node_id()` confine le re-mint
aux annonces PROPRES. 3 tests load-bearing avec controles positif+negatif decisifs.
NOTE : le 3e test cite par le plan (`replay_keeps_stale_ticket_when_blob_is_gone`)
existait verbatim a la cloture S75 `8b53c38` ; le hotfix post-S75 `38c5578` a
INVERSE le comportement GC'd-blob (retourne None pour cesser d'advertir un blob
non-servable) et le test a evolue en `replay_drops_announcement_when_blob_is_gone`
— coherent avec le code au tip, 0 zombie. 0 bump wire.
Findings : T6 (P2), WS-3/PD-5 hoisting (P2), double-broadcast publish (P3).

### Track C — Phase B NodeDirectoryEntry + domaine + ingest gate + authoring (`f6637d3`) — PASS

Sain sur les 4 sous-questions. `DOMAIN_NODE_DIRECTORY_V1=b"nexus-node-directory-v1"`
disjoint des 18 domaines existants (construction `<domain><0x00><JCS>`, prouve
par test d'inegalite-octets ET rejet de signature cross-domaine) ;
`NODE_DIRECTORY_FORMAT_VERSION=1` NOUVEAU, aucun `*_FORMAT_VERSION` existant
bumpe (additif pur). Curator + directory passent par le MEME gate
`verify_signed_list_ingest<T:SignedList>` (steps 6/7/8) ; aucun chemin reseau
ne contourne le gate. Drop self-spoof via `announcement_claims_own_node_id` +
self-guard registre. Route `POST /api/daemon/directory/publish` derriere
`auth_required` ; caps a l'authoring (MAX_ENTRIES, troncature UTF-8-safe,
archive_hash valide-non-tronque) ; verrou-4 ferme par `own_entries` (match
node_id) PLUS garde `blobs.has(hash)`. Les 4 vrais gaps (revision non-atomique,
archive_hash tronque, discriminateur trop tolerant, poison own_entries) captures
par les rounds review/Codex et fixes presents.
Findings : durabilite revision dependante de SBFB_HOME (P2, mitige systemd),
identites forgees observed non tarifees (P2, residuel assume).

### Track D — Phase C ingest annuaire + durabilite locator (`821aa8c`) — CONCERN

Coeur SAIN et bien teste : locator `anchors.json {pubkey,ticket,revision}`
re-valide par le gate partage a chaque re-fetch (ticket jamais autorite) ; floor
anti-rollback REBOOT-DURABLE prouve par vrai test disque (revision=5 lu, RAM vide,
re-pull, rejet rev-4) + cas re-pull-echoue (RAM vide → floor `>=P` restaure
same-revision). Subscription-gating a l'ingest (step 4 AVANT tout fetch) ET au
read + eviction a l'unsubscribe (verrou 5). WIRE-1/WIRE-2/DBQ-1 reellement fermes
et testes. WIRE-3 + CARRY-3 re-routes EXPLICITEMENT vers S76, pas tombes
silencieusement.
**CONCERN** : CARRY-3 (residu cote aggregator) — `is_open_source` est stocke
VERBATIM depuis un `ProjectAnnouncement` gossipe NON signe (`runtime.rs:2231`),
et le chokepoint downgrade `trustworthy_open_source` ne s'applique QU'a l'index
recherche, pas a `/browse`. Le front Phase F (NodeCatalog) consomme `/browse`
source==="direct" → un pair peut gossiper `is_open_source:true` matchant
(pid,hash) d'une app catalogue pour forger un badge « Source verifiable ». La
review Phase F n'a traite que le sens inverse (curator/nodedirectory hardcodes
false). Classe **P1** par cet auditeur (consommateur live au tip) — voir §5
finding #2. NOTE : le skeptic #1 a REFUTE la variante NodeCatalog du faux
marquage (le finder `source==="direct"` exclut structurellement curator/nodedirectory),
mais le residu CARRY-3 vise le canal `direct` lui-meme (annonce gossip direct
non signee), distinct de la variante refutee. Discordance interne tracee :
voir §5.
Findings : CARRY-3 spoof is_open_source canal direct (P1 selon auditeur Track D,
voir adjudication §5), known_entry_count double-compte (P2).

### Track E — Phase D pull multi-provider + SeedRegistry + /nodes (`0010450`) — PASS

SAIN. `fetch_hash_multi`/`fetch_and_pin_multi` telechargent le hash nu via
`Downloader.download` avec vec ORDONNE ancre-d'abord, `MAX_FETCH_PROVIDERS=16`
enforce DANS la primitive (pas convention d'appelant) ; integrite BLAKE3 prouvee
(provider sans le hash echoue, ne substitue jamais d'octets). blob_serve 4e tier
+ seed_voluntary fallback resolvent via `directory_snapshot()` filtre
is_subscribed (verrou-5), garde `h==want_hash`, timeout 120s. SeedRegistry prod :
SEED-1 clamp, SEED-2 double cap anti-displacement, normalisation hex lowercase
write+read (chaque defense testee). `/nodes` additive dans authed_routes ;
`/browse` BYTE-IDENTIQUE confirme. 0 bump wire, 0 dep. Les 2 items deferes
(PULL-3, sampling anti-Sybil) genuinement absents et honnetement traces P2 → S76.
Findings : PULL-3 cross-tier failover (P2), sampling anti-Sybil seeder tail (P2),
blob-serve dials drive-by + amplification N*8 (P3).

### Track F — Phase E ancre VPS headless + duress (`1486fc9`) — CONCERN

Globalement solide : `[seed] keep_online_projects` defaut compile VIDE (verrou 3,
tripwire test) + clamp lowercase-64hex + PAS de section [directory] ; boot driver
`run_boot_seed_driver` correctement duress-gate EN TETE, dedup, resolution
direct>M18>annuaire figee par test, content-hash verifie, abort+join shutdown ;
route `POST /api/daemon/seed/request` duress short-circuit avant signature,
self-guard parse, invite_token TOUJOURS transmise (claim self-designation fausse
corrigee), mint 409 ; unit systemd epingle SBFB_HOME+NEXUS_GRID_ROOT, AF_NETLINK,
@system-service, CapabilityBoundingSet vide.
**CONCERN P1** : deux chemins de boot publient des donnees du VRAI data root sous
la cle DECOY en duress, sans aucun gate, automatiquement :
(1) `reannounce_seeds_at_boot` (`runtime.rs:862-864` → `feed_sync.rs:160-200`)
lit `list_keep_online_enabled()` (vraies lignes) et emet un `SeedAnnounced` signe
leurre par app ; (2) republication feed S66 (`runtime.rs:769-795` + orphan
recovery `799-854`) emet l'INTEGRALITE du feed reel sous la cle leurre — ce
2e chemin n'est NI dans THREAT_MODEL NI dans la review Phase E. Voir §5 finding #1.
Findings : duress leak `reannounce_seeds_at_boot` (P1), duress leak republication
feed S66 (P1, meme lot), surfaces seed_voluntary/set_keep_online sans gate duress
(P2), re-drive-on-ingest fenetre morte 1er boot (P3).

### Track G — Phase F front node-Browse + verrou-4 + WEB-1 (`4f52bea`, hotfixes `fdb4fb1`/`173426e`) — PASS

Audit statique du verrou-4 : SAIN. Le candidat P1 #1 (faux marquage derive d'une
source non-editeur via NodeCatalog) n'existe PAS — les marqueurs « Source
verifiable »/« Version derivee » derivent EXCLUSIVEMENT de `publisherEntry`,
filtre strict `source==="direct"` + match exact (pid,hash). Les boucles curator
(`browse.rs:684`) ET nodedirectory (`browse.rs:803`) hardcodent
`is_open_source:false`/`provenance_hash:None` ; aucun chemin front ne les lit
pour fabriquer un marqueur. Test lock-4b load-bearing non regresse (PID_ORPHAN
asserte AUCUN badge + pinne `source==="direct"` strict). Badge Q7 front-compose
correct, /browse byte-identique. WEB-1 self_pin_enabled 3-etats correct
(precedence echo>intent>defaut-ON, 0 setState-in-effect). AddAnchorDialog = vraie
action subscribe. lock-3 tenu.
Findings : discriminateur curator-vs-ancre absent sur lignes « en attente »
/nodes (P2, copy UI honnete), hotfixes co-signes modele non-S75 hors gate Codex
(P3, fixes verifies sains).

### Track H — Phase G carries + acceptance survives-VPS-death (`8b53c38`) — PASS

Les 4 carries d'hygiene S74 sont REELS et correctement cibles, chacun couvert par
un test decisif : CARRY-5 (clamp offset+q, limit.min(100) pre-existant non
re-touche), CARRY-2 (`reject_result_on_guardrail_trip` Rejected TERMINAL aux 2
ingress, zombie path mort), PULL-1 (`strip_zip_member` avant hash, no-op
byte-identique), FORK-1 (`MAX_ARCHIVE_ENTRIES=4096` avant write). Acceptance
survives-VPS-death a une trace horodatee re-jouable (`.git/S75_PHASE_G_ACCEPTANCE.md`,
UTC, node_ids reels, render HTTP 200 19926 o, marqueurs [!] d'echec HONNETES sur
peer_count:0). THREAT_MODEL §15.1 rows S75 presents. Commit touche exactement les
5 fichiers annonces. Le skeptic #4 a REFUTE le candidat « pas de trace re-jouable ».
Findings : binaire d'acceptance builde working-tree non re-execute sur le commit
committe ni pousse (P2), double-rendu /browse cross-canal non detecte par
l'acceptance corrige en hotfix post (P2), coquilles nommage/compteur disclosees
(P3).

### Track I — Wire 0-bump + pre-launch (transverse) — PASS

Cargo.lock diff VIDE, 0 changement Cargo.toml → 0 delta dependance ; iroh 0.98 +
decisions Day-0 intactes, pas de wasmtime. Tous les `*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`
= 1 ; `INVITE_FORMAT_VERSION=2` deja a 2 au base (pas un bump S75) ;
`NODE_DIRECTORY_FORMAT_VERSION=1` NOUVEAU additif. Aucun decoder legacy/multi-version ;
le seul handling « legacy » (`normalize_outbox_payload`, hotfix `38c5578`) est une
migration d'etat LOCAL persiste, explicitement NOT a wire-format legacy decoder.
Tous les nouveaux `serde(default)` portent un rationale runtime-tolerance.
Day-0 D1-D5 preservees ; seul contact D3 passe par pivot_proposal + sign-off PO.
Findings : champ /browse `from_subscribed` ajoute post-S75 par UX-ARRIVAL → /browse
non byte-identique au tip exact 38c5578 (P3, additif serialize-only tolere Zod),
ANCHORS_SCHEMA_VERSION=1 nouveau schema de persistance pas wire (P3, OK additif).

### Track J — 5 verrous anti-recentralisation (TRIPWIRE lock-3) — PASS

Les 5 verrous tiennent au tip 38c5578. lock-1 : DeployFromRepoRequest sans champ
hote/cible/node_id ; formulaire Deploy.tsx sans champ hote. lock-2 : boucle
NodeDirectory `out.push()` additif (jamais substitution grille curator) ; front
isole l'ambiant dans section « Decouvert sur le reseau » separee.
lock-3 (TRIPWIRE) : grep `135.181.42.188`/`192.168.1.53` dans crates/ = ZERO ;
tous les hex-64 sous `#[cfg(test)]` ; CuratorConfig+SeedConfig derivent Default
Vec vide ; config.toml.example default_curators=[]/keep_online_projects=[] ;
systemd sans IP/anchor hardcode ; AddAnchorDialog placeholder inerte. lock-4 :
verified/derived/provenance UNIQUEMENT depuis publisherEntry source==="direct".
lock-5 : seeding app etrangere = onClick explicite ; ingest annuaire non-abonne
ni fetch ni dial (record observed = metadata d'enveloppe). Le skeptic #2 a REFUTE
la violation lock-3 alleguee (par analyse statique exhaustive : 0 IP/node_id
prod hors tests, default vides compiles).
Findings : surface UX-ARRIVAL observed ingeree hors cycle gate standard (P3,
locks tiennent, couvert live).

### Track K — Meta-process (G8/reviews/Codex/commits) — PASS

G8 7/7 : preflights A-G avec verdicts attendus (A SCOPE-CUT-CONSISTENT, B EXECUTE,
C/D/F PLAN-ADAPT, E/G SCOPE-CUT-CONSISTENT), 0 DESIGN-CONFLICT ; chaque PLAN-ADAPT
porte une evidence ground-truth file:line. Reviews 6/6 + G toutes `## Verdict: PASS`
format exact ; Phase A PASS authentique (2 P1 test-only fermes in-phase).
Codex artefacts a-g BRUTS (per-livrable, file:line, evidence d'execution).
Commits : 7 feat A-G titres conformes, 9 sections ## exactes. G1 design_review
present avec scoring D1-D5, D3 ⚠️ = gate d'arbitrage PO resolu via pivot_proposal.
Findings : divergence nommage test plan-vs-code consignee (P3), Codex Phase A en
francais (CONFIRME vs CONFIRMED) — asymetrie lightcheck (P3).

### Track HARDENING — THREAT_MODEL + nouvelles surfaces — PASS

Chaque nouvelle surface S75 a une row THREAT_MODEL §15.1 (v8+v8.1) et le code
correspond aux mitigations revendiquees : node_directory.rs separation de domaine
testee, caps per-field + MAX_ENTRIES a sign ET verify, archive_hash strict ;
ingest subscription-gated ; oracle blob-serve resout subscribed-only + cap +
timeout ; routes /seed/request + /nodes + /directory/publish dans authed_routes ;
duress short-circuit avant signature sur les 3 routes producteur + driver ;
registre observed borne dans la primitive ; systemd durci. Residuels fresh-flood +
seeder-tail + duress freres correctement documentes → S76.
Findings : mitigation THREAT_MODEL fausse — blob-serve PAS bearer-gate (route
publique deliberee) mais la cellule revendique « loopback bearer requis » (P2,
drift doc) ; LOOPBACK_ENDPOINTS_TRUST_TIERS §3 non mis a jour pour les routes
S74/S75 (premisse plan d'audit fausse, drift cumule 2 sprints) (P2) ; residuels
fresh-flood/seeder-tail/duress documentes (P3).

### Track UX-ARRIVAL + hotfixes post-S75 (`e980d7e`/`fdb4fb1`/`173426e`/`38c5578`) — PASS

Coeur securite sain : registre observed RAM borne dans la primitive (cap 256 +
eviction stalest + TTL 48h + rate-limit 1/min, Mutex pour atomicite), capture
etape 4 AVANT drop NotSubscribed et AVANT tout fetch (test 2-noeuds ticket
infetchable) ; SEC-UXARR-1 resolu a la racine (`from_subscribed` CATALOG-BACKED
contre le catalogue Ed25519-verifie, pas l'appartenance node_id) ; /nodes observed
= 2 champs cheap-envelope .strict() ; THREAT_MODEL §15.1 v8.1 documente les 2
residuels. Hotfixes : fdb4fb1 dedup display-only (octets /browse inchanges),
173426e reconcile keep-online via React Query read, 38c5578 inverse deliberement
T3 (kept-online=pinne skip-GC ⇒ blob absent = app retiree).
Findings : surface UX-ARRIVAL absente du sprint76_audit_plan d'origine a inscrire
(P2), publisher-binding observed identites forgees non tarifees (P2), dedup
curator/direct 2 cartes 2 sections (P3), pas de TTL recepteur direct entries (P3),
incoherence cosmetique nom test Zod (P3).

### Track E2E (CI + bridge allowlist + frontend coverage) — CONCERN

Tracks pre-identifies du plan, audites statiquement.
Findings : CI etape [10] playwright = no-op (aucun config/spec, green trompeur,
pre-existant S10) (P2) ; drift allowlist bridge TS(15)/Rust(10) + moindre-privilege
par app NON applique au runtime (`manifest.methods` jamais consulte au dispatch) —
pas d'evasion sandbox (les 15 sont des capacites hote deliberees), drift contrat
(P2) ; couverture frontend 5/10 pages sans test, suite Vitest 100% mockee (P2,
dette de test). Aucun n'est une regression S75.

### Track G1 — presence design_review — PASS

`sprint75_design_review.md` present avec scoring complet D1✅ D2✅ D3⚠️ D4✅ D5✅,
le ⚠️ D3 etant le gate d'arbitrage PO documente, resolu via
`sprint75_pivot_proposal.md` present (la piece du sign-off D3). Gate non bypasse.

---

## 5. Findings list (trie par severite)

### Adjudication des 4 candidats P1 (verdicts skeptics)

| # | Candidat (plan §6) | Verdict skeptic | Severite finale |
|---|---|---|---|
| #1 | Front lit is_open_source des boucles curator/nodedirectory hardcodees → faux marquage (NodeCatalog) | **REFUTE** (finder source==="direct" exclut structurellement curator/nodedirectory ; lock-4b load-bearing) | Pas un finding (variante NodeCatalog) — voir P1-AUDIT-2 pour le residu CARRY-3 distinct (canal direct) |
| #2 | node_id/IP/pubkey hard-code dans crates/ hors tests (lock-3) | **REFUTE** (analyse statique exhaustive : 0 IP/node_id prod, default vides compiles, tous hex sous #[cfg(test)]) | Pas un finding |
| #3 | Chemin duress publie/annonce des donnees du vrai data root malgre le gate driver | **CONFIRME** (2 chemins automatiques au boot, signes sous cle leurre) | **P1** |
| #4 | Acceptance survives-VPS-death / C6 / Docker n'a pas de trace re-jouable | **REFUTE** (trace horodatee `.git/S75_PHASE_G_ACCEPTANCE.md` + 3 claims cardinaux corrobores dans le code) | Pas un finding |

**Discordance interne tracee (CARRY-3)** : le skeptic #1 REFUTE le faux marquage
via NodeCatalog (correct : le finder front exclut curator/nodedirectory). L'agent
Track D souleve un residu DISTINCT — le canal `direct` lui-meme (annonce gossip
`ProjectAnnouncement` non signee) peut porter `is_open_source:true`, stocke
verbatim a `runtime.rs:2231` et lu par `publisherEntry` source==="direct".
**Adjudication de l'auditeur** : ce residu est **REEL** mais **P2, pas P1**, car
(a) c'est le carry CARRY-3 deja IDENTIFIE et re-route explicitement vers S76
(Track D confirme le re-routage), (b) la recommandation S74 (downgrade
`trustworthy_open_source` a l'ingress aggregator) est connue et non un fix
d'urgence, (c) l'invariant cardinal tient : un badge forge ne sert jamais
d'octets absents, BLAKE3 reste l'autorite, et le verrou-4 ne doit pas etre
presente comme attestation crypto (a documenter THREAT_MODEL). Il ne bloque pas
le gate au-dela du P1 duress deja present. Logge P2 `CARRY-3-AGGREGATOR-SANITIZE`.

### Table findings

| Sev | Id | Track | Finding | Evidence |
|---|---|---|---|---|
| **P0** | — | — | Aucun | — |
| **P1** | DURESS-BOOT-LEAK | F | `reannounce_seeds_at_boot` + republication feed S66 publient/diffusent des donnees du VRAI data root signees sous la cle leurre, automatiquement a chaque boot, sans gate duress | `runtime.rs:862-864` + `feed_sync.rs:160-200` ; `runtime.rs:769-795`+`799-854` (2e chemin non documente) |
| P2 | DURESS-FRERES-LOCAL | F/E2E | `seed_voluntary` + `set_keep_online` mutent le vrai data root sous duress (local-only, pas d'emission wire) — incoherence modele duress | `http.rs:1529-1577`, `http.rs:2066/2206` |
| P2 | CARRY-3-AGGREGATOR-SANITIZE | D | `is_open_source` gossipe non signe stocke verbatim /browse → badge forgeable (residu aggregator, consomme live par verrou-4 front) | `runtime.rs:2231`, `publish.rs:181` |
| P2 | PULL-3 | D/E | Cross-tier failover absent (ticket mort → pas de bascule directory/multi-provider) | `http.rs` seed_voluntary ~2099-2189 |
| P2 | SYBIL-SEEDER-TAIL | D/E | Crowding lexicographique du dial set (sort hex, cap N premiers) — pas de sampling anti-Sybil | `seed_registry.rs:182-194`, `http.rs` directory_pull_providers |
| P2 | T6-OUTBOX-DIRECT | B | Handler `GossipCmd::Outbox` broadcast non teste en direct (2 noeuds) | `runtime.rs:1787` |
| P2 | WS-3/PD-5 | B | `my_endpoint_addr()` + double normalize appeles par-entree a chaque passe replay (efficience) | `runtime.rs:1655-1850` |
| P2 | REVISION-HOME-DURABILITY | C | Anti-rollback revision depend de SBFB_HOME resolvable (mitige systemd) | `http.rs:1492-1515` |
| P2 | OBSERVED-FORGED-IDS | C/UX | Identites forgees observed non tarifees individuellement (PoW lie publisher,topic pas payload) | `iroh_runtime.rs:507-521`, `:1148-1160` |
| P2 | KNOWN-ENTRY-OVERCOUNT | D | `known_entry_count` double-compte app en curator-list ET annuaire (superset honnete) | `iroh_runtime.rs:773-786` |
| P2 | DISCRIMINATEUR-CURATOR-ANCRE | G | Lignes « en attente » /nodes ne distinguent pas curator pur vs ancre-non-annoncee | `Nodes.tsx:164,347` |
| P2 | THREAT-BLOBSERVE-BEARER | HARDENING | THREAT_MODEL revendique « loopback bearer requis » sur blob-serve qui est route PUBLIQUE deliberee | `THREAT_MODEL §15.1`, `http.rs:252-255` |
| P2 | LOOPBACK-TIERS-STALE | HARDENING | LOOPBACK_ENDPOINTS_TRUST_TIERS §3 non mis a jour routes S74/S75 (drift cumule 2 sprints) | `LOOPBACK_ENDPOINTS_TRUST_TIERS.md §3` |
| P2 | CI-PLAYWRIGHT-NOOP | E2E | Etape CI [10] playwright = 0 spec/config → green trompeur (pre-existant S10) | `.github/workflows/ci.yml:84-85` |
| P2 | BRIDGE-ALLOWLIST-DRIFT | E2E | Allowlist bridge TS(15)/Rust(10) + moindre-privilege par app non applique runtime | `protocol.ts:20-44`, `sbfb-manifest/lib.rs:52-63`, `useBridge.ts:224-373` |
| P2 | FRONTEND-COVERAGE-GAP | E2E | 5/10 pages sans test, suite Vitest 100% mockee (dette de test) | `web/src/pages/__tests__/` |
| P2 | UX-ARRIVAL-PLAN-INSCRIPTION | UX | Surface UX-ARRIVAL absente du sprint76_audit_plan d'origine (a inscrire) | `e980d7e` body §Carry closure |
| P3 | DOUBLE-BROADCAST-PUBLISH | B | Double-broadcast au publish (live + Outbox re-mint) — idempotent, pre-existant | `deploy.rs`, `runtime.rs:1787` |
| P3 | BLOBSERVE-DIALS-AMPLIF | E | blob-serve GET (directory-only) → N*8 dials sans dedup in-flight (loopback-only) | `http.rs:2141` |
| P3 | RE-DRIVE-ON-INGEST | F | Driver one-shot → fenetre morte 1er boot ancre fraiche (remede operateur) | `http.rs:1718-1726` |
| P3 | HOTFIXES-HORS-CODEX | G | Hotfixes fdb4fb1/173426e co-signes modele non-S75 hors gate Codex/preflight (fixes sains) | git log |
| P3 | BROWSE-NOT-BYTE-IDENTIQUE-TIP | I | Champ /browse `from_subscribed` post-S75 → /browse non byte-identique au tip 38c5578 (additif tolere) | `http.rs:929-935` |
| P3 | DIVERS-NITS | K/UX/H | Nommage test plan-vs-code, Codex FR vs EN, coquille DOMAIIN, dedup curator/direct 2 cartes, pas de TTL recepteur direct, ANCHORS_SCHEMA_VERSION=1 | divers |

---

## 6. Commits fix attendus (CONDITIONAL PASS — prealable au kickoff S76)

S76 Phase A est **bloque** tant que ces commits ne sont pas landed. Le P1 est un
lot duress unique (pas des rustines separees) :

1. **`fix(sprint75): gate reannounce_seeds_at_boot on identity mode (duress no-op)`**
   — court-circuit `gossip_publish_in_duress(identity_mode) == Noop → return 0`
   AVANT la lecture des lignes keep_online et tout emit, miroir de
   `run_boot_seed_driver` (`http.rs:1739`). Wrapper l'appel `runtime.rs:862-864`.
   + test `boot_seed_reannounce_noop_in_duress` (miroir de
   `boot_seed_driver_noop_in_duress` `http.rs:5735`) verifiant ZERO emission feed
   sous `IdentityMode::Duress`.

2. **`fix(sprint75): gate boot feed republish (S66 6c-5/6c-5b) on duress`**
   — gater les etapes `runtime.rs:769-795` (republish `replay_all`) ET
   `runtime.rs:799-854` (orphan recovery) en duress (skip toute republication
   feed). + test asserting ZERO `publish_feed_entry_to_docs` sous duress.
   + documenter le chemin dans THREAT_MODEL (il manque actuellement).

> Les deux fixes ferment le meme invariant (aucune emission wire-observable
> du vrai data root sous la cle leurre). Documenter dans THREAT_MODEL la
> frontiere wire-emit (P1, ferme ici) vs local-mutate (P2 DURESS-FRERES-LOCAL,
> route S76). Apres landing : re-jouer la fail-fast Windows nextest + Docker
> canonique (gate avant push) ; le delta attendu = +2 tests (un par chemin).

---

## 7. P2 a logger en tech debt (vers PATTERNS.md / plan S76, sans code change)

Routes vers S76 (conception, pas re-implementation en Phase 0) :

- **Lot duress freres local-only** (DURESS-FRERES-LOCAL) : `seed_voluntary` +
  `set_keep_online` — court-circuit duress (no-op + reponse leurre coherente),
  meme lot que le P1 wire-emit, frontiere documentee.
- **PULL-3 cross-tier failover** : chaine de fallback ordonnee
  (ticket mort → directory → multi-provider) + call-site driver E.
- **Sampling anti-Sybil seeder tail** : random sampling de l'ensemble
  fresh-seeder au-dela des N premiers (residuel availability-only, ancre slot 0
  non crowdable).
- **Re-drive-on-ingest** : re-driver le boot driver a l'ingest d'un annuaire
  couvrant un project_id configure (fenetre morte 1er boot).
- **Discriminateur curator-vs-ancre** : `listCurators().entries` distingue les
  deux familles sans changement wire (lignes « en attente » /nodes).
- **CARRY-3-AGGREGATOR-SANITIZE** : re-appliquer le downgrade
  `trustworthy_open_source` (is_open_source && provenance_hash.is_some() &&
  repo_url.is_some()) a l'INGRESS aggregator (`runtime.rs:2231`), single
  chokepoint partage avec l'index ; documenter THREAT_MODEL §15.1 que /browse
  is_open_source est spoofable et que verrou-4 n'est PAS une attestation crypto.
- **SeedAnnounced ne converge pas cross-noeud** (constat acceptance live G,
  peer_count:0 ~10 min) : investiguer la propagation feed (sync doc cross-swarm),
  lier a PULL-3 (registre toujours-vide affaiblit le dial-set).
- **Annuaire du seeder n'annonce pas ce qu'il seede** (catalog_len:0 live) :
  question design PO (section « seeded » distincte non-autoritaire vs verrou-4 +
  modele F-Droid).
- **Publisher-binding du registre observed** : lier la capture observed a
  l'identite PoW du publisher (avec le lot duress), borne la forge d'identites.
- **THREAT-BLOBSERVE-BEARER** : corriger la cellule mitigation de la row oracle
  blob-serve (route publique par construction, amplification bornee par
  resolution subscribed-only + cap + timeout, contenu public-par-hash).
- **LOOPBACK-TIERS-STALE** : ajouter les 7 routes S74+S75 a l'inventaire §3
  avec tier cible (toutes T0 ; candidat T1 /directory/publish + /seed/request)
  + corriger la phrase du plan d'audit affirmant une couverture inexistante.
  Drift cumule 2 sprints — proche du seuil 3+ (escalade G7 si non traite).
- **CI-PLAYWRIGHT-NOOP** : ajouter >=1 spec Playwright reel (l'infra
  global-setup/teardown spawn deja un vrai daemon) OU retirer l'etape [10] +
  install browsers.
- **BRIDGE-ALLOWLIST-DRIFT** : aligner BRIDGE_METHOD_ALLOWLIST Rust sur les 15
  (ou test de parite TS/Rust) ; si moindre-privilege par app voulu, passer
  `manifest.methods` jusqu'a useBridge et rejeter au dispatch ; sinon documenter
  que `methods` est purement declaratif.
- **FRONTEND-COVERAGE-GAP** : smoke tests render pour Network/Curators/Projects/
  OnboardingEmpty/ProjectDetail (au minimum les pages a logique).
- **T6-OUTBOX-DIRECT** : test 2-noeuds GossipCmd::Outbox neighbor_count>0
  (pattern hijack-guard A).
- **WS-3/PD-5** : hoister `my_endpoint_addr()` once-per-pass au replay.
- **KNOWN-ENTRY-OVERCOUNT** : dedup par (pid,hash) SI une UX future affiche le
  compteur comme « N apps decouvrables ».
- **REVISION-HOME-DURABILITY** + **OBSERVED-FORGED-IDS** : surveiller au S76
  (mode deploiement sans home pinne ; publisher-binding observed).
- **UX-ARRIVAL-PLAN-INSCRIPTION** : inscrire formellement la surface UX-ARRIVAL
  (registre observed + from_subscribed + split arrival) dans sprint76_audit_plan
  comme track additionnel, marque couvert par le present audit.

**Externes inchanges (a reporter tels quels)** : P2-A-1 rand (exemption
upstream), P2-AUDIT-2 iroh pre-release (pin 0.98), T-NN+2 iframe Rust-wasm
(§P34), P3-OS-1 operator_server. **LT-2** : trigger ARME + dry-run prive FAIT
(trace G), flip publie = decision PO. Aucun n'atteint 3 reports sans exemption.

---

## 8. P3 laisses sans action

- **DOUBLE-BROADCAST-PUBLISH** : double-broadcast au publish (live + Outbox
  re-mint) — idempotent, dedup recepteur par project_id, pre-existant. Nit.
- **BLOBSERVE-DIALS-AMPLIF** : N*8 dials sans dedup in-flight — route
  loopback-authenticated, cache anti-amplification = durcissement post-launch.
- **RE-DRIVE-ON-INGEST** : fenetre morte 1er boot, remede operateur documente
  (restart). Carry S76 (conception).
- **HOTFIXES-HORS-CODEX** : hotfixes Cas D hors adversarial Codex/preflight
  (substitue par verif live cross-machine, decision PO) — fixes verifies sains.
- **BROWSE-NOT-BYTE-IDENTIQUE-TIP** : `from_subscribed` post-S75 additif
  serialize-only tolere Zod — la propriete byte-identique tenait PENDANT S75
  (D/F/G), pas au tip exact (UX-ARRIVAL hors-sprint).
- **DIVERS-NITS** : nommage test plan-vs-code consigne, Codex Phase A FR
  (CONFIRME vs CONFIRMED), coquille `DOMAIIN_NODE_DIRECTORY_V1` (double I) dans
  verification.md (code correct), dedup curator/direct 2 cartes 2 sections, pas
  de TTL recepteur sur les direct entries, ANCHORS_SCHEMA_VERSION=1 (schema
  persistance pas wire), incoherence cosmetique nom test Zod. A nettoyer
  opportunement au menage S76. Aucun impact code.

---

## 9. Notes on audit completeness

- **Track A (re-run des suites)** : non re-execute par cet auditeur — joue
  separement par le main thread en serial (Windows nextest --workspace, Docker
  Linux canonique, web Vitest/coverage/size). Le self-report annonce Windows
  1755/1755, Docker 1759/1759, web 367/367 coverage 87.17/79.01/85.92/88.5,
  size 6/6. L'audit a verifie statiquement la PRESENCE et la nature load-bearing
  des tests cles cites (rows 7/9/10/11/12/13/14/15) dans le code, pas leur
  passage runtime. **Note importante** : apres landing des 2 fix duress (§6),
  la fail-fast doit etre re-jouee (delta attendu +2 tests).
- **Docker canonique** : gate AVANT PUSH (`feedback_wsl_before_push`), pas
  avant commit. Non re-execute en Phase 0 ; consigne pour le push (decision PO).
- **Live cross-machine (acceptance survives-VPS-death + C6)** : NON re-execute
  (requiert SSH mac 192.168.1.53 + vps 135.181.42.188 + systemd). L'audit a
  verifie la TRACE consignee `.git/S75_PHASE_G_ACCEPTANCE.md` (horodatee,
  node_ids reels, render HTTP 200, marqueurs d'echec honnetes) + corrobore les
  3 claims cardinaux dans le CODE (default vides compiles ; boot driver
  config-driven 0 hardcode ; `fetch_falls_back_to_seeder_when_anchor_offline`
  ancre morte → seeder deterministe). Conforme au §0 du plan (« l'audit verifie
  la trace, ne re-exige le live que si absente/incoherente » — elle est ni l'un
  ni l'autre). Le skeptic #4 a refute le candidat « pas de trace re-jouable ».
- **Surfaces hors cycle gate standard** : le mini-cycle UX-ARRIVAL `e980d7e` +
  3 hotfixes Cas D (`fdb4fb1`/`173426e`/`38c5578`) ont ete livres APRES la
  verification S75 (`8b53c38`), sans round review/Codex standard (process lean
  assume PO, substitue par verif live PC/Mac/VPS). L'audit les a couverts par
  analyse statique independante (tracks UX-ARRIVAL/I/J/HARDENING) — locks tiennent,
  securite saine — et confirme que ces surfaces doivent etre INSCRITES au
  sprint76_audit_plan pour tracabilite (P2 UX-ARRIVAL-PLAN-INSCRIPTION).
- **Couverture des tracks** : 13 tracks du plan + G1 presence tous couverts.
  Les 4 candidats P1 du §6 explicitement tranches (1 confirme, 3 refutes).
  Aucune zone du diff `0e2fb6b..38c5578` laissee non auditee.
