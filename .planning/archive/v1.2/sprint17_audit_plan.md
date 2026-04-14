# Sprint 17 — Audit plan pour Sprint 18 Phase 0

**Ecrit** : 2026-04-14 (Phase F wrap-up)
**Commit stack a auditer** : `297fd50` (Phase A) + `c275ebd`
(Phase B) + `7dea299` (Phase C) + `872f48a` (Phase D) + `721686c`
(VALIDATED_BLUEPRINT) + `<this wrap-up>`

---

## Mode d'emploi pour la session fraiche

1. Lire dans l'ordre :
   - memory (`MEMORY.md`, `nexus_grid_pivot.md`,
     `sprint_audit_gate.md`, `feedback_approach.md`)
   - `git log --oneline d18e19e..HEAD` (le range Sprint 17)
   - `.planning/archive/v1.2/sprint17_kickoff.md` (D1..D5 gelees,
     **NE PAS rebattre**)
   - `.planning/archive/v1.2/sprint17_plan.md`
   - `.planning/archive/v1.2/sprint17_verification.md`
   - **ce document**
2. **NE PAS lire** les docs/security/ livres en entier avant
   d'avoir forme une opinion track par track. Ces docs captent la
   narration livreur — l'audit doit **challenger** pas confirmer.
3. Timebox suggere : **2-3h**.
4. Livrable : `.planning/active/sprint17_audit_findings.md`
   (session fraiche qui audite produit le doc, meme layout que
   `sprint16_audit_findings.md`). Au demarrage Sprint 18, les 5
   docs S17 + `sprint17_audit_findings.md` seront migres en
   `archive/v1.2/` via `git mv` si besoin (normalement ils sont
   deja dans archive via le wrap-up).
5. Commits fix eventuels (P0/P1) doivent atterrir avant le
   premier commit Sprint 18 Phase A. Format
   `fix(sprint17): <track>-P<n> — <short>`.

---

## Scope auditable

Sprint 17 livre **6 documents security** (0 code, 0 test) +
1 wrap-up :

| Surface | Phase | Livrable |
|---|---|---|
| Adversary taxonomy + scenarios | A `297fd50` | `docs/security/ADVERSARIES.md` + 6 `adversaries/T{0-5}.md` + `ATTACK_SCENARIOS.md` |
| P2P attack surface | B `c275ebd` | `docs/security/P2P_THREATS.md` |
| GPU compute threats | C `7dea299` | `docs/security/COMPUTE_THREATS.md` |
| Hardening roadmap | D `872f48a` | `docs/security/HARDENING_ROADMAP.md` |
| Validated long-term blueprint | Bonus `721686c` | `docs/security/VALIDATED_BLUEPRINT.md` + README racine pointers + README security index |
| Wrap-up (close + scope-cut + migrate) | F `<this>` | verification + audit plan + updates CLAUDE/SPRINT_LOG/memory + migration active -> archive |

Chacun a son track ci-dessous (A-E) plus un track meta (docs
coherence + scope cut Phase E legitime).

---

## Track A — Adversary taxonomy T0-T5 (Phase A)

**Question centrale** : la taxonomie T0-T5 est-elle coherente
(sans overlap, sans gap), les 12 scenarios sont-ils realistes et
ancres dans l'etat SBFB actuel (pas fictifs) ?

**Methodes** :

1. Grep `docs/security/ADVERSARIES.md` : verifier que chaque tier
   (T0-T5) a un persona distinct, un budget chiffre, un timeline,
   une motivation, et une fiche detaillee dans
   `adversaries/T{n}-*.md`.
2. Lire chaque fiche `T0-curious-user.md` a `T5-state-targeted.md` :
   verifier que les capabilities claimees sont **attestees par
   references externes** (Pegasus victims, Cellebrite UFED leaks,
   IMSI catchers documentes).
3. `ATTACK_SCENARIOS.md` : pour chaque scenario 1-12, verifier
   que :
   - chain d'attaque est concrete (pas abstraite "l'attaquant
     compromet")
   - mitigation status reference **code reel SBFB** (pas
     hypothetique)
   - tier assigne (T1-T5) est coherent avec fiche adversary
4. Cross-check : table §2 (tiers synthetique) coherente avec
   table §3 (mapping tier -> app risk gate) coherent avec Gate
   mapping dans `HARDENING_ROADMAP.md §7`.

**P0/P1 probable si** : un tier sans persona, un scenario sans
mitigation status, une incoherence gate-mapping.

---

## Track B — P2P threats coverage (Phase B)

**Question centrale** : les 7 vecteurs reseau (Sybil, Eclipse,
gossip, DHT, BGP/relay, traffic analysis, ISP block) couvrent-ils
bien la surface P2P actuelle de SBFB, ou existe-t-il des vecteurs
non-traites (ex: amplification DDoS, NTP-style reflection) ?

**Methodes** :

1. Grep `docs/security/P2P_THREATS.md` §8 synthese : verifier que
   chaque vecteur a un row dans le tableau T1-T5.
2. Verifier que chaque vecteur a :
   - Definition academique + reference paper (Heilman 2015 Bitcoin
     Eclipse, Douceur 2002 Sybil, etc.)
   - Etat SBFB actuel (concret, pas abstrait)
   - Attack scenarios contextualises
   - Mitigation options avec sequencing Sprint X
   - Coverage verdict (❌/⚠️/✅) coherent avec etat S16
3. Comparer vs threat models de projets comparables :
   - libp2p threat model (IPFS docs)
   - Tor threat model (torspec)
   - Bitcoin P2P threat model (Heilman, Decker)
   - Nym threat model
   Identifier vecteur eventuellement manque.
4. Verifier que chaque ❌/⚠️ de §8 est trace vers un item
   `HARDENING_ROADMAP.md` matrix avec sprint cible.

**P0/P1 probable si** : vecteur majeur absent (ex: eclipse-by-BGP
oublie), coverage verdict optimiste vs code reel.

---

## Track C — Compute threats coverage (Phase C)

**Question centrale** : les 7 classes menace GPU compute-sharing
(prompt leak, spoof, theft, extract, inject, side-channel, DoS)
sont-elles couvertes avec references academiques recentes, et
les mitigation sequencing sont-ils coherent avec l'etat S16
consent + caps ?

**Methodes** :

1. Grep `docs/security/COMPUTE_THREATS.md` §8 synthese : verifier
   chaque classe a row T1-T5.
2. Verifier chaque classe a :
   - Definition + references 2020-2026 (Carlini 2021 extracting
     training data, Kirchenbauer 2023 watermark, LeftoverLocals
     CVE-2023-4969, GPUHammer 2025, etc.)
   - Etat SBFB actuel post-S16 (consent + caps worker-side)
   - Attack scenarios + impact level
   - Mitigation options avec sequencing (ephemeral workers,
     redundancy voting, rate limit, TEE)
3. Cross-check avec
   [`VALIDATED_BLUEPRINT.md` couche 6](../../docs/security/VALIDATED_BLUEPRINT.md#couche-6--compute-tee-attested)
   : alignement mitigations + bricks TEE (sev 7.1, tdx-guest,
   nvml-wrapper) avec classes menace.
4. Verifier references academiques : year + venue + paper title
   complet pour chacune.

**P0/P1 probable si** : classe oubliee (ex: FHE leakage
side-channel non-mentionne), reference fabriquee, mitigation
sequencing incoherent avec roadmap D.

---

## Track D — Hardening roadmap coherence (Phase D)

**Question centrale** : la matrix 27 threats + roadmap S18-30
sont-ils implementables dans la capacite reelle du projet
(solo dev), et les gates 1-4 unlocking sequence sont-ils
logiquement coherents ?

**Methodes** :

1. Compter effort total S18-30 : sommer LOC estimees par sprint
   (~22100 LOC + ~650 tests), comparer a la capacite historique
   SBFB (sprints 0-16 moyenne ~1500 LOC + ~50 tests par sprint).
2. Verifier que chaque item matrix §1 est trace vers au moins un
   sprint roadmap §3.
3. Verifier dependency graph §6 :
   - Sybil resistance precede rate-limit mature (sinon rate-limit
     contournable)
   - Multi-relai federation precede warrant canary + pluggable
     transports
   - Encryption at rest precede Keychain/DPAPI
4. Gates sequencing §7 : verifier coherence
   - Gate 1 fin S18 : items S18 suffisent-ils ? (quick wins +
     supply chain baseline + multi-relai phase 1)
   - Gate 2 fin S22 : encryption at rest (S20) + rate-limit (S21)
     + Sybil base (S22) + supply chain (S18) = defendable ?
   - Gate 3 effectif S29 : post-audit externe required —
     clause "ship-blocker ethique" bien documentee ?
   - Gate 4 ~S35-38 : clause non-code (partnerships + beta ferme
     + ethics review board) bien explicite comme blocker ?
5. Quick-wins §4 + big-rocks §5 coherence vs roadmap §3 : items
   cites dans quick-wins apparaissent-ils effectivement dans
   sprint cible ?

**P0/P1 probable si** : sprint roadmap orphelin (item matrix sans
sprint), dependency graph incoherent, gate unlocking trop
optimiste (ex: Gate 3 sans audit), effort total S18-30 manifestement
supra-capacite solo (sans partnership / funding).

---

## Track E — VALIDATED_BLUEPRINT briques validees (commit bonus `721686c`)

**Question centrale** : les 50+ briques OSS citees sont-elles
correctement validees contre docs 2026, les 3 zones rouges
identifiees (wasmtime CVE, libp2p-gossipsub CVE, libcrux semantic
gaps) sont-elles ancrees dans des sources verifiables ?

**Methodes** :

1. Pour chaque brique citee GO/CAUTION/REPLACE dans
   `VALIDATED_BLUEPRINT.md`, verifier :
   - Version 2026 coherente avec crates.io / docs.rs current state
   - License declaree coherente avec repo upstream
   - Claim "production ready / prod-tested" ancre dans reference
     externe
   - CVE history referencee (RUSTSEC / GitHub Security Advisories)
2. Verifier les **3 zones rouges** :
   - Wasmtime CVE avril 2026 : advisories Bytecode Alliance
     retrouvables ? (CVE-2026-34971, CVE-2026-34945)
   - libp2p-gossipsub CVE-2026-33040 + CVE-2026-34219 : github
     security advisories visibles ?
   - Symbolic Software 7 avril 2026 hax/libcrux semantic gaps :
     URL du blog post accessible ?
3. Verifier les **8 briques ajoutees** post-validation (aws-lc-rs,
   gotatun, ring, zeroize, secrecy, sntrup761x25519, Creusot 0.9,
   Kani 0.66) : chacune effectivement mature 2026 ?
4. Verifier les **9 briques retirees** : raisons documentees
   (zkgroup archive, hickory-dns not-recommended-prod, lyrebird no
   Rust port, etc.) ?
5. Cross-check position vs OSS state-of-the-art : claims "= Signal
   PQXDH" / "= Tor client" / "= Briar + VeraCrypt" defendables ?

**P0/P1 probable si** : brique citee inexistante / archivee,
version claimee superieure a release reel, CVE citee sans source,
claim "formally verified" sans nuance post-Symbolic Software
findings.

---

## Track F — Scope cut Phase E legitimite

**Question centrale** : le scope-cut Phase E (RELEASE_GATES.md +
PARTNERSHIPS.md + DISCLOSURE.md, ~750 LOC) est-il honnetement
justifie par redondance BLUEPRINT, ou masque-t-il un livrable
manque ?

**Methodes** :

1. Lire `sprint17_verification.md` §Scope-cut Phase E : les 3
   raisons invoquees (redondance BLUEPRINT, items restants
   ONG-facing, tradeoff cost/marginal) sont-elles chacune
   ancrees dans les docs livres ?
2. Cross-check `VALIDATED_BLUEPRINT.md` :
   - Gates 1-4 sequencing : bien couvert §7 roadmap + couche 8 +
     section position ?
   - Partnerships Amnesty/HRW/CPJ/EFF/Cure53/ToB mentionnes dans
     couche 10 Operational security ?
   - Disclosure pattern (security.txt + PGP + SLA + embargo)
     mentionne couche 10 ?
3. Lister **items Phase E restants non-redondants** : enforcement
   mechanism app-by-app formel, outreach template emails, SLA
   CVE workflow, audit vendor couts negocies. Sont-ils **vraiment
   reports a sprint OpSec dedie** ou a besoin urgent Sprint 18 ?
4. Verdict : scope-cut legitime ou rabattement paresseux ?

**P1 probable si** : item Phase E critique pour Sprint 18
(enforcement gates, policy disclosure) non couvert ailleurs et
report sans justificatif solide.

---

## Track G — Docs coherence globale + hygiene

**Question centrale** : tous les cross-references entre docs
security/ + README.md + CLAUDE.md + SPRINT_LOG.md sont-ils
coherents ?

**Methodes** :

1. Grep tous les liens `](docs/security/*.md)` et
   `](../../docs/security/*.md)` : chacun resoud vers un fichier
   existant ?
2. `docs/security/README.md` index : 9 docs listees (expected :
   README, THREAT_MODEL, RUNTIME_ISOLATION, ADVERSARIES,
   ATTACK_SCENARIOS, P2P_THREATS, COMPUTE_THREATS,
   HARDENING_ROADMAP, VALIDATED_BLUEPRINT) + dossier adversaries/.
3. `README.md` racine §Security : 8 pointeurs + RUNTIME_ISOLATION +
   VALIDATED_BLUEPRINT bien listes ?
4. `CLAUDE.md` §Etat actuel : Sprint 17 CLOSED avec commits listes
   A-D + VALIDATED_BLUEPRINT + scope-cut Phase E ?
5. `docs/claude/SPRINT_LOG.md` §v1.2 : row Sprint 17 DONE avec tip
   cloture + nb commits + docs livres ?
6. memory `nexus_grid_pivot.md` frontmatter description : tip
   sync avec HEAD post-wrap-up, compteurs tests inchanges, etat
   Sprint 17 CLOSED ?

**P2 probable si** : dead link, index README incomplet, memory
stale.

---

## Livrables audit S17 attendus

Session fraiche produit :

1. `.planning/active/sprint17_audit_findings.md` — findings P0-P3
   tracks A-G + verdict global PASS / CONDITIONAL PASS / FAIL
2. Commits eventuels `fix(sprint17): <track>-P<n> — <short>` si
   P0/P1 identifies
3. Si PASS : Sprint 18 peut demarrer (session suivante ouvre
   kickoff + plan S18)
4. Si CONDITIONAL PASS : fix P0/P1 atterrir, CONDITIONAL PASS
   leve, puis Sprint 18 demarre
5. Si FAIL : re-ouvrir Sprint 17 (rare, improbable pour sprint
   recherche pure)

---

## Timebox + output format

- Duree : **2-3h session fraiche**
- Format findings : tables tracks A-G avec columns
  `# | Severite | Track | Finding | Fix` (meme layout que
  `sprint15_audit_findings.md` + `sprint16_audit_findings.md`)
- Severite : P0 (blocker Sprint 18) / P1 (fix required before
  first S18 commit) / P2 (defer next audit) / P3 (note for posterity)
- Verdict final en bottom : PASS / CONDITIONAL PASS (listant
  P0/P1 requis) / FAIL (listant showstoppers)
