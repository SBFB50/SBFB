# S74 — Design UX/UI « Disponibilité continue » (hosting / redondance des apps)

> Statut : **décidé** (workflow judge-panel, 2026-06-07). À intégrer au kickoff S74
> comme décision **D-DISPO**. Directive PO 2026-06-07 : *« faire tout pour ce sprint,
> les prochains ont d'autres objectifs »* → **S74 absorbe TOUT le programme
> disponibilité/hosting** (Phase A front + 2ᵉ passe pin local + cross-nœud ex-LT-5
> tiré en avant), car S75 (GPU partagé cross-machine) et S76 (sharding) sont engagés
> ailleurs.

## 1. Question d'origine
« Au moment de la publication, faut-il proposer local / VPS / etc. pour héberger
l'app ? » + « Le VPS Linux garde-t-il les 3 projets en ligne ? »

**Réponse retournée par l'analyse** : NON à un champ « cible/hôte » au publish (ça
ré-attribuerait l'auteur ET recentraliserait). La disponibilité devient une
**préoccupation continue** sur la fiche app, **découplée de l'identité**.

## 2. Méthode
Workflow judge-panel (10 agents) : 2 cadrage (contraintes SBFB code-vérifiées +
prior-art OSS) → 4 propositions UX distinctes → 3 juges (UX non-tech / archi-trust /
pragmatique shipping) → synthèse.

## 3. Verdict
**Hybride « Disponibilité continue »** : socle **C** (panneau séparé par app) +
greffes **A** (rappel au bon moment) / **D** (réparation communautaire) / **B**
(garde-fou + page Mes seeds différée).

Scores *overall* : **A** 5/4/5 · **C** 4/5/4 (seul à 5/5 en *trust_security*) ·
**D** 4/4/3 · **B** 3/3/3. C = colonne vertébrale (le découplage temporel
publish-identité / panneau-disponibilité **EST** l'invariant SBFB rendu UI) ; greffe
de A pour neutraliser son seul cran de complexité (le rappel hors-ligne déclenché par
l'état capture le moment mérité sans imposer de décision au publish).

## 4. Principe — 3 invariants rendus VISIBLES (pas juste documentés)
1. **PUBLIER = identité locale signée.** Le formulaire publish ne gagne AUCUN champ
   cible/hôte. Seul ajout = une ligne de vérité sous le CTA. Choisir où publier
   ré-attribuerait l'auteur → interdit par construction.
2. **DISPONIBILITÉ = préoccupation CONTINUE.** Vit sur la fiche app dans un panneau
   latéral, jamais dans un formulaire jetable.
3. **AUTEUR (immuable) et DISPONIBILITÉ (mutable) scellés SÉPARÉMENT.** « Garder en
   ligne ne change jamais l'auteur. »

## 5. Mockups

### Flow 1 — Publier (jargon replié, vérité visible, identité intacte)
```
+--------------------------------------------------------------+
|  Publier une app                                             |
|  Clone un depot Git public, verifie l'identite, et met       |
|  l'app en ligne sur le reseau.                               |
|  URL du depot Git   [ https://codeberg.org/moi/mon-app.git ] |
|  Nom du projet      [ mon-app                              ] |
|  Description        [ Une app web P2P...                   ] |
|  +--------------------------------------------------------+  |
|  |  [Rocket]  Publier sur le reseau                       |  |  <- CTA unique
|  +--------------------------------------------------------+  |
|  (i) Ton noeud signe cette app et la garde en ligne.        |  <- VERITE, pas un choix
|      Elle reste joignable tant que ton noeud tourne.        |     (anti-piege IPFS, AVANT le clic)
+--------------------------------------------------------------+
```
Carte succès (remplace le `<dl>` Hash/Provenance/Commit brut de `Deploy.tsx:151-174`) :
```
(v) App publiee et en ligne — Ton noeud la garde en ligne.
   ( • En ligne sur ton noeud )
   (!) Quand tu fermes ton noeud, l'app reste en ligne
       seulement si un autre pair la garde.
   [ Voir la fiche de l'app ]      > Details techniques   (hashs repliés = avancé)
```

### Flow 2 — Fiche app : top-bar
```
| < Explorer | mon-app  (•)En ligne | Signature verifiee |
|                         [ (•) Disponibilite ]  [ Source ] |  <- remplace le badge "blob:<hash>"
```

### Flow 3 — Panneau latéral « Disponibilité » (Sheet shadcn glass dark)
```
+----------------------------------------+
|  Disponibilite                      X  |
|  AUTEUR                  <<< SCELLÉ     |
|   (v) Publiee par ton noeud            |
|       Signature verifiee               |
|   L'auteur est fige par la signature.  |
|   Garder en ligne ne change jamais     |
|   l'auteur.                            |
|  ETAT                    <<< probe réel browse.rs
|   (•) En ligne — joignable par tous    |
|       Verifie il y a 12 s — Reverifier |  <- last_probed_at + browsePull existant
|  QUI LA GARDE EN LIGNE   <<< mutable    |
|   o Ton noeud           (•) En ligne   |
|     [===O] Garder en ligne        ON   |  <- lecture-seule Phase A ; OFF = 2e passe
|  COPIES DE SECOURS                      |
|   Aucune copie de secours.             |
|   [ + Inviter un pair ]      (Bientôt) |  <- cross-nœud = jamais un faux bouton actif
+----------------------------------------+
```
État HORS LIGNE (greffe A — déclenché par l'état, jamais au publish) :
```
(!) Cette app est hors ligne : ce noeud est ferme. Elle redeviendra joignable au
    prochain demarrage. Pour la garder en ligne meme PC eteint, ajoute une copie
    de secours.                          (bandeau ambre, 1x/session/app, dismissible,
                                          SEULEMENT mes apps)
```
État APP TOMBÉE (greffe D — réparation, boucle atelier-fork) :
```
(SignalZero) Personne ne garde cette app en ligne en ce moment.
Tu as le code source — remets-la en ligne en un clic.
            [  La remettre en ligne  ]   (= forker→redeploy sous TON identité)
```

## 6. Strings FR exactes (pour le prompt Claude Design Phase E)
| Élément | Texte |
|---|---|
| Gate (rename) | **Aucun noeud actif** |
| Nav rail (rename) | **Publier** |
| Sous-titre publish | Clone un depot Git public, verifie l'identite, et met l'app en ligne sur le reseau. |
| CTA publish | Publier sur le reseau |
| Vérité sous CTA | Ton noeud signe cette app et la garde en ligne. Elle reste joignable tant que ton noeud tourne. |
| Succès titre | App publiee et en ligne |
| Succès pill | En ligne sur ton noeud |
| Succès avertissement | Quand tu fermes ton noeud, l'app reste en ligne seulement si un autre pair la garde. |
| Succès repli avancé | Details techniques |
| Bouton top-bar | Disponibilite |
| Pastille reachable/unreachable/unknown | En ligne / Hors ligne / Verification… |
| Auteur — mon app / distante | Publiee par ton noeud / Publiee par un autre noeud |
| Auteur aide (invariant) | L'auteur est fige par la signature. Garder une app en ligne ne change jamais son auteur. |
| État en ligne / hors ligne | En ligne — joignable par tous / Hors ligne — relance ton noeud pour la rediffuser |
| État fraîcheur / action | Verifie il y a {duree} / Reverifier |
| Section seeds titre | Qui la garde en ligne |
| Seed mon nœud / app distante | Ton noeud / Ce noeud (consultation) |
| Toggle / toggle OFF aide | Garder en ligne / App stockee mais plus diffusee — elle disparaitra si aucun autre pair ne la garde. |
| Copies de secours — vide | Aucune copie de secours. Si ton noeud s'eteint, l'app devient hors ligne. |
| Copies de secours — CTA / badge / VPS | Inviter un pair de confiance / Bientôt / Mon serveur |
| Rappel hors-ligne (greffe A) | Cette app est hors ligne : ce noeud est ferme. Elle redeviendra joignable au prochain demarrage. Pour la garder en ligne meme PC eteint, ajoute une copie de secours. |
| App tombée titre / forkeur / CTA | Personne ne garde cette app en ligne en ce moment. / Tu as le code source — remets-la en ligne en un clic. / La remettre en ligne |
| Mes seeds — note anti-recentralisation (greffe B) | Garder un VPS comme defaut pour tout le monde recentraliserait le reseau. Tes seeds restent les tiens. |

## 7. Défauts (zéro piège, anti-recentralisation par construction)
- Au publish : **aucun choix d'hôte** ; le nœud local signataire est de facto le 1ᵉʳ seeder.
- « Garder en ligne » = **opt-OUT explicite** (ON par défaut pour mon app), jamais un opt-in oublié.
- « Copies de secours » **vide par défaut** ; le VPS n'apparaît JAMAIS comme cible suggérée, seulement si l'utilisateur l'ajoute, sous le libellé possessif « Mon serveur ».
- Panneau Disponibilité **fermé par défaut** ; la préoccupation redondance est suggérée SEULEMENT quand l'état réel passe « Hors ligne ».
- App distante (pas la mienne) : panneau **lecture seule**.

## 8. Garde-fou anti-recentralisation (5 verrous câblés dans l'UI)
1. Zéro champ cible/hôte nulle part (un dropdown « publier sur X » = serveur central de fait).
2. Redondance **additive jamais substitutive** : « ajouter une copie de secours », pas « choisir un hôte ».
3. VPS = « **Mon serveur** » (possessif), jamais défaut universel ni suggestion d'office.
4. Provenance/signature **toujours celles de l'auteur** quel que soit le seeder (modèle Radicle : seed ≠ autorité).
5. Suggestion **déclenchée par l'état observé** (« Hors ligne »), jamais poussée au publish ; formulée « ajoute une COPIE » (décentralisant).

## 9. Phasage (PO veut TOUT en S74)
**Phase A — 100 % front, primitives existantes** (probe `browse.rs` humanisé +
`verifyQuery` provenance + `browsePull` + StatusPill/Dot) :
- Nettoyage publish (carte succès + ligne de vérité, hashs repliés).
- Rename « coordinateur » → « nœud »/« réseau » (Deploy.tsx, AppShell CoordinatorPicker, daemon.ts var interne).
- Bouton « Disponibilité » (remplace le badge `blob:<hash>`) → Sheet latéral.
- Panneau lecture : Auteur scellé / État (mapping reachable/unreachable/unknown) / Qui-la-garde.
- Toggle « Garder en ligne » **lecture-seule** (ON honnête).
- Rappel hors-ligne conditionnel (greffe A) + placeholder « app tombée » → `/deploy` prérempli (greffe D).

**2ᵉ passe bornée (locale, PAS de protocole cross-nœud)** :
- Toggle OFF/ON fonctionnel = intention `keep_online` par project_id en table **locale** (type M16/M17, pas wire) + skip-GC du blob + re-annonce au boot réutilisant le pattern outbox (#7 `restore_browse_from_outbox` / #8 `publish_announcement`). Pin **local** persistant.

**Cross-nœud (ex-LT-5, tiré en avant en S74 par directive PO)** :
- Protocole `request_seed(project_id)` cross-nœud **authentifié** (preuve que c'est MON nœud qui demande).
- Porter sur un autre nœud (mon VPS / pair de confiance) : « fetch + épingle + re-annonce » authentifié, provenance de l'AUTEUR intacte (seeder ≠ co-auteur).
- Invitation de pair par lien révocable (modèle Tailscale share) + approbation côté pair (modèle Resilio).
- Re-seed + re-annonce persistante par un pair distant après reboot.
- Registre de seeders (op feed `SeedAnnounced` en raw-op `serde_json::Value`, **sans** bump FEED_FORMAT_VERSION) → compteur communautaire + état multi-seed.
- (vision, à abstraire) re-allocation/failover façon IPFS Cluster — jamais un réglage numérique pour un non-technique.
- Résolution du faux-vert NAT (`deploy.rs:456` self→Reachable, `last_probed_at:None`) : probe externe / signal quorum tiers (« Visible seulement par toi pour l'instant »).

## 10. Décision pour le kickoff — D-DISPO
Adopter « Disponibilité continue ». (1) publish gagne 0 champ hôte (publish = acte
d'identité local signé ; seul ajout = ligne de vérité + carte succès, hashs repliés) ;
(2) hôte/redondance dans un panneau latéral « Disponibilité » sur la fiche app, Section
AUTEUR scellée séparée de Section « Qui la garde en ligne » ; (3) Phase A 100 % front
sur primitives existantes ; (4) 2ᵉ passe = pin local persistant ; (5) cross-nœud
(seed/VPS/pair, compteur, multi-seed) **livré en S74** (directive PO), avec les
boutons « Bientôt » remplacés par du réel au fil des phases — JAMAIS un faux bouton
actif tant que le protocole sous-jacent n'est pas là ; (6) rename « coordinateur ». 5
verrous anti-recentralisation câblés UI.

## 11. Questions PO à trancher au kickoff (Checkpoint §11)
- Faux-vert NAT : libellé honnête « En ligne (vu de ton nœud) » pour les apps
  self-publiées dès le pilote, ou attendre le probe externe ?
- « La remettre en ligne » (app tombée) : `/deploy` prérempli (re-signature, cohérent
  fork) **ou** futur « adopter le blob » sans re-signature ? (sémantique d'auteur différente)
- Page « Mes seeds » (greffe B) : livrée en S74 (vision + empty-state + note
  anti-recentralisation) en plus du cross-nœud réel ?
- Compteur communautaire : best-effort « Toi + d'autres pairs » sans nombre, ou
  nombre exact dès le registre de seeders ?
- Carry sécurité `isHttpsUrl` : appliquer le scheme-guard aux nouvelles ancres
  `repo_url` (panneau Source, app tombée) ET aux ancres pré-existantes non gardées
  (Browse:469-481, BrowsedProject:365-376) dans le même lot.
- Rename « coordinateur » : toute l'UI en S74, ou seulement les écrans publish/dispo touchés ?

## 12. Carries de session à folder dans S74
- Shell hotfix `a53b9f6` (auto-add coordinateur same-origin) — base du rename « nœud ».
- Coverage T14 : écrire les tests FileUploadBlock (35 % → ≥ seuils), retirer le
  masquage `| tail` du fail-fast, ajouter `bootstrap.ts` à `coverage.include`.
- Carries audit S73 (cf. `sprint73_audit_findings.md`) : P2-A-1 rand, P2-AUDIT-2 iroh,
  T-NN+2 wasm, P3-OS-1, LT-2 Radicle PENDING + nouveaux P2 candidats.
