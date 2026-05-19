# Audit de completude — SYNTHESIS_factory_rrv_protocol.md

**Date :** 2026-05-19
**Methode :** Lecture exhaustive des 14 documents source + lecture complete
de la synthese. Chaque fait majeur, tableau, decision, schema et
recommandation du source est trace dans la synthese.

---

## 1. Table de couverture document par document

| # | Document source | Faits/decisions majeurs | Captures | Manquants | Couverture |
|---|----------------|------------------------|----------|-----------|------------|
| 1 | `sbfb_rrv_code_factory_vision_pitch.md` | 8 | 8 | 0 | **100%** |
| 2 | `sbfb_project_factory_rrv_oss_research.md` | 42 | 40 | 2 | **95%** |
| 3 | `rrv_scoped_search_compute_groups.md` | 28 | 27 | 1 | **96%** |
| 4 | `s70_s72_rrv_research.md` | 48 | 45 | 3 | **94%** |
| 5 | `s65_s75_factory_babel_canary_research.md` | 38 | 37 | 1 | **97%** |
| 6 | `factory_deploy_constraint_research.md` | 22 | 22 | 0 | **100%** |
| 7 | `factory_first_feasibility_audit.md` | 35 | 33 | 2 | **94%** |
| 8 | `factory_gates_audit.md` | 30 | 29 | 1 | **97%** |
| 9 | `babel_canary_scope_validation.md` | 26 | 25 | 1 | **96%** |
| 10 | `protocol_neutrality_api_audit.md` | 24 | 23 | 1 | **96%** |
| 11 | `protocol_neutrality_prior_art.md` | 32 | 31 | 1 | **97%** |
| 12 | `factory_as_client_gap_analysis.md` | 28 | 28 | 0 | **100%** |
| 13 | `rrv_protocol_boundary_analysis.md` | 22 | 22 | 0 | **100%** |
| 14 | `rrv_scope_ordering_analysis.md` | 18 | 17 | 1 | **94%** |
| **TOTAL** | | **401** | **387** | **14** | **96.5%** |

---

## 2. Detail des manques par document

### Doc #2 — `sbfb_project_factory_rrv_oss_research.md`

1. **Etat worktree observe (section 1.3)** — Le document source mentionne
   l'etat precis du worktree au moment de la recherche (fichiers modifies,
   fichiers non trackes) et l'implication "ne pas melanger cette research
   avec une phase code Sprint 64". Ce contexte temporel n'est pas repris
   dans la synthese. **Impact : negligeable** (contexte ephemere).

2. **Detail des 14 questions ouvertes (section 14)** — Le source contient
   10 questions ouvertes detaillees. La synthese reprend la majorite dans
   la section 10.1 mais omet la question #3 (Copier comme binaire externe
   vs via lib Python dans un broker), qui est partiellement couverte par
   Q3 mais sans le detail Python. **Impact : faible** (Q3 de la synthese
   couvre l'essentiel).

### Doc #3 — `rrv_scoped_search_compute_groups.md`

1. **Regles worker detaillees (section 7.2)** — Le source detaille 10
   regles worker (modele autorise, VRAM max, watts max, horaires, quotas,
   langues acceptees, type de tache, publication ou non, kudos/recompense).
   La synthese mentionne le concept de regles de consentement worker
   (section 8.2) mais ne reprend pas la liste exhaustive des 10 regles.
   **Impact : faible** (compute public est long terme, non planifie
   avant S73+).

### Doc #4 — `s70_s72_rrv_research.md`

1. **Sonic (section 2.4) et Meilisearch (section 2.5)** — Le source
   analyse ces deux alternatives Rust et les rejette avec justification.
   La synthese ne mentionne ni Sonic ni Meilisearch dans la comparaison
   FTS5 vs Tantivy (section 4.3). **Impact : faible** (les deux sont
   rejetes, l'absence de mention ne change pas la decision).

2. **DOAP / Description of a Project (section 3.5)** — Le source evalue
   DOAP comme reference pour SearchManifest. La synthese ne le mentionne
   pas. **Impact : negligeable** (DOAP est rejete, seuls les champs
   conceptuels sont notes comme reference).

3. **AppStream Linux (section 3.4)** — Le source analyse le format
   AppStream (freedesktop) comme inspiration pour SBFB.json enrichi
   (metadata_license, categories structurees, keywords, changelogs).
   La synthese ne mentionne pas AppStream. **Impact : faible** (les
   champs inspires d'AppStream sont presents dans le manifest v2 de
   la synthese, mais sans la reference d'inspiration).

### Doc #5 — `s65_s75_factory_babel_canary_research.md`

1. **Synchronisation canonique requise (section 13)** — Le source
   liste 6 fichiers a corriger (roadmap v2, CLAUDE.md,
   doc_classification.md, claudemd_update_plan.md, .planning/active,
   PUBLISH_MODEL.md). La synthese ne reprend pas cette liste de
   corrections canoniques. **Impact : faible** (liste operationnelle,
   pas une decision d'architecture).

### Doc #7 — `factory_first_feasibility_audit.md`

1. **Gouvernance Option A (section 5, Option A)** — Le source decrit
   en detail l'option "Gouvernance absorbee dans S65-S66" avec
   estimation de scope creep (~470 LOC supplementaires) et verdict
   REJETE. La synthese mentionne le verdict de l'Option D recommandee
   mais ne reprend pas le detail du calcul de rejet de l'Option A
   (les ~470 LOC et le risque de dilution). **Impact : faible** (la
   decision finale est correctement capturee).

2. **S68 UI /factory composants detailles (section 4.1)** — Le source
   detaille 6 composants React (TemplateSelector ~60 LOC,
   VariablesForm ~100 LOC, DiffViewer ~200 LOC, PreviewFrame ~50 LOC,
   PublishChecklist ~80 LOC, FactoryPage layout ~80 LOC) pour un
   total de ~570 LOC. La synthese mentionne "~570 LOC React" en
   section 11, Phase 2 S68 Phase C, mais ne reprend pas le detail
   composant par composant. **Impact : negligeable** (le total est
   capture, le detail est operationnel).

### Doc #8 — `factory_gates_audit.md`

1. **Comparaison gates workflow G1-G9 vs gates Factory — detail
   complet (section 7.1-7.2)** — Le source contient un tableau
   comparatif complet des gates workflow vs Factory et identifie le
   probleme de collision de nommage. La synthese reprend la decision
   de renommage FG0-FG10 (section 3.4) et note la reference (doc#8
   §7.3) mais ne reproduit pas le tableau comparatif complet des deux
   systemes de gates. **Impact : negligeable** (la decision de
   renommage est capturee, le tableau est un detail justificatif).

### Doc #9 — `babel_canary_scope_validation.md`

1. **Comparaison S69 original vs S69 revise (section 8)** — Le source
   contient un tableau comparatif detaille des activites S69 original
   (code applicatif 0j, tests infra 3j, invite/onboarding 2j,
   feedback/guides 2j, go/no-go 1j) vs S69 revise (code applicatif
   5-8j, tests infra reduit, etc.) avec le verdict "scope creep mais
   avec bonne raison". La synthese ne reprend pas ce tableau
   comparatif. **Impact : faible** (la conclusion "scope comparable"
   est implicite dans la roadmap de la synthese).

### Doc #10 — `protocol_neutrality_api_audit.md`

1. **Table complete des 68 routes HTTP (section 1)** — La synthese
   resume la repartition 40P/19W/7S/3D (section 2.2) mais ne
   reproduit pas la table complete route par route. **Impact :
   negligeable** (la table est un inventaire reference, le resume
   de repartition est suffisant pour la synthese).

### Doc #11 — `protocol_neutrality_prior_art.md`

1. **Plan d'action concret (section 7.5)** — Le source propose un
   plan en 5 etapes (S66-S67 stabiliser API events, S67 definir
   contrat API, S67-S68 Factory sidecar, S70 RRV indexeur, post-S72
   documenter schemas ops). La synthese reprend ces elements de
   maniere distribuee dans la sequence de travail (section 11) et
   dans l'annexe F (evenements daemon) mais ne les presente pas
   comme un plan d'action numerote. **Impact : negligeable** (le
   contenu est present, la forme est differente).

### Doc #14 — `rrv_scope_ordering_analysis.md`

1. **Tableau cout/valeur par utilisateur (section 4)** — Le source
   contient 3 sous-tableaux detailles (FlowUP, testeurs pilote,
   futurs developpeurs) avec des ratios cout/valeur par scope. La
   synthese capture la conclusion (section 4.2 : "@protocole d'abord")
   mais ne reprend pas les 3 tableaux detailles de ratio. **Impact :
   faible** (la conclusion est capturee, les tableaux sont la
   justification detaillee).

---

## 3. Faits majeurs correctement captures (echantillon representatif)

Les faits suivants, consideres critiques, sont correctement presents
dans la synthese :

- Vision boucle RRV-Factory-App-Brique (doc#1 -> synthese section 1.1)
- Decision FTS5 avant Tantivy, gate post-S75 (doc#4/5 -> synthese D1)
- Decision node_id hors manifest, Option D (doc#6 -> synthese D3, section 3.7)
- 5 options de contrainte node_id avec matrice de comparaison (doc#6 -> synthese section 3.7)
- Factory hors daemon, crate sbfb-factory (doc#12 -> synthese D2, section 3.1-3.2)
- Ordonnancement @protocole avant @dev avant @web (doc#14 -> synthese D6, section 4.2)
- SBFB.json v2 complet avec struct Rust et exemples JSON (doc#6 -> synthese section 3.8, annexe A)
- Gates Factory FG0-FG10 avec LOC, testabilite, code reutilisable (doc#8 -> synthese section 3.4)
- Proof Card data model + formule de score deterministe (doc#4 -> synthese section 4.6)
- SearchManifest wire format complet + opt-in + gossip (doc#4 -> synthese section 4.7)
- 8 labels de preuve (doc#2 -> synthese section 4.8)
- Prior art 5 protocoles P2P + 7 patterns communs (doc#11 -> synthese section 12)
- 4 anti-patterns documentes (doc#11 -> synthese section 12.3)
- Babel canari scope MVP + scope cuts recommandes (doc#5/9 -> synthese section 5.2)
- Bridge methods 11/11 completes (doc#9 -> synthese section 5.2)
- Briques OSS P0/P1/P2 avec 38 projets references (doc#2 -> synthese section 6)
- Anti-decisions OSS (doc#2 -> synthese section 6.4)
- Supply chain regles (doc#2 -> synthese section 7.3)
- Compute distribue : batch vs inference temps reel (doc#3 -> synthese section 8)
- CuratorVouched/CuratorDisendorsed payloads Rust (doc#12 -> synthese annexe D)
- Effort estime S67-S69 : ~3230 LOC, +69-94 tests (doc#7 -> synthese annexe E)
- Prerequisits avant S67 : checklist 7 items (doc#7 -> synthese annexe E)
- Evenements daemon contrat d'integration (doc#11 -> synthese annexe F)
- Routes W verdict externalisation (doc#10 -> synthese annexe G)
- 16 decisions gerees dans table D1-D16 (consolide -> synthese section 9.1)
- 7 tensions resolues (consolide -> synthese section 9.2)
- 17 questions ouvertes prioriees (consolide -> synthese section 10.1)
- Risques P0/P1/P2/P3 consolides (14 docs -> synthese section 13)
- 25 tests d'acceptance Babel (doc#5/9 -> synthese section 5.3)
- Tests Factory + RRV consolides (doc#2/4/8 -> synthese section 14)

---

## 4. Qualite de la synthese

### Points forts

- **Tracabilite systematique** : Chaque fait est reference `(doc#N §X)`.
  Permet de remonter au source pour detail.
- **Deduplication effective** : Les 14 documents contenaient de
  nombreuses repetitions (SBFB.json v2 decrit dans doc#5, #6, #7 ;
  gates dans doc#5 et #8 ; briques OSS dans doc#2). La synthese
  elimine les doublons et consolide.
- **Decisions explicites** : La table D1-D16 est un apport net de la
  synthese — aucun document source ne contenait une table unifiee
  des decisions.
- **Questions resolues vs ouvertes** : La synthese distingue clairement
  les questions tranchees (section 10.2) des questions encore ouvertes
  (section 10.1). C'est un travail de curation absent des sources
  individuelles.
- **Schemas Rust et JSON complets** : Les annexes A et D reproduisent
  les schemas critiques sans perte.

### Points faibles

- **Volume** : La synthese fait ~1990 lignes. C'est dense mais
  proportionnel a la quantite de contenu source (~15 000 lignes
  cumulees).
- **Certaines sous-tables perdues** : Les tableaux comparatifs fins
  (composants React, regles worker, cout/valeur par persona, S69
  original vs revise) ne sont pas reproduits. Acceptable car les
  conclusions sont presentes, mais un lecteur cherchant le detail
  devra consulter le source.

---

## 5. Score de couverture global

| Metrique | Valeur |
|----------|--------|
| Faits majeurs dans les sources | 401 |
| Faits captures dans la synthese | 387 |
| Faits manquants | 14 |
| **Couverture globale** | **96.5%** |
| Faits manquants a impact > faible | **0** |
| Faits manquants a impact faible | 8 |
| Faits manquants a impact negligeable | 6 |

---

## 6. Verdict

**La synthese est de haute qualite.** Les 14 manques identifies sont
tous d'impact faible ou negligeable :

- 6 sont des details contextuels ou ephemeraux (etat worktree, liste
  de fichiers a corriger, detail composants React)
- 5 sont des alternatives rejetees dont l'absence ne change pas les
  decisions (Sonic, Meilisearch, DOAP, AppStream, Option A Gouvernance)
- 3 sont des tableaux detailles dont la conclusion est presente mais
  pas le tableau complet (regles worker, S69 comparatif, cout/valeur
  par persona)

**Aucune decision, aucun schema structurel, aucune recommandation
majeure ne manque.** Le document est utilisable comme reference
autonome pour les sprints S67-S75 sans consulter les sources,
sauf pour des details operationnels de second niveau.
