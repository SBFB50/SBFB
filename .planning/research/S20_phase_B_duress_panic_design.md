# Sprint 20 Phase B — Duress PIN + panic wipe design

**Status** : design doc amont (G1 Design Review Board satisfait via
`sprint20_design_review.md §D3`, ce document approfondit le
rationale implementation + legal + ergonomie pour le reviewer
auditor).

**Date** : 2026-04-16
**Owner** : FlowUP
**Phase A dependance** : `05271fa` — keystore.rs double layer
Argon2id + AES-256-GCM + OS keyring deja vivant.

---

## 1. Rationale — fake keypair (noop) vs wipe-immediate

### Option retenue : fake keypair mode Duress

Quand l'utilisateur est force d'entrer un PIN (garde-frontiere,
saisie equipement, contrainte physique), il peut taper le PIN
duress (second PIN enregistre au setup). Le daemon boote alors
avec un **fake Ed25519 keypair** (reel keypair, different du
normal, stocke dans `identity_duress.enc`). Le mode runtime est
marque `IdentityMode::Duress`. Toutes les operations qui
publieraient des donnees reelles deviennent des noop :

- Gossip publish : le daemon simule un send (log debug), mais
  `TopicSender::broadcast` n'est jamais appele → aucun paquet
  reseau emis sous la fake identity.
- Curator subscribe : accepte la requete HTTP mais n'ajoute rien
  a l'attention set runtime → pas de liste reelle traitee ni
  resignee avec la fake identity.
- Task dispatch : rejette avec 503 `service in maintenance mode`
  → l'adversaire qui observe le daemon voit un comportement
  "daemon minimal" plausible plutot qu'un kill switch explicite.

### Option rejetee 1 : wipe-immediate (GrapheneOS-style)

Le PIN duress declenche immediatement la destruction des
donnees. **Rejete pour 3 raisons** :

1. **Tell-tale post-seizure** : l'adversaire qui ecrit l'image
   disque AVANT de forcer le PIN voit une machine wipee apres,
   conclusion immediate "il avait quelque chose a cacher". Le
   fake-keypair mode garde une surface observable plausible.
2. **Legal risk amplifie** : un wipe declenche est plus
   facilement qualifie d'**obstruction of justice** (18 USC 1512
   aux USA, Code penal §434-4 en France) qu'un etat "rien a
   voir ici" obtenu par decoy. Cf. §4 pour la frontiere
   juridique precise.
3. **Irreversibilite bloque recovery** : si le PIN duress est
   tape par erreur (ergonomie : 4-6 chiffres sous stress),
   l'utilisateur perd tout definitivement. Le fake-keypair mode
   est recoverable (redemarrer avec le vrai PIN) tant que le
   panic wipe explicite n'a pas ete declenche.

### Option rejetee 2 : hidden volume VeraCrypt-style

Deux volumes chiffres, le mot de passe duress ouvre le volume
"benign", le mot de passe reel ouvre le volume "real". **Rejete
pour 2 raisons** :

1. **Detection forensics moderne** : Passware Kit 2025 supporte
   VeraCrypt 1.26.15 hidden partitions via Memory Analysis attack.
   La deniability VeraCrypt est revendiquee mais affaiblie
   academiquement (Springer 2023-2024). Cf. `sprint20_design_review.
   md §D3 angle mort 1`.
2. **Scope cross-platform** : SBFB tourne sur Linux / macOS /
   Windows et ne peut pas s'appuyer sur un FS-level hidden
   volume homogene. L'approche double-blob-fichier reste simple
   et uniforme (8 lignes de difference entre les 2 chemins
   `identity.enc` / `identity_duress.enc`).

---

## 2. Indistinguabilite wire

### Proprietes visees

Un adversaire passif observant le reseau ne doit **pas pouvoir
distinguer** un daemon en mode Normal d'un daemon en mode
Duress via le trafic P2P seul.

### Proprietes obtenues (Phase B)

| Canal observable | Normal | Duress | Distinguabilite |
|---|---|---|---|
| Gossip publish outgoing | envoi reel | noop (no send) | **observable** : le peer ne voit aucun message de ce node → mais peer ne peut pas distinguer "silent daemon" de "daemon in duress" |
| Curator subscribe incoming | process + ack | drop silencieux | **non observable** : le peer ne voit aucun difference |
| Node presence (iroh endpoint) | node_id publie | node_id publie (fake) | **observable sur longue periode** : un peer qui correle historiquement le node_id vs le nouveau verrait que c'est un node different |
| Task handler HTTP | 200 OK avec resultat | 503 maintenance | **local only** : le peer n'appelle pas cette route |

**Limite documentee** : un adversaire qui savait le node_id
Normal AVANT la seizure et qui observe le reseau APRES peut
distinguer (nouveau node_id = fake). Le mode Duress degrade
l'indistinguabilite du cote "historique", pas du cote "present".
Cette propriete est **acceptee** : la Phase B protege contre la
saisie sous contrainte AU MOMENT DE LA SAISIE, pas contre un
adversaire omniscient qui garde un historique complet.

### Blob size indistinguabilite

Les 2 blobs `identity.enc` / `identity_duress.enc` sont
**strictement identiques en taille** : `BLOB_HEADER_LEN (48) +
SECRET_KEY_BYTES (32) + TAG_LEN (16) = 96 bytes`. La forme
d'onde du filesystem (2 fichiers de 96 bytes) reste la meme, un
forensics qui dumpe le dossier `~/.sbfb/shell-daemon/keyring/`
voit deux blobs indistinguables par contenu sans tentatives de
decrypt.

Test `duress_blob_indistinguishable_size_from_normal` verifie
cette propriete.

---

## 3. 5-tap gesture — ergonomie

### Choix : Ctrl+Shift+Alt+W x5 dans 3s

Le raccourci panic wipe est volontairement **difficile a
declencher accidentellement** (3 modifieurs + lettre
non-habituelle + 5 repetitions + fenetre courte). Le compromis :

- **Trop facile** = kill switch declenche par erreur → data loss
  grave (clavier GPG avec modifieurs, accident de chat qui
  marche sur le clavier, hotkey conflict avec une extension
  navigateur).
- **Trop difficile** = utilisateur sous stress n'arrive pas a
  taper correctement → wipe echoue au moment critique.

5 taps dans 3s = 1.67 Hz rythmique, soutenable sous stress mais
refuse les doubles-clics accidentels. Ctrl+Shift+Alt+W est le
hotkey le plus isole qu'on puisse trouver (W n'est pas bindee
par Windows Explorer, Chrome, VSCode, Tmux).

### Feedback UI — aucun

**Decision explicite** : l'UI ne montre aucun feedback visuel
entre les taps (pas de compteur "3/5 taps"). Raison :

1. **Adversaire aux cotes de l'utilisateur** : un geolier ou
   garde-frontiere qui voit "3/5" comprend que quelque chose se
   passe et arrete l'utilisateur.
2. **Deniable par design** : tapoter sans feedback = plausible
   accident clavier. Avec feedback = aveu explicite.

Le user apprend le geste lors du setup initial via
`docs/security/DURESS.md §3.2`, pas via l'UI runtime.

---

## 4. Legal implications

### Frontiere juridique documentee

Le panic wipe et le mode Duress peuvent etre qualifies
**obstruction of justice** / **entrave a la justice** selon la
juridiction et le contexte d'activation :

| Juridiction | Qualification possible | Circonstance declenchante |
|---|---|---|
| USA | 18 USC 1519 (destruction of records) | Investigation federale active, subpoena non respecte |
| France | Code penal §434-4 (destruction de preuves) | Instruction ouverte, requisition judiciaire |
| UK | Criminal Justice and Police Act 2001 §49 | Section 49 notice pour remise de cle |
| Australie | Crimes Act 1914 §3LA | Court-ordered disclosure key |
| Canada | Criminal Code §139 (obstructing justice) | Investigation active |

**La Phase B ne recommande PAS** l'usage du duress PIN ou du
panic wipe dans un contexte de procedure judiciaire active. Le
cas d'usage legitime documente est **la contrainte physique
extra-judiciaire** (garde-frontiere sans subpoena, vol de
device, contrainte par un tiers non-Etat).

`docs/security/DURESS.md §4` documente la frontiere avec un
**legal warning** explicite affiche a l'init :

```
WARNING: Activating duress mode or triggering panic wipe in the
context of an active judicial proceeding, subpoena, or lawful
disclosure order may constitute obstruction of justice under
your jurisdiction's laws. This feature is provided for
protection against extra-judicial coercion (border crossings,
device theft, physical threats). Consult local legal counsel
before using in any investigation context.
```

### Recovery — impossible par design

Le panic wipe est **irreversible**. Aucune backup n'est conservee
cote daemon, aucune clef de recuperation n'existe. Ce point est
documente en gras dans `docs/security/DURESS.md §5` pour prevenir
les tickets support post-wipe du type "comment recuperer mes
kudos ?".

---

## 5. Angles morts non couverts Phase B (scope cuts)

- **Timing indistinguabilite unlock** : `unlock_differential`
  essaie normal d'abord puis duress — temps CPU double (~6s au
  lieu de 3s) pour duress. Mitigation partielle : Argon2id est
  deterministe donc l'ecart est predictible et documente. Fix
  path S23+ : derivation parallele des deux KDF avec cancel de
  la branche perdante.
- **Coldboot RAM acquisition** : si l'adversaire fait un coldboot
  attack apres panic wipe, il peut encore recuperer le keypair
  depuis DIMM pre-zeroize. Mitigation : zeroize RAM avant exit,
  mais une attaque physique <60s post-exit est hors scope Phase B
  (couvert Sprint 22 TPM/Secure Enclave migration).
- **Network analysis correlation historique** : cf. §2, accepte.
- **Coercion vers le vrai PIN par threat-of-violence** : pas de
  defense technique possible, c'est un probleme OpSec humain.

Ces items sont **explicitement hors-scope S20** et tracks dans
`docs/security/HARDENING_ROADMAP.md §4` sprints ulterieurs.

---

## 6. Dependances implementation

| Composant | Source | Role |
|---|---|---|
| `LocalFileKeyStore` Phase A | `crates/nexus-core-rs/src/keystore.rs` | extension `init_duress` reuse `derive_kek1` + `combine_keks` + `write_atomic` |
| `IdentityMode` Phase A | idem | deja present (`Normal` / `Duress`), consomme par daemon runtime |
| `keyring-rs` 3.6 Phase A | deja pin workspace | second account `identity-kek-wrap-duress` |
| `tokio::process::exit` | std | forced exit apres wipe |
| `React + useEffect` | stack web existante | keybind listener |

Aucune nouvelle dep runtime. Seule addition : un
`axum::routing::post("/panic/wipe", ...)` + un composant React.

---

## 7. Tests plan (repris du plan §5.3)

13 Rust + 2 Vitest = 15 tests.

Primitive (6) : duress blob creation + unlock matching + PIN
differential + wrong PIN reject + keypair distinctness + size
equality.

Integration daemon (3) : boot in Duress mode publishes empty,
rejects curator subscribe, rejects task dispatch.

Panic wipe (4) : blobs removed, RAM zeroized, sqlite + blob
cache deleted, process exits.

Frontend (2) : 5-tap triggers POST, 4-tap or slow does not.

---

## 8. Rationale pre-launch protocol policy

BLOB_VERSION reste a `0x01` — la Phase B n'introduit pas de
nouveau format de blob, elle ajoute un **second slot** avec le
meme format. Un futur Phase / Sprint qui changerait le schema
re-minterait les deux blobs simultanement.

Flag bit `use_keyring_layer` (bit 0) est reutilise tel quel.
Aucun bit reserve n'est alloue pour "is_duress" : la distinction
Normal/Duress est portee par le **nom du fichier** + le **slot
keyring**, pas par un flag dans le blob. Cela preserve
l'indistinguabilite binaire byte-par-byte entre les deux blobs.
