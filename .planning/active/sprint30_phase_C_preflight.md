# Sprint 30 Phase C — preflight G8

Date : 2026-04-26 | HEAD : `c50976a` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- `feedback_approach.md` : pick deepest technical option, research
  before code, context7 obligatoire avant code touchant lib externe.
  Phase C utilise frost-ed25519 2.1 (lib crypto) → context7 query
  tentee (lib non indexee), WebSearch effectue en remplacement.
- `sprint14_keyoxide_decision.md` : deploy from source, Ed25519
  identity. Non-conflictuel — canary key est distincte de la node
  identity (persistante vs ephemere).
- `feedback_context7_systematic.md` : context7 obligatoire. frost-
  ed25519 non resolu par context7 (pas dans l'index). Fallback :
  WebSearch ZcashFoundation/frost + RustSec advisory scan.
- Tensions plan vs memory : aucune.

## Scans (all clean)

### S1a — OSS prior art research

Projets recherches :
- **ZcashFoundation/frost** (github.com/ZcashFoundation/frost) :
  reference implementation Rust de FROST RFC 9591. `frost-ed25519`
  2.1 est le crate ciphersuite Ed25519 de cette implementation.
  Trail of Bits audit 2023. API : `generate_with_dealer()` +
  `round1::commit()` + `round2::sign()` + `aggregate()`.
- **Blockstream ChillDKG** (github.com/BlockstreamResearch/bip-
  frost-dkg) : DKG distribue pour FROST. Pertinent pour post-v1.0
  (D1 rejete DKG distribue pour S30). Confirme que trusted dealer
  est le bon point de depart.
- **RFC 9591** (FROST threshold spec, jan 2025) : definit K >= 2,
  trusted dealer procedure Appendix C, interactive signing round1/
  round2/aggregate. Le plan est conforme.
- **BIP-445 FROST Signing** (v0.4.1, 2026-03-03) : standard Bitcoin
  pour FROST. Confirme que trusted dealer + DKG distribue sont les
  2 chemins standards.

Finding : **APPROACH-ALIGNED** — le plan utilise exactement la
procedure trusted dealer + interactive ceremony definie par RFC 9591
et implementee par ZcashFoundation/frost. L'evolution de in-process
scaffolding (frost.rs S20 E.2) vers distribution layer (dkg.rs +
ceremony.rs S30 C) suit le meme pattern que le ZcashFoundation FROST
demo CLI (separation keygen / signing ceremony avec fichiers JSON
intermediaires).

### S1b — Deps/libs versions

- `frost-ed25519` : workspace pin `"2.1"`, pas de release > 2.1.0
  en avril 2026 (WebSearch confirme). Zero CVE RustSec sur
  frost-ed25519 (RUSTSEC-2026-0075 concerne libcrux-ed25519, pas
  frost-ed25519). API `generate_with_dealer` + `round1/round2/
  aggregate` stable depuis 2.0.
- Zero advisory crates.io sur frost-core / frost-ed25519.

Finding : S1b clean — 0 delta.

### S2 — Decisions historiques traversees

- `04c9621` S18 E2 : canary auto-publisher **REJETE** threat-model
  ("Ed25519 key en GHA secret = compromission GHA = compromission
  cle"). Phase C = CLI/HTTP manuel, pas auto-publish → **NON-
  APPLICABLE** par construction. La cle canary reste strictement
  maintainer-only via CLI. Confirme par 6 preflights anterieurs
  (S20 E, S22 C, S22 F, S23 E, S25 B, S26 C). Reverse-commit
  check : 0 reversion trouvee, decision toujours active.
- Memory `feedback_approach.md` : "pick deepest, no band-aid".
  Phase C utilise FROST threshold (deepest option vs single-key).
  Aligne.
- Archive scan `grep -rE "DEVIATION|rejected" .planning/archive/`
  sur canary/frost/dkg/warrant : 0 conflit avec l'approche Phase C.

Finding : S2 clean.

### S3 — Threat model coverage (FULL SCAN — nouveau composant securite)

**Escalation** : Phase C introduit un nouveau composant de securite
(DKG ceremony + distribution layer + HTTP admin endpoints).

Threat matrix Phase C :

| Threat | Couvert par Phase C | Regression |
|---|---|---|
| T-canary-key-exfil | **RENFORCE** (FROST K=2/N=3 → exfil 1 cle insuffisant) | Aucune |
| T-canary-gag-order | **RENFORCE** (threshold → refus 1 participant bloque signing) | Aucune |
| T-canary-coercion | Inchange (duress_ack layer, hors scope Phase C) | Aucune |
| T-canary-spoof-network | Inchange (CANARY.txt bootstrap pubkey) | Aucune |
| T-canary-registry-spoof | Inchange (coord-side passive registry) | Aucune |
| T-FROST-threshold-malicious | **DEJA DOCUMENTE** WARRANT_CANARY_HARDENING §2 (K=2 compromis simultanement → fraude possible, mitige par recrutement cross-juridiction post-v1.0) | Aucune |

Vecteurs nouveaux Phase C :
- **HTTP admin endpoints** (`POST /api/canary/frost/*`) : trust tier
  T0, derriere `auth_required` middleware (bearer + Host + Origin +
  peer creds). Meme surface d'authentification que `/panic/wipe` et
  les autres routes admin. Pas de nouveau vecteur non-couvert.
- **Key share files JSON** (`canary-share-{N}.frost`) : fichiers
  locaux, perms fichier OS. Documentes dans ops runbook §4.2
  ("distribue via 3 canaux distincts, aucune copie disque clair").
  Pas un vecteur reseau.
- **Config canary.toml** : fichier local, chemins vers shares.
  Pas de secret dans le TOML (paths only, pas key material).

HARDENING_ROADMAP §3 S30 prescrit "Warrant canary Niveau 1
enforcement" → Phase C est le livrable prescrit. Pas de pre-
requirement manquant.

Finding : S3 clean — 0 regression, phase renforce couverture
canary. HTTP endpoints proteges par auth T0 existant. Aucun
nouveau vecteur non-modelise.

### S4 — Wire format / pre-launch invariants (fast-path)

- Phase C ne touche PAS `canonical.rs` ni `schemas/`.
- `CANARY_VERSION = 1` (mod.rs:70) : inchange.
- `DOMAIN_WARRANT_CANARY_V1` : inchange.
- `WARRANT_CANARY_TOPIC_SEED` : inchange.
- Wire format `CanarySigned` v1 : inchange. La signature FROST
  aggregee est byte-identique a une signature Ed25519 single-key
  (invariant §6 WARRANT_CANARY_HARDENING.md, teste par
  `frost_sig_verifiable_by_standard_ed25519_verifier`).
- Day 0 D1 (trusted dealer, CLI wiring, pas recrutement) : preserve.
- Pre-launch protocol policy : pas de bump version.

Finding : S4 clean (fast-path verified).

## Note implementation

Le code existant `frost.rs` (S20 Phase E.2, ~493 LOC, 6 tests)
fournit deja les primitives :
- `frost_keygen_trusted_dealer()` — DKG trusted dealer
- `frost_sign_with_shares()` — signing ceremony in-process
- `FrostCanarySigner` — in-process holder de toutes les shares

Les nouveaux modules Phase C ajoutent la **couche distribution** :
- `dkg.rs` : serialisation JSON des key shares individuelles +
  pubkey package pour distribution cross-machine air-gapped
- `ceremony.rs` : workflow step-by-step file-based (chaque round
  lit/ecrit un fichier JSON temporaire)
- HTTP endpoints + CLI subcommands : delegent aux memes core
  functions

Les 6 tests listes dans le plan (§6.3) ont des noms qui
recoupent des tests existants dans `frost.rs::tests`. Les nouveaux
tests doivent cibler la serialisation JSON, le file I/O, et les
HTTP endpoints — pas re-tester les primitives FROST.

## Telemetrie preflight

- Duree totale : ~4m
- S1a : ~2m / 4 projets OSS consultes (ZcashFoundation/frost,
  Blockstream ChillDKG, RFC 9591, BIP-445) / finding : APPROACH-
  ALIGNED
- S1b : ~1m / 1 lib scannee (frost-ed25519 2.1) / finding : clean
- S2 : ~30s / 7 fichiers cibles, 1 commit historique scanne
  (04c9621 S18 E2) / finding : clean (non-applicable)
- S3 : FULL / ~30s / 6 threats mappes, 0 regression, 3 vecteurs
  nouveaux evalues (HTTP T0 + key files + config TOML), 0 gap
- S4 : fast-path / ~15s / CANARY_VERSION=1, DOMAIN inchange,
  wire format preserve

## Action

Proceder code phase C.
