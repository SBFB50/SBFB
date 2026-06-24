# Prompt — continuer la réflexion « doctrine du contrat-pour-LLM »

> À coller au démarrage d'une prochaine session. Hors-process / découverte.
> Le chat précédent n'est PAS porté : lis les artefacts cités (source de vérité),
> ne fais confiance à ce prompt sur aucun fait — vérifie dans le code.

## Lis d'abord (artefacts réels, session précédente)
- `examples/daisyui-animejs-showcase/knowledge/README.md` + `daisyui/README.md` — les 2 knowledge packs (anime.js 93 primitives / daisyUI 68 composants), méthode 5-couches, ancrés source.
- `…/knowledge/factory-integration-design.md` + `…/factory-integration-hardened.md` — capacité Factory « app-authoring » (design durci, 8 questions tranchées, contrat CSP).
- `…/knowledge/ideas/IDEAS.md` + `sbfb-mapping.md` — Idea Engine (génératif → curation humaine) + mapping SBFB.
- `.planning/active/sprint79_factory_kickoff.md` / `_plan.md` / `_design_review.md` + `_NEXT_SESSION.md` — le sprint Factory PRÊT (à ne pas confondre avec CETTE réflexion).

## La thèse (à continuer, pas à re-prouver)
**Toute primitive de FRONTIÈRE du projet** (wire format, protocole, API publique, contrat d'app — *pas* un helper interne) **= un contrat source-ancré, drift-gated, consommable par un LLM.** Ce n'est pas « plus de doc » : c'est un **graphe de surfaces ancrées dans la source et gatées contre le drift**.

## Les couches identifiées (nœuds + arêtes)
**Nœuds :**
1. **CODE** = le QUOI (comportement actuel).
2. **ÉTIQUETTE** (schéma **généré**, drift-gated, **par phase**) = le CONTRAT (forme + invariants, toujours à jour car le build casse si schéma ≠ code).
3. **COMMIT ATOMIQUE** (titré `feat(scope): Sprint N Phase X`, body à sections imposées, signé/provenance) = le POURQUOI/QUAND/DELTA.
4. **GUIDE + `llms.txt`** (synthèse humaine/agent, **en clôture**, Truth-Stack `repo>planning>commits>prompts>chat` + règle « Not evidenced ») = l'INDEX.

**Arêtes :**
5. **Commentaires de provenance in-code** (`// Sprint N Phase X · §P64 · décision PO #N`) = les liens code↔décision↔contrat, en rang-1, **survivent au refactor**. ⚠️ uniquement en **arrière vers de l'immuable** (sprint/phase/§/decision# qui ont eu lieu), **jamais des promesses** — cf. carry réel `STALE-PHASE-K-COMMENTS` (des `// lands in Phase K` ont menti). À gater par `source-ref-check`.

## La règle de cadence (tranchée)
- **Étiquette générée → CHAQUE phase**, dans le commit de la primitive (gratuit car généré ; la gate ne peut pas pourrir).
- **Guide (synthèse) → UNE seule phase de clôture** (image complète figée).
- Ni « une phase de doc par phase » (lourd), ni « tout à la fin » (faux pendant le sprint, bâclé à l'arrivée).
- Leçon de L/M/N (S77) : **L** (schémas) aurait dû être dispersé par phase ; **M/N** (synthèse) correctement groupés à la fin.

## Les 3 compagnons prouvés efficaces cette session (à intégrer)
1. **Priorité des sources** : `schéma/type généré > tests > llms.txt/doc officielle > prose ; jamais inventer`. (daisyUI **livre** un `llms.txt` officiel ; anime des `.d.ts` ; on génère via schemars.)
2. **Sonde de comportement** : toute primitive **vivante** livre une sonde rejouable qui rend un **verdict machine** (`PASS/ADJUST/BLOCK`), pas une capture. (seis-probe/gears-probe ont **trouvé un vrai bug** + permis le tuning ; c'est exactement le `scripts/acceptance/b3_*.sh` du projet.)
3. **Vérification adversariale des FAITS** : un agent **indépendant** confronte le contrat à la source avant qu'il soit cru. (a **trouvé le trou** de `check-csp.mjs` : `form-action`/`base-uri` manquants + drift esm→umd.)
- (+) Principe génératif : **la machine réduit/note, l'humain arbitre le goût** (modèle Idea Engine).

## Pourquoi ça aide RRV (objectif)
RRV = garbage-in/garbage-out ; sa qualité = qualité de la source. Les **étiquettes** le rendent **vérifiable-par-machine** (le « V ») ; le **llms.txt** **navigable sans halluciner** ; les **commits** = couche décision **attribuable + signée** ; les **commentaires** = les **arêtes** zéro-saut en rang-1. Les 5 modes (@research/@dev/@audit/@security/@product) lisent chacun une couche. Gains mesurables : justesse, **fraîcheur garantie** (drift-gate), anti-hallucination (Not-evidenced), vérifiabilité, cohérence multi-agent.

## Où ça doit vivre (à TRANCHER — ta tâche)
- **Process ICI** (`docs/claude/` + `docs/rust/PATTERNS.md`) : un pattern nommé + un **check gate-map** « primitive de frontière → contrat + gate ».
- **Process portable Factory** (`docs/agent/AGENT_SYSTEM.md` + content-model `docs/factory/knowledge/`) : la **généralisation** ; **S79 en est la 1ère instance** (anime+daisyUI).
- **Dogfood** : sur **S78** (qui produit une primitive **vivante** = l'orchestrateur shard live → cas-test idéal de la sonde ; `b3_shard_pipeline.sh` EST la sonde). **Garder l'ajout S78 LÉGER** (la règle + sa gate), pas un mini-sprint doc — ne pas diluer le P1 sharding.

## Questions ouvertes (la tâche concrète)
1. Formaliser la doctrine en **une page** — où exactement (PATTERNS ici + AGENT_SYSTEM Factory) ?
2. Définir le **check gate-map** précis : qu'est-ce qui FAIL ? (étiquette absente sur une primitive de frontière ? source-ref non résolu ? sonde absente sur une primitive vivante ?)
3. Décider : **dogfood léger sur S78** maintenant, ou attendre S79 ?
4. Arbitrer la cadence **étiquette-par-phase** vs la réalité des types qui *churnent* en cours de sprint (régénération = OK car généré).
5. **Vérifier dans le code actuel ce qui existe DÉJÀ** pour ne pas réinventer : drift-tests schemars (`test_schema_snapshot_matches_struct`, `shard_schema_*`), le `source-ref-check`/`check-sharding-docs.sh` de la Phase N S77, la provenance Ed25519/tree-walk, les commentaires `// Sprint …`.

## Garde-fous
- Hors-process / découverte ; **lecture seule** sur `.planning/` sauf instruction explicite.
- **Une AUTRE session peut tourner en parallèle** sur ce repo (worktrees `dazzling-cannon`/`agent-a97fde30`) → collision possible sur `.planning/active/`. Si tu écris du planning, vérifie l'emplacement réel après écriture.
- **Source-ancré** : chaque fait se vérifie dans le code ; ce prompt n'est pas une autorité.
