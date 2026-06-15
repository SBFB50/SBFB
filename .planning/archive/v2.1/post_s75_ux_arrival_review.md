# Review — mini-cycle UX-ARRIVAL (post-S75)

Date : 2026-06-11
HEAD de base : `10a311c` (changements non commités, commit unique a venir)
Process : Workflow multi-agent (fallback README §7) — 5 dimensions
adversariales en parallele → 2 skeptics refute-by-default par P0/P1 →
synthese main thread. 11 agents, ~1.31M tokens.
Preflight : `post_s75_ux_arrival_preflight.md` (PLAN-ADAPT).

## Verdict: PASS

(Promu apres reconciliation Codex R2 — cf. §Codex reconciliation.)

## Dimensions

| Dimension | Verdict agent | Findings |
|---|---|---|
| Correctness | CONCERN | 1 P1 (adjudique P2), 1 P2, 1 P3 |
| Security | CONCERN | 1 P1 (confirme 2/2), 1 P3, 1 NIT |
| Wire/Protocol | CONCERN | 1 P1 (confirme, meme root cause que Security), 1 P3, 1 NIT |
| Test integrity | PASS | 1 P2, 2 P3, 2 NIT |
| Conformite UX/Design | PASS | 2 P3, 2 NIT |

## Findings P0/P1 — adjudication skeptics et resolution

### SEC-UXARR-1 / WIRE-UXA-1 — `from_subscribed` forgeable (P1, CONFIRME 2/2 et 2/2)

`ProjectAnnouncement` ne porte AUCUNE signature : son `node_id` est une
string librement revendiquee (validee 64-hex seulement). La premiere
implementation derivait `from_subscribed` de « le node_id reclame est
dans l'attention set » — un annonceur payant UN PoW pouvait nommer la
pubkey d'une ancre publique abonnee et placer son app dans « Tes
sources » (et le hero « En vedette »). Contournement de frontiere de
confiance, confirme par 4 skeptics sur les 2 findings jumeaux (un
skeptic a propose le downgrade P2 « UI-placement-only » — non retenu :
le placement de confiance EST le produit de ce mini-cycle).

**Resolution (in-cycle, root cause)** : `from_subscribed` est desormais
**CATALOG-BACKED** (`http.rs browse_views` + `subscribed_catalog_index`) :
un `direct` n'est classe « mes sources » que si sa paire
`(project_id, archive_hash)` figure dans le catalogue **Ed25519-verifie**
de l'annuaire signe du noeud reclame. Un spoofer ne peut pas inserer de
row dans un catalogue signe ; les vraies apps d'un noeud abonne y sont
par construction du pivot PULL (publish → revision>0 → re-annonce boot).
Une entry sans `archive_hash` n'est jamais classee sur claim nu. Un
noeud abonne SANS annuaire publie voit ses push `direct` rester en
section decouverte — honnete, coherent avec la ligne « en attente » de
/nodes. Test decisif : `browse_views_derives_from_subscribed` (fixture
SpoofApp : node_id abonne + paire hors catalogue → false). THREAT_MODEL
§15.1 +1 row dediee (sev. brute H → residuelle L).

### UX-OBS-RATELIMIT-UNAUTH — rate-limit observed par identite non tarifee (P1, ADJUDIQUE P2 : 1 refute + 1 downgrade)

Mecanisme exact (verifie ligne par ligne par les 2 skeptics) : le champ
`node` de l'enveloppe d'annonce n'est pas authentifie et le PoW gossip
est lie a `(publisher, topic)`, PAS au payload — un seul PoW couvre N
annonces nommant des pubkeys forgees distinctes. Le rate-limit 1/min
borne la churn PAR identite reclamee et le cap 256 borne la taille
residente, mais rien ne tarife les identites forgees une a une : un
flood determine peut remplir le registre observed et evincer les hints
honnetes. Adjudication : PAS un P1 — meme classe que le residuel
fresh-flood du SeedRegistry (§15.1, assume S75), surface
non-autoritaire (la grille et les noeuds abonnes sont intacts ;
s'abonner a une cle forgee ne donne qu'une ligne « en attente », jamais
un fetch). Le defaut REEL etait l'overclaim documentaire (« each
costing a gossip PoW »).

**Resolution** : doc honnete (commentaires `MAX_OBSERVED_DIRECTORIES` +
`OBSERVED_REFRESH_MIN_SECS` reecrits + row THREAT_MODEL reformulee avec
les 2 residuels explicites). Le **publisher-binding** (lier la capture
observed a l'identite PoW du publisher) est route a l'audit S76 avec le
lot duress.

## P2/P3/NIT — resolution

| ID | Sev. | Resolution |
|---|---|---|
| UX-OBS-SELF-NODE / SEC-UXARR-2 | P2/P3 | FIXE — self-guard au point de capture (etape 4) : une annonce reclamant NOTRE node_id (echo gossip de notre broadcast ou forge distante) n'entre jamais en observed ; symetrie restauree avec le garde projet `announcement_claims_own_node_id`. Teste (extension `observed_recorded_without_any_fetch`). |
| TI-1 | P2 | FIXE — fixture cap-eviction decorrelee (pubkeys DESCENDENT quand les ts montent) : une eviction par ordre de cle serait demasquee. |
| TI-2 | P3 | FIXE — test Vitest `prefill efface a la reouverture manuelle` (Escape → bouton manuel → champ vide). |
| TI-3 | P3 | FIXE — contenu du registre pinne apres le rebond anti-displacement (newcomer absent, resident intact). |
| TI-4 | NIT | FIXE — rate-limit traverse par le chemin d'ingest COMPLET (2e annonce immediate → drop, last_seen inchange). |
| TI-5 / WIRE-UXA-2 | NIT/P3 | FIXE — key-count de l'enveloppe /nodes pinne (exactement {nodes, observed}, y compris la forme vide) : le seam du contrat Zod `.strict()`. |
| WIRE-UXA-3 | NIT | FIXE — commentaire `BrowseListResponse` mentionne les 2 cles derivees. |
| UXC-1 | P3 | FIXE — `EmptyState ambientOnly` : « Aucune app dans tes sources » + CTA explicite quand la section decouverte est pleine. |
| SEC-UXARR-3 | NIT | FIXE — copy ObservedRow prudente : « Annonce entendue sur le reseau » (notre observation, pas une agence prouvee). |
| UXC-4 | NIT | FIXE (partiel) — div `nodes-list` non rendue quand seuls des observes existent. Le niveau de titre h2 (page Nodes) vs h3 (page Browse) : pages distinctes, pas d'incoherence intra-page — non retenu. |
| UX-OBS-DUP-CURATOR-DIRECT | P3 | ACCEPTE/DOCUMENTE — une app vouchee curator (row sans archive_hash, cle `pid::`) et poussee direct-inconnu (cle `pid::hash`) rend 2 cartes dans 2 sections. Enracine dans la cle de dedup S75 pre-existante. Le « fix » suggere (exclure de l'ambiant tout pid present en mes-sources) ouvrirait un shadowing par pid (suppression d'une carte attaquante ≠ probleme, mais suppression d'une VERSION legitime distincte = perte) — refuse. Route en note au prochain audit. |
| UXC-2 | P3 | REFUTE (analyse main thread) — une identite abonnee ne peut pas etre simultanement « en attente » ET « observee » en regime stable : le daemon exclut les abonnes d'observed a l'ecriture (gate + primitive) ET a la lecture (re-gate snapshot), et la purge s'execute dans `subscribe()`. Seule une staleness TRANSITOIRE entre les deux queries React Query (invalidees ensemble par AddAnchorDialog) peut juxtaposer brievement les deux lignes — auto-resorbee au refetch. Accepte. |
| UXC-3 | NIT | ACCEPTE — `last_seen` transporte mais non rendu : choix assume (pas de date-formatting ; la fraicheur est portee par l'ordre freshest-first du daemon). Le champ est dans le payload pour un rendu futur. |

## Items verifies sains (extraits des notes des 5 agents)

- Bornes du rate-limit exactes (t0+59 rejete, t0+60 accepte ; horloge
  qui recule → rejet sans panic) ; TTL/rate-limit sans interference
  (48h >> 60s) ; cap jamais depasse (eviction 1-pour-1, tie-break
  deterministe (ts, pubkey)) ; pas de lock tenu a travers un await ;
  pas d'ordre de lock croise observed↔attention.
- Wire : 0 bump (FORMAT_VERSION/ANNOUNCEMENT_VERSION/DOMAIN_* tous
  intouches), enveloppe gossip intacte, 0 dep ; `observed` TOUJOURS
  emis cote Rust + `.optional()` Zod (tolerance) ; rows tolerantes
  (P37) ; `BrowseEntrySchema.strict()` + cle Rust shippent ensemble.
- Front : split sur l'ensemble DEDUPE, merge OR de la classification
  complete (le test deux-canaux existant a attrape la 1re version qui
  n'OR-ait que les flags — fixe in-cycle avant la review), hero jamais
  sur l'ambiant, section vide non rendue, cap honnete avec compteur
  « X sur N ».
- Verrous 1-5 : additive (grille superset de MES sources + section
  separee MEME page), rien de pre-rempli par defaut (prefill = action
  utilisateur explicite, remount par key), subscribe explicite,
  no-fetch absolu pour non-abonnes (teste).

## Codex reconciliation

Artefact : `post_s75_ux_arrival_codex_review.md` (sortie brute
`codex exec -o`, GPT-5.5).

- **R1 : 18 CONFIRMED + 1 GAP → OVERALL: FAIL.** Le GAP : le rate-limit
  1/min par identite n'est pas durable a travers une eviction par cap
  (l'etat du limiteur EST l'entree residente — evincee, l'identite est
  re-acceptee immediatement). Adjudication AS-DESIGNED : s'evincer exige
  d'etre la stalest du registre entier (256 identites plus fraiches =
  deja le regime flood multi-identites assume en residuel §15.1) ; UNE
  identite ne peut pas s'auto-churner (re-admise, elle est la plus
  fraiche et re-tombe sous la fenetre) ; hors flood, une entree ne sort
  que par le TTL 48h >> 60s ; un store de limiteur survivant a
  l'eviction devrait lui-meme etre cappe — meme question de
  displacement deplacee d'un niveau. Resolution : raisonnement inscrit
  au point exact du rate-check (`record_observed_directory`), 1 phrase
  THREAT_MODEL, et test pinnant la re-admission-puis-re-rate-limit
  (queue de `observed_registry_cap_evicts_stalest`). Tests observed
  4/4 re-verts.
- **R2 : 23 CONFIRMED, 0 GAP, OVERALL: PASS** — couvre les 7 livrables
  (registre + /nodes observed + catalog-backed + split front + prefill
  + Zod + THREAT_MODEL), les tests des deux cotes, et l'honnetete des
  residuels documentes.

Verdict promu PASS. 2 P1 review fermes in-cycle (catalog-backed +
doc honnete/self-guard), 1 GAP Codex adjudique-documente-teste,
12 P2/P3/NIT traites, 3 acceptes documentes.
