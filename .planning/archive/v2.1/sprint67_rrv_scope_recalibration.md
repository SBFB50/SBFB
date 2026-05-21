# Sprint 67 - RRV Scope Recalibration

**Date** : 2026-05-21
**Statut** : decision de cadrage active pour S68/S69 kickoff.
**Question** : faut-il garder `@dev` dans l'Arc 2, ou valider le pilote
avec `@protocole` seulement ?

## Verdict

Gate 1 ne doit pas dependre de `@dev`.

Le chemin objectif pour S68/S69 est :

```text
S68 : Proof Cards @protocole + publish gate + UX confiance
S69 : Babel dogfood via Factory + pilote ferme 2-3 personnes
S70+ : @dev source index / source-only OSS seed, si Gate 1 est propre
```

`@dev` reste dans la vision produit, mais devient un enrichissement
post-pilote par defaut. Il peut entrer plus tot seulement comme stretch
strictement non bloquant.

## Faits repo

- Roadmap v4 avait deja D6 : `@protocole` avant `@dev` avant `@web`.
- Roadmap v4 classait deja `@dev index dans sbfb-factory` comme non
  obligatoire et deplacable, avec `@protocole suffit`.
- Sprint 67 livre des bases reelles pour `@protocole` : FTS5 daemon
  search, feed entries, provenance/browse metadata, `sbfb-manifest`, et
  `sbfb-factory` create/validate.
- Le code courant de `sbfb-factory` expose seulement `create` et
  `validate`. Il n'y a pas encore de commande `import`, pas de
  tree-sitter, pas d'index symboles, pas de source-only manifest.
- `deploy-from-repo` cible une app SBFB : repo public HTTPS, `SBFB.json`,
  `index.html`, archive zip, provenance. Un gros repo source OSS n'est
  pas publiable par ce chemin sans nouveau contrat.

## Implication S68

S68 doit privilegier les preuves que les testeurs verront :

- ProofCard calculee depuis les faits protocole disponibles.
- `proof_card_get` si l'UI/app en a besoin.
- Publish gate et provenance visibles.
- Recherche `@protocole` montrant Babel et les apps publiees.
- Wording de confiance factuel : preuve complete/incomplete, pas
  "trust score".

`@dev` tree-sitter, scan risques code, citations fichier/ligne/hash, et
source-only OSS seed sont hors chemin critique S68.

## Implication S69

Babel est un dogfood utilisateur. FlowUP cree Babel avec Factory et le
protocole ; Claude/Codex ne doivent pas traiter "coder Babel" comme un
livrable sprint classique. Les agents maintiennent l'infrastructure :

- templates et validation Factory ;
- publish path ;
- Proof Cards ;
- recherche/provenance ;
- bugs decouverts par dogfood ;
- support pilote ferme.

## OSS seed pour `@dev`

Ingerer de gros repos GitHub est possible comme vision, mais pas avec le
contrat courant :

- un repo source externe n'est pas une app SBFB ;
- `SBFB.json` v2 est un manifest d'application ;
- `deploy-from-repo` exige `index.html` et produit une archive app ;
- FTS5 actuel indexe des metadonnees protocole, pas des millions de
  fichiers/lignes ;
- les labels GitHub/discovery et SBFB/provenance doivent rester separes.

La bonne forme S70+ est un corpus curate de repos pertinents, borne par
taille/fichiers/langages, hash par commit/fichier, etiquete
`external OSS source index`, jamais `verified SBFB app`.

## Non-goals explicites S68/S69

- Pas de seed massif des "plus gros projets GitHub".
- Pas de confusion entre source externe indexee et app verifiee.
- Pas de SearchManifest network avant son sprint.
- Pas de tree-sitter comme critere de Gate 1.
- Pas de Babel code ownership par agent si FlowUP dogfood l'app.
