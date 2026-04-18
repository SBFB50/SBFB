# Sprint 20 Phase E — pivot proposal G8

Date : 2026-04-18
HEAD : 59225ee
Verdict : DESIGN-CONFLICT (STOP code, attendre arbitrage user)

Premiere application reelle du gate G8 (cf. `docs/claude/README.md
§6.9` + skill `.claude/skills/nexus-phase-preflight/SKILL.md`).

---

## 1. Le conflit

Plan `sprint20_plan.md §8 Phase E` propose 2 features :

1. **Warrant canary auto-publish scheduler coord-side** : *« Scheduler
   coord-side qui signe + publie un canary heartbeat mensuel
   automatiquement »*
2. **Dual-transport detection + WSS TCP 443 fallback** : probe UDP
   QUIC 3x au boot, fallback `RelayMode::Custom relay_wss_only=true`

**Item 1 = DESIGN-CONFLICT** sur scan S2 (decisions historiques
traversees) + S3 (threat model regression).

**Item 2 = clean** sur les 4 scans, peut proceder tel-quel.

---

## 2. Evidence factuelle

### S1 — SOTA 2026 vs design

- **iroh 0.97 `RelayMode::Custom`** : confirme via grep
  `crates/nexus-core-rs/src/relay_config.rs` + `node.rs` que les
  primitives existent. Reste a verifier via context7
  `/websites/rs_iroh` au pre-research Phase E le wire exact de
  `relay_wss_only` (peut etre un flag relay-side, pas client-side).
- **`frost-ed25519` v2.x** : RFC 9591 publie janvier 2025 (ZF
  Frost), audit Trail of Bits 2023, produit signatures Ed25519
  RFC 8032-valides verifiables par n'importe quel verifier
  standard. Crate Rust `frost-ed25519` actif maintenance 2026.
  → opportunite scaffolding threshold sigs sans break wire
  format `CanarySigned` existant.
- **TEE attestation 2026** : Intel TDX prod 2024+ (Xeon 4th gen+),
  AMD SEV-SNP prod EPYC 7003+, AWS Nitro Enclaves prod. HW non
  disponible RTX 5080 dev (CPU consumer) → impl reelle = sprint
  dedie 25-30, primitive trait abstrait livrable maintenant.
- **drand timelock encryption 2024** : production-ready (League
  of Entropy beacon), mais dechiffrement deterministe → un
  operateur sous gag order peut intercepter le dechiffrement et
  publier signatures fake-valides. Pas une solution dead-man
  switch.
- **Verdict S1** : findings (FROST opportunity, TEE roadmap clair),
  PAS bloquant. SOTA delta = pas de CVE bloquante, mais opportunite
  d'aller deep grace a primitives mature.

### S2 — Decisions historiques traversees

```bash
git log --all --grep="DEVIATION\|rejected\|threat-model" -- \
  packages/nexus-coordinator/ crates/nexus-shell-daemon-core/src/canary*
```

**Finding bloquant** :

- Commit **`04c9621`** (2026-04-15, S18 Phase E2 — *warrant canary
  monthly Ed25519 gossip publish*). Body §"Infra" extrait :

  > « DEVIATION deliberee vs plan §E2 : le plan demandait un
  > auto-publisher (cron qui signe + commit + push), rejete pour
  > raisons threat-model. Stocker la cle Ed25519 en GHA secret ≡
  > compromission GHA = compromission cle. Un maintainer sous
  > gag order pourrait etre contraint de "laisser tourner le cron"
  > → signatures valides perpetuelles alors que le projet est
  > backdoored = cassure du dead-man switch. A la place, GitHub
  > email le maintainer sur scheduled-workflow-fail = notification
  > dead-man switch correcte. »

  → Sprint 18 a explicitement rejete EXACTEMENT le pattern propose
  par S20 plan §8.1 item 1. La rationale threat-model reste valide
  6 mois plus tard (rien n'a change cote menace gag order).

- Commit `04c9621` body "CLI" : la cle canary `~/.sbfb/canary-key.key`
  est strictement humaine (load via CLI manuel `sbfb canary publish`),
  jamais accessible a un scheduler automatise.

**Verdict S2** : FINDING BLOQUANT. Le plan S20 §8.1 item 1
contredit directement decision documentee S18 E2 avec rationale
threat-model encore valide.

### S3 — Threat model coverage

Si on livrait l'auto-publish scheduler coord-side :

| Threat | Couverture pre-Phase-E | Couverture post-Phase-E (auto-publish) |
|---|---|---|
| **T-canary-gag-order** (operateur sous gag order produit canary fake-valid) | Couvert : signature humaine requise = dead-man switch fonctionne | **REGRESSION** : scheduler signe automatiquement = dead-man switch casse |
| **T-canary-key-exfil** (vol cle canary) | Couvert : cle uniquement sur poste maintainer | **REGRESSION** : cle accessible au coord process = nouvelle attack surface (process memory, disk, syscall) |
| **T-canary-coercion** (operateur force a continuer) | Couvert : maintainer doit volontairement signer | **REGRESSION** : "laisser tourner le cron" = signature continue sans action active maintainer |

Item 2 (WSS fallback) n'introduit aucune regression — defensive
baseline anti-DPI.

**Verdict S3** : FINDING BLOQUANT sur item 1. Item 2 clean.

### S4 — Wire format / pre-launch invariants

- `CanarySigned` struct definie S18 E2 : `version = 1`, ne PAS
  bumper. Le pivot doit preserver la struct existante.
- `DOMAIN_WARRANT_CANARY_V1` = `"nexus-warrant-canary-v1"` figee.
- FROST-ed25519 produit signatures Ed25519 RFC 8032 = **wire
  format unchanged**, verifiable par `verify_canary` existant.
  → scaffolding threshold OK sans bump version.
- Day 0 S20 D1..D5 : verifier que pivot ne touche pas. Lecture
  `sprint20_kickoff.md §4` confirme D1-D5 portent sur encryption
  at rest, duress PIN, panic wipe, structured output, cap G7 —
  aucun ne traite explicitement le canary scheduler. Pas de
  rebattu Day 0.

**Verdict S4** : clean (wire format invariants preservables par
pivot federation foundations).

---

## 3. Synthese verdict

| Scan | Verdict | Bloquant ? |
|---|---|---|
| S1 SOTA 2026 | findings (opportunite FROST/TEE) | non |
| S2 historique | finding `04c9621` rejette item 1 | **OUI** |
| S3 threat model | regression T-canary-gag-order si item 1 | **OUI** |
| S4 wire format | clean | non |

**Verdict global : DESIGN-CONFLICT.** Item 1 plan §8.1 = invalid.
Item 2 plan §8.1 = clean (peut etre conserve).

Pivot necessaire avant ecriture code. Code STOP.

---

## 4. Options

### Option A — Scope-cut conforme historique

**Description** : Phase E livre uniquement item 2 (dual-transport
WSS TCP 443 fallback). Item 1 (auto-publish scheduler) marque
"rejected, S18 E2 prevails" + retrait du plan §8. Aucune nouvelle
infrastructure canary.

**Coût** : ~4 tests Rust (transport_probe). Phase E reduite ~50 %.

**Bénéfice** : alignment maximal decision S18 E2. Aucun risque
threat-model. Sprint S20 termine plus tot, reste budget pour
audit gate S19 plus approfondi.

**Invariants preserves** : wire format unchanged ✅, threat model
respect ✅, Day 0 OK ✅.

**Recommandation** : alternative conservative.

### Option B — Adapt minimal (staleness alarm only)

**Description** : Phase E livre item 2 + un check coord-side qui
detecte staleness CANARY.txt (mtime > 25 jours) et emet un log
warn + sentry event + push notification operateur. **Aucune cle
exposee, aucune signature automatique** — juste un rappel passif.

**Coût** : ~3 tests Python (check coord-side) + 4 tests Rust
(transport_probe) = 7 tests total.

**Bénéfice** : capture l'intention pragmatique du plan S20 §8.1
item 1 ("ne pas oublier de publier") sans casser le dead-man
switch. Operateur recoit notification mais doit signer manuellement.

**Invariants preserves** : wire format unchanged ✅, threat model
respect ✅ (aucune signature auto), Day 0 OK ✅.

**Recommandation** : intermediate, faible risque.

### Option C — Deep-evolution federation foundations

**Description** : Phase E pivote vers infrastructure cryptographique
du Niveau 1 warrant canary resilience (TEE-attested + threshold
signatures + federated multi-canary), livrant les **primitives +
scaffolding** sans enforcement reel (qui necessite community 3+
maintainers cross-juridiction = sprint dedie 25-30).

Sous-taches :

- **E.1** : `CanarySigner` trait abstraction (refactor pure,
  extrait Ed25519 logic actuelle dans `Ed25519CanarySigner` impl).
  +2 tests (trait roundtrip identical to baseline).
- **E.2** : FROST-ed25519 primitive K-of-N (default K=1/N=1
  equivalent baseline, opt-in K=2/N=3 via flag). Crate
  `frost-ed25519 v2.x`. Produit signatures Ed25519-valid wire-
  compatible. +5 tests (DKG K=2/N=3 produit sig valide ; K=1/N=1
  round-trip ; aggregate refuse partial < K ; tampered share
  rejected ; cross-verify standard Ed25519 verifier accepts).
- **E.3** : Federated `CanaryRegistry` coord-side — subscribe
  topic gossip, persist `~/.sbfb/canary-registry.json`, expose
  `GET /api/canary/network-health` (pubkeys observees + freshness
  per pubkey). +4 tests Python.
- **E.4** : Duress ack channel — nouveau topic gossip
  `nexus-grid/canary-duress-ack/v1` + CLI `sbfb canary ack`,
  registry tracks ack ages separement (signal anti-coercion plus
  fin que canary monthly). +3 tests Rust.
- **E.5** : `AttestationProvider` trait + `NoopAttestation` impl
  + roadmap doc `WARRANT_CANARY_HARDENING.md`. Decouplage
  CanarySigner != attestation requirement. +2 tests (Noop returns
  dummy ; CanarySigner standalone OK).
- **E.6** : Transport probe + WSS TCP 443 fallback (item 2 plan
  original, intact). +4 tests Rust.
- **E.7** : Documentation extensive — `WARRANT_CANARY_HARDENING.md`
  threat model layers + FROST DKG procedure cross-juridiction +
  TEE roadmap, `PATTERNS.md §P31` CanarySigner+FROST+Federated,
  `HARDENING_ROADMAP §3` ligne S25-30 nouveau "Warrant canary
  Niveau 1 enforcement". 0 tests.

**Coût** : ~20 tests (vs 8 plan original = ratio 2.5x exact, dans
budget cap garde-fou 4). 6 fichiers nouveaux Rust + 3 nouveaux
Python + 1 doc security majeur.

**Bénéfice** : pose les rails cryptographiques pour Niveau 1
enforcement futur sans break wire format. Alignment avec etat de
l'art 2026 (FROST RFC 9591 jan 2025, TEE attestation prod). Phase E
devient cas d'ecole pour articulation primitive→wire→enforcement
Sprint 19.1 pattern.

**Invariants preserves** : wire format `CanarySigned v1` unchanged
✅ (FROST sig = Ed25519 RFC 8032 valide), threat model respect ✅
(aucune signature auto, primitives opt-in), Day 0 OK ✅ (kickoff
S20 §4 D1-D5 ne traite pas canary scheduler).

**Recommandation** : default deep-evolution. Maximal long-term
horizon, ratio test budget exactement au cap, aucune regression.

---

## 5. Recommandation default

**Option C** parce que :

1. Le plan original S20 §8.1 etait sub-optimal (drift S2 + S3).
   Reverter a Option A (scope-cut) = honorer S18 mais perdre
   l'opportunite de poser scaffolding Niveau 1.
2. SOTA 2026 (FROST RFC 9591 mature, TEE attestation prod) rend
   la primitive scaffolding livrable maintenant sans speculation.
3. Wire format preservable a 100% (FROST = Ed25519 wire-compatible).
4. Test budget cap respecte (2.5x exact, dans cap garde-fou 4).
5. Pas YAGNI — consumer roadmap explicite (Sprint 25-30 community
   federation, item ajoute HARDENING_ROADMAP §3).
6. Reste DANS theme sprint S20 = security hardening (kickoff §1).
7. User a explicitement demande "le plus deep possible, le plan
   est adaptable si jugement factuel" 2026-04-18 conversation.

---

## 6. Garde-fous (cf. README §6.9)

- [x] **Pivot evidence-based** : 5 sources externes verifiables
  (commit `04c9621` + RFC 9591 + ZF Frost audit ToB 2023 +
  iroh 0.97 grep relay_config.rs + memory feedback_approach.md
  rule pick-deepest)
- [x] **Pivot ne rebat pas Day 0 sans escalation** : kickoff S20
  §4 D1-D5 ne traite pas canary scheduler ni wire format canary,
  donc pas d'escalation Day 0 requise
- [x] **Pivot ne casse pas pre-launch wire** : `CanarySigned v1`
  preserve, FROST = Ed25519 RFC 8032 wire-compatible, aucun bump
  version
- [x] **Test budget cap respecte** : 20 tests = 2.5x plan original
  (8 tests), exactement au cap garde-fou 4 (cap 2.5x)
- [x] **Pivot dans theme sprint** : S20 = "security hardening",
  warrant canary federation = security hardening direct
- [x] **Pivot ferme gap claire (pas YAGNI)** : consumer roadmap
  explicite ajoute HARDENING_ROADMAP §3 ligne S25-30 "Warrant
  canary Niveau 1 enforcement"
- [x] **Pivot retrospective trackee** : ajouter ligne "Pivot
  retrospective Phase E" dans `sprint20_audit_plan.md` (sera
  redige Phase F) track meta-process pour audit gate S21

---

## 7. Arbitrage user

User a arbitre **Option C** durant conversation 2026-04-18 :

> *« le plus deep possible, le plan est adaptable si jugement
> factuel »*

Suite de l'arbitrage :

> *« Option: α — Codify now, deep »*

Codification G8 livree commit `59225ee` 2026-04-18.

Ce document materialise la procedure G8 a posteriori sur le cas
qui l'a declenche, conformement au pattern. Pour les sprints
futurs, le skill `nexus-phase-preflight` emit ce document AVANT
arbitrage user (pas apres) — ce cas est l'exception "premiere
application" puisque G8 lui-meme n'existait pas au moment de
l'arbitrage.

---

## 8. Suite immediate

1. **Commit chore(planning)** (incluant ce document + update plan
   §Phase E reflet pivot Option C avec sous-taches E.1-E.7).
2. **Code Phase E sous-taches E.1-E.6** dans l'ordre, avec E.7
   doc en parallele pendant que les tests tournent.
3. **Commit feat phase E** atomique avec body documentant pivot
   retrospective + reference ce document + Step 7 garde-fous
   verifies.
4. **Phase F redige `sprint20_audit_plan.md`** avec track
   "Pivot retrospective Phase E" pour audit gate S21.
5. **`nexus-phase-auditor`** receive dimension supplementaire
   "Pivot retrospective" lors de la review Phase E pre-commit.

---

## Refs

- `docs/claude/README.md §6.9` (G8 source-of-truth)
- `.claude/skills/nexus-phase-preflight/SKILL.md` (skill
  implementation)
- Commit `59225ee` (G8 codification + skill)
- Commit `04c9621` (S18 E2 deviation auto-publisher → verifier
  GHA workflow, source du finding S2)
- RFC 9591 FROST threshold signatures (jan 2025)
- ZF FROST Trail of Bits audit 2023
- memory `feedback_approach.md` rule #7 (G8 = mecanisme
  procedural pour pick-deepest)
