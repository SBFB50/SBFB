# T0 — Utilisateur legitime mal configure

**Tier** : T0 (non-hostile)
**Budget** : zero (pas un adversaire economique)
**Timeline** : immediat (chaque update, chaque install fresh)
**Ecrit** : Sprint 17 Phase A (2026-04-14)

---

## 1. Profil

T0 n'est **pas un adversaire** au sens propre. C'est l'utilisateur
legitime — FlowUP, contributeur, tester — qui produit un risque
security par mauvaise configuration, oubli ou comportement non
documente. On le modelise en tant que tier car **la majorite des
incidents security documentes en OSS (2020-2025) provient de
mauvaises defaults, pas d'attaques cibleees**. Ignorer T0 revient
a laisser le sol glissant et blamer l'utilisateur qui tombe.

Le cas de figure principal : user installe SBFB, lance le daemon,
une de ses actions ouvre une surface que ni l'user ni SBFB ne
voient. Pas d'intent hostile. L'exploit vient d'un T1+ qui
detecte cette surface plus tard.

## 2. Capabilites (non-adversarielles)

- Installer / desinstaller le produit
- Editer `~/.sbfb/*.json` (consent.json, auth_token, daemon.key)
- Partager des liens, screenshots, logs dans des canaux publics
  (Discord, GitHub issues)
- Copier-coller des commandes trouvees dans des tutoriels tiers
- Ignorer des warnings UI (cliquer "OK" sans lire)

## 3. Modes de defaillance typiques

| Mode | Cause probable | Impact | Frequence |
|---|---|---|---|
| Share bearer token accidentellement | Screenshot `~/.sbfb/` dans une issue GitHub | Elevation local → iframe RCE | Haute |
| Update skipped 6+ mois | Auto-update off + notif ignoree | CVE connue non patchee | Haute |
| Consent GPU accorde "tous" distraitement | Dialog 4 niveaux, choix top-down | Worker accepte n'importe quoi | Moyenne |
| Whitelist L3 pollue | User ajoute toute app croisee par curiosite | Permet compute theft T2+ | Moyenne |
| Relance daemon avec flags debug | Tuto blog stale suggere `--unsafe` | Bypasses Host/Origin check | Basse |
| Partage `daemon.key` via sync cloud | Dropbox `~/.sbfb/` auto-sync | Identite volee | Basse |

## 4. Motivations

- Faire marcher le produit plus vite
- Contourner un warning qui semble "trop prudent"
- Aider en fournissant logs (sans savoir qu'ils contiennent des secrets)
- Experimenter des features (whitelist toutes les apps pour voir)

## 5. Observable indicators

T0 laisse des traces dans les logs et l'etat local :

- Fichiers `.sbfb/*.json` dans un repo public (grep sur GitHub)
- Screenshots Discord avec hexadecimal 64 chars (bearer token)
- `usage.json` avec pattern d'acceptation tout-automatique
- Version daemon pas bump depuis N mois (check via `daemon.version`)

## 6. Mitigations SBFB actuelles

**Livre (Sprint 16 et anterieur)** :

- `auth_token` + `daemon.key` avec permissions `0600` (Unix) et
  DACL user-only (Windows), rend copy-paste plus visible (fichier
  caches, pas dans presse-papier).
- Consent dialog 4 niveaux avec default restrictif (niveau 1 =
  mes projets uniquement).
- Bearer + Host + Origin checks cote daemon (Sprint 16 Phase A) :
  meme si user expose un endpoint, il faut 3 facteurs corrects.
- Daily cap GPU auto-reset a minuit local (usage.json), limite la
  purge impact si T0 accorde "tous" par erreur.

**Partiel** :

- Pas d'auto-update (roadmap Sprint 18+). User doit checker
  manuellement `git pull && cargo build`.
- Pas de "red zone" UX pour actions destructives (delete keypair,
  elevate consent) — dialog binaire uniquement.
- Whitelist L3 ajoutable sans confirmation secondaire.

**Absent** :

- Pas d'onboarding security-aware (Sprint 18+).
- Pas de detection exfil accidentelle (grep sur paste public :
  ca ne peut pas etre detecte cote daemon).
- Pas de "sanity check periodique" (daemon hebdo : "ton consent
  est sur niveau 4 depuis 30 jours, toujours d'accord ?").

## 7. Priorisation

T0 est **le tier ou la prevention a le plus gros ROI** : petits
changements UX = gros delta security pour la masse. A traiter
Sprint 18 Phase quick-wins :

- Confirmation double sur consent niveau 4 (whitelist globale)
- Warning fluide si `usage.json` pattern "tout accepte" depuis
  30 jours
- Export anonymise pour bug reports (strippe keypair, token,
  consent)

## 8. Mitigations obligatoires par Gate

| Gate | Requirement T0 |
|---|---|
| 1 (DnD Forge) | Defaults restrictifs ok |
| 2 (TransLingua) | + onboarding 3 ecrans (key mgmt, consent, disclosure) |
| 3 (PolitiScan) | + sanity check periodique + export anonymise bug reports |
| 4 (LibanLive) | + formation OpSec ouverte (EFF template) + app signed every run |

## 9. References

- OWASP Top 10 2021 — A05 Security Misconfiguration
- Stanford study "The Costly Consequences of Default Settings" (2022)
- Signal "Safety Number" pattern — secondary confirmation pour
  actions rares
