# Comment câbler une app app-authoring (anime.js + daisyUI)

*Guide pratique (Diátaxis). Pour le pourquoi, voir
[`EXPLANATION.md`](./EXPLANATION.md) ; pour les symboles exacts, voir
[`REFERENCE.md`](./REFERENCE.md).*

> **Statut : PROVISIONAL.** Lire la bannière d'honnêteté **avant** de câbler quoi
> que ce soit. **Caveat cardinal : lint statique ≠ garantie runtime ;
> connaissance consommée, jamais autoritaire** (**0 verdict PASS**).

## Ce qui existe vraiment (et ce qui n'existe pas)

- Le **gate CSP** (`run_gate_csp_authoring`), les **deux packs** (anime.js 4.5.0,
  daisyUI 5.5.23) et le **template daisyUI** sont livrés et testés
  hermétiquement.
- Le **parcours in-vivo de bout en bout** (auteur réel → publish → rendu chez un
  pair) et l'**efficacité générative** du prompt-kind / copilote sont
  `Not evidenced` (PROVISIONAL). La plomberie est câblée ; rien n'a été mesuré
  in-vivo.
- La connaissance **n'accorde aucune dispense** : le gate CSP est **non-délégable**
  (hors `--skip-gates`), et l'historique de chat n'est pas autoritaire.

## 1. Partir du template daisyUI

Le template `daisyui` (`crates/sbfb-factory/src/template_engine.rs`, entrée
`DAISYUI_TEMPLATE`) scaffolde une app SBFB + anime.js **vendorisé** + un `app.css`
daisyUI compilé, CSP-safe par défaut. C'est le point de départ recommandé : il
livre déjà la vendorisation et la recette de build correctes.

## 2. Vendoriser anime.js en UMD classique

Place `anime.umd.js` (4.5.0) sous `vendor/` et charge-le avec un `<script>`
**classique** exposant le global `window.anime`. **Jamais** `type=module`, jamais
de CDN, jamais d'import ESM : `connect-src 'none'` + l'origine opaque sous COEP
`require-corp` rendent tout import distant impossible. Rafraîchir le pack =
**ré-extraction manuelle au bump de version** (pas d'auto-fetch).

## 3. Compiler `app.css` build-time

daisyUI est compilé **build-time** avec la CLI Tailwind v4 :
`tailwindcss -i src/input.css -o app.css --minify`. Le `src/input.css` utilise
`@import "tailwindcss" source(none);` + `@source "./index.html"` / `@source
"./app.js"` (plus une safelist pour les classes construites au runtime) + `@plugin
"daisyui";`. Pas de `tailwind.config.js` en v4. Le résultat est **un seul `app.css`
same-origin**, zéro requête sortante. Le thème par défaut same-origin est
`sbfb-reflect` (oklch dark custom).

## 4. Respecter les pièges CSP de la fiche

La fiche [`../../prompts/agent/app-authoring.md`](../../prompts/agent/app-authoring.md)
énumère les **9 pièges CSP durs** (motion-path `cx=0`, box-shadow statique, SVG
`var(--color-*)`, morphTo mono-trace, `prefers-reduced-motion` → état-final,
`connect-src 'none'`, onScroll local-only, inertie sous reduced-motion, UMD
classic-script) et les verdicts per-classe daisyUI. Elle est **consommée et
affichée, jamais autoritaire** : elle montre la contrainte, elle ne lève aucun
gate.

## 5. Passer le gate CSP (non-délégable)

À la publication, `run_gate_csp_authoring` scanne le workspace en **trois tiers**
et **bloque** si un asset *authored* touche une primitive réseau, charge une URL
absolue hors `CSS_URL_ALLOW`, ou utilise `<script type=module>`. Ce gate
s'exécute **hors** du bloc `--skip-gates` : aucune dispense possible. Un échec
nomme l'asset et la directive violée — corrige l'asset, ne contourne pas le gate.

## 6. Vérifier au runtime (self-check viewer)

Le gate est statique : il ne voit pas le code assemblé à l'exécution (`fetch` via
`atob`, etc.). Le **self-check viewer** (Sprint 79 Phase H) rejoue l'app dans le
vrai iframe-host de prod sous la CSP réelle et capture les violations au niveau
navigateur. **lint statique ≠ garantie runtime** : utilise les deux.

## Récapitulatif du câblage

| Étape | Surface | État réel |
|---|---|---|
| Template | `DAISYUI_TEMPLATE` | livré (vendored UMD + `app.css` compilé) |
| Vendor | `<script>` classique `window.anime` | doctrine figée (jamais `type=module`) |
| Build | `tailwindcss --minify` → `app.css` | recette build-time, 0 dép runtime |
| Gate | `run_gate_csp_authoring` (3 tiers) | **non-délégable**, bloquant publish |
| Runtime | self-check viewer (Phase H) | filet runtime, pas preuve totale |

Les symboles, versions et tiers exacts sont dans [`REFERENCE.md`](./REFERENCE.md).
