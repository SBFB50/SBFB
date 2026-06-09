# Sprint 74 — Kickoff (Atelier fork + programme Disponibilite/Hosting complet — pin local + seed cross-noeud authentifie)

**Ecrit** : 2026-06-07 (post-audit gate S73 PASS `2fe3b30`).
**Type** : **sprint PAIR** (feature lourde) **a tres forte charge** — combine
**(1) l'atelier fork** (chercher→forker→editer→REDEPLOY sous sa propre
identite noeud, roadmap v5 S74) **et (2) le programme Disponibilite/Hosting
complet**, dont le **cross-noeud (ex-LT-5 redundancy persistence)** est
**TIRE EN AVANT** depuis le post-launch par **directive PO 2026-06-07**
(« faire tout pour ce sprint, les prochains ont d'autres objectifs » —
S75 GPU et S76 sharding sont engages ailleurs). Une **phase dette dediee
(Phase F)** absorbe les carries audit S73 + la dette coverage T14.
**Tip master d'entree** : `a53b9f6` (`fix(shell): auto-register the
same-origin daemon as the default coordinator`). 9 hotfixes Cas D locaux
non pousses depuis l'audit gate S73 (`47b8c59`..`3b7ef54` daemon #1-#8 +
`a53b9f6` shell). HEAD = `a53b9f6`.
**Phase 0 audit Sprint 73** : **DEJA JOUE** — `2fe3b30`
(`chore(planning): Sprint 73 audit findings — PASS (S74 Phase 0)`),
verdict **PASS** (0 P0, 0 P1, 14 P2, 8 P3). **Aucun fix `fix(sprint73)`
requis** ; les 14 P2 sont routes au plan S74 (§6), pas re-implementes en
Phase 0.
**Version archive** : v2.1 — « Protocole Neutre + Factory/RRV » (OPEN). S74
continue le meme arc (3.5 Factory Complete Vision, roadmap v5), aucune
release publiee depuis → **reste v2.1**. Migration S73 `active/`→
`archive/v2.1/` **deleguee au main thread** apres ce kickoff (NE PAS migrer
ici).
**Roadmap source** : `.planning/roadmap_v5_factory_complete_vision.md`.
Sprint **4 sur 6** (S71-S76), Arc 3.5 « Factory Complete Vision ».

---

## Sources WebSearch + context7 + code consultees (pre-gel)

Recherche G9 effectuee AVANT de figer D1..D5, surtout sur le **protocole
« demande de seed » cross-noeud authentifie**. Dates absolues, URLs, code
file:line.

### OSS prior art — seed / replication / approbation de pair (D1, D3, D4)

| Source | Date | Finding cle |
|--------|------|-------------|
| radicle.dev/guides/seeder + /guides/protocol (Heartwood) | 2024-2025 | **Seeding policy** par noeud = liste de repos qu'il replique ; **clone/init met a jour la policy** (auto-seed de ce qu'on heberge). **Delegates** (cles dans l'identity doc + threshold de signatures) = autorite de la version canonique ; **un seeder n'est PAS un delegate** — il replique sans pouvoir signer. `--scope followed` borne les annonces qu'on recoit aux delegates + suivis. **= seeder ≠ co-auteur cable dans le protocole**, exactement l'invariant SBFB « heberger ≠ publier ». |
| ipfscluster.io/documentation (CRDT pinset + allocations) | 2021-2025 | Pinset = **CRDT Merkle-DAG** (eventual consistency, go-ds-crdt) ; `replication_factor_min/max` + **allocations** (quels pairs epinglent) ; une sortie de pair sous `replication_factor_min` **re-declenche une re-allocation** (failover). **Modele de reference pour le compteur multi-seed + re-allocation** — mais c'est un **reglage numerique** : pour un non-technique, on l'**abstrait** (« copies de secours », pas `replication_factor=3`). |
| tailscale.com/kb/1084/sharing + /kb/1388 + /kb/1464 | 2024-2025 | **Invite link** single-use OU reusable (≤1000), **expire 30j**, **revocable** depuis la console. Les ACL du partageur **s'appliquent toujours** au pair invite. **Quarantine par defaut** : une machine partagee recoit les connexions entrantes mais **ne peut pas en initier** → accepter un partage n'expose pas son reseau. **= modele lien d'invitation revocable + quarantine** pour « inviter un pair de confiance a seeder ». |
| docs.syncthing.net/users/introducer (device approval) | 2024-2025 | Connecter un pair exige une **approbation explicite** (sauf Introducer) ; l'echange liste les **dossiers mutuellement partages** ; un retrait cote introducer se propage a la prochaine connexion. **= approbation cote pair AVANT fetch+pin** (le pair distant doit dire oui). |
| ipshipyard.com/blog/2025-dht-provide-sweep + ipfs/kubo#9389 | 2025 / 2022 | Provider records **republies toutes les 22h** (expiration 48h) ; 2025 « Provide Sweep » etale le re-announce. **= re-annoncer est un cout recurrent reel** ; un seeder qui reboot DOIT re-annoncer pour rester joignable (valide le pattern outbox replay au boot + un re-announce periodique). |

### Code SBFB lu (file:line) — l'infra existe deja

| Fichier:ligne | Finding cle |
|---|---|
| `crates/nexus-core-rs/src/blobs.rs:77-88` | `add_bytes` **pin deja** via tag nomme (hex du hash) « so the store does not garbage-collect it ». **`fetch_ticket:140-163`** telecharge (Downloader) mais **ne tag pas** le blob fetche → un blob seede par un pair distant n'est PAS protege du GC tant qu'on n'ajoute pas le tag. **Gap pin-cross-noeud isole et chirurgical.** |
| `crates/nexus-core-rs/src/node.rs:48-49,313-324` | `FsStore` (redb) quand `data_dir` set → **blobs persistent across reboot** ; `MemStore` sinon. Le daemon prod utilise FsStore. **La persistance disque est deja la** — il manque le **tag/protect + re-annonce** au boot. |
| `crates/nexus-core-rs/src/node.rs:341-344` | Router multiplexe **3 ALPN** (`BLOBS_ALPN`/`GOSSIP_ALPN`/`DOCS_ALPN`). Un `request_seed` cross-noeud = soit un **nouvel ALPN** dedie (req/resp QUIC), soit une **op feed gossip** (raw-op `SeedAnnounced`). D1 tranche. |
| `crates/nexus-shell-daemon/src/runtime.rs:1750-1771` | `restore_browse_from_outbox` : au boot, **re-ingere les annonces persistees** (PowEnvelope decode SANS re-verif PoW — donnee locale fiable) → repeuple Browse + re-indexe. **Pattern de re-annonce-au-boot prouve (#7), directement reutilisable** pour la re-annonce de seed. |
| `crates/nexus-shell-daemon/src/deploy.rs:445-463` | **Faux-vert NAT** : self-publie → `status: BrowseStatus::Reachable, last_probed_at: None` (« we never dial ourselves »). Honnete pour le **self**, mais affiche « En ligne » sans probe externe. D4 tranche le libelle. |
| `crates/nexus-shell-daemon/src/deploy.rs:423-443` | `/deploy` persiste deja l'annonce a l'outbox (hotfix #8 `3b7ef54`, `publish_announcement` = helper canonique broadcast→persist→index→cache). **La voie de re-deploy/fork-redeploy passe par ce helper.** |
| `crates/nexus-coordinator-rs/src/db.rs:10,228-229,302-303` | Migrations via `rusqlite_migration` `M::up` (`user_version`). M16 `result_text`, M17 FTS5 triplet. **Prochaine = M18** (table locale `keep_online`, schema **local** pas wire — type M16/M17). |

### Versions deps confirmees (lockfile, INCHANGEES S74)

`iroh 0.98`/`iroh-docs 0.98`/`iroh-gossip 0.98`/`iroh-blobs 0.100` (pin
gele), `rusqlite 0.36` (SQLite 3.49.2 bundled), `rusqlite_migration`,
`axum 0.8.9`, `reqwest 0.12.28`, `serde_json` (raw-op `Value`). Front
`web/` : React + Vite + TS + Tailwind + shadcn (Sheet) + Zustand + React
Query. **Aucune nouvelle dep crypto** : l'auth cross-noeud reutilise la
signature Ed25519+JCS (`canonical.rs`) deja en place ; le seed re-annonce
reutilise le PoW envelope + l'outbox.

**Decision crypto/spec nouvelle ?** Le seul element a **specifier** est le
**`SeedRequest` cross-noeud authentifie** (preuve Ed25519 que c'est MON
noeud — ou un pair invite — qui demande). C'est de la **composition** de
primitives existantes (Ed25519+JCS+domain constant, comme `Task`/`Result`),
**pas une nouvelle primitive**. La checklist `[DETER]` Rust-first +
crypto-spec s'applique a D1 (cf. `sprint74_design_review.md`).

---

## §1 Constat d'entree

### §1.1 D'ou on part

S73 a CLOSE (PASS, `2fe3b30`) en cablant la **recherche reseau** : reindex
FTS5 a chaud, `SearchResult` enrichi du **triplet provenance**
(`repo_url`+`commit_sha`+`archive_hash`+`provenance_hash`), barre de
recherche shell. On peut maintenant **trouver** une app sur le reseau. S74
ferme la boucle produit centrale de SBFB :

> On **cherche** (S73) → on **forke** dans l'atelier → on **edite** (agent
> Operator) → on **REDEPLOY sous sa propre identite noeud** → et on
> **garde l'app en ligne**, y compris quand le PC est eteint, via un
> **pair de seed** — sans jamais recentraliser ni re-attribuer l'auteur.

Le constat de cartographie : **l'infra existe a ~85-90 %**. Le triplet
provenance (S73) alimente le fork ; `deploy.rs::publish_announcement` (#8)
est le helper canonique de re-deploy ; `restore_browse_from_outbox` (#7) est
le pattern de re-annonce-au-boot ; `blobs.rs::add_bytes` pin deja via tag ;
`FsStore` persiste deja les blobs. Les manques sont **(a)** le **cablage
fork** (clone `repo_url@commit` ou reconstruction blob → nouveau workspace →
re-deploy) ; **(b)** le **panneau Disponibilite** front sur primitives
existantes ; **(c)** le **pin local persistant** (`keep_online` + tag/protect
+ re-annonce boot) ; **(d)** le **seed cross-noeud authentifie** (le seul
vrai morceau de fondation — `SeedRequest` + tag du blob fetche + approbation
de pair + re-annonce persistante distante). **(a)(b)(c) = cablage ; (d) = la
seule fondation, et c'est le pull-forward de LT-5.**

### §1.2 Ancrage roadmap v5 (Arc 3.5) + reconciliation du pull-forward LT-5

Arc 3.5 « Factory Complete Vision » (roadmap v5, CANON), 6 sprints S71-S76.
Position : **sprint 4/6**.

```
S71 assainir+securite+reconciliation (DONE)
  └─ S72 provider routing (DONE)
       └─ S73 recherche reseau cablee (DONE)
            └─ S74 atelier fork + Disponibilite/Hosting COMPLET  ← ICI
                 │   (+ ex-LT-5 redundancy persistence TIRE EN AVANT)
                 └─ S75 GPU partage PROUVE cross-machine
                      └─ S76 STRETCH: sharding pipeline
```

**Reconciliation du pull-forward de LT-5 (load-bearing — PO 2026-06-07)** :
- La roadmap v5 §3 borne S74 a « atelier fork + templates + packaging » et
  range la **redundancy/seed cross-noeud (LT-5)** en **post-v1.0 / S75**
  (ROADMAP_COMMITMENTS S73 : « LT-5 = 1er deploiement multi-worker OU v1.0
  go-live, non declenche »).
- **La directive PO 2026-06-07 tire LT-5 en avant dans S74** car S75 (GPU
  cross-machine) et S76 (sharding) sont **engages sur le compute**, pas sur
  le hosting. Le hosting/disponibilite est **orphelin** de tout sprint aval
  → si on ne le fait pas en S74, il glisse au-dela de v2.1.
- **COUT/RISQUE assume et signale** : un seul sprint qui porte
  **atelier-fork + Disponibilite front + pin local + protocole seed
  cross-noeud authentifie + approbation de pair + re-annonce distante
  persistante** est **gros** (probablement 7 phases A-G). Le **cross-noeud
  complet (d)** est le segment le plus lourd et le plus risque (nouveau
  `SeedRequest`, tag de blob fetche, NAT, approbation, persistance distante).
  **Decoupage propose §5 :** front + fork + pin local (Phases A-D) =
  livrables surs ; cross-noeud (Phases E-F) = le segment a risque de
  debordement. **D5 + Checkpoint §11 laissent au PO l'arbitrage de
  l'ampleur reelle du cross-noeud** (full E2E 2-noeuds vs « invitation +
  fetch+pin + re-annonce » sans le registre multi-seed complet vs design
  prouve + slice). Garde-fou design : **JAMAIS un faux bouton actif** (un
  cran cross-noeud non livre reste « Bientot », visible mais inerte).

**Dependances aval** : S74 redeploie sous identite locale (reutilise S73
triplet) ; le seed cross-noeud (S74) est **distinct** du compute
cross-machine (S75) — l'un replique des **blobs** (disponibilite), l'autre
route des **taches** (calcul). S74 NE livre PAS le GPU cross-machine (S75),
NE livre PAS le sharding (S76), NE livre PAS un editeur Monaco (PO-9 :
l'agent edite).

### §1.3 Compteurs tests entree (tip `a53b9f6`)

| Suite | Count |
|---|---|
| Rust nextest (canonique CI Linux) | **1570** (Windows natif = 1566 ; ecart +4 = `#[cfg(unix)]` structurel) |
| Vitest (`web/`) | **294** (289 audit S73 + 5 `bootstrap.test` du hotfix `a53b9f6`) |
| Vitest `factory-operator` | 7 |
| size-limit | 6/6 |
| **Total** | **~1877** |

**Re-mesure obligatoire** au `plan.md §1` sur le SHA reel post-kickoff (les
9 hotfixes Cas D locaux ont deja bouge le tip ; le compte Rust reste 1570
canonique, le shell hotfix a ajoute +5 Vitest deja comptes).
**DECOUVERTE T14 a inscrire** (carry coverage) : le gate
`npm run test:coverage` est **ROUGE pre-existant** (79.9/71.89/78.68/78.73
vs seuils 85/90/78/85 ; `FileUploadBlock` 35 % trace dans `vitest.config.ts`
depuis S9), **masque par `| tail`** dans le fail-fast. CI-enforced
(verify.sh step12 + GHA ci.yml[7]) mais les commits locaux ne sont pas
pousses → GHA pas encore rouge. **A traiter Phase F.**

### §1.4 Pre-launch protocol policy (rappel)

Rien n'est pousse vers origin (**37+ ahead**, rien pousse). **Reconciliation
locale libre.**

- `FEED_FORMAT_VERSION = 1`, `PROJECT_ANNOUNCEMENT_VERSION = 1`,
  `TASK_FORMAT_VERSION` restent **inchanges**. La nouvelle op
  **`SeedAnnounced`** (registre de seeders, D3) est un **raw-op
  `serde_json::Value`** ajoute au feed → **NE bump PAS** `FEED_FORMAT_VERSION`
  (CLAUDE.md:354-366 ; les noeuds anciens stockent/propagent l'op inconnue
  sans l'interpreter). Le `SeedRequest` cross-noeud (D1) est un **message
  req/resp** (ALPN dedie OU op feed selon D1), **pas** un changement
  d'enveloppe feed.
- Migration SQLite **M18** (table locale `keep_online` : `project_id`,
  `enabled`, `pinned_at`, `archive_hash`) = schema **local daemon**, pas un
  wire format. Reconstructible (la verite = l'outbox + provenance + feed).
- `#[serde(default)]` legitime pour la robustesse runtime (un seeder distant
  qui envoie un `SeedAnnounced` minimal → champs omis a zero, pas 422).
- Pas de tolerant decoder multi-version. Pas de test « legacy decode ».

---

## §2 Goal

> Sprint 74 ferme la **boucle produit** de SBFB et livre le **programme
> Disponibilite/Hosting complet**. **(1) Atelier fork** : depuis un hit de
> recherche (triplet S73), forker un projet reseau (clone `repo_url@commit`
> via forge, **ou** reconstruction depuis le blob `archive_hash` en repli —
> PO-5) dans un **workspace cible distinct du repo nexus**, l'editer via
> l'agent Operator, et **REDEPLOY sous sa propre identite noeud** (helper
> canonique `publish_announcement`, provenance re-signee — seeder ≠
> co-auteur, l'auteur du fork c'est MON noeud). **(2) Disponibilite
> continue** (design **D-DISPO** fige, `s74_disponibilite_ux_design.md`) :
> publish gagne **zero champ hote** (acte d'identite local signe + ligne de
> verite + carte succes hashs replies) ; un **panneau lateral
> « Disponibilite »** sur la fiche app rend VISIBLES les 3 invariants
> (AUTEUR scelle / ETAT probe reel / QUI-LA-GARDE) ; le toggle **« Garder en
> ligne »** devient fonctionnel (pin **local** persistant : table `keep_online`
> M18 + tag/protect du blob + re-annonce au boot via le pattern outbox #7).
> **(3) Seed cross-noeud (ex-LT-5 tire en avant)** : un **`SeedRequest`
> authentifie** Ed25519 permet a un pair (mon VPS / pair de confiance, via
> **invitation revocable + approbation cote pair**) de **fetch+tag+
> re-annonce-persistante** un blob — la provenance de l'AUTEUR reste
> intacte ; un registre **`SeedAnnounced`** (raw-op) alimente un compteur
> communautaire. **5 verrous anti-recentralisation cables UI** (zero champ
> cible/hote, redondance additive, « Mon serveur » possessif, seeder ≠
> autorite, suggestion declenchee par l'etat « Hors ligne »). **Le faux-vert
> NAT** (`deploy.rs:456`) est **honnete** (« En ligne (vu de ton noeud) »
> pilote, ou probe externe — D4/Checkpoint).
> **Critere SMART : 100% des rows fail-fast vertes au
> `sprint74_verification.md §Fail-fast checklist`, mesure binaire au Phase G
> wrap-up.** La fail-fast checklist (cf. plan §Fail-fast) EST la source of
> truth mesurable du goal. **Garde-fou ampleur** : si le cross-noeud complet
> (Phases E-F) deborde, l'atelier-fork (A-C) + Disponibilite front + pin
> local (D) restent livrables — chaque cran cross-noeud non livre reste
> « Bientot » (jamais un faux bouton actif), arbitre au Checkpoint §11.

---

## §3 Phase 0 — Audit gate Sprint 73

**DEJA JOUE.** `sprint73_audit_findings.md` (`2fe3b30`), 9 tracks A-I +
reconciliation off-sprint, verdict **PASS**. Orchestration multi-agent
(11 agents, ~1.53M tok, anti-anchoring). Resume :

- **0 P0, 0 P1** — aucun `fix(sprint73)` requis avant Phase A. Les **3
  candidats P1 TOUS REFUTED** (B.1 db.rs:443 seul writer prod post-guardrail ;
  worker-pump/runtime.rs:1906 exemption P54 genuine + vert ; wire/
  SearchManifest 4 variantes intactes).
- **14 P2** routes S74 (§6) : quorum guardrail zombie (B.2), ReleasePublished
  non-indexe (FRESHNESS), rowid partition browse/feed tripwire (ROWID), invariant
  `is_open_source⇒provenance_hash` non re-applique au browse (B.6), M17
  boot-recovery warn-only (H.1), reconstructibilite browse-rows (H.2), rate-limit
  search residual + THREAT_MODEL §11 stale (D.1), isHttpsUrl mono-vecteur +
  3 ancres pre-existantes (B.5), 3 tests C/D sous-asserent (E.3),
  SearchResultsView sans `isError` skeleton infini, OFF-SPRINT-2 deploy per-app
  sans test, OFF-SPRINT-2b /publish+gossip gardent node_id, system_prompt vide
  regles inertes (B.4, PATTERNS), hot-upsert warn-only sans catch-up (C.4,
  PATTERNS).
- **8 P3** : nits (runtime-1906 exemption, baseline 1544, F-HEADER espacement,
  last_err token-free OK, etc.).
- **9 hotfixes Cas D off-sprint** (post-audit, `47b8c59`..`3b7ef54` +
  `a53b9f6`) : recherche/rendu/identite/EXECUTE-reseau/freshness/deploy-outbox
  + shell auto-add coordinateur — **a connaitre, le rename « coordinateur »
  s'appuie dessus** (carry G7).

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Protocole seed cross-noeud : `SeedRequest` Ed25519-signe + ALPN dedie req/resp (pas op feed pour la demande)

**Sources consultees** : Radicle seeder/protocol (seeding policy + delegates
≠ seeders), Tailscale sharing (invite revocable + quarantine), Syncthing
(approbation explicite de pair). Code lu : `node.rs:341-344` (Router 3 ALPN),
`blobs.rs:140-163` (`fetch_ticket` non-tagge), `canonical.rs` (Ed25519+JCS
deja la), `pow_gossip.rs`/`public_feed.rs:82-118` (raw-op feed).

**Retenu** : un **`SeedRequest` cross-noeud signe Ed25519+JCS** porte par un
**nouvel ALPN dedie** (`sbfb/seed/0`) ride par le Router iroh existant
(req/resp QUIC, comme blobs/docs). Le requesteur signe `{ project_id,
archive_hash, archive_ticket, requester_node_id, nonce, ts }` avec sa cle de
noeud (preuve que c'est **MON** noeud — ou un pair invite porteur d'un token
d'invitation, D4). Le pair distant **verifie la signature + l'invitation**,
**approuve** (cote pair, modele Syncthing — D4), puis `fetch_ticket` →
**tag/protect** le blob fetche (corrige le gap `blobs.rs:140` : le seeder
DOIT tagger pour skip-GC) → persiste l'intention seed (M18 `keep_online` cote
seeder) → re-annonce. **La demande passe par un ALPN req/resp, PAS par le
feed** : une demande de seed est un acte **point-a-point cible** (« porte
CETTE app sur CE pair »), pas un broadcast. **Le registre/etat multi-seed
(D3) est, lui, une op feed** `SeedAnnounced` (broadcast d'un fait observable).

**Rejete** :
- *Demande de seed via op feed broadcast* : un broadcast « seede-moi ca »
  est un appel a l'aide non-cible → surface Sybil/spam (ARES 2024) + ne porte
  pas l'auth point-a-point. La demande est **dirigee** vers un pair choisi.
  Rejete (la demande = ALPN ; le **fait** « X seede Y » = op feed D3).
- *Re-utiliser l'ALPN docs/gossip* : melange les contrats de protocole +
  pas de place pour le handshake d'approbation. Un ALPN dedie est propre.
  Rejete.
- *Auth par token partage simple (pas de signature)* : un token rejouable
  par tout porteur ne prouve pas l'identite du noeud demandeur. Ed25519+JCS
  (deja la) + nonce anti-replay. Rejete.

**Implications code** : NEW protocole `crates/nexus-shell-daemon/src/seed.rs`
(ou `nexus-core-rs` selon couche) : `SeedRequest`/`SeedResponse` types +
signature + handler ALPN ; `blobs.rs` (tag du blob fetche cote seeder —
corrige `fetch_ticket` ou ajoute `fetch_and_pin`) ; `node.rs:341-344`
(`.accept("sbfb/seed/0", …)`). **Zero wire feed** (ALPN distinct).

### D2 — Pin local persistant : table `keep_online` (M18) + tag/protect blob + re-annonce au boot (pattern outbox #7)

**Sources consultees** : IPFS reprovide 22h (re-annoncer = cout recurrent),
code `runtime.rs:1750-1771` (`restore_browse_from_outbox` re-annonce-boot
prouve), `blobs.rs:77-88` (`add_bytes` tag deja), `node.rs:48,313-324`
(FsStore persiste), `db.rs:228-229,302-303` (M16/M17 pattern).

**Retenu** : le toggle « Garder en ligne » (D-DISPO, ON par defaut pour mon
app) ecrit une intention `keep_online` dans une **table locale M18**
(`project_id` PK, `enabled` bool, `archive_hash`, `pinned_at`). Au **deploy**
(self) le blob est **deja tagge** (`add_bytes`) ; le pin ajoute **(a)** la
ligne `keep_online=true`, **(b)** la garantie de **skip-GC** (tag conserve
tant que `enabled`), **(c)** la **re-annonce au boot** : `restore_*` lit
`keep_online` (en plus de l'outbox) et re-broadcast l'annonce pour les apps
gardees en ligne. OFF = `enabled=false` → tag retire (le blob peut etre
GC'd) + arret de re-annonce (UX : « stockee mais plus diffusee — disparait
si aucun autre pair ne la garde »). **Pin LOCAL uniquement** dans cette
decision (le cross-noeud = D1/D3). **`rebuild`/re-annonce best-effort**
(warn-only, comme C.4) — un echec transitoire ne bloque pas le boot.

**Rejete** :
- *Pin = champ dans l'outbox existant* : l'outbox = annonces a rejouer
  (semantique broadcast), pas un etat de politique de retention. Une table
  dediee `keep_online` est l'etat de verite (modele IPFS pinset). Rejete.
- *Skip-GC implicite « tout est garde »* : sans opt-OUT explicite, on ne
  peut jamais liberer l'espace + on ment a l'utilisateur (« plus diffusee »).
  Le toggle EST l'etat. Rejete.
- *Re-annonce sur timer 22h type IPFS maintenant* : le re-announce-au-boot
  (outbox #7) + le NeighborUp replay couvrent le pilote ferme ; un timer
  periodique est un raffinement post-launch (scope cut). Rejete pour S74.

**Implications code** : `db.rs` (M18 `keep_online` + getters/setters) ;
`deploy.rs`/`publish.rs` (set `keep_online=true` au self-deploy) ;
`runtime.rs:1750-1771` (`restore_*` lit `keep_online` + re-annonce) ;
`blobs.rs` (tag conserve / retire selon `enabled`) ; route HTTP
`POST /api/daemon/keep-online` (toggle, loopback auth). **Zero wire** (M18
local).

### D3 — Registre de seeders : op feed `SeedAnnounced` (raw-op) + compteur « Toi + N pairs » (pas de nombre exact fragile)

**Sources consultees** : IPFS Cluster (compteur de replicas + allocations),
Radicle (announce scope par interet), `public_feed.rs:82-118` (raw-op 4
variantes, `FEED_FORMAT_VERSION=1`), CLAUDE.md:354-366 (pre-launch raw-op).

**Retenu** : quand un noeud (self ou pair distant) commence a seeder une app,
il broadcast une op feed **`SeedAnnounced`** (raw-op `serde_json::Value` :
`{ project_id, seeder_node_id, archive_hash, ts, sig }`) → **NE bump PAS**
`FEED_FORMAT_VERSION`. Le panneau Disponibilite agrege les `SeedAnnounced`
recents (TTL, comme les provider records) → **etat multi-seed** + compteur.
**Compteur best-effort « Toi + d'autres pairs »** (pas de nombre exact en
S74) : un nombre exact dans un reseau gossip eventually-consistent est
**fragile** (un pair tombe → le nombre ment) ; « Toi + N pairs (vus
recemment) » avec TTL est honnete. **D5/Checkpoint** laisse au PO le choix
nombre exact vs best-effort. La **re-allocation/failover** (IPFS Cluster
`replication_factor_min`) est **hors-scope S74** (vision, abstraite, jamais
un reglage numerique pour un non-technique).

**Rejete** :
- *Bump `FEED_FORMAT_VERSION` pour `SeedAnnounced`* : viole la pre-launch
  policy (raw-op extensible). Rejete.
- *Nombre exact de seeders affiche* : fragile en gossip eventually-consistent
  (le nombre ment quand un pair tombe sans retract). Best-effort + TTL.
  Rejete (sauf arbitrage PO Checkpoint).
- *Re-allocation auto type IPFS Cluster* : reglage numerique +
  failover automatique = sur-ingenierie pour un pilote ferme + UX
  non-technique. Vision, hors-scope. Rejete pour S74.

**Implications code** : `public_feed.rs` (helper construire/valider
`SeedAnnounced` raw-op — **pas** une 5e variante d'enum, un raw-op) ;
`feed_sync.rs` (ingest `SeedAnnounced` → agregat seed) ; `browse.rs`/
nouveau `seed_registry` (etat multi-seed + TTL) ; front panneau « Qui la
garde en ligne » + « Copies de secours » (D-DISPO §5). **Zero bump wire**.

### D4 — Faux-vert NAT + invitation/approbation de pair : libelle honnete pilote + invite revocable + approbation cote pair

**Sources consultees** : Tailscale (invite single-use/reusable revocable
30j + quarantine), Syncthing (approbation explicite), code `deploy.rs:445-457`
(self→Reachable last_probed_at:None), `browse.rs` probe (TTL+quorum/DNS).

**Retenu** (3 volets) :
1. **Faux-vert NAT** : pour les apps **self-publiees**, afficher
   **« En ligne (vu de ton noeud) »** (libelle honnete pilote) plutot que
   « En ligne — joignable par tous » tant qu'aucun **probe externe / signal
   quorum tiers** ne confirme la joignabilite. **D5/Checkpoint** : libelle
   honnete pilote des S74 (simple, zero infra) **vs** attendre le probe
   externe (plus juste, plus de travail). **Recommandation : libelle honnete
   pilote** (le probe externe est un raffinement).
2. **Invitation de pair** : lien d'invitation **revocable** (modele
   Tailscale — single-use ou reusable borne, expiration) qui porte un token
   signe autorisant un pair tiers a emettre un `SeedRequest` (D1) pour MES
   apps. Le requester non-self prouve l'invitation, pas l'identite de mon
   noeud.
3. **Approbation cote pair** : le pair **destinataire** d'un `SeedRequest`
   **approuve explicitement** (modele Syncthing — « ce noeud te demande de
   garder X en ligne ; accepter ? ») avant fetch+pin. Pas de seed
   silencieux impose.

**Rejete** :
- *« En ligne — joignable par tous » pour le self sans probe* : c'est le
  faux-vert actuel, ment a l'utilisateur (NAT). Rejete (libelle honnete).
- *Invitation = partage de la cle de noeud* : exposerait l'identite signataire.
  Un token d'invitation distinct, revocable, borne. Rejete (Tailscale model).
- *Seed automatique sans approbation* : un pair ne doit jamais se faire
  imposer de stocker/diffuser du contenu. Approbation explicite. Rejete.

**Implications code** : `browse.rs`/front (libelle « vu de ton noeud ») ;
NEW `invite` seed (token signe revocable — reutilise le pattern invite
existant `nexus-coordinator-rs` si applicable) ; handler `SeedRequest` (D1)
gate sur approbation ; front « Inviter un pair de confiance » +
notification d'approbation cote pair. **Ampleur arbitree D5/Checkpoint.**

### D5 — Ampleur reelle livrable S74 : front+fork+pin-local surs (A-D) ; cross-noeud (E-F) borne, jamais un faux bouton actif

**Sources consultees** : directive PO 2026-06-07 (« tout pour ce sprint ») +
risque scope (un seul sprint = atelier + dispo + pin + protocole seed) +
design D-DISPO §5/§9 (« JAMAIS un faux bouton actif »).

**Retenu** : decoupage explicite en **3 segments de confiance** :
- **Segment 1 — SUR (Phases A-D)** : atelier fork (clone/reconstruction →
  workspace → re-deploy) + panneau Disponibilite front (lecture, primitives
  S73) + **pin local persistant** (D2, M18 + tag + re-annonce boot). **100 %
  livrable**, zero protocole nouveau cross-noeud.
- **Segment 2 — A RISQUE (Phases E-F)** : seed cross-noeud (D1 `SeedRequest`
  ALPN + tag blob fetche + D4 invitation/approbation + D3 `SeedAnnounced`
  registre). **C'est le pull-forward de LT-5** — le plus lourd. Borne :
  livrer **(a)** le protocole `SeedRequest` authentifie + fetch+tag+pin cote
  seeder + **(b)** la re-annonce persistante distante au boot + **(c)** un
  **E2E 2-noeuds reel** (le pair distant garde l'app en ligne apres reboot,
  provenance auteur intacte). Le **registre multi-seed + compteur** (D3) est
  le cran le plus aval — si E-F debordent, il reste « Bientot ».
- **Segment 3 — VISION (jamais S74)** : re-allocation/failover IPFS-Cluster,
  timer re-annonce 22h, probe externe NAT complet, page « Mes seeds »
  complete. Scope cuts.

**Regle d'or design** : **chaque cran non livre reste un bouton « Bientot »
visible mais inerte** (D-DISPO §5 (5) : « jamais un faux bouton actif »). Un
toggle « Garder en ligne » lecture-seule (Phase A) devient fonctionnel
(Phase D pin local) ; « Inviter un pair » reste « Bientot » jusqu'a ce que
E-F le cablent. **L'arbitrage de l'ampleur exacte du Segment 2 est rendu au
PO au Checkpoint §11.**

**Rejete** :
- *Tout livrer en bloc sans segmentation* : risque de sprint qui ne ferme
  pas (rien de fini). La segmentation garantit qu'A-D ferment meme si E-F
  glissent. Rejete.
- *Faux boutons actifs « ca marchera bientot »* : ment a l'utilisateur (v1 =
  prod). « Bientot » inerte honnete. Rejete.
- *Cross-noeud en design-only (rien de code S74)* : la directive PO veut le
  cross-noeud **livre** en S74. Design-only = dernier recours si E-F
  debordent (arbitrage PO). Rejete comme defaut.

**Implications** : le plan §5 phase **A-D = surs**, **E-F = cross-noeud
borne**, **G = wrap-up**. Chaque phase a un critere binaire ; A-D livrables
independamment de E-F.

---

**Acknowledged review findings (G1)** :

Scoring (renseigne par `sprint74_design_review.md`) :
**D1 ⚠️, D2 ✅, D3 ⚠️, D4 ⚠️, D5 ✅.**
Rigor signal G4 satisfait (**3 ⚠️ sur 5** — sprint a forte composante
fondation cross-noeud, la cible gold 1-2/5 est depassee de façon **assumee**
car D1/D3/D4 introduisent du protocole/produit nouveau, pas du cablage).

- **D1 ⚠️** : **nouveau protocole cross-noeud** (`SeedRequest` ALPN
  Ed25519). C'est de la **fondation**, pas du cablage. Decision :
  **acknowledge + adjust** — composition de primitives existantes
  (Ed25519+JCS+nonce, comme `Task`/`Result`), pas de nouvelle primitive
  crypto ; ALPN req/resp comme blobs/docs ; checklist [DETER] crypto-spec a
  satisfaire au preflight Phase E. Le ⚠️ trace que c'est le segment risque.
- **D3 ⚠️** : op feed `SeedAnnounced` + compteur. Decision :
  **acknowledge + adjust** — raw-op (zero bump wire), compteur best-effort
  (pas de nombre exact fragile), re-allocation hors-scope. Le ⚠️ trace que
  c'est le cran le plus aval (peut rester « Bientot » si E-F debordent).
- **D4 ⚠️** : 3 volets produit (faux-vert + invitation + approbation), 3
  arbitrages PO ouverts. Decision : **acknowledge + arbitrage Checkpoint** —
  recommandations posees (libelle honnete pilote, invite revocable Tailscale,
  approbation Syncthing), PO tranche l'ampleur.

---

## §5 Plan Phase outline A..G

### Phase A — Disponibilite front (D-DISPO segment 1) : publish nettoye + panneau lecture + rename « coordinateur »→« noeud »

100 % front sur primitives S73 existantes (probe `browse.rs` humanise +
`verifyQuery` provenance + StatusPill/Dot). Carte succes publish (remplace le
`<dl>` Hash/Provenance/Commit de `Deploy.tsx:151-174`) + ligne de verite sous
CTA (hashs replies « Details techniques ») ; **0 champ hote** au publish.
Bouton « Disponibilite » (remplace le badge `blob:<hash>`) → **Sheet lateral**
shadcn : Section AUTEUR scellee / Section ETAT (mapping reachable/unreachable/
unknown) / Section QUI-LA-GARDE. Toggle « Garder en ligne » **lecture-seule**
(ON honnete). Rappel hors-ligne conditionnel (greffe A, declenche par l'etat)
+ placeholder « app tombee » → `/deploy` prerempli (greffe D). **Rename
« coordinateur »→« noeud »/« reseau »** (AppShell CoordinatorPicker,
`daemon.ts` var interne, nav, AddCoordinatorDialog, OnboardingEmpty) —
s'appuie sur le hotfix `a53b9f6`. Strings FR D-DISPO §6 exactes.
**Critere : `web/` tsc+lint+vitest+build+size+scan-en propres ; panneau
lecture rend AUTEUR/ETAT/QUI-LA-GARDE ; rename propage ; 0 champ hote ; 0
faux bouton actif.** G1 design_review present (gate Phase A).

### Phase B — Atelier fork backend : workspace cible + clone forge / reconstruction blob (PO-5)

Notion de **projet cible distinct du repo nexus** (`process::repo_root`
pointe nexus aujourd'hui, G17). Depuis le triplet S73 : clone
`repo_url@commit_sha` (forge) **ou** reconstruction depuis le blob
`archive_hash` (repli — fetch_ticket + unzip) → nouveau workspace atelier.
Pre-requis browse-indexing inscrits (audit S73) : **rowid partition
browse/feed** (C.3, AVANT tout browse-indexing prod) + **re-application de
l'invariant `is_open_source⇒provenance_hash`** au chemin browse (B.6).
**Critere : un hit search → workspace forke (forge OU blob) ; rowid partition
en place ; invariant provenance re-applique ; tests fork forge + fork blob.**

### Phase C — Atelier fork → REDEPLOY sous identite locale (helper `publish_announcement`) + boucle UI

`reseau→atelier→redeploy` : le workspace forke est re-deploye via le helper
canonique `deploy.rs::publish_announcement` (#8) → **provenance re-signee par
MON noeud** (seeder ≠ co-auteur ; le fork EST un nouvel acte d'auteur local).
Bouton UI « Forker dans l'atelier » + « La remettre en ligne » (app tombee,
greffe D → `/deploy` prerempli, **re-signature coherente fork** — arbitrage
D4/Checkpoint). Templates : confirmer static + static-reader (react/pyodide =
arbitrage PO-8 Checkpoint). **Critere : boucle chercher→forker→editer→redeploy
prouvee (test E2E mono-noeud) ; provenance du fork = identite locale ;
OFF-SPRINT-2/2b (deploy per-app + /publish+gossip node_id) traites.**

### Phase D — Pin local persistant (D2) : `keep_online` M18 + tag/protect + re-annonce boot + toggle fonctionnel

Migration **M18** `keep_online` (local) ; toggle « Garder en ligne »
**fonctionnel** (ON/OFF) via `POST /api/daemon/keep-online` ; tag/protect du
blob selon `enabled` (skip-GC) ; **re-annonce au boot** : `restore_*` lit
`keep_online` + re-broadcast (pattern outbox #7). Carries audit search/freshness
ici si peu couteux : **H.1 M17 boot-recovery non-silencieuse**, **H.2
reconstructibilite browse-rows**. **Critere : toggle ON→app re-annoncee au
boot (test reboot simule) ; OFF→tag retire + plus de re-annonce ; M18 verte ;
le pin survit a un redemarrage du daemon.**

### Phase E — Seed cross-noeud (D1+D4 segment 2) : `SeedRequest` ALPN Ed25519 + fetch+tag+pin cote seeder + invitation/approbation

**Le pull-forward LT-5 — segment a risque.** NEW protocole ALPN `sbfb/seed/0`
(`SeedRequest`/`SeedResponse` signes Ed25519+JCS+nonce) ride par le Router ;
le seeder **verifie sig + invitation (D4) + approuve** → `fetch_ticket` →
**tag/protect** (corrige `blobs.rs:140` gap) → persiste `keep_online` cote
seeder → re-annonce. Invitation revocable (D4 vol.2) + approbation cote pair
(D4 vol.3). Faux-vert NAT libelle « vu de ton noeud » (D4 vol.1). **Critere :
`SeedRequest` authentifie (sig invalide → rejet) ; un pair fetch+tag+pin un
blob sur invitation+approbation ; provenance auteur intacte ; E2E 2-noeuds
reel (pattern §P57) : le pair garde l'app joignable.** **Si deborde →
arbitrage Checkpoint** (slice = `SeedRequest` + fetch+tag sans
invitation-révocable complete ; ou design prouve + E2E minimal).

### Phase F — Seed cross-noeud (D3 segment 2 aval) : re-annonce persistante distante au boot + registre `SeedAnnounced` + compteur multi-seed

Re-annonce **persistante** par le pair distant **apres reboot** (le seeder
relit son `keep_online` au boot, comme Phase D mais cote pair). Registre op
feed **`SeedAnnounced`** (raw-op, zero bump) → agregat seed + **etat
multi-seed** + compteur « Toi + N pairs » (best-effort TTL, D3/Checkpoint).
Front « Qui la garde en ligne » multi-seed + « Copies de secours »
fonctionnel (remplace « Bientot »). **Critere : un pair distant re-annonce
apres reboot (E2E) ; `SeedAnnounced` ingere → compteur ; multi-seed visible.**
**Cran le plus aval : si deborde → reste « Bientot » (D5).**

### Phase G — Wrap-up + dette (coverage T14 + carries audit S73)

`sprint74_verification.md` (fail-fast rempli) + `sprint75_audit_plan.md`.
**Dette coverage T14** : ecrire les tests `FileUploadBlock` (35 %→seuils
85/90/78/85), **retirer le masquage `| tail`** du fail-fast (verify.sh
step12), **ajouter `bootstrap.ts` a `coverage.include`**. **Carries audit
S73 non traites en A-F** (B.2 quorum zombie + statut terminal + test
redundancy>1 ; FRESHNESS ReleasePublished indexable ; D.1 recadrer
THREAT_MODEL §11 ; B.5 normaliser isHttpsUrl 3 ancres + test multi-vecteur ;
SEARCH-VIEW-THROW-SKELETON `query.isError` ; E.3 renforcer 3 tests C/D ; B.4
+ C.4 PATTERNS). `PATTERNS.md` (pattern seed cross-noeud + pin local) +
memory + SPRINT_LOG row + CLAUDE.md. **Critere : 100% fail-fast verts (y
compris `test:coverage` ENFIN vert, `| tail` retire) ; 2 docs planning ;
carries audit traites ou re-route explicitement ; PATTERNS + memory a jour.**

---

## §6 Items carry/dette

### Carries audit S73 (14 P2 — `sprint73_audit_findings.md`)

| Item | Source | Phase S74 | Exit condition |
|---|---|---|---|
| C.3 ROWID-PARTITION (tripwire AVANT browse-indexing prod) | audit S73 | **Phase B** | rowid feed/browse partitionnes avant tout browse-indexing prod. |
| B.6 `is_open_source⇒provenance_hash` non re-applique au browse | audit S73 | **Phase B** | invariant spec §2.1 re-applique au chemin browse-index. |
| OFF-SPRINT-2 deploy per-app sans test | audit S73 | **Phase C** | test non-regression deploy per-app (multi-app → cartes distinctes). |
| OFF-SPRINT-2b /publish+gossip gardent node_id | audit S73 | **Phase C** | per-app project_id sur /publish (http.rs:1004) + gossip (runtime.rs:1569, publish.rs:39). |
| H.1 M17 boot-recovery warn-only → index vide silencieux | audit S73 | **Phase D** | recovery non-silencieuse (metrique/log eleve, pas warn noye). |
| H.2 reconstructibilite limitee tranche feed | audit S73 | **Phase D** | browse-rows reconstructibles OU carry documente (depend browse-indexing B). |
| B.2/E.2 quorum guardrail zombie + statut terminal + test redundancy>1 | audit S73 | **Phase G** | branche trip pose Rejected ; test daemon redundancy>1 × trip. |
| FRESHNESS-RELEASE-UNINDEXED (nom de projet ReleasePublished) | audit S73 | **Phase G** | ReleasePublished indexe project_name/category (matchable) + test. |
| D.1 THREAT_MODEL §11 stale (« debounce de fait » faux) | audit S73 | **Phase G** | recadrer §11 (residual loopback single-user, pas debounce) ; clamp `q`/`offset` si retenu. |
| B.5 isHttpsUrl mono-vecteur + 3 ancres pre-existantes | audit S73 | **Phase G** | normaliser sur Browse:471 / BrowsedProject:367 / VerificationDetail:185 + test multi-vecteur. |
| SEARCH-VIEW-THROW-SKELETON (skeleton infini drift Zod) | audit S73 | **Phase G** | branche `query.isError` → carte d'erreur + test. |
| E.3 3 tests Phase C/D sous-asserent | audit S73 | **Phase G** | renforcer assertions (REWRITE upsert, ReleasePublished, upgrade reel M17). |
| B.4 system_prompt vide → 3/4 regles output-filter inertes | audit S73 | **Phase G** | documenter THREAT_MODEL §14 (PATTERNS). |
| C.4 hot-upsert warn-only sans catch-up runtime | audit S73 | **Phase G** | documenter PATTERNS (+ metrique drift si peu couteux). |

### Carry G7 — chantier UX bootstrap & coordinator-model (memory 2026-06-07)

| Item | Phase S74 | Exit condition |
|---|---|---|
| Design **D-DISPO** (`s74_disponibilite_ux_design.md`) | Phases A-F | Disponibilite continue cablee (front + pin local + cross-noeud borne). |
| Rename « coordinateur »→« noeud »/« reseau » | **Phase A** | AppShell CoordinatorPicker, daemon.ts, nav, AddCoordinatorDialog, OnboardingEmpty. |
| **Dette coverage T14** (FileUploadBlock + `| tail` + `bootstrap.ts`) | **Phase G** | tests FileUploadBlock ≥ seuils ; masquage `| tail` retire ; `bootstrap.ts` ∈ `coverage.include` ; `test:coverage` vert. |

### Carries reconduits S74 (exemptes / hors-scope)

| Item | Reports | Justification (renouvelee 2026-06-07) |
|---|---|---|
| P2-A-1 (rand upstream) | exemption | Blocker amont (crate `rand` fix non publie). Toujours non publie. |
| P2-AUDIT-2 (iroh transitives pre-release) | herite | Pin iroh 0.98 (decision gelee). Pas d'upgrade 1.0 stable publie. |
| T-NN+2 (iframe Rust-wasm) | exemption | Depend upstream wasm (PATTERNS §P34). Pas de changement amont. |
| P3-OS-1 (operator_server OR duplique) | pre-existant S70 | Trigger = prochaine modif `handle_artifact_draft`. Reconduit sauf si Phase B/C le touche. |
| LT-5 redundancy persistence | ex-post-launch | **PULL-FORWARD S74** (directive PO) — Phases E-F. N'est plus reconduit. |
| LT-3/LT-4 (Gini>0.70 / biometric) | post-v1.0 | Aucune sous-condition remplie. Latent. |

### ROADMAP_COMMITMENTS (Regle 3 — conditions evaluees 2026-06-07)

| LT | Condition | Etat 2026-06-07 |
|---|---|---|
| **LT-2 Radicle** | tag v1.0 **pousse** vers origin + GitHub Release | **PENDING** — tag v1.0 pose localement, **PAS pousse** (37+ ahead, rien pousse). Condition NON remplie → reste latent. |
| **LT-5** | 1er deploiement multi-worker OU v1.0 go-live | **TIRE EN AVANT par directive PO** (hosting orphelin de S75/S76) → Phases E-F S74. Le commitment est honore en avance, pas attendu. |
| LT-6 | iroh > 0.97 | RESOLVED S32 (iroh 0.98). |
| LT-7 | pre-v1.0 (Tier 1+2 DONE) | Gate satisfait. Tier 3 worker quorum E2E → S75. Non declenche S74. |

**Trigger ROADMAP_COMMITMENTS pour S74** : **LT-5 honore en avance** (Phases
E-F). LT-2 reste PENDING (rien pousse).

---

## §7 Scope cuts (exhaustif)

| # | Item | Sprint cible | Rationale (factuel) |
|---|---|---|---|
| 1 | GPU partage volontaire prouve cross-machine | S75 | S74 replique des **blobs** (disponibilite). Le routage de **taches** cross-machine = S75 (leve B-3). |
| 2 | Quorum redundancy>1 cross-MACHINE reel | S75 | Hors hosting. Le seed cross-noeud (S74) ≠ compute cross-machine (S75). |
| 3 | Sharding pipeline « gros modele » | S76 STRETCH | Jamais avant preuve S75. |
| 4 | Re-allocation/failover auto (IPFS Cluster `replication_factor_min`) | post-launch | Vision D3 — reglage numerique + failover auto = sur-ingenierie + UX non-technique. Abstrait, jamais un reglage. |
| 5 | Timer re-annonce periodique 22h (type IPFS reprovide) | post-launch | Le re-annonce-au-boot (#7) + NeighborUp replay couvrent le pilote ferme. Raffinement. |
| 6 | Probe externe NAT complet (signal quorum tiers de joignabilite) | S75+ | D4 vol.1 : S74 = libelle honnete « vu de ton noeud ». Le probe externe est un raffinement (depend infra reseau S75). |
| 7 | Page « Mes seeds » complete (greffe B) | post-S74 / arbitrage | D-DISPO §11 : empty-state + note anti-recentralisation possibles S74 ; la page complete = aval (arbitrage Checkpoint). |
| 8 | Editeur Monaco dans l'atelier | jamais (PO-9) | L'agent edite, l'operateur supervise (terminal/chat/diff). Decision gelee. |
| 9 | Templates etendus react/pyodide | arbitrage PO-8 Checkpoint | static + static-reader surs S74 ; react/pyodide = decision kickoff (Checkpoint). |
| 10 | SearchManifest reseau-large (op feed broadcast d'index) | post-launch | Defere S73 (D3). Distinct de `SeedAnnounced` (fait observable cible, pas un index broadcast). |
| 11 | Compteur seed nombre exact | arbitrage D3/Checkpoint | Best-effort « Toi + N pairs » TTL recommande (nombre exact fragile en gossip). PO tranche. |
| 12 | Tantivy | gate post-S75 si >50K docs | Gele (CLAUDE.md:306). FTS5 reste l'engine. |
| 13 | Streaming token-par-token worker reseau distant | jamais (PO-14) | Decision gelee S72. |
| 14 | Rate-limit per-client search (residual T-SEARCH-DOS) | Phase G recadre | D.1 audit : recadrer THREAT_MODEL §11 (residual loopback) ; livrer le clamp `q`/`offset` si retenu, sinon documenter. |

---

## §8 Tracabilite scope

Mapping de chaque item « What's NOT » du sprint precedent (S73 §7) sur son
traitement S74.

| Item S73 « What's NOT » (§7) | Sprint + Phase S74 |
|---|---|
| #2 `sbfb-factory search/open/fork` (Factory tire du reseau) | **S74 Phase B-C** (atelier fork) |
| #3 Notion de projet cible distinct nexus (`process::repo_root`) | **S74 Phase B** |
| #4 reseau→atelier : clone `repo_url@commit` ou reconstruction blob (PO-5) | **S74 Phase B** |
| #5 Templates etendus (react, pyodide) | **S74 Phase C** (arbitrage PO-8) |
| #1 SearchManifest reseau-large | Reconduit **post-launch** (§7 #10) |
| #6 GPU partage cross-machine | Reconduit **S75** (§7 #1) |
| #7 Quorum redundancy>1 cross-MACHINE | Reconduit **S75** (§7 #2) |
| #8 Sharding pipeline | Reconduit **S76 STRETCH** (§7 #3) |
| #11 Rate-limit per-client search | **S74 Phase G** (§7 #14, D.1 recadre) |
| #12 Webhook/SSE feed push | Reconduit **S75+** |
| #13 Token-par-token WAN | **jamais** (§7 #13, PO-14) |
| #14 Pagination boutons | Reconduit si corpus le justifie |
| **NEW S74** : Disponibilite/Hosting (pin local + seed cross-noeud LT-5) | **S74 Phases A,D,E,F** (pull-forward PO) |

---

## §9 Risk register (R1..R8)

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | **Scope creep** : un seul sprint (atelier + dispo + pin + protocole seed cross-noeud) deborde et ne ferme rien | **Eleve** | **Eleve** | **D5 segmentation stricte** : A-D surs (fork + dispo front + pin local), E-F cross-noeud borne. Criteres binaires par phase. Si E-F debordent, A-D livrables + crans cross-noeud restent « Bientot ». **Arbitrage ampleur PO Checkpoint §11.** |
| R2 | Le `SeedRequest` ALPN (D1) = nouveau protocole sous-estime (handshake, approbation, NAT dial) | Moyen | Eleve | D1 : composition de primitives existantes (Ed25519+JCS+nonce ; ALPN comme blobs/docs) ; preflight [DETER] crypto-spec Phase E ; E2E 2-noeuds reel (§P57) AVANT CLOSED. Slice de repli (Checkpoint). |
| R3 | Le blob fetche par le seeder n'est pas tagge → GC silencieux (gap `blobs.rs:140`) | Moyen | Eleve | D1 : corriger `fetch_ticket` (ou `fetch_and_pin`) pour **tagger** le blob cote seeder ; test « blob seede survit au GC + reboot ». |
| R4 | Faux-vert NAT persiste : « En ligne » self ment quand le noeud est derriere NAT non-joignable | Moyen | Moyen | D4 vol.1 : libelle honnete « En ligne (vu de ton noeud) » des S74 ; probe externe = raffinement S75 (scope cut #6). |
| R5 | Re-attribution d'auteur au fork/seed (violation invariant gele) | Faible | **Eleve** | Invariant gele : fork = re-signature par MON noeud (nouvel auteur local) ; seed = provenance auteur **intacte** (seeder ≠ co-auteur, modele Radicle delegate). Test « provenance auteur inchangee apres seed ». 5 verrous anti-recentralisation cables UI. |
| R6 | Migration M18 `keep_online` ou re-annonce-boot casse le boot du daemon | Faible | Eleve | D2 : M18 = ADD TABLE local (additif, reconstructible) ; re-annonce best-effort warn-only (n'echoue pas le boot, comme C.4/#7). Test reboot simule. |
| R7 | OFF-SPRINT-2b incomplet (/publish+gossip node_id) cause des collisions multi-app pendant le fork-redeploy | Moyen | Moyen | Phase C : completer per-app project_id sur /publish + gossip AVANT de prouver la boucle fork-redeploy (sinon le re-deploy collisionne). |
| R8 | Dette coverage T14 (`test:coverage` rouge masque par `| tail`) reste invisible et casse GHA au 1er push | Moyen | Moyen | Phase G : ecrire tests FileUploadBlock ≥ seuils + retirer `| tail` + ajouter `bootstrap.ts`. Rend le gate honnete AVANT le push origin (LT-2). |

---

## §10 Audit gate pattern — rappel

- **Phase 0** : DEJA JOUE (§3) — `sprint73_audit_findings.md` (`2fe3b30`),
  verdict PASS (0 P0, 0 P1, 14 P2, 8 P3). Aucun fix requis. Les 14 P2 routes
  au plan S74 (§6).
- **Phase de sortie (G)** : produit les deux livrables obligatoires dans un
  commit `docs(sprint74)` : `sprint74_verification.md` (self-report fail-fast
  rempli, **incl. `test:coverage` vert apres retrait `| tail`**) +
  `sprint75_audit_plan.md` (feuille de route session fraiche S75). Sans ces
  deux fichiers, le sprint ne ferme pas (§3.3).
- Phase G met a jour `docs/rust/PATTERNS.md` + `docs/shell/PATTERNS.md` (seed
  cross-noeud + pin local) + memory + SPRINT_LOG + CLAUDE.md.

---

## §11 Checkpoint de validation

**Arbitrages PO a trancher AVANT le plan detaille fige.** Les 6 questions du
design D-DISPO (`s74_disponibilite_ux_design.md §11`) + l'ampleur cross-noeud
+ le decoupage de phases. **Les decisions research-decisives D1 (ALPN),
D2 (pin local table+tag+boot) sont posees ; D3/D4/D5 portent les bifurcations
produit.**

1. **D5 / AMPLEUR CROSS-NOEUD (load-bearing, R1)** — le decoupage A-D surs
   (atelier-fork + dispo front + pin local) / E-F cross-noeud borne est-il
   accepte ? Et l'ampleur du Segment 2 : (a) **full** `SeedRequest` +
   invitation revocable + approbation + re-annonce distante + registre
   multi-seed ; (b) **borne** `SeedRequest` + fetch+tag+pin + E2E 2-noeuds
   sans le registre multi-seed (reste « Bientot ») ; (c) **design prouve +
   slice** si E-F debordent ? **Recommandation : viser (a), replier sur (b),
   (c) en dernier recours — chaque cran non livre reste « Bientot ».**
2. **D4 vol.1 / FAUX-VERT NAT** — libelle honnete « En ligne (vu de ton
   noeud) » pour les apps self-publiees **des le pilote** (recommande,
   zero infra), **ou** attendre le probe externe (plus juste, plus de
   travail, scope cut #6) ?
3. **D4 « La remettre en ligne » (app tombee)** — `/deploy` prerempli
   (re-signature, coherent fork — recommande) **ou** futur « adopter le
   blob » sans re-signature (semantique d'auteur differente) ?
4. **Greffe B / page « Mes seeds »** — livrer en S74 (vision + empty-state +
   note anti-recentralisation) en plus du cross-noeud reel, **ou** la
   reporter (scope cut #7) ?
5. **D3 / COMPTEUR communautaire** — best-effort « Toi + N pairs (vus
   recemment) » sans nombre fragile (recommande) **ou** nombre exact des le
   registre `SeedAnnounced` ?
6. **B.5 / scheme-guard `isHttpsUrl`** — appliquer aux nouvelles ancres
   `repo_url` (panneau Source, app tombee) **ET** aux 3 ancres pre-existantes
   non gardees (Browse:471, BrowsedProject:367, VerificationDetail:185) dans
   le meme lot Phase G (recommande) ?
7. **PO-8 / TEMPLATES** — react + pyodide ajoutes en S74 (kickoff arbitre)
   **ou** static + static-reader surs seulement, react/pyodide reportes ?
8. **Rename « coordinateur »** — toute l'UI en S74 (Phase A, recommande pour
   coherence) **ou** seulement les ecrans publish/dispo touches ?

### Arbitrages PO — Checkpoint §11 (resolus 2026-06-07)

1. **AMPLEUR CROSS-NOEUD = (a) FULL E-F**, plancher A-D GARANTI. Viser Phase E
   (`SeedRequest` + fetch+tag+pin + invitation/approbation) ET Phase F
   (re-annonce distante + registre `SeedAnnounced` + compteur multi-seed).
   Degradation gracieuse : si E-F debordent, A-D restent livrables + chaque
   cran non fini reste « Bientot » (jamais un faux bouton actif). [directive
   PO « faire tout pour ce sprint, les prochains ont d'autres objectifs »]
2. **FAUX-VERT NAT** = libelle honnete **« En ligne (vu de ton noeud) »** des
   le pilote (recommandation retenue).
3. **« La remettre en ligne » (app tombee)** = **`/deploy` prerempli re-signe**
   (coherent atelier-fork ; recommandation retenue).
4. **Page « Mes seeds »** = **livree en S74** (vision + empty-state + note
   anti-recentralisation), en plus du cross-noeud reel.
5. **COMPTEUR communautaire** = **best-effort « Toi + N pairs (vus recemment) »**
   TTL (recommandation retenue ; pas de nombre exact fragile).
6. **`isHttpsUrl`** = **normaliser les 3 ancres pre-existantes**
   (Browse:471, BrowsedProject:367, VerificationDetail:185) + les nouvelles
   ancres, meme lot Phase G (recommandation retenue).
7. **TEMPLATES (PO-8)** = **static + static-reader + react + pyodide** — le PO
   AJOUTE react/pyodide en S74. Impact : Phases B/C portent 4 templates (cout
   surveille ; concours au risque R1, garde-fou degradation E-F inchange).
8. **Rename « coordinateur » → « noeud »/« reseau »** = **toute l'UI** en S74
   (Phase A ; recommandation retenue).

**Note process** : agents `nexus-*` non enregistres → ce kickoff est un
**substitut** main thread + Workflow (fallback documente). Reviews phases =
agent `general-purpose` independant ; supervision = hooks backstop (D17).
Codex gate (CLI GPT 5.5) BLOQUANTE review→commit sur les phases code. La
**recherche profonde cross-noeud (D1/D3/D4)** est faite (5 modeles OSS cites
§Sources) — les bifurcations restantes sont **produit**, pas factuelles.
