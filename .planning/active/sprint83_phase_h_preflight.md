# Sprint 83 Phase H — Preflight G8

**Sprint** : 83 — chantier « Workflow Agents » (43 phases A..AJ1).
**Phase** : H — Compilateur Rust autoritaire (`CompiledPlan.v1`), plan canonique
`.planning/active/sprint83_plan.md:680-728` (H.1 scope :682-693, H.2 livrables :695-702,
H.3 tests :704-713, H.4 acceptation :715-721, H.5 commit :723-728).
**Date** : 2026-07-21. **HEAD** : `24ff47e2` (Phase G committée), arbre propre.
**Méthode** : mode ultracode Workflow — **5 agents de scan** (S1a prior-art OSS profond,
S1b dépendances/CVE/graphe transitif, S2 décisions historiques traversées, S3 couverture
THREAT_MODEL, S4 invariants wire/frontières), **3 passes adversariales** (exactitude-code,
cohérence-décisions, sécurité-invariant) de réfutation + `findings_neufs`, puis cette synthèse.
Le synthétiseur a **rejoué à HEAD** les mesures porteuses : stub `CompiledPlan {}` vide
(`contracts.rs:82-85`) + registre `len == 7` (`:139-147`), tag réservé
`DOMAIN_COMPILED_PLAN_V1` « confers NO acceptance » (`canonical.rs:384-395`), census DOMAIN gelé
== **29** (`check-frontier-contracts.sh:218-221`), `PackDigestKind::{CompiledPlan,GraphSemantic}`
(`pack.rs:203-219`), `edges: Vec<WorkflowEdge>` vs `nodes: BTreeMap` (`graph.rs:640-642`),
`FROZEN_NODE_KINDS: [&str; 18]` + SET-check `node_profile_frozen` (`graph.rs:87,1130-1138`),
19 sites annotés `Phase H` dans `graph.rs`, `BudgetPolicy` 4 dims « REQUIRED and separate…
DIMENSION BY DIMENSION » (`agent_definition.rs:168-187`), golden ideas `join.mode="any"` +
`publish-decision.kind="human.review"` (`ideas.workflow.golden.json:44,91-92`), le hazard
`preserve_order` documenté in-tree (`sbfb-workflow-engine/Cargo.toml:15`), THREAT_MODEL dont la
DERNIÈRE section est **§18** (`docs/security/THREAT_MODEL.md:1699`, aucune §19 / aucun asset
compilateur), et 6 deps workspace + `[dev-dependencies]` vide (`Cargo.toml:11-31`).
**Verdict** : **PLAN-ADAPT** (aucun DESIGN-CONFLICT). Signaux de scan : S1a=PLAN-ADAPT,
S1b=EXECUTE, S2=PLAN-ADAPT, S3=PLAN-ADAPT, S4=PLAN-ADAPT.

---

## 1. Rationale du verdict

**Aucune décision Day-0 (D3/D4 GELÉES) ni aucun invariant du plan n'est en tension structurelle
avec Phase H.** Le design du compilateur (autorité sémantique UNIQUE hors-LLM, validation statique
§10.5, fermeture des capacités AU COMPILE, `CompiledPlan.v1` immuable, digests sémantiques,
invariant Plan-Then-Execute testable) est **état-de-l'art et convergent** avec le prior art réel
2026 (LangGraph `compile()`, Microsoft Agent Framework 1.0, Agentproof arXiv:2603.20356, CaMeL
arXiv:2503.18813, Temporal/Nix déterminisme, Argo/GHA validation DAG). Le point le plus load-bearing :
**LangGraph `compile()` valide topologie/reachability mais NE FERME AUCUN manifeste de capacités**
(les outils sont liés dans les closures de node, swappables au runtime) — la fermeture-au-compile
SBFB est un ajout SOTA genuine, pas une réinvention. `kickoff:486-489` a REJETÉ le graphe-as-code
LangGraph (non signable/inspectable) : rien de rejeté n'est ré-ouvert.

Le verdict n'est **pas DESIGN-CONFLICT** : aucune alternative rejetée au kickoff (graphe-as-code,
gate substring `SENSITIVE_ACTIONS`, `recursion_limit` global, A2A/MCP comme wire) n'est ré-ouverte,
et aucun arbitrage PO n'est requis. `git log --grep=DEVIATION/rejected` sur le crate est **VIDE**.

Le verdict n'est **pas EXECUTE** parce que le texte du plan H (`:680-728`) **omet des obligations
matérielles CONSIGNÉES ailleurs dans le dossier contraignant du sprint** : (a) les 6 règles
statiques du freeze §8 (`sprint83_node_profile_freeze.md:127-156`), déclarées par le freeze lui-même
« ABSENTES de la liste §10.5 », alors que H.1/H.3 ne citent que « validation statique §10.5 » ;
(b) la liste §10.5 réelle = **13 refus** (`research §10.5:898-912`), pas les 6 identifiants de test
nommés en H.3 — soit **19 classes de refus** à couvrir, un test discriminant chacune ; (c) la
compatibilité de ports (nominale par digest) que le freeze §7 assigne à H mais qu'aucune liste
n'énumère ; (d) l'élargissement du test Plan-Then-Execute aux **9 classes** interdites de §19.3
(H.3 n'en nomme que 3) + un bras rouge discriminant anti-vacuité ; (e) la posture domaine/signature
du CompiledPlan (PROSE, 0 nouveau DOMAIN, census 29) ; (f) l'acquittement du carry P2 THREAT_MODEL.
Rien n'est retiré du périmètre — il est **précisé** — donc pas SCOPE-CUT-CONSISTENT.

**Un seul risque de sévérité haute** : la vacuité du test `plan_then_execute_invariant` et de
`four_goldens_compile` (le crate est PUR, 0 LLM réel ; un test tautologique passe pour la mauvaise
raison — learning M3 Phase G : `four_goldens_validate` a nécessité des contrôles négatifs pour être
non-vacueux, `graph.rs:964`).

---

## 2. Base de départ mesurée (rejouée à HEAD `24ff47e2`)

| Fait | Mesure | Source |
|---|---|---|
| Stub cible `CompiledPlan.v1` | `pub struct CompiledPlan {}`, `deny_unknown_fields`, `title` | `contracts.rs:82-85` |
| Registre contrats | `len == 7`, `CompiledPlan.v1` = index 5 (6/7) | `contracts.rs:101-147` |
| Drift-gate schema | `schema_snapshot_stub` **STRICT** (manquant/vide = FAILURE) | `contracts.rs:180-220` |
| Tag domaine `DOMAIN_COMPILED_PLAN_V1` | RÉSERVÉ Phase F, « confers NO acceptance », 0 producteur | `canonical.rs:384-395` |
| Census DOMAIN | **29**, gate clean, `COMPILED_PLAN` dans la NAME LIST gelée | `check-frontier-contracts.sh:218-221` |
| Infra digest | `PackDigestKind::{GraphSemantic,CompiledPlan}` + `digest_for`→`canonical_bytes`(JCS)→`blake3_hex` | `pack.rs:203-219,1327-1351,477-491` |
| `edges` vs `nodes` | `edges: Vec<WorkflowEdge>` (ordre-significatif), `nodes: BTreeMap` (trié) | `graph.rs:640-642` |
| Roster gelé | `FROZEN_NODE_KINDS: [&str; 18]` + SET-check discriminant | `graph.rs:87,1130-1138` |
| Contrat d'implém. de H | **19 sites** annotés `Phase H` dans `graph.rs` (cycle/budget/edge/capability/handoff/join/reachability) | `graph.rs` grep |
| `BudgetPolicy` | 4 dims REQUISES séparées, comparaison enfant≤parent DIMENSION BY DIMENSION | `agent_definition.rs:168-187` |
| Contrôles négatifs G | `golden_negative_controls_reject` « keep `four_goldens_validate` non-vacuous » | `graph.rs:964-968` |
| `pack.digests.compiled_plan` | validé pour la FORME seule ; test `six_digests_stable` calcule via `_ => document.clone()` (doc BRUT) | `pack.rs:195-201,1519-1530` |
| THREAT_MODEL | v21, DERNIÈRE section = **§18** (aucune §19, aucun asset compilateur/plan-signé) | `THREAT_MODEL.md:1699` |
| Deps crate cible | 6 workspace-deps, `[dev-dependencies]` **vide** | `Cargo.toml:11-31` |
| Piège `preserve_order` | feature active en `--workspace` (ollama-rs→schemars), inactive en `-p` | `sbfb-workflow-engine/Cargo.toml:15` |

---

## 3. Scan S1a — Prior art OSS profond (verdict de scan : PLAN-ADAPT)

Le design est SOTA ; les findings sont des ajouts de FORME/tests tirés du prior art réel 2026.
Tous **CONFIRMED** en adversarial sauf deux **ADJUSTED** (sur-affirmation resserrée).

- **S1a-01 — Fermeture-au-compile = ajout SOTA genuine. [INFO, CONFIRMED]** LangGraph `compile()`
  valide topologie/reachability/entrypoints mais ne ferme AUCUN manifeste de capacités (tools dans
  les closures, swappables au runtime). SBFB : `graph.rs:653` « every `capability.call` must
  reference one of them (Phase H). Requesting is never obtaining » + `canonical.rs:379-380` « the
  pack format grants no capability: it declares requests that an authority intersects elsewhere ».
  → Verrouiller que le CompiledPlan est l'UNIQUE autorité de capacités ; ne jamais s'inspirer du
  tool-binding dynamique LangGraph.
- **S1a-02 — Canonicalisation d'ordre des arrays pour `graph_semantic_digest`. [MAJOR→scoped, ADJUSTED]**
  `graph_semantic_view` (`pack.rs:1522`) fait `.map().collect()` SANS tri ; `edges` est un `Vec`
  (`graph.rs:642`), JCS trie les MEMBRES d'objet jamais les éléments d'array. **Ajustement
  adversarial** : (1) la « later phase » qui pinne l'identité des nodes (`pack.rs:207-209`) = **G**
  (nodes en `BTreeMap`), PAS H — l'attribution du finding à H est imprécise ; (2) « WA-T1.2
  structurellement non-atteignable » est SUR-AFFIRMÉ : au read-path H, le compilateur lit UN pack
  pinné unique → canvas et outline lisent le MÊME `edges` Vec → digest identique trivialement ; le
  risque d'ordre ne mord que pour deux surfaces d'authoring indépendantes (éditeur canvas/outline =
  **Phase T**). → Requirement RÉEL mais SCOPÉ : H DOIT imposer un ordre canonique des edges (tri
  stable `(from,from_port,to,to_port)`) + tout `Vec` non-identifiant AVANT le
  `graph_semantic_digest` de niveau CompiledPlan, avec un test positif « deux ordres d'edges → même
  digest ». Locus à trancher (voir NEUF-01/NEUF-SEC-02, §10).
- **S1a-03 — Refus « edge port-incompatible » (nominal par digest). [MAJOR, CONFIRMED]** MS Agent
  Framework 1.0 et Agentproof élèvent la compatibilité de type inter-ports au rang de check statique
  de première classe. Le freeze §7 (`:124-125`) assigne « la compatibilité de ports (nominale par
  digest, v1) » à H, mais ni §8 ni §10.5 ne l'énumèrent, et H.3 n'a aucun test port-mismatch.
  `§10.5:905 « output non type »` (node sans schéma) ≠ « edge dont `output_schema_ref` producteur ≠
  `input_schema_ref` attendu ». → Ajouter un refus explicite (nominale v1 = `blake3(output_schema)
  != blake3(input attendu)`) + `static_validation_rejects_port_type_mismatch`. Scope-cut « nominale,
  pas de sous-typage structurel » assumé mais TESTÉ.
- **S1a-04 — Émettre `node_instance_id` per-occurrence (enabler runtime), PAS l'enforcement CaMeL. [MAJOR→scoped, ADJUSTED]**
  CaMeL attache les capacités aux VALEURS et les vérifie par-tool-call. **Ajustement adversarial
  (3 angles)** : `§12.1:969` mandate pour H une « fermeture des capacités requises » comme
  ENSEMBLE/SET + `§12.1:970` « IDs d'occurrence uniques » ; l'enforcement least-authority PAR-EFFET
  (`effect_id`) est explicitement RUNTIME (`§19.3:1612`, Phase J/K) et l'intersection 7 couches =
  Phase L. → H FIXE/ferme le SET (et peut indexer l'appartenance par node comme DONNÉE), émet
  `node_instance_id` pour que J/K/L PUISSENT lier par-occurrence ; H n'implémente PAS le binding
  d'enforcement per-effet. Pour l'invariant COMPILE-TIME « aucun LLM n'accorde une capacité », le
  test au niveau SET du plan immuable SUFFIT.
- **S1a-05 — Compilation = fonction pure déterministe. [INFO, CONFIRMED]** Temporal/Nix imposent
  `compile(pack, version+profil) → bytes identiques`. État favorable : `nodes`/`field_map` en
  `BTreeMap` (`graph.rs:640,181`), 0 `HashMap`. `node_instance_id` DOIT être une fonction pure du
  chemin structurel (`§12.2:994-996`), jamais compteur/UUID/timestamp. → Ajouter le test non-listé
  « compile(pack) deux fois → CompiledPlan byte-identique » (reproductibilité) — cardinal pour des
  `*.compiled.golden.json` stables + le gate dual-platform Win+Docker face au piège `preserve_order`.
- **S1a-06 — Bornes structurelles `repeat`/`map` > `recursion_limit` global. [INFO, CONFIRMED]**
  `recursion_limit` LangGraph = cap runtime global (bugs de boucle infinie documentés), rejeté
  `kickoff:494-496`. `freeze §5:84-96` : v1 sans back-edge, `edges` = liste libre (le parse G accepte
  `A→B,B→A`), rejet DAG = refus H. → Le check d'acyclicité H traite les corps `repeat`/`map`
  (subworkflows pinnés) comme la SEULE itération légale + refuse tout back-edge sur `edges`.
- **S1a-07 — Refus d'edge DUPLIQUÉE (self-loop déjà subsumé par le cycle). [MAJOR→ciblé, ADJUSTED]**
  MS AF valide « duplicate edges ». Le freeze §8 (`:131-144`) liste l'intégrité référentielle et le
  rejet de cycle mais PAS la détection d'edge dupliquée. **Ajustement adversarial** : le **self-loop
  `from==to` est un cycle trivial DÉJÀ subsumé par la règle 3 §8** (« rejet de tout cycle sur edges
  quelconques ») — la sous-claim « §8 liste le cycle mais pas le self-loop » est inexacte. Seule la
  classe **edge-dupliquée `(from,from_port,to,to_port)` identique** est un vrai ajout : elle double
  silencieusement le fan-in et peut rendre insatisfiable `quorum(n)` d'un join. → Ajouter
  `static_validation_rejects_duplicate_edge`, chaîné avec la satisfiabilité du join.

---

## 4. Scan S1b — Dépendances / CVE / graphe transitif (verdict de scan : EXECUTE)

Phase H s'exécute avec **0 dépendance runtime/dev neuve**. Tous **CONFIRMED**.

- **S1b-01 — 0 dep neuve. [INFO]** 6 deps du crate toutes `{ workspace = true }` (`Cargo.toml:11-27`),
  `[dev-dependencies]` VIDE (`:29-31`). Coder `CompiledPlan.v1` + validateur + digests + committer
  les `*.compiled.golden.json` ne touche NI `Cargo.toml` NI `Cargo.lock` : 0-bump/0-dep par
  construction. Le body H annonce `git diff Cargo.toml Cargo.lock` VIDE (comme G).
- **S1b-02 — Erreurs hand-rolled, PAS `thiserror`. [MINOR]** `thiserror` est au workspace
  (`Cargo.toml racine:87`) MAIS le crate hand-roll ses erreurs (`PackError` = enum manuel + `impl
  Display`/`Error`, `pack.rs:149-188`). → `CompileError`/`CompiledPlanError` hand-rolled, une
  variante nommée par classe de refus avec `path`/`detail`. N'introduit ni `thiserror` ni `anyhow`.
- **S1b-03 — BTreeMap + toposort maison, PAS `indexmap`/`petgraph`. [INFO]** Ni `indexmap` ni
  `petgraph` en `[workspace.dependencies]` (transitifs seuls) ; BTreeMap est le patron du crate.
  → CompiledPlan indexe par `BTreeMap` (clés triées, feature-indépendantes) ; le DAG-check (freeze
  §8.3) = DFS/Kahn maison, zéro crate graphe. Ne jamais importer `IndexMap` directement.
- **S1b-04 — Piège `preserve_order` ACTIF. [MAJOR]** `serde_json` feature `indexmap` ← `preserve_order`
  ← `schemars/preserve_order` ← `ollama-rs 0.3.4` (mesuré `cargo tree`). En `--workspace`
  `serde_json::Value` backing = `IndexMap` (ordre d'insertion), en `-p` = `BTreeMap` (trié) →
  l'ordre des membres du fichier `*.compiled.golden.json` diverge entre modes de build. Deux
  disciplines OBLIGATOIRES : (a) TOUT digest via `serde_jcs`/`digest_for`, JAMAIS `serde_json::to_vec`
  (`sbfb-workflow-engine/Cargo.toml:15` documente le hazard) ; (b) comparaison golden/snapshot en
  `serde_json::Value` (jamais string), régénération sous build `--workspace`, tests joués `-p` ET
  `--workspace`.
- **S1b-05 — Infra digest COMPLÈTE et GELÉE (0 règle neuve). [INFO]** `PackDigestKind::CompiledPlan`
  (`pack.rs:214`) et `::GraphSemantic` (`:210`) existent déjà, domain-séparés via `digest_for`
  → `canonical_bytes`(`serde_jcs` + `reject_nulls` + préfixe domaine `\0`) → `blake3_hex`
  (`pack.rs:1327-1351,477-491`). → Le digest CompiledPlan = `digest_for(PackDigestKind::CompiledPlan,
  &value)` ; le graph_semantic = `digest_for(PackDigestKind::GraphSemantic, &value)`. Zéro nouvelle
  primitive crypto : H câble, ne ré-invente pas (freeze §7 « AUCUNE règle de digest neuve »).
- **S1b-07 — RUSTSEC propre pour les deps de H. [INFO]** serde_json 1.0.149 / schemars 1.2.1 /
  blake3 1.8.5 / serde_jcs 0.2.0 / ed25519-dalek 2.2.0 : aucun advisory. Les ignores `deny.toml`
  (quick-xml, ed25519-dalek RC) sont iroh-internes ; le rouge supply-chain CI-1 = LICENSES
  (async_executors/borrow-or-share), orthogonal, ne touche AUCUNE dep de H.
- **S1b-08 — schemars 1.2.1 draft 2020-12 hérité. [INFO]** Le schéma CompiledPlan héritera
  automatiquement du draft ; `every_contract_emits_nonempty_draft_2020_12_object` (`contracts.rs:149`)
  le couvre. Régénérer le snapshot anti-drift SOUS `--workspace` (bytes déterministes). `schemars
  0.9.0` au lock = orphelin bénin — ne pas retirer.

---

## 5. Scan S2 — Décisions historiques traversées (verdict de scan : PLAN-ADAPT)

`git log --grep=DEVIATION/rejected/scope-cut` sur le crate = VIDE. Tous **CONFIRMED** (re-mesurés
verbatim ; un ajustement de comptage mineur signalé).

- **S2-01 — 6 règles freeze §8 CONSIGNÉES POUR H, OMISES du texte du plan. [MAJOR (ex-BLOCKER)]**
  Le freeze §8 (`:127-156`) énumère verbatim 6 règles (l'en-tête dit « Cinq » mais liste SIX, rule 6
  post-Codex r1), déclarées « ABSENTES de la liste §10.5 » — donc « valider §10.5 » seule les OMET.
  **Ajustement de sévérité adversarial (BLOCKER→MAJOR)** : ces règles NE sont PAS absentes du dossier
  contraignant — elles sont CONSIGNÉES dans un fichier pivot obligatoire du préflight sous
  « Consigné pour Phase H ». Le défaut est une **sous-énumération du TEXTE du plan** (PLAN-ADAPT), pas
  une spec manquante qui halterait le sprint. → H implémente les 6 refus EN PLUS des 13 §10.5, un
  test discriminant chacun (§8 + §9 ci-dessous).
- **S2-02 — §10.5 = 13 refus, pas 6. [MAJOR]** `research §10.5:898-912` (13 refus verbatim) vs plan
  H.3:707-709 (6 identifiants) qui affirme pourtant « chaque règle §10.5 ». Non-nommés donc à risque
  d'omission : output non typé, provider incompatible, secret literal, endpoint/code exécutable,
  publication sans provenance/HITL/éval. → 13 + 6 = **19 classes**.
- **S2-04 — FRONTIER = PROSE, DOMAIN déjà réservé, 0 minté. [MAJOR]** `DOMAIN_COMPILED_PLAN_V1` EXISTE
  RÉSERVÉ (Phase F, `canonical.rs:384-395`, « confers NO acceptance »), compté dans le census 29 gelé
  (`check-frontier-contracts.sh:221`), 0 producteur/vérifieur. AM-3 (`kickoff:536`) liste les
  primitives recevant un tag machine `// FRONTIER:` — PackV1/GraphV1/ProfileV1/ProjectionV1/routes/
  IPC/wire/MCP — et **n'y inclut PAS CompiledPlan**. Précédent : `graph.rs:609-619` porte PROSE,
  « no `// FRONTIER:` tag » (pinné par digest, non signé seul). → Voie **(a)** : CompiledPlan reste
  LOCAL/dérivé non-signé → frontière PROSE + refresh du schema snapshot, **AUCUN nouveau `const`
  DOMAIN/version**, AUCUN `// FRONTIER:` machine. Le plan pointe vers (a) (H.1/H.2 ne demandent
  aucune signature).
- **S2-05 / S2-06 — Remplir le stub 6/7 + 9 familles de champs §12.1. [MAJOR]** `contracts.rs:82-85`
  stub `CompiledPlan {}` (deny_unknown_fields + title). Le remplir → garder `len == 7`
  (`every_contract_emits_...`) + rafraîchir `contracts.schema.json` (`schema_snapshot_stub` STRICT,
  `UPDATE_SNAPSHOTS=1`) dans le MÊME commit. Champs REQUIS (`research §12.1:965-975`, verbatim) :
  digest du pack source ; digests exacts workflows/agents/prompts/schemas/outils ; fermeture des
  capacités requises ; IDs d'occurrence uniques ; routes/joins/bornes normalisés ; budgets hérités
  et atténués ; politique de placement ; politique de retry/timeout/cancel ; version et profil du
  compilateur. Plan immuable pendant le run (`:977`). `node_instance_id` encode le chemin complet
  (`§12.2:994-996`).
- **S2-07 — Frontières de scope : H = `sbfb-workflow-core` UNIQUEMENT. [MAJOR]** Le crate
  `sbfb-workflow-engine` EXISTE DÉJÀ (Phase E, `event.rs` porte un `RunEvent` réel) — **H NE LE TOUCHE
  PAS**. Run store = I, reducer/scheduler/états/executors/cancel = J, effets/retries/idempotence = K,
  intersection 7-couches/atténuation/coffre/HITL = L/L1, miroir TS/canvas = T. H = compile(pack)
  → CompiledPlan.v1 + validation statique + **fermeture BI-NIVEAU** des capacités
  (`requested_capabilities` pack ⊇ `capability_requests` workflow [refus « capacité absente du
  manifeste »] ET `capability_ref ∈ capability_requests` [forme testée G `graph.rs:1262`, le refus
  est H]), pas l'atténuation par couches.
- **S2-08 — Invariant Plan-Then-Execute ratifié D4, testable. [MAJOR]** `kickoff:454-459` verbatim :
  « le graphe signé EST le plan ; aucun LLM n'étend le graphe, n'accorde une capacité, ne change une
  destination ni ne produit un verdict process autoritaire… Prompt injection : jamais topologie/
  outils/destinations/grants/classification/HITL/budgets (§19.3) ». Carry P2 « graphe signé = plan »
  routé « H, sinon AH » (`freeze:162-164`). → Test de propriété (voir S3-05 : 9 classes + bras rouge).
- **S2-09 — Budgets : comparaison PAR dimension. [MAJOR]** `BudgetPolicy` = 4 dims REQUISES séparées
  (`agent_definition.rs:168-187`, « DIMENSION BY DIMENSION with no unit conversion »). Refus H :
  budget node ≤ workflow (`graph.rs:648`) ET, pour `agent.invoke`, double borne ≤ définition d'agent
  ET ≤ workflow (`graph.rs:206,214-216`). → Vérifier CHAQUE dimension séparément, jamais un agrégat.
- **S2-10 — 19 sites `Phase H` dans `graph.rs` = le contrat d'implémentation écrit par G. [MAJOR]**
  cycle (`:23,411`), budget (`:206,214-216,648`), intégrité edges + reachability (`:61,637`),
  capability declared-here (`:112,267,653`), handoff ∈ allowed_handoffs (`:247`), satisfiabilité join
  (`:392`), output ∈ artifact_outputs (`:471`), retry target wired (`:550`), entrypoints non-vides
  (`:637`). Chaque annotation = un refus statique + un test.
- **S2-11 — Pureté D3 : H reste 100% `sbfb-workflow-core`. [MAJOR]** Sans Tokio/réseau/DB
  (`kickoff:360-362`). Les identifiants (`node_instance_id`/`effect_id`) sont DÉFINIS/normalisés par
  le plan mais CONSOMMÉS/incrémentés par le moteur (I/J/K).
- **S2-12 — Alternatives rejetées + roster 18 gelé. [INFO]** `kickoff:485-499` (graphe-as-code, gate
  substring, `recursion_limit`, A2A/MCP wire — à NE PAS re-débattre). Scope cut #4 (`:764-765`) : node
  code-execute/plugins → §10.5 refus « endpoint/code exécutable » + « secret literal ». Roster = 18
  kinds gelés, `sequence` = mode de composition (edges), pas un node kind (freeze §2).
- **S2-13 — Anti-drift = snapshot Rust SEUL (miroir TS → T). [MINOR]** Le miroir TS/canvas/enveloppe
  présentation sont décalés Phase T (commit G). H livre l'anti-drift Rust (`contracts.schema.json`) ;
  ne pas sur-livrer la parité TS (voir S4-05).
- **S2-14 — Commit + T1. [INFO]** `feat(workflow-core): Sprint 83 Phase H — compilateur Rust
  autoritaire (CompiledPlan.v1)`, body 9 sections, T1 = `N-A-no-frontend-change`. Golden ideas
  exemplifie déjà `join any` + publication `human.review` (`ideas.workflow.golden.json:44,91-92`).

---

## 6. Scan S3 — Couverture THREAT_MODEL (verdict de scan : PLAN-ADAPT)

Le compilateur ferme un ensemble STATIQUE clos ; 3 resserrements à absorber. Tous **CONFIRMED**.

- **S3-01 — Acquittement THREAT_MODEL à trancher. [MAJOR]** Le carry P2 « graphe signé = plan » route
  « H, sinon AH » (`freeze:162-164`, commit G body), mais H.5 (`:723-728`) ne liste AUCUNE mise à jour
  THREAT_MODEL et la Phase AH s'approprie déjà « THREAT_MODEL vNN au §19 du pack » (`plan:1942`). H
  est le foyer NATUREL (§10.5 devient réel pour la 1re fois). → **Décision** : H SÈME l'asset (une
  §19 dans le doc de sécurité documentant que le compilateur EST le point d'enforcement de
  « le graphe signé EST le plan » — cohérent README §4 docs-contrat : une autorité neuve livrée =
  sa surface de menace documentée), AH fait la confrontation adversariale complète ; sinon carry
  STANDING vers AH avec le risque tracé (invariant enforced en H mais non documenté canoniquement
  H..AG). **Recommandé : H sème, AH confronte.**
- **S3-02 — Ne pas confondre §19.3 Research Pack et THREAT_MODEL. [INFO]** THREAT_MODEL v21, DERNIÈRE
  section = **§18** (`:1699`) ; aucune §19, aucune section compilateur/prompt-injection. Le « §19.3 »
  cité partout est une section du **Research Pack** (`workflow_agents…:1597`), PAS de THREAT_MODEL.
  Toute claim H doit préciser la source ; si H ajoute une section, elle sera §19 (première) du doc de
  sécurité (cadence §18 : §7 mitigations + hash commit).
- **S3-03 — Ensemble statique clos = §10.5 (13) + freeze §8 (6). [INFO]** Un test par CLASSE de refus ;
  les 6 règles §8 doivent être des classes DISTINCTES (§9).
- **S3-04 — Règle 6 (provenance des discriminants de classification). [MAJOR]** `freeze §8:145-156` :
  un `switch`/`classifier` dont dépend un `capability.call` d'egress/publication ne peut discriminer
  sur un champ de CLASSIFICATION (public/privé) produit par une SORTIE de modèle — cette classe vient
  d'une source déterministe ou d'un `human.review`. Portée bornée aux champs de classification (un
  quality-gate modèle = classe différente). Le golden ideas l'exemplifie (`publish-decision` =
  `human.review`, `ideas.workflow.golden.json:91-92`). H.3 ne nomme AUCUN test → gap. → Ajouter
  `static_validation_rejects_classification_from_model`.
- **S3-05 — Élargir Plan-Then-Execute aux 9 classes §19.3 + bras rouge discriminant. [MAJOR]** H.3
  (`:710-711`) ne nomme que 3 des 9 classes interdites (`research §19.3:1599-1610` : topologie, outils,
  destinations, grants, classification, schémas de sortie, HITL, budgets/bornes, identité publisher).
  Risque de vacuité (crate PUR, 0 LLM). → Élargir aux 9 classes (ou justifier la réduction) + bras
  discriminant : (i) sortie-modèle hostile injectée → CompiledPlan/digest INCHANGÉ ; (ii) MÊME contenu
  injecté comme input PACK → plan DIFFÉRENT (sensibilité du compilateur à l'input réel). Sans (ii),
  tautologie (learning M3 Phase G ; mémoire `feedback_mutation_before_trust`). **Réutiliser le patron
  existant `golden_negative_controls_reject` `graph.rs:964`** (NEUF-02).
- **S3-06 — Deux honnêtetés à respecter dans les claims H. [MAJOR]** (a) rejet de cycle par
  COMPILATION, pas « by construction » (`freeze §5:84-96` : `edges` = liste libre, `graph.rs:23`
  « ANY cycle over arbitrary edges is a Phase H compiler refusal ») ; (b) digest FAISANT AUTORITÉ =
  `content` (recalculé de PACK.lock, `pack.rs:196`), PAS `graph_semantic` (vue recomputable). → Rédiger
  « DAG prouvé par compilation » ; lier CompiledPlan au pack root signé, ne jamais ériger
  `graph_semantic_digest` en équivalent-signature.
- **S3-07 — Malléabilité du digest sémantique fermée par la règle 4. [MAJOR]** `graph_semantic_view`
  strippe TOUT membre ∈ `PRESENTATION_KEYS` à toute profondeur → un node-id ou une clé `field_map`
  nommée `layout`/`zoom` parse (deny_unknown_fields ne couvre pas les clés de map, `freeze §6:108-114`)
  puis est STRIPPÉE → deux plans distincts, même digest. Hazard BIDIRECTIONNEL (smuggling attaquant OU
  collision légitime). → Implémenter la règle 4 §8 (data-driven keys ∌ PRESENTATION_KEYS) + faire
  porter à `CompiledPlan.pack_digest` le `content` digest EXACT du pack signé, Ed25519 vérifiée AVANT
  compile.
- **S3-08 — Bornage d'honnêteté : H ferme le STATIQUE, pas le runtime. [INFO]** Revalidation tool-call
  (`§19.3:1612`) = J ; SSRF/DNS (`§19.5`) = P ; binding `secret_ref` + HITL signé = L1 ; intersection
  7 couches = L ; store durable = I. H = FERMETURE statique des capacités + refus secrets LITTÉRAUX,
  PAS le binding runtime. → Section « Scope cuts » du commit H : énoncer ce que le compilateur NE
  ferme PAS (anti sur-affirmation « Plan-Then-Execute prouvé » : H prouve la moitié statique).
- **S3-09 — État de départ : stub vide, invariant « porté par la FORME ». [INFO]** `CompiledPlan {}`
  (`contracts.rs:82-85`) ; invariant NON enforced (commit G body). H = 1er point où il devient
  enforceable/testable ; le drift-gate rougira de `{}` vers la forme pleine.

---

## 7. Scan S4 — Invariants wire format / frontières (verdict de scan : PLAN-ADAPT)

Phase H est **wire-clean par construction** : 0 DOMAIN neuf, 0 bump, census 29 intact, registre
`len==7`, aucun test legacy. Tous **CONFIRMED** sauf S4-05 **ADJUSTED**.

- **S4-01 — 0 DOMAIN neuf, census 29 intact. [INFO]** Les 3 tags réservés
  (`COMPILED_PLAN/RUN_EVENT/PACK_INSTALL_RECEIPT`) posés Phase F ; `DOMAIN_COMPILED_PLAN_V1` déclaré
  UNIQUEMENT `canonical.rs:395` (+ entrée NAME LIST du census), 0 producteur. → Vérifier `bash
  scripts/check-frontier-contracts.sh` exit 0 au commit (critère H.4). Ne PAS ajouter de `DOMAIN_*`
  ni `*_FORMAT_VERSION`.
- **S4-02 — CompiledPlan.v1 = artefact LOCAL hôte non signé en H. [INFO]** `research §12.1:953-978`
  (produit par le compilateur Rust hôte, immuable pendant le run) ; la signature « attestation
  compilateur » (`§17.1`) = rôle SÉPARÉ différé, absent de §H.2. → H produit en mémoire/fixture,
  JAMAIS `canonical_bytes(DOMAIN_COMPILED_PLAN_V1)` — l'activer forcerait une décision census.
- **S4-03 — Piège `preserve_order` (doublon de S1b-04). [MAJOR]** Digest via `serde_jcs` ; régénérer
  `contracts.schema.json` sous profil workspace ; le compare `Value` du snapshot reste vert dans les
  2 profils mais le DIFF git textuel diverge selon le profil de régénération.
- **S4-04 — Remplissage du stub = identité 6/7, mécanique D/F/G. [INFO]** Éditer le stub → champs ;
  `UPDATE_SNAPSHOTS=1` ; `len==7` toujours vert ; conserver `deny_unknown_fields` + `title =
  "CompiledPlan.v1"`. Ne PAS toucher les 6 autres identités.
- **S4-05 — Miroir TS INEXISTANT : voie (b) snapshot Rust SEUL. [MAJOR, ADJUSTED]** Le test H.3
  `compiled_plan_matches_snapshot (Rust/TS)` nomme un miroir TS qui n'existe pas (grep `web/` pour
  `SBFBWorkflowGraph|CompiledPlan|contracts.schema.json` = VIDE ; seul `pack_digest.test.ts` = miroir
  DIGEST du PACK). **Ajustement adversarial** : la « décision requise (a)/(b) » est trompeuse — la
  décision est DÉJÀ prise : (1) commit G scope cut route « miroir TS → T » ; (2) H est
  `T1=N-A-no-frontend-change`. Ajouter un miroir TS en H toucherait `web/` et violerait le scope cut
  ET T1=N-A. → **Voie (b)** : l'anti-drift Rust/TS de H = le seul snapshot Rust `contracts.schema.json` ;
  la formulation « (Rust/TS) » du plan est plus large que ce que G a livré (graph/agent n'ont pas de
  miroir TS) — le miroir TS effectif est Phase T.
- **S4-06 — CompiledPlan.v1 naît v1, 0 test legacy. [MINOR]** Pre-launch (CLAUDE.md §Pre-launch) :
  aucun test legacy/backward-decode, `deny_unknown_fields` fail-closed conservé, pas de
  `#[serde(default)]`-compat. Distinguer au commit : identité CONTRAT remplie (schema dans
  `contracts.schema.json`) vs tag DOMAIN de SIGNATURE toujours réservé (0 producteur) — 1 ligne de
  prose census (`check-frontier-contracts.sh:207-211`) à ajuster pour honnêteté, **NAME LIST inchangée**
  (gate reste vert).

---

## 8. Règles consignées freeze §8 → obligations H (verbatim + comment H les honore)

Le freeze `sprint83_node_profile_freeze.md:127-156` consigne **six** règles (l'en-tête dit « Cinq »
mais en énumère six ; obligation matérielle = **6**), déclarées « ABSENTES de la liste §10.5 » :

1. **« Intégrité référentielle des edges : chaque `edge.from`/`edge.to` nomme un node existant
   (« node inaccessible » ne couvre pas un edge vers un id inexistant). L'unicité des node ids est
   déjà structurelle (map keyed by id). »** → H : refus si `edge.from`/`edge.to` ∉ `nodes` (BTreeMap) ;
   test `static_validation_rejects_dangling_edge`. Unicité node id = acquise par `BTreeMap`
   (`graph.rs:640`). **+ S1a-07 : refus edge-dupliquée** `(from,from_port,to,to_port)` identique.
2. **« Unicité des ports par label : un `switch`/`classifier` dont chaque label (+ `default`) a
   exactement une arête sortante ; un `join` dont les `source_node_ids` existent et alimentent
   réellement le join. »** → H : par node de routage, bijection label→arête (aucun label sans arête,
   aucune arête surnuméraire) ; join satisfiable (`quorum(n) ≤ #sources distinctes après dedup`).
   Chaîne avec S1a-07. Tests `..rejects_switch_label_without_edge`, `..rejects_unsatisfiable_join`.
3. **« Rejet de tout cycle sur edges quelconques (cible « tout plan compilé est un DAG », cf. §5 — le
   parse G ne le garantit pas). »** → H : DAG-check maison (Kahn/DFS, `edges` Vec libre), refus de tout
   back-edge ; l'itération légale = corps `repeat`/`map` (subworkflows pinnés). Test
   `static_validation_rejects_cycle` (`A→B, B→A`). **Claim honnête : DAG PAR COMPILATION, pas by
   construction** (S3-06). Subsume les self-loops (S1a-07 ADJUSTED).
4. **« Clés data-driven disjointes de `pack::PRESENTATION_KEYS` : tout node id ET toute clé de
   `field_map` (et toute clé data-driven future) hors des 10 noms réservés (cf. §6 —
   `deny_unknown_fields` ne couvre pas les clés de map). »** → H : refus si un node id ou une clé
   `field_map` ∈ les 10 `PRESENTATION_KEYS` (ferme la malléabilité du `graph_semantic_digest`, S3-07,
   hazard bidirectionnel). Test `static_validation_rejects_presentation_key_as_data`.
5. **« Labels de routage sains : `"default"` ∉ `branch_labels` (collision avec le port réservé) et
   `branch_labels`/`decision_labels` non vides. »** → H : refus si `"default"` apparaît dans
   `branch_labels`, ou si `branch_labels`/`decision_labels` sont vides. Test `..rejects_default_in_branch_labels`.
6. **« Provenance des discriminants de classification (§19.3, Codex r1 GAP1) : un `switch`/`classifier`
   dont dépend un `capability.call` d'egress ou de publication ne peut PAS discriminer sur un champ de
   CLASSIFICATION (public/privé, niveau de confidentialité) produit par une sortie de modèle — cette
   classe de champ vient d'une source déterministe ou d'un `human.review`, jamais d'un LLM. »** → H :
   **analyse data-flow** sur les ports typés EXISTANTS (producteur `output_schema` du discriminant → le
   `capability.call` d'egress/publication), refus si le discriminant de classification remonte à une
   sortie de modèle. **NEUF-COH-01 : par ANALYSE, jamais par un nouveau champ de schéma** (ajouter une
   annotation « classification » rouvrirait le gel FORME de G et casserait le drift-gate). Portée bornée
   aux champs de classification. Test `static_validation_rejects_classification_from_model`.

**S'y ajoutent (freeze §8, l.158-161), déjà listés §10.5 et déclarés-par-la-forme-G, dont le REFUS est
H** : satisfiabilité du join (all vs branches exclusives), budget enfant ≤ parent PAR dimension,
`capability_ref ∈ capability_requests`, handoff autorisé par le donneur (`∈ allowed_handoffs`), refs
pinnées résolubles.

---

## 9. Validation §10.5 — liste exhaustive des classes de refus à implémenter

**13 refus §10.5** (`research §10.5:898-912`, verbatim) + **6 règles freeze §8** (§8 ci-dessus) +
**1 SOTA** (compat de ports, freeze §7 / S1a-03) = **20 classes**, un test discriminant
mutation-killed chacune. Le plan H.3 n'en nomme que 6 : les 14 restantes sont à nommer.

| # | Classe de refus | Source | Nommée H.3 ? |
|---|---|---|---|
| 1 | node ou sortie inaccessible | §10.5 | oui (`unreachable_node`) |
| 2 | cycle non porté par un node de boucle borné | §10.5 | non |
| 3 | `map` sans limite de cardinalité | §10.5 | oui (`unbounded_map`) |
| 4 | join dont branches et politique incohérentes | §10.5 | non |
| 5 | référence (schéma/agent/prompt/sous-workflow) non pinnée | §10.5 | oui (`unpinned_ref`) |
| 6 | output non typé | §10.5 | non |
| 7 | capacité demandée absente du manifeste du pack | §10.5 | non |
| 8 | provider incompatible (modèle / sortie structurée) | §10.5 | non |
| 9 | handoff non autorisé par l'agent cible | §10.5 | oui (`unauthorized_handoff`) |
| 10 | secret literal | §10.5 | oui (`secret_literal`) |
| 11 | endpoint ou code exécutable interdit | §10.5 | non |
| 12 | budget enfant > budget parent (PAR dimension) | §10.5 | oui (`budget_overflow`) |
| 13 | publication sans provenance / HITL / évaluation requise | §10.5 | non |
| 14 | intégrité référentielle des edges (+ edge dupliquée S1a-07) | freeze §8.1 | non |
| 15 | unicité des ports par label + join satisfiable | freeze §8.2 | non |
| 16 | rejet de tout cycle sur edges quelconques (DAG) | freeze §8.3 | non |
| 17 | clés data-driven ∌ `PRESENTATION_KEYS` | freeze §8.4 | non |
| 18 | labels de routage sains (`default` ∉ branch_labels ; non vides) | freeze §8.5 | non |
| 19 | provenance des discriminants de classification | freeze §8.6 | non |
| 20 | edge port-incompatible (nominale par digest v1) | freeze §7 / S1a-03 | non |

NB : #2 et #16 se recouvrent (#16 est strictement plus fort : « tout cycle », pas seulement « cycle
non porté par une boucle bornée ») — les tester tous deux ; #16 est la cible v1 (DAG).

---

## 10. Passe adversariale — ce qui a été réfuté / ajusté, et pourquoi

**Aucun finding entièrement RÉFUTÉ** : les 3 passes (exactitude-code, cohérence-décisions,
sécurité-invariant) ont re-mesuré chaque finding — tous **CONFIRMED** sauf **4 ADJUSTED** (sur-affirmation
resserrée). 5 `findings_neufs` intégrés.

### 10.1 Findings ajustés (survivent, resserrés)

- **S1a-02 / S1b-06 ADJUSTED** — le mécanisme est réel mais deux sur-affirmations tombent : (1)
  l'identité des nodes a été pinnée par **G** (BTreeMap), pas H ; (2) « WA-T1.2 structurellement
  non-atteignable » est faux au read-path H (canvas et outline lisent le MÊME `edges` Vec d'un pack
  pinné unique → digest identique trivialement). La canonicalisation d'ordre des edges reste une
  obligation forward SCOPÉE (nécessaire pour le digest de niveau CompiledPlan, et pour deux surfaces
  d'authoring indépendantes = Phase T), pas un blocage.
- **S1a-04 ADJUSTED (3 angles)** — le principe CaMeL est sain mais H NE mandate PAS un binding
  `capability_ref→{node_instance_id[]}` d'enforcement : `§12.1:969` mandate un SET (fermeture) ;
  l'enforcement least-authority PAR-EFFET est RUNTIME (Phase J/K) et l'intersection 7 couches = Phase L.
  H fixe le SET + émet `node_instance_id` comme ENABLER runtime.
- **S1a-07 ADJUSTED** — le self-loop `from==to` est un cycle trivial DÉJÀ subsumé par la règle 3 §8 ;
  seule l'edge-dupliquée est un vrai ajout.
- **S2-01 ADJUSTED (sévérité BLOCKER→MAJOR)** — les 6 règles §8 sont CONSIGNÉES dans un fichier pivot
  obligatoire, pas absentes ; le défaut est une sous-énumération du texte du plan (PLAN-ADAPT MAJOR),
  pas un BLOCKER qui halte le sprint.
- **S4-05 ADJUSTED** — pas de « décision (a)/(b) ouverte » : le scope cut TS→T + T1=N-A imposent la
  voie (b) (snapshot Rust seul). Aligné sur S2-13.

### 10.2 Findings neufs intégrés (tous MINOR/INFO, aucun DESIGN-CONFLICT)

- **NEUF-01 / NEUF-SEC-02 [MINOR] — Locus de la canonicalisation d'ordre à trancher.** Le
  `graph_semantic_view` faisant autorité pour « canvas=outline » est une primitive du module PACK
  gelée Phase F (freeze §7 « AUCUNE règle de digest neuve ») ; H ne peut pas trivialement la ré-écrire.
  → H fait porter la normalisation d'ordre au **digest de niveau CompiledPlan** (`digest_for(GraphSemantic,
  &plan_normalisé)`, que H owne), PAS à la primitive PACK ; le `pack.graph_semantic` signé (émis par le
  Hub) est canonicalisé à l'authoring, Phase T. Fixer cette couche dans le plan H pour que le tri
  d'edges (S1a-02) atterrisse au bon endroit.
- **NEUF-02 [INFO] — Réutiliser le patron de contrôles négatifs existant.** Le bras rouge discriminant
  du test Plan-Then-Execute (S3-05) doit se calquer sur `golden_negative_controls_reject`
  (`graph.rs:964-968`, « keep `four_goldens_validate` non-vacuous »), pas inventer une structure neuve.
- **NEUF-COH-01 [INFO] — Compiler-logic-only, jamais étendre les schémas gelés.** Les 19+ refus doivent
  s'implémenter comme LOGIQUE de compilateur lisant les schémas graph/agent DÉJÀ GELÉS (SET + FORME,
  `freeze §4`). Ajouter le moindre champ (annotation « classification » pour la règle 6, champ de
  provenance de port pour la compat) rouvrirait le gel FORME de G et casserait `schema_snapshot_stub`.
  La règle 6 s'enforce par ANALYSE data-flow sur les ports typés existants.
- **NEUF-SEC-01 [MINOR] — Facette signée `pack.digests.compiled_plan` = placeholder shape-only.**
  Aujourd'hui `pack.rs:195-201` : `compiled_plan` « validated for SHAPE only… binding them to the
  artefacts they name needs those artefacts, which this pure crate never receives » ; le test
  `six_digests_stable` calcule le digest via `_ => document.clone()` (doc BRUT, `pack.rs:1519-1530`).
  H est la 1re/SEULE phase où l'artefact compilé existe in-crate. → H tranche consciemment : soit il
  ferme le binding (`compiled_plan := digest_for(CompiledPlan, plan_réel)`), soit il documente que ça
  reste une revendication forward vérifiée plus tard — sans quoi les `*.compiled.golden.json` (H) et
  les pack goldens (F) portent une sémantique `compiled_plan` incohérente. Pas un trou de sécurité
  actif (content digest + Ed25519 + compilateur déterministe couvrent l'intégrité du graphe).

---

## 11. Plan d'implémentation concret (le code suit CE préflight, pas le texte brut du plan)

### 11.1 Blocants — à trancher AVANT le premier `Edit`

- **B1 — Énumérer les 20 classes de refus (§9), un test discriminant mutation-killed chacune.** Le
  plan H.3 n'en nomme que 6 : nommer les 14 restantes (13 §10.5 dont 7 non-nommés + 6 freeze §8 + compat
  de ports). Ne PAS s'arrêter aux 6 identifiants du plan.
- **B2 — Test `plan_then_execute_invariant` NON vacueux.** 9 classes §19.3 (ou réduction justifiée) +
  bras rouge discriminant : (i) sortie-modèle hostile → CompiledPlan/digest INCHANGÉ ; (ii) MÊME contenu
  en input pack → plan DIFFÉRENT. Patron `graph.rs:964` (NEUF-02). Idem `four_goldens_compile` : prouver
  qu'un pack malformé/muté est REFUSÉ (pas seulement que les 4 valides compilent).
- **B3 — Posture FRONTIER = PROSE, 0 DOMAIN, census 29.** Ne PAS écrire `// FRONTIER: CompiledPlan
  domain=… version=…` (casse le gate/census). CompiledPlan local/non-signé en H. Voie (a) S2-04.
- **B4 — Locus du `graph_semantic_digest` (NEUF-01/NEUF-SEC-02).** Normalisation d'ordre au niveau
  CompiledPlan (owné par H), pas dans la primitive PACK gelée. Décider AVANT de coder le tri d'edges.
- **B5 — Acquittement THREAT_MODEL (S3-01).** Trancher : H sème une §19 « Plan-Then-Execute / graphe
  signé = plan » (recommandé) OU carry STANDING explicite vers AH (jamais un silence).

### 11.2 Modules / fichiers (ordre d'implémentation)

1. **`crates/sbfb-workflow-core/src/compile.rs`** (neuf, `pub mod compile;` dans `lib.rs`) : SPDX
   `AGPL-3.0-or-later` en tête. `compile(pack) -> Result<CompiledPlan, CompileError>`. `CompileError`
   hand-rolled (enum + `Display` + `Error`, patron `PackError` `pack.rs:149-188`), une variante nommée
   par classe de refus avec `path`/`detail`. 0 dep (`thiserror`/`anyhow` interdits).
2. **Remplir `CompiledPlan` (`contracts.rs:82-85`)** — 9 familles de champs §12.1 (S2-06), indexées par
   `BTreeMap` (déterminisme, S1a-05/S1b-03) : `pack_digest` (= `content` du pack signé, S3-07),
   digests exacts workflows/agents/prompts/schemas/outils, `capability_closure` (SET fermé, indexable
   par node comme DONNÉE — pas d'enforcement per-effet, S1a-04), `node_instances` (`node_instance_id`
   fonction pure du chemin structurel, `§12.2`), routes/joins/bornes normalisés, budgets hérités +
   atténués (4 dims), politique placement, politique retry/timeout/cancel, `compiler_version`+profil.
   Conserver `deny_unknown_fields` + `title`. `len==7` intact.
3. **Validateur statique** — 20 refus (§9), DAG-check maison (Kahn/DFS), analyse data-flow pour la
   règle 6 (NEUF-COH-01 : lecture des schémas gelés, JAMAIS extension). Chaque refus lit les 19 sites
   annotés `Phase H` de `graph.rs` (S2-10) comme contrat.
4. **Digests + normalisation d'ordre** — `digest_for(PackDigestKind::CompiledPlan/GraphSemantic, …)`
   via `canonical_bytes` (JCS), JAMAIS `serde_json::to_vec` (S1b-04). Tri stable des `edges`
   `(from,from_port,to,to_port)` + tout `Vec` non-identifiant AVANT le `graph_semantic_digest` de
   niveau CompiledPlan (B4).
5. **Fixtures `*.compiled.golden.json`** (4 packs, Factory PREMIER) — régénérées via `UPDATE_SNAPSHOTS`
   sous `cargo test --workspace`, comparées en `serde_json::Value`, tests joués `-p` ET `--workspace`.
6. **Refresh `contracts.schema.json`** — `UPDATE_SNAPSHOTS=1` dans le MÊME commit (S2-05, S4-04),
   `schema_snapshot_stub` STRICT reste vert, `len==7`.
7. **Prose census (S4-06)** — ajuster 1 ligne (`check-frontier-contracts.sh:207-211` + PATTERNS §P70)
   distinguant « identité contrat remplie » vs « tag DOMAIN réservé non-signé », NAME LIST inchangée.
8. **THREAT_MODEL §19 (B5, si H sème)** — asset « graphe signé = plan / Plan-Then-Execute », compilateur
   = point d'enforcement ; cadence §18 (§7 mitigations + hash commit).

### 11.3 Tests (au-delà des 6 nommés au plan)

- `compile::tests::four_goldens_compile` (Factory premier) + **bras rouge** pack muté REFUSÉ.
- `compile::tests::static_validation_rejects_{…}` — **20 classes** (§9), mutation-killed chacune.
- `compile::tests::plan_then_execute_invariant` — 9 classes §19.3 + bras discriminant (i)/(ii).
- `compile::tests::compile_is_deterministic` — `compile(pack)` deux fois → byte-identique (S1a-05).
- `compile::tests::graph_semantic_digest_order_independent` — deux ordres d'edges → même digest (S1a-02).
- Reproductibilité dual-platform : verts Win natif ET Docker `sbfb-ci` 1.94 (piège `preserve_order`).

### 11.4 Points de vigilance

- `preserve_order` : digest via `serde_jcs` uniquement ; comparaison en `Value` ; régénération sous
  `--workspace` ; `-p` ET `--workspace` verts (S1b-04/S4-03).
- Digest canonique : `content` = autorité de signature (pack root), `graph_semantic`/`compiled_plan` =
  dérivés LIÉS, jamais surface d'intégrité autonome ; Ed25519 vérifiée AVANT parse (S3-07).
- Honnêteté des claims : « DAG prouvé par COMPILATION » (pas by construction) ; « H prouve la moitié
  statique de Plan-Then-Execute, le runtime revalide en J » (S3-06/S3-08).
- Déterminisme : BTreeMap partout, `node_instance_id` = chemin structurel pur, aucun compteur/UUID/
  timestamp (S1a-05).

### 11.5 Interdits

- Pas de `DOMAIN_*`/`*_FORMAT_VERSION` neuf (census gelé 29) ; pas de `// FRONTIER:` machine sur
  CompiledPlan (non signé) ; pas de `canonical_bytes(DOMAIN_COMPILED_PLAN_V1)` en H.
- Pas de touche à `Cargo.toml`/`Cargo.lock` (0 dep) ; pas de `thiserror`/`anyhow`/`indexmap`/`petgraph`/
  crate `jsonschema` ; pas de dev-dep (`proptest`/`quickcheck`).
- Pas de touche à `sbfb-workflow-engine` (Phase I/J), à `web/`/`tools/factory-operator/` (T1=N-A,
  miroir TS = Phase T), au module PACK gelé (`graph_semantic_view`).
- Pas de nouveau champ de schéma graph/agent (rouvrirait le gel FORME de G, NEUF-COH-01) ; la règle 6
  = analyse data-flow.
- Pas de comparaison goldens/snapshot en string (piège `preserve_order`).

### 11.6 Invariants à vérifier (commande par invariant)

| Invariant | Commande | Attendu |
|---|---|---|
| census intact | `bash scripts/check-frontier-contracts.sh` | exit 0, `[29 frozen]` |
| registre 7 + 4 goldens compilent | `cargo nextest run -p sbfb-workflow-core --locked` | exit 0, `len == 7` |
| 20 refus testés + non vacueux | tests `static_validation_rejects_*` + bras rouge | pack muté → ROUGE |
| Plan-Then-Execute discriminant | `plan_then_execute_invariant` (9 classes + (i)/(ii)) | (i) digest inchangé, (ii) plan différent |
| reproductibilité | `compile_is_deterministic` | byte-identique |
| 0 dep neuve | `git diff Cargo.toml Cargo.lock` | vide |
| ordre-robuste | tests `-p` ET `--workspace` | verts dans les deux |
| schema à jour | `UPDATE_SNAPSHOTS=1` puis run sans var | pas de drift |
| SPDX | `bash scripts/check-spdx.sh` | exit 0 |
| 0 diff web/ | `git diff --stat web/ tools/factory-operator/` | vide |
| dual-platform | `fmt --check` ; `clippy --workspace --all-targets -D warnings` ; `nextest --workspace` ; `test --doc` ; `build -p nexus-shell-daemon --release` | verts Win **ET** Docker `sbfb-ci` (1.94) |

---

## 12. Scope NON couvert par H (frontières I/J/K/L/L1/P/T — honnêtes)

Le compilateur ferme le **STATIQUE**. NE ferment PAS en H :

- **Phase I** — run store durable SQLite, `RunEvent.v1` + upcasters + snapshots jetables + inbox/outbox
  + fencing single-writer. Le crate `sbfb-workflow-engine` EXISTE déjà (Phase E) ; H ne le touche pas.
- **Phase J** — reducer/scheduler déterministe, machine d'états (11 run / 10 node §12.3), executors de
  nodes, cancel honnête, `outcome_unknown` (préparé). `node_instance_id`/`effect_id` définis par le plan,
  CONSOMMÉS par le moteur.
- **Phase K** — `effect_id`/`attempt_id`/reçus/idempotence, revalidation tool-call runtime (`§19.3:1612`).
- **Phase L / L1** — intersection 7 couches / atténuation per-effet / audience / profondeur / coffre
  `secret_ref` + binding keystore / HITL signé lié à `run_id+plan_digest+node_instance_id`. H ne fait que
  la FERMETURE statique du SET des capacités + refus des secrets LITTÉRAUX, PAS le binding runtime.
- **Phase P** — SSRF/DNS/redirect (`§19.5`).
- **Phase T** — miroir TS / canvas / enveloppe présentation / run effectif du palier WA-T1.2
  (`NOT-RUN` jusqu'à AI2). L'anti-drift de H = snapshot Rust seul.
- **Signature « attestation compilateur »** (`§17.1`) — rôle séparé différé, hors §H.2.

---

## 13. Verdict G8

**PLAN-ADAPT.**

Justification (§4.5.7) — **aucune décision Day-0 (D3/D4 GELÉES) ni aucun invariant du plan n'est en
tension structurelle** : autorité sémantique unique hors-LLM, validation statique §10.5, fermeture des
capacités au compile, `CompiledPlan.v1` immuable, digests sémantiques, invariant Plan-Then-Execute
restent exactement ceux du kickoff `:424-499` et du Research Pack §10.5/§12. Le prior art réel 2026
(LangGraph, MS Agent Framework, Agentproof, CaMeL, Temporal/Nix, Argo/GHA) **valide** ces choix — la
fermeture-au-compile est un ajout SOTA genuine. Le verdict n'est **pas DESIGN-CONFLICT** : aucune
alternative rejetée n'est ré-ouverte, aucun arbitrage PO n'est requis ; toutes les corrections sont
des précisions de FORME de schéma, d'énumération de refus et de spécification de test, chacune appuyée
sur une evidence code/OSS rejouée à HEAD (`24ff47e2` : stub vide 6/7, tag réservé « confers NO
acceptance », census 29, `edges` Vec vs `nodes` BTreeMap, 19 sites `Phase H`, `BudgetPolicy` 4 dims,
golden ideas `any`+`human.review`, hazard `preserve_order`, THREAT_MODEL plafonné à §18). Le verdict
n'est **pas EXECUTE** parce que le texte du plan H omet 14 des 20 classes de refus, réduit le test
Plan-Then-Execute à 3 des 9 classes, ne re-liste pas les 6 règles freeze §8, laisse la posture
domaine/signature et l'acquittement THREAT_MODEL implicites, et nomme un miroir TS inexistant. Il n'est
**pas SCOPE-CUT-CONSISTENT** : rien n'est retiré — le périmètre est conservé et précisé.

**Approche corrigée que le code suit** (autorité : ce préflight) : (1) énumérer les **20 classes de
refus** (§9), un test discriminant chacune ; (2) rendre `plan_then_execute_invariant` et
`four_goldens_compile` NON vacueux (9 classes §19.3 + bras rouge discriminant, patron `graph.rs:964`) ;
(3) remplir le stub `CompiledPlan` (9 familles §12.1, BTreeMap, `node_instance_id` chemin pur) + refresh
`contracts.schema.json` même commit, `len==7` ; (4) FRONTIER en PROSE, 0 DOMAIN, census 29 ; (5)
normaliser l'ordre des edges au niveau CompiledPlan (B4/NEUF-01), digests via `serde_jcs`, dual-platform ;
(6) implémenter la règle 6 par analyse data-flow sur les schémas GELÉS (NEUF-COH-01) ; (7) trancher
l'acquittement THREAT_MODEL (H sème §19, AH confronte) ; (8) borner honnêtement (DAG par compilation ;
H = moitié statique de Plan-Then-Execute ; runtime revalide en J) ; (9) body 9 sections conforme H.5,
annonçant « 0 bump wire, census 29 intact, 0 dep runtime », scope cuts datés (runtime → I/J/K, 7 couches
→ L, secret binding/HITL → L1, SSRF → P, miroir TS/canvas → T), report THREAT_MODEL tracé.
