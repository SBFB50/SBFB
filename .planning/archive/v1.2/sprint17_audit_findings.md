# Sprint 17 — Audit findings (Sprint 18 Phase 0 gate)

**Auditeur** : Claude Code session fraiche (Sprint 18 Phase 0)
**Date** : 2026-04-14
**Tip audite** : `60b539a` (`chore(sprint17): close S17 + scope-cut Phase E + migrate plans`)
**Commit stack Sprint 17** :
- `f75b2c6` chore(planning) — close Sprint 16 + open Sprint 17
- `297fd50` Phase A — adversary taxonomy T0-T5 + 12 attack scenarios
- `c275ebd` Phase B — P2P attack surface deep-dive (7 vecteurs)
- `7dea299` Phase C — GPU compute sharing threats (7 classes)
- `872f48a` Phase D — gap analysis + hardening roadmap
- `721686c` bonus — VALIDATED_BLUEPRINT.md (13 couches long-terme)
- `60b539a` Phase F wrap-up — close + scope-cut Phase E + migrate plans

**Timebox observe** : ~2h (5 tracks A-E en parallele via agents + F/G sequentiels main thread).

---

## Verdict global : **CONDITIONAL PASS**

- **P0** : 0
- **P1** : 7 (G-1 dangling refs, D-1 Gate 3 table, A-1 mapping T4, A-2 terminologie, B-1 Sybil tier, C-1 Tramer/Carlini, E-1 libp2p-gossipsub)
- **P2** : 19 (tech debt docs, sequencing conflicts, refs faibles)
- **P3** : 13 (posterity notes, nits)

Sprint 18 Phase A peut demarrer apres un seul commit `fix(sprint17)` qui traite les 7 P1 ensemble (tous sont des editions de docs dans `docs/security/`, zero code touche). Les 19 P2 sont loggees comme dette a reprendre au fil des sprints S18-S30 (certaines se resoudront naturellement quand le code correspondant atterrira). Les 13 P3 restent tels quels.

**Condition d'unblock** : **1 seul commit docs** `fix(sprint17): audit-P1 — resolve 7 findings (scope-cut refs, Gate 3 clarif, tier Sybil reclass, academic refs)` avant `feat(sprint18): Phase A — ...`.

---

## Resume executif

Top 5 findings :

1. **P1-G1** — Le scope-cut Phase E acte dans `60b539a` laisse **12 cross-references dangling** vers `RELEASE_GATES.md` / `PARTNERSHIPS.md` / `DISCLOSURE.md` dans 5 docs livres (`ADVERSARIES.md`, `ATTACK_SCENARIOS.md`, `P2P_THREATS.md`, `COMPUTE_THREATS.md`, `HARDENING_ROADMAP.md`). Ces docs n'existent pas (scope-cut officialise) mais sont linked comme s'ils existaient. Dead-link volume significatif pour des docs fondateurs security. Fix recommande : creer 3 stubs courts qui expliquent le scope-cut et pointent vers les sources canoniques (HARDENING_ROADMAP §7 pour gates, VALIDATED_BLUEPRINT couche 10 pour partnerships + disclosure).

2. **P1-D1** — `HARDENING_ROADMAP.md §7` table donne "Gate 3 Sprint debloquant = **S27**" puis l'explication §7.472-474 dit "Gate 3 effectif = **fin S29**" (post-audit externe). Les deux sont vrais mais la table induit le lecteur en erreur — un solo dev qui lit la table saute l'explication et pense Gate 3 debloquable S27 alors que l'audit externe Cure53/ToB (S29) est le blocker effectif.

3. **P1-B1** — `P2P_THREATS.md §8` table synthese donne `B-Sybil` verdict Tier max = **T5** (state actor). C'est techniquement vrai comme plafond mais trompeur : un T2 criminel organise moderne (RaaS, botnets 10^4+ IP) peut exploiter Sybil sans PoW (cost-of-identity = 0 actuellement). `HARDENING_ROADMAP §1` classe aussi Tier max = T5 → une app Gate 2 (T0-T2) est vulnerable a Sybil pre-S19 PoW mais le mapping actuel suggere "seul T5 peut". Fix : reclass `B-Sybil` Tier max = T2+ pre-S19, T5 post-S19 PoW + S22 kudos-weighted.

4. **P1-C1** — `COMPUTE_THREATS.md §1.6` ligne 137 attribue paper "Stealing Part of a Production Language Model" a "Tramer et al. 2024" mais §4.6 ligne 421 attribue correctement a "Carlini et al. 2024". Le paper reel est Carlini 2024 (Tramer 2016 est foundational distinct "Stealing ML Models via Prediction APIs"). Correction 1-liner ligne 137.

5. **P1-E1** — `VALIDATED_BLUEPRINT.md` cite `libp2p-gossipsub` CVE-2026-33040 / CVE-2026-34219 comme zone rouge mais SBFB **n'utilise pas libp2p-gossipsub** — il utilise `iroh-gossip` 0.97 (stack iroh native). Cette confusion arch peut induire en erreur un lecteur qui pense le projet vulnerable. Fix : ajouter une phrase explicite "SBFB utilise `iroh-gossip`, pas `libp2p-gossipsub` — CVE n'affectent pas directement, mais documente pour futur iroh-gossip si pattern similaire".

Les 3 commits `fix(sprint17): audit-P1 — ...` proposes sont decrits en §Commits fix a lander.

---

## Track A — Adversary taxonomy T0-T5 + 12 scenarios — **CONDITIONAL PASS**

**Methode rollee** : opinion pre-lecture via grep code S16 (consent levels, bearer token, CSP, provenance SLSA L1) pour lister mitigations reellement implementees. Puis lecture `ADVERSARIES.md` (343 LOC) + 6 fiches T0-T5 + `ATTACK_SCENARIOS.md` (770 LOC). Cross-check §2 tiers synthetique vs §3 mapping tier→gate vs `HARDENING_ROADMAP §7`.

**Strengths** : taxonomie T0-T5 coherente, tiers distincts avec personas/budget/timeline chiffres, fiches detaillees riches, 12 scenarios avec chains concretes, mitigations status references code S16 reel (loopback `d7c265a`, consent `3247e88`, provenance `10bbc63`, etc.). Glossaire §4 tres bien fourni (0-day, dragnet, Pegasus, Cellebrite, side-channel).

| # | Sev | Track | Finding | Emplacement | Fix suggere |
|---|---|---|---|---|---|
| A-1 | P1 | A | Mapping §3 `Tier mitige jusqu'a → Gate min` omet T4 explicitement : T0-T3 → Gate 3, T0-T5 → Gate 4. T4 (state mass surveillance dragnet) est implicite dans Gate 4 sans intermediate. Un lecteur peut conclure "Gate 3 adresse T4" alors que T4 scenario 8 dragnet metadata est "Absent" S16 | `ADVERSARIES.md` §3 table lines 117-122 | Clarifier : "Gate 3 mitige T0-T3 + **partial T4 dragnet metadata** (Tor S25 + mixnet futur), Gate 4 mitige T0-T5 complet" |
| A-2 | P1 | A | Terminologie mitigation status inconsistante : `ATTACK_SCENARIOS.md` §6 scenario par scenario utilise "Couvert partial" / "Absent" / "Couvert" en prose, `HARDENING_ROADMAP §1` matrix utilise symboles `❌ Absent / ⚠️ Partiel / ✅ Couvert`. Le lecteur doit mapper lui-meme | `ATTACK_SCENARIOS.md` §1-12 `Current SBFB mitigation status` blocks | Standardiser sur 3 etats `❌ Absent` / `⚠️ Partiel` / `✅ Couvert` avec une ligne chacun dans chaque scenario, prefixe explicite au lieu de prose |
| A-3 | P2 | A | Capabilities T2-T5 citent Pegasus "~20+ gouvernements clients" + Cellebrite UFED "~10-20k$ unite" + XZ-utils backdoor "2024" **sans URLs sources** | `ADVERSARIES.md` lines 186-189, 196; `ATTACK_SCENARIOS.md` line 113 | Ajouter hyperlinks : Citizen Lab Pegasus reports, Cellebrite pricing EFF article, CVE-2024-3094 XZ |
| A-4 | P2 | A | Tier T1→T2 transition sans critere quantitatif formel — budget <1k$ vs 10-100k$ est une zone grise ("script kiddie qui achete 1 0-day = T2 ?") | `ADVERSARIES.md` §1.2 lines 84-86 | Ajouter : "T1→T2 = acquisition asset >5k$ (0-day, pentest team) OU coordination >50 personnes" |
| A-5 | P2 | A | Scenario 6 prerequisite cite "SBFB en beta fermee Gate 3 (PolitiScan annonce)" — PolitiScan est app hypothetique Sprint 30+, scenario sonne fictif | `ATTACK_SCENARIOS.md` §6 line 314 | Remplacer "PolitiScan" par "app hypothetique generic high-stakes journalistic" OU ajouter disclaimer "app de type PolitiScan" |
| A-6 | P3 | A | T5 "Fork Profile B" formulation `T5-state-targeted.md §10` suggere commitment futur — "Decision formelle Sprint 30+" sans gate approprie | `adversaries/T5-state-targeted.md` §10 | Rephrase : "Potential fork pour Gate 4 maximal si overhead UX inacceptable mainline" |
| A-7 | P3 | A | THREAT_MODEL.md cross-ref aux 12 scenarios manquant — scenarios devraient map a STRIDE findings S16 | `ATTACK_SCENARIOS.md` §1-12 header | Ajouter pour chaque scenario : "Derive de `THREAT_MODEL.md §5.X` STRIDE …" |

---

## Track B — P2P threats coverage — **CONDITIONAL PASS**

**Methode rollee** : opinion pre-lecture via litterature (Douceur 2002 Sybil, Heilman 2015 Eclipse, Urdaneta 2011 DHT survey, Bitcoin/Ethereum P2P threat models, Briar/Tor threat models) pour anticiper vecteurs attendus + potentiels oublies (amplification DDoS, NTP reflection, Eclipse-by-BGP cascade, SNI leak, CT gap). Puis lecture `P2P_THREATS.md` (843 LOC) + cross-check `HARDENING_ROADMAP §1` matrix.

**Strengths** : 7 vecteurs couverts solidement (Sybil, Eclipse, gossip, DHT, BGP, traffic analysis, ISP block), chaque section definition academique + refs papers + etat SBFB + scenarios + mitigation sequence avec sprint cible. Refs academiques majoritairement correctes (Heilman 2015, Douceur 2002).

| # | Sev | Track | Finding | Emplacement | Fix suggere |
|---|---|---|---|---|---|
| B-1 | P1 | B | `B-Sybil` tier max = T5 dans `§8` synthese et `HARDENING_ROADMAP §1`, mais Sybil pre-S19 (cost-of-identity zero, pas de PoW) est exploitable par T2+ criminals modernes (botnets RaaS 10^4+ IP). Un reader conclut "Sybil = state actor-only" — faux | `P2P_THREATS.md §8` + `HARDENING_ROADMAP §1` row `B-Sybil` | Reclass Tier max `B-Sybil` a **T2+ pre-S19**, **T5 post-S19 PoW + S22 kudos-weighted**. Ajouter note "Sybil pre-PoW = zero cost-of-identity, exploitable T2+ criminals modernes" |
| B-2 | P2 | B | Eclipse-by-BGP cascade non articule comme scenario composite. `§5` couvre BGP hijack metadata leak, `§2` couvre Eclipse via peer selection, mais BGP+Eclipse combine (attaquant controle AS → biais peer pool + censure) n'est pas trace | `P2P_THREATS.md §5` | Ajouter sous-scenario §5.3 "Eclipse-by-BGP : AS hijack + peer pool bias cascade" |
| B-3 | P2 | B | NoDHT poisoning record flooding mitige par Ed25519 sig (`§4.2`) mais **record flooding upstream storage exhaustion** (noie DHT pkarr → forcing fallback relais n0 unique → transforms en B-Eclipse) pas explicite | `P2P_THREATS.md §4.2-4.3` | Ajouter dependency explicite : "DHT flooding → forcing relay fallback → cascade en Eclipse si relais single-point" |
| B-4 | P2 | B | SNI leakage (pre-content metadata) non mitige. Relais `*.n0.computer` = ClientHello SNI broadcast fingerprintable meme avec payload encrypted | `P2P_THREATS.md §7` (DPI/DNS) | Ajouter mitigation option : "ECH (Encrypted ClientHello) sur relai TLS, depend TLS cert pinning S19" |
| B-5 | P2 | B | Certificate Transparency monitoring relay cert pas mentionne. Un state actor obtient CA signature via subpoena → MITM relay → client trust | `P2P_THREATS.md §5.4` | Ajouter "CT log monitoring per-relay-cert + warrant canary" S20 |
| B-6 | P2 | B | Gossip replay attack (ProjectAnnouncement outdated, no sequence numbers per-announce) non mentionne — minor severity (content immutable Ed25519) mais UX degrade possible | `P2P_THREATS.md §3.1` | Note sous `§3 Hors scope` ou `§3.1` : "Gossip replay = mitige par validation semantique app-level (repo_url live check)" |
| B-7 | P2 | B | NTP reflection attacks via DHT lookups (spoof source = victim IP → reply flood) : SBFB complicit meme si upstream iroh-pkarr | `P2P_THREATS.md §4.2-4.4` | Ajouter §4.4 mitigation : "Rate limit UDP reflection per-source-IP (iroh upstream PR)" |
| B-8 | P3 | B | Amplification fanout gossip iroh-gossip default ~8 → fanout^2 flood. Pas de cap dynamic per-peer malice score | `P2P_THREATS.md §3.1 + §3.4` | Clarifier fanout trade-off adversarial |
| B-9 | P3 | B | `HARDENING_ROADMAP §1` dit `B-BGP` app-risk C (critical), mais `VALIDATED_BLUEPRINT §5.6` RPKI/route monitoring listed comme "Med" optional — mismatch severity | `HARDENING_ROADMAP §1` vs `VALIDATED_BLUEPRINT Couche 5` | Aligner : RPKI monitoring mandatory Gate 2+ si B-BGP = critical |
| B-10 | P3 | B | PQC gap cross-ref absent : `§9 Hors scope` mentionne Ed25519 stable mais `VALIDATED_BLUEPRINT Couche 1` mandate ML-DSA-65 + ML-KEM-1024 hybrid post-2030 | `P2P_THREATS.md §9 Hors scope` | Footnote : "Ed25519 keys acceptable S18-25, migration hybrid Ed25519+ML-DSA-65 Sprint 26+ mandatory Gate 3+" |

---

## Track C — Compute threats coverage — **CONDITIONAL PASS**

**Methode rollee** : opinion pre-lecture via papers attendus 2020-2026 (Carlini 2021/2024, Kirchenbauer 2023, LeftoverLocals Chen 2023, GPUHammer Zhang 2023, Oswald/Jung, NVIDIA CVE history, Perez/Ribeiro prompt injection) + classes potentiellement oubliees (membership inference, data poisoning, FHE leakage, CUDA container escape, cold-boot VRAM extraction, trojaned weights). Puis lecture `COMPUTE_THREATS.md` (844 LOC) + code S16 (`consent.rs`, `runtime.rs`) + cross-check `VALIDATED_BLUEPRINT Couche 6 TEE`.

**Strengths** : 7 classes bien couverts (prompt leak, spoof, theft, extract, inject, side-channel, DoS), definitions precises, scenarios concrets, mitigations tabulees avec sprint cible, 13/14 refs academiques correctes, cross-ref TEE Couche 6 coherent.

| # | Sev | Track | Finding | Emplacement | Fix suggere |
|---|---|---|---|---|---|
| C-1 | P1 | C | Paper attribution conflict : `§1.6 line 137` = "Tramer et al. 2024, Stealing Part of a Production Language Model" vs `§4.6 line 421` = "Carlini et al. 2024, Stealing Part..." — meme titre, auteur primaire different. Paper reel = **Carlini 2024** (Tramer 2016 est distinct) | `COMPUTE_THREATS.md §1.6 line 137` | Corriger ligne 137 : `Carlini et al. 2024` |
| C-2 | P2 | C | LeftoverLocals venue "(Trail of Bits research)" imprecis — Trail of Bits est disclosure lab, pas venue academique. Paper reel cite USENIX 2024 ou CVE-2023-4969 direct | `COMPUTE_THREATS.md §6.1 line 558, §6.3 line 582, §6.6 line 643` | Clarifier : "(CVE-2023-4969, Trail of Bits disclosure + USENIX Security 2024)" |
| C-3 | P2 | C | Classes oubliees : **membership inference attack** sur LLM (subsumable §4 model extraction mais pas explicite), **trojaned weights** (poisoning amont au training), **CUDA container escape** post-CVE-2024-0126 pattern | `COMPUTE_THREATS.md §9 Hors scope` | Ajouter phrase : "Membership inference subsume §4 (meme primitive rate-limit + watermark). Trojaned weights amont au training hors-scope (worker charge modele externally). CUDA container escape = side-channel §6 variant" |
| C-4 | P3 | C | Sequencing conflict inter-sections : `§1.5` VRAM wipe Sprint 23 vs `§6.5` VRAM wipe Sprint 22, `§4.5` rate-limit Sprint 22 vs `§7.5` rate-limit-per-identity Sprint 21 | `COMPUTE_THREATS.md §1.5, §4.5, §6.5, §7.5` | Arbitrer via `HARDENING_ROADMAP §3` (Phase D source of truth) : VRAM wipe S22-23 (§6 wipe + §1 ephemeral workers), rate-limit primitive S21 core + S22 kudos-weighted |
| C-5 | P3 | C | Watermark Kirchenbauer 2023 ref correcte mais scalabilite LLM >70B (Llama 405B, Qwen 72B) non quantifiee empiriquement | `COMPUTE_THREATS.md §4.4 line 398` | Note posterity : "Overhead watermark sur models >70B non-publie, empirical test pre-S27 impl" |
| C-6 | P3 | C | Kudos-weighted priority queue (§7.4-7.5) assume kudos Sybil-resistant — mais P2P_THREATS §1 Sybil resistance status "indicative roadmap S18-30" → circularite acceptee (deja doc §7.733) | `COMPUTE_THREATS.md §7.2-7.5` | Posterity note : "Fallback S23 rate-limit + cooldown independent kudos si Sybil delayed" |

---

## Track D — Hardening roadmap coherence — **CONDITIONAL PASS**

**Methode rollee** : capacite historique sprints 0-16 (moyenne ~1500 LOC + ~50 tests/sprint) + invariants P2P (Sybil → rate-limit, multi-relai → transport hardening, encryption → Keychain/DPAPI) + gates logic (audit externe requis Gate 3). Lecture `HARDENING_ROADMAP.md` (499 LOC) + cross-check matrix §1 vs roadmap §3 vs gates §7.

**Strengths** : matrix 27 threats complete (traces A-S1..A-S12 + B-Sybil..B-ISPBlock + C-PromptLeak..C-DosFlood), dependency graph §6 solide, quick-wins §4 + big-rocks §5 tracables, Gate 4 ~S35-38 correctement decline (non-code items decalent).

| # | Sev | Track | Finding | Emplacement | Fix suggere |
|---|---|---|---|---|---|
| D-1 | P1 | D | `§7` table donne "Gate 3 Sprint debloquant = **S27**" puis §7.472-474 explication dit "Gate 3 effectif = **S29** (post-audit externe)". La table seule trompe | `HARDENING_ROADMAP.md §7` table line 469 | Modifier colonne "Sprint debloquant" : `Gate 3` → "**S29** (tech S27 + audit externe S29)" pour que la table soit self-consistent |
| D-2 | P2 | D | `§5 Big-rocks` encryption at rest : lib cross-platform `keyring-rs` incomplete vs wrapping platform-specific non-decidee — blocker potentiel S20 kickoff si 3-4 jours tech spike non-alloues | `§5 + §6 invariants line 449-451` | Declarer D1 Sprint 20 kickoff futur : "keyring-rs 3.x fork adapte ou wrapping platform-native (Keychain/DPAPI/libsecret)" |
| D-3 | P2 | D | `§3 Sprint 21` rate-limit depend `S19 PoW` (sinon contournable botnet) mais `§3 Sprint 19` ne formalise pas PoW comme exit gate mandatory | `§3 Sprint 19 + 21` | Ajouter exit gate Sprint 19 : "PoW Hashcash gossip live + tested avant passage Sprint 20" |
| D-4 | P2 | D | `§3 Sprint 28` Nym integration "phase 1 (1500 LOC, test feasibility)" = research-masque-en-code. `§3 Sprint 30` split inference "research prototype" pareil. Risk S28-30 glissent S31-32 si Nym infeasible | `§3 Sprint 28, 30` | Scoper phase 1 : "SOCKS wrapper + relay bootstrap + glue test" ~800 LOC, defer "phase 2 full integration" S31+ |
| D-5 | P2 | D | `§3 Sprint 27` Sybil mature "trust-web bootstrapped par Amnesty-class ONG" depend `§3 Sprint 28` outreach → cycle dependency (S27 avant S28) | `§3 Sprint 27 + 28` | Documenter : "Amnesty-class ONG letter of intent = exit gate S27, outreach S28 parallel pas sequence" |
| D-6 | P3 | D | `§3 Sprint 22` structured output (S20) "bloque tool-calling design S22" — S22 sandbox items list "dry-run" pas "design block" | `§3 Sprint 20, 22` | Clarifier : S20 grammar = input format filter, S22 sandbox = execution gate, independent |
| D-7 | P3 | D | `§3 Sprint 26` no-sharing policy "refuse task ou warn" ambigu — warn = no impact, refuse = workers hors-ligne | `§3 Sprint 26 line 283-284` | S26 phase 0 decision : strict (refuse) vs soft (warn), document impact capacity planning |
| D-8 | P3 | D | `§4 Quick-wins` "Rate limit per-identity sliding (§7)" listed S21 ~400 LOC mais §3 S21 rate-limit = 1400 LOC total (400 core + 500 client SDK + 300 filter + 200 quarantine queue) | `§4 row 12 line 371` | Clarifier : quick-win = 400 LOC core seul, les 1000 LOC autres sont S21 multi-item |

---

## Track E — VALIDATED_BLUEPRINT briques OSS + zones rouges — **CONDITIONAL PASS**

**Methode rollee** : opinion pre-lecture sur briques attendues stack SBFB (aws-lc-rs, rustls 0.23, wasmtime 43, Arti, keyring-rs, ML-KEM, cargo-vet, osv-scanner, Sigstore, Creusot, Kani, Intel TDX, AMD SEV-SNP, NVIDIA NVML). Lecture `VALIDATED_BLUEPRINT.md` (698 LOC). Verification WebSearch + context7 MCP sur 14 briques critiques + 3 zones rouges.

**Strengths** : 3 zones rouges confirmees par advisories officiels (Bytecode Alliance wasmtime 9 avril 2026, GHSA libp2p-gossipsub, Symbolic Software 7 avril 2026 eprint IACR 2026/192). aws-lc-rs FIPS 140-3 ML-KEM confirmee. rustls-post-quantum crate confirmee. Arti 2.2.0 fevrier 2026 confirmee. Creusot 0.9.0 POPL 2026 confirmee. 9/9 briques retirees justifiees (zkgroup archived, hickory-dns not-recommended, lyrebird Go-only, etc.). Position "match Signal PQXDH" cryptographiquement correcte (X25519 + ML-KEM-1024).

| # | Sev | Track | Finding | Emplacement | Fix suggere |
|---|---|---|---|---|---|
| E-1 | P1 | E | `VALIDATED_BLUEPRINT.md Couche 3` cite `libp2p-gossipsub` CVE-2026-33040 / CVE-2026-34219 mais SBFB **n'utilise pas libp2p-gossipsub** (stack iroh-gossip 0.97 native). Confusion arch trompeuse | `VALIDATED_BLUEPRINT.md Couche 3 gossip` | Ajouter phrase : "SBFB utilise `iroh-gossip` 0.97 (stack iroh native), PAS `libp2p-gossipsub`. CVE cites pour awareness ecosysteme, pas exposure directe SBFB" |
| E-2 | P2 | E | `VALIDATED_BLUEPRINT Couche 1 identity` claim `ML-KEM-1024` preference mais standard production 2026 = `X25519MLKEM768` (AWS, Signal PQXDH, CloudFlare) | `VALIDATED_BLUEPRINT.md Couche 1 PQ` | Clarifier : "ML-KEM-1024 offre marge plus grande mais standard 2026 = ML-KEM-768 via X25519MLKEM768. Tradeoff latency/storage documente" |
| E-3 | P2 | E | `Kani 0.66.0` version non verifiee via releases officielles (dernieres stables GitHub = 0.63.x) | `VALIDATED_BLUEPRINT.md Couche 11 formal verif` | Remplacer par `Kani 0.63.0` ou verifier release 0.66 real |
| E-4 | P3 | E | Wasmtime `CVE-2026-34945` classe "Low Severity" dans blueprint mais advisory Bytecode Alliance = CVSS 9.0 Critical | `VALIDATED_BLUEPRINT.md Couche 8 sandbox` | Clarifier severity post-publication (possible revision classification) |
| E-5 | P3 | E | hax semantic gaps "5" simplification — recherche Symbolic eprint IACR 2026/192 identifie 13 vulns (4 en code verified, 9 en unverified). Downgrade libcrux → "OK primitives secondaires" OK mais 5→13 nuance | `VALIDATED_BLUEPRINT.md Couche 11 libcrux` | Preciser : "13 vulns detectees Symbolic Software 7 avril 2026 (4 verified, 9 unverified), dont 5 semantic gaps pipeline hax→F*" |

---

## Track F — Scope-cut Phase E legitimite — **PASS**

**Methode rollee** : lecture `sprint17_verification.md §Scope-cut Phase E` + cross-check `HARDENING_ROADMAP §7 Gates` + `VALIDATED_BLUEPRINT Couche 10 Operational security` pour verifier redondance claim.

**Verdict** : scope-cut **legitime**. Les 3 items Phase E planifies (`RELEASE_GATES.md` + `PARTNERSHIPS.md` + `DISCLOSURE.md` ~750 LOC) sont majoritairement couverts par :

- **Gates 1-4 sequencing** : `HARDENING_ROADMAP §7` donne mapping Gate→Sprint detaille avec Gate 1 S18 / Gate 2 S22 / Gate 3 effectif S29 / Gate 4 S35+ ✓
- **Partnerships** : `VALIDATED_BLUEPRINT Couche 10` cite OTF Red Team Lab ($43.5M FY2025, Cure53/Include Security audit gratuit), NLnet NGI0 Commons (21.6M EUR), OpenSSF Alpha-Omega ($12.5M 2026), Sovereign Tech Agency DE, ISRG Prossimo, HackerOne Community Edition ✓
- **Disclosure pattern** : `VALIDATED_BLUEPRINT Couche 10` mentionne "security.txt + PGP key + 90 days embargo + CVE assignment workflow GitHub Security Advisories" + vetting contributeurs XZ-pattern (GPG signing, 30-day delay, 4-eyes review) ✓

Items restants **non-redondants** effectivement reportes acceptables :

| # | Sev | Track | Finding | Emplacement | Fix suggere |
|---|---|---|---|---|---|
| F-1 | P2 | F | Responsible disclosure policy concrete (security.txt dans repo + PGP key publique + embargo SLA formel) : BLUEPRINT Couche 10 mentionne pattern mais `.well-known/security.txt` n'existe pas encore. Sans canal de disclosure, projet n'a pas de SPOC pour bug reports externes | Repo root | Creer `.well-known/security.txt` + `SECURITY.md` stub Sprint 19+ (quick-win, pas blocker S18) |
| F-2 | P2 | F | Audit vendor shortlist comparatif (Cure53 vs Trail of Bits vs NCC vs Kudelski vs Radically Open Security) avec couts negocies : necessaire S28 kickoff pour RFP S28, absent actuellement | `docs/security/` | Bloquer S28 kickoff : shortlist produit au plus tard debut S27 par fondation/sponsor responsable |

Items restants legitimement reportes a **sprint OpSec dedie** futur :
- Enforcement mechanism formel app-by-app (ProjectAnnouncement `gate_tier` TBD S18+ deja dans `HARDENING_ROADMAP §7 line 491-492`)
- Outreach template emails partnerships (relationnel pre-S28, pas docs-critical)

**Conclusion Track F** : scope-cut officialise dans `60b539a` est **intellectually honest** — pas de rabattement paresseux. Les 2 items P2 (F-1 disclosure stub + F-2 audit shortlist) sont dette tracee avec sprint cible clair.

---

## Track G — Docs coherence globale + hygiene — **CONDITIONAL PASS**

**Methode rollee** : grep tous liens `](RELEASE_GATES.md)` + `](PARTNERSHIPS.md)` + `](DISCLOSURE.md)` + `](docs/security/*.md)` racine + sub-refs ; verif README index docs/security/ (9 docs + dossier adversaires/) ; verif README racine §Security (9 pointeurs) ; verif CLAUDE.md §Etat actuel Sprint 17 CLOSED ; verif SPRINT_LOG.md v1.2 row S17 ; verif memory tip.

**Strengths** : README docs/security/ index complete (9 entries + dossier adversaires/), README racine §Security pointeurs alignes (9), CLAUDE.md §Etat actuel liste Sprint 17 CLOSED avec commits A-D + BLUEPRINT + scope-cut Phase E explicite, memory tip `60b539a` matche HEAD `60b539a` (sync OK), 6 fiches `adversaries/T0-T5.md` toutes presentes.

| # | Sev | Track | Finding | Emplacement | Fix suggere |
|---|---|---|---|---|---|
| G-1 | **P1** | G | **12 dangling cross-refs** vers `RELEASE_GATES.md` (10), `PARTNERSHIPS.md` (1), `DISCLOSURE.md` (1) dans 5 docs livres. Scope-cut acte `60b539a` mais les refs n'ont pas ete nettoyees | `HARDENING_ROADMAP.md` (lines 14, 462, 485, 496), `ADVERSARIES.md` (113, 299), `ATTACK_SCENARIOS.md` (357), `P2P_THREATS.md` (19, 815), `COMPUTE_THREATS.md` (22, 776, 790) | **Choix Option A (recommande)** : creer 3 stubs courts `docs/security/RELEASE_GATES.md` + `PARTNERSHIPS.md` + `DISCLOSURE.md` qui expliquent scope-cut et pointent vers sources canoniques (HARDENING_ROADMAP §7 pour gates, VALIDATED_BLUEPRINT Couche 10 pour partnerships + disclosure). **Option B** : editer les 12 refs dans les 5 docs pour rediriger |
| G-2 | P2 | G | `docs/security/README.md` header line 6 "Ecrit en Phase E du Sprint 16" est misleading — entries lines 18-24 sont Sprint 17 Phase A-D livrables, pas S16 | `docs/security/README.md` line 6 | Update : "Ecrit Phase E Sprint 16 (S16 docs initial), etendu Sprint 17 A-D + BLUEPRINT (9 docs total au tip `60b539a`)" |
| G-3 | P3 | G | `SPRINT_LOG.md §v1.2` row Sprint 17 ne mentionne pas le tip final `60b539a` (seulement commits A-D + BLUEPRINT + scope-cut narrative) | `docs/claude/SPRINT_LOG.md` lines 65-109 | Ajouter ligne "Tip final `60b539a` post wrap-up Phase F" a la fin du paragraphe S17 |

---

## Hors-scope observations

### Politique pre-launch bien respectee

Tous les docs Sprint 17 respectent `CLAUDE.md §Pre-launch protocol policy` — aucune mention de version bump, de tolerant decoder, de backward-compat. Les wire formats restent pins a v1. Sprint 17 etant recherche pure, 0 impact sur les canonical schemas.

### Position vs OSS state-of-the-art 2026 credible

Les claims `VALIDATED_BLUEPRINT.md` (match Signal PQXDH, Shopify/Fastly sandbox Wasmtime, Briar Arti embed, Mullvad DAITA, Signal SPQR formal verif) sont defendables d'apres WebSearch. Aucune fraude detectee. Claim "leader unique OSS sur compute-sharing defense-in-depth 7 classes" est raisonnable — aucun autre projet OSS ne couvre simultanement les 7 classes de menace GPU compute.

### Risques P0 post-S17 confirmes

Les 3 risques P0 documentes dans `sprint17_verification.md` risques suivis sont **tous valides par audit Track E** :
- R-iroh-audit : pas de SECURITY.md iroh public, risque reel
- R-pyodide-escape : CVE-2025-68668 n8n classe documentee
- R-wasmtime-cve : 12 CVE avril 2026 dont 2 Critical (CVE-2026-34971 + CVE-2026-34945 CVSS 9.0 tous deux)

Ces 3 risques devraient etre inscrits comme **items Sprint 18 explicites** au kickoff (cargo-audit CI + pin wasmtime 43.0.1+ dans S18 est deja dans `HARDENING_ROADMAP §3 Sprint 18`).

---

## Commits fix a lander AVANT Sprint 18 Phase A

Un seul commit recommande (7 P1 ensemble, tous editions docs) :

### `fix(sprint17): audit-P1 — resolve 7 findings from S18 Phase 0 audit`

**Contenu** :

1. **G-1 — Dangling refs Phase E scope-cut** (option recommande : Option A stubs)
   - Creer `docs/security/RELEASE_GATES.md` (~60 LOC) : stub qui explique scope-cut Sprint 17 Phase F + redirect vers `HARDENING_ROADMAP.md §7` (gates mapping Sprint) + note "contenu formel reporte a sprint OpSec dedie futur"
   - Creer `docs/security/PARTNERSHIPS.md` (~50 LOC) : stub + redirect vers `VALIDATED_BLUEPRINT.md Couche 10 Operational security` (OTF/NLnet/OpenSSF/ISRG/HackerOne)
   - Creer `docs/security/DISCLOSURE.md` (~40 LOC) : stub + redirect vers `VALIDATED_BLUEPRINT.md Couche 10` pattern (security.txt + PGP + 90d embargo + CVE workflow) + note F-1 `.well-known/security.txt` S19+
   - Update `docs/security/README.md` index : ajouter 3 stubs avec mention "Phase E scope-cut (stub pointeur)"

2. **D-1 — Gate 3 table §7 clarification**
   - Edit `docs/security/HARDENING_ROADMAP.md §7` ligne 469 : `Gate 3 | T0-T3 | **S29 (tech S27 + audit externe)** | ...`
   - Supprimer la "Correction Gate 3" paragraphe ligne 472-474 (devient redondant si la table est claire)

3. **A-1 — Mapping T4 clarification**
   - Edit `docs/security/ADVERSARIES.md §3` table ligne 121 : ajouter row `| T0-T4 partial | Gate 3+ (Tor S25 + mixnet S28+) | apps investigations cross-border |`
   - OU ajouter note sous table : "Gate 3 mitige T0-T3 + **partial T4 dragnet metadata** via Tor S25. T4 complet necessite Gate 4"

4. **A-2 — Terminologie couvert/partial/absent standardisee**
   - Edit `docs/security/ATTACK_SCENARIOS.md` §1-12 : pour chaque `Current SBFB mitigation status` bloc, prefix `❌ Absent / ⚠️ Partiel / ✅ Couvert` au lieu de prose

5. **B-1 — Sybil tier reclass**
   - Edit `docs/security/P2P_THREATS.md §8` synthese + `docs/security/HARDENING_ROADMAP.md §1` matrix row `B-Sybil` : Tier max = **T2+ pre-S19, T5 post-S19 PoW + S22 kudos-weighted**

6. **C-1 — Carlini 2024 ref correction**
   - Edit `docs/security/COMPUTE_THREATS.md §1.6 line 137` : `Tramer et al. 2024` → `Carlini et al. 2024`

7. **E-1 — libp2p-gossipsub vs iroh-gossip clarification**
   - Edit `docs/security/VALIDATED_BLUEPRINT.md Couche 3 gossip` : ajouter note explicite "SBFB utilise `iroh-gossip` 0.97 native, PAS `libp2p-gossipsub`. CVE-2026-33040/34219 cites pour awareness ecosysteme, pas exposure directe SBFB"

**LOC estime total commit fix** : ~200 LOC (majoritairement 3 stubs ~150 LOC + edits 50 LOC).

**Delta tests** : 0 (docs-only fix).

---

## Verdict final : **CONDITIONAL PASS**

Sprint 18 Phase A (kickoff + plan) peut demarrer **apres le commit** `fix(sprint17): audit-P1 — ...` ci-dessus.

Les 19 P2 sont loggees comme **dette docs** a reprendre au fil des sprints S18-S30 — certaines se resoudront naturellement quand le code correspondant atterrira (ex: D-2 keyring-rs decision S20 kickoff, D-3 PoW exit gate S19 kickoff, F-1 security.txt S19+).

Les 13 P3 restent tels quels — nits de formulation et cross-refs mineurs, pas d'action requise.

Au demarrage Sprint 18, `sprint17_audit_findings.md` migre `.planning/active/` → `.planning/archive/v1.2/` via `git mv` (meme pattern que `sprint16_audit_findings.md` avant).
