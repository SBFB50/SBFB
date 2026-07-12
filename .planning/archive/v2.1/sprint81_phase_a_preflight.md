# Sprint 81 — Phase A — Preflight (deep, 5 scans + 5 vérifications adversariales)

**Date** : 2026-07-02
**Phase** : A — Fix convergence materializer (wf4) dans `nexus-coordinator-rs` (`feed_materializer.rs` + `public_feed.rs`). **0-bump wire strict** (JCS / `DOMAIN_FEED_V1` / `FEED_FORMAT_VERSION` intacts), **AVANT le bump iroh** (bisectabilité : un échec post-bump = iroh, pas le materializer).
**Verdict** : **PLAN-ADAPT**

> La décision Day-0 **D5 (fix in-sprint Phase A avant bump) est CONFIRMÉE, non contredite** : aucun finding ne touche un invariant gelé, iroh reste strictement hors-périmètre, le 0-bump est authentique (S1b + S4 SUPPORTS-EXECUTE, adversariaux confirment 0-dep / 0-bump). Mais la **formulation LITTÉRALE des livrables gelés cache ≥5 pièges** qui, implémentés au pied de la lettre, produiraient un bug ou une régression. Le plan doit être corrigé avec evidence OSS + code concrète, **sans toucher aucun Day-0** — d'où PLAN-ADAPT (pas EXECUTE, pas DESIGN-CONFLICT). L'approche corrigée est figée au §3.

> Orchestration : 5 scans (S1a OSS prior-art / S1b Deps-CVE / S2 décisions historiques / S3 threat model / S4 wire-invariants) + 5 vérifications adversariales, agents Opus 4.8 1M. Faits load-bearing **re-vérifiés en main-thread** avant synthèse contre le code réel (tip `b1f174e`) : `apply()` overwrite inconditionnel ReleasePublished (feed_materializer.rs:54-58) ET SourceBecameStale (:60-73) ; `materialize_full` fold non-vérifié (:95-101) ; `materialize_verified` fold en ordre DB APRÈS `verify_chain` (:107-115, verify_chain @109) ; `verify_entry` sans check prev_hash (public_feed.rs:591-631, doc :588-590) ; `verify_chain` per-auteur order-independent + Err all-or-nothing (:660-698, Err :688-695) ; `FEED_MAX_FUTURE_SECS`=30j (:636) ; seq AUTOINCREMENT local (db.rs:158) + `ORDER BY seq ASC` (:1259) ; ingest distant seq:0 + `created_at=feed_entry.timestamp` signé (feed_sync.rs:358/365) + `verify_entry` appelé AVANT insert avec warn-only drop (:269-272).

---

## 1. Synthèse des 5 scans (corrections adversariales intégrées)

| Scan | Objet | Verdict scan | Adversarial | Apport load-bearing (net des corrections) |
|---|---|---|---|---|
| **S1a** OSS prior-art | Résolution d'ordre déterministe sur log signé chaîné par prev_hash | SUPPORTS-PLAN-ADAPT | **CONFIRMED** | La primitive = **forêt de chaînes per-auteur** (prev_hash PER-AUTHOR, public_feed.rs:507-510), pas une chaîne globale. `verify_chain` DÉJÀ order-independent + rejette fork/gap. Divergence = le **FOLD en ordre seq local**. D5 = bonne direction (= Matrix state-res v2 `lexicographical_topological_sort`, git `--topo-order`, SSB buffering, automerge/yjs `(counter,actorId)`), mais 3 P1 sur la formulation littérale. |
| **S1b** Deps/CVE | Dépendances du chemin de fix | SUPPORTS-EXECUTE | **CONFIRMED** | **0 dep, 0 changement Cargo.toml**. Tri topo trivial à la main (`verify_chain` démontre déjà le pattern by_prev + walk itératif, 0 récursion). Ed25519 transitif via nexus-core-rs. cargo-deny CI actif, 0 advisory sur le chemin. `rand` = advisory ignorée mais HORS chemin (test-only). |
| **S2** Décisions historiques | Cohérence du fix avec l'historique décisionnel | SUPPORTS-PLAN-ADAPT | **CONFIRMED** | L'order-independence est **déjà actée pour `verify_chain`** (cedadd3 S64, « résout iroh-docs LiveEvent out-of-order ») ; le materializer n'a **jamais** été porté = GAP de suivi, pas un choix. L'overwrite inconditionnel = **non documenté** (aucun corps de commit ne le justifie). Le fold non-vérifié EST documenté (§5.1 skip-verify perf-optim). |
| **S3** Threat model | Nouvelles surfaces de menace du fix | SUPPORTS-PLAN-ADAPT | **PARTIAL** (cœur fiable ; 3 corrections de complétude/sévérité) | 3 menaces non couvertes §10 : (1) projection **non-consommée en prod** → fix = baseline, pas correction d'un symptôme live ; (2) fold-après-verify_chain = **régression dispo all-or-nothing** ; (3) garde monotone-ts + **absence d'autorisation auteur→project_id** = hijack **persistant** (+30j). |
| **S4** Wire-invariants | 0-bump producteur→consommateur | SUPPORTS-EXECUTE | **PARTIAL** (cœur fiable ; 3 corrections de complétude) | 0-bump **authentique** : prev_hash déjà dans `FeedEntry` + `FeedEntryCanonical` + `to_canonical` (déjà signé + dans entry_hash). Tie-break 100% content-derived (`created_at=feed_entry.timestamp` signé, feed_sync.rs:365). **Aucune migration M20**. Omission trace : le writer d'ingest distant `feed_sync.rs` EST l'origine du hors-ordre. |

**Aucune claim REFUTED.** Les 2 verdicts adversariaux PARTIAL (S3, S4) portent sur des **corrections de complétude/sévérité** qui RENFORCENT le cœur des scans, jamais ne le réfutent. Les corrections sont intégrées ci-dessous et au §3.

---

## 2. Corrections adversariales intégrées (une claim corrigée n'est jamais gardée telle quelle)

| Origine | Claim scan | Correction intégrée |
|---|---|---|
| S1a-adv | « feed_sync.rs:234-235 documente l'arrivée d'entrées hors-ordre » | **Ligne-preuve remplacée** : le vrai témoin out-of-order = `test_verify_chain_out_of_order_insertion` (public_feed.rs:1301-1354, cmt :1348 « simulates out-of-order iroh-docs arrival »). Le fond (verify_entry existence-check casserait la convergence) tient. |
| S1a-adv | Déterminisme du tuple laissé implicite | **Confirmé explicitement** : `created_at=feed_entry.timestamp` (timestamp SIGNÉ de l'auteur, feed_sync.rs:365) ∈ `FeedEntryCanonical` → tuple 100% content-derived → identique cross-noeud. |
| S2-adv | INFO #6 « hits UNIQUEMENT 2 fichiers » | **Dénombrement corrigé en 3** (feed_materializer.rs + lib.rs:19 + doc-comment public_feed.rs:7). **Conclusion (0 consommateur runtime) INCHANGÉE et load-bearing.** |
| S2-adv | P2 #4 (SourceBecameStale) + P2 #5 (ordre intra-auteur) sous-cotés | **Élevés vers P1-adjacent** : un tri (timestamp,author,hash) PLAT casserait un test EXISTANT `test_materialize_source_stale` (feed_materializer.rs:304-317, release+stale même seconde → tie-break pourrait appliquer Release EN DERNIER → source_stale=false → assertion cassée). Le fix DOIT clef l'ordre intra-auteur sur la chaîne prev_hash. |
| S3-adv | Finding #1 evidence « multi_daemon.rs appelle materialize_* » | **Corrigé** : multi_daemon.rs ne fait qu'un COMMENTAIRE (:85), 0 appel hors feed_materializer.rs. La projection est encore PLUS non-consommée → finding #1 renforcé. |
| S3-adv | Couverture limitée à `latest_release_hash` | **Élargie** : `apply()` est order-dependent AUSSI pour `source_stale`/`published`/`last_updated` (2 arms). La garde monotone doit couvrir la transition Release↔Stale, pas seulement le hash. Le T1 asserte la `PublicRegistryView` ENTIÈRE (`#[derive(PartialEq)]` :16 le permet). |
| S3-adv | #2/#3 notés P1 « présent » | **Requalifiés « P1 au câblage »** : impact LIVE présent = nul (0 consommateur) ; ce sont des design-time defects conditionnés au câblage futur du materializer. Le verdict PLAN-ADAPT reste calibré (aucun Day-0 contredit). |
| S4-adv | Trace producteur→consommateur amputée | **Complétée** : `verify_entry` est un consommateur LIVE des entrées fraîches du wire (feed_sync.rs:269, AVANT insert). Une garde de FORMAT prev_hash tirerait à CHAQUE ingest distant → doit tolérer le hors-ordre. |
| S4-adv | « tri topo prev_hash porte l'ordre global » | **Précisé** : prev_hash ne donne AUCUN ordre entre auteurs différents. La convergence inter-auteurs (2 auteurs, même project_id) vient ENTIÈREMENT du tie-break (timestamp,author,hash), pas de la topologie. |

---

## 3. Décision load-bearing FIGÉE — approche D5 corrigée (le code suit CE §, pas le libellé du kickoff)

> **Cadre invariant (aucun ne bouge)** : D5 fix-in-Phase-A avant bump = respecté. iroh hors-périmètre. 0-bump wire strict (aucun `FEED_FORMAT_VERSION`/`DOMAIN_*_V1`/`*_ANNOUNCEMENT_VERSION` touché ; prev_hash/timestamp/author/entry_hash sont des champs DÉJÀ produits+signés+stockés). 0 dep, 0 migration M20. Le REJET dur des forks intra-auteur (équivocation) est CONSERVÉ.

### 3.1 Ordre du fold = k-way merge déterministe sur la forêt per-auteur (PAS un tri plat)
Le DAG est une **forêt de chaînes causales per-auteur** (prev_hash PER-AUTHOR, public_feed.rs:507-510). L'algorithme correct = **fusion k-way des K chaînes** (Kahn sur le ready-set = têtes d'auteurs), analogue à Matrix `lexicographical_topological_sort` et git `--topo-order` :
- **Intra-auteur** : l'ordre autoritaire est la **chaîne prev_hash** (walk genesis→ via le pattern `by_prev` déjà présent dans `verify_chain` :670-686), **JAMAIS le timestamp brut** (aucun plancher passé n'existe — seul `FEED_MAX_FUTURE_SECS`=30j plafonne le futur ; un skew/adversaire pourrait ré-ordonner intra-chaîne).
- **Inter-auteurs** : le tie-break `(timestamp, author_pubkey, entry_hash)` départage UNIQUEMENT les têtes concurrentes du ready-set. 100% content-derived (`created_at=feed_entry.timestamp` signé) → identique cross-noeud. `(timestamp, author)` totalise déjà ; `entry_hash` est quasi-mort (utile seulement si on tolérait un fork intra-auteur, ce qu'on rejette).

### 3.2 Ordering ≠ verification (NE PAS forcer verify_chain dans materialize_full)
Le livrable « fold APRÈS verify_chain » pris au pied de la lettre **contredit §5.1** (PUBLIC_FEED_SPEC : `materialize_full` MAY skip verify — perf-optim documentée) ET **ne fixe rien** : `materialize_verified` (:107-115) folde AUSSI en ordre DB → **déjà divergent**. Le fix porteur = appliquer l'**ordre topo-dérivé (§3.1) à la logique de projection**, de sorte que `materialize_full` ET `materialize_verified` convergent, **sans** injecter `verify_chain` dans `materialize_full`. Ordering et verification sont **orthogonaux**.

### 3.3 Disponibilité : préfixe per-auteur, pas d'Err globale (all-or-nothing interdit)
`verify_chain` est **tout-ou-rien cross-auteur** (Err au 1er auteur cassé/forké, :688-695). Le mettre sur le chemin de projection = **régression dispo** : un prédécesseur en retard (le scénario ciblé !) ou un auteur qui forke SA chaîne ferait échouer la vue de TOUS les projets. Le fold applique le **plus long préfixe atteignable PAR AUTEUR** (walk genesis→ tant que le maillon suivant est présent) et laisse les **suffixes orphelins non-appliqués** (appliqués quand le trou se comble). Convergence eventual garantie : même set présent → même préfixe déterministe. Les forks intra-auteur restent **rejetés durs** (équivocation), en isolation per-auteur (un auteur malveillant ne DoS pas les autres).

### 3.4 Garde monotone : clé = rang de chaîne (Lamport), PAS timestamp mural ni seq, sur LES DEUX arms
La garde monotone doit clef sur la **MÊME clé que l'ordre du fold** (rang/profondeur de chaîne per-auteur + tie-break). Interdits :
- **timestamp mural** = antidatable (+30j), **non-monotone avec la causalité** → une garde-ts peut VETO une entrée causalement postérieure à ts plus bas → perte d'update légitime ; **aggravé** par l'absence d'autorisation auteur→project_id (`apply()` ne vérifie pas author==owner, feed_materializer.rs:43-58) → un attaquant post-daté gagne ET verrouille le hijack jusqu'à 30j (**pire qu'aujourd'hui** où le dernier arrivé gagne, donc réversible).
- **seq local** = divergent par construction (db.rs:158).

Si le fold applique déjà dans l'ordre total déterministe, **le last-write-wins DANS cet ordre EST la garantie monotone** ; une garde explicite n'est utile que si elle porte sur cette clé. La garde doit couvrir **ReleasePublished ET SourceBecameStale** (les 2 écrivent `last_updated=entry.timestamp` inconditionnellement, :54-58 / :60-73) sinon `source_stale`/`last_updated` restent non-convergés sur un interleave Release/Stale. **Protège le test existant** `test_materialize_source_stale` (ordre intra-auteur prev_hash → Release puis Stale déterministe).

### 3.5 verify_entry : garde de FORMAT uniquement, jamais existence/linkage
`verify_entry` est une fonction **single-entry pure** (public_feed.rs:591-631, sans contexte prédécesseur) appelée à l'ingest distant **AVANT insert** (feed_sync.rs:269, warn-only drop :270-272). Si le check devient « prev_hash référence une entrée existante », toute entrée arrivant AVANT son prédécesseur (le cas normal iroh-docs) serait **rejetée → convergence IMPOSSIBLE** (= la feature elle-même). Le check ajouté = **FORMAT seul** (`prev_hash == GENESIS_PREV_HASH` ou hex-64 bien-formé). La LIAISON reste le job de la projection/`verify_chain` sur l'ENSEMBLE. NB : prev_hash ∈ `to_canonical` (:199) → une altération est DÉJÀ rejetée par le recompute `entry_hash` (:601-607) ; le format-check est un **durcissement mineur** (rejette un prev_hash vide/malformé signé cohérent), pas le cœur du fix. **Ne pas sur-promettre** dans la doc.

### 3.6 Incrémental non-sound sous ordre content-dérivé
Une fois l'ordre dérivé du contenu, une entrée arrivant TARD mais triant AVANT le frontier ne peut PAS être `apply()`-ée en fin (`materialize_incremental` :147-149) sans diverger du full-rebuild. `apply()` reste idempotent (LWW + dédup entry_hash), le problème est la **réinsertion EN MILIEU d'ordre**. Correction : `materialize_incremental` **détecte un ré-ordonnancement** (entrée entrante triant avant le frontier) et **FULL-rebuild** dans ce cas. Le test `test_cursor_restart_consistency` (:417-434) et `test_cursor_persist_resume` (`view2==full` :347) ne testent QUE l'arrivée en-ordre → le T1 doit exercer AUSSI le chemin incrémental.

### 3.7 Doc-contract PUBLIC_FEED_SPEC à mettre à jour (frontière protocole, canon S80)
Le tri topo remplace l'ordre `seq` → **§6 « Ordering » (ligne 200 « ordered by seq »)** devient faux → MAJ (règle d'ordre topologique + tie-break + garde monotone). Si `materialize_full` change de sémantique, ajuster la phrase **§5.1** skip-verify. Sinon code et spec divergent (frontière docs-contract).

---

## 4. Approche d'implémentation

**Surfaces (2 fichiers, 0-bump)** :
1. `crates/nexus-coordinator-rs/src/feed_materializer.rs` — nouvelle fonction d'ordonnancement (k-way merge per-auteur §3.1) appliquée dans `materialize_full` (:95-101) ET `materialize_verified` (:107-115) ; garde monotone sur clé de rang dans/autour de `apply()` couvrant les 2 arms (§3.4) ; détection de ré-ordonnancement dans `materialize_incremental` (§3.6). Réutilise le pattern `by_prev` de `verify_chain` (0 dep, `std::collections::HashMap` déjà importé :10).
2. `crates/nexus-coordinator-rs/src/public_feed.rs` — `verify_entry` (:591) : garde de FORMAT prev_hash (§3.5). NE PAS toucher la version-garde :592 ni le canonical.
3. `docs/protocol/PUBLIC_FEED_SPEC.md` — §6 Ordering + §5.1 (§3.7).

**Tests attendus (+4..6 Rust)** — alimentent le sous-test T1 (2) « convergence ingest hors-ordre » :
- **Convergence hors-ordre** : insérer les mêmes entrées chaînées dans 2 ordres opposés (2 DB) → asserter `PublicRegistryView` **byte-identique** (`assert_eq!` sur la vue ENTIÈRE, pas seulement `latest_release_hash` — inclut `source_stale`/`published`/`last_updated`). Exercer `materialize_full` ET `materialize_incremental`.
- **Tie-break déterministe cross-auteur** : 2 auteurs, même project_id, ordres d'arrivée opposés → même gagnant (par `(timestamp,author,hash)`).
- **Garde monotone** : republish honnête à rang supérieur gagne ; entrée antidatée à rang inférieur ne veto pas l'update légitime (protège contre le hijack persistant).
- **prev_hash format rejeté** : `verify_entry` rejette un prev_hash malformé, ACCEPTE une entrée hors-ordre (prédécesseur absent) — garantit que l'ingest hors-ordre n'est PAS cassé.
- **Non-régression** : `test_materialize_source_stale`, `test_cursor_persist_resume` (`view2==full`), `test_verify_chain_out_of_order_insertion` restent verts (à CONSERVER, pas des zombies).

**Gate** : commit propre dédié `fix(coordinator): Sprint 81 Phase A — <titre> (wf4)`, **JAMAIS** dans le commit de bump iroh (Phase B). 0-bump vérifié : `FEED_FORMAT_VERSION`/`DOMAIN_FEED_V1` intacts, aucune migration.

---

## 5. Risques résiduels / cibles adversariales

1. **« Le materializer n'est consommé nulle part »** (S3-F1, S2-INFO#6, confirmé adv) → le fix est une **baseline de bisectabilité préventive**, PAS la correction d'un symptôme Browse live. Cadrage HONNÊTE à porter au commit body : la « divergence PublicRegistryView » est **LATENTE-par-construction**. Le T1 asserte la **fonction de projection directement**, pas un endpoint live. Le Browse live dérive `latest_release_hash` d'AUTRES surfaces (ProjectAnnouncement gossip / provenance_records) — leur convergence propre est HORS Phase A.
2. **Tri plat par tuple casse un test existant** → obligation §3.1 (ordre intra-auteur = chaîne prev_hash) ; vérifié contre `test_materialize_source_stale` (:304-317).
3. **Garde-ts vs fold causal = contradiction** → obligation §3.4 (garde sur la clé de rang, pas ts mural).
4. **verify_entry en check d'existence casse l'ingest hors-ordre** → obligation §3.5 (FORMAT seul) ; `verify_entry` est un consommateur LIVE (feed_sync.rs:269).
5. **Régression dispo all-or-nothing** → obligation §3.3 (préfixe per-auteur, pas d'Err globale).
6. **Incrémental divergent sous réordonnancement** → obligation §3.6 (détection + full-rebuild) ; T1 exerce le chemin incrémental.
7. **Borne DoS du fold** (S3-P2, P2) : `replay_all` charge tout sans LIMIT (public_feed.rs:561-583). Pas de `MAX_FEED_ENTRIES` (précédent `MAX_ARCHIVE_ENTRIES=4096` S75). **Différé** (impact nul tant que non-consommé) → carry P2, garantir au moins un tri topo linéaire déterministe.
8. **Fail-mode ingest warn-only drop** (S3-P2) : anti-pattern visé par S81 A2 (distinct). Phase A touche `verify_entry` mais NE change PAS ce fail-mode → hors-scope A, tracé A2.

---

## 6. Scope

**DANS le scope S81 Phase A** :
- Ordonnancement k-way merge per-auteur (§3.1) dans `materialize_full` + `materialize_verified`.
- Garde monotone sur clé de rang, 2 arms (§3.4).
- Préfixe per-auteur sans Err globale + rejet dur des forks intra-auteur (§3.3).
- `verify_entry` garde de FORMAT prev_hash (§3.5).
- Détection de ré-ordonnancement dans `materialize_incremental` (§3.6).
- MAJ doc-contract PUBLIC_FEED_SPEC §6 + §5.1 (§3.7).
- +4..6 tests (convergence hors-ordre full+incrémental, tie-break, garde monotone, prev_hash format) + non-régression des tests existants.
- 0-bump wire, 0 dep, 0 migration, commit dédié séparé du bump.

**HORS scope (carry / différé / autre phase)** :
- Autorisation auteur→project_id (binding author==owner) → hors wf4, carry threat model §10 (le vrai correctif du hijack cross-auteur).
- Borne `MAX_FEED_ENTRIES` sur le fold → carry P2.
- Fail-mode warn-only drop → Phase A2 (self-heal ×2 runtime.rs:2518/:2606, bug DISTINCT).
- Bump iroh, pins =1.0.1 → Phase B.
- Câblage prod du materializer (aucun consommateur aujourd'hui) → post-Phase A.

---

## 7. Repères kickoff vs code réel (tip `b1f174e`)

**TOUS les repères Phase A du kickoff sont EXACTS — aucun périmé** (contraste explicite avec le self-heal `:2617→:2518/:2606` périmé AILLEURS, hors Phase A). Vérifié en main-thread :

| Repère kickoff | Code réel | Statut |
|---|---|---|
| feed_materializer.rs overwrite ReleasePublished `:54-58` | `status.published=true` @54 → `status.last_updated=entry.timestamp` @58 | **EXACT** |
| feed_materializer.rs doc materialize_full `:89-94` | doc `/// ... does NOT verify the hash-chain` | **EXACT** |
| feed_materializer.rs fold `:95-101` | `pub fn` @95, `replay_all` @96, `new` @97, boucle apply 98-100, `Ok` @101 | **EXACT** |
| public_feed.rs verify_entry sans prev_hash `:588-591` | doc « Does NOT check prev_hash linkage » :588-590, signature `pub fn verify_entry` @591 | **EXACT** |
| public_feed.rs verify_entry `~:585-603` (plage kickoff) | fonction complète = **:585-631** (le corps s'étend jusqu'à :631) | **léger raccourci, NON périmé** |

**À COMPLÉTER au-delà du kickoff** (non listés, load-bearing) : `materialize_verified` `:107-115` folde AUSSI en ordre DB après `verify_chain` (@109) → **divergent lui aussi** (le kickoff ne cite que `materialize_full`). `SourceBecameStale` `:60-73` = 2e arm à overwrite inconditionnel → la garde monotone doit le couvrir. Repères de support vérifiés : db.rs:158 seq AUTOINCREMENT ; db.rs:1256-1259 `ORDER BY seq ASC` ; feed_sync.rs:358 seq:0 + :365 `created_at=feed_entry.timestamp` signé + :269 `verify_entry` AVANT insert (warn-only :270) ; public_feed.rs:660-698 `verify_chain` (Err :688-695) ; :636 `FEED_MAX_FUTURE_SECS`=30j ; :507-510 prev_hash per-auteur ; :1301-1354 `test_verify_chain_out_of_order_insertion` (à CONSERVER). Consommateurs prod de `materialize_*`/`PublicRegistryView` : **0** (grep repo-wide = feed_materializer.rs def+tests + lib.rs:19 mod-decl + public_feed.rs:7 doc-comment).

---

## Verdict: PLAN-ADAPT

**Justification (§4.5.7)** :
- **PAS de DESIGN-CONFLICT** : aucun finding confirmé ne contredit une décision Day-0/PO gelée. D5 (fix in-sprint Phase A avant bump) est CONFIRMÉE au code par les 5 scans ; iroh strictement hors-périmètre ; 0-bump authentique (S1b + S4 SUPPORTS-EXECUTE, adversariaux confirment 0-dep/0-bump/0-migration). Les seules tensions sont avec (a) le **libellé littéral** des livrables gelés et (b) des **spec-docs modifiables** (PUBLIC_FEED_SPEC §5.1/§6) — jamais un invariant gelé.
- **PLAN-ADAPT (pas EXECUTE)** : l'approche D5 doit être corrigée avec evidence concrète — l'ordre du fold doit être un **k-way merge per-auteur** (pas un tri plat), `verify_entry` un **check de FORMAT** (pas d'existence), la garde monotone sur une **clé de rang** (pas timestamp/seq), le fold un **préfixe per-auteur sans Err globale**, l'incrémental détectant le réordonnancement, et le doc-contract mis à jour. L'approche corrigée est figée au §3 ; **le code suit ce §, pas le libellé du kickoff**.

**Findings P1+ à traiter pendant la phase** :
1. **[P1] verify_entry = FORMAT-only** (§3.5) — un check d'existence casserait l'ingest hors-ordre (feed_sync.rs:269 AVANT insert).
2. **[P1] Fold en ordre topo-dérivé, PAS « fold après verify_chain »** (§3.2) — sinon ne fixe rien (`materialize_verified` divergent aussi) + contredit §5.1.
3. **[P1] k-way merge/Kahn per-auteur, intra-auteur = chaîne prev_hash** (§3.1) — un tri plat casse `test_materialize_source_stale` + réordonne sous ts non-monotone.
4. **[P1] Garde monotone sur clé de rang (Lamport), les 2 arms** (§3.4) — ts mural = hijack persistant + veto d'update légitime ; seq = divergent.
5. **[P1] Préfixe per-auteur sans Err globale** (§3.3) — verify_chain all-or-nothing = régression dispo DoS-able.
6. **[P1] Doc-contract PUBLIC_FEED_SPEC §6 + §5.1** (§3.7) — le tri topo contredit « ordered by seq » (frontière protocole).

**Carry / hors-scope tracés** : autorisation auteur→project_id (§10 threat model) ; borne `MAX_FEED_ENTRIES` (P2) ; fail-mode warn-only drop (Phase A2) ; incrémental non-sound testé mais optimisation abandonnable si nécessaire (§3.6, P2).
