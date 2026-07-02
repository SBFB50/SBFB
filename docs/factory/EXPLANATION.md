# Comment marche l'app-authoring SBFB

*Document d'explication (Diátaxis). Pour le mode d'emploi, voir
[`HOW_TO_WIRE.md`](./HOW_TO_WIRE.md) ; pour les symboles exacts, voir
[`REFERENCE.md`](./REFERENCE.md).*

> **Statut : PROVISIONAL** pour le parcours in-vivo de bout en bout (cf.
> [`README.md`](./README.md)). **Caveat cardinal : lint statique ≠ garantie
> runtime ; connaissance consommée, jamais autoritaire** (**0 verdict PASS**).

---

## Le problème

Une app SBFB s'exécute dans un **environnement hostile à l'auteur** : une iframe
`sandbox="allow-scripts"` **sans** `allow-same-origin` (origine opaque/null), sous
une CSP qui coupe **tout** le réseau. Pas de `fetch`, pas de CDN, pas de Worker,
pas de `crypto.subtle`, pas de Google Fonts. Une app qui « marche en local » avec
un import ESM ou un asset distant **ne se charge pas** une fois publiée. La
capacité app-authoring existe pour que l'agent qui écrit l'app connaisse ces
contraintes **avant** d'écrire une ligne, et pour qu'un gate les **vérifie** avant
publication.

## Pourquoi une iframe scellée

L'isolation est une **défense en profondeur**. `allow-scripts` sans
`allow-same-origin` donne une **origine opaque** même dans un onglet de premier
niveau : pas d'accès au `localStorage` / cookies de l'origine du daemon, pas de
scope Service Worker. La CSP ajoute `connect-src 'none'` (aucun fetch),
`worker-src 'none'`, `frame-src 'none'`, `object-src 'none'`, plus `base-uri
'none'` et `form-action 'none'` qui bouchent deux vecteurs d'exfiltration que
`connect-src` ne couvre pas (une `<form action>` est une **navigation**, pas une
connexion ; un `<base href>` détourne les URL relatives). Cette politique a **une
seule source de vérité** : la constante `BLOB_SERVE_CSP` de
[`../../crates/nexus-core-rs/src/csp.rs`](../../crates/nexus-core-rs/src/csp.rs),
injectée par le daemon sur **chaque** réponse (y compris 404).

## Pourquoi un gate déterministe

Le gate CSP (`run_gate_csp_authoring`) est un **scan statique déterministe** —
regex sur les assets, **aucun composant ML, aucun scoring opaque**. Il **importe**
`BLOB_SERVE_CSP` (jamais de re-hardcode, jamais de lecture d'un commentaire
périmé) et dérive les directives à `'none'` via `none_directives`, de sorte qu'un
ajout futur de directive `'none'` à la politique casse un test de couverture
tant qu'une règle de détection n'est pas ajoutée (anti-drift). Il scanne en
**trois tiers** : source *authored* (0 réseau + 0 URL absolue hors allowlist + pas
de `<script type=module>`), *compiled* (`app.css` : 0 réseau + URL absolues ∈
`CSS_URL_ALLOW`), *vendored* (`vendor/*.umd.js` : 0 primitive réseau live).

Surtout, le gate est **non-délégable** : il s'exécute **hors** du bloc
`--skip-gates` (Day-0 « scellage 100 % Factory »). La connaissance accorde
**aucune** dispense CSP. Un lint authoring est **additif** : il ne relâche jamais
FG5/FG6/FG8/COEP/COOP/Ed25519.

## Pourquoi « consommée, jamais autoritaire »

Les packs et la fiche `app-authoring` sont de l'**aide à l'écriture** : ils
*montrent* la contrainte sandbox, ils ne l'**accordent** pas. La capacité
préserve l'invariant Factory cardinal : **0 verdict PASS** émis par la
connaissance, brouillon d'artefact anti-PASS préservé, et le context-pack porte
`chat_history_authoritative = false`. Quand un doute subsiste, **le pack et les
gates tranchent**, pas l'historique de conversation. C'est ce qui empêche une
capacité « d'aide » de devenir une autorité de publication déguisée.

## Le filet runtime — lint statique ≠ garantie runtime

Un lint déterministe ne voit **pas** le code assemblé à l'exécution (`fetch` via
`atob`, `form.action` / `base.href` / `img.src` construits dynamiquement, un
`//host` protocole-relatif isolé dans une string JS). Ces cas sont rattrapés par
(a) la **CSP runtime** chez chaque client et (b) le **self-check viewer** (Sprint
79 Phase H), qui rejoue l'app dans le **vrai iframe-host de prod** sous la CSP
**réellement servie** et observe les violations au **niveau navigateur**. Le gate
garantit la conformité des assets **livrés** + le feedback auteur, **pas**
l'absence d'exfiltration à l'exécution. **Filet, pas preuve totale.** Détail :
[`FACTORY_GATES.md`](./FACTORY_GATES.md) (FG-CSP-authoring) et les patterns Rust
[`../rust/PATTERNS.md`](../rust/PATTERNS.md) §P71.

## Pourquoi vendoriser en UMD classique

anime.js est **vendorisé** sous `vendor/anime.umd.js` et chargé par un `<script>`
**classique** (jamais `type=module`). Raison : `connect-src 'none'` rend tout
import distant impossible, et une **origine opaque sous COEP `require-corp`** ne
peut pas satisfaire le CORS d'un module ESM. daisyUI/Tailwind sont compilés
**build-time** en un seul `app.css` same-origin — l'archive runtime n'embarque
**aucune** dépendance et ne fait **aucune** requête sortante.

## L'API de pilotage de l'Operator — un sous-domaine distinct

Depuis le Sprint 80, le poste de pilotage (front greenfield
`tools/factory-operator/`) lit cinq routes loopback du serveur Rust :
amorçage cookie, diff d'arbre de travail calculé côté Rust, registre de
gates restitué 1:1, flux de conversation, et inventaire des documents du
projet (arrivé par l'arc off-sprint `94eb030`, indexé à l'audit gate S80).
Ce sont des **frontières au
sens du test-acteur** (un runtime distinct les lit), mais elles vivent
**hors** de l'iframe scellée et de sa politique `BLOB_SERVE_CSP` — les
mélanger au contrat de scellage serait une erreur de catégorie. Leur
référence contractuelle vit dans
[`REFERENCE.md` §Operator control-plane API](./REFERENCE.md) ; les
mitigations de cette surface restent dans le
[modèle de menace](../security/THREAT_MODEL.md), source unique.
