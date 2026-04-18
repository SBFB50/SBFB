# Sprint 20 Phase E — preflight G8

Date : 2026-04-18
HEAD : `b634c23`
Verdict : **SCOPE-CUT-CONSISTENT** (procede code phase, sous-tache
E.6 ajustee inline ; carry-over docs S+1 non requis pour ce
finding car ajustement absorbe en phase).

Contexte : reprise Phase E post-crash filesystem (working tree
restaure depuis HEAD `b634c23`). Le pivot G8 vers Option C
"federation foundations + WSS fallback" reste arbitre user
2026-04-18 (cf. `sprint20_phase_E_pivot_proposal.md` §7), commit
`bd16e64` ayant deja persiste l'update plan §Phase E. Ce preflight
re-valide les 4 scans factuels avant la re-implementation.

---

## Scans

### S1 — SOTA 2026 vs design

**Libs scannees** :

| Lib / spec | Source | Verdict |
|---|---|---|
| `iroh = "0.97"` (existing) | context7 `/websites/rs_iroh` query "RelayMode Custom relay_wss_only WSS TCP 443 fallback" + WebSearch iroh 0.91 changelog | **FINDING** — voir ci-dessous |
| `frost-ed25519` (nouveau, E.2) | context7 resolve-library-id (timeout/no match direct, retry retourne ed25519-dalek + noble) + WebSearch ZcashFoundation/frost crate Rust | clean (crate existe ZcashFoundation, pas d'advisory specifique 2026 ; pre-research E.2 detaille version exacte au moment du code) |
| `RFC 9591 FROST` (jan 2025) | WebSearch RFC 9591 erratum status 2026 | clean (pas d'erratum publie 2026, RFC stable IRTF) |
| `RUSTSEC advisory check` | WebSearch "rustsec advisory frost-ed25519 CVE 2026" | clean (RUSTSEC-2026-0075 sur **libcrux-ed25519** non-applicable — on n'utilise pas libcrux ; deja documente memory comme R-libcrux-hax P2) |
| Libs internes (`async-trait`, `serde`, `time`, `thiserror`) | sampling LITE Step 3bis (version string check uniquement) | clean (deja workspace-pinned, pas de bump pre-launch) |

**FINDING S1 (non-bloquant)** :

Plan §8.1 E.6 mentionne :

> *« bascule `RelayMode::Custom` avec `relay_wss_only = true` »*

Verification factuelle context7 + WebSearch :

1. **iroh 0.91.0 a supprime l'option TCP raw** vers les relays (cf.
   blog post `iroh-0-91-0-the-last-relay-break`). Citation :
   *« you can now only communicate to the relay servers using
   WebSockets »*. iroh 0.97 herite de cette decision.
2. **`RelayMode::Custom` existe** (context7 confirme variant
   enum), mais prend un `RelayMap`, **pas** un flag
   `relay_wss_only`.
3. WSS = WebSockets = **TCP 443 par defaut, mode unique cote
   relay** depuis 0.91.
4. Le "UDP QUIC vs WSS TCP 443 fallback" du plan correspond en
   realite a l'architecture native iroh :
   - **Direct path** : QUIC hole-punched UDP entre peers
   - **Relay path** : WSS TCP 443 vers le relay (deja unique mode,
     pas un toggle a activer)
   - Fallback **automatique** quand hole-punching echoue (gere par
     iroh sans configuration client-side).

**Classification** : finding non-bloquant (pas CVE, pas breaking
API utilisee, pas RFC revision security-impact). Le plan §8.4
critere d'acceptation prevoit explicitement ce gap :

> *« Pre-research E.6 obligatoire context7 `iroh` 0.97 `RelayMode::
> Custom` + `relay_wss_only` semantics (peut etre flag relay-side
> vs client-side, a confirmer) »*

→ Le pre-research **confirme le doute** que le plan avait anticipe.

**Ajustement E.6 inline (pas de pivot, pas de scope-cut S+1)** :

Implementation E.6 reduite a son intention reelle :

| Original (plan) | Ajuste (post-S1) |
|---|---|
| Probe UDP QUIC 3x 10s + bascule explicite `RelayMode::Custom relay_wss_only=true` | Probe UDP QUIC 3x 10s **diagnostic only** + log `warn!` si QUIC fail (le fallback WSS est automatique cote iroh, rien a wirer client-side) |
| `transport_probe.rs` setup `RelayMode::Custom` au boot | `transport_probe.rs` emit metric `transport.degraded_mode = true` + log structure pour ops |
| 4 tests (probe success / probe timeout / wss connect / config reload) | 4 tests **inchanges** : (a) probe QUIC success → no warn ; (b) probe QUIC timeout 3x → warn emitted + degraded metric set ; (c) iroh fallback transparent vers WSS verifie via integration test ; (d) probe rerun on config reload detecte changement |

**Test budget cap** : 0 delta (4 tests preserves), aucun ajout de
sous-tache. Implementation simplifiee, pas etendue.

**Theme sprint** : intact (anti-DPI defense observable + ops
visibility pour mode degrade).

### S2 — Decisions historiques traversees

**Sampling Step 3bis** : 3 groupes de fichiers, scan `git log
--max-count=100`.

```
Group canary :
  git log --grep="DEVIATION|rejected|scope-cut|deliberate|threat-model" \
    -- crates/nexus-shell-daemon-core/src/canary*

Group transport / iroh runtime :
  git log --grep="..." -- crates/nexus-shell-daemon-core/src/iroh_runtime.rs \
    crates/nexus-shell-daemon-core/src/transport_probe.rs

Group coordinator canary :
  git log --grep="..." -- packages/nexus-coordinator/src/nexus_coordinator/canary*
```

**Findings S2** :

| Sha | Sprint | Subject | Reverse-commit status | Adresse par pivot Option C ? |
|---|---|---|---|---|
| `04c9621` | S18 E2 | warrant canary monthly Ed25519 gossip publish — body : *« DEVIATION deliberee vs plan §E2 : auto-publisher rejete pour raisons threat-model. Stocker la cle Ed25519 en GHA secret ≡ compromission GHA = compromission cle. »* | **Pas de reversion ulterieure** trouvee (`git log --all --grep="04c9621"` → 0 hit ; `git log 04c9621..HEAD --grep="revert\|undo\|reopen" -- canary*` → 0 hit). Decision threat-model toujours active. | **OUI, par construction** — Option C (federation foundations) ne livre AUCUN scheduler / cron / cle accessible auto-process. CanarySigner trait + FROST + duress_ack n'introduisent aucune signature automatisee : la cle canary reste strictement maintainer-only via CLI manuel. |
| `sprint18_plan.md:749` | S18 (planning) | mention "DEVIATION deliberee vs plan initial" | meme decision que `04c9621`, ligne planning correspondante | meme reponse |

Archive scan `.planning/archive/v*/sprint*_*.md` : aucun autre
finding rejet sur federation canary, FROST, ou transport probe.

Memory feedback scan : aucune regle "ne JAMAIS faire X" applicable
a federation foundations / threshold sigs / transport probe.

**Verdict S2** : **clean**. Le seul finding historique (`04c9621`)
est pleinement honore par construction du pivot Option C. Pas de
DESIGN-CONFLICT.

### S3 — Threat model coverage

**Threats mappees Phase E** :

| Threat | Pre-Phase-E (S18 baseline) | Post-Phase-E (federation foundations) | Verdict |
|---|---|---|---|
| **T-canary-gag-order** | Couvert : signature humaine = dead-man switch fonctionne | **PRESERVE** : aucune signature auto introduite (par design Option C) | clean |
| **T-canary-key-exfil** | Couvert : cle uniquement sur poste maintainer | **PRESERVE + AMELIORE** : FROST K=2/N=3 opt-in permet repartition cross-poste cross-juridiction (vol single key insuffisant) | clean (amelioration optionnelle) |
| **T-canary-coercion** (operateur force a continuer) | Couvert : maintainer doit volontairement signer | **AMELIORE** : duress_ack channel daily granularite (signal anti-coercion plus fin que canary monthly) | clean |
| **T-canary-spoof-network** (faux canary publie sur gossip topic) | Couvert : signature Ed25519 + DOMAIN_WARRANT_CANARY_V1 + pubkey bootstrap CANARY.txt | **PRESERVE** : federated CanaryRegistry track pubkeys observees, expose `network-health` pour permettre cross-check humain | clean |
| **T-DPI-ISP** (FAI bloque QUIC UDP) | Partiellement couvert : iroh fallback WSS automatique (depuis 0.91) | **AMELIORE** : ajout observability via probe diagnostic + log degraded metric (E.6 ajuste S1) | clean (defense en profondeur) |
| **T-FROST-threshold-malicious** (NEW threat introduit ?) | N/A | **NEW threat introduit, mais mitigated par design** : default K=1/N=1 = baseline equivalent (pas de regression) ; opt-in K=2/N=3 documente comme procedure cross-juridiction (require recruitment community S25-30) | clean (opt-in seulement) |
| **T-canary-registry-spoof** (NEW threat federated registry) | N/A | **NEW threat introduit** : registry `~/.sbfb/canary-registry.json` peut etre poison par publish bogus topic | mitigation : aggregation + signature verify per-canary-published (chaque entry registry doit valider Ed25519 sig avant ajout) ; documente WARRANT_CANARY_HARDENING.md |

**HARDENING_ROADMAP §3 ligne S20** : mentionne "Encryption at rest
keypair + duress PIN + panic wipe" comme prerequis Gate 2 — Phases
A/B/D/D deja livrees commits `05271fa..7ea68a6`. La Phase E
"federation foundations" est NOUVELLE dans le pivot Option C, pas
dans la roadmap initiale ; sous-tache E.7 ajoutera ligne S25-30
"Warrant canary Niveau 1 enforcement" (cross-juridiction recruitment
+ TEE).

**Verdict S3** : **clean**. 2 nouveaux threats introduits
(T-FROST-threshold-malicious, T-canary-registry-spoof) sont
mitigated par design (opt-in / signature verify per-entry). Aucun
threat existant ne regresse. Defense en profondeur ajoutee sur
T-canary-key-exfil + T-canary-coercion + T-DPI-ISP.

### S4 — Wire format / pre-launch invariants

**Scans `_VERSION` + canonical** :

```
crates/nexus-core-rs/src/canonical.rs:
  pub const DOMAIN_WARRANT_CANARY_V1: &[u8] = b"nexus-warrant-canary-v1";

crates/nexus-shell-daemon-core/src/canary.rs:
  pub const CANARY_VERSION: u16 = 1;
```

**Phase E impact wire** :

| Invariant | Status post-Phase-E |
|---|---|
| `CANARY_VERSION = 1` | **PRESERVE** (E.1 refactor pure ; E.2 FROST sig = Ed25519 RFC 8032 valid byte-for-byte) |
| `DOMAIN_WARRANT_CANARY_V1` figee | **PRESERVE** (FROST aggregation ne change pas le canonical bytes : meme struct, meme domain tag, meme signature octet-comparable a Ed25519 standalone) |
| Pas de tolerant decoder multi-version | **PRESERVE** (aucun bump version, donc pas besoin de decoder multi) |
| `#[serde(default)]` ajoutes legitimes runtime tolerance | **A VERIFIER au moment du code** : nouveaux fields `CanaryRegistry` Python (E.3) + duress_ack message (E.4) doivent documenter rationale runtime tolerance inline si `#[serde(default)]` utilise |
| Pre-launch protocol policy CLAUDE.md | **PRESERVE** (canary v=1 figee, pas de bump pre-tag-v1.0) |
| Day 0 D1..D5 sprint S20 (kickoff §4) | **PRESERVE** : D1 keystore double-layer, D2 Argon2id parametres, D3 duress fake-keypair, D4 llguidance, D5 cap G7 — aucun ne traite federation canary, transport probe, FROST. Aucune rebattu Day 0. |
| Decisions actees `nexus_grid_pivot.md §Decisions actees` | **PRESERVE** (12 items + extensions S12/S13 ne contredisent pas federation canary) |
| **Nouveau** topic gossip `nexus-grid/canary-duress-ack/v1` (E.4) | Nouveau topic = pas un bump version d'un topic existant. Coexiste avec `nexus-grid/warrant-canary/v1` S18. OK pre-launch (libre d'ajouter topics, pas libre de bump existing). |

**Verdict S4** : **clean**. Wire format `CanarySigned v1` integralement
preserve. FROST signatures restent Ed25519 RFC 8032 valides
(verifiables par n'importe quel verifier standard sans modification
client). Day 0 + decisions historiques actees preservees.

---

## Synthese verdict

| Scan | Verdict | Bloquant ? |
|---|---|---|
| S1 SOTA 2026 | **finding non-bloquant** : `relay_wss_only` n'existe pas, ajustement E.6 inline (probe diagnostic only) | non |
| S2 historique | clean (`04c9621` finding adresse par construction Option C) | non |
| S3 threat model | clean (2 nouveaux threats introduits, mitigated par design) | non |
| S4 wire format | clean (CANARY_VERSION = 1 preserve, FROST = Ed25519 wire-compat) | non |

**Aggregation Step 6** : 0 finding bloquant + 1 finding non-bloquant
→ **SCOPE-CUT-CONSISTENT**.

Particularite : le finding S1 E.6 est absorbe **inline en phase**
(implementation simplifiee, 0 delta tests, 0 sous-tache retiree).
Pas de carry-over S+1 requis. Le scope-cut est *intra-phase
implementation detail*, pas *inter-phase deferral*.

---

## Garde-fous (cf. README §6.9, requis meme pour SCOPE-CUT-CONSISTENT
intra-phase)

- [x] **Evidence-based** : context7 `/websites/rs_iroh` query
  RelayMode Custom + WebSearch iroh 0.91 changelog blog post +
  WebSearch RUSTSEC 2026 + WebSearch RFC 9591 erratum + git log
  reverse-commit check `04c9621`
- [x] **Day 0 respect** : ajustement E.6 ne touche pas D1..D5 sprint
  S20 (verifie kickoff §4)
- [x] **Wire format** : aucun bump `*_VERSION` introduit, FROST
  preserve `CanarySigned v1` byte-for-byte (Ed25519 RFC 8032)
- [x] **Test budget cap** : 20 tests Phase E inchanges (pivot
  Option C original), 0 delta de l'ajustement E.6 (4 tests preserves
  semantique simplifiee)
- [x] **Theme sprint** : intact (security hardening — federation
  + anti-coercion + observability ops mode degrade)
- [x] **Pas YAGNI** : federation foundations consument roadmap
  explicite S25-30 ajoutee HARDENING_ROADMAP §3 par sous-tache E.7
- [x] **Retrospective trackee** : note Phase F : ajouter ligne
  *« Pivot retrospective Phase E (continued from 2026-04-18 G8
  arbitrage) + S1 finding E.6 ajustement inline runtime
  re-confirmed 2026-04-18 post-crash »* dans
  `sprint20_audit_plan.md` track meta-process

---

## Action

1. **Procede code Phase E** sous-taches E.1 → E.7 dans l'ordre du
   plan §8 :
   - E.1 `CanarySigner` trait abstraction (refactor pure)
   - E.2 FROST-ed25519 primitive K-of-N (default K=1/N=1)
   - E.3 Federated `CanaryRegistry` coord-side + endpoint
     `/api/canary/network-health`
   - E.4 Duress ack channel topic + CLI `sbfb canary ack`
   - E.5 `AttestationProvider` trait + `NoopAttestation` + roadmap
     doc
   - **E.6 ajuste** : `transport_probe.rs` UDP QUIC probe
     diagnostic + log `warn!` + metric `transport.degraded_mode`
     (PAS de bascule `RelayMode::Custom relay_wss_only` — n'existe
     pas, fallback iroh natif)
   - E.7 Documentation extensive (WARRANT_CANARY_HARDENING.md +
     PATTERNS.md §P31 + HARDENING_ROADMAP §3 ligne S25-30)

2. **Pre-research E.2 obligatoire au moment du code** : confirmer
   crate ZcashFoundation/frost-ed25519 version exacte v2.x +
   advisory check fresh + verifier API DKG K-of-N + signature
   aggregate (context7 + WebSearch fresh).

3. **Pre-research E.3 obligatoire au moment du code** : verifier
   pattern aggregator gossip subscribe coord-side reutilise (cf.
   S18 E2 `04c9621` pattern subscribe + persist + reload).

4. **Commit feat phase E atomique** avec body documentant :
   - Pivot retrospective reference `sprint20_phase_E_pivot_proposal.md`
   - **S1 finding E.6 ajustement** reference ce preflight.md
   - Garde-fous Step 8 verifies (7/7)
   - Working tree audit G5 obligatoire (verifie par `nexus-phase-review`
     skill avant commit)
   - Tests delta exact +20 (E.1 +2 + E.2 +5 + E.3 +4 + E.4 +3 +
     E.5 +2 + E.6 +4 = 20)

5. **Phase F redige `sprint20_audit_plan.md`** avec track
   "Pivot retrospective Phase E + S1 finding inline absorption"
   (track meta-process, audit gate S21).

6. **`nexus-phase-auditor`** receive dimension supplementaire
   "Pivot retrospective + intra-phase G8 finding traceability"
   lors de la review Phase E pre-commit.

---

## Refs

- `docs/claude/README.md §6.9` (G8 source-of-truth)
- `.planning/active/sprint20_phase_E_pivot_proposal.md` (G8 codification
  retrospective + arbitrage user 2026-04-18 Option C deep-evolution)
- `.planning/active/sprint20_plan.md §8 Phase E` (plan post-pivot
  Option C)
- iroh blog `iroh-0-91-0-the-last-relay-break` (suppression TCP raw
  vers relays, WSS unique mode)
- iroh docs `RelayMode` enum (context7 `/websites/rs_iroh` query
  2026-04-18)
- IETF RFC 9591 FROST jan 2025 (status erratum 2026 : aucun erratum
  publie)
- ZF Frost Trail of Bits audit 2023 (cite pivot proposal §2 evidence)
- RUSTSEC advisory check 2026-04-18 (RUSTSEC-2026-0075 libcrux-ed25519
  non-applicable, deja documente memory R-libcrux-hax P2)
- commit `04c9621` (S18 E2 reverse-commit check : pas de reversion
  ulterieure trouvee, decision threat-model active)
