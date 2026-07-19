# Sprint 83 — Matrice AT/OS réservée + premier contact panel

**Phase** : B « Fondation accessibilité, i18n et préférences privées »
**Date** : 2026-07-18 · **HEAD au dépôt** : `ce99c97`
**Statut** : RÉSERVATION (AM-10) — engagement owner/dispo/fallback/exit par
ligne. La **preuve de conformité** (tests AT réels) n'appartient PAS à la Phase
B (AM-8, fondation) : elle se joue aux surfaces d'autorat puis au panel **AI1**
(round 2) ; le premier contact du panel rémunéré se joue en **W1** (round 1,
AM-9). Ce document est le préalable engagé, pas un résultat.

---

## 1. Version EN 301 549 FIGÉE (WA-16 / Q6 — décision PO 2026-07-19)

**Décision PO tranchée** (question §11 WA-16/Q6 close) : la Phase B fige

- **EN 301 549 V4.x (draft)** — la rédaction alignée sur **WCAG 2.2 AA**, la
  cible technique du chantier (kickoff §2, gelée Day-0).

Il n'y a **plus d'alternative ouverte** : V4.x est la version de référence
consignée. Assomption assumée par le PO : V4.x n'est **pas encore** la version
harmonisée à l'OJEU (son statut réglementaire exact n'est pas vérifiable depuis
le dépôt) ; le chantier privilégie l'alignement technique WCAG 2.2 AA sur la
harmonisation légale courante (V3.2.1 = WCAG 2.1 AA), décision PO explicite.

Le harness a11y (`workflow-hub-a11y.spec.ts`) tague **`wcag22aa`** (+ le set
cumulatif 2.0/2.1/2.2), cohérent avec V4.x. Condition binaire B.4 « EN 301 549
version FIGÉE consignée » : **SATISFAITE**.

---

## 2. Matrice 5 OS/AT (réservée — AM-10 : owner / dispo / fallback / exit)

| # | OS + AT | Navigateur | Owner (réservé) | Disponibilité / lead-time | Fallback | Critère d'exit (AI1) |
|---|---|---|---|---|---|---|
| AT-1 | Windows 11 + **NVDA** | Firefox + Chrome | Mainteneur (poste dev natif) | Immédiate (NVDA gratuit, poste RTX 5080) | Cloud AT Windows+NVDA (BrowserStack) si le poste dev tombe | 0 barrière P0/P1 sur les surfaces d'autorat + runs |
| AT-2 | Windows 11 + **JAWS** | Chrome | Mainteneur | Licence 40-min mode OU licence annuelle — **lead-time achat** | NVDA (AT-1) couvre le gros du lecteur d'écran Windows ; JAWS = confirmation | 0 barrière P0/P1 ; sinon carry tracé (jamais masqué) |
| AT-3 | macOS + **VoiceOver** | Safari | Mainteneur (Mac M2 du rig sharding) | Immédiate (VoiceOver intégré, Mac présent) | Cloud AT macOS+VoiceOver (BrowserStack / MacStadium) si le Mac M2 indisponible | 0 barrière P0/P1 |
| AT-4 | iOS + **VoiceOver** | Safari mobile | Mainteneur (dispositif/cloud à engager) | Appareil iOS OU BrowserStack/cloud AT — **lead-time** | Simulateur iOS (VoiceOver limité) + cloud | 0 barrière P0/P1 mobile ; sinon carry |
| AT-5 | Android + **TalkBack** | Chrome mobile | Mainteneur (dispositif/cloud à engager) | Appareil Android OU cloud AT — **lead-time** | Émulateur Android + cloud | 0 barrière P0/P1 mobile ; sinon carry |

**Clause AM-10** : une ressource non sécurisée (JAWS, appareil mobile, cloud AT)
= **retour PO** pour re-scope (fallback/exit), **jamais un carry masqué**. Les
lignes AT-1 et AT-3 sont **couvertes par le rig existant** (Windows + Mac du
banc sharding) ; AT-2/AT-4/AT-5 ont un lead-time d'acquisition à engager avant
la phase de conformité (surfaces + AI1), pas en Phase B.

---

## 3. Panel utilisateurs rémunéré — premier contact (round 1 = W1, AM-9)

Le panel d'utilisateurs en situation de handicap est **rémunéré** (AM-9). Deux
rounds : **round 1 en W1** (surfaces d'autorat R..U + runs V disponibles),
**round 2 en AI1** (matrice AT complète). Le **premier contact** (recrutement +
calendrier) a un **lead-time long** : il est engagé EN PARALLÈLE dès maintenant
(checkpoint ressources #1, AM-10), pas au moment de W1.

| Élément | Engagement (réservation, pas résultat) |
|---|---|
| Owner du recrutement | Mainteneur (FlowUP) — à déléguer si panel intermédiaire |
| Cible round 1 (W1) | 3–5 participant·es couvrant lecteur d'écran, moteur, cognitif/COGA, basse vision |
| Canal de recrutement | Associations handicap + communautés AT (NVDA/VoiceOver) + plateformes de test rémunéré accessibilité — **short-list à arrêter au checkpoint pré-W1** (action engagée), **lead-time semaines** |
| Rémunération | Prévue (AM-9) — barème à fixer au checkpoint pré-W1 |
| Fallback | Si recrutement insuffisant à W1 : audit expert AT rémunéré + report d'un sous-lot de participant·es à AI1 (AM-10, tracé) |
| Critère d'exit | Round 1 : barrières P0/P1 routées en rework **AVANT X** ; round 2 (AI1) : 0 barrière P0/P1 |

**Ce que Phase B livre (AM-10, fondation)** : l'**ENGAGEMENT** du premier contact
— owner, cible, canaux, rémunération, fallback et exit **RÉSERVÉS** — PAS
l'outreach effectif. Par la séquence même du plan (round 1 = W1), les mails et le
calendrier sont émis au checkpoint **pré-W1** (AM-9/AM-10), pas en Phase B (AM-8
fondation). B.2 « premier contact panel round 1 = W1 » est donc satisfait au
niveau **réservation** requis par le checkpoint ressources #1 ; l'outreach
effectif est **routé pré-W1** (tracé, jamais masqué). Ce n'est pas un contact
abouti et ne le prétend pas.

---

## 4. Récapitulatif checkpoint ressources #1 (AM-10)

- Matrice AT **réservée** : 5 lignes, chacune owner/dispo/fallback/exit. ✅
- Panel round 1 : premier contact **engagé** (lead-time long reconnu), owner +
  cible + fallback + exit. ✅
- EN 301 549 : version **FIGÉE** — **V4.x (draft), alignée WCAG 2.2 AA**
  (décision PO 2026-07-19, WA-16/Q6 close, plus d'alternative ouverte). ✅
- Défaillance externe (JAWS, mobile, panel) ⇒ **retour PO**, jamais carry
  masqué (AM-10).
