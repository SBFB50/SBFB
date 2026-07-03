# Sprint 81 Phase B — Review (Workflow ultracode + agent de synthèse)

> Phase B (bump iroh 0.98 → =1.0.1 bi-axe, `sprint81_plan.md:124-145`,
> supersedée par le préflight PLAN-ADAPT `sprint81_phase_b_preflight.md §4`
> 9 points + §7 carries) : `chore(deps)` — pins EXACTS iroh =1.0.1 /
> iroh-docs =0.101.0 / iroh-gossip =0.101.0 / iroh-blobs =0.103.0 +
> rust-version 1.91 + fix pkarr DOUBLE + re-datages + absorption MSRV
> collapsible_if. Arbre SALE, HEAD `fdb8ad7`, diff NON committé (58 fichiers
> modifiés + 2 untracked planning, re-vérifié `git status --porcelain` = 60
> lignes). 8 dimensions de review + 8 vérifications adversariales (toutes
> CONFIRMED — 0 REFUTED, 0 ADJUSTED survivant au niveau synthèse).

## Verdict: PASS

> **Diff Phase B substantivement CONFORME au contrat préflight qui supersede
> la lettre.** 0 P0 / 0 P1 sur les 8 dimensions. Le bump est point-unique
> (Cargo.toml 4 pins exacts + rust-version 1.91 + commentaire re-daté
> 2026-07-03, Cargo.lock figé, 0 crate SBFB touché au manifest — D7 tenu),
> le fix pkarr est le port VERBATIM §4.3 (3e arg `DnsResolver::new()` +
> rename `CaTlsConfig`, 0 occurrence `custom_server_cert_verifier`/
> `insecure_skip_verify`, posture TLS EmbeddedWebPki inchangée sur pièces
> vendored), les 6 re-datages contractés §4.4 sont faits avec les réfs
> upstream exactes, `node.rs`/`dispatch_loop.rs`/`canonical.rs` sont
> INTOUCHÉS (0-bump wire prouvé par grep du diff entier : 0 hit
> `DOMAIN_*_V1`/`FEED_FORMAT_VERSION`/`*_FORMAT_VERSION`/ALPN), et les
> ~139 hunks d'absorption MSRV (rust-version 1.91 ≥ 1.88 active
> `collapsible_if` sous `-D warnings`) ont été vérifiés UN PAR UN
> sémantique-préservants (if imbriqués sans else → chaînes `&&`, ordre
> d'évaluation, court-circuits et side-effects identiques — y compris sur
> les chemins sécurité auth/consent/canary/fork/zip-slip/anti-rollback).
> Baseline verte sous le nouveau lock : nextest Win 2028/2028 0-skip
> (delta 0 net tenu), 4 tests A2 PASS, CONTROL A4 PASS toujours
> NON-convergent (tripwire PAS flippé) + GREEN A4 PASS — re-joués de
> première main en D5.
>
> **PASS-PENDING = review OK, Codex PAS ENCORE JOUÉ** (gate bloquante
> review → Codex → commit). En sus de Codex, **4 P2 consolidés**, aucun ne
> bloquant le commit SI le body les honore :
> **(1)** snapshot Mac PENDING (SSH timeout reproduit empiriquement) vs
> contrat §4.2(a) « Win + Mac au moment du commit B » → règle BLOQUANTE
> nommée au body + pré-condition Phase F/H. **(2)** doc-comment A2 des
> tests `runtime.rs:4209-4212` resté daté « iroh-docs 0.98 » alors que
> l'ancre sémantique « Replica not found » du re-grep §4.4 le matche →
> fix 1 ligne comment-only avant commit (ou exception consignée).
> **(3)** la liste des 139 absorptions MSRV ne vit qu'en scratchpad
> session-éphémère → le §4.6 la veut AU BODY ; **si le body ne la porte
> pas, ce P2 DEVIENT P1**. **(4)** delta sémantique DNS du client pkarr
> (config gelée à la construction + fallback Google warn-only + pas de
> `reset()`) induit par le port verbatim → à NOMMER au body + note
> THREAT_MODEL routée Phase G.
>
> Les 14 P3 sont documentables au body (précision de qualification des
> lints, ancre env-classe Docker exacte, paire d'oracles CONTROL/GREEN,
> repères périmés, inventaire re-datage routé C/D/E/G, N-A-no-new-frontier,
> §P73 candidate). Séquence : honorer (1)-(4) au body → Codex →
> réconciliation → promotion PASS → commit `chore(deps)`.

## Portée du diff (constats croisés des 8 dimensions)

`git status --porcelain` à HEAD `fdb8ad7` = **58 fichiers modifiés + 2
untracked**, 0 fichier parasite (partition prouvée D2/D4/D7 : 55 `.rs` =
51 fichiers de la liste collapse ∪ exactement les 4 livrables nommés
{pkarr_resolver.rs, docs.rs, discovery.rs, blobs.rs} ; + Cargo.toml,
Cargo.lock, kickoff [D] ; untracked = préflight + cargo_tree_d) :

- `Cargo.toml` — 4 pins exacts `=1.0.1`/`=0.101.0`/`=0.101.0`/`=0.103.0`,
  commentaire re-daté « crates.io API 2026-07-03 » + mention
  `redb-v2-migration` + « do NOT disable default features »,
  `rust-version = "1.91"` (§4.1 verbatim).
- `Cargo.lock` — figé, deltas conformes §4.5 (dalek dual-tree 2.2.0 +
  3.0.0-rc.0, redb ×2 by-design 3.1.3+4.1.0 [2.6.3 disparu], iroh-util
  0.6.0 neuf, noq 1.0.1, irpc 0.17.0, iroh-metrics 1.0.1, quinn 0.11.9
  inchangé) + transitifs additionnels hors liste nominale (8 in / 3 out,
  cf. P3-8) tous dans la fermeture iroh/n0/dalek — D7 tenu (0 manifest
  de crate SBFB au diff).
- `crates/nexus-core-rs/src/pkarr_resolver.rs` — UNIQUE correction de
  code : imports `:40-41` (`CaTlsConfig` + `DnsResolver`),
  `CaTlsConfig::default().client_config(default_provider())` `:112`,
  3e arg `DnsResolver::new()` `:119`, doc-comments rafraîchis sans
  surestimation (« same trust posture as the 0.98 EmbeddedWebPki
  variant » = vrai sur sources vendored iroh-relay 0.98/1.0.1).
- Re-datages comment-only §4.4 : `docs.rs:53` (0.101 Clone), `:151-158`
  (A2, réfs upstream store.rs:24-27/api.rs:262-265), `:385-404`
  (start_sync recalibré 0.101 : live.rs:408-414/:713, state.rs:97, cap 5
  store.rs:17 — promesse « Recalibrate at the bump » remplacée par le
  constat) ; `discovery.rs:4`/`:117` (1.0.1) ; `blobs.rs:87` (0.103) ;
  `runtime.rs:2522-2538` (site prod A2 re-daté 0.101).
- **~139 hunks d'absorption MSRV** sur 51 fichiers (cause : rust-version
  1.91 ≥ 1.88 let-chains → clippy `collapsible_if` MSRV-gated s'active
  sous `-D warnings`) : ~137 collapses if-imbriqués→let-chains + 2
  `manual_is_multiple_of` (pii_redactor.rs:86, llm/shard.rs:170 — même
  cause racine, lint frère) + 1 entrée de liste qui est en réalité le
  re-datage runtime.rs:2525. Tous vérifiés 1:1 sémantique-préservants
  (D1 hunk par hunk + D6 chemins sécurité + D3 code de test).
- `.planning/active/sprint81_kickoff.md` — question [D] requalifiée
  « sur COPIE » (`:494-497`), seul hunk.
- `?? sprint81_phase_b_preflight.md` (346 lignes, PLAN-ADAPT en tête) +
  `?? sprint81_phase_b_cargo_tree_d.txt` (capture réelle, dalek ×2 +
  redb ×2 visibles).

**INTOUCHÉS prouvés** (par absence du diff, pas par déclaration) :
`node.rs` (SEED_ALPN:68/SHARD_ALPN:80 garantis — le repère plan/préflight
« node.rs:24 » est périmé, grep = 0 commentaire de version dans le
fichier), `dispatch_loop.rs` (CONTROL :557 + GREEN :643 byte-intacts),
`canonical.rs`, `csp.rs`, `gossip.rs` (checkpoint §4.8 : pur recompile,
pow_gossip.rs = 2 collapses MSRV sans rapport), siblings sync-set
`runtime.rs:2552-2564`/`:2647-2659` (carry → Phase C, dernier hunk
runtime.rs finit ~:2538), les 4 tests A2 (`runtime.rs:4173-4319`) et les
artefacts T2 committés, `web/` et `tools/` (0 fichier).

## Restitution par dimension (8 + vérifications adversariales)

### Évaluation D1 — lecture intégrale du diff hunk par hunk

**PASS.** Diff lu EN ENTIER (58 modifiés + 2 untracked + Cargo.lock).
Conformité préflight totale sur les points bloquants : §4.1 exact, §4.3
verbatim (0 pattern TLS interdit), re-datages cohérents avec les repères
upstream §6, node.rs/dispatch_loop.rs/siblings INTOUCHÉS. Les ~137
collapses vérifiés un par un : STRICTEMENT sémantique-préservants (cas
sensibles levés sur fichier réel : http.rs:1614 else-if attaché au if
EXTERNE :1613 ; llm_bridge.rs else-if sans else aval ;
`seen.insert`/`current_hunk.take`/`obj.remove` court-circuités à
l'identique ; branche text_delta correctement NON-collapsée). fmt re-joué
VERT. Aucun hunk non-déclaré. 3 P3 : 2 hunks sont du lint
`manual_is_multiple_of` (pas `collapsible_if`) — qualification à corriger
au body ; l'entrée de liste runtime.rs:2525 est un re-datage, pas un
collapse ; snapshot Mac PENDING (consolidé P2-1).

### Évaluation D2 — vérification des 3 blocs §7.4 + honnêteté des claims

**PASS** (PASS-PENDING au niveau global, Codex non joué). Tous les gates
rapides re-joués verts ; 100 % des claims vérifiables sur pièces ont
tenu : fmt Win CLEAN, git status = 0 fichier surprise, nextest list =
2028 == baseline fdb8ad7, balance `#[test]` au diff 0/0, arithmétique
Docker 2030+2=2032=2028+4 `cfg(unix)` cohérente, cargo_tree_d.txt réel,
snapshots Win daemon+worker valides au `tar -t`, garde S3a-2 pkarr tenue.
Classification des 2 fails Docker e2e SOLIDE : mécanisme exact PRÉ-bump
= TEST-ISOLATION-SBFB-HOME (`sprint78_audit_plan.md:129-138`, collision
singleton `/root/.sbfb` en concurrence conteneur) + dossier historique
S52/S57/S77 sous 0.98 — aucune surface iroh, les 2 tests verts Win natif.
Restent déclarés-seulement (cohérents baselines S80/A4, non contredits) :
clippy Win/Docker, doctests, release, web 411/coverage/size/scan,
operator 201/gates/E2E. 1 P2 (Mac, consolidé P2-1) + 2 P3 (ancre
env-classe exacte à citer au body ; CONTROL = oracle unilatéral, à
statuer en PAIRE avec le GREEN). Adversarial D2-1 : CONFIRMED (SSH Mac
re-tenté de première main → « Connection timed out », gap non fermable
séance tenante ; sévérité P2 juste — déclenchement = déploiement manuel
délibéré, aucun chemin automatique).

### Évaluation D3 — branch coverage sémantique

**PASS.** Le seul changement de comportement runtime (pkarr_resolver.rs)
est couvert par les 5 tests existants sur son chemin de construction
(chaîne complète `CaTlsConfig::default().client_config(...)` +
`PkarrRelayClient::new(url, tls, DnsResolver::new())`, construction hors
runtime tokio sûre par source vendored hickory). Delta tests 0 net prouvé
indépendamment (nextest list 2028 + ZÉRO ligne `fn `/attribut changée sur
les 183 hunks). Les 8 collapses en code de test sont
sémantique-préservants (asserts byte-identiques, timeouts inchangés).
Tests A2 byte-identiques à fdb8ad7 ; tests A4 dans un fichier à diff
VIDE. 1 P2 (delta DNS pkarr, consolidé P2-4, adversarial CONFIRMED sur
sources vendored 0.98 vs 1.0.1 : getaddrinfo per-lookup → config gelée
à la construction + fallback Google 8.8.8.8 warn-only + `reset()` jamais
câblé côté SBFB ; mitigants : feature opt-in `SBFB_PKARR_RELAYS` vide par
défaut, dégradation = échec lookup jamais fausse donnée, paquets pkarr
signés) + 1 P3 (chemin de panic théorique NOUVEAU via l'`expect` interne
de `DnsResolver::new()`, pratiquement inatteignable, aucun fix SBFB
possible sans dépasser le verbatim §4.3).

### Évaluation D4 — scope cuts sémantiques + invariants sprint

**PASS.** Les 8 invariants tenus au diff réel : (a) iroh STRICTEMENT SEUL
(D7) ; (b) 0 feature iroh ajoutée ; (c) aucun rust-toolchain.toml ;
(d) 0 bump wire (grep -U0 = 0 hit, canonical.rs/node.rs HORS diff,
FEED_FORMAT_VERSION:20 hors hunks) ; (e) 3 sites sync-set INTACTS
(bisectabilité préservée) ; (f) cartographie mécanique complète — 152
hunks -U0 dans crates/ = 139 sites scratchpad + 13 hunks attendus
hors-liste (pkarr ×5, docs ×5, discovery ×2, blobs ×1), comptes par
fichier égaux partout, AUCUN hunk hors zone répertoriée ; (g) artefacts
T2 tracked HORS diff ; (h) web/ et tools/ = 0 fichier. 2 P2 (consolidés
P2-2 doc-comment A2 :4211 [adversarial CONFIRMED : l'ancre « Replica not
found » du re-grep §4.4 matche runtime.rs:4209, les jumeaux prod SONT
re-datés — incohérence interne réelle avec le futur claim « carry A2
FERMÉ »] + P2-1 Mac [adversarial CONFIRMED]) + 3 P3 (node.rs:24 périmé
à documenter ; mentions stale hors scope B à router C/E/G ; deltas lock
au-delà de la liste nominale §4.5 — transitifs purs, à compléter au
registre du body).

### Évaluation D5 — research grounding (conformité au contrat préflight)

**PASS.** Les 9 points §4 et les carries §7 vérifiés pièce par pièce
contre le diff réel : §4.1/§4.3/§4.4/§4.5/§4.8 HONORÉS (détail en
Portée) ; §4.6 : absorption mécanique 1:1 prouvée (couverture 100 % des
fichiers, spot-checks sur TOUS les édits manuels + fichiers
iroh-adjacents, 0 changement de signature/constante wire au diff entier) ;
§4.7 RE-VÉRIFIÉ DE PREMIÈRE MAIN : 4/4 tests A2 PASS + CONTROL A4 PASS
toujours NON-convergent (tripwire PAS flippé) + GREEN A4 PASS sous le
nouveau lock ; §4.2 PARTIEL (Win fait, Mac PENDING — consolidé P2-1,
adversarial CONFIRMED avec re-test SSH empirique) ; kickoff [D]
requalifié. Carries §7 : A2 fermable par constat (0 hit `contains(` au
diff, matchers :2538/:2633 intacts, 4 tests verts) ; A4 sibling PAS fixé
(bien Phase C) ; CONTROL intact. 3 P3 : liste des 139 sites en scratchpad
seul (consolidé P2-3) ; mentions stale dont doc_sync.rs:13 citant des
lignes upstream périmées de la doctrine sync-set (router C) ; repère
préflight « 4 tests A2 :4212-4291 » n'en couvre que 3, le 4e
(`boot_feed_namespace_fail_fast_on_docs_error`) est à :4319 — vérifié
vert.

### Évaluation D6 — sécurité deep

**PASS** (0 P0, 0 P1, 3 P3). (a) Posture TLS pkarr EXACTEMENT préservée
(CaTlsConfig::default() = Mode::EmbeddedWebPki identique 0.98↔1.0.1 sur
sources vendored ; scan diff entier = 0 occurrence insecure/skip_verify/
custom_server_cert ; la confiance pkarr ne repose pas sur le DNS :
Ed25519 + TLS WebPKI + quorum 2/3). (b) Fenêtre one-way redb : tar Win
daemon BYTE-IDENTIQUE à l'arbre réel (comm bidirectionnel = 0 écart,
couvre LES DEUX stores redb) + tar worker complet ; AUCUNE migration déjà
déclenchée (0 `.backup-redb-v2-tuples`, mtimes < snapshot) ; Mac PENDING
= trou ACCEPTABLE (b3_live_pc_vps.sh ne touche que le VPS, aucun
scp/rsync/build vers le Mac — claim « aucun binaire 0.101 sans
déploiement manuel » VRAI sur pièces). (c) Sweep collapsible_if sur
chemins sécurité : CHAQUE collapse re-vérifié individuellement (auth
constant_time_eq même ordre, consent caps, canary anti-rollback, fork
zip-slip Err(UnsafePath) même condition, operator handle_bootstrap
mauvais-token → index neutre INCHANGÉ, events-core write_event toujours
émis sous les mêmes gardes) ; guardrails.rs et node.rs INTOUCHÉS.
(d) R-iroh-audit P0 non contredit (0 ligne ajoutée mentionnant
audit/Gate/pilote). Bonus : rustls-platform-verifier SORT de l'arbre
(neutre-à-positif). 3 P3 : obligations body PENDING-commit (D8 verbatim,
règle boot, registre) ; delta lock transitif plus large que « iroh-util
seule vraie nouvelle entrée » ; store VPS = aussi un store réel dans la
fenêtre one-way → étendre la règle de boot « Win, Mac ET VPS ».

### Évaluation D6bis — test-acteur docs-contract (§6.12)

**PASS — N-A-no-new-frontier CONFIRMÉ contre le diff réel.** 0 route
HTTP ajoutée/modifiée (grep patterns routing = 0 hit), 0 payload/enveloppe
JSON changé (0 ligne derive/serde ; json! ré-indentés à contenu
identique ; réponses 303/400/200 byte-identiques y compris SET_COOKIE
handle_bootstrap et fall-through index neutre), frontières EXISTANTES
lues par runtimes distincts byte-stables (blob-serve apps.rs, /api/gates
gates.rs, process.rs LintDiagnostic, sprint_history.rs, deploy.rs
provenance, llm_bridge.rs SSE). Aucun prompt-kind/knowledge pack/
docs/factory//llms.txt/WIRING_SPEC touché. La conclusion préflight §10
est VALIDE. 2 P3 : consigner l'étiquette explicite N-A-no-new-frontier
au body ; documenter le repère §10 « node.rs touché au commentaire :24 »
périmé (node.rs INTOUCHÉ = conformité a fortiori).

### Évaluation D7 — livrables + conventions/patterns

**PASS.** Tous les livrables de la lettre livrés ou requalifiés avec
preuve conforme au préflight qui supersede ; artefacts planning complets
(préflight 346 lignes PLAN-ADAPT en tête + cargo_tree_d réel) ;
partition parfaite du sweep ; baseline (d) structurellement tenue ;
conventions code EN / planning FR / 0 emoji / fmt VERT live. 3 P2
(consolidés : P2-2 doc-comment :4211 [adversarial CONFIRMED], P2-3 liste
au body [adversarial CONFIRMED — aggravant : la checklist body §4.9
prédate la découverte du lint, seule §4.6 porte l'obligation → un commit
composé depuis §4.9 seul absoberait silencieusement 139 sites], P2-1 Mac
[adversarial CONFIRMED]) + 3 P3 (7 mentions stale hors livrables à router
C/D/E ; commit shape `chore(deps)` sans précédent MAIS fixé par préflight
§11, l'ancien `feat(sprint32)` = convention abandonnée, aucune action ;
aucun §P PATTERNS.md ne couvre « bump rust-version → lints clippy
MSRV-gated s'activent » → §P73 candidate).

## Findings consolidés (0 P0, 0 P1, 4 P2, 14 P3) + arbitrages adversariaux

Aucun P0/P1 sur les 8 dimensions. Les 8 vérifications adversariales
jouées ont TOUTES rendu CONFIRMED (D2-1, D3-1, D4-P2-1, D4-P2-2, D5-1,
D7-1, D7-2, D7-3) — aucun finding réfuté à écarter, aucun ajusté à
substituer au niveau synthèse (les REFUTED/ADJUSTED amont, p.ex. S4-1 du
préflight, avaient déjà été absorbés avant cette review). La
consolidation dé-duplique les recouvrements inter-dimensions.

### P2-1 — Snapshot Mac PENDING vs contrat §4.2(a) « Win + Mac au moment du commit B » (D1-3/D2-1/D4-P2-2/D5-1/D7-3, adversarial CONFIRMED ×4)
- **Constat** : `C:\Users\FlowUP\sbfb-snapshots\s81-phase-b\` contient
  UNIQUEMENT les 2 tars Win (daemon 1 067 776 o, 76 entrées,
  `shell-daemon/iroh/docs.redb` + `iroh/blobs/` + `node_key` +
  `local-worker/data/docs.redb` ; worker 40 018 o, `data/docs.redb` +
  `blobs/` + `worker.key` — contenus vérifiés `tar -tzf`), datés
  03/07 12:23. AUCUNE archive Mac. SSH Mac re-tenté de première main
  par l'adversarial → « Connection timed out » : le snapshot est
  matériellement imprenable séance tenante.
- **Calibration P2 (pas P1)** : l'échec exigerait DEUX violations
  manuelles d'une règle documentée (déployer un binaire 0.101 sur le
  Mac PUIS booter sur store réel) — aucun chemin automatique
  (`b3_live_pc_vps.sh:236` ne SSH que le VPS, b3_shard_pipeline ne
  pousse aucun binaire). Pas P3 : terme explicite du garde opérationnel
  protégeant une surface de perte de données réelle (migration redb 2→4
  one-way, atomicité crash-mid-migration NON vérifiée avant F).
- **Disposition** : au body, règle BLOQUANTE NOMMÉE (pas une prose) :
  « aucun binaire 0.101 déployé ou booté sur le Mac avant snapshot
  Mac » + snapshot Mac en pré-condition explicite de la checklist
  Phase F/H (re-tenter SSH chaque session, au plus tard avant F). Le
  commit shape §11 prévoit de claimer « snapshot D3 cond.3 avancé » —
  SANS le qualificatif Mac PENDING ce serait un sur-claim.

### P2-2 — Doc-comment A2 des tests `runtime.rs:4209-4212` non re-daté (D4-P2-1/D7-1, adversarial CONFIRMED ×2)
- **Constat** : le bloc de tête des 4 tests A2 dit encore « In iroh-docs
  0.98 open_doc never returns Ok(None) », hors de tout hunk du diff,
  alors que l'ancre sémantique « Replica not found » du re-grep ordonné
  par §4.4 le matche (runtime.rs:4209 parmi 8 hits) et que les jumeaux
  prod (runtime.rs:2522-2538) et docs.rs:151-158 SONT re-datés 0.101.
  Incohérence interne avec le futur claim body « carry A2 FERMÉ par
  constat ». Zéro impact sémantique (constat identique sous 0.101,
  préflight §6.5) ; re-dater le commentaire de tête ne viole pas la
  clause « ne toucher ni les 4 tests A2 » (c'est du comment-only, pas
  du code de test).
- **Disposition** : fix UNE LIGNE avant commit (même formule que
  :2526-2529, p.ex. « In iroh-docs 0.101 (unchanged from 0.98) ») — ou
  à défaut exception explicitement consignée au body.

### P2-3 — Liste des 139 absorptions MSRV : le §4.6 la veut AU BODY, elle ne vit qu'en scratchpad éphémère (D5-2/D7-2, adversarial CONFIRMED — **escalade P1 si le body ne la porte pas**)
- **Constat** : préflight §4.6 verbatim « absorber en B UNIQUEMENT si
  mécanique 1:1 ... avec liste explicite fichier:ligne au body ...
  jamais d'absorption silencieuse ». La liste existe
  (`s81b_collapse_sites.txt`, 139 entrées, 51 fichiers, partition
  croisée conforme au diff bidirectionnellement) mais le scratchpad est
  session-local et non repo-visible. Aggravant relevé par
  l'adversarial : la checklist body §4.9 prédate la découverte du lint
  — un commit composé depuis §4.9 seul produirait une absorption
  silencieuse de 139 sites, invérifiable par l'audit gate S81.
- **Disposition** : au commit, matérialiser la liste dans le repo —
  intégralement au body OU comme artefact committé
  (`.planning/active/sprint81_phase_b_collapse_sites.txt`) référencé
  par le body, avec cause (« rust-version 1.85→1.91 ≥ 1.88 active
  collapsible_if MSRV-gated sous -D warnings ») + nature du transform
  (if imbriqués sans else → let-chains ; machine-applicable + 14
  manuels) + la qualification CORRIGÉE : DEUX lints (collapsible_if
  ~137 + manual_is_multiple_of 2 : pii_redactor.rs:86,
  llm/shard.rs:170), et 1 entrée de liste = re-datage (runtime.rs:2525).

### P2-4 — Delta sémantique DNS du client pkarr non déclaré (D3-1, adversarial CONFIRMED sur sources vendored)
- **Constat** : sous 0.98 la résolution DNS du client pkarr = défaut
  reqwest (getaddrinfo système, à CHAQUE lookup) ; sous 1.0.1
  `DnsResolver::new()` LIT la config système UNE FOIS à la construction
  (fallback SILENCIEUX warn-only vers Google DNS 8.8.8.8/8.8.4.4 si la
  lecture échoue) et SBFB ne câble aucun `reset()` → un daemon
  long-lived qui change de réseau garde la config DNS du boot pour ses
  résolutions pkarr. Le doc-comment réécrit ne mentionne ni le gel ni
  le fallback. Delta INDUIT par le port verbatim §4.3 lui-même (donc
  conforme au contrat), mais pour un module dont la raison d'être est
  l'eclipse defence, l'egress DNS tiers sur échec de lecture config est
  un delta de posture à nommer. Mitigants : feature opt-in
  (`SBFB_PKARR_RELAYS` vide par défaut), dégradation = échec de lookup
  jamais fausse donnée (paquets pkarr signés).
- **Disposition** : NOMMER au body (« DnsResolver::new() = config
  système lue au boot, fallback Google DNS warn-only, pas de reset()
  câblé — delta vs getaddrinfo per-lookup 0.98 ») + router une note
  THREAT_MODEL vers Phase G (§8.4 y route déjà les mises à jour
  THREAT_MODEL) ; optionnel Phase E : le check « survie URL
  dns.iroh.link/pkarr » du préflight §8.2 absorbe la vérification au
  vif.

### P3 (14, dé-dupliqués — tous documentables au body, aucun ne bloque)

1. **Qualification des lints** (D1-1) : la cassure résiduelle MSRV =
   DEUX lints clippy, pas un seul — à corriger dans la déclaration du
   body (absorbé dans P2-3).
2. **Entrée de liste runtime.rs:2525** (D1-2) : re-datage doc-comment
   A2, pas un collapse — noter si la liste est archivée (absorbé P2-3).
3. **Ancre env-classe Docker exacte** (D2-2) : citer
   `sprint78_audit_plan.md §8 TEST-ISOLATION-SBFB-HOME` (+ dossier
   S52/S57/S77 sous 0.98) et NON « documentée depuis A2 » (le body A2
   documente une classe VOISINE) ; coller les lignes d'échec exactes du
   run Docker ; nommer le carry hermétisation avec son fix root-cause
   connu (`.env("SBFB_HOME", tmp)` sur les e2e daemon-spawn — même P2
   fermé en S80 pour operator_server, jamais appliqué à e2e.rs).
4. **Paire d'oracles CONTROL/GREEN** (D2-3) : le CONTROL est un oracle
   unilatéral (fenêtre négative 8s, un flip masqué par une convergence
   >8s false-passerait) — statuer au body CONTROL + GREEN comme PAIRE
   (négatif 8s + positif convergent dans le même run 2028/2028) pour
   que l'audit gate lise la preuve complète.
5. **Chemin de panic théorique** (D3-2) : `expect` interne de
   `DnsResolver::new()` (hickory build) vs promesse « returning an
   error » du doc-comment — pratiquement inatteignable, non testable,
   aucun fix sans dépasser le verbatim ; une phrase au body suffit.
6. **node.rs:24 = repère plan/préflight périmé** (D4-P3-1/D6BIS-2) :
   node.rs INTOUCHÉ (grep = 0 commentaire de version), conformité a
   fortiori — documenter pour que l'audit gate ne cherche pas un hunk
   comment-only inexistant.
7. **Mentions « iroh 0.98 »/« 0.100 » stale hors scope B**
   (D4-P3-2/D5-3/D7-4) : inventaire à lister nominalement au body,
   routé — Phase C : doc_sync.rs:13 (cite des lignes upstream périmées
   de la doctrine sync-set), dispatch_loop.rs:547 + runtime.rs:4211*
   (avec le fix sibling ; *sauf si P2-2 fixé en B), Cargo.toml
   nexus-core-rs:43 (signature PkarrRelayClient::new citée vient de
   changer) ; Phase D : blobs.rs:437 ; Phase E : gossip.rs:259/:740,
   tls_pinning.rs:32, http.rs:3213 ; Phase G : narratifs Cargo.toml
   daemon:83, age_witness.rs:6/:21 (+ THREAT_MODEL:22 déjà routé G).
8. **Deltas lock transitifs complets** (D4-P3-3/D6-P3-2) : au-delà de
   la liste nominale §4.5 — in : serdect, base16ct, rand_pcg,
   objc2-security-foundation, objc2-core-wlan, iroh-util, arc-swap,
   redb (2e entrée) ; out : vergen-lib, rustls-platform-verifier
   (surface TLS transitive réduite, posture EmbeddedWebPki inchangée —
   neutre-à-positif, mérite une ligne), netlink-packet-route. Compléter
   le registre du body pour le gate `cargo tree -d` Phase G.
9. **Repère « 4 tests A2 :4212-4291 » incomplet** (D5-4) : le 4e test
   est à :4319 — citer :4212-4330 ou les 4 noms au body.
10. **Obligations body PENDING-commit** (D6-P3-1) : D8 verbatim
    « upgrade ≠ Gate 1 / Gate 3, R-iroh-audit P0 inchangé, pilote reste
    fermé » + règle boot + registre engagements A/A2/A3/A4 + carries
    statués — vérifiables seulement au commit (hooks lightcheck =
    backstop).
11. **Store VPS dans la fenêtre one-way** (D6-P3-3) : étendre la règle
    au body : « pas de boot store réel — Win, Mac ET VPS — avant
    Phase F PASS » ; snapshot VPS avant tout déploiement 0.101 sur
    l'ancre (Phase H le prévoit).
12. **Étiquette docs-contract au body** (D6BIS-1) : ligne explicite
    « §6.12 : N-A-no-new-frontier — bump deps + collapses MSRV-gated,
    0 route ajoutée/modifiée, 0 payload JSON changé, frontières
    existantes byte-stables ».
13. **Commit shape `chore(deps)` sans précédent** (D7-5) : fixé par
    préflight §11, conforme au shape canonique — AUCUNE action.
14. **§P73 candidate PATTERNS.md** (D7-6) : « a rust-version bump
    silently widens clippy's MSRV-gated lint set under -D warnings —
    budget a mechanical sweep and verify semantic preservation » — dans
    ce commit ou au wrap-up K.

## Corrections requises / à reporter au body (par priorité)

1. **AVANT commit (fix 1 ligne, P2-2)** : re-dater
   `runtime.rs:4211` « In iroh-docs 0.98 » → 0.101 (comment-only, même
   formule que :2526-2529, ne touche aucun test) — ou exception
   consignée au body.
2. **AU BODY, MANDATORY sinon escalade P1 (P2-3)** : liste des 139
   absorptions MSRV fichier:ligne (ou artefact committé référencé) +
   cause rust-version 1.91 + qualification DEUX lints (collapsible_if
   ~137 + manual_is_multiple_of 2) + l'entrée re-datage :2525.
3. **AU BODY (P2-1)** : snapshot Mac PENDING + règle BLOQUANTE nommée
   « aucun binaire 0.101 déployé/booté sur le Mac avant snapshot Mac »
   + pré-condition Phase F/H (re-tenter SSH chaque session) + ne PAS
   claimer « snapshot D3 cond.3 avancé » sans le qualificatif PENDING.
4. **AU BODY (P2-4)** : nommer le delta DNS pkarr (config gelée au
   boot + fallback Google warn-only + pas de reset()) + note
   THREAT_MODEL routée Phase G.
5. **AU BODY (contrat §4.9/§11)** : D8 verbatim ; règle « pas de boot
   store réel — Win, Mac ET VPS — avant Phase F PASS » ; registre
   engagements A/A2/A3/A4 ; carries statués (A2 FERMÉ par constat
   store.rs:24-27/api.rs:262-265 ; A4 sibling STATUÉ → Phase C,
   mécanisme 0.101 vérifié inchangé) ; CONTROL + GREEN statués comme
   PAIRE d'oracles ; delta tests 0 net (Win 2028 0-skip / Docker 2032).
6. **AU BODY (env-classe)** : les 2 fails Docker e2e = ancre exacte
   `sprint78_audit_plan.md §8 TEST-ISOLATION-SBFB-HOME` (pré-bump,
   dossier S52/S57/S77) + lignes d'échec exactes + carry hermétisation
   e2e.rs (`.env("SBFB_HOME", tmp)`).
7. **AU BODY (inventaires)** : N-A-no-new-frontier §6.12 explicite ;
   node.rs INTOUCHÉ (repère :24 périmé) ; repère 4 tests A2
   :4212-4330 ; deltas lock complets 8 in / 3 out ; mentions stale
   routées C/D/E/G.
8. **APRÈS body prep** : Codex (`codex exec`, output BRUT →
   `sprint81_phase_b_codex_review.md`) → réconciliation → promotion
   PASS → commit `chore(deps): Sprint 81 Phase B — ...` (shape §11).
9. **Optionnel** : §P73 PATTERNS.md (ici ou wrap-up K).

## État des suites §7.4

- **Rust Win** : **VERT** — build workspace 3m50s (zéro erreur après
  fix pkarr, sonde préflight exacte), fmt re-joué EXIT 0 par la review,
  clippy -D warnings OK (déclaré, cohérent avec le sweep), nextest
  **2028/2028 0-skip re-vérifié** (== baseline fdb8ad7, delta 0 net ;
  4 tests A2 + CONTROL A4 non-convergent + GREEN A4 re-joués PASS de
  première main en D5), doctests 6/6, release daemon 7m56s post-sweep.
- **Docker sbfb-ci** : **VERT-avec-env-classe** — 2030/2032 + 2 fails
  e2e daemon-spawn (sigint / start_writes) = classe pré-bump
  TEST-ISOLATION-SBFB-HOME (`sprint78_audit_plan.md:129-138`, dossier
  S52/S57/S77 sous 0.98, aucune surface iroh, les 2 verts Win natif ;
  re-runs duo/solo verts). Arithmétique 2028+4 `cfg(unix)` = 2032
  cohérente. PAS une régression bump — consigner au body (correction 6).
- **web** : lint 0 err / 5 warn préexistants, tsc OK, unit 411/411,
  coverage verte (2 flakys GpuConsentDialog re-run solo 17/17, classe
  `vitest_env_variance`, web intouché au diff — 0 fichier), build OK,
  size 5/5, scan-en-strings clean — ATTESTÉS contexte, non contredits
  (0 fichier web au diff).
- **operator** : lint OK, build OK, unit 201/201, gates 6/6 exit 0,
  size 5/5, E2E Playwright 10/10 — ATTESTÉS contexte (0 fichier
  tools/ au diff).
- **Codex** : **PAS JOUÉ** — c'est le PENDING du verdict.

## Carries

- **Phase B (avant/au commit)** : P2-2 fix 1 ligne runtime.rs:4211 ;
  P2-3 liste collapses au body (sinon P1) ; P2-1 Mac PENDING consigné ;
  P2-4 delta DNS nommé.
- **Phase C** : fix sibling sync-set `runtime.rs:2552-2564`/`:2647-2659`
  + tests miroir (carry A4 STATUÉ ici, PAS fixé en B — conforme §7.2) ;
  re-datages doc_sync.rs:13 + dispatch_loop.rs:547 (avec le fix) ;
  test ticket-0.98-parse-sous-nouveau-lock + vérif `.contains` au vif
  (préflight §8.1) ; re-scoper l'item DocsNamespaceId (probable no-op).
- **Phase E** : mentions stale surfaces runtime (gossip.rs:740,
  tls_pinning.rs:32, http.rs:3213) ; check survie URL dns.iroh.link/pkarr
  (absorbe la vérification au vif du resolver P2-4).
- **Phase F** : snapshot Mac = PRÉ-CONDITION (P2-1) ; atomicité
  crash-mid-migration PR #105 ; chemin migration store BLOBS redb2 sous
  0.103 ; validation sur COPIE (D3 cond.4).
- **Phase G** : note THREAT_MODEL posture DNS pkarr (P2-4) +
  THREAT_MODEL:22 depuis sa valeur réelle ; `cargo deny` post-bump ;
  gate `cargo tree -d` avec le registre lock COMPLET (P3-8) ; narratifs
  Cargo.toml/age_witness/blobs.rs:437.
- **Phase H** : snapshot VPS avant bascule live (+ règle boot étendue
  VPS, P3-11) ; re-check crates.io 1.0.2/OSV avant push.
- **Carry hermétisation e2e.rs** (env-classe Docker) : fix root-cause
  connu `.env("SBFB_HOME", tmp)` — router K/dette (correction 6).
- **Wrap-up K** : §P73 PATTERNS.md candidate (P3-14).

## Codex reconciliation

**PAS ENCORE JOUÉ** — d'où le PASS-PENDING. Séquence non-négociable :
honorer les corrections 1-7 (body prep + fix 1 ligne) → `codex exec` sur
le diff FINAL (output brut collé dans
`sprint81_phase_b_codex_review.md`, jamais réécrit) → réconciliation par
le main thread (critère d'arrêt : CLEAN ou P2/P3 documentés) → promotion
du présent verdict en PASS → commit `chore(deps)`. Le hook lightcheck
exigera l'artefact Codex au commit (Check 7) et le préflight existe déjà
(Check 8).

## Codex reconciliation

Codex GPT 5.5 round 1 (`sprint81_phase_b_codex_review.md`, output brut
`codex exec -o`) : **7/8 CONFIRMÉ, 0 GAP, 1 PARTIEL** (Livrable 5,
absorption MSRV). Le PARTIEL affirme que des effets de bord
(`broadcast`, `set_tag`, `emit_seed_announced`, `write_event`) auraient
été « déplacés d'un corps vers une condition » par les collapses
let-chains. **Réconcilié FAUX POSITIF sur pièces** (git diff -U6 des 4
sites cités) : chacun de ces appels était DÉJÀ le scrutinee d'un
`if let Err(e) = ...` INTERNE dans le code d'origine — c'est-à-dire déjà
évalué comme condition, jamais comme corps. Le collapse
scrutinee-interne → maillon de chaîne `&&` préserve exactement l'ordre
d'évaluation gauche-droite et le court-circuit des ifs imbriqués
(sémantique let-chains Rust ≥ 1.88). Aucun appel n'a migré d'un corps
vers une condition ; aucun fix code requis ; suites non re-jouées (0
changement). La contrainte du prompt visait le hasard réel « statement
de corps plié en condition » — absent du diff.

Les 4 P2 de la review sont HONORÉS avant commit : (1) snapshot Mac
PENDING + règle bloquante au body ; (2) `runtime.rs:4211` re-daté 0.101
(fix comment-only, tests A2 re-joués 6/6 PASS) ; (3) liste des 139
absorptions matérialisée dans l'artefact committé
`.planning/active/sprint81_phase_b_collapse_sites.txt` (référencé au
body) ; (4) delta sémantique DNS pkarr nommé au body + note
THREAT_MODEL routée Phase G. Verdict promu **PASS**.
