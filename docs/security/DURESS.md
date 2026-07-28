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
  shell), suivi d'un `process::exit(0)`. Le mot « irreversible »
  ne vaut pas uniformement : les etages n'ont PAS la meme force,
  et le resume doit les distinguer (mesure S83 L1, bornes 25/26 du
  threat model ; classement rectifie par la verification croisee
  Codex round 2) :
  - **HARD-FAIL** — les deux blobs d'identite (`identity.enc`,
    `identity_duress.enc`) : zero-overwrite + unlink, erreur
    PROPAGEE. Si cet etage echoue, le wipe echoue. Ce n'etait
    pas strictement vrai avant le round 3 : la sonde d'existence
    utilisait `Path::exists()`, qui convertit une erreur d'acces
    en « absent », donc un blob PRESENT mais illisible prenait la
    branche no-op et le wipe se declarait reussi. Corrige en
    `try_exists()`, qui ne rend `false` que pour une absence
    reelle et propage le reste.
  - **BEST-EFFORT, echec avale** — les deux scratchs
    `.enc.tmp`, `subscriptions.json` (le fichier de persistance
    Sprint 11 ; ce n'est PAS `state.sqlite`, que le service ne
    recoit jamais) et `blob-cache/`.
    Un fichier verrouille (antivirus, rotation en vol) n'avorte
    PAS la purge — arbitrage delibere, une purge complete valant
    mieux qu'un abandon sur du debris — mais il SURVIT alors, sans
    aucun signal : le rapport interne ne porte que DEUX booleens
    (subscriptions, blob-cache) et ne sait pas exprimer l'echec
    d'un scratch — `wipe_all` l'avale, puis `keystore_wiped` passe
    a `true` quand meme. Et la reponse HTTP renvoie
    `{"wiped": true}` sur tout `Ok` sans meme consulter ce rapport.
    **L'operateur ne peut donc pas distinguer une purge complete
    d'une purge partielle** (constat Codex rounds 2 et 3).
  - **INERTE** — les entrees OS keyring : le geste est conserve
    mais ne detruit rien, ces entrees n'existant pas a
    l'execution (builder MOCK, borne 25).
  - **NON RECLAME** — le **coffre de secrets** (`SecretVault`,
    credentials tiers) : aucune methode de destruction n'existe.
    **Ni le run store `workflow_runs.sqlite3`** (S83 Phase I), que
    le panic wipe ne nomme nulle part. Le commit qui cablera l'un
    ou l'autre devra etendre **les paragraphes §1, §3.3 ET §5 —
    les TROIS** (le round 3 de la verification croisee a montre
    que viser « les DEUX » laissait §3.3 mentir), sous peine de
    porter ici une promesse fausse.

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
| Vol d'equipement + attaquant essaie de brute-forcer offline avec le blob identity.enc | Phase A Argon2id **SEUL** — la couche keyring n'existe pas a l'execution (builder MOCK, mesure S83 L1, borne 25 du threat model). Le mur reel est Argon2id(PIN) : ~82 ms/tentative, donc `10^6 x 82 ms` = **~82 000 s ≈ 23 h** pour epuiser un PIN a 6 chiffres sur un seul thread (~11 h en moyenne). Le chiffre se RECALCULE a chaque changement de parametres, il ne se recopie pas ; une ferme de crackage parallelise d'autant |
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
  panic wipe **5 fois rapidement**. Le daemon wipe + exit. Ce qui
  a ete efface l'est **definitivement** — aucune recuperation
  possible (cf. §5). Mais l'ecran ne dit pas ce qui a REELLEMENT
  disparu, et tout n'est pas tente : seuls les deux blobs
  d'identite sont HARD-FAIL ; les scratchs, `subscriptions.json`
  et `blob-cache/` sont BEST-EFFORT (echec avale) ; les entrees
  keyring sont INERTES (rien a effacer) ; le coffre de secrets et
  le run store `workflow_runs.sqlite3` ne sont PAS RECLAMES du
  tout. La reponse est la meme dans tous ces cas (classement
  complet en §1). Ne pas lire le succes affiche comme « tout est
  parti ».

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

**Le panic wipe est irreversible.** Irreversible, oui — mais pas uniformement
COMPLET : les etages n'ont pas la meme force (cf. le classement HARD-FAIL /
BEST-EFFORT / INERTE / NON RECLAME en §1). Ce qui disparait ne revient pas ; ce
qui a resiste n'est signale nulle part a l'operateur.

Aucune backup n'est conservee cote daemon :

- `identity.enc` est zero-overwrite puis unlink
- `identity_duress.enc` est zero-overwrite puis unlink
- **Les deux scratchs d'ecriture `identity.enc.tmp` et
  `identity_duress.enc.tmp` sont reclames** (S83 L1), en BEST-EFFORT a echec
  avale : un scratch verrouille (antivirus, rotation en vol) n'avorte pas la
  purge — arbitrage delibere, une purge complete valant mieux qu'un abandon sur
  un fichier de debris — mais il SURVIT alors, silencieusement : sans cela, une mort de
  process entre le `sync_all` et le `rename` d'un `rotate_pin` laissait a cote
  du blob une copie COMPLETE et scellee sous l'ANCIEN PIN — et sa seule presence
  attestait qu'un effacement avait eu lieu (borne 26)
- Les deux entrees OS keyring (`identity-kek-wrap` + `identity-
  kek-wrap-duress`) sont `delete_credential()` — geste conserve, mais
  security-neutral tant que la couche 2 est inerte : sous le builder MOCK rien
  n'a persiste, donc rien ne survit a effacer (borne 25)
- `subscriptions.json` est unlink et `blob-cache/` est `remove_dir_all` —
  **tous deux BEST-EFFORT**. C'est bien `subscriptions.json` que le runtime
  passe au service (`opts.paths.subscriptions_json`), JAMAIS un `state.sqlite` :
  le champ s'appelle `state_db_path` pour des raisons historiques, et cette
  confusion d'etiquette a survecu jusqu'au round 3. L'echec n'est PAS
  enregistre au sens ou on l'attendrait : le rapport ne porte que deux booleens,
  et il ne sait pas exprimer un echec partiel de l'etape 1 (`wipe_all` avale un
  scratch en echec puis `keystore_wiped` passe a `true`). La reponse HTTP renvoie
  `{"wiped": true}` sur tout `Ok` sans meme lire ce rapport, donc l'operateur ne
  voit pas la difference entre une purge complete et une purge partielle
  (constats Codex rounds 2, 3 et 4)
- **Le coffre de secrets (`SecretVault`) n'est PAS reclame** : ni son blob, ni
  ses scratchs. Il n'expose aucune methode de destruction (borne 12 : 0 API de
  re-key, 0 `wipe`). Un panic wipe laisse donc les credentials tiers scelles sur
  le disque sous un PIN a 6 chiffres, et la disparition SELECTIVE prouve en
  outre qu'un effacement a eu lieu.
- **Le run store `workflow_runs.sqlite3` n'est PAS reclame non plus** (S83
  Phase I) : il est ouvert separement et n'est aucun des trois chemins passes au
  service de panic wipe.
- Le commit qui cablera l'un ou l'autre devra etendre les paragraphes **§1, §3.3
  ET §5 — les TROIS**, sous peine de porter une promesse fausse dans le runbook
  lu sous contrainte. Viser « les DEUX » a laisse §3.3 mentir au round 3, puis
  viser §1 et §3.3 a laisse CE paragraphe mentir au round 4 : la consigne se
  perime a chaque section ajoutee, donc elle nomme desormais les trois
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
