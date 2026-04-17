# Duress PIN + panic wipe — nexus-grid / SBFB

Livre Sprint 20 Phase B (2026-04-16). Pair avec
[`HARDENING_ROADMAP.md §3 S20`](HARDENING_ROADMAP.md) item 2+3
(duress PIN + panic wipe).

---

## 1. Vue d'ensemble

Deux mecanismes defensifs qui reposent sur le keystore chiffre
Sprint 20 Phase A :

- **Duress PIN** (`sbfb init-duress --pin <pin>`) : un second
  PIN qui deverrouille une **fake identite Ed25519** (decoy). Le
  daemon boote avec cette fake identite et passe en mode noop
  runtime : gossip publish stop court, curator subscribe drop
  silencieux, task dispatch renvoie 503 maintenance.

- **Panic wipe** (chord `Ctrl+Shift+Alt+W` x5 dans 3s dans le
  shell) : destruction irreversible de tous les secrets daemon
  (blobs normal + duress + entrees OS keyring + subscriptions +
  blob cache) suivie d'un `process::exit(0)`.

Les deux modes sont **recoverables tant qu'on n'a pas declenche
le panic wipe** : un boot en Duress mode laisse le blob normal
intact, un redemarrage avec le vrai PIN rebascule en Normal.

Ces features sont destinees a la **contrainte extra-judiciaire**
(garde-frontiere, vol de device, contrainte physique par un
tiers non-Etat). Elles ne sont PAS recommandees dans un
contexte de procedure judiciaire active — cf. §4 Legal warning.

---

## 2. Threat model

### 2.1 Couvert

| Scenario | Couvert par |
|---|---|
| Garde-frontiere demande le PIN pour unlocker le device avant de rendre l'equipement | Duress PIN |
| Vol d'equipement + attaquant essaie de brute-forcer offline avec le blob identity.enc | Phase A Argon2id + keyring layer |
| Contrainte physique : attaquant force l'unlock, exfiltre le node_id | Duress PIN (decoy node_id) |
| Seizure du device + attaquant tente de reconstituer les secrets depuis RAM post-boot | Drop + Zeroize sur Identity |
| Device compromis : attaquant co-located + user a quelques secondes pour reagir | Panic wipe 5-tap |

### 2.2 Non couvert (scope cuts Phase B)

| Scenario | Pourquoi non couvert | Mitigation future |
|---|---|---|
| Attaquant avec historique network avant + apres seizure | Duress degrade l'indistinguabilite historique (nouveau node_id) | S22+ hardware identity keystore (TPM/Secure Enclave) |
| Coldboot RAM attack <60s post-wipe | Out of scope Phase B | S22+ secure-boot + encrypted swap |
| Coercion vers le vrai PIN par threat-of-violence | Pas de defense technique possible | Humain / OpSec |
| Timing side-channel : `unlock_differential` prend ~2x le temps pour le PIN duress | Documente dans le design doc | S23+ parallel KDF avec cancel |

---

## 3. Operator runbook

### 3.1 Setup initial

Le user exécute dans l'ordre :

```bash
# 1) Initialise l'identite normale (Phase A)
sbfb init --pin 1234

# 2) Ajoute le slot duress avec un PIN DIFFERENT
sbfb init-duress --pin 9999

# Verifier que les deux blobs existent :
ls -la ~/.nexus-grid/shell-daemon/keyring/
# identity.enc         (96 bytes)
# identity_duress.enc  (96 bytes)  <-- meme taille, indistinguable
```

**Contrainte** : les deux PINs doivent etre **nettement
differents** (pas juste `1234` vs `1235`). Sous stress, un guard
qui voit l'user hesiter une demi-seconde avant de taper peut
realiser qu'il y a un choix. Le PIN duress doit etre un PIN que
l'user a memorise comme "le PIN" dans un contexte different —
par exemple le PIN d'une vieille carte bancaire.

### 3.2 Ergonomie panic wipe

Le chord `Ctrl+Shift+Alt+W` doit etre frappe **5 fois** dans une
fenetre glissante de **3 secondes**. Le shell n'affiche **aucun
feedback** entre les taps (pas de "3/5") — par design, pour
preserver la deniabilite du geste.

Repetition recommandee : **rapide et rythmique** (~1.7 Hz).
Trop lent → la fenetre expire et le compte remonte a zero
silencieusement.

**False-trigger rate** : extremement bas. Ctrl+Shift+Alt+W n'est
pas lie par Windows Explorer, Chrome, VSCode ni tmux. Les 3
modificateurs empechent les tap accidentels.

### 3.3 Usage en situation

- **Garde-frontiere demande le PIN** : entrer le PIN **duress**.
  Le daemon boote en mode noop. Si le guard inspecte le browse
  (ou lance une requete `POST /publish-blob`), il voit un daemon
  "normal mais vide" plutot qu'un kill switch explicite.

- **User realise qu'il va etre intercepte** : frapper le chord
  panic wipe **5 fois rapidement**. Le daemon wipe + exit. Le
  device est **definitivement efface** — aucune recuperation
  possible (cf. §5).

---

## 4. Legal warning

> **WARNING** : Activating duress mode or triggering panic wipe
> in the context of an active judicial proceeding, subpoena, or
> lawful disclosure order may constitute **obstruction of
> justice** under your jurisdiction's laws. This feature is
> provided for protection against **extra-judicial coercion**
> (border crossings, device theft, physical threats). Consult
> local legal counsel before using in any investigation context.

### 4.1 Qualifications possibles par juridiction

| Juridiction | Qualification | Circonstance declenchante |
|---|---|---|
| USA | 18 USC 1519 (destruction of records) | Investigation federale active, subpoena non respecte |
| France | Code penal §434-4 (destruction de preuves) | Instruction ouverte, requisition judiciaire |
| UK | Criminal Justice and Police Act 2001 §49 | Section 49 notice pour remise de cle |
| Australie | Crimes Act 1914 §3LA | Court-ordered disclosure key |
| Canada | Criminal Code §139 (obstruction) | Investigation active |

**Le fait de posseder** un duress PIN ou le panic wipe n'est
**pas** un delit. Le delit potentiel est le **declenchement
dans un contexte judiciaire specifique**. La Phase B ne fournit
aucune telemetrie sur l'usage, donc aucune preuve technique de
"activation dans ce contexte" ne se leve dans les logs.

### 4.2 Cas d'usage recommandes

**Legitimes** :
- Garde-frontiere / douanier demande l'unlock sans procedure
  judiciaire formelle
- Vol de device avec contrainte
- Journaliste / activiste en zone hostile
- Dissident exfiltrant des sources

**A eviter sans conseil juridique** :
- User sous investigation active avec subpoena pending
- User avec obligation legale de disclosure (Section 49 UK,
  court order equivalent)
- User dans une procedure civile avec discovery en cours

---

## 5. Recovery — impossible par design

**Le panic wipe est irreversible.**

Aucune backup n'est conservee cote daemon :

- `identity.enc` est zero-overwrite puis unlink
- `identity_duress.enc` est zero-overwrite puis unlink
- Les deux entrees OS keyring (`identity-kek-wrap` + `identity-
  kek-wrap-duress`) sont `delete_credential()`
- `subscriptions.json` est unlink
- `blob-cache/` est `remove_dir_all`
- Aucune clef de recuperation n'est derivee

**Consequences apres panic wipe** :
- Le **node_id change** au prochain `sbfb init`. Les peers qui
  vous connaissaient a l'ancien node_id ne vous trouvent plus.
- Les **kudos accumules sont perdus** — le kudos ledger est
  lie au node_id.
- Les **projets publies** disparaissent du reseau (signature
  Ed25519 du publish invalidee par nouveau node_id).
- La **liste curator** de l'user est a re-signer depuis zero.

Ce point est volontaire : une recovery mechanism briserait
l'axiome de securite — si **l'user** peut recuperer, un
adversaire qui a compromis l'user peut aussi.

**Backups off-device** : hors-scope Phase B. Un user qui
souhaite des backups robustes peut dupliquer manuellement le
`identity.enc` (plus le PIN) dans un storage offline chiffre
(cle USB, cloud E2E). La doc de ces pratiques appartient a un
runbook OpSec user, pas au daemon.

---

## 6. References

- Design doc Phase B : `.planning/research/S20_phase_B_duress_
  panic_design.md`
- Phase A keystore : `crates/nexus-core-rs/src/keystore.rs` +
  `docs/rust/PATTERNS.md §Sprint 20.1` / `§Sprint 20.2`
- HARDENING_ROADMAP §3 S20 : items 2+3 cloture
- Threat model source : `docs/security/ATTACK_SCENARIOS.md
  §A-S9 Checkpoint-seize`
- GrapheneOS duress discussion : forum GrapheneOS 2024-2026
  (rejete comme wipe-immediate, cf. design doc §1)
- VeraCrypt hidden volume analysis : Passware Kit 2025 +
  Springer 2023-2024 (rejete, deniability affaiblie)
