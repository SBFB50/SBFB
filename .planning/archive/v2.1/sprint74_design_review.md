# Sprint 74 — Design Review Board (G1)

**Date** : 2026-06-07 (post-audit gate S73 PASS `2fe3b30`).
**Sprint** : 74 — Atelier fork + programme Disponibilite/Hosting complet
(publish nettoye + panneau Disponibilite + pin local persistant + seed
cross-noeud authentifie — ex-LT-5 tire en avant).
**Reviewer** : substitut `nexus-sprint-kickoff` (agent non enregistre →
fallback main thread + Workflow, cf. memory process note). Recherche G9
WebSearch (5 modeles OSS seed/replication/approbation) + 7 cartographies
code SBFB (file:line verifie). Design produit fige en amont
(`.planning/research/s74_disponibilite_ux_design.md`, workflow judge-panel
10 agents, decision D-DISPO).

---

## Scoring

| D# | Titre | Source recente | Alternative comparee | [DETER] Crypto | [DETER] Rust | Code verifie | Verdict |
|---|---|---|---|---|---|---|---|
| D1 | Protocole seed cross-noeud = `SeedRequest` Ed25519+JCS sur **ALPN dedie req/resp** (`sbfb/seed/0`), pas op feed pour la demande | ok (Radicle seeder/protocol 2024-25 ; Syncthing approval ; Tailscale share) | ok (op-feed-broadcast / re-use docs-gossip ALPN / token-non-signe compares) | **a satisfaire** (composition Ed25519+JCS+nonce, pas nouvelle primitive ; domain-constant + anti-replay au preflight Phase E) | ok (iroh Router ALPN Rust-native ; reutilise `canonical.rs`) | ok (`node.rs:341-344` Router 3 ALPN, `blobs.rs:140-163` fetch non-tagge, `canonical.rs` lus) | ⚠️ (nouveau protocole cross-noeud = fondation, pas cablage) |
| D2 | Pin local persistant = table `keep_online` (M18 local) + tag/protect blob + re-annonce au boot (pattern outbox #7) | ok (IPFS reprovide/pinset 2025 ; #7 restore prouve in-repo) | ok (pin-dans-outbox / skip-GC-implicite / timer-22h compares) | N/A (etat local, pas de wire/crypto) | ok (rusqlite_migration M::up Rust-native ; reutilise `add_bytes` tag + `restore_*`) | ok (`blobs.rs:77-88` tag, `node.rs:313-324` FsStore, `runtime.rs:1750-1771` restore, `db.rs:228-303` M-pattern lus) | ✅ |
| D3 | Registre seed = op feed `SeedAnnounced` (raw-op `Value`, zero bump) + compteur **best-effort « Toi + N pairs »** TTL | ok (IPFS Cluster compteur/allocations ; raw-op pre-launch policy) | ok (bump-version / nombre-exact / re-allocation-auto compares) | N/A (raw-op signe reutilise sig feed existante) | ok (serde_json::Value extensible, pas de 5e variante d'enum) | ok (`public_feed.rs:82-118` raw-op 4 variantes, CLAUDE.md:354-366 lus) | ⚠️ (cran le plus aval — peut rester « Bientot » si E-F debordent) |
| D4 | Faux-vert NAT « vu de ton noeud » + invitation revocable (Tailscale) + approbation cote pair (Syncthing) | ok (Tailscale invite/quarantine 2024-25 ; Syncthing approval) | ok (probe-externe-now / cle-noeud-partagee / seed-auto-silencieux compares) | partiel (token d'invitation signe revocable — preflight Phase E) | ok (Rust-native ; reutilise pattern invite coordinator si applicable) | ok (`deploy.rs:445-457` self→Reachable faux-vert lu) | ⚠️ (3 volets produit, 3 arbitrages PO ouverts) |
| D5 | Ampleur = front+fork+pin-local surs (A-D) ; cross-noeud (E-F) borne ; jamais un faux bouton actif | ok (directive PO 2026-06-07 + D-DISPO §5/§9) | ok (tout-en-bloc / faux-boutons-actifs / design-only compares) | N/A | N/A (decision de cadrage produit) | ok (D-DISPO §9 phasage + audit S73 carries lus) | ✅ |

**Resume** : D1 ⚠️, D2 ✅, D3 ⚠️, D4 ⚠️, D5 ✅.
Rigor signal G4 : **3 ⚠️ sur 5** — **au-dessus** de la cible gold 1-2/5,
**assume**. Justification : S74 n'est PAS un sprint de cablage pur (comme
S73 a 90-97 % d'infra) — c'est un sprint a **forte composante fondation
cross-noeud** (D1 nouveau protocole, D3 nouvelle op feed, D4 nouveau modele
produit invitation/approbation). Les 3 ⚠️ sont **honnetes** : ils tracent
exactement le **segment de risque** (le pull-forward LT-5). Aucune des 5
decisions ne rebat une Day-0 gelee.

---

## Findings

### D1 ⚠️ — nouveau protocole cross-noeud (`SeedRequest` ALPN Ed25519) = fondation, pas cablage

**Detail** : contrairement a S73 (cablage sur infra existante), D1 introduit
un **nouveau protocole point-a-point** : un `SeedRequest` signe transporte
par un **nouvel ALPN** (`sbfb/seed/0`) ride par le Router iroh
(`node.rs:341-344` accepte deja 3 ALPN). C'est le segment le plus lourd du
sprint (le pull-forward de LT-5). Risque : sous-estimation du handshake
(approbation D4), du dial NAT cross-noeud, et du **tag du blob fetche**
(`blobs.rs:140-163` `fetch_ticket` **ne tag pas** le blob telecharge → GC
silencieux si non corrige).

**Mitigation (kickoff §4 D1 + plan Phase E + R2/R3)** :
1. **Composition, pas nouvelle primitive crypto** : `SeedRequest` = `{
   project_id, archive_hash, archive_ticket, requester_node_id, nonce, ts }`
   signe Ed25519+JCS (deja en place `canonical.rs`, comme `Task`/`Result`) +
   domain-constant + nonce anti-replay. La checklist [DETER] crypto-spec est
   a satisfaire **au preflight Phase E** (pas une nouvelle courbe/signature).
2. **ALPN req/resp comme blobs/docs** : pattern iroh Router etabli, pas un
   transport nouveau. La demande est **point-a-point cible** (pas un
   broadcast), donc pas de surface Sybil/spam (contrairement a un broadcast
   « seede-moi »).
3. **Tag du blob fetche cote seeder** : corriger `fetch_ticket` (ou ajouter
   `fetch_and_pin`) pour tagger le blob comme `add_bytes` le fait deja
   (`blobs.rs:77-88`) → skip-GC + survie reboot. Test dedie (R3).
4. **E2E 2-noeuds reel (§P57)** AVANT de declarer CLOSED — le pair distant
   garde l'app joignable apres reboot, provenance auteur intacte.
5. **Slice de repli** (D5/Checkpoint) : si Phase E deborde, livrer
   `SeedRequest` + fetch+tag+pin + E2E minimal sans l'invitation-revocable
   complete (qui reste « Bientot »).

**Decision** : **acknowledge + adjust** — le ⚠️ reste (c'est de la fondation,
intrinsequement plus risque qu'un cablage), mitige par la composition de
primitives existantes + le segment borne (D5) + l'E2E reel obligatoire.

### D3 ⚠️ — op feed `SeedAnnounced` + compteur (cran le plus aval)

**Detail** : le registre de seeders est une op feed raw-op `SeedAnnounced`
(`public_feed.rs:82-118` = raw-op extensible, 4 variantes, `FEED_FORMAT_VERSION
=1`) alimentant un etat multi-seed + compteur. C'est le **cran le plus aval**
du Segment 2 (Phase F) : si Phases E-F debordent, c'est le premier a rester
« Bientot ».

**Grounding factuel (research G9)** :
- IPFS Cluster modelise un **compteur de replicas + allocations**, mais
  c'est un **reglage numerique** ; pour un non-technique on l'**abstrait**
  (« copies de secours », pas `replication_factor=3`).
- Un **nombre exact** dans un reseau gossip eventually-consistent est
  **fragile** (un pair tombe sans retract → le nombre ment). IPFS reprovide
  (provider records expirent 24-48h) confirme qu'un compteur de presence est
  par nature **TTL-borne et best-effort**. → **« Toi + N pairs (vus
  recemment) »** est l'affichage honnete (D5/Checkpoint laisse le nombre
  exact au PO).
- Raw-op = **zero bump wire** (CLAUDE.md:354-366 pre-launch policy) : ajouter
  `SeedAnnounced` ne casse aucun noeud (op inconnue stockee/propagee sans
  interpretation).

**Mitigation** : raw-op (pas de 5e variante d'enum, pas de bump) ; compteur
best-effort TTL ; **re-allocation/failover auto = hors-scope** (scope cut #4,
vision) ; le registre est le cran le plus aval → reste « Bientot » si E-F
debordent (D5).

**Decision** : **acknowledge + adjust** — le ⚠️ trace que c'est aval et
optionnel-au-besoin ; raw-op + best-effort + hors-scope failover le bornent.

### D4 ⚠️ — 3 volets produit (faux-vert NAT + invitation + approbation), 3 arbitrages PO

**Detail** : D4 porte 3 decisions **produit** distinctes, chacune avec un
arbitrage PO ouvert (Checkpoint §11) :
1. **Faux-vert NAT** (`deploy.rs:445-457` : self→`Reachable`,
   `last_probed_at:None`) → libelle honnete « En ligne (vu de ton noeud) »
   pilote **vs** probe externe (scope cut #6).
2. **Invitation de pair** revocable (modele Tailscale : single-use/reusable,
   expiration, revocation) — token signe distinct de la cle de noeud.
3. **Approbation cote pair** (modele Syncthing : le destinataire approuve
   explicitement avant fetch+pin) — pas de seed silencieux impose.

**Grounding factuel (research G9)** :
- Tailscale : invite single-use OU reusable (≤1000), expire 30j, revocable ;
  **quarantine par defaut** (la machine partagee ne peut pas initier de
  connexion) → accepter un partage n'expose pas son reseau.
- Syncthing : connecter un pair exige une **approbation explicite** (sauf
  Introducer) ; l'echange liste les dossiers mutuellement partages.
- Radicle : un **seeder n'est pas un delegate** (les delegates portent
  l'autorite de signature ; les seeders repliquent sans signer) → **seeder ≠
  co-auteur cable dans le protocole**, exactement l'invariant SBFB.

**Mitigation** : les 3 volets ont des recommandations posees (libelle honnete
pilote ; invite revocable Tailscale ; approbation Syncthing) ; le PO tranche
l'ampleur au Checkpoint. Un token d'invitation signe revocable est a
[DETER]-specifier au preflight Phase E (composition, pas nouvelle primitive).

**Decision** : **acknowledge + arbitrage Checkpoint** — le ⚠️ trace les 3
arbitrages produit ouverts ; les recommandations sont grounded sur 3 modeles
OSS.

---

## Checklist [DETER] (applicable)

### Crypto/spec
- **D1 `SeedRequest`** : a satisfaire **au preflight Phase E** — Ed25519+JCS
  (deja en place `canonical.rs`, reutilise pour `Task`/`Result`) +
  **domain-constant** dedie (`b"sbfb-seed-request-v1"` ou equivalent) +
  **nonce anti-replay** + `ts` borne. **Composition de primitives existantes,
  PAS une nouvelle primitive crypto** (pas de nouvelle courbe, pas de nouveau
  schema de signature). La spec exacte (champs canonicalises, ordre JCS,
  verification cote seeder) est figee au preflight Phase E, pas inventee ici.
- **D4 token d'invitation** : token signe revocable (reutilise le pattern
  invite `nexus-coordinator-rs` si applicable) — a [DETER]-specifier preflight
  Phase E (forme du token, revocation, expiration).
- **D2/D3** : pin local (etat SQLite, pas de crypto) ; `SeedAnnounced`
  raw-op reutilise la signature feed existante (pas de nouvelle crypto).

### Rust-first
- [x] D1 retenu **Rust-native** (iroh Router ALPN req/resp ; reutilise
  `canonical.rs` Ed25519+JCS). Alternatives comparees : op-feed-broadcast
  (rejete : pas point-a-point, surface Sybil), re-use docs/gossip ALPN
  (rejete : melange contrats), token non-signe (rejete : rejouable).
- [x] D2 retenu **Rust-native** (`rusqlite_migration` M18 + `blobs.rs` tag +
  `restore_*` re-annonce). Alternatives : pin-dans-outbox (rejete :
  semantique broadcast ≠ etat retention), skip-GC implicite (rejete : ment
  + jamais liberable), timer-22h (rejete : raffinement post-launch).
- [x] D3 retenu **Rust-native** (serde_json::Value raw-op extensible).
  Alternatives : bump version (rejete : pre-launch policy), nombre exact
  (rejete : fragile gossip), re-allocation auto (rejete : sur-ingenierie).
- [x] D4 retenu **Rust-native** (handler ALPN + token signe). Alternatives :
  probe-externe-now (rejete : depend infra S75), cle-noeud partagee (rejete :
  expose l'identite), seed-auto silencieux (rejete : impose le stockage).
- D5 : decision de cadrage produit — N/A Rust-first.
- Exemptions : Phase A (frontend UX shell `web/` — exemption §6.1.1) ;
  panneau Disponibilite + rename + carte succes = `web/`.

---

## Conclusion

5 D-decisions, **toutes ancrees dans le code reel** (file:line verifie par
7 cartographies) **et la recherche factuelle** (G9, 5 modeles OSS dates
2024-2025 : Radicle, IPFS Cluster, Tailscale, Syncthing, IPFS reprovide).
**3 ⚠️ assumes** (D1 fondation cross-noeud, D3 cran aval, D4 arbitrages
produit), **0 ❌**. Le sprint a une **double nature honnete** : un segment de
**cablage sur infra existante** (atelier-fork + dispo front + pin local =
A-D, ~85-90 % d'infra prete : triplet S73, `publish_announcement` #8,
`restore_*` #7, `add_bytes` tag, FsStore) **et** un segment de **fondation**
(seed cross-noeud E-F = le pull-forward LT-5, le seul vrai morceau neuf). La
segmentation D5 + l'arbitrage d'ampleur PO Checkpoint §11 sont la mitigation
centrale du risque de debordement (R1). Aucune decision ne rebat une Day-0
gelee ; les invariants (publier=identite locale signee, heberger≠publier,
seeder≠co-auteur, 5 verrous anti-recentralisation, iframe sandbox, wire
pre-launch raw-op/M18-local, iroh 0.98) sont **rendus visibles** par le
design, pas seulement documentes. **G1 PASS** (avec 3 ⚠️ traçant le segment
de risque, comme attendu d'un sprint a composante fondation).
