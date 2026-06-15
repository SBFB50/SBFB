# Sprint 75 — Design Review Board (G1)

> Board G1 du kickoff S75 (pivot découverte PULL node-centrique + ancre VPS).
> Score chaque décision D1-D5 gelée, challenge l'approche AVANT le code, et
> signale les ⚠️ avec leur ajustement inline. Source : workflow décision
> `wdeedndsh` (5 agents recherche profonde + panel adversarial 3-substrats
> avocat/juge ; ~1.07M tokens, 9 agents, 206 tool-uses).

## Méthode

Le kickoff a été produit par un panel **adversarial** : 3 avocats ont argumenté
chacun POUR un substrat (A curator-list étendue / B NodeDirectoryEntry sibling /
C SearchManifest tiré en avant), puis un juge a tranché avec comparaison
explicite — **sans hériter du lean curator-list de la recherche**. La décision
substrat (D1) n'est donc pas un rubber-stamp : elle a survécu à 3 plaidoyers
contradictoires + un arbitrage code-grounded.

## Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅, D5 ✅

Rigor signal G4 satisfait (1 ⚠️ sur 5 — le ⚠️ D3 est l'arbitrage PO requis, pas
un défaut de conception).

---

### D1 — Substrat annuaire : `NodeDirectoryEntry` sibling ✅

**Décision** : nouveau type signé `NodeDirectoryEntry` sous
`DOMAIN_NODE_DIRECTORY_V1`, réutilisant la machinerie `CuratorList`.

**Challenge du board** : pourquoi pas A (réutiliser `CuratorList`, plus haut
reuse) ? Le juge a tranché sur **3 axes vérifiés dans le code** :
1. **Honnêteté du modèle de données** : `CuratorProjectRef` (`curator.rs:137-154`)
   ne porte QUE `{project_id, project_name, category, description}` — **pas
   d'`archive_hash`** — et conflate `project_id == node_id` (`browse.rs:632-634`).
   Le pivot a besoin d'un nœud hébergeant N apps, chacune `(project_id,
   archive_hash)` distincte. A doit donc AJOUTER 3 champs à un type dont chaque
   doc-comment dit « vouches for » → A paie le coût de B (« un nouveau struct »)
   tout en gardant la dette sémantique. B mappe le modèle sans surcharge.
2. **Coût d'audit** : `curator.rs:589-602` teste la séparation de domaine
   cross-replay comme propriété **first-class**. Un discriminateur in-bytes (A)
   donne la distinction d'octets mais PAS la séparation de domaine → drift
   surface (un relecteur doit vérifier le check sur CHAQUE ingest+read path).
3. **Précédent déjà dans le repo** : S74 Phase E a livré
   `DOMAIN_SEED_REQUEST_V1` comme type sibling signé (`canonical.rs:201-219`).
   B n'est **pas** un pattern nouveau — c'est un template testé à copier. Le
   coût marginal a déjà été payé il y a un sprint.

**Prior art unanime** (R2) : chaque système mature a donné à l'auto-publication
son PROPRE type par-publisher (NIP-65 kind:10002, Radicle INVENTORY, F-Droid
index signé par dépôt, BEP-44). AUCUN n'a surchargé une primitive de curation.

**Verdict** : ✅ décision robuste, survit au plaidoyer adversarial. Reuse max de
la crypto/DoS sans surcharge sémantique.

### D2 — Sort du push/PoW : FIX-A re-mint-on-replay d'abord, indépendant ✅

**Décision** : corriger le bug live **en premier et indépendamment** (Phase A) :
stocker le payload `ProjectAnnouncement` **non-wrappé** dans l'outbox ; à chaque
site de replay (`runtime.rs:1513/1544/1615`) re-wrapper avec un PoW **frais**
(`PowSolveCache` ≈ gratuit) ET **re-minter l'`EndpointAddr`/`BlobTicket`** depuis
`my_endpoint_addr()` courant. **Ne PAS affaiblir** `MAX_PROOF_AGE_SECS=1800` — le
re-mint rend la fenêtre existante *correcte*, pas supprimée. Le helper re-mint
d'adresse est ensuite **réutilisé** par le path pull.

**Challenge** : pourquoi pas FIX-B seul (rendre le push caduc via le pivot) ?
Rejeté : tant que tous les pairs ne tournent pas le client pull, le gossip
legacy est le SEUL canal de découverte ; le laisser cassé = zéro découverte
pendant le rollout. La moitié re-mint-adresse est un **prérequis dur** que le
path pull a aussi besoin (un catalogue tiré doit servir une adresse fraîche
dialable).

**Verdict** : ✅ root cause adressée (adresse + PoW), pas de band-aid, pas
d'affaiblissement de l'invariant anti-replay. Helper réutilisé downstream.

### D3 — Modèle opérationnel VPS (rôles ancre) ⚠️ → arbitrage PO requis

**Décision** : VPS = **deux rôles bornés**. (1) DIRECTORY (pattern *room* SSB) :
publie un `NodeDirectoryEntry`, point de rencontre, peu coûteux. (2) SEED
(pattern *pub* SSB, **borné** par la leçon overload pub→room) : seede SEULEMENT
MES apps + invites explicitement acceptées, budget disque + policy de seeding
par-projet (modèle Radicle), JAMAIS un miroir universel du réseau. Headless :
driver config-driven « seed ces project_ids au boot » + 1er appelant prod de
`request_seed`. Ancre dans MON `config.toml` `default_curators`, JAMAIS
hard-codée.

**Pourquoi ⚠️** : ce n'est PAS un défaut de conception — c'est que **D3 touche
la décision Day-0 D3/s73 (SearchManifest DEFER)** et exige un `pivot_proposal.md`
+ **sign-off PO** (cf. `sprint75_pivot_proposal.md`). Le juge a clarifié que le
pivot **ne tire PAS** SearchManifest en avant (il construit `NodeDirectoryEntry`,
un objet distinct), donc le DEFER tient — mais la frontière (catalogue-publisher
≠ aggregator/index-node) doit être validée par le PO avant le code.

**Ajustement inline** : produire `sprint75_pivot_proposal.md` (FAIT) + obtenir
le sign-off PO sur les 3 points (annuaire ≠ SearchManifest ; ancre = publisher
borné pas aggregator ; substrat B-hybride) AVANT la Phase A. Le ⚠️ se lève au
sign-off.

### D4 — Durabilité catalogue distant : persister les ancres, re-pull au boot ✅

**Décision** : forme F-Droid « le fingerprint persiste, l'index est re-fetché »
— **déjà l'architecture** (`iroh_runtime.rs:35-37` : l'attention-SET est
durablement persistée dans `subscriptions.json`, les ENTRÉES sont RAM-only par
design). Persister les **node_ids d'ancre** (déjà durable via subscriptions/
config) + transformer le ré-arrivage passif en **re-pull actif au boot** des
`NodeDirectoryEntry` des ancres abonnées. Ferme le gap load-bearing :
`direct_entries` in-memory (`browse.rs:272`) + restore OWN-apps-only
(`runtime.rs:1876-1897`) pour les catalogues DISTANTS.

**Challenge** : pourquoi pas persister les entrées de catalogue distantes en
durable ? Rejeté : RAM-only + re-pull est le design délibéré endossé par le
prior art (F-Droid) ; persister des catalogues périmés invite le résidu
over-count/stale-blob que l'annuaire doit éviter. **Foyer naturel des carries
S74** WIRE-1 (indexer ReleasePublished par nom), WIRE-2 (seed-count keyé
(project_id,archive_hash)), DBQ-1 (keep_online hash-SOT) — à concevoir dedans.

**Verdict** : ✅ ferme le seul vrai trou architectural (R4 load-bearing gap), via
le pattern prior-art, sans introduire de DB centrale.

### D5 — Wire additif 0-bump + pull multi-provider ✅

**Décision** : purement **ADDITIF, 0-bump** (nouveau DOMAIN + type signé,
orthogonal à `FEED_FORMAT_VERSION`/`CURATOR_LIST_FORMAT_VERSION` — exactement le
pattern S74 SeedRequest, aucun decoder legacy pre-launch). Liveness = **sonde
pull live + content-addressing BLAKE3** (propriété OBSERVÉE, pas un claim
d'horloge PoW). **Fetch multi-provider IN-SCOPE** (carry PULL-2) : plumber les
`seeder_node_id` de `SeedRegistry` (aujourd'hui consommés seulement comme
compteur d'affichage) dans le vecteur de providers de `download()` (`fetch_ticket`
dial EXACTEMENT un endpoint aujourd'hui, `blobs.rs:170-193`).

**Challenge** : single-provider suffit ? Rejeté : un pull node-centrique où
l'ancre est offline doit retomber sur n'importe quel détenteur du hash BLAKE3
(modèle swarm BitTorrent) ; single-endpoint rend l'annuaire fragile. PoW-clock
liveness ? Rejeté : remplacé par la sonde live (la cure du bug racine).

**Verdict** : ✅ wire pre-launch propre, résilience multi-seeder additive,
liveness honnête. BLAKE3 reste le gate d'intégrité (annonce forgée sur-compte
mais ne sert jamais d'octets absents).

---

## Concerns transverses (notés, non bloquants)

| # | Concern | Mitigation (→ plan) |
|---|---|---|
| C1 | **Drift de l'ingest-arm dupliqué** : le sibling NodeDirectoryEntry duplique ~20 lignes du gate subscription/cap/revision (`iroh_runtime.rs:518-582`). | Extraire un helper générique `ingest<T: SignedList>` — **livrable de phase**, pas afterthought (Q2). |
| C2 | **Résidu stale-catalog / over-count** : une entrée peut annoncer un catalogue dont les blobs ont été GC. | Déjà câblé : BLAKE3 = vérité de joignabilité + sonde live reste l'autorité, jamais le compteur catalogue. |
| C3 | **Gap acquisition seed headless VPS** : `keep_online` re-annonce la fraîcheur feed au boot mais ne **re-fetch_and_pin PAS** un blob jamais déployé. | Driver config-driven « seed ces project_ids au boot » + 1er appelant prod `request_seed` (net-new, load-bearing — Phase E). |
| C4 | **Pénalité cold-start vs Radicle/F-Droid** (coût lock-3) : install fraîche avec `default_curators` vide = Browse VIDE jusqu'à ce que l'user ajoute une ancre. | Trade accepté pour la garantie plus dure ; l'UX doit faire de « ajouter une ancre » une intention de 1er-run claire, pas un écran vide mort (Phase F). |
| C5 | **Tripwire abonnement ancre (lock-3)** : le SEUL DESIGN-CONFLICT à interdire = hard-coder `135.181.42.188`/son node_id dans un `default_curators` compilé livré à tous. Actuellement NON déclenché (`config.rs:249-250` vide). | Guard/review explicite : toute liste d'ancre default non-vide compilée = DESIGN-CONFLICT. |
| C6 | **Gap de découverte fenêtre-rollout** : tant que les pairs ne tournent pas le client pull, FIX-A est le SEUL canal. | FIX-A doit lander et être **E2E vérifié cross-machine (Win↔Mac)** AVANT que le path pull soit gated dessus (Phase A → gate avant Phase C). |

## Checkpoint board

- [x] D1-D5 figées avec retained/rejected/code-implications (cf. kickoff §5).
- [x] Substrat décidé par panel adversarial, pas hérité (3 avocats + juge).
- [x] 5 verrous anti-recentralisation vérifiés vs le design (cf. kickoff §4 +
  checklist garde-fous 15 items).
- [x] Test « le réseau survit à la mort du VPS » énoncé + démonstration de passage.
- [ ] **D3 ⚠️ : sign-off PO sur `sprint75_pivot_proposal.md`** (3 points) — gate
  avant Phase A.
- [x] 6 risques ouverts + 8 questions plan documentés (kickoff §10 + plan).
- [x] Amendement roadmap v5 enregistré (kickoff §8).
