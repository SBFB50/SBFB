# NEXUS GOV — Roadmap Open Source & Financement

## Mission
Outil citoyen autonome de transparence politique.
IA 100% locale, zero cloud, zero censure, zero interet prive.
Detecte automatiquement les contradictions entre ce que les politiciens disent et ce qu'ils votent.

---

## Sprint 0 — Nettoyage & Publication (1-2 jours)
**Objectif : repo public propre et professionnel**

### 0.1 Nettoyage code
- [ ] Supprimer secrets/credentials du git history (git filter-branch)
- [ ] Verifier .gitignore couvre : .env, data/, *.db, __pycache__, node_modules/, .claude/
- [ ] Supprimer fichiers temporaires, logs, caches
- [ ] Verifier aucun API key/token hardcode dans le code

### 0.2 Documentation
- [ ] README.md (FR + EN) — mission, screenshots, features, architecture, quick start
- [ ] INSTALL.md — guide d'installation detaille (Windows, Linux, Mac)
- [ ] ARCHITECTURE.md — schema systeme, workers, event flow
- [ ] CONTRIBUTING.md — comment contribuer, code style, PR process
- [ ] CODE_OF_CONDUCT.md — Code de conduite contributeurs
- [ ] SECURITY.md — politique de donnees, vie privee, presomption d'innocence
- [ ] CHANGELOG.md — historique des versions

### 0.3 Licence & Legal
- [ ] LICENSE — AGPL-3.0 (meme licence que regards-citoyens/nosdeputes.fr)
- [ ] LEGAL.md — mentions legales, droit de la presse, donnees publiques
- [ ] Declaration presse en ligne (modele de lettre au procureur)

### 0.4 Configuration
- [ ] .env.example complet (toutes les variables documentees)
- [ ] docker-compose.yml verifie et documente
- [ ] pyproject.toml avec metadata projet (auteur, licence, URLs)
- [ ] start.bat / start.sh documentes

### 0.5 Publication
- [ ] Creer repo GitHub public : github.com/FlowUP/nexus-gov
- [ ] Push code nettoye
- [ ] Activer GitHub Issues + Discussions
- [ ] Creer les labels : bug, feature, good-first-issue, help-wanted, civic-tech

---

## Sprint 1 — Identite Visuelle & Demo (3-5 jours)
**Objectif : donner envie aux gens de contribuer et financer**

### 1.1 Branding
- [ ] Logo NEXUS GOV (simple, pro, tricolore subtil)
- [ ] Banner GitHub repo
- [ ] Favicon + meta tags OpenGraph (preview quand partage sur Twitter/Discord)
- [ ] Palette couleurs officielle (deep navy, deja dans le CSS)

### 1.2 Video demo
- [ ] Screencast 2-3 min : sidebar, hemicycle, scan, contradictions, graph, recherche RAG
- [ ] Voix off ou sous-titres FR
- [ ] Upload YouTube + embed dans README
- [ ] Version courte 30s pour Twitter/X

### 1.3 Landing page
- [ ] Page simple (peut etre le README GitHub ou une page Notion publique)
- [ ] Lien vers : repo, demo video, Open Collective, Discord
- [ ] Stats live : "1145 politiciens, 8000+ votes, 31 workers autonomes"

### 1.4 Communaute
- [ ] Creer Discord NEXUS GOV (canaux : general, dev, civic-tech, bugs, idees)
- [ ] Creer Twitter/X @NexusGovFR
- [ ] Creer compte Bluesky @nexusgov.bsky.social

---

## Sprint 2 — Financement (1-2 semaines)
**Objectif : recolter 15,000 EUR pour le serveur M5 Ultra**

### 2.1 Open Collective
- [ ] Creer : opencollective.com/nexus-gov
- [ ] Description avec mission, budget transparent, objectif 15K
- [ ] Tiers de donation : 5 EUR (citoyen), 20 EUR (supporter), 100 EUR (sponsor)
- [ ] Badge "Backed by Open Collective" dans README

### 2.2 GitHub Sponsors
- [ ] Activer GitHub Sponsors sur le repo
- [ ] Tiers mensuels : 3 EUR, 10 EUR, 25 EUR
- [ ] FUNDING.yml dans le repo

### 2.3 Campagne de lancement
- [ ] Post Hacker News : "Show HN: Open source political AI — 100% local, detects contradictions"
- [ ] Post r/france : "J'ai cree un outil IA open source qui detecte les contradictions des politiciens"
- [ ] Post r/opensource, r/selfhosted, r/dataisbeautiful
- [ ] Thread Twitter/X avec screenshots + lien Open Collective
- [ ] Contacter Regards Citoyens (nosdeputes.fr) — proposition de collaboration
- [ ] Contacter Next.ink, Numerama, Siecledigital — article presse tech FR
- [ ] Contacter Mediapart — article outil citoyen

### 2.4 Suivi
- [ ] Mettre a jour Open Collective avec chaque milestone atteint
- [ ] Post mensuel "Transparence" : combien recolte, combien depense, quoi construit

---

## Sprint 3 — Stabilisation Production (1-2 semaines)
**Objectif : NEXUS tourne 24/7 de maniere fiable**

### 3.1 Robustesse
- [ ] Fix tous les crash restants (scan, workers, SSE)
- [ ] Supervision : healthcheck endpoint + alertes si down
- [ ] Auto-restart backend (systemd service ou PM2)
- [ ] Logs rotatifs (loguru rotation, max 100MB)
- [ ] Backup automatique SQLite (quotidien, 30 jours retention)

### 3.2 Performance
- [ ] Benchmark complet : temps de scan, temps de reponse API, memoire
- [ ] Optimiser les requetes SQL lentes (EXPLAIN ANALYZE)
- [ ] Cache API (Redis ou in-memory) pour les endpoints lourds
- [ ] Code splitting frontend (lazy load des tabs)

### 3.3 Tests
- [ ] Couvrir les 31 workers avec tests d'integration
- [ ] Tests E2E : scan → positions → contradictions → alertes
- [ ] CI GitHub Actions : pytest + tsc + build sur chaque PR
- [ ] Badge coverage dans README

### 3.4 Securite
- [ ] Audit dependances (pip-audit, npm audit)
- [ ] Rate limiting sur tous les endpoints publics
- [ ] CSP headers pour le frontend
- [ ] CORS strictement configure pour production

---

## Sprint 4 — Serveur Dedie (quand finance)
**Objectif : NEXUS tourne sur le M5 Ultra 24/7**

### 4.1 Achat & Setup
- [ ] Commander Mac M5 Ultra 512GB (Apple Store ou reconditionne)
- [ ] macOS server setup : Homebrew, Python 3.13, Node.js, Docker
- [ ] Installer Ollama + modeles (gemma heretic, nomic-embed)
- [ ] Cloner repo, installer deps, lancer start.sh

### 4.2 Reseau
- [ ] Nom de domaine : nexusgov.fr (ou .org)
- [ ] Certificat HTTPS (Let's Encrypt)
- [ ] Reverse proxy (Caddy ou Nginx)
- [ ] DynDNS si heberge a domicile, ou IP fixe OVH/Scaleway

### 4.3 Protection juridique
- [ ] Declaration service de presse en ligne au procureur
- [ ] Mentions legales sur le site
- [ ] Politique de donnees (tout est public, zero donnee privee)
- [ ] Contact avocat droit de la presse (consultation ~200 EUR)

### 4.4 Monitoring
- [ ] Uptime monitoring (UptimeRobot gratuit ou Grafana)
- [ ] Alertes Telegram/Discord si serveur down
- [ ] Dashboard public de sante du systeme
- [ ] Metriques : politiciens scannes, contradictions detectees, uptime

---

## Sprint 5 — Croissance & Impact (continu)
**Objectif : communaute active et impact citoyen reel**

### 5.1 Contributions externes
- [ ] Taguer les issues "good first issue" pour les nouveaux contributeurs
- [ ] Mentorer 2-3 contributeurs sur Discord
- [ ] Accepter les PRs de la communaute (review < 48h)
- [ ] Hackathon civic tech (en partenariat avec Open Data France ou Etalab)

### 5.2 Donnees & Couverture
- [ ] Etendre aux eurodeputes FR complets (votes, commissions)
- [ ] Ajouter les elus regionaux/departementaux (data.gouv.fr)
- [ ] Ajouter les maires des grandes villes
- [ ] Partenariat avec Regards Citoyens pour partage de donnees

### 5.3 Impact
- [ ] Newsletter automatique "Alerte Politique" envoyee chaque semaine
- [ ] Bot Twitter/Bluesky qui publie les contradictions detectees
- [ ] Widget embeddable pour les medias (iframe contradiction du jour)
- [ ] API publique documentee pour les journalistes

### 5.4 Institutionnel
- [ ] Presenter le projet a la DINUM (Direction du Numerique de l'Etat)
- [ ] Proposer NEXUS comme outil a Transparency International France
- [ ] Contact avec les equipes de Regards Citoyens, Projet Arcadie, Datan
- [ ] Candidater au prix "Civic Tech" (Prix Territoire Innovant, OGP Awards)

---

## Budget Transparent

| Poste | Montant | Priorite |
|---|---|---|
| Mac M5 Ultra 512GB | ~14,000 EUR | Sprint 4 |
| Nom de domaine .fr (10 ans) | ~120 EUR | Sprint 4 |
| Certificat SSL | Gratuit (Let's Encrypt) | Sprint 4 |
| Consultation avocat presse | ~200 EUR | Sprint 4 |
| Total | **~14,320 EUR** | |
| Marge imprevus (5%) | ~680 EUR | |
| **Objectif Open Collective** | **15,000 EUR** | |

**Cout de fonctionnement mensuel : ~30 EUR** (electricite + internet)
Pas de cloud, pas d'abonnement, pas de SaaS. Le serveur est autonome.

---

## Timeline

| Semaine | Sprint | Livrable |
|---|---|---|
| S1 | Sprint 0 | Repo public, README, LICENSE |
| S1-S2 | Sprint 1 | Video demo, Discord, branding |
| S2-S4 | Sprint 2 | Open Collective, campagne, premiers dons |
| S4-S6 | Sprint 3 | Tests, CI, stabilisation |
| S6+ | Sprint 4 | Serveur dedie (quand finance) |
| Continu | Sprint 5 | Croissance, couverture, impact |

---

## Valeurs du projet

1. **Transparence totale** — code ouvert, finances ouvertes, donnees ouvertes
2. **Zero cloud americain** — tout tourne en local, juridiction francaise
3. **Zero censure** — LLM local uncensored, aucun filtre sur les resultats
4. **Zero opinion** — faits sources, pas de jugement, presomption d'innocence
5. **Souverainete numerique** — le serveur appartient aux citoyens, pas a une entreprise
6. **Perennite** — AGPL garantit que le code reste libre pour toujours
