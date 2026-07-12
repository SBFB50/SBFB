# Sprint 81 Phase G — Review (Workflow ultracode + agent de synthèse)

> Phase G « CI / MSRV / convergence crypto + docs sécurité » (nom canonique,
> regex README §4 `Phase [A-Z]+[0-9]?`). Contrat = `sprint81_phase_g_preflight.md`
> (verdict **PLAN-ADAPT**, LE RÉFÉRENTIEL) : le plan écrit lui-même la
> bifurcation « flip `deny.toml:107` **OU** lever P2-AUDIT-2-RESIDUEL » ; le lock
> committé (`70dd845`) NE converge PAS (deux arbres `ed25519-dalek`
> 2.2.0 + 3.0.0-rc.0 + deux pré-versions `-rc`), donc la seule branche mécaniquement
> accessible est **CARRY P2-AUDIT-2-RESIDUEL (S82)** — exactement la branche SINON
> déléguée au préflight. Périmètre FERMÉ §4 = 12 livrables + carries E2/E3/F, tous
> docs + config supply-chain, **0 code**, **delta tests attendu = 0**.
>
> Working tree SALE, tip `70dd845` (Phase F), RIEN n'est stagé. Synthèse de **5
> dimensions** (D1 diff ligne-à-ligne / D2 contrat préflight / D3 sécurité deep /
> D4 invariants+patterns+langue+discipline / D5 cohérence inter-docs) + vérifications
> adversariales — **toutes re-vérifiées à la source sur disque** par cette synthèse
> (fichier:ligne + `git diff` + code cité par claim).
>
> **Suites §7.4 en cours côté main thread** (Rust Win complet + web complet en
> background ; Docker suivra ; `cargo deny check` COMPLET vérifié main thread =
> advisories/bans/licenses/sources ok). Cette review est **conditionnelle** à ces
> suites : le main thread les réconcilie au body AVANT toute promotion. Delta tests
> attendu = 0 (config + docs uniquement).

## 1. Périmètre et staging

`git status --short` + `git diff --name-only HEAD` = **EXACTEMENT** les 8 fichiers
attendus, 0 fichier parasite, 0 fichier hors-scope :

- `M Cargo.lock` (+4/−4) — **EXACTEMENT** `anyhow 1.0.102→1.0.103` (`:195`,
  version+checksum) + `crossbeam-epoch 0.9.18→0.9.20` (`:1371`, version+checksum) ;
  **AUCUN** bloc `[[package]]` ajouté ou retiré → 0 dep runtime neuve, 2 `cargo update`
  semver-compat.
- `M Cargo.toml` — commentaire des pins iroh rafraîchi (1.0.2 existe mais pin
  ENCORE `ed25519-dalek "=3.0.0-rc.0"` → n'unblock pas la convergence, pin `=1.0.1`
  gardé par choix) + tripwire « do NOT disable default features » (iroh-docs 0.101
  redb-v2-migration). `rust-version` non touché (le hunk démarre L30 ; `:24 = "1.91"`
  landé Phase B `c899d54`).
- `M deny.toml` (+38/−12) — 6 ignore-with-reason ajoutés (0119 hickory-proto +
  0098/0099/0104 rustls-webpki + 0194/0195 quick-xml) + ignore rand RUSTSEC-2026-0097
  RETIRÉ + `[bans] multiple-versions` reste `warn` (commentaire actualisé : flip
  BLOQUÉ → P2-AUDIT-2-RESIDUEL S82).
- `M docs/release/STORE_MIGRATION_OPS.md` — micro-adaptation fraîcheur : snapshot Mac
  `PENDING → PRIS 2026-07-08` (chemin + contenu vérifié).
- `M docs/security/EXTERNAL_AUDIT_SCOPE.md` — §2.4 table portée à =1.0.1/0.101/0.103
  (+ iroh-docs ajouté) + note R-iroh-audit P0 reconfirmée VERBATIM ; §2.7 checklist
  rejouée + replay S81 consigné avec preuve `cargo tree`.
- `M docs/security/HARDENING_ROADMAP.md` — `last_validated 2026-06-03→2026-07-08` +
  trigger « iroh > 0.98 » FIRED remplacé par trigger re-armé (breaking >1.0.x /
  iroh-docs 0.102+ / yank ed25519-dalek 3.0.0-rc.0) + entrée `audited_findings` S81.
- `M docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` — front-matter `last_validated`
  bumpé + §3.1 : 2 routes T0 lecture (GET /api/git/diff, GET /api/gates) + double
  transport bearer + PTY live (terminal/ws).
- `M docs/security/THREAT_MODEL.md` (+117) — §1.1 (:22) stack versionnée ; §5.4 row E
  (:195) 0.98→=1.0.1 + note upgrade≠audit ; §14 (:818) nit S80-H-4 ; **§15.4 NOUVEAU**
  (zéro-n0 + hot-join E3 + stores F) ; §17 v15.
- `?? .planning/active/sprint81_phase_g_preflight.md` — contrat, ne se review pas
  lui-même (à stager avec la phase).

`git diff --name-only` = uniquement `.md`/`.toml`/`.lock` → **0 `.rs`/`.ts`/`.tsx`,
0 fichier de test, delta tests = 0 par construction**, conforme au périmètre
supply-chain+docs. Le piège S2-11 (carry front `ed00b4a` / `GateIssueView`) est ÉVITÉ
(0 fichier `web/`, 0 hit `GateIssueView`).

## 2. Cohérence de l'arbitrage hickory ignore+carry (jugé à la demande)

**Le code a pris `ignore+carry` (HICKORY-024-RUSTSEC → S82) là où la review F
routait « bump 0.24→0.26 en G ». Cet arbitrage est COHÉRENT et DÉFENDABLE**, pour
quatre raisons vérifiées :

1. **Le préflight G a explicitement rouvert les deux branches** (« fix 0.24→0.26 **OU**
   ignore+carry », §4 livrable 4 + S1b tableau) — le choix n'est pas un dérapage
   silencieux, il est dans le mandat de la phase.
2. **Discipline mono-axe Day-0 #10 « iroh STRICTEMENT SEUL » (anti-bundle,
   bisectabilité).** Un bump majeur hickory 0.24→0.26 charrie du churn API réel dans
   `dns_fallback.rs` (construction du resolver + types de config réécrits en 0.25) —
   c'est du code non-supply-chain qui casserait la nature « 0 code » de G et la
   bisectabilité du sprint iroh-only.
3. **Exposition bornée vérifiée au code** (pas un band-aid) : `DnsFallbackConfig`
   default `enabled: false` (`dns_fallback.rs:127`), gate env `SBFB_DNS_FALLBACK_ENABLED`
   (`:67`) ; hickory confiné à `dns_fallback.rs` ; le quorum pkarr prod
   (`dht_quorum.rs`) utilise des clients pkarr-relay HTTP, **PAS** hickory ; les octets
   DNS sont des paquets pkarr Ed25519-vérifiés en aval. La classe résiduelle SBFB est
   DoS/nulle même si l'advisory intrinsèque de 0098/0099 est intégrité (cf. finding
   G-D3-1).
4. **Divergence documentée** (deny.toml reasons + HARDENING `audited_findings` +
   carry HICKORY-024-RUSTSEC groupé avec P2-AUDIT-2-RESIDUEL).

Verdict de cohérence : **RETENU**. La chaîne d'atteinte est vérifiée par `cargo tree`
— `rustls-webpki 0.101.7` tiré EXCLUSIVEMENT par `hickory-resolver 0.24 ← nexus-core-rs`
(via `rustls 0.21.12`), `quick-xml 0.39.4` tiré EXCLUSIVEMENT par
`iroh 1.0.1 → netwatch → netdev → plist` (macOS-gated). Aucune sous-déclaration de
surface.

## 3. Synthèse par dimension

- **D1 — Exactitude factuelle du diff.** Diff globalement fidèle : lock = les 2 seuls
  `cargo update` documentés, versions consignées (EXTERNAL §2.7/§2.4, THREAT §1.1/§5.4)
  matchent le lock, non-convergence ed25519/curve25519 réelle (`cargo tree -d` ambigu →
  CARRY correct), claims code de §15.4/LOOPBACK vérifiés un-à-un (fail-loud, chokepoint
  zéro-n0, gossip sans verbe leave, garde migration aux 2 boundaries, rename-FIRST,
  routes/PTY/cookie/Sec-Fetch-Site/GateStatus 5-valeurs). **UN défaut bloquant (P1)** :
  le claim d'auth SSE est INVERSÉ dans THREAT_MODEL §14 ET LOOPBACK §3.1 (cf. G-D1-1).
  Sinon 1 P3 (mis-citation `pkarr_resolver.rs`).
- **D2 — Contrat préflight (complétude + zéro sur-scope).** 8 fichiers = mapping 1:1
  sur les 12 livrables §4 + carries E2/E3/F, 0 fichier hors-périmètre, 0 code (delta 0
  par construction). Les 7 adaptations §5(c) toutes honorées ; piège S2-11 évité ;
  arbitrage hickory cohérent (§2). Résiduels de complétion mineurs non-bloquants : 4e
  foyer R9/D8 = commit body (attendu absent en review pré-commit, G-D2-1).
- **D3 — Sécurité deep (honnêteté des risques + zones interdites).** 0 zone rouge
  rouverte (warrant canary / guardrails / capability / pkarr ops / loopback code — 0
  `.rs` au diff). Les 6 ignores NE sous-déclarent PAS le risque réel (atteinte
  reached-only-via vérifiée). §15.4 : sévérités défendables et auto-cohérentes (SPOF
  H→M, jointure M→M, silent-loss M→L fail-loud, forge H→Nil Ed25519). R9/D8 verbatim
  aux 3 foyers doc. **UN défaut d'honnêteté (P2)** : le commentaire-groupe deny.toml
  qualifie TOUT le groupe hickory de « DoS-class » alors que 0098/0099 sont une laxité
  de validation de certs (classe intégrité) (G-D3-1). Les reasons per-advisory
  0098/0099, elles, sont honnêtes.
- **D4 — Invariants + patterns + langue + discipline.** 0 bump wire (aucun `*_VERSION`
  / `DOMAIN_*_V1` touché) + 0 dep runtime neuve tenus par construction ; §6.12
  N-A-no-new-frontier tenu (les 2 routes LOOPBACK = REVALIDATION de surfaces S80, pas
  code neuf) ; §17 exemption §7/§8 légitime (G = docs+config, pas de mitigation code) ;
  §15.4 unique et bien placée avant §16 ; langue par section-hôte respectée ; cible
  re-pointée du commentaire deny.toml (`audited_findings`) existe et est peuplée.
- **D5 — Cohérence inter-docs + honnêteté globale.** Les 5 docs + Cargo.toml + le T2
  JSON racontent UNE histoire cohérente (versions iroh identiques partout,
  P2-AUDIT-2-RESIDUEL / HICKORY-024-RUSTSEC nommés à l'identique, cross-ref réciproque
  IROH_SELFHOST_OPS §8 ↔ THREAT_MODEL §15.4, §17 v15 sans sur/sous-claim). 0 défaut
  bloquant. Une dérive PRÉ-EXISTANTE HORS périmètre fermé (`VALIDATED_BLUEPRINT.md`
  toujours « iroh 0.97 ») → carry (G-D5-1).

## 4. Findings confirmés (severité calibrée + disposition)

| id | dim | sév | disposition | claim (re-vérifié à la source) |
|---|---|---|---|---|
| **G-D1-1** | D1 | **P1** | **FIX-IN-PHASE (2 docs) avant commit** | **Claim d'auth SSE INVERSÉ.** THREAT_MODEL §14 (`:822-825`) et LOOPBACK §3.1 (bloc double-transport neuf) affirment que le SSE `fetch`+`ReadableStream` **POSE** le header `x-sbfb-token` (« ce chemin POSE le header » / « c'est le chemin du front Operator, y compris pour le SSE »). **FAUX, contredit par le code cité** : `useTokenStream.ts:134-138` envoie UNIQUEMENT `credentials:'same-origin'` + `accept` (jamais `x-sbfb-token`), commentaire `:9-10` « the HttpOnly cookie rides automatically » ; `auth.rs:53-54` groupe explicitement « **SSE/WS cannot set the `x-sbfb-token` header** » — le front navigateur ride le cookie HttpOnly pour TOUT (API, SSE, WS). Le transport header (essayé D'ABORD, `auth.rs`) est le chemin NON-navigateur (CLI, scripts d'acceptance, proxy Vite). |
| **G-D3-1** | D3 | **P2** | **FIX-IN-PHASE (deny.toml)** | **Commentaire-groupe deny.toml mislabel.** Le commentaire de groupe hickory affirme « every advisory in this group is DoS-class (no integrity/confidentiality impact) ». VRAI pour 0104 (panic CRL) + 0119 (O(n²) DoS) ; **FAUX pour 0098/0099** = laxité d'enforcement de name-constraints (URI/wildcard) → classe intégrité/authentification, pas DoS (vérifié contre l'advisory-db locale : 0098/0099 sans catégorie `denial-of-service`). Le résiduel SBFB reste borné DoS/nul (opt-in default-off + TLS validé + pkarr Ed25519 aval), mais la phrase de résumé sur-généralise dans un fichier orienté-auditeur (dimension D3/R9/D8). Les reasons per-advisory 0098/0099 (« name-constraint laxity », sans claim DoS) sont, elles, honnêtes. |
| **G-D1-2** | D1 | **P3** | body / optionnel | **§15.4 mis-cite `pkarr_resolver.rs`.** La row S dit « verification cote resolveur (`pkarr_resolver.rs`) » ; or ce module est le QuorumResolver canary S19 qui, par doc-comment (`:22-24`), NE parse PAS le `SignedPacket` (délègue à iroh), et le chemin zéro-n0 passe par `iroh::…::PkarrResolver` (`node.rs:508`), pas par `pkarr_resolver.rs`. Propriété de fond CORRECTE (paquet Ed25519-signé, forge impossible) ; seul le pointeur fichier est double-imprécis. Fix optionnel : citer le modèle de confiance pkarr comme §15.3, ou pointer `PkarrResolver` iroh. |
| **G-D2-1** | D2 | **P3** | **commit body G** | **4e foyer R9/D8 encore à écrire.** Le préflight §7 exige le libellé « upgrade ≠ Gate 1/Gate 3, R-iroh-audit P0 INCHANGÉ, pilote reste ferme » en 4 foyers ; (a) EXTERNAL_AUDIT_SCOPE §2.4, (b) THREAT_MODEL §5.4+§17 v15, (c) HARDENING (verbatim dans `last_validated`) PRÉSENTS ; (d) = commit body, ATTENDU absent en review pré-commit. Rappel de complétion, pas un défaut du diff. Le commiteur DOIT poser le libellé dans le body Phase G. |
| **G-D3-2** | D3 | **P3** | optionnel | **reason hickory « operator-configured » imprécis.** Le commentaire dit « only talks to operator-configured DoH/DoT endpoints » ; en réalité `DnsFallbackConfig::default()` code en dur Cloudflare/Google (`dns_fallback.rs:46-53,124-155`) et `load_dns_fallback_from_env` ne lit que enabled+domain. Imprécision dans le sens SÛR (des resolvers réputés épinglés = argument d'exposition PLUS fort). Fix : « pinned reputable DoH/DoT resolvers (Cloudflare/Google) ». |
| **G-D5-1** | D5 | **P3** | **carry (hors périmètre G)** | **`VALIDATED_BLUEPRINT.md` contredit l'upgrade.** `:156-157` disent encore « iroh 0.97 (1.0 pas encore) … SBFB deja pinne 0.97 » ; pin réel = `=1.0.1`. Dérive PRÉ-EXISTANTE (disait 0.97 alors que SBFB était sur 0.98 AVANT S81), fichier NON dans le périmètre §4 FERMÉ → correctement PAS un bloqueur G. Nit d'hygiène inter-doc → carry. |

Findings réfutés : **aucun**.

## 5. Checks passés (échantillon load-bearing, re-vérifiés sur disque)

- Cargo.lock diff = EXACTEMENT anyhow 1.0.103 + crossbeam-epoch 0.9.20, 0 `[[package]]`
  ajouté/retiré ; `cargo tree -d` ed25519-dalek + curve25519-dalek AMBIGUS → flip
  mécaniquement impossible → CARRY correct.
- deny.toml : `[bans] multiple-versions` reste `warn` ; P2-AUDIT-2 JAMAIS déclaré CLOSED
  (seules occurrences = `P2-AUDIT-2-RESIDUEL`) ; retrait ignore rand RUSTSEC-2026-0097
  cohérent (rand 0.8.6 hors plage révisée, exemption upgrade rand 0.9 intacte).
- reached-only-via vérifié : rustls-webpki 0.101.7 ← hickory-resolver 0.24 SEUL ;
  quick-xml ← iroh 1.0.1 → plist SEUL (macOS-gated).
- §15.4 fail-loud (`discovery_override.rs:142,161`), chokepoint prod unique
  `apply_zero_n0_discovery` (`node.rs`, preset Minimal, N0 défaut inchangé), gossip
  Command = Broadcast/BroadcastNeighbors/JoinPeers sans verbe leave, garde
  `refuse_recreate_on_interrupted_migration` aux 2 boundaries, backup rename-FIRST.
- LOOPBACK §3.1 : routes GET /api/git/diff / /api/gates / terminal/ws existent ; PTY réel
  (openpty + spawn + write stdin) ; cookie `sbfb_operator` + Sec-Fetch-Site + bootstrap
  GET /?token + Set-Cookie HttpOnly;SameSite=Strict ; GateStatus = 5 variantes.
- Front SSE = `fetch` + `getReader()` (ReadableStream), JAMAIS EventSource — partie
  « fetch+ReadableStream jamais EventSource » VRAIE (seule la sous-clause « POSE le
  header » est fausse, G-D1-1).
- EXTERNAL §2.7 replay : aes-gcm 0.10.3, frost-ed25519 3.0.0, iroh 1.0.1, blobs 0.103.0,
  gossip/docs 0.101.0 == lock ; §2.4 note R-iroh-audit P0 reconfirmée verbatim.
- 0 bump wire (21 `*_VERSION` + 24 `DOMAIN_*_V1` intacts), 0 dep runtime neuve,
  no-reopen SATISFAIT (aucun warrant canary/guardrails/capability/PKARR_RELAY_OPS
  touché), N-A-no-new-frontier tenu, langue par section-hôte respectée.

## 6. Dispositions pour le main thread

1. **[BLOQUANT — FIX-IN-PHASE avant commit] Corriger le claim d'auth SSE inversé
   (G-D1-1) dans LES DEUX docs.** Formulation correcte, identique dans THREAT_MODEL §14
   et LOOPBACK §3.1 : le front navigateur s'authentifie via le **cookie**, y compris
   pour le SSE ; le header `x-sbfb-token` est le **transport non-navigateur** (CLI /
   scripts / proxy Vite server-to-server) ; la protection CSRF du GET SSE est donc
   `SameSite=Strict` + la garde cookie-path `Sec-Fetch-Site: same-origin`, PAS une
   immunité par en-tête. C'est LA correction phare que la phase visait (nit S80-H-4) —
   la livrer inversée dans deux docs orientés-auditeur sous-déclare la surface que
   Sec-Fetch-Site/SameSite doivent couvrir.
2. **[RECOMMANDÉ FORTEMENT — FIX-IN-PHASE] Reformuler le commentaire-groupe deny.toml
   (G-D3-1)** pour séparer la classe intrinsèque de l'advisory du résiduel SBFB :
   « 0119/0104 sont DoS-class ; 0098/0099 sont des faiblesses d'enforcement de
   name-constraints de certificats (classe intégrité/authentification, pas DoS per
   RUSTSEC). Le résiduel SBFB reste néanmoins borné DoS/nul car le fallback est opt-in
   default-off, ne dialogue qu'avec des endpoints TLS-validés, et les octets DNS sont
   des paquets pkarr vérifiés Ed25519 de bout en bout en aval ; exploiter 0098/0099
   exige de surcroît une mis-émission CA et n'est atteignable qu'après vérification de
   signature. » P2 non-bloquant au sens P0/P1, mais la phase est PRÉCISÉMENT la
   dimension honnêteté supply-chain (D3/R9/D8) → à corriger avant commit.
3. **[commit body] Poser le libellé R9/D8 verbatim dans le body Phase G (4e foyer,
   G-D2-1)** : « upgrade ≠ Gate 1 / Gate 3, R-iroh-audit P0 INCHANGÉ, pilote reste
   ferme ».
4. **[optionnel P3]** : (a) corriger le pointeur `pkarr_resolver.rs` en §15.4 (G-D1-2) ;
   (b) « operator-configured » → « pinned reputable resolvers (Cloudflare/Google) » dans
   la reason hickory (G-D3-2).
5. **[carry, hors périmètre G]** : mettre à jour `VALIDATED_BLUEPRINT.md:156-157`
   (iroh 0.97→=1.0.1) dans un lot d'hygiène doc ultérieur (G-D5-1).
6. **Réconcilier les suites §7.4 au body** (Rust Win complet + Docker sbfb-ci + web +
   operator ; `cargo deny check` complet vérifié main thread) — **delta tests = 0**
   attendu. La review est conditionnelle à ces suites : les réconcilier AVANT toute
   promotion.
7. **Gate Codex** `codex exec -o` (output brut, jamais réécrit ; critère d'arrêt =
   « CLEAN ou P2/P3 documentés »). La promotion du verdict à PASS est réservée au main
   thread APRÈS application des fix-in-phase (1-2), réconciliation des suites, et Codex.

## 7. Gates + invariants + rappel décisions

- **P2-AUDIT-2-RESIDUEL (carry S82)** : le lock ne converge PAS (2 arbres ed25519 +
  2 `-rc`) → RÉSIDUEL, jamais CLOSED ; `deny.toml multiple-versions` reste `warn` ; le
  déblocage dépend d'une release upstream iroh repassant ed25519-dalek 3.x du RC au
  stable (le bump vers 1.0.2 NE résout PAS — RC identique).
- **HICKORY-024-RUSTSEC (carry S82)** : ignore+carry adopté (vs bump 0.24→0.26),
  cohérent avec Day-0 #10 « iroh STRICTEMENT SEUL » + churn API `dns_fallback.rs` +
  nature 0-code de G + exposition bornée opt-in default-off DoS-class (§2). Groupé avec
  P2-AUDIT-2-RESIDUEL.
- **R9/D8** : libellé verbatim présent aux 3 foyers doc (EXTERNAL §2.4 corps verbatim,
  THREAT §5.4+§17 v15, HARDENING `last_validated`) ; foyer (d) = commit body à poser
  (G-D2-1). ABSENT de warrant-canary/guardrails/capability (fichiers non touchés) →
  no-reopen respecté.
- **0 bump wire, 0 dep runtime neuve, R-iroh-audit P0 INCHANGÉ, pilote reste ferme.**
  N-A-no-new-frontier tenu (Track K docs-contrat non déclenché par G).
- **TOOLCHAIN-LABEL = statu quo** (aucun rust-toolchain.toml ; arbitre fmt/clippy =
  Docker `rust:1.94` SHA-pin ; Win flottante ≥1.95 ; GHA @stable ; MSRV 1.91 plancher)
  — décision consignée HARDENING.

## Codex reconciliation

Rapport Codex GPT 5.5 lu (`sprint81_phase_g_codex_review.md`, output
brut `codex exec -o`, jamais réécrit — round 1, 3e lancement : les 2
premiers runs background ont été tués env [contention lock cargo
package-cache au 1er], le run foreground a abouti) : **9 livrables — 8
CONFIRMÉ / 0 GAP / 1 PARTIEL**.

- **PARTIEL livrable 6 (HARDENING_ROADMAP:28)** : l'entrée
  `audited_findings` groupait les 4 advisories hickory en
  « DoS-class », contredisant la précision deny.toml (0098/0099 =
  laxité name-constraints, classe authentification) — l'incohérence
  inter-doc résiduelle du finding review G-D3-1 (fixé dans deny.toml
  seulement). Sévérité P3 doc-cohérence → **FIXÉ in-phase**
  (classes intrinsèques séparées, miroir exact de deny.toml) +
  documenté au commit body. Critère d'arrêt boucle « CLEAN ou P2/P3
  documentés » : ATTEINT (fix .md pur, suites non invalidées).

Dispositions review appliquées AVANT Codex : **G-D1-1 (P1) FIXÉ**
(claim auth SSE remis à l'endroit dans THREAT_MODEL §14 + LOOPBACK
§3.1 : le front navigateur ride le cookie pour TOUT y compris le SSE
`fetch`+`ReadableStream` ; header = transport non-navigateur ; CSRF
cookie-path = SameSite=Strict + `Sec-Fetch-Site`) ; **G-D3-1 (P2)
FIXÉ** (deny.toml classes intrinsèques séparées du résiduel borné) ;
**G-D1-2 + G-D3-2 (P3 optionnels) FIXÉS** (pointeur `PkarrResolver`
iroh en §15.4 ; « pinned reputable resolvers Cloudflare/Google ») ;
G-D2-1 → libellé R9/D8 posé au commit body (4e foyer) ; G-D5-1
(`VALIDATED_BLUEPRINT.md` iroh 0.97 stale, pré-existant hors
périmètre) → **carry** lot hygiène doc (audit S81/Phase K).

Suites §7.4 réconciliées sur le diff FINAL (fixes = .md + deny.toml,
sans impact compile ; `cargo deny check advisories` re-vert
post-fixes) : Rust Win fmt/clippy/nextest **2056/2056 0-skip**/
doctests/release VERTS ; Docker canonique `sbfb-ci` rust:1.94
fmt/clippy/nextest **2060/2060 0-skip**/doctests VERTS (1 échec
fail-fast `e2e start_writes_running_json_and_responds_to_health`
requalifié classe env Docker-on-Windows : solo 0.148s PASS +
workspace complet re-run PASS) ; web complet VERT (lint/tsc/Vitest
411/coverage 79.01-86.02-88.59/build/size/scan — flake
GpuConsentDialog requalifié solo 17/17, classe `vitest_env_variance`
déjà documentée au run F) ; operator complet VERT (Vitest 201/
6 gates/size) ; `cargo deny check` 4 catégories OK. **Delta tests =
0** (aucun fichier de test touché), conforme au contrat.

## Verdict: PASS
