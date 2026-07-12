# Sprint 81 Phase E — Review (Workflow ultracode + agent de synthèse)

> Phase E « Surfaces fragiles transport re-cert (3 crates) »
> (`sprint81_plan.md:179-204`), **supersédée par le préflight
> PLAN-ADAPT** `sprint81_phase_e_preflight.md`. La lettre décrivait une
> « re-certification compile + handshake » des surfaces transport
> `shard.rs`/`seed_protocol.rs`/`pkarr_resolver.rs`/`relay_config.rs`/
> `node.rs` + un check nommé de survie URL pkarr/relais + un PLAN B C8
> pré-provisionné. Comme A/A2/A4/B/C/D, la lettre était **pré-bump** : le
> bump B (`c899d54`) a déjà absorbé toute la surface transport (nextest
> vert sous iroh 1.0.1) et les handshakes `sbfb/seed/0` + `sbfb/shard/1`
> in-process sont **déjà verts** (delta handshake ≈ 0, S3-8 réhabilité).
> Le vrai périmètre E-core = **4 gestes minces** : (1) NO-OP re-cert
> documentée ; (2) +1 test net tripwire hermétique pkarr ; (3) lot
> doc-stale 5 sites ; (4) artefact T2 du CHECK NOMMÉ live. Le SPLIT E'
> (PLAN B C8 : code neuf override discovery + relais/pkarr self-hosted +
> acceptance zéro-n0) est reporté au commit suivant. Arbre SALE, HEAD
> `7bd3578`, diff NON committé (6 fichiers + 2 untracked planning). Review
> menée sur le diff COMPLET ligne-par-ligne + grounding vendored/repo,
> synthèse de 4 dimensions (D1 facts / D2 test / D3 scope / D6 patterns) +
> vérifications adversariales, réconciliées ici.

## Verdict: PASS

> *(Verdict émis CONCERN par la synthèse Workflow — promu **PASS**
> après application de TOUTES les dispositions [réconciliation driver
> ci-dessous] et réconciliation Codex round 1 : 5/6 CONFIRMÉ, 0 GAP,
> 1 PARTIEL documentaire consigné.)*
>
> **Diff Phase E substantivement CONFORME au contrat préflight qui
> supersede la lettre.** 0 P0. **1 P1 trivialement corrigeable** (fix
> 1 ligne, 0 impact test) + 6 P3 (documentables au body / carry / à
> laisser tel quel). Le diff est minimal et exactement celui annoncé :
> 5 doc-comments transport re-datés/re-fondés + **1 seul test net**
> hermétique (`default_pkarr_url_matches_iroh_upstream_const`,
> `pkarr_resolver.rs:212`) + PATTERNS §T20 status-update + 2 artefacts
> planning (préflight + T2). **0 code prod fonctionnel**, 0 `Cargo.lock`/
> `Cargo.toml`, 0 dep, 0 store ouvert, 0 constante wire — bisectabilité
> préservée (la recompilation appartient à B). Delta +1 net cohérent
> (Win 2039→2040 / Docker 2043→2044). Les 7 classes de claims doc
> load-bearing ont été re-vérifiées **indépendamment contre le vendored**
> (tls.rs:141, endpoint.rs:713, defaults.rs 4 relais + drop `iroh-canary`,
> gossip net.rs:84-86, DEFAULT_RELAY_QUIC_PORT=7842, tokio_websockets,
> pkarr.rs:127 byte-identique) : **6/7 EXACTES**, le piège §12-P3 du
> préflight (re-datage aveugle de `tls_pinning.rs`) a été **évité** — le
> fond est réellement re-vérifié (le hook amont `custom_server_cert_
> verifier` a bien atterri en 1.0). Le split E' est respecté à la lettre
> (0 fuite de code override-discovery / zéro-n0 dans ce diff) ; aucun
> claim « fait » interdit, jamais « shard SAUVÉ ».
>
> **CONCERN = 1 P1 doc-accuracy à corriger AVANT commit, dans le commit
> même qui corrige les doc-stale** (`gossip.rs:747` : le parenthétique
> `(nexus-shell-daemon)` mislocalise `shard.rs`, qui vit dans
> `nexus-core-rs` — le MÊME crate que `gossip.rs`). Ce n'est pas un défaut
> runtime ; c'est une inexactitude factuelle **introduite par ce diff**
> dans une phase dont le SEUL livrable est l'exactitude des doc-comments.
> Fix d'une ligne, 0 impact test. Une fois corrigé (+ les P3 tranchés),
> le diff est promouvable PASS au gate Codex.
>
> Séquence : corriger E-DOC-1 → honorer les P3 au body → Codex →
> réconciliation → promotion PASS → commit `test(core)`.

## Périmètre et staging

`git status --short` = **8 lignes** (6 fichiers modifiés + 2 untracked
planning), 0 fichier parasite :

- `crates/nexus-core-rs/src/gossip.rs` (2 doc-comments : `:257-259`
  Arc<Inner> 0.97→0.101 re-vérifié `net.rs:84-86` ; `:739-747` `MemoryLookup`
  1.0.1 + renvoi handshake shard/seed).
- `crates/nexus-core-rs/src/pkarr_resolver.rs` (+1 `#[test]`
  `default_pkarr_url_matches_iroh_upstream_const` `:211-239`, synchrone
  pur).
- `crates/nexus-core-rs/src/relay_config.rs` (2 doc-comments : `:5`
  three→four ; `:18-25` byte-for-byte retiré + hostnames `use1-1/usw1-1/
  euc1-1/aps1-1 .relay.n0.iroh.link` + drop label `iroh-canary`).
- `crates/nexus-core-rs/src/tls_pinning.rs` (1 doc-comment `:32-53`
  re-fondé : blocker amont GONE en 1.0.1, `custom_server_cert_verifier`
  + `ca_tls_config`, T20 non-fermé — câblage SBFB PENDING).
- `crates/nexus-shell-daemon-core/src/transport_probe.rs` (1 doc-comment
  `:23-33` WSS-only « still true under 1.0.1 » + `DEFAULT_RELAY_QUIC_PORT`
  7842 = discovery, pas data-path).
- `docs/rust/PATTERNS.md` (§T20 : titre + blockquote status-update `:974`,
  `:975-988`).
- `?? .planning/active/sprint81_phase_e_preflight.md` — artefact préflight
  (relu en entier, approche corrigée PLAN-ADAPT que le code suit).
- `?? .planning/active/sprint81_t2_e_discovery_survival.json` — artefact
  T2 (moitié LIVE du CHECK NOMMÉ ; vocabulaire T2 clos ; sonde 5 cibles ;
  note `IROH_FORCE_STAGING_RELAYS`).

**Cohérence Rust** : `git diff '*.rs' | rg '^\+pub (fn|mod|struct|const)'`
= **0 nouvelle surface publique**. Le seul code ajouté est un `#[test]`
dans un `mod tests` existant (`#[cfg(test)]`). **0 code prod fonctionnel**
— les 5 autres hunks Rust sont exclusivement des doc-comments (`///`/
`//!`/`//`).

**INTOUCHÉS prouvés par absence du diff** (split E' + carries honorés) :
`node.rs` (0 fuite override discovery : `grep` du diff = 0
`clear_address_lookup`/`PkarrPublisher::builder`/`PkarrResolver::builder`/
`RelayMode::Custom`/`address_lookup(`), `shard.rs`/`seed_protocol.rs`
(handshakes déjà verts, non ré-implémentés), `http.rs:3213` (déjà fermé en
C — correctement NON touché), `age_witness.rs` (carry K), `Cargo.toml`/
`Cargo.lock`/`deny.toml` (0 dep), tout `DOMAIN_*`/`_FORMAT_VERSION` (grep
du diff = 0), `web/`, `tools/`. **Aucun store redb ouvert.**

## Vérification trois blocs

Suites §7.4 **fournies au contexte (déjà jouées et vertes), auditées en
cohérence — non relancées** (lourdes) :

- **Rust Win** : fmt 0 ; clippy `--all-targets -D warnings` 0 ; nextest
  **2040/2040 0-skip** (baseline D 2039 → **+1 EXACT**, cohérent avec le
  seul test neuf) ; doctests exit 0 ; release build OK.
- **Docker sbfb-ci** rust:1.94 : fmt 0 ; clippy 0 ; nextest **2044/2044
  0 fail 0 skip** (baseline D 2043 → +1 exact) ; doctests 6/6.
- **Front (web + operator)** : **EN COURS en parallèle**. **AUCUN fichier
  front touché par le diff** (`git status` = 0 fichier `web/`/`tools/`) →
  insensible par construction. **État = pending** au moment de la review :
  le body devra noter « front pending » si les suites ne sont pas closes
  au commit ; leur résultat ne peut PAS régresser sur un diff qui ne
  touche que `nexus-core-rs` + `nexus-shell-daemon-core` (Rust) +
  `PATTERNS.md`.

**Preuve sémantique ciblée** : l'API du test neuf est compile-prouvée
(présente dans nextest 2040). Les deux consts comparées sont
**GÉNUINEMENT INDÉPENDANTES** — `DEFAULT_PKARR_RELAY_URL` (littéral
`pkarr_resolver.rs:55`) vs `iroh::address_lookup::N0_DNS_PKARR_RELAY_PROD`
(littéral vendored `iroh-1.0.1/src/address_lookup/pkarr.rs:127`, re-exporté
via `address_lookup.rs:123 pub mod pkarr` + `:128 pub use pkarr::*`). Deux
`pub const &str` distincts dans deux crates → l'`assert_eq` **casse au
prochain bump** si iroh déplace sa const (drift réellement attrapé, pas une
tautologie de symbole partagé).

## Delta tests

**+1 Rust net** — annoncé et observé cohérent partout : Win 2039→2040,
Docker 2043→2044. **0 zombie ajouté, −0.** L'unique test neuf est le
tripwire de parité pkarr ; **0 handshake ré-implémenté** (l'interdiction du
préflight §3/§8 est respectée : `sbfb/seed/0` [13 `#[tokio::test]`] +
`sbfb/shard/1` [`shard_handshake_admits_member:515`,
`_rejects_non_member:537`] déjà verts → delta handshake = 0). Aucun test
`legacy-decode` à supprimer (Phase E = re-cert doc, 0 redéfinition de
format). Le `// Note:` de `gossip.rs:739` est un commentaire, pas un test.

## Couverture de branche du fichier modifié

Le diff n'introduit **aucune branche ni méthode de code prod** (5
doc-comments = prose ; le test neuf EST de la couverture). Le test est
linéaire (0 `if`/`match`). Assertions :

- `assert_eq!(DEFAULT_PKARR_RELAY_URL, N0_DNS_PKARR_RELAY_PROD, …)` —
  parité de const (tripwire drift amont, `:230-234`).
- `Url::parse` OK + `scheme()=="https"` (`:235-238`) — garde HTTPS.
  **Quasi-redondante avec l'assert_eq exact qui la précède** (D2-1, P3) :
  si l'égalité de chaîne passe, la chaîne est le littéral fixe valide-HTTPS
  → parse/scheme ne font un travail indépendant que dans le scénario
  quasi-nul où LES DEUX consts dériveraient vers une même URL non-HTTPS.
  Harmless, auto-documente l'invariant, à conserver tel quel.

**Hermétisme confirmé** : 0 réseau / 0 store / 0 env — le test lit une
`const &str` compile-time (pas `n0_dns()` qui lit l'env), donc il n'a pas
besoin d'`ENV_GUARD` (Mutex `:290`) et n'en prend pas, correctement
(contrairement aux `load_quorum_resolvers_from_env_*` voisins qui mutent
`SBFB_PKARR_RELAYS`). Insensible à `IROH_FORCE_STAGING_RELAYS` par
construction (il pin la const PROD, pas la sélection env — le doc-comment
le note honnêtement).

## Sécurité et protocole

- **Test PUR sans réseau** : compare une const + parse une chaîne, aucun
  `Endpoint`/`create_node`/`FsStore`/`data_dir`. Aucun dial possible.
- **0 bump wire SBFB** : `grep` du diff = 0 `DOMAIN_*`/`_FORMAT_VERSION` ;
  ALPN `sbfb/seed/0`+`sbfb/shard/1` non touchés (23 `DOMAIN_*_V1` intacts).
  La const pkarr est une URL d'iroh, pas un wire SBFB.
- **0 dep** : `Cargo.toml`/`Cargo.lock`/`deny.toml` absents du diff. Split
  E' (qui ajouterait éventuellement `test-utils` dev-dep) reporté.
- **`.expect(...)`** sur parses statiques (URL const) — légitime en
  `#[test]` (échec = bug d'infra de test). `grep unsafe|todo!|
  unimplemented!` du diff = 0.
- **Surface d'attaque inchangée** : les doc-comments corrigent des données
  de version périmées ; `tls_pinning.rs` documente honnêtement que la
  posture reste **WebPKI-only au runtime** (T20 non-câblé) — le blocker
  amont est levé mais le câblage SBFB reste une dette ouverte (routée E'/
  G), PAS fermée par ce diff. Aucune sur-vente de sécurité.

*(Dimensions 4 [correctness/logique] et 5 [security/protocol] du barème
6-dimensions ne figuraient pas dans le paquet de synthèse : la correctness
est trivialement N-A [0 code prod fonctionnel], et le volet security/
protocol est couvert par cette section — conforme au format Phase D qui
portait « Security And Protocol » plutôt qu'une dimension numérotée
séparée.)*

## Recherche et G8

- **G8 préflight présent et complet** : `sprint81_phase_e_preflight.md`,
  verdict **PLAN-ADAPT** (5e « lettre pré-bump », structurel/attendu),
  6 scans (S1a API transport / S1a2 discovery pkarr+relais+PLAN B / S1b
  deps-CVE-lock / S2 décisions / S3 threat / S4 wire) + 6 vérifications
  adversariales. Le code SUIT l'approche corrigée du préflight, PAS la
  lettre du plan.
- **Claims doc load-bearing vérifiés au vendored (non pris sur parole)** :
  - **(a)** `iroh-gossip-0.101.0 net.rs:84-86` — `#[derive] pub struct
    Gossip { inner: Arc<Inner> }` (cité exact `:84-86`).
  - **(c)** `iroh-1.0.1 defaults.rs:27-33` — quatre hosts `use1-1/usw1-1/
    euc1-1/aps1-1 .relay.n0.iroh.link` vs `iroh-0.98.2` `*.relay.n0.iroh-
    canary.iroh.link` (label `iroh-canary` retiré ; « three→four » exact —
    les DEUX versions avaient déjà quatre relais, seul le LABEL a changé,
    aucune induction en erreur ; « byte-for-byte » retiré à raison).
  - **(d)** `iroh-relay-1.0.1 tls.rs:141 custom_server_cert_verifier` (mode
    `CustomServerCertVerifier`) + `iroh-1.0.1 endpoint.rs:713 ca_tls_config`
    (doc `:706-712` nomme « iroh relays, pkarr servers, DoH resolvers »
    comme consommateurs → le hook atteint réellement le chemin relais visé
    par le cert-pinning) ; hook ABSENT en 0.98.0 → **LANDED in 1.0 exact**.
  - **(e)** `iroh-relay-1.0.1 client/conn.rs:85/:112 tokio_websockets` +
    `defaults.rs:7 DEFAULT_RELAY_QUIC_PORT=7842` (« QUIC address
    discovery »).
  - **(f)** un SEUL `Endpoint::builder` prod = `node.rs:318`.
  - **(g)** `N0_DNS_PKARR_RELAY_PROD` atteignable via `pub use pkarr::*` ;
    `pkarr.rs:127 == DEFAULT_PKARR_RELAY_URL (:55)` → tripwire PASSE
    aujourd'hui.
- **Piège §12-P3 ÉVITÉ** : le fond de `tls_pinning.rs` a bien été
  re-vérifié (pas un sed aveugle) — le hook amont existe réellement.
- **iroh strictement seul (D7)** : 0 dep ajoutée. **Toolchain 1.94 (D6).**

## Scope cuts

- **Conformité PLAN-ADAPT** : rien au-delà du scope préflight. 0 code prod
  fonctionnel, 0 store ouvert, **split E' respecté à la lettre** (aucune
  fuite de code override-discovery / relais-pkarr self-hosted / acceptance
  zéro-n0 ; T2 `residual_risk` route explicitement C8 → E').
- **Carries corrects hors-code** : E'(PLAN B C8), F(dualité redb +
  durabilité pins + anchors degrade + snapshot Mac), G(silent-loss
  discovery + quinn-proto 0.11.14 RÉSOLU + threat PLAN B + câblage T20),
  K(age_witness.rs + repères plan périmés + sites BlobTicket daemon),
  I-J(RTT multipath LIVE). Aucun n'est du code E.
- **RTT multipath shard `UNVERIFIED-high-risk`** correctement borné I/J ;
  **jamais « shard SAUVÉ »** dans le diff.
- **docs-contract §6.12 (test-acteur)** : le test pinne une CONSTANTE
  UPSTREAM iroh (`N0_DNS_PKARR_RELAY_PROD`) dans le MÊME crate ; aucune
  frontière SBFB (wire/API/app/LLM) créée, aucun format persisté →
  **N-A-no-new-frontier**, aucune étiquette due. À consigner au body.

## Restitution des dimensions

| Dim | Portée | Verdict local | Réconciliation |
|---|---|---|---|
| **D1 — facts vs vendored** | Exactitude des claims doc vs iroh 1.0.1 / iroh-relay 1.0.1 / gossip 0.101 / 0.98.2 | **CONCERN** | 6/7 classes de claims EXACTES (evidence positive au body). 1 P1 (E-DOC-1 crate misattribution) tenu adversarialement. 1 P3 (E-DOC-2 conflation pkarr). Manques M1/M2 intégrés (P3). |
| **D2 — sémantique test + zombies + coverage** | +1 net, hermétisme, drift réel, branches | **PASS** | Tripwire genuine non-tautologique, 0 zombie, interdiction handshake respectée, absence justifiée d'ENV_GUARD. D2-1 (garde parse redondante) P3. MANQUE-1/2 = doublons E-DOC-2/E-DOC-1. |
| **D3 — scope + préflight PLAN-ADAPT** | 4 livrables E-core, split E', Day-0, carries | **PASS** | Exécution fidèle du préflight ; 0 fuite E' ; http.rs:3213/age_witness intacts ; piège §12-P3 évité. Manque = doublon gossip.rs shard.rs (P3) + note T2 non-re-vérifiable (autre dimension). |
| **D6 — patterns + conventions + docs-contract** | Langue, anti STALE-PHASE-K, §6.9/§6.12, style | **PASS** | Anglais partout ; in-code CLEAN (aucune phase future nommée in-code, seul le tracker §T20 route par lettres) ; §6.9 respecté ; 0 étiquette docs-contract due. D6-1 (routing E'/G volatil) + D6-2 (orpheline reflow) + OBSERVATION-2 (Fix-path 1) P3. |

**Convergence** : 3 dimensions PASS + 1 CONCERN (D1). Le seul finding
bloquant-avant-commit est **E-DOC-1 (P1)**, remonté par D1 (remit exact =
exactitude doc) et confirmé adversarialement sans downgrade ; D2/D3/D6 ont
surfacé le MÊME défaut à P3 dans leurs remits secondaires. **Réconciliation
= P1 « à corriger avant commit »** (la dimension dont c'est le cœur de
métier prime, et une phase de re-certification doc-accuracy ne doit pas
livrer une nouvelle inexactitude de localisation crate) ; **trivialement
corrigeable** (1 ligne, 0 impact test) → verdict global **CONCERN**, jamais
FAIL.

## Findings

### P0 — aucun

### P1 (1, à corriger AVANT commit)

**E-DOC-1 — `gossip.rs:747` : le doc-comment ajouté attribue `shard.rs` à
`nexus-shell-daemon`, or `shard.rs` vit dans `nexus-core-rs` (le MÊME crate
que `gossip.rs`).** Le hunk `gossip.rs:745-747` (ajouté par ce diff) dit :
« Two-node handshake coverage at the transport layer lives in `shard.rs`
and `seed_protocol.rs` (nexus-shell-daemon) instead. » Vérif filesystem :
les tests handshake `shard_handshake_admits_member` (`crates/nexus-core-rs/
src/shard.rs:515`) + `shard_handshake_rejects_non_member` (`:537`) vivent
dans **`crates/nexus-core-rs/src/shard.rs`** (Cargo.toml `name =
nexus-core-rs`) = MÊME crate que `gossip.rs`. Il n'existe **AUCUN
`shard.rs` sous `nexus-shell-daemon`/`nexus-shell-daemon-core`** (`find
crates -name shard.rs` → `nexus-core-rs/src/shard.rs`,
`nexus-core-rs/src/schemas/shard.rs`, `nexus-worker-core/src/llm/shard.rs`
seulement). Seul `seed_protocol.rs` est effectivement dans
`crates/nexus-shell-daemon/src/seed_protocol.rs`. Le parenthétique
`(nexus-shell-daemon)` porte, en lecture naturelle, sur la liste « shard.rs
and seed_protocol.rs » → **faux pour shard.rs**. Aggravant : 3 fichiers
`shard.rs` distincts au workspace → un mainteneur lisant `gossip.rs`
(nexus-core-rs) irait chercher `shard.rs` dans `nexus-shell-daemon` et ne
l'y trouverait pas. **Pourquoi P1 (et non P3)** : c'est exactement la classe
de défaut (doc-comment inexact sur la localisation crate) que la Phase E
prétend corriger — livrer une nouvelle inexactitude dans le commit qui
solde les doc-stale est auto-contradictoire. Désambiguïsation load-bearing,
fix 1 ligne, 0 impact test. **Disposition** : corriger avant commit, p.ex.
« lives in `shard.rs` (this crate, nexus-core-rs) and `seed_protocol.rs`
(nexus-shell-daemon) instead. » `crates/nexus-core-rs/src/gossip.rs:747`.

### P3 (6, documentables au body / carry / à laisser tel quel)

**E-DOC-2 — `pkarr_resolver.rs:218-221` : le commentaire du test tripwire
conflate le déplacement des hostnames RELAIS avec un déplacement de l'URL
PKARR (qui n'a PAS bougé).** Le commentaire dit « if upstream ever moves
the n0 pkarr relay (the relay hostnames DID move at 1.0,
`*.relay.n0.iroh-canary.iroh.link` -> `*.relay.n0.iroh.link`) ». Fait
vérifié : l'URL pkarr `https://dns.iroh.link/pkarr` est **byte-identique**
`iroh-0.98.2 pkarr.rs:133` == `iroh-1.0.1 pkarr.rs:127` — elle n'a jamais
bougé ; ce sont les hostnames de la FLOTTE de relais (constante sœur qui
gouverne `relay_config.rs`, classe d'endpoint distincte) qui ont perdu le
label `iroh-canary`. Chaque fait est individuellement vrai, mais la
juxtaposition laisse entendre que la const pkarr a subi le rename. L'ASSERT
reste valide ; seule la rationale est imprécise. **Disposition** :
optionnel — reformuler p.ex. « n0's published URLs do churn — the relay
hostnames dropped the `iroh-canary` label at 1.0 » pour ne pas suggérer que
l'URL pkarr a bougé. Non bloquant. `crates/nexus-core-rs/src/
pkarr_resolver.rs:220`.

**D2-1 — `pkarr_resolver.rs:235-238` : garde `Url::parse` + `scheme()==
"https"` quasi-redondante avec l'`assert_eq` exact qui la précède.** Si
l'égalité de chaîne (`:230-234`) passe, la chaîne EST le littéral fixe
valide-HTTPS → parse+scheme ne peuvent échouer indépendamment que dans un
scénario quasi-nul. **Disposition** : **CONSERVER tel quel** — garde
documentaire cheap qui auto-documente l'invariant HTTPS, pas un défaut.
Aucune action.

**M1 — incohérence de version iroh 0.97 vs 0.98 entre DEUX fichiers édités
dans le MÊME commit.** `docs/rust/PATTERNS.md:994` (corps §T20 retenu) dit
« because iroh **0.97** exposes » (et le titre `:974` « was: no public hook
in **0.97** ») ; `crates/nexus-core-rs/src/tls_pinning.rs:33` dit « At
delivery time (iroh **0.98**, context7 … 2026-04-16) ». Les deux docs
décrivent le MÊME fait historique (livraison S19 Phase C) et divergent sur
la version iroh. Non tranchable par les seules sources vendored (fait
historique SBFB). **Disposition** : harmoniser sur une seule version dans
les deux fichiers, ou noter au body. `docs/rust/PATTERNS.md:994` +
`crates/nexus-core-rs/src/tls_pinning.rs:33`.

**D6-1 — `PATTERNS.md §T20:988` : le routing de carry par lettres de phase
« (E'/G routing) » est peu durable.** PATTERNS.md est le tracker de dette
(zone autorisée, hors anti STALE-PHASE-K qui vise l'IN-CODE) : nommer une
phase y est permis. Mais « (E'/G routing) » est une conjecture (E' OU G)
qui deviendra stale si le carry re-route. L'IDENTITÉ de la dette (câbler
`PinValidator` dans `ca_tls_config` au site endpoint builder `node.rs`) est
durable ; le numéro de phase est volatile. **Disposition** : reformuler
pour découpler la dette de son routing (renvoi au ledger de carries du
préflight §10 comme source de vérité), ou assumer comme édition-tracker.
`docs/rust/PATTERNS.md:988`.

**D6-2 — `relay_config.rs:8` : le reflow du doc-comment laisse une ligne
orpheline courte.** La ré-écriture « three »→« four (NA east, NA west, EU,
AP) » a poussé les mots et laissé « `//! mix — without` » (~18 chars) seul
alors que le fichier remplit à ~62 chars. `rustfmt` ne reflow pas les
`//!` → l'orpheline persiste comme écrite. Purement cosmétique.
**Disposition** : reflow le paragraphe avant commit, ou porter en carry K.
`crates/nexus-core-rs/src/relay_config.rs:8`.

**OBSERVATION-2 (D6) — `PATTERNS.md §T20:981-982` : « Fix-path 1 below is
therefore realisable **without any upstream PR** » est auto-contradictoire à
la lettre.** « Fix path 1 » défini plus bas (`:1010`) EST littéralement
« **Upstream iroh PR** proposing `ClientBuilder::custom_cert_verifier` ».
Dire « fix-path 1 (= soumettre une PR) est réalisable sans PR » se
contredit à la lettre ; l'intention (le HOOK que la PR visait existe
maintenant amont via `ca_tls_config`, donc la PR est inutile) est claire en
contexte. Confiné au tracker, n'affecte aucune frontière ni le code.
**Disposition** : optionnel — reformuler « the endpoint hook Fix-path 1
sought now exists upstream, so the PR is unnecessary », ou assumer comme
édition-tracker. `docs/rust/PATTERNS.md:982`.

### Notes (non-findings — carry / body)

- **M2 (D1, carry K)** — le corps T20 RETENU (`PATTERNS.md:1015-1016`) situe
  le câblage futur « likely in `crates/nexus-shell-daemon-core/src/
  iroh_runtime.rs` » alors que le nouveau blockquote redirige vers
  `crates/nexus-core-rs/src/node.rs` (l'unique `Endpoint::builder`,
  `:318`). Le blockquote supersede correctement (pattern status-on-top /
  history-below) → PAS un défaut introduit par le diff ; à traiter en carry
  doc K si `iroh_runtime.rs` devient trompeur.
- **MANQUE-3 (D2, skip/carry)** — le doc-comment de `DEFAULT_PKARR_RELAY_URL`
  au site de définition (`pkarr_resolver.rs:47-55`) décrit la const comme
  « reference value operators can copy » sans mentionner qu'elle DOIT rester
  en lockstep avec l'upstream ; l'invariant vit dans le test. Couplage
  self-évidemment intentionnel → marginal, honnêtement défendable de ne rien
  changer.

## Dispositions

**À corriger AVANT commit** :
- **E-DOC-1 (P1)** — désambiguïser `gossip.rs:747` : « `shard.rs` (this
  crate, nexus-core-rs) and `seed_protocol.rs` (nexus-shell-daemon) ». Fix
  1 ligne, 0 impact test. **Bloquant du verdict PASS.**

**À trancher au body (documenter ou corriger à froid, non bloquant)** :
- E-DOC-2 (P3) — reformuler ou noter la conflation hostnames-relais / URL
  pkarr au commentaire `pkarr_resolver.rs:220`.
- M1 (P3) — harmoniser 0.97/0.98 entre `PATTERNS.md:994` et
  `tls_pinning.rs:33`, ou noter au body.
- D6-1 / OBSERVATION-2 (P3) — découpler le routing E'/G et lever la
  contradiction « Fix-path 1 sans PR » dans `PATTERNS.md §T20`, ou assumer
  comme éditions-tracker.
- D6-2 (P3) — reflow orpheline `relay_config.rs:8` (ou carry K).

**À laisser tel quel** :
- D2-1 (P3) — garde `Url::parse`+`scheme` conservée (documentaire cheap).

**À router en carry** :
- M2 → carry K (localisation `iroh_runtime.rs` obsolète dans le corps T20).
- MANQUE-3 → carry K ou skip (doc def-site lockstep, marginal).

**Checklist body (9 sections canoniques + delta cumulé)** : (1) delta
handshake = 0 ACTÉ (seed+shard déjà verts, S3-8 réhabilité) ; (2) NO-OP
re-cert transport (byte-diff vendored absorbé par bump B) ; (3) +1 test net
= tripwire pkarr, Win 2039→2040 / Docker 2043→2044 ; (4) CHECK NOMMÉ live =
artefact T2 (jamais unit test ; `IROH_FORCE_STAGING_RELAYS` noté) ; (5) lot
doc-stale 5 sites + `http.rs:3213` NON touché (fermé C) ; (6) §T20 flip =
hook amont LANDED en 1.0 mais câblage SBFB PENDING → carry, NON fermé ; (7)
SPLIT E' = PLAN B C8 reporté au commit suivant ; (8) carries E'/F/G/K/I-J ;
(9) RTT multipath → I/J, JAMAIS « shard SAUVÉ » ; (10) invariants 0 bump
wire (23 `DOMAIN_*_V1`, ALPN verbatim), 0 dep, toolchain 1.94 ; (11)
docs-contract : **aucune frontière touchée** (5 doc-comments + 1 test
hermétique pinnant une constante upstream iroh) → aucune étiquette due,
`check-frontier-contracts.sh` non concerné ; (12) **front pending** si les
suites web/operator ne sont pas closes au commit.

## Residual Risk

- **Résidu `tls_pinning` T20 non-câblé** : le hook amont existe (1.0.1) mais
  la posture runtime reste WebPKI-only ; câblage SBFB routé E'/G, **NON
  fermé** par ce diff — le body ne doit pas sur-vendre.
- **Sonde LIVE T2 non re-vérifiable depuis le seul diff** : les observations
  de `sprint81_t2_e_discovery_survival.json` (IPs, timings, codes HTTP)
  relèvent de l'honnêteté-acceptance T2 (autre dimension) — l'artefact est
  correctement classé E-core, vocabulaire clos, `residual_risk` route C8 →
  E'. Non bloquant côté scope.
- **`IROH_FORCE_STAGING_RELAYS`** : sous cet env, la sonde live frappe
  staging ; le tripwire statique pin la const PROD — noté honnêtement au
  test et au T2.
- **Env session (05/07)** : reboots/fins de session tuent les shells en vol ;
  piège `cmd | tail && …` masque les exit codes — non pertinent pour le
  test unit pur de `nexus-core-rs` ; pertinent pour l'acceptance E' LIVE.

## Réconciliation driver post-review (2026-07-05, avant Codex)

**Dimensions D4 (artefacts) + D5 (sécurité deep) REJOUÉES** en fan-out
avant-plan (fallback §7.1 — les 2 agents Workflow avaient crashé au cap
StructuredOutput ; la synthèse ci-dessus n'intégrait que 4/6 dimensions) :

- **D4 artefacts — PASS** (sous réserve D4-1 corrigé). Rejeu LIVE de la
  sonde : 6/6 observations reproduites à l'IP près. Vocabulaire T2 clos
  conforme A3/A4 ; tripwire cité par nom exact ; hostnames = defaults
  vendored 1.0.1 ; 400-malformé correctement borné « handler vivant ».
  Findings : **D4-1 (P2)** off-by-one « 8 tests existants » → 7 (le 8e EST
  le tripwire ; propagé du préflight §5.1 vers le T2) — **CORRIGÉ** dans le
  JSON ; erratum préflight consigné ici (artefact point-in-time non
  réécrit) ; **D4-2 (P3)** attribution du fallback relais à
  `relay_config.rs` au lieu du preset `node.rs` — **CORRIGÉ** (JSON) ;
  **D4-3 (P3)** frère « byte-for-byte » stale MANQUÉ par le lot :
  `node.rs:324-328` — **CORRIGÉ** (même lot doc-stale, 7e site) ;
  **D4-4 (NOTE)** phrasé `probe_env_note` — **CORRIGÉ** (curl ne consulte
  pas l'env iroh).
- **D5 sécurité — PASS.** 8/8 claims techniques du diff re-vérifiés
  CONFIRMÉS contre le vendored (dont `tls.rs:141-146` + type `:221-223`,
  `endpoint.rs:713-716` injecté `sock_opts.tls_config:260-274`, doc
  upstream `:706-712`). Table fail-open/fail-closed + threat model T2-T5
  byte-intacts. Fuite sonde T2 = acceptable (infra n0 publique, relation
  de trafic préexistante, 0 secret committé). Findings : **D5-1 (P3)**
  phrase runtime explicite — **CORRIGÉ** (`tls_pinning.rs` : « Until that
  wiring lands, the live relay path remains WebPKI-only — even when a
  relay-pins.json file is present ») ; **D5-2 (P3)** blast-radius
  `ca_tls_config` (gate AUSSI pkarr/DoH TLS ; `custom_server_cert_verifier`
  REMPLACE WebPKI, `with_extra_roots` ignoré `tls.rs:150` ; la row
  fail-closed `NoPin` doit être scopée aux hostnames relais) — **CORRIGÉ**
  (caveat ajouté au blockquote §T20 pour le câbleur futur).

**Dispositions de la synthèse APPLIQUÉES** (toutes, diff re-touché) :

- **E-DOC-1 (P1) CORRIGÉ** — `gossip.rs` : « `shard.rs` (this crate) and
  `seed_protocol.rs` (nexus-shell-daemon) ».
- **E-DOC-2 (P3) CORRIGÉ** — commentaire tripwire : « this URL has stayed
  byte-identical since 0.98.2, but the *relay* hostnames DID move at 1.0 »
  (plus aucune conflation pkarr/relais).
- **M1 (P3) CORRIGÉ + TRANCHÉ PAR GIT** — la vérité historique est
  **iroh 0.97** : le commit S19 Phase C `540bb51` (création de
  `tls_pinning.rs`) portait `Cargo.toml` `iroh = "0.97"`. Le doc-comment
  original « iroh 0.98 relay client » était déjà faux ; harmonisé sur
  « iroh 0.97 per the S19 Phase C lockfile » (`tls_pinning.rs`), PATTERNS
  §T20 déjà exact.
- **D6-1 (P3) CORRIGÉ** — routing découplé : renvoi au ledger
  `sprint81_phase_e_preflight.md §10` au lieu de « (E'/G routing) ».
- **OBSERVATION-2 (P3) CORRIGÉ** — reformulé : « The endpoint hook that
  fix-path 1 below set out to obtain via an upstream PR now exists
  upstream, so neither the PR nor the forked connect path (fix-path 2) is
  needed » + « The body below is kept verbatim as the S19-era historical
  record ».
- **D6-2 (P3) CORRIGÉ** — reflow orpheline `relay_config.rs:8`.
- **D2-1 CONSERVÉ tel quel** (disposition suivie) ; **M2 + MANQUE-3 →
  carry K** (au body).

**Suites relancées après corrections** : voir addendum ci-dessous (les
corrections sont doc-comments + 1 string de commentaire test + artefacts
planning — re-vérification 2 plateformes exigée avant commit).

## Codex reconciliation

**Codex GPT 5.5 round 1 (2026-07-05,
`sprint81_phase_e_codex_review.md` = output brut `codex exec -o`) :
5/6 CONFIRMÉ, 0 GAP, 1 PARTIEL → réconcilié, 0 fix code.**

- Livrables 1 (tripwire — indépendance des consts confirmée + run ciblé
  PASS), 2 (lot doc-stale 7 sites — chaque ancre vendored re-vérifiée
  par Codex, dont E-DOC-1 corrigé constaté au working tree), 3 (T2 JSON
  valide, tripwire cité par nom, « 7 pre-existing », residual_risk,
  0 secret), 4 (périmètre négatif strict — 0 fuite E', 0 dep, 0 wire,
  0 store, handshakes non ré-implémentés), 6 (delta +1 exact, 0 test
  supprimé) : **CONFIRMÉ**.
- Livrable 5 **PARTIEL** (documentaire, pas un défaut code) : le
  préflight en tête annonce « 5 doc-comments » alors que le diff final
  post-review en couvre 7 (le 6e = `node.rs` D4-3 découvert PAR la
  review ; le 7e = `PATTERNS.md` §T20 que le préflight §13 annonçait
  déjà dans le commit shape). **Disposition** : le préflight est un
  artefact point-in-time pré-review — il n'est PAS réécrit (même
  doctrine que l'erratum D4-1 « 8 tests » ci-dessus) ; la réconciliation
  driver documente les 2 sites additionnels et le body du commit liste
  les 7 sites. Aucune boucle Codex supplémentaire requise (critère
  d'arrêt : CLEAN ou P2/P3 documentés — atteint).

Suites post-corrections : Docker sbfb-ci fmt 0 / clippy 0 / nextest
2044 (2043 verts + 1 flaky de charge `boot_path_reenters_sync_set_...`
**requalifié PASS solo 6s** — classe Phase C « dial 2-nœuds perdable
sous pic CPU », le run tournait en parallèle du bloc Windows ; test
intouché par le diff) / doctests 8 ok. Windows : fmt 0 / clippy 0 /
doctests 8 ok / release OK ; le nextest workspace a subi une erreur
d'infra de compilation (`simple_dns` rlib — contention cargo avec les
runs ciblés lancés par Codex sur le même target dir) → re-run complet
relancé, résultat consigné au body avant commit.

**Security delta** : aucun — doc-comments + 1 test pur non-routable,
0 store, 0 wire, 0 dep, 0 code prod fonctionnel.

---

**CONCERN** : phase cohérente, vérifiée driver-side (2 plateformes vertes,
+1 exact), minimale par conception, split E' respecté. 0 P0. **1 P1
trivialement corrigeable** (`gossip.rs:747` crate misattribution — à
corriger avant commit) + 6 P3 (documentables/carry/laisser). Une fois
E-DOC-1 corrigé, le gate Codex peut promouvoir en `## Verdict: PASS`.
