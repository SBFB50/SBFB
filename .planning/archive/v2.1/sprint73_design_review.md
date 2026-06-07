# Sprint 73 — Design Review Board (G1)

**Date** : 2026-06-03 (post-audit gate S72 PASS `087e781`).
**Sprint** : 73 — Recherche reseau cablee (FTS5 fraicheur + SearchResult
enrichi + barre shell + securite/dette).
**Reviewer** : recherche multi-agent independante (workflow `wq01d17lj`,
12 agents, ~1.24M tokens — 8 cartographies code + 3 recherches G9 + 1 scan
historique G8), synthese main thread. Agent `nexus-sprint-kickoff` non
enregistre → fallback main thread + Workflow (cf. memory process note).

---

## Scoring

| D# | Titre | Source recente | Alternative comparee | [DETER] Crypto | [DETER] Rust | Code verifie | Verdict |
|---|---|---|---|---|---|---|---|
| D1 | Reindex FTS5 a chaud = upsert incremental par `feed seq` (`INSERT OR REPLACE`) au point feed_sync, pas rebuild | ok (sqlite.org FTS5 doc 3.50.x ; rusqlite 0.36 #1226 2024) | ok (triggers external-content / contentless-delete / rebuild-per-ingest compares) | N/A | ok (rusqlite Rust-native, SQLite bundled 3.50.x) | ok (`search.rs:34-128`, `feed_sync.rs:113-281`, `db.rs:211-222` lus) | ✅ |
| D2 | Enrichir SearchResult avec triplet provenance en colonnes **UNINDEXED** + migration FTS5 DROP/recreate (M17) | ok (FTS5 UNINDEXED pattern existant ; BrowseEntry preuve) | ok (INDEXED vs UNINDEXED vs JOIN-au-query compares) | N/A | ok (Rust serde Option<String>, additive) | ok (`search.rs:7-16`, `browse.rs:170-225`, `public_feed.rs:32-40` lus) | ⚠️ (migration FTS5 non-ALTER = DROP/recreate) |
| D3 | **Defer SearchManifest** ; S73 = feed-local-replique (+ design note forme correcte noeud-index opt-in) | ok (F-Droid/IPFS/Nostr/Radicle/SSB 2024-2026 ; ARES 2024 Sybil) | ok (broadcast-by-all / noeud-index opt-in / feed-local compares) | N/A (defer ; pas de wire/crypto livre S73) | N/A (decision produit + design) | ok (`public_feed.rs:82-118`, CLAUDE.md:354-366, historical scan lus) | ⚠️ (defere un item nomme par la roadmap — acknowledged, grounded) |
| D4 | Barre recherche shell = champ dedie dans Browse via `searchBrowse()` + React Query (palette Ctrl+K reste navigation) | ok (pattern listBrowse + DaemonResult + Zod existants) | ok (champ Browse / header AppShell / palette unifiee compares) | N/A | N/A (frontend UX — exemption Rust-first) | ok (`Browse.tsx:39-108`, `api/daemon.ts:297-311`, `useBridge.ts:359-371` lus) | ✅ |
| D5 | Guardrail de sortie AVANT persist `result_text` sur les **2 chemins** (HTTP + validator_loop) ; split validate_result pre/post-guardrail | ok (audit S72 P2-RESULT-TEXT-GUARDRAIL-ORDER, evidence file:line) | ok (rollback-apres-persist / guardrail-HTTP-seul / doc-only compares) | N/A | ok (refacto Rust validator) | ok (`validator.rs:25-89,155`, `http.rs:1485-1540`, `validator_loop.rs:62-80` lus) | ✅ |
| D6 | worker-pump (P2-A-1 3/3) = fix root-cause cross-platform `#[tokio::test(flavor="multi_thread")]`, pas exemption | ok (tokio #2499/#7049 ; iroh blog actor-thread ; example multi_thread in-repo) | ok (multi_thread / timeout-interne / exemption CI-Linux compares) | N/A | ok (tokio Rust-native, fix matche prod) | ok (`dispatch_loop.rs:146-261`, `runtime.rs:864`, `examples/two_nodes_docs_sync.rs` lus) | ✅ |

**Resume** : D1 ✅, D2 ⚠️, D3 ⚠️, D4 ✅, D5 ✅, D6 ✅.
Rigor signal G4 satisfait (**2 ⚠️ sur 6** — dans la cible gold 1-2/5 mise a
l'echelle).

---

## Findings

### D2 ⚠️ — migration FTS5 par DROP/recreate (FTS5 ne supporte pas ALTER TABLE ADD COLUMN)

**Detail** : enrichir `search_index` (table virtuelle FTS5, M15
`db.rs:213-221`) avec `repo_url`, `commit_sha`, `archive_hash`,
`provenance_hash` impose une migration. FTS5 **ne supporte pas**
`ALTER TABLE ADD COLUMN` sur une table virtuelle — la migration M17 doit
**DROP la table + la recreer** avec le nouveau jeu de colonnes, puis
repopuler. Risque : perte de donnees si la migration echoue a mi-course.

**Mitigation (kickoff §4 D2 + plan §Phase D + R3)** :
1. La migration est **append-only safe par construction** : l'index est
   integralement reconstructible depuis le `public_feed` (source de
   verite). DROP+recreate puis `rebuild_from_feed()` au prochain boot (deja
   appele runtime.rs:778) repopule tout — aucune donnee unique ne vit dans
   l'index. Pre-launch : aucun index externe a preserver.
2. Les 4 nouvelles colonnes sont **UNINDEXED** (retournees, pas
   full-text-matchables) — coherent avec `project_id`/`op_type`/`source_type`
   deja UNINDEXED. Un hash/commit n'est pas un token de langage naturel :
   l'INDEXER gonflerait l'index 20-30% (research finding) pour un MATCH
   sans valeur. Le triplet sert au **fork** (S74), pas a la recherche
   full-text.
3. Tester la migration sur DB replica avant (pattern M1-M16 eprouve).

**Decision** : **acknowledge + adjust** — le ⚠️ reste (la recreation de
table virtuelle est plus risquee qu'un ADD COLUMN), mitige par la
reconstructibilite integrale depuis le feed + colonnes UNINDEXED.

### D3 ⚠️ — defere un livrable nomme par la roadmap (« decider SearchManifest »)

**Detail** : la roadmap v5 §3 liste pour S73 « decision SearchManifest vs
feed-local-replique **selon audit S72** ». La decision **est** prise — c'est
« **defer** » — mais defer un item que la roadmap nomme merite un
acknowledgement explicite (ce n'est pas un oubli).

**Grounding factuel (research G9, 7 modeles etudies)** :
- Les deux vrais gaps S73 (fraicheur + enrichissement pour forker) se
  resolvent **entierement** cote feed-local (le feed est deja gossip-
  replique en DB locale `feed_sync.rs`, la recherche est deja une FTS5 bm25
  locale `search.rs`). SearchManifest n'apporte **rien** en couverture en
  pilote ferme (tout le pilote partage le meme feed gossip).
- La forme « broadcast par tous » de SearchManifest est precisement celle
  que les systemes matures **abandonnent** : Nostr delegue aux relays
  (NIP-50), IPFS aux noeuds de delegated routing, F-Droid a un index signe
  par depot. ARES 2024 : broadcast DHT ouvert = censure/DoS mono-machine.
- La politique pre-launch (`CLAUDE.md:354-366`) rend le feed raw-op
  extensible : ajouter `SearchManifestPublished` plus tard ne bump PAS
  `FEED_FORMAT_VERSION` et ne casse aucun noeud. **Defer ne ferme aucune
  porte** ; PO-13 (« cabler les deux a terme ») reste honore.

**Mitigation** : capturer la **conception correcte** du futur SearchManifest
(noeud-index opt-in signe Ed25519, modele relay/seed-node, anti-spam
signature+kudos, critere de declenchement = federation partielle post-launch)
dans `.planning/research/s73_searchmanifest_index_node_design.md` (Phase D).
Le design dur est fait et pret a coder, sans code protocole speculatif.

**Decision** : **acknowledge + adjust** — defer documente + design note de la
forme correcte. Le ⚠️ trace que la roadmap nommait l'item ; il n'est pas
oublie, il est tranche (defer) avec grounding et reserve sous sa forme
correcte.

**Note historique (DESIGN-CONFLICT evite)** : un doc de recherche fossile
(`s70_s72_rrv_research.md:985-987`) affirme qu'ajouter `SearchManifestPublished`
au feed = breaking change → bump v2. C'est **faux** sous la politique
pre-launch actuelle (`CLAUDE.md:355-357` : raw-op, pas de bump). Comme S73
**defere** SearchManifest, ce conflit ne mord pas — mais il est note pour le
sprint qui implementera (verifier `PUBLIC_FEED_SPEC.md §9` au preflight wire).

---

## Checklist [DETER] (applicable)

### Crypto/spec
- N/A pour le **code livre S73** : feed-local-replique est de l'indexation
  locale (FTS5) + enrichissement DTO + UX. Aucune primitive crypto ni wire
  format reseau touche (D3 defere le wire SearchManifest). Le guardrail D5
  reordonne un filtre existant, ne touche pas la signature Ed25519+JCS des
  results (deja en place S71). Le design note SearchManifest (D3 mitigation)
  **specifie** la crypto correcte du futur wire (Ed25519+gossip+domain
  constant) mais ne la code pas — donc pas de [DETER] crypto a satisfaire
  cette session, la spec est capturee pour le sprint d'implementation.

### Rust-first
- [x] D1 retenu **Rust-native** (`rusqlite 0.36` + SQLite bundled 3.50.x,
  `INSERT OR REPLACE` upsert). Alternatives comparees : triggers
  external-content (rejete : table standalone, pas de content table 1:1),
  contentless-delete (rejete : casse le read path qui SELECT les colonnes
  d'affichage), rebuild-per-ingest (rejete : O(n)/entry, amplification DoS).
- [x] D5 retenu **Rust-native** (refacto `validator.rs` split pre/post-
  guardrail, `default_output_chain` existant repositionne).
- [x] D6 retenu **Rust-native** (tokio test flavor, fix matche prod
  `nexus-worker/src/main.rs:41` multi_thread + example in-repo).
- D2 : schema/DTO (additif serde) — N/A Rust-first (pas de choix runtime).
- D3 : decision produit + design doc — N/A Rust-first.
- Exemptions : D4 (frontend UX shell `web/` — exemption §6.1.1).

---

## Conclusion

6 D-decisions, **toutes ancrees dans le code reel** (file:line verifie par
8 cartographies) **et la recherche factuelle** (G9, sources datees 2024-2026).
2 ⚠️ honnetes (D2 migration FTS5, D3 defer documente), 0 ❌. Aucune decision
ne rebat une Day-0 gelee ; FTS5 reste l'engine (Tantivy gate post-S75 — piege
fossile ecarte). Le sprint est du **cablage + durcissement** sur une infra a
~90-97%, pas de la fondation. G1 PASS.
