# Sprint 81 Phase G — Préflight G8 (Workflow ultracode) — CI / MSRV / convergence crypto + docs sécurité

> **Verdict : PLAN-ADAPT.** Le plan Phase G écrit lui-même la bifurcation décisive
> (« flip `deny.toml:107` **OU** lever P2-AUDIT-2-RESIDUEL », `sprint81_plan.md:309-311`).
> Les 5 scans + 14 vérifications adversariales (13 CONFIRMED / 1 REFUTED-corrigé) convergent,
> et **j'ai re-vérifié la preuve cardinale moi-même** (`cargo tree -d` sur le lock committé,
> tip `70dd845`). **La branche « flip » est mécaniquement inaccessible** : le lock ne converge PAS,
> donc la seule action correcte est **CARRY P2-AUDIT-2-RESIDUEL (S82)** — exactement la branche
> SINON que le plan délègue au préflight. Trois autres livrables sont déjà faits ou drifté par
> rapport au plan, ce qui force un PLAN-ADAPT chirurgical (aucun Day-0 contredit) :
>
> 1. **[DÉCIDEUR — flip IMPOSSIBLE] Deux critères du gate doublement faux.** Le lock a
>    `ed25519-dalek` **2.2.0** (SBFB security-critical, pin workspace `Cargo.toml:62 "2.1"`) **ET**
>    **3.0.0-rc.0** (iroh 1.0.1 interne), plus `curve25519-dalek` **4.1.3 ET 5.0.0-rc.0** — soit
>    **2 arbres `ed25519-dalek`** ET **2 pré-versions `-rc` dupliquées** (re-vérifié moi-même :
>    `Cargo.lock:2101/2117` + `:1484/1501`, `grep -cE '-rc' = 2`). Le critère plan « un seul arbre
>    `ed25519-dalek` + 0 `*-pre`/`*-rc` dupliqués » (`plan:309-310`) échoue **sur les deux axes**.
>    Au-delà : `cargo deny check bans` compte **72 groupes multi-version** (S1b), un flip
>    `warn→deny` rendrait la CI rouge sur des dizaines de doublons. **NE PAS flipper ; lever
>    P2-AUDIT-2-RESIDUEL (carry S82)** (R6/C7, `plan:324`).
> 2. **[RASSURANT — sécurité intacte] La crypto SBFB 2.x ne s'effondre PAS sur l'arbre RC iroh.**
>    `cargo tree -i` (S1b/S2/S4) : `ed25519-dalek 2.2.0` n'est tiré QUE par `nexus-core-rs`
>    (canary/curator/provenance/task) + `nexus-trace-core` ; le `3.0.0-rc.0` est **confiné** à
>    `iroh`/`iroh-base` 1.0.1. Les deux sous-arbres coexistent — **signatures SBFB sur ligne stable
>    auditée, jamais sur un RC**. Non-flip = churn de graphe, **PAS** régression sécurité.
> 3. **[DÉJÀ FAIT] `rust-version 1.85→1.91` (livrable 2) landé en Phase B `c899d54`.** `Cargo.toml:24
>    = "1.91"` (re-lu). Livrable G = **vérification + trace**, pas édition.
> 4. **[REFUTÉ — livrable (4) « verts » FAUX en l'état] `cargo deny check advisories` = ROUGE
>    (exit 1, 8 erreurs RUSTSEC + 1 ignore périmé).** Gate CI actif chaque push
>    (`supply-chain.yml`). C'est le vrai travail neuf de G : 2 `cargo update` triviaux + décision
>    hickory-resolver 0.24→0.26 (fix OU ignore+carry) + 2 ignore-with-reason quick-xml + nettoyage
>    ignore rand périmé. **Dépasse « 0 delta tests + docs » → adaptation de périmètre.**
> 5. **[ANCRES driftées] docs sécurité pas byte-exactes vs plan.** `THREAT_MODEL:22 = "blobs 0.97"`
>    (pas 0.98) ; `:128` = case DFD **sans version** ; `§15.x zéro-n0 N'EXISTE PAS` (créer §15.4,
>    §15.3 déjà dédoublé) ; nit `EventSource` vit à `THREAT_MODEL:816`, PAS dans LOOPBACK. Édits
>    adaptés en conséquence.
>
> **0 bump wire (21 `*_VERSION` + 24 `DOMAIN_*_V1` tous hors périmètre), 0 dep runtime neuve
> (2 `cargo update` semver-compat), R-iroh-audit P0 INCHANGÉ, pilote reste ferme.** Ce n'est PAS
> un DESIGN-CONFLICT : aucun livrable ne contredit une décision Day-0 (l'upgrade iroh 0.98→1.0 EST
> le mandat sanctionné de S81 ; le carry P2-AUDIT-2 est cohérent avec la pré-launch policy). Le plan
> lui-même écrit la bifurcation → **PLAN-ADAPT**.

---

## 1. Contexte + gates calendaires

**Phase G (plan `sprint81_plan.md:305-327`)** : verts dual-platform + **gate de convergence
supply-chain** (`cargo tree -d` → flip `deny.toml:107` OU carry) + amendements `docs/security/`.
Surfaces = supply-chain (`cargo tree -d` / `deny.toml`) + docs. Workspace. **Delta tests attendu = 0**
(gates supply-chain + docs). Le corps S81 restant après F : **G..K**.

**Gates C8 (rappel E2/E3, `nexus_grid_pivot` memory)** : bascule flotte **25/08**, plan B actif
**15/09**, EOL relais n0 **30/09**. Aujourd'hui **2026-07-08** → fenêtre ouverte. **Phase H
DÉBLOQUÉE** (gate C8 zéro-n0 SATISFAIT le 05/07 `a085853`, +27 j d'avance ; snapshot Mac PRIS
2026-07-08). G n'a **aucune pression bloquante** propre.

**Note process** : les carries « routés G » proviennent des bodies E2 (`82afd0b`), E3 (`e05338f`),
F (`70dd845`) + reviews `sprint81_phase_{e2,e3,f}_review.md`. Le sprint **réserve explicitement les
édits `THREAT_MODEL.md` à G** (constat F §8) → G est le point d'atterrissage doc de ces carries.
**Ne pas sur-scoper** G avec du code non-supply-chain (interdits R9/D8, cf. §7).

---

## 2. La preuve cardinale re-vérifiée par la synthèse (état réel du lock)

Quatre scans (S1b/S2/S4 + adversarial) ont dumpé le lock ; **je l'ai re-fait moi-même** (`cargo tree
-d --locked --offline` + `grep` direct de `Cargo.lock`, tip `70dd845`). Résultat identique et
non-ambigu :

| Crate | Versions dans le lock | Ancre `Cargo.lock` | Consommateur (cargo tree -i) |
|---|---|---|---|
| `ed25519-dalek` | **2.2.0** + **3.0.0-rc.0** | `:2101` / `:2117` | 2.2.0 = `nexus-core-rs`+`nexus-trace-core` (DIRECT) ; 3.0.0-rc.0 = `iroh`+`iroh-base` 1.0.1 |
| `curve25519-dalek` | **4.1.3** + **5.0.0-rc.0** | `:1484` / `:1501` | 4.1.3 = ligne SBFB stable ; 5.0.0-rc.0 = iroh |
| pré-versions `-rc` totales | **exactement 2** | `grep -cE '^version = "…-rc' = 2` | ed25519-dalek 3.0.0-rc.0 + curve25519-dalek 5.0.0-rc.0 |

**Commande de preuve (re-exécutée) :** `cargo tree -d --locked --offline -p ed25519-dalek` →
`error: specification 'ed25519-dalek' is ambiguous` listant `ed25519-dalek@2.2.0` **et**
`ed25519-dalek@3.0.0-rc.0`. Idem `curve25519-dalek@4.1.3` / `@5.0.0-rc.0`.

**Conséquences directes, chaînées :**

- **Le critère du gate de convergence (`plan:309-310`) est DOUBLEMENT faux** : ni « un seul arbre
  `ed25519-dalek` » (2 arbres) ni « 0 `*-rc` dupliqués » (2 pré-versions). Le flip
  `deny.toml:107 warn→deny` est **mécaniquement impossible** → **CARRY P2-AUDIT-2-RESIDUEL**.
- **La duplication RC est ENTIÈREMENT interne à iroh 1.0.1** — c'est la nature historique de
  P2-AUDIT-2 (« iroh pre-release transitives »), reportée de 0.98 sur 1.0.1. Ne se résout pas en G,
  ni par bump vers 1.0.2 (S1a : iroh 1.0.2 épingle **toujours** `ed25519-dalek = "=3.0.0-rc.0"`).
  **Dépend d'une release upstream d'iroh** qui repasserait ed25519-dalek 3.x du `-rc` au stable.
- **La crypto SBFB security-critical reste isolée sur 2.2.0/4.1.3** (frost-ed25519 3.0.0 →
  `nexus-shell-daemon-core` ; `ed25519-dalek = "2.1"` `Cargo.toml:62` → `nexus-core-rs`). Aucun
  chemin SBFB ne linke le RC. Le second contrôle du livrable (1) — « le 2.x SBFB ne s'effondre PAS
  sur l'arbre RC d'iroh » — **PASSE**.

---

## 3. Constats par scan (evidence fichier:ligne vérifiée) + arbitrage adversarial

### S1a — SOTA/upstream (crates.io + rustsec.org, 2026-07-08)

| # | Finding | Verdict adversarial | Arbitrage synthèse |
|---|---|---|---|
| 1 | iroh 1.0.1 **ET** 1.0.2 épinglent `ed25519-dalek = "=3.0.0-rc.0"` → flip impossible, lever P2-AUDIT-2-RESIDUEL | **CONFIRMED** (curl crates.io re-query + `Cargo.lock` local) | **RETENU** — bloqueur cardinal, cohérent §2. |
| 2 | Bump vers iroh 1.0.2 NE résout PAS la convergence (RC upstream dans iroh, pas un pin SBFB) | **CONFIRMED** | **RETENU** — carry S82 = vrai carry, dépend d'une release iroh future. |
| 3 | iroh 1.0.2 existe (2026-07-06) → commentaire `Cargo.toml:34-35` « no 1.0.2 exists » **PÉRIMÉ** ; mais 1.0.2 = patch **sans breaking** (rate-limit relay, fairness recv, regr. test Win, bench dns) → pin `=1.0.1` tenu (R6/C7 OK) | **CONFIRMED** (crates.io max_stable=1.0.2 + release notes) | **RETENU** — livrable (3)/veille : rafraîchir le commentaire OU acter pin `=1.0.1` par choix (reproductibilité), MAJ optionnelle. |
| 4 | Veille iroh-docs 0.102+ **NON fired** : docs/gossip=0.101.0, blobs=0.103.0 == pins SBFB `Cargo.toml:43-45` | **CONFIRMED** (crates.io max_stable) | **RETENU** — trigger reste ARMÉ (seuil : iroh-docs 0.102+ OU yank ed25519-dalek 3.0.0-rc.0). |
| 5 | ed25519-dalek 3.0.0 **STABLE** publiée 2026-07-06 ; 3.0.0-rc.0 **NON yankée** | **CONFIRMED** | **RETENU** — nouveau trigger de veille : yank éventuel du rc.0 → `yanked="deny"` (`deny.toml:45`) casserait le pin exact d'iroh. |
| 6 | RustSEC upstream propre : 0 advisory iroh/blobs/gossip/docs/redb/pkarr ; ed25519(≥2.0)/curve25519(≥4.1.3)/sha2 patchés | **REFUTED partiel** (sha2 patché ≥**0.9.8** pas ≥0.9.6 ; conclusion sécurité intacte) | **CORRIGÉ** : libeller « sha2 ≥0.9.8 » ; SBFB sur 0.10.9+0.11.0 → sains. **404-page = preuve-du-négatif, autorité = S1b lock réel.** |
| 7 | `rust-version 1.91` déjà fait Phase B `c899d54` | **CONFIRMED** | **RETENU** — cf. §4 livrable (2). |

### S1b — deps/CVE réelles du workspace (lecture seule, `--locked`, cargo-deny 0.19.2)

| # | Finding | Verdict adversarial | Arbitrage synthèse |
|---|---|---|---|
| F1 | **`cargo deny check advisories` ÉCHOUE (exit 1) — 8 erreurs RUSTSEC + 1 ignore périmé** → livrable (4) « verts » **REFUTÉ en l'état** | **CONFIRMED** (re-run indépendant) | **RETENU — DOMINANT**, cf. §4 livrable (4). |
| F2 | Flip multiple-versions non viable (72 groupes) → carry P2-AUDIT-2-RESIDUEL | **CONFIRMED** (`cargo deny check bans` = 72, termine « bans ok » en warn) | **RETENU** — cohérent §2. |
| F3 | Crypto SBFB (ed25519 2.2.0, curve25519 4.1.3, frost 3.0.0, x509-parser 0.17.0) **ISOLÉE** de l'arbre RC | **CONFIRMED** (`cargo tree -i`) | **RETENU** — non-flip = churn, pas régression. |
| F4 | `rust-version 1.91` posé Phase B `c899d54` (hunk `-1.85`/`+1.91`) | **CONFIRMED** (`git show c899d54`) | **RETENU**. |
| F5 | Lock a dérivé depuis l'artefact Phase B (sha1 +1, x509-parser +1, iroh stack) | **CONFIRMED** | **RETENU** — artefact `sprint81_phase_b_cargo_tree_d.txt` périmé, regénérer si référencé §2.7. |
| F6 | `cargo-audit` **ABSENT** localement ; gate advisory canonique = `cargo-deny` (subsume RustSec) | **CONFIRMED** | **RETENU** — reformuler livrable (4) « cargo-deny check advisories vert » (pas de gate cargo-audit distinct). |

**Détail des 8 erreurs RUSTSEC (S1b, autoritatif sur le lock) :**

| RUSTSEC | Crate lock | Classe | Fixabilité |
|---|---|---|---|
| 2026-0190 | anyhow 1.0.102 | unsound | **cargo update → 1.0.103** (dry-run OK, 0 code) |
| 2026-0204 | crossbeam-epoch 0.9.18 | vuln | **cargo update → 0.9.20** (dry-run OK, 0 code) |
| 2026-0119 | hickory-proto 0.24.4 | vuln (O(n²) name compression) | **enraciné pin SBFB `hickory-resolver="0.24"` `Cargo.toml:443`** → bump 0.24→0.26 OU ignore+carry ; iroh tourne déjà 0.26.1 |
| 2026-0098/0099/0104 | rustls-webpki 0.101.7 | vuln ×3 | idem racine hickory 0.24 (via rustls 0.21.12) — non atteignable par cargo update |
| 2026-0194/0195 | quick-xml 0.39.4 | vuln ×2 (DoS) | **enraciné iroh `=1.0.1`** (quick-xml←plist←netdev←netwatch←iroh) → **ignore+carry upstream** |
| 2026-0097 (warning) | rand 0.8 (ignore périmé) | doc-hygiène | ignore `deny.toml:64` « no crate matched » → nettoyer/mettre à jour |

### S2 — décisions historiques traversées

| # | Finding | Verdict | Arbitrage |
|---|---|---|---|
| 1 | Lock ne converge PAS (2 arbres ed25519 + curve25519 5.0.0-rc.0) → flip interdit, carry S82 | CONFIRMED | **RETENU** (§2). |
| 2 | 2.x SBFB isolée du RC (sous-arbres disjoints) | CONFIRMED | **RETENU**. **NIT** : le plan cite `Cargo.toml:58` pour ed25519-dalek, l'ancre réelle = **`:62`** (`:58` = commentaire serde_jcs). |
| 3 | `rust-version 1.91` déjà Phase B `c899d54` | CONFIRMED | **RETENU**. |
| 4 | LOT-LOOPBACK-DOC libellés exacts S80 H-1/2/3/4 (`sprint80_audit_findings.md:251-282,405-408`) | CONFIRMED | **RETENU** — cf. §4 livrable (6). |
| 5 | TOOLCHAIN-LABEL (S80-A-2, `:61-68,409-410`) : pas de rust-toolchain.toml aujourd'hui | CONFIRMED | **RETENU** — décision §5(b). |
| 6 | Carries E2 réservés G (THREAT_MODEL §15.x + SPOF/≥2 relais pkarr distincts + silent-loss + hickory-0119 + T20 PinValidator) | CONFIRMED | **RETENU** (§6). |
| 7 | Carries E3 réservés G (unsubscribe sans verbe leave + boot-duress élargi + reconnexion-après-drop ; + **S75 re-drive boot-SEED OVERDUE 3/3 NON fermé**) | CONFIRMED | **RETENU** (§6). |
| 8 | Carries F réservés G (T-STORE-MIGRATION-CRASHWINDOW + T-STORE-FIXTURE-LEAK + T-BLOBS-DURABILITY + correction kickoff C4/C5 + bump hickory) | CONFIRMED | **RETENU** (§6). |
| 9 | Re-décision Topologie A-vs-B **avant 25/08** (PO 2026-07-05 `bf07960` = B co-logée 0 €) → G **TRACE**, ne tranche pas | CONFIRMED | **RETENU** — décision PO ouverte à consigner §15.x. |
| 10 | `deny.toml` = 1 seul commit (`d7ab281` S18) ; tightening « Sprint 19+ » jamais fait → flip serait le 1er changement | CONFIRMED | **RETENU** — fichier reste inchangé (flip bloqué), commentaire actualisable. |
| 11 | **PIÈGE** : `sprint81_audit_plan.md:78` « Phase G (carry `ed00b4a`) » = **front S80 Phase G** (GateIssueView), **HORS périmètre** S81-G | CONFIRMED (`git show -s ed00b4a`) | **RETENU** — NE PAS absorber. |
| 12 | Cibles docs (THREAT_MODEL:22/128/195, EXTERNAL_AUDIT_SCOPE §2.4/2.7, HARDENING:5) existent + encore STALE (0.97/0.98) — non pré-éditées | CONFIRMED | **RETENU** (§4 livrable 5/6). |

### S3 — couverture threat model

| # | Finding | Verdict | Arbitrage |
|---|---|---|---|
| 1 | `THREAT_MODEL:22 = "blobs 0.97"` (pas 0.98) — ancre driftée sur le corps de version | CONFIRMED (re-lu) | **RETENU** — édit `:22` = **0.97→=1.0.1** (rattrapage). |
| 2 | `THREAT_MODEL:128` = case DFD `iroh QUIC (ChaCha20-Poly1305 + Ed25519)` **sans version** | CONFIRMED (re-lu) | **RETENU** — **no-op version** sur `:128`. |
| 3 | `THREAT_MODEL:195` = « Version pinnee 0.98 … + cargo-audit \| M » — **seul 0.98 byte-exact**, résiduel M | CONFIRMED (re-lu) | **RETENU** — édit `:195` = **0.98→=1.0.1**, GARDER M, rationale wire-freeze, foyer R9/D8. |
| 4 | `§15.x zéro-n0 N'EXISTE PAS` ; §15.3 déjà **dédoublé** (dashboard S76 `:1025` + keepalive WAN S77 `:1054`) | CONFIRMED | **RETENU** — **créer §15.4** (éviter §15.3). |
| 5 | `EXTERNAL_AUDIT_SCOPE §2.4:82-84` STALE (gossip 0.97 / blobs 0.99 / pkarr 0.97) | CONFIRMED (re-lu) | **RETENU** — porter à =1.0.1 / 0.101 / 0.103 (+ iroh-docs 0.101 absent). |
| 6 | Note R-iroh-audit P0 `§2.4:94-97` — **reconfirmer VERBATIM** (seule la version change) | CONFIRMED (re-lu) | **RETENU** — foyer canonique R9/D8. |
| 7 | `§2.7:128-136` checklist `cargo tree -p iroh/iroh-blobs/ed25519-dalek` — **rejouer** (+ iroh-gossip/docs) | CONFIRMED | **RETENU** — preuve de convergence documentée. |
| 8 | `HARDENING_ROADMAP:5` trigger « iroh release > 0.98 » **FIRED** ; `:3` last_validated 2026-06-03 à bumper ; note « iroh 1.0.0-rc.0 defere » obsolète | CONFIRMED (re-lu) | **RETENU** — bump + audited_findings S81. |
| 9 | **NO-REOPEN SATISFAIT** : aucun trigger CAPABILITY/GUARDRAILS/WARRANT_CANARY/LOOPBACK-own/PKARR ne réfère iroh ; seul HARDENING fire ; IROH_SELFHOST_OPS déjà à =1.0.1 (écrit 05/07) | CONFIRMED | **RETENU** (§7). |
| 10 | **NIT §14 EventSource MISATTRIBUÉ** : vit à `THREAT_MODEL:816` (pas LOOPBACK qui finit à §9 `:267`) ; front S80 = fetch+ReadableStream | CONFIRMED | **RETENU** — corriger à `THREAT_MODEL:816` (H-4). |
| 11 | `§17` évolution log — dernière v14 (S77 K `:1487`) → entrée **v15** | CONFIRMED | **RETENU** — preuve de revalidation (pas de front-matter triggers sur ces 2 docs). |

### S4 — invariants wire format + build

| # | Finding | Verdict | Arbitrage |
|---|---|---|---|
| 1 | Phase G ne touche **AUCUN** wire (21 `*_FORMAT/ANNOUNCEMENT/SCHEMA_VERSION` + 24 `DOMAIN_*_V1` tous hors périmètre) | CONFIRMED | **RETENU** — 0 bump wire par construction. |
| 2 | Convergence échoue → carry P2-AUDIT-2-RESIDUEL | CONFIRMED | **RETENU** (§2). |
| 3 | 2.x SBFB ne s'effondre pas sur le RC iroh (crypto signée intacte) | CONFIRMED | **RETENU**. |
| 4 | `rust-version` = **métadonnée MSRV** (plancher build), 0 effet wire/runtime, hérité par 12 crates ; png-to-icns n'hérite pas, web=JS | CONFIRMED | **RETENU** — livrable (2) = vérif sémantique. |
| 5 | `deny.toml` = cargo-deny only (0 runtime) ; flip casserait la CI (base64/bitflags/reqwest/ed25519 dupliqués) | CONFIRMED | **RETENU**. |
| 6 | TOOLCHAIN-LABEL : aucun rust-toolchain.toml ; Win 1.95 / GHA @stable / Docker 1.94 / MSRV 1.91 → **reco statu quo** | CONFIRMED | **RETENU** — décision §5(b). |
| 7 | **N-A-no-new-frontier** (§6.12) : livrables = config build + prose docs, aucune API loopback/wire neuve ; LOT-LOOPBACK = **revalidation** de frontières S80 existantes | CONFIRMED | **RETENU** — Track K docs-contrat **non déclenché** par G. |
| 8 | Nit `THREAT_MODEL:22 = 0.97` | CONFIRMED | **RETENU** (miroir S3-1). |

---

## 4. Périmètre contractuel de la phase (liste FERMÉE + état réel constaté)

Livrables du plan `sprint81_plan.md:305-327` + carries réservés G (S2), chacun avec son état réel :

| # | Livrable (plan) | État réel constaté | Action G |
|---|---|---|---|
| **(1a)** | `cargo tree -d` gate de convergence → flip `deny.toml:107` **OU** carry | **Lock NE CONVERGE PAS** (2 arbres ed25519 + 2 `-rc`, §2) | **CARRY P2-AUDIT-2-RESIDUEL (S82)** — branche SINON. `deny.toml:107` reste `warn`. Actualiser le commentaire « Sprint 19+ » → « reporté S82, bloqué par arbre RC iroh 1.0.1 ». |
| **(1b)** | Vérif que `ed25519-dalek 2.x` SBFB ne s'effondre PAS sur l'arbre RC iroh (plan cite `Cargo.toml:58`) | **PASSE** (2.2.0 isolé, §2/S1b-F3). Ancre réelle = **`Cargo.toml:62`** (pas `:58`) | **CONSTATER** + corriger le repère de ligne. |
| **(2)** | `rust-version 1.85→1.91` (`Cargo.toml:24`, D6) | **DÉJÀ FAIT Phase B `c899d54`** (`:24 = "1.91"`, re-lu) | **VÉRIFICATION + TRACE**, pas d'édit. |
| **(3)** | Trigger de veille iroh-docs 0.102+ | **NON fired** (docs/gossip 0.101, blobs 0.103 == pins) ; **nouveau** : commentaire `Cargo.toml:34-35` « no 1.0.2 exists » PÉRIMÉ (1.0.2 = patch 2026-07-06) | **ARMER/documenter** : seuil iroh-docs 0.102+ **OU** yank ed25519-dalek 3.0.0-rc.0. Optionnel : rafraîchir le commentaire OU acter pin `=1.0.1` par choix. |
| **(4)** | `cargo-deny` / `cargo-audit` verts | **REFUTÉ** : `cargo deny check advisories` = exit 1, **8 erreurs RUSTSEC + 1 ignore périmé** ; `cargo-audit` **absent** (subsumé par cargo-deny) | **ADAPTATION** : 2 `cargo update` (anyhow→1.0.103, crossbeam-epoch→0.9.20) + **décision hickory** (fix 0.24→0.26 OU ignore+carry) + 2 ignore-with-reason quick-xml (upstream iroh) + nettoyer ignore rand 0097. Reformuler « cargo-deny check advisories vert ». |
| **(5)** | Amendements `THREAT_MODEL.md:22,128,195` (0.98→1.0.1 + rationale wire-freeze, M reste) | `:22 = 0.97` (pas 0.98) ; `:128` **sans version** ; `:195 = 0.98` (byte-exact) | **ÉDIT ADAPTÉ** : `:22` 0.97→=1.0.1 ; `:128` **no-op version** ; `:195` 0.98→=1.0.1 + M + rationale + R9/D8. |
| **(5b)** | `EXTERNAL_AUDIT_SCOPE §2.4/§2.7` (R-iroh-audit verbatim + rejouer checklist) | `§2.4:82-84` STALE (0.97/0.99/0.97) ; note P0 `:94-97` VERBATIM ; `§2.7:128-136` checklist présente | **ÉDIT** : versions → =1.0.1/0.101/0.103 (+ iroh-docs) ; **note P0 verbatim** ; rejouer + consigner `cargo tree`. |
| **(5c)** | `HARDENING_ROADMAP.md:5` (trigger FIRED + bump last_validated) | trigger `:5` « iroh > 0.98 » FIRED ; `:3` last_validated 2026-06-03 ; note « 1.0.0-rc.0 defere » obsolète | **ÉDIT** : bump last_validated S81 + audited_findings + libellé R9/D8. |
| **(6)** | **LOT-LOOPBACK-DOC** (S80 H-1/2/3/4) : revalidation `LOOPBACK §3.1` (routes git/diff+gates + double transport cookie + PTY) + nit §14 EventSource + last_validated | `LOOPBACK §3.1:103-111` **manque** git/diff + gates ; `:110` terminal/ws = « lecture cast » STALE (S80 D = PTY live) ; cookie non décrit §3.1 ; last_validated `:3`=2026-06-03 ; **nit EventSource vit à `THREAT_MODEL:816`, PAS LOOPBACK** | **ÉDIT** : ajouter 2 routes T0 lecture-seule + double transport cookie HttpOnly + `?token` bootstrap + maj PTY ; bump last_validated ; corriger EventSource à `THREAT_MODEL:816`. |
| **(7)** | **TOOLCHAIN-LABEL** (S80-A-2) : décision pin rust-toolchain.toml ou statu quo | **Pas de rust-toolchain.toml** ; Win 1.95 / GHA @stable / Docker 1.94 / MSRV 1.91 | **DÉCISION §5(b) = statu quo** + corriger le label stale « Win 1.94 ». |
| **(8-E2)** | Carries E2 → THREAT_MODEL §15.x zéro-n0 | §15.x n'existe pas | **CRÉER §15.4** (relocation trust n0→opérateur bornée Ed25519 + SPOF ≥2 relais pkarr distincts + host relais ≠ host ancre + silent-loss élargie + T20 PinValidator carry + **TRACER re-décision Topologie A-vs-B avant 25/08**). |
| **(8-E3)** | Carries E3 → doc §15.x | — | **DOCUMENTER** : asymétrie unsubscribe (iroh-gossip 0.101 sans verbe `leave`) + boot-duress élargi + reconnexion-après-drop. **+ escalader S75 re-drive boot-SEED OVERDUE 3/3 (NON fermé)** : fermer explicitement ou re-justifier blocker externe. |
| **(8-F)** | Carries F → THREAT_MODEL/runbook | — | **DOCUMENTER** : T-STORE-MIGRATION-CRASHWINDOW (résidu L) + T-STORE-FIXTURE-LEAK + T-BLOBS-DURABILITY (§15=intégrité pas durabilité) + correction kickoff C4/C5 + tripwire feature. **hickory bump = livrable (4)**. |

**Périmètre FERMÉ.** Hors périmètre explicite : le carry front `ed00b4a` (GateIssueView, S2-11) ;
tout code non-supply-chain ; toute réouverture warrant canary / loopback (au-delà LOT) / guardrails /
capability toggles (R9/D8, §7).

---

## 5. Décisions du préflight

### (a) Flip `deny.toml` OU carry — **CARRY P2-AUDIT-2-RESIDUEL (S82)**

**Fait décisif (re-vérifié par moi, §2)** : le lock committé (`70dd845`) porte `ed25519-dalek` en
**2.2.0 ET 3.0.0-rc.0** (`Cargo.lock:2101/2117`) **et** `curve25519-dalek` en **4.1.3 ET 5.0.0-rc.0**
(`:1484/1501`) — **2 arbres ed25519 + 2 pré-versions `-rc` dupliquées** (grep `-rc` = 2). Les DEUX
conditions du gate (`plan:309-310`) échouent. `cargo deny check bans` = **72 groupes** multi-version.

→ **NE PAS flipper `deny.toml:107`** (reste `warn`). **Lever P2-AUDIT-2-RESIDUEL, carry S82.**
P2-AUDIT-2 **jamais marqué CLOSED** (R6/C7). Le carry dépend d'une release upstream d'iroh repassant
ed25519-dalek du `-rc` au stable (bump vers 1.0.2 NE résout PAS — RC identique, S1a-2). Actualiser le
commentaire `deny.toml:107` (« Sprint 19+ » → report S82 motivé par l'arbre RC iroh 1.0.1).

### (b) TOOLCHAIN-LABEL — **statu quo (Docker-canonique arbitre) + corriger le label stale**

**Constat** : aucun `rust-toolchain.toml` dans le repo ; Windows dev = rustc **1.95.0** (flottant) ;
GHA `rust-ci.yml`/`ci.yml` = `dtolnay/rust-toolchain@stable` (flottant → 1.95) ; Woodpecker canonique
= `rust:1.94` **SHA-pin** (`.woodpecker/ci-linux.yml`) ; MSRV déclarée = **1.91** (plancher ; les 3
surfaces ≥ 1.91, cohérent). `rust-version` = métadonnée MSRV, **0 effet wire/runtime** (S4-4).

**Décision = STATU QUO.** Rationale : (i) l'arbitre fmt/clippy **EST déjà** le Docker canonique
`rust:1.94` (règle process « Docker-canonique avant push », leçon S76) ; (ii) un pin
`rust-toolchain.toml=1.94` tuerait le drift fmt 1.95↔1.94 MAIS ajoute une friction réelle
(auto-download rustup pour contributeurs) + churn de bump à chaque release Rust + changerait le
comportement `@stable` en CI ; (iii) MSRV=plancher suffit à garantir la compilabilité. **Action** :
documenter le drift + **corriger le label factuellement faux** « Win 1.94 » (`verification.md:67`,
rustc local ≥ 1.95) en « toolchain locale non pinnée ; canonique = Docker `rust:1.94` ». Décision
informable PO (non bloquante ; aucun effet wire/runtime dans les deux options).

### (c) Adaptations PLAN-ADAPT (evidence concrète)

1. **Livrable (1) → branche CARRY** (au lieu du flip) — evidence §2 : lock non-convergent.
2. **Livrable (2) → vérification+trace** (au lieu d'édit) — evidence : `Cargo.toml:24="1.91"` posé
   `c899d54` Phase B (`git show`).
3. **Livrable (4) → +remédiation supply-chain** (au lieu de checkbox « verts ») — evidence :
   `cargo deny check advisories` exit 1, 8 RUSTSEC (S1b). **Le delta tests reste 0** (fixes = bumps
   de version + ignore-with-reason, pas de code testable neuf).
4. **Livrable (5) → édits adaptés** : `:22` 0.97→=1.0.1 (pas 0.98) ; `:128` no-op version (pas de
   0.98) ; `:195` 0.98→=1.0.1 — evidence : lecture directe des 3 lignes.
5. **Carry E2 → créer §15.4** (au lieu de « §15.x ») — evidence : §15.3 déjà dédoublé (`:1025`/`:1054`).
6. **Nit EventSource → `THREAT_MODEL:816`** (au lieu de « LOOPBACK §14 ») — evidence : LOOPBACK
   finit à §9 `:267`, `EventSource` n'apparaît qu'à `THREAT_MODEL:816`.
7. **Repère plan `Cargo.toml:58` → `:62`** pour le pin ed25519-dalek.

Aucune de ces adaptations ne contredit une décision gelée Day-0 (cf. §7). Toutes sont soit une
**branche déjà écrite par le plan** (1), soit un **fait déjà landé** (2), soit une **correction
d'ancre driftée** (4/5/6/7), soit une **remédiation supply-chain forcée par un gate CI rouge** (3).

---

## 6. Carries entrants réservés G (à documenter, pas coder)

**E2 (zéro-n0, `82afd0b` + `sprint81_phase_e2_review.md:236-260,283-288`)** :
- **THREAT_MODEL §15.4 (à CRÉER)** : surface zéro-n0 self-hosted — relocation trust+availability
  n0→opérateur **BORNÉE Ed25519** ; SPOF + jointure métadonnées-relais × contenu-ancre → exiger
  **≥2 relais pkarr DISTINCTS non-n0** + host relais ≠ host ancre ; silent-loss discovery **élargie**
  (1 host vs flotte n0 4 régions) ; **T20 PinValidator/tls_pinning** toujours carry (WebPKI-only,
  `insecure_skip_verify=#[cfg(test)]` only) ; D5-5 cross-ref `IROH_SELFHOST_OPS §7/§8`.
- **hickory-proto 0.24.4 RUSTSEC-2026-0119** = livrable (4) (bump 0.24→0.26 OU ignore+carry).
- **Re-décision Topologie A-vs-B avant 25/08** (PO 2026-07-05 `bf07960` = **B co-logée 0 €**) → G
  **TRACE la décision PO ouverte** dans §15.4 (SPOF opéré-hôte + jointure + QUIC addr discovery off
  en Topologie B), **ne tranche pas** (constat acceptance `a085853`).

**E3 (hot-join, `e05338f` + `sprint81_phase_e3_review.md:236-258`)** :
- **asymétrie unsubscribe** — iroh-gossip 0.101 n'expose **aucun verbe `leave`**
  (`Command` = Broadcast/BroadcastNeighbors/JoinPeers) → le pair reste voisin HyParView jusqu'au
  churn, ingest droppé par `is_subscribed=false`, fuite bornée au transport.
- **résidu boot-duress PRÉ-EXISTANT élargi** (dial subscribe + fetch repull + replay outbox sous clé
  leurre ; E3 ne l'aggrave PAS, hot-join duress-safe-par-placement).
- **reconnexion-après-drop** (join sans ajout au bootstrap-set du topic figé → re-bootstrap seulement
  au reboot).
- **[SÉPARÉ, à escalader]** carry **S75 re-drive-on-ingest boot-SEED driver OVERDUE 3/3 NON fermé** —
  distinct du défaut E3 → **fermer explicitement ou re-justifier blocker externe** à cet audit gate.

**F (redb migration, `70dd845` + `sprint81_phase_f_review.md:288-300`)** :
- **T-STORE-MIGRATION-CRASHWINDOW** (`migrate_redb_v2_tuples.rs:166↔167`) : fenêtre rename↔persist →
  store vide → recreate silencieux NON capté par A2 (garde `refuse_recreate_on_interrupted_migration`
  codée en F ferme le silent-loss si backup survit) ; mitigation tar snapshot (Win+Mac PRIS) +
  caveat Linux rename-clobber. **Résidu L.**
- **T-STORE-FIXTURE-LEAK** : migration crée `docs.redb.backup-redb-v2-tuples` + `docs.db.migrate<rand>`
  → nettoyage runbook.
- **T-BLOBS-DURABILITY (dégradé)** : wipe moot (blobs v3 ouvre in-place) → note « §15 = intégrité,
  pas durabilité » + contenu re-importable depuis `iroh/blobs/data/*.data` (BLAKE3).
- **correction kickoff C4/C5** (feature `redb-v2-migration` EXISTE + défaut) + tripwire « aucun
  `default-features=false` sur iroh-docs ».

---

## 7. Gates tenus (invariants R6/C7 + R9/D8 + no-reopen)

- **P2-AUDIT-2 jamais pré-clos** (R6/C7, `plan:324`) : le lock ne converge PAS (§2) → **RESIDUEL,
  carry S82, jamais CLOSED**.
- **Libellé obligatoire R9/D8** : **« upgrade ≠ Gate 1 / Gate 3, R-iroh-audit P0 INCHANGÉ, pilote
  reste ferme »** — à poser en 4 foyers : (a) `EXTERNAL_AUDIT_SCOPE §2.4:94-97` (note P0 verbatim) ;
  (b) `THREAT_MODEL §5.4:195` + `§17` v15 ; (c) `HARDENING_ROADMAP` audited_findings S81 ; (d) commit
  body G. **NE PAS** l'ajouter dans warrant canary / guardrails / capability toggles (hors scope).
- **Zones NON rouvertes** (S3-9, no-reopen SATISFAIT) : aucun trigger `CAPABILITY_TOGGLES` /
  `GUARDRAILS_ARCHITECTURE` / `WARRANT_CANARY_HARDENING` / `LOOPBACK`-own / `PKARR_RELAY_OPS` ne
  réfère iroh ; seul `HARDENING_ROADMAP:5` fire ; `IROH_SELFHOST_OPS` déjà à =1.0.1 (écrit 05/07,
  aucune revalidation due). LOT-LOOPBACK = **revalidation** de frontières S80 existantes, PAS
  réouverture de surface.
- **§6.12 : N-A-no-new-frontier** (S4-7) — livrables = config build (`deny.toml`, métadonnée
  `Cargo.toml`, éventuel `rust-toolchain.toml`) + prose docs. Aucune API loopback/wire lue par un
  runtime distinct. **Track K docs-contrat NON déclenché** par G.
- **0 bump wire** par construction (21 `*_VERSION` + 24 `DOMAIN_*_V1` hors périmètre, S4-1).
- **0 dep runtime neuve** : livrable (4) = 2 `cargo update` semver-compat (anyhow, crossbeam-epoch)
  + ignore-with-reason ; aucune dep ajoutée. `cargo-audit` non installé (subsumé).
- **Day-0 tenus** : l'upgrade iroh 0.98→1.0 EST le mandat sanctionné de S81 (memory
  `guardian_db_integration_eval` : « upgrade iroh 1.0 = GO conditionnel sprint dédié SEUL, relais N0
  EOL 2026-09-30 »). Le pin iroh reste `=` (reproductibilité, décision gelée). AGPL/loopback/curator
  crypto/blob-serve inchangés.

---

## VERDICT FINAL : **PLAN-ADAPT**

Phase G ne contredit **aucune** décision gelée Day-0 → **pas de DESIGN-CONFLICT / pas de STOP
arbitrage PO**. Elle prend la **branche déjà écrite par le plan** (« flip OU carry ») du côté
**CARRY P2-AUDIT-2-RESIDUEL**, contrainte par l'**état réel du lock re-vérifié par la synthèse**
(2 arbres `ed25519-dalek` 2.2.0/3.0.0-rc.0 + 2 pré-versions `-rc`, `Cargo.lock:2101/2117/1484/1501`),
et **adapte** quatre livrables sur des faits concrets :
(1) flip→carry (lock non-convergent, evidence §2) ;
(2) `rust-version 1.91` déjà landé Phase B `c899d54` → vérification, pas édit ;
(4) `cargo deny check advisories` **ROUGE** (8 RUSTSEC) → remédiation supply-chain (2 `cargo update`
+ décision hickory + ignore-with-reason quick-xml + nettoyage rand), **delta tests = 0** ;
(5/6) ancres docs driftées (`:22`=0.97 pas 0.98, `:128` sans version, §15.x inexistant→§15.4, nit
EventSource à `:816` pas LOOPBACK) → édits adaptés.
**TOOLCHAIN-LABEL = statu quo** (Docker `rust:1.94` arbitre fmt/clippy, MSRV 1.91 plancher, corriger
le label stale « Win 1.94 »). Gates R6/C7 + R9/D8 tenus, no-reopen SATISFAIT, N-A-no-new-frontier.
**0 bump wire, 0 dep runtime neuve, R-iroh-audit P0 inchangé, pilote reste ferme.** Le seul point
DESIGN-CONFLICT candidat (un livrable qui casserait un Day-0) n'existe pas : le carry P2-AUDIT-2 est
au contraire **exigé** par la pré-launch policy et le mandat même de S81.
