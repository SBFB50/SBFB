**Verdict ROUND 6 — Phase H S83 : CONFIRMÉ-FERMÉ.**

- **Test A** apporte une preuve différentielle complète : seul `parent.allowed_handoffs` change. Le cas autorisé compile, le cas interdit exige précisément `UnauthorizedHandoff` sur `hand`. Cela confirme que `agent.as_tool` n’est pas le donneur et que la recherche remonte jusqu’à l’`agent.invoke` orchestrateur.
- **Test B** isole bien `Subworkflow` comme effet aval : le `capability.call` est remplacé par un `transform`, les capability requests sont supprimées et le child est fourni. `expect_error` n’accepte que `ClassificationFromModel` sur `sw`, donc aucun rejet parasite ne peut rendre le test vert.
- Les résultats annoncés — crate **106/106**, workspace Windows **2251**, Docker **2255 + 1 skip** — couvrent l’absence de régression.

Le motif du `PARTIEL` de R5, à savoir les assertions tronquées, est entièrement levé. Aucun GAP résiduel sur ces deux preuves. Résultats de suites acceptés comme fournis, sans relance indépendante.