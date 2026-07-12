# Review — Sprint 81 Phase A : Fix convergence materializer (wf4) — `nexus-coordinator-rs`

**Date :** 2026-07-02
**Périmètre :** working tree NON COMMITÉ (tip `b1f174e`, branche master) — **3 fichiers modifiés + 1 artefact untracked**, 0-bump wire strict, AVANT le bump iroh (Phase B).
Modifiés (`git diff`) : `crates/nexus-coordinator-rs/src/feed_materializer.rs` (+631/−50 ; fold k-way merge déterministe per-auteur + safe-append incrémental + 6 tests) ; `crates/nexus-coordinator-rs/src/public_feed.rs` (garde de FORMAT prev_hash dans `verify_entry` + 1 test) ; `docs/protocol/PUBLIC_FEED_SPEC.md` (§5.1 orthogonalité + §6 Ordering storage≠projection + §7 reordering fallback + vecteur 14).
Untracked (lu en ENTIER) : `.planning/active/sprint81_phase_a_preflight.md` — contrat de conformité, **verdict PLAN-ADAPT**, §3 approche corrigée figée (6 obligations P1), §5 risques résiduels, §6 scope.
**Autorité de conformité :** PLAN-ADAPT — le code suit le §3 du préflight, PAS le libellé littéral du kickoff.
**Orchestration :** review deep pré-Codex — 7 dimensions (correctness ligne-à-ligne, suites+delta, branch-coverage sémantique, scope cuts + 0-bump, research-grounding conformité §3, sécurité DEEP, livrables+patterns+test-acteur docs-contrat) → vérification adversariale par-finding (CONFIRMED / DOWNGRADED / REFUTED) → synthèse réconciliée sur le DIFF réel. Faits load-bearing re-vérifiés en main-thread contre le code réel (feed_materializer.rs:42-116/180-291/340-405, public_feed.rs:561/587-620, PUBLIC_FEED_SPEC.md:163-231, THREAT_MODEL.md:549-560).

---

## Conclusion (pré-Codex) : PASS-PENDING

Le diff livre l'intégralité du contrat Phase A tel que figé au §3 du préflight (PLAN-ADAPT). Les **6 obligations P1** sont toutes FIDÈLES au code réel avec evidence vérifiée : (1) `verify_entry` = FORMAT-only (jamais existence) ; (2) fold en ordre topo-dérivé partagé par `materialize_full` ET `materialize_verified` (jamais « fold après verify_chain ») ; (3) k-way merge per-auteur, intra-auteur = chaîne prev_hash (jamais tri plat) ; (4) garde monotone LWW-dans-l'ordre couvrant les 2 arms Release+Stale (jamais ts mural ni seq) ; (5) préfixe per-auteur sans Err globale (fork = hard-stop isolé) ; (6) doc-contract PUBLIC_FEED_SPEC §6/§5.1/§7/vecteur 14 à jour.

Le cœur algorithmique est **CORRECT et convergent** : même jeu d'entrées ⇒ même `PublicRegistryView` (métadonnées comprises via `#[derive(PartialEq)]`) sur tous les chemins normaux. La soundness du safe-append incrémental (incrémental == full-rebuild) a été démontrée formellement en dimension 1 et re-vérifiée adversarialement. **0-bump authentique** (FEED_FORMAT_VERSION / DOMAIN_FEED_V1 / JCS / canonical intacts), **0 dep** (seul ajout d'import = std `HashSet`/`OnceLock`), **0 migration DB**, **iroh strictement hors-périmètre**, commit dédié séparé du bump. Delta +7 tests CONFIRMÉ exact contre le diff (6 feed_materializer + 1 public_feed).

**Aucun P0, aucun P1.** Conformément au critère §4.5.7 : PASS-PENDING (review OK, Codex pas encore passé — **jamais un verdict committable**). Le seul P2 CONFIRMED est un **trou de couverture de branche** (le conjoint monotone-guard `(c)` n'est jamais le facteur décisif testé du fallback incrémental), impact live nul (materializer 0-consommateur prod), recommandé en fix bon marché avant le gate Codex, non bloquant. Les deux P2 sécurité-deep ont été **DOWNGRADED à P3** après vérification adversariale (l'un = doc-honnêteté résiduelle sur `T-FEED-CLOCK-SKEW`, l'autre = borne DoS subsumée par le carry `MAX_FEED_ENTRIES` déjà acté).

**Décompte findings (après filtrage adversarial) :** P0 = 0 · P1 = 0 · P2 = 1 (CONFIRMED) · P3 = 10 (dont 2 DOWNGRADED depuis P2) · **0 finding réfuté**. Aucun viol cardinal, aucune régression cachée, aucun bump wire/Day-0, aucune nouvelle surface réseau prod.

---

## Résultats des suites (§7.4) — chiffres exacts fournis par le main-thread (NON relancés)

- **Rust Windows natif :** `fmt --check` clean · `clippy --workspace --all-targets -D warnings` clean · nextest **2021/2021** 0-skip (baseline S80 = 2014, **delta +7**) · doctests OK · `cargo build -p nexus-shell-daemon --release` OK.
- **Docker canonique `sbfb-ci` rust:1.94 :** `fmt` clean · `clippy` clean · nextest **2025/2025** (baseline 2018, **delta +7**) · doctests OK.
- **Cohérence Win/Docker :** les 7 tests neufs sont tous dans `nexus-coordinator-rs`, aucun ne porte de gate `#[cfg]` (grep `+#[cfg` sur le diff = vide) → delta +7 identique Win (2014→2021) et Docker (2018→2025), aucune divergence possible par construction. Re-run local du crate cible : `cargo nextest run -p nexus-coordinator-rs --locked` = 339/339 0-skip (338 unit + 1 intégration), 7/7 nouveaux tests PASS par filtre nom.
- **web/ :** lint 0 errors (5 warnings préexistants) · tsc clean · Vitest **411/411** (6 flaky de CHARGE observés en runs parallèles — `GpuConsentDialog` timeouts jsdom ; 17/17 verts ×2 en solo + 411/411 à machine calme ; code web INCHANGÉ depuis l'audit gate vert du matin) · coverage **87.27/79.01/86.02/88.59 ≥ 85/78/85/85** · build OK · size 6/6 · scan-en-strings clean.
- **tools/factory-operator :** Vitest **201/201** · gates discipline 6/6 · size budgets OK.
- **Delta tests de la phase :** **+7 Rust** (6 feed_materializer convergence + 1 public_feed prev_hash format). Le préflight §4/§6 attendait +4..6 — dépassement de +1 documenté et justifié (couverture fork-isolation + orphan-gap séparées) ; la cohérence annonce(+7)/réel(+7) est intacte, ce n'est PAS une divergence de delta.

---

## Résumé des 7 dimensions

| # | Dimension | Résultat |
|---|---|---|
| 1 | Correctness ligne-à-ligne (materializer wf4) | OK — cœur CORRECT et convergent ; 6 obligations P1 respectées ; soundness safe-append (incrémental == full) démontrée formellement ; `max_applied_key` = MAX de toutes les clés appliquées (`is_none_or` :104). **2 P3 latents** (walk sans visited-set / branche hash-mismatch all-or-nothing pré-existante). 0 P0/P1/P2. |
| 2 | Suites §7.4 + cohérence delta | CLEAN — +7 exact (6+1), 0 test supprimé/renommé, additif strict, 339/339 crate cible. 0 finding. |
| 3 | Branch-coverage sémantique | FINDINGS — 4 chemins centraux de `ordered_for_fold_from` exercés avec asserts probants (fork, gap/orphelin, tie-break, rang>ts antidaté) ; verify_entry FORMAT 4 formes + accepté hors-ordre. **1 P2** (conjoint monotone-guard `(c)` jamais décideur testé) **+ 2 P3** (branche duplicate inatteignable via DB / early-return `is_empty` neuf non exercé). |
| 4 | Scope cuts + invariants 0-bump | CLEAN — 0-bump authentique (seule surface wire = durcissement FORMAT prev_hash), 0 dep, 0 migration, iroh hors-périmètre, hors-scope §6 absents (0 binding auteur→project_id, 0 MAX_FEED_ENTRIES, feed_sync.rs non touché), pré-launch policy OK (0 zombie legacy-decode), préflight untracked porte PLAN-ADAPT. 0 finding. |
| 5 | Research-grounding (conformité préflight §3) | FINDINGS — les 6 obligations P1 FIDÈLES avec evidence ; convergence prouvée (safe-append ⇒ incrémental == full). **3 P3 informationnels** (safe-append dégrade en rebuild pour bursts même-ts / garde `max_applied_key` non exercée sur vue propre / `PartialEq` sur champs bookkeeping) — tous correctness-safe, tolérés par §3.6 (incrémental « optimisation abandonnable »). |
| 6 | Sécurité DEEP (menaces S3) | FINDINGS — crypto saine, isolation d'équivocation OK, 0 fuite d'état (champs privés, 0 Serialize), garde FORMAT non sur-promise. **2 findings DOWNGRADED P2→P3** : tie-break ts = hijack cross-auteur durable (vrai correctif author→owner correctement carry ; résiduel = doc-staleness `T-FEED-CLOCK-SKEW`) ; k-way merge O(N·K) + self-fork gèle le fast-path (borne DoS subsumée par carry `MAX_FEED_ENTRIES`). |
| 7 | Livrables + Patterns + Test-acteur docs-contrat (§6.12) | OK — livrables présents et fidèles, Named-constants respecté (`GENESIS_PREV_HASH` réutilisé partout, 0 magic string dans le diff), langue EN/FR conforme, 0 commentaire de provenance-futur, AUCUNE frontière NEUVE (materialize_* déjà `pub`, signature verify_entry inchangée, 3 champs PublicRegistryView PRIVÉS). **1 P3** (liste §5.1 remote-sync n'indexe pas la garde de FORMAT prev_hash). |

---

## Conformité au préflight (PLAN-ADAPT) — les 6 obligations P1 tenues

| Obligation §3 | État | Évidence (working tree) |
|---|---|---|
| §3.1 k-way merge per-auteur, intra-auteur = chaîne prev_hash (jamais tri plat) | CONFORME | walk prev_hash `feed_materializer.rs:213-233` ; k-way merge sur clé `(timestamp, author_pubkey, entry_hash)` `:242-273` (comparaison `:258`) ; tie-break ne départage QUE les têtes du ready-set. |
| §3.2 fold en ordre topo-dérivé, PAS « fold après verify_chain » ; `materialize_full` ET `materialize_verified` convergent | CONFORME | les deux passent par `fold_all` (`:283-291` ; `materialize_full` :293-, `materialize_verified` folde via le même `fold_all` après `verify_chain` séparé) ; ordering ⊥ verification. |
| §3.3 préfixe per-auteur, pas d'Err globale ; fork = hard-stop isolé | CONFORME | gap/orphelin `:231-235` ; fork intra-auteur hard-stop `:225-229` ; aucune Err globale sur `materialize_full` ; `test_fork_isolation_no_global_error` (A stoppé à a1, B intact, `materialize_verified` reste Err). |
| §3.4 garde monotone sur clé de rang, 2 arms Release+Stale (jamais ts mural ni seq) | CONFORME | LWW-dans-l'ordre via `apply_in_order` `:99-107` ; `apply` couvre les 2 arms ; `max_applied_key` = MAX (`is_none_or` :104) sert le safe-append incrémental ; `test_out_of_order..._full` asserte `published` ET `!source_stale`. |
| §3.5 `verify_entry` = garde de FORMAT seule, jamais existence/linkage | CONFORME | rejet si `prev_hash != GENESIS_PREV_HASH` et pas hex-64 minuscule `public_feed.rs:605-616` ; `test_verify_entry_prev_hash_format` prouve qu'une entrée hors-ordre (prédécesseur absent) est ACCEPTÉE. |
| §3.6 détection réordonnancement incrémental → full-rebuild déterministe | CONFORME | `append_is_sound` conjonction `has_unapplied` + no-gap/fork/dup + `max_applied_key` strict `:373-380` → sinon `materialize_full` `:391-397` ; `test_out_of_order_ingest_converges_incremental` (:822) exerce le chemin incrémental. |
| §3.7 doc-contract PUBLIC_FEED_SPEC §6 + §5.1 (+ §7 + vecteur 14) | CONFORME (1 P3 mineur) | §5.1 orthogonalité `:166-168` ; §6 storage≠projection `:202-231` ; §7 reordering fallback ; vecteur 14 (`+test_out_of_order_ingest_converges_full`). Trou mineur : liste énumérée §5.1 remote-sync (`:170-179`, étape 4) n'indexe pas la garde prev_hash → P3 dimension 7. |

---

## Invariants cardinaux — tenus de bout en bout

- **0-bump wire strict :** `FEED_FORMAT_VERSION` / `DOMAIN_FEED_V1` / JCS / `FeedEntryCanonical` / `to_canonical` INCHANGÉS ; les 3 champs ajoutés à `PublicRegistryView` (`applied_tips`, `max_applied_key`, `has_unapplied`) sont PRIVÉS, sur une projection mémoire non-sérialisée (`derive Debug/Clone/PartialEq`, pas de serde) — ce n'est PAS un type wire.
- **0 dep, 0 migration :** Cargo.toml/Cargo.lock absents du diff ; db.rs hors diff (0 CREATE/ALTER TABLE, 0 M20) ; seul ajout d'import = std.
- **iroh strictement hors-périmètre :** changements confinés à `nexus-coordinator-rs` + docs ; rien sous `nexus-core-rs` ni `nexus-shell-daemon*`.
- **Pré-launch policy :** 0 test legacy-decode ajouté (tous les nouveaux tests utilisent la version courante) ; `test_verify_chain_out_of_order_insertion` reste load-bearing (order-independence via prev_hash), PAS un zombie.
- **Commit dédié :** `fix(coordinator): Sprint 81 Phase A — …`, JAMAIS dans le commit de bump iroh (Phase B) ; bisectabilité préservée.
- **Langue :** code/identifiants/commentaires/logs EN ; FR uniquement dans le préflight `.planning`.

---

## Findings (P0:0 · P1:0 · P2:1 · P3:10 · réfutés:0)

### P2 — trou de couverture de branche (recommandé AVANT Codex, bon marché, NON bloquant)

- **P2-1 (branch-coverage) — Le garde monotone (conjoint `c`) n'est jamais le facteur décisif testé du fallback incrémental.** `feed_materializer.rs:373-380` — `append_is_sound` est une conjonction : (a) `!view.has_unapplied`, (b) `!new_has_unapplied`, contrôle de longueur, (c) garde monotone `max_applied_key < fold_key(entry)` (:376-380, reconstruction du tuple :378). Le SEUL test atteignant `append_is_sound==false` est `test_out_of_order_ingest_converges_incremental` (:822, appel décisif :851), et il déclenche le fallback via le conjoint **(a)** : `view1` provient de `fold_all([e3])` avec `has_unapplied=true` (orphelin), qui court-circuite AVANT que (c) ne soit pesé. Le scénario paradigmatique de §3.6 — une vue PROPRE (`has_unapplied=false`, chaîne nouvelle complète, longueur OK) recevant une arrivée tardive triant AVANT le frontier (ex. 2e auteur antidaté ingéré après coup) — n'est couvert par aucun test ; c'est pourtant le seul cas où (c) est l'unique décideur. **Adversarial : CONFIRMED (sévérité maintenue P2).** Scénario concret reachable construit et prouvé : auteur B avec 1er timestamp < frontier de A, ingéré incrémentalement sur vue propre → (a)=true, (b)=true, longueur OK, seul (c) est faux, forçant le full-rebuild correct. Une inversion de sens `<`→`>` ligne 378 (ou tuple mal reconstruit) passerait TOUS les tests actuels (`test_cursor_persist_resume` :606 ne valide que append-sound=true en-ordre). Impact live nul (0 consommateur, fallback full-rebuild toujours correct — optimisation abandonnable §3.6) → P2, pas P1. **Correctif suggéré (bon marché) :** un test incrémental sur vue propre (has_unapplied=false) avec arrivée tardive cross-auteur triant avant le frontier, asserté `view == materialize_full`.

### P3 — latents / informationnels (0 impact live confirmé, materializer 0-consommateur prod)

- **P3-1 (diff-correctness) — Walk de `ordered_for_fold_from` sans visited-set : boucle infinie théorique sur cycle de hash.** `feed_materializer.rs:218-233` suit `by_prev.get(current)` sans retirer l'entrée ni tracer les visités ; seuls fork (`len!=1`) ou None arrêtent. Un jeu d'entrées dont les `entry_hash` formeraient un cycle boucle indéfiniment. **Reachabilité NULLE** : l'ingest `feed_sync.rs:269` vérifie `entry_hash=blake3(canonical)` avant insert → un cycle de hash est cryptographiquement infaisable, et `verify_chain` (public_feed.rs, `by_prev.remove`) délègue déjà l'input non-vérifié. Durcissement mineur.
- **P3-2 (diff-correctness) — Branche hash-mismatch de `materialize_incremental` utilise `verify_chain` all-or-nothing.** `feed_materializer.rs:401-404` fait `replay_all` + `verify_chain` avant fold ; c'est tout-ou-rien, alors que les autres replis (reorder-rebuild :394, cursor-None :408) tolèrent le fork (vue partielle). **Branche PRÉ-EXISTANTE (identique dans HEAD, pas introduite par la phase)**, chemin de récupération corruption seulement, 0 consommateur prod. Latent.
- **P3-3 (branch-coverage) — Branche duplicate (`seen.insert==false`) non testée et inatteignable via la DB.** `feed_materializer.rs:197-202` (`duplicates=true` → `has_unapplied`) : `CREATE UNIQUE INDEX idx_feed_entry_hash` (db.rs) + `INSERT INTO` simple → `replay_all` ne rend jamais de doublon. Purement défensive.
- **P3-4 (branch-coverage) — Early-return `new_entries.is_empty()` (2 arms) neuf, non exercé.** `feed_materializer.rs:343-348` — aucun test ne re-matérialise avec curseur à jour et 0 entrée nouvelle. Équivalent à l'ancien comportement (boucle vide), pas de régression démontrée.
- **P3-5 (research-grounding) — Safe-append incrémental dégrade en full-rebuild pour les bursts même-auteur/même-timestamp.** `feed_materializer.rs:56-62` + `:376-380` : le 3e composant de la clé est `entry_hash` (blake3 ≈ aléatoire) ; deux publish dans la même seconde ⇒ classement pile-ou-face ⇒ `append_is_sound=false` fréquent. CORRECTNESS PRÉSERVÉE (rebuild exact), coût de perf latent. CONFORME §3.6 (optimisation abandonnable) ; pour l'ingest REMOTE le rebuild est de toute façon dominant (`has_unapplied`).
- **P3-6 (research-grounding) — Garde `max_applied_key` du chemin incrémental non exercée par un test dédié où `has_unapplied=false`.** Recoupe P2-1 sous l'angle research-grounding : couverte indirectement via le chemin full (`test_intra_author_chain_order_beats_backdated_timestamp` :910). Raffinement de couverture, non-bloquant.
- **P3-7 (research-grounding) — `PublicRegistryView` dérive `PartialEq` sur des champs de bookkeeping (`applied_tips`, `max_applied_key`, `has_unapplied`).** `feed_materializer.rs:66-83` : pour les tests de convergence c'est plus STRICT et correct (métadonnées = fonctions pures du set appliqué). Note informationnelle : un futur consommateur comparant deux vues obtiendrait une égalité dépendant d'un état interne non user-visible. Garantie de convergence de la spec tenue et renforcée.
- **P3-8 (sécurité DEEP, DOWNGRADED depuis P2) — Tie-break par timestamp : hijack de statut projet cross-auteur durable (+30j) — `T-FEED-CLOCK-SKEW` non amendé.** `feed_materializer.rs:48-56/258` (FoldKey timestamp-first + merge `<`) + `:114-162` (`apply` sans binding author→project_id) : un auteur B post-datant jusqu'à `now+30j` (accepté à l'ingest `feed_sync.rs:278`, plafond `FEED_MAX_FUTURE_SECS`) gagne le tie-break de façon CONVERGENTE, et le propriétaire légitime republiant à ~now ne peut plus reprendre la main → override de statut stable network-wide. **Le vrai correctif (autorisation author==owner) est CORRECTEMENT carry** (préflight §6/§3.4, threat model §10). **Adversarial : DOWNGRADED P2→P3.** La prémisse technique est réelle et vérifiée, mais le DÉFAUT reporté est documentation-only : (a) `THREAT_MODEL.md:560` porte encore le résiduel « L (past timestamps accepted, ordering by seq) » — « ordering by seq » est désormais FAUX (le fold n'ordonne plus par seq) → doc-staleness réelle à amender ; (b) `PUBLIC_FEED_SPEC.md:229-231` « a backdated entry cannot veto a causally later update » n'est PAS faux (tient intra-auteur, et cross-auteur une entrée *antidatée* PERD le tie-break — l'attaque réelle est le *post-datage*) → nitpick de sur-lisibilité, pas une erreur de spec. Item doc-only, impact live nul, correctif réel déjà carry ⇒ P3. **À porter au commit body + amender `T-FEED-CLOCK-SKEW`.**
- **P3-9 (sécurité DEEP, DOWNGRADED depuis P2) — k-way merge O(N·K) attaquant-contrôlé + un self-fork gèle le fast-path incrémental.** `feed_materializer.rs:242-273` (scan linéaire des K têtes par entrée émise, K = auteurs distincts Sybil-contrôlés) ; `:228` (self-fork → `has_unapplied=true`) + `:373` (`append_is_sound` exige `!has_unapplied` → full-rebuild perpétuel). Mécanismes réels et code-vérifiés. **Adversarial : DOWNGRADED P2→P3.** (1) 0 consommateur prod, latent-par-construction ; (2) `has_unapplied→full-rebuild` est correct-par-design (§3.6 — un feed forké ne PEUT pas être incrémentalement appended safely) ; (3) la borne DoS dominante (`replay_all` sans LIMIT, `public_feed.rs:561`) pré-existe ET est déjà tracée carry P2 `MAX_FEED_ENTRIES` (préflight §5.7) — capper N borne le merge à O(cap²), donc le O(N·K) est SUBSUMÉ ; (4) la « déviation §5.7 au moins linéaire » est une mésinterprétation (§5.7 demande une linéarisation topo déterministe, livrée par le k-way merge, pas une borne temps O(N)) ; (5) Sybil K≈N requiert de payer le PoW par entrée. Résidu actionnable = micro-opt `BinaryHeap` O(N log K) — nice-to-have P3.
- **P3-10 (livrables/docs-contract) — La liste §5.1 remote-sync n'indexe pas la nouvelle garde de FORMAT prev_hash.** `public_feed.rs:605-616` ajoute un rejet de `prev_hash` malformé dans `verify_entry` (appelé sur les entrées du wire AVANT insert) ; la clôture docs-contrat met à jour §6/§5.1(orthogonalité)/§7/vecteur 14 mais la LISTE énumérée « Remote sync — For each received entry » (`PUBLIC_FEED_SPEC.md:170-179`, étape 4 « Field format validation ») n'inclut pas `prev_hash`. Un implémenteur tiers construisant un vérificateur compatible ne saurait pas qu'un prev_hash malformé doit être rejeté. Gap de complétude mineur (le préflight §3.5 cadre ce guard comme « durcissement mineur, ne pas sur-promettre »). **Correctif suggéré :** une ligne dans la liste §5.1 (ou étendre l'étape 4).

### Findings réfutés / neutralisés

**Aucun réfuté.** Les 7 dimensions ont confirmé leurs findings ; les 2 seuls ajustements de sévérité sont les DOWNGRADED P2→P3 des findings sécurité-deep P3-8/P3-9 (traces adversariales intégrées ci-dessus). Le P2-1 branch-coverage a été CONFIRMÉ à son palier après construction d'un scénario reachable.

---

## Scope cuts vérifiés (préflight §6)

**DANS le scope (livré) :** ordonnancement k-way merge per-auteur ; garde monotone 2 arms ; préfixe per-auteur sans Err globale + rejet dur des forks ; `verify_entry` FORMAT prev_hash ; détection réordonnancement incrémental ; MAJ PUBLIC_FEED_SPEC §6/§5.1/§7/vecteur 14 ; +7 tests ; 0-bump/0 dep/0 migration/commit dédié. **Tous présents et vérifiés.**

**HORS scope (correctement ABSENTS du diff) :** autorisation auteur→project_id (0 binding dans `apply` :114-162) ; borne `MAX_FEED_ENTRIES` (aucune LIMIT ajoutée) ; fail-mode warn-only drop (`feed_sync.rs` non touché) ; bump iroh + pins =1.0.1 (Phase B) ; câblage prod du materializer (0 consommateur). **Aucun scope-creep : exactement 3 fichiers tracés + 1 untracked préflight.**

---

## Note flaky web (env, non-liée à la phase)

Le code web/ est **INCHANGÉ** depuis l'audit gate vert du matin (0 `.rs`/`.tsx` de la phase ne touche web/). Les 6 flaky observés = timeouts jsdom `GpuConsentDialog` sous charge parallèle (classe de variance de charge documentée `vitest_env_variance`), re-verts 17/17 ×2 en solo + 411/411 à machine calme. **Condition mécanique de commit (non-bloquante pour la review) :** arbre propre-vert web/ obligatoire au wrap-up.

---

## Carries tracés (à porter au commit body / phases ultérieures)

1. **Autorisation auteur→project_id (binding author==owner)** — vrai correctif du hijack de statut cross-auteur (P3-8) ; hors wf4 ; carry **threat model §10**.
2. **Borne `MAX_FEED_ENTRIES` sur le fold** (précédent `MAX_ARCHIVE_ENTRIES=4096` S75) — carry **P2** (préflight §5.7) ; borne la DoS `replay_all`/k-way merge (subsume P3-9).
3. **Fail-mode warn-only drop à l'ingest** — anti-pattern visé par **Phase A2** (self-heal ×2 `runtime.rs:2518`/`:2606`, bug DISTINCT) ; Phase A touche `verify_entry` mais NE change PAS ce fail-mode.
4. **Amender `T-FEED-CLOCK-SKEW`** (`THREAT_MODEL.md:560`) — résiduel « ordering by seq » périmé par le fold wf4 (P3-8) ; à corriger au commit ou en carry doc-honnêteté explicite.
5. **Indexer la garde de FORMAT prev_hash dans la liste §5.1 remote-sync** (P3-10) — 1 ligne, bon marché, candidat à balayer dans CE commit.
6. **Micro-opt `BinaryHeap` du k-way merge** (O(N log K), P3-9) — nice-to-have post-Phase A.

---

## Actions avant commit

1. **Recommandé (bon marché) AVANT Codex :** corriger **P2-1** — ajouter un test incrémental sur vue propre (`has_unapplied=false`) où la garde `max_applied_key` (conjoint `c`) est l'UNIQUE décideur du full-rebuild (arrivée tardive cross-auteur triant avant le frontier), asserté `== materialize_full`. Non bloquant pour le verdict.
   **→ FAIT in-phase (post-review, pré-Codex)** : `test_incremental_key_reorder_on_clean_view_triggers_full_rebuild` (`feed_materializer.rs`, auteur B frais depuis genesis à ts=50 < frontier ts=100 : (a) vraie, (b) vraie, longueur OK, seul (c) décide → full rebuild ; assert `== materialize_full` + le gagnant reste ea). Crate 340/340 vert, fmt clean. **Delta de phase corrigé : +8 tests** (7 feed_materializer + 1 public_feed) — 2014→2022 Win / 2018→2026 Docker (re-runs workspace confirmés post-correction).
2. **Candidats à balayer dans CE commit :** P3-10 (1 ligne §5.1) + amendement `T-FEED-CLOCK-SKEW` (P3-8) — cohérence docs-contrat/threat-model.
   **→ FAITS in-phase** : `PUBLIC_FEED_SPEC.md` §5.1 étape 4 indexe `prev_hash "genesis"/lowercase hex-64` ; `THREAT_MODEL.md` T-FEED-CLOCK-SKEW Residual réécrit (« ordering by seq » périmé supprimé ; résiduel post-datage +30j inter-auteurs explicite, correctif binding author→project_id carry §10).
3. **Condition mécanique :** re-verdir web/ (flake de charge `GpuConsentDialog`, non-lié) — arbre propre-vert obligatoire. **→ FAIT** : 411/411 + coverage verts à machine calme.
4. **BLOQUANT — Gate Codex** (`codex exec`, gpt-5.5, raw dans `sprint81_phase_a_codex_review.md`) : boucler jusqu'à CLEAN ou P2/P3 documentés, puis promouvoir review→PASS.
5. **Discipline commit :** 1 commit `fix(coordinator): Sprint 81 Phase A — <titre> (wf4)`, body riche, delta tests cumulé (**+8 Rust → 2022 Win / 2026 Docker**), scope cuts respectés, **JAMAIS** dans le commit de bump iroh (Phase B). Ligne frontière docs-contrat au body (ci-dessous).
6. **Vérifications lourdes dual-platform Docker AVANT push.**

**Ligne d'étiquette frontière docs-contrat (commit body) :** `Frontière docs-contrat : docs/protocol/PUBLIC_FEED_SPEC.md §6 (Ordering — storage seq ≠ projection fold order), §5.1 (vérification ⊥ ordering), §7 (reordering fallback), vecteur 14 — frontière protocole lue par un nœud tiers ; 0 frontière neuve (materialize_* déjà pub, verify_entry signature inchangée, champs PublicRegistryView privés).`

---

## Verdict: PASS

**Justification (§4.5.7) :**
- **PAS de FAIL :** aucun P0/P1 confirmé. Les 6 obligations P1 du préflight sont toutes FIDÈLES au code réel ; le cœur wf4 est correct et convergent (soundness safe-append démontrée + re-vérifiée adversarialement) ; 0-bump / 0 dep / 0 migration authentiques ; iroh hors-périmètre.
- **PAS de CONCERN :** un SEUL P2 review (trou de couverture de branche, corrigé in-phase — Actions §1), pas de P2 structurels multiples. Les 2 P2 sécurité-deep ont été DOWNGRADED à P3 (doc-honnêteté + borne DoS subsumée par carry existant).
- **PASS :** promu après réconciliation Codex (ci-dessous) — review Workflow OK + gate croisée GPT 5.5 jouée, GAPs traités.

---

## Codex reconciliation

Rapport brut : `.planning/active/sprint81_phase_a_codex_review.md` (output `codex exec -o`, non réécrit). Verdicts par livrable : 1 OK, 2 OK, 3 PARTIEL, 4 OK, 5 OK. Checks adversariaux Codex : aucun scénario de non-convergence résiduel, aucun soundness-hole incrémental, aucune casse cursor/verified/up_to, scope invariants OK (`git diff --name-only` = 4 fichiers attendus + review/preflight), tests relancés localement par Codex (2+1 passed).

- **GAP P2 (unique) — `PUBLIC_FEED_SPEC.md` §5.1 étape 3 sur-promettait un « per-author prev_hash chain linkage » par entrée reçue**, contradictoire avec l'ingest hors-ordre voulu (`verify_entry` = format-only AVANT insert, `feed_sync.rs:269`). **CORRIGÉ** : l'étape 3 précise désormais que le linkage se vérifie sur l'ensemble DISPONIBLE (replay `verify_chain()` / fold materializer), jamais en rejet pré-insert d'un prédécesseur manquant ; l'effet sur la projection est différé au comblement du trou (§6). Correction doc-only (0 code, 0 test impacté) — suites inchangées : Win 2022/2022 0-skip + Docker 2026/2026 (re-runs post-P2-1), web 411/411, operator 201/201.
- **P3 (process)** — le review.md portait encore PASS-PENDING au moment du scan Codex : c'est la séquence normale (PASS-PENDING = pré-Codex, promotion PASS = ici même, post-réconciliation). Aucun fix requis.

Boucle Codex arrêtée au critère « CLEAN ou P2/P3 documentés » : P2 corrigé, P3 process sans objet, 0 P0/P1 sur 2 rounds de gate (review Workflow adversarial + Codex).
