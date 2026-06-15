# Sprint 75 — Pivot proposal : `NodeDirectoryEntry` n'est PAS le SearchManifest différé

> **Artefact d'arbitrage Day-0.** La découverte PULL node-centrique touche
> légèrement la décision gelée **D3 (SearchManifest DEFER, PO-13, S73)**. Le
> workflow G8 / README exige qu'une décision touchant une décision Day-0 passe
> par un `pivot_proposal.md` + **arbitrage PO explicite AVANT tout code**. Ce
> document trace la ligne et demande ton sign-off.

## 1. Pourquoi ce document

Le prompt de kickoff S75 (§3, §6) et le risque G8 anticipaient que le pivot
« reframe SearchManifest de DEFER à découverte primaire » = un **overturn**
d'une décision Day-0. **Le panel de décision adversarial (3 avocats + juge,
workflow `wdeedndsh`) a trouvé l'inverse** : le meilleur substrat **ne tire PAS
SearchManifest en avant**. Il construit un type **distinct, purpose-built**
(`NodeDirectoryEntry`) et **garde SearchManifest différé**. Donc D3 n'est pas
réécrit *en esprit* — mais deux choses le **touchent** et doivent être tranchées
explicitement pour que le kickoff ne contredise pas silencieusement une décision
gelée.

## 2. La décision en une ligne

**Substrat retenu = B-hybride : `NodeDirectoryEntry`**, nouveau type signé sous
`DOMAIN_NODE_DIRECTORY_V1`, réutilisant *verbatim* la machinerie crypto/runtime
de `CuratorList` (sign/verify Ed25519+JCS, revision monotone anti-rollback, caps
DoS 256+par-champ, attention-set opt-in + persistance `subscriptions.json`,
ingest gossip 9-étapes subscription-gated, read-path `BrowseAggregator` flatten).
Le **payload** reprend la forme « index signé de ce que j'héberge » de F-Droid /
SearchManifest — mais comme **liste humainement affichable** de
`{node_id, project_id, archive_hash, name, category, description}`, **PAS** un
digest de couverture Bloom.

- **Substrat C (SearchManifest tiré en avant)** = **REJETÉ. Le DEFER tient.**
- **Substrat A (surcharger `CuratorList`)** = rejeté comme *type primaire*, mais
  sa machinerie est la fondation réutilisée par B.

## 3. La frontière — `NodeDirectoryEntry` vs SearchManifest

C'est le cœur du sign-off. Les deux sont « un index signé qu'un client tire » —
mais ce sont **deux objets différents, à deux couches différentes** :

| Axe | `NodeDirectoryEntry` (construit en S75) | SearchManifest (reste différé, s73 D3) |
|---|---|---|
| **Problème résolu** | Découverte / Browse : « montre-moi le catalogue d'un nœud connu et laisse-moi tirer une app » | Recherche fédérée : « cherche des projets qu'un nœud n'a JAMAIS reçus via gossip » (federation-partielle) |
| **Payload** | Liste **humainement affichable** d'apps `{node_id, project_id, archive_hash, name, category, description}` → cartes F-Droid | **Digest de couverture** Bloom/Merkle des `project_id` (test d'appartenance) → non affichable |
| **Couche** | Read-side projection de la découverte | Index full-text fédéré au-dessus de la recherche |
| **Rôle du nœud** | **Catalogue-publisher** : publie SON propre catalogue + seede SES apps + invites acceptées | **Aggregator / index-node** : agrège les feeds de PLUSIEURS curateurs |
| **Déclencheur** | **MAINTENANT** : bug live (apps invisibles aux nouveaux pairs, fenêtre PoW 30 min) | Différé : 3 signaux empiriques post-launch (federation-partielle observée / demande mesurée / corpus >50K docs — s73 §5) |
| **Domaine signé** | `DOMAIN_NODE_DIRECTORY_V1` (nouveau, disjoint) | `DOMAIN_SEARCH_MANIFEST_V1` (nouveau, disjoint — non posé) |

**On ne peut pas rendre des cartes F-Droid depuis un filtre de Bloom.** C'est la
preuve que ce sont deux objets distincts, pas le même sous deux noms.

## 4. Les deux points qui touchent D3 (et comment on les garde propres)

**(1) Collision de nom / frontière de scope.** Un relecteur pourrait lire le
pivot comme « construire le SearchManifest différé sous un autre nom ». La
réponse : la table §3 trace la ligne. `NodeDirectoryEntry` = catalogue d'UN
nœud (ses propres apps), tiré par un client qui a choisi de s'y abonner.
SearchManifest = digest de couverture agrégeant PLUSIEURS feeds de curateurs pour
la recherche cross-nœud. Couche différente, payload différent, déclencheur
différent.

**(2) Gravité du « rôle index-node ».** Le rôle VPS-ancre-annuaire (D3) est
structurellement proche du « rôle index-node » qu'introduit SearchManifest.
La réponse : **l'ancre est un CATALOGUE-PUBLISHER, pas un AGGREGATOR.** Elle
publie seulement SON propre node-directory + seede SES propres apps + invites
explicitement acceptées. Elle **n'indexe PAS** les feeds des autres nœuds, **ne
devient PAS** un étage de couverture-recherche. Ça garde le risque de
concentration NIP-66 dehors et préserve le rationale DEFER broadcast-Sybil de
s73 (ARES 2024).

## 5. Ce qui reste différé (inchangé)

- **SearchManifest lui-même** (le digest de couverture, l'agrégation
  multi-curateurs, le `DOMAIN_SEARCH_MANIFEST_V1`, le rôle index-node, la
  query fédérée). Réservé aux 3 déclencheurs empiriques post-launch (s73 §5).
- **Si un déclencheur §5 se présente post-launch**, SearchManifest se pose
  **AU-DESSUS** de l'annuaire : l'annuaire alimente le catalogue (browse+pull) ;
  SearchManifest alimenterait la recherche cross-nœud. Les deux couches
  cohabitent, l'une ne remplace pas l'autre.
- **Tantivy** reste gelé (gate post-S75 >50K docs). FTS5 reste l'engine.

## 6. Cadre anti-recentralisation (le test que ça passe)

L'ancre VPS reste **« Mon serveur »** : un `node_id` dans MON `config.toml`
(`default_curators`, vide par défaut dans le binaire — `config.rs:245-251`),
JAMAIS hard-codé dans une liste livrée à tous. L'annuaire est une **liste signée
répliquée** (forme curator-list : Ed25519 + JCS + revision monotone + cap 256),
propagée gossip+blob — **n'importe qui en publie une**, c'est **une parmi N**
(dépôt F-Droid / relay Nostr). **Le réseau survit à la mort du VPS** : un nouveau
nœud avec `default_curators` vide bootstrappe via gossip + toute autre ancre
qu'il configure ; les apps seedées restent joignables tant qu'un détenteur du
hash BLAKE3 répond (content-addressing = vérité de joignabilité). Le seul
**DESIGN-CONFLICT à interdire** : hard-coder `135.181.42.188` (ou son node_id)
dans un `default_curators` compilé livré à tous.

## 7. Arbitrage PO demandé (3 sign-offs)

Avant tout code Phase A, je demande ton sign-off explicite sur :

1. **L'annuaire construit en S75 = `NodeDirectoryEntry` (catalogue browse+pull),
   PAS le SearchManifest différé.** Le DEFER de D3/s73 tient ; SearchManifest
   reste réservé aux déclencheurs §5 post-launch et se poserait AU-DESSUS de
   l'annuaire, pas à sa place.
2. **L'ancre VPS = catalogue-publisher borné (room+pub pattern SSB), pas
   aggregator.** Elle publie son catalogue + seede ses apps + invites acceptées ;
   elle n'agrège pas les feeds des autres.
3. **Substrat = B-hybride (`NodeDirectoryEntry` + machinerie CuratorList +
   forme-pull F-Droid + plumbing-abonnement curator-list).** C rejeté, A rejeté
   comme type primaire.

Si tu valides ces 3 points, le kickoff S75 (D1-D5 gelées) tient et la Phase A
peut démarrer (après son G8 preflight). Si tu veux ajuster la frontière D3 ou le
substrat, c'est **le moment sans coût** de le faire.

---

**Source** : workflow décision `wdeedndsh` (5 agents recherche profonde + panel
3-substrats avocat/juge, ~1.07M tokens) ; doc D3 différé
`.planning/research/s73_searchmanifest_index_node_design.md` ; prior art OSS
(NIP-65 outbox, Radicle Heartwood inventory, F-Droid index-v2, SSB rooms, BEP-44,
IPFS reprovide, ARES 2024).
