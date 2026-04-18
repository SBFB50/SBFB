<!--
written: 2026-04-18  # Sprint 20 Phase E.7
last_validated: 2026-04-18  # G2 — written same day, no re-audit owed
triggers_revalidate:
  - "frost-ed25519 release > 2.1 (semver bump on K-of-N API)"
  - "RFC 9591 erratum publication (FROST threshold spec)"
  - "TEE attestation backend onboarded (TDX/SEV-SNP/Nitro/H100 CCM driver release)"
  - "Niveau 1 enforcement sprint kicked off (S25-30 maintainer recruitment)"
  - "Canary key wire format bump (CANARY_VERSION > 1, post-tag-v1.0 only)"
audited_findings:
  - "2026-04-18 G8 S2 finding 04c9621 honored by construction Option C : aucune signature canary ne peut etre produite par un scheduler/cron — strictement maintainer-only via CLI. Reverse-commit check : pas de reversion S18 E2 trouvee, decision threat-model toujours active."
-->

# Warrant Canary Hardening — Niveau 0 → Niveau 1 Roadmap

**Sprint 20 Phase E.7** consolide la strategie de durcissement du
warrant canary (Sprint 18 Phase E2 baseline → federation
foundations Sprint 20 Phase E → Niveau 1 enforcement Sprint
25-30). Source-of-truth pour tout developpement canary post-S20 ;
toute modification du module `crates/nexus-shell-daemon-core/
src/canary/` doit reciter ce doc dans son commit body et mettre
a jour la matrice §3 si elle change.

## 1. Definition

Un **warrant canary** est une declaration mensuelle signee qui
affirme que le mainteneur n'a recu aucun ordre legal secret
(National Security Letter, gag order, subpoena confidentielle)
demandant de modifier / backdoorer / divulguer le code projet ou
les donnees utilisateur. La publication mensuelle continue est
la preuve d'independance ; une publication manquee (>45 jours
stale) est le signal **dead-man switch** que le projet est
potentiellement compromis ou contraint au silence.

Le pattern remonte a [The rsync.net warrant canary 2006](https://
www.rsync.net/resources/notices/canary.txt) et a ete adopte par
Apple Transparency Reports, Reddit (jusqu'en 2016), Tumblr,
Cloudflare. Sa force : *contraindre un mainteneur a publier un
canary frauduleux* est typiquement aussi illegal sous la
plupart des juridictions (US, UE, UK) que de leur demander de
mentir au tribunal. Ainsi le manque de canary est legalement
defendable comme "j'ai cesse de declarer car je ne pouvais pas
honnetement signer cette declaration".

## 2. Threat model layers

Trois threats canary explicitement traques (`docs/security/
THREAT_MODEL.md` §STRIDE — repudiation + tampering) :

| Code | Threat | Description |
|---|---|---|
| **T-canary-gag-order** | Operator force a continuer | Un mainteneur sous gag order est legalement contraint de continuer toute publication automatisee (cron, scheduler) tant qu'il garde la cle accessible — la signature continue de produire un canary valide alors que le projet est backdoored. |
| **T-canary-key-exfil** | Vol de la cle de signature canary | Un attaquant qui vole `<sbfb_home>/canary-key.key` peut publier des canaries fake-valides indefiniment, masquant un compromise. |
| **T-canary-coercion** | Operator force a continuer (extension) | Variante T-canary-gag-order ou la coercion est physique / legale courante (interrogation, garde a vue) plutot qu'un gag order federal — granularite plus fine que le canary mensuel. |

Threats reseau (federated layer) :

| Code | Threat | Description |
|---|---|---|
| **T-canary-spoof-network** | Faux canary publie sur le gossip topic | Un attaquant injecte sur `nexus-grid/warrant-canary/v1` un canary signe avec sa propre cle Ed25519 ; un verifier naif l'accepte. Mitigated par le bootstrap pubkey en `CANARY.txt` racine du repo (chain of trust = git remote verification). |
| **T-canary-registry-spoof** | Empoisonnement registry coord-side | Un attaquant POST `/api/canary/observed` avec un canary forge. Mitigated par signature verify per-entry (futur — Phase E.3 en place enregistre passively, le verifier signature reste cote operator/verifier). |
| **T-FROST-threshold-malicious** | Maintainer FROST malicieux | Avec K=2/N=3 : un maintainer sur 3 ne suffit pas. Avec K=2 maintainers compromis simultanement : peut produire des sigs frauduleuses. Mitigated par recrutement cross-juridiction (les 2 doivent etre coerces sous des systems legaux differents). |
| **T-DPI-ISP** | FAI bloque QUIC UDP | Un FAI hostile bloque tout QUIC UDP, empechant le node de joindre le reseau pour broadcast un canary. Mitigated automatiquement par iroh 0.91+ qui fallback WSS TCP 443 sans config client-side ; observability via `transport_probe.rs` (S20 E.6). |

## 3. Strategy : 4-layer defense

Le warrant canary est durci en 4 couches independantes,
montant en sophistication. Chaque couche est livrable
incrementalement sans casser les precedentes (le wire format
`CanarySigned v1` reste fixe a travers toutes).

| Couche | Sprint | Status post-S20 Phase E | Threats couverts |
|---|---|---|---|
| **L0 — Single-key, human-driven** | S18 E2 | **LIVRE** | T-canary-spoof-network (via CANARY.txt bootstrap pubkey) |
| **L1a — Federation primitives (signing)** | S20 E.1 + E.2 | **LIVRE (scaffolding)** | T-canary-key-exfil (FROST K=2/N=3 opt-in primitive disponible) |
| **L1b — Federation primitives (observability)** | S20 E.3 + E.4 | **LIVRE** | T-canary-coercion (duress_ack daily channel), federated registry observability |
| **L1c — Federation primitives (attestation)** | S20 E.5 | **LIVRE (scaffolding NoopAttestation)** | (prep) T-canary-key-exfil renforce via TEE attest, decouple sign != attest |
| **L1 — Enforcement reel** | S25-30 | A LIVRER | Recruit 3+ maintainers cross-juridiction, distribute FROST K=2/N=3 shares per FROST DKG procedure ci-dessous, wire AttestationProvider impl TEE |
| **L2 — Federated multi-canary cross-project** | post-v1.0 | A SCOPER | Plusieurs projets P2P SBFB-compatible publient un canary commun, attaquant doit coercer K projets simultanement |

## 4. FROST DKG procedure cross-juridiction (Niveau 1 enforcement)

A executer une fois lors du recrutement de la federation
(typiquement K=2/N=3 ou K=3/N=5). Ces etapes sont les
prerequisites du flip Niveau 0 → Niveau 1 enforcement, plannifie
Sprint 30 (`HARDENING_ROADMAP.md §3 Sprint 30 — Warrant canary
Niveau 1 enforcement`).

### 4.1 Recrutement participants

- Recruter `N` mainteneurs (default N=3) sous des juridictions
  legales **independantes** (US + UE + UK est l'archetype ; UE +
  UE + UE n'apporte rien car GDPR/RGPD est fonge a Bruxelles).
- Verifier que chaque participant a la capacite legale de
  refuser un gag order de leur juridiction sans s'auto-incriminer.
- Documenter publiquement les juridictions des participants dans
  `CANARY.txt` (`Federation: US (jurisdiction-1), DE (jurisdiction-2),
  ZA (jurisdiction-3)`) — la transparence est la garantie cle.

### 4.2 DKG (trusted dealer initial, can switch to DKG real later)

Phase E.2 baseline = **trusted dealer** (un mainteneur principal
genere les K shares localement et distribue). Pour Niveau 1 :

1. Le maintainer principal (par convention le tag `OWNER` du
   repo) execute `sbfb canary frost trusted-dealer --k 2 --n 3`
   localement sur une machine **air-gapped** (preferablement un
   live-USB Tails ou equivalent).
2. La sortie : 3 fichiers `canary-share-{1,2,3}.frost` + 1
   fichier `canary-pubkey-package.frost` (public).
3. Le maintainer principal **detruit** le RNG seed (zeroize
   memory + reboot machine) puis distribue chaque share via 3
   canaux **distincts** (PGP-encrypted email + Signal +
   physical USB key in person).
4. Chaque participant range sa share dans son OS keyring sous le
   service `sbfb-canary-frost-share`. Aucune copie sur disque clair.
5. Le maintainer principal publie le `canary-pubkey-package.frost`
   public dans `CANARY.txt` et/ou un commit dedie au repo.

**Critical** : `generate_with_dealer` du trusted dealer concentre
le risque de compromise initial sur la machine du maintainer
principal. Pour Niveau 1+ post-v1.0, basculer vers le **DKG
distributed** (`frost::keys::dkg::part1/part2/part3`) qui ne
necessite jamais qu'une seule machine voit toutes les K shares
en clair.

### 4.3 Procedure de signature mensuelle (post-Niveau 1)

Le canary mensuel est signe via FROST round-1 / round-2 /
aggregate. Sequence operationnelle :

1. Le coordinator (un des K participants, role tournant) collecte
   le headline du jour (typiquement `nytimes.com` first-page
   headline 2026-04-15 UTC) + la date.
2. Coordinator demarre un canal de coordination (Signal group
   chat, video call, etc.) avec K-1 autres participants.
3. Coordinator emet `sbfb canary frost round1` qui produit un
   `commitment.json` ; chaque participant fait pareil sur sa
   machine (jamais en remote-desktop, jamais en CI).
4. Tous envoient leur `commitment.json` au coordinator (Signal
   attachment, BLAKE3-verified).
5. Coordinator construit le `signing_package.json` et le diffuse
   aux K-1 participants.
6. Chaque participant emet `sbfb canary frost round2 --signing-
   package signing_package.json` qui produit un
   `share.json` signe par leur share.
7. Coordinator collecte les K shares et execute
   `sbfb canary frost aggregate` qui produit le `Canary` final
   (wire-identique a un Canary single-key).
8. Coordinator broadcast le canary sur gossip + commit
   `CANARY.txt` au repo (PR review par K-1 autres participants
   avant merge).

Les step 2-7 sont **synchrones** (FROST round-1/round-2 est
interactif). Un participant indisponible bloque la procedure
jusqu'a ce qu'un autre prenne le relai (ce qui est le point :
**l'indisponibilite signale un probleme**, exactement le
comportement dead-man switch attendu).

## 5. AttestationProvider — TEE roadmap

Phase E.5 livre le trait abstrait + l'impl `NoopAttestation`
(zero TEE, zero hardware dependency). Sprint 25-30 wirera des
impls concrets selon le hardware partenaire :

| Backend | Hardware requis | Sprint cible | Status 2026-04 |
|---|---|---|---|
| `TdxQuoteAttestation` | Intel Xeon 4th gen+ | S25-30 | TDX prod 2024+, [Intel Trust Authority](https://www.intel.com/content/www/us/en/security/trust-authority.html) public API |
| `SnpReportAttestation` | AMD EPYC 7003+ | S25-30 | SEV-SNP prod, [SEV Tool](https://github.com/AMDESE/sev-tool) attestation flow |
| `NitroAttestation` | AWS Nitro Enclaves | S26+ | Prod stable, NSM API + AWS KMS attestation |
| `H100CcmAttestation` | NVIDIA H100 CCM | S30 | Driver release 2025-Q3, [NVIDIA Confidential Computing](https://docs.nvidia.com/confidential-computing/) |

Decouplage Phase E.5 : un mainteneur avec hardware TEE peut
utiliser `Ed25519CanarySigner` (single key) **+** `TdxQuote
Attestation`, ou `FrostCanarySigner` (threshold) **+**
`NoopAttestation` (no TEE), ou les deux. Les axes signing /
attestation sont independants.

## 6. Wire format invariants (frozen)

Liste des invariants que toute futur evolution canary DOIT
preserver — sinon le commit doit bumper `CANARY_VERSION` (autorise
seulement post-tag v1.0 conforme `CLAUDE.md §Pre-launch protocol
policy`).

| Invariant | Source | Pourquoi |
|---|---|---|
| `CANARY_VERSION = 1` | `crates/nexus-shell-daemon-core/src/canary/mod.rs` | Pre-launch policy : pas de bump avant v1.0 |
| `DOMAIN_WARRANT_CANARY_V1 = b"nexus-warrant-canary-v1"` | `crates/nexus-core-rs/src/canonical.rs` | Domain separation : un canary sig ne peut pas etre rejoue comme task / result / claim / invite / kudos / curator-list / provenance / pow / duress-ack |
| `WARRANT_CANARY_TOPIC_SEED = b"nexus-grid/warrant-canary/v1"` | `crates/nexus-shell-daemon-core/src/canary/mod.rs` | Topic gossip stable : tous les nodes joinent le meme |
| `CanarySigned` field set | `crates/nexus-shell-daemon-core/src/canary/mod.rs` | Tout ajout de field invalide les sigs precedentes |
| FROST sig = Ed25519 RFC 8032 valid | `crates/nexus-shell-daemon-core/src/canary/frost.rs` E.2 | Wire format CanarySigned v1 reste fixe a travers le pivot K=1 single-key → K-of-N threshold |
| Maintainer key NEVER accessible to a scheduler / cron / CI | enforced by S18 E2 04c9621 + S20 E.1 trait abstraction | Dead-man switch integrity |

Phase E.4 ajoute un nouveau **topic gossip** (pas un bump version) :

| Invariant | Source |
|---|---|
| `DURESS_ACK_VERSION = 1` | `crates/nexus-shell-daemon-core/src/canary/duress_ack.rs` |
| `DOMAIN_DURESS_ACK_V1 = b"nexus-duress-ack-v1"` | `crates/nexus-core-rs/src/canonical.rs` |
| `DURESS_ACK_TOPIC_SEED = b"nexus-grid/canary-duress-ack/v1"` | `crates/nexus-shell-daemon-core/src/canary/duress_ack.rs` — distinct du canary topic, partitionnement gossip independant |

## 7. Operator runbook

### 7.1 Daily (post-Niveau 1, optional Niveau 0)

Si Niveau 1 actif **et** duress_ack channel use : `sbfb canary
ack --message "<headline du jour>"` chaque matin. Skip = signal
silencieux (1-2 jours = warn ; 7+ jours = alarm).

### 7.2 Monthly

Le 15 de chaque mois UTC (date conventionnelle ; toute date <30
jours avant la precedente fonctionne) :

1. Verifier qu'aucune circonstance ne contredit la declaration
   warrant canary (pas de NSL recue, pas de subpoena, etc.).
2. Si tout est clair : `sbfb canary publish "<NYT first-page
   headline 2026-MM-DD>"` (Niveau 0) OU executer la procedure
   FROST §4.3 (Niveau 1).
3. Verifier le canary genere : `sbfb canary verify CANARY.txt`.
4. Commit `CANARY.txt` au repo et push.
5. **Si pas tout clair** : NE RIEN PUBLIER. Le silence est
   defense legale. Optionnellement, alerter via canal
   alternatif (autre projet OSS, mastodon, etc.) que le canary
   n'a pas ete publie.

### 7.3 If a maintainer is unreachable (Niveau 1)

Niveau 1 K-of-N tolere `N - K` mainteneurs absents. Si plus de
`N - K` sont absents, le canary **ne peut pas etre signe** —
c'est exactement le comportement dead-man switch attendu.

**Ne pas** chercher a contourner via :
- Bypass les K shares manquantes
- Re-keygen a chaud sans les mainteneurs absents
- Signer en single-key et faire passer comme threshold

Toute tentative de bypass casse le modele de menace et doit etre
refusee par les mainteneurs presents.

### 7.4 Post-incident recovery

Si un mainteneur a ete compromis (cle volee, machine seizee) :

1. Tous les mainteneurs presents executent une **rotation FROST**
   (`frost::keys::refresh_dkg_*` per RFC 9591 §3.4).
2. Le verifying_key change → toutes les CANARY.txt precedentes
   sont **invalidees** (les anciens canary signatures ne
   verifient plus contre la nouvelle pubkey).
3. Le maintainer principal annonce publiquement la rotation +
   les juridictions actuelles + le nouveau verifying_key dans le
   commit CANARY.txt + un blog post.
4. Le mainteneur compromis est retire du recap juridique (ou
   remplace par un nouveau si recrutement).

## 8. Refs

- `crates/nexus-shell-daemon-core/src/canary/mod.rs` (S18 E2 wire
  format + S20 E.1 trait migration)
- `crates/nexus-shell-daemon-core/src/canary/signer.rs` (S20 E.1)
- `crates/nexus-shell-daemon-core/src/canary/frost.rs` (S20 E.2)
- `crates/nexus-shell-daemon-core/src/canary/duress_ack.rs` (S20 E.4)
- `crates/nexus-shell-daemon-core/src/canary/attestation.rs` (S20 E.5)
- `crates/nexus-shell-daemon-core/src/transport_probe.rs` (S20 E.6
  ajusté inline post-G8 : observability-only, pas de wire)
- `packages/nexus-coordinator/src/nexus_coordinator/canary_registry.py` (S20 E.3)
- `packages/nexus-coordinator/src/nexus_coordinator/api/canary.py` (S20 E.3)
- `docs/security/HARDENING_ROADMAP.md §3 S30` (Niveau 1 enforcement
  consumer of S20 E primitives)
- `docs/security/THREAT_MODEL.md §STRIDE §LINDDUN` (T-canary-* mapping)
- `.planning/active/sprint20_phase_E_pivot_proposal.md` (G8
  codification retrospective + arbitrage user 2026-04-18 Option C)
- `.planning/active/sprint20_phase_E_preflight.md` (G8 re-validation
  post-crash 2026-04-18)
- IETF [RFC 9591](https://datatracker.ietf.org/doc/rfc9591/) FROST jan 2025
- ZF Frost [GitHub repo](https://github.com/ZcashFoundation/frost)
  + [crates.io frost-ed25519 v2.1](https://crates.io/crates/frost-ed25519)
- [ZF Frost Trail of Bits audit 2023](https://github.com/trailofbits/publications)
- iroh [blog 0.91 last relay break](https://www.iroh.computer/blog/iroh-0-91-0-the-last-relay-break)
  (S20 E.6 G8 finding source — TCP raw removed, WSS TCP 443 unique mode)
- Sprint 18 E2 commit `04c9621` (warrant canary baseline + auto-publish
  rejection rationale, S20 G8 S2 finding source)
