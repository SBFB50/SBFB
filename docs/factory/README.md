# Factory app-authoring — créer une app SBFB animée (anime.js + daisyUI) en iframe scellée

La capacité **app-authoring** outille le process de fabrication d'apps SBFB pour
maîtriser **anime.js** (animation) et **daisyUI** (composants, sur Tailwind v4)
**à l'intérieur de l'iframe scellée** du réseau — `sandbox="allow-scripts"` sans
`allow-same-origin`, sous une CSP qui interdit tout réseau. Elle se compose d'un
**gate CSP déterministe**, de deux **packs de connaissance versionnés**, d'un
**prompt-kind portable** et d'un **template de démarrage** daisyUI.

Ce dossier est le **hub de documentation** de la capacité. Il suit le modèle
[Diátaxis](https://diataxis.fr/) : quatre types de documents pour quatre besoins
distincts.

---

## Statut : PROVISIONAL là où ça compte

Le **gate CSP** (`run_gate_csp_authoring`), les **deux packs** (anime.js 4.5.0,
daisyUI 5.5.23) et le **template daisyUI** sont **livrés et testés
hermétiquement** (Sprint 79, phases A→H). Ce qui reste **PROVISIONAL /
`Not evidenced`** : le **parcours in-vivo de bout en bout** (un auteur réel qui
crée → publie → voit son app rendue chez un pair) et l'**efficacité générative**
du prompt-kind / du copilote Ollama. La plomberie est câblée et testée ; aucune
app réellement écrite par un LLM n'a été mesurée in-vivo. Aucune phrase de cette
doc ne doit prétendre « shipped » / « en production » de ce qui est seulement
statique et local.

### Caveat cardinal

> **Caveat cardinal — lint statique ≠ garantie runtime ; connaissance consommée,
> jamais autoritaire.** Le gate CSP est un **scan statique déterministe** des
> assets *livrés* (faux-négatifs assumés : `fetch` via `atob`, `form.action` /
> `base.href` construits au runtime) ; le filet runtime est le **self-check
> viewer** (Sprint 79 Phase H), qui rejoue l'app sous la CSP réelle. La
> connaissance (packs, prompt-kind, fiche) est **consommée et affichée, jamais
> autoritaire** : **0 verdict PASS**, l'historique de chat n'est pas autoritaire.
> Le code (`nexus_core_rs::csp`) et les gates déterministes décident.

Détail des surfaces sandbox et du partage statique/runtime : voir le modèle de
menace, [`../security/THREAT_MODEL.md`](../security/THREAT_MODEL.md). Cette doc
**renvoie** au modèle de menace, elle ne le duplique pas.

---

## Les quatre documents

| Type Diátaxis | Document | Pour | Audience |
|---|---|---|---|
| **Explication** | [`EXPLANATION.md`](./EXPLANATION.md) | comprendre *pourquoi* l'iframe scellée, le gate déterministe, le consommé-jamais-autoritaire | humain |
| **Guide pratique** | [`HOW_TO_WIRE.md`](./HOW_TO_WIRE.md) | *comment câbler* une app sous le contrat CSP (vendoriser l'UMD, compiler `app.css`, passer le gate) | humain |
| **Référence** | [`REFERENCE.md`](./REFERENCE.md) | les symboles, packs, versions et tiers exacts (jumeau humain des sources rank-1) | humain + agent |
| **Tutoriel** | _(différé)_ | un walkthrough end-to-end runnable | — |

> **Pourquoi pas de tutoriel ?** Un tutoriel promet un parcours qui *marche du
> premier coup*. Tant que le parcours in-vivo de bout en bout est `Not evidenced`
> (statut PROVISIONAL ci-dessus), un walkthrough sur-promettrait. À la place, la
> preuve exécutable est [`examples/csp_contract.rs`](./examples/csp_contract.rs)
> (compilé+exécuté par le test d'intégration) et le gate lui-même.

---

## Contrat machine-lisible

La **source machine-actionnable** pour un agent est
[`WIRING_SPEC.md`](./WIRING_SPEC.md) (contrat source-ancré : chaque clause cite un
fichier rank-1 `path:Symbol`), indexée par [`llms.txt`](./llms.txt). Le détail des
gates de publication vit dans [`FACTORY_GATES.md`](./FACTORY_GATES.md). La **vérité
CSP unique** est la constante `BLOB_SERVE_CSP` de
[`../../crates/nexus-core-rs/src/csp.rs`](../../crates/nexus-core-rs/src/csp.rs) ;
en cas de divergence, **le code Rust fait foi**.
