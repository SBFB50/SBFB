# T1 — Script kiddie / troll anonyme

**Tier** : T1
**Budget** : <1k$ (materiel + cloud jetable)
**Timeline** : heures a jours
**Skill** : suit des tutos, pas de 0-day
**Ecrit** : Sprint 17 Phase A (2026-04-14)

---

## 1. Profil

Adolescent ou jeune adulte avec Kali Linux bootable, discord de
"hackers", compte Telegram pour echanger des scripts. Action
majoritairement opportuniste : trouve une cible par scan
aleatoire, joue avec 2-3 jours, passe a autre chose si ca
resiste. **Zero 0-day achete** — uniquement outils publics et
CVE du mois en cours.

Variante "troll" : meme profil technique mais motivation
harassment ciblee (ex : contributeur OSS vocal sur Twitter qui
irrite T1). Peut rester focus sur une cible plus longtemps (jours
a semaines) mais sans escalade en sophistication.

## 2. Capabilites techniques

**Outils utilises** :

- Nmap (scan port + OS fingerprint)
- Metasploit / Cobalt Strike cracked
- Burp Suite Community / OWASP ZAP
- Scripts Github public (exploit PoCs publies apres CVE)
- Crackmes / stresser web (DDoS hire ~50$/j)
- Shodan / Censys (recon passive)
- SQLmap, dirbuster, sublist3r

**Ce qu'il peut faire** :

- Port scan externe / internal si trouve RCE web
- SQL injection, XSS, CSRF basiques
- Credential stuffing (listes leaked)
- DDoS L4/L7 limite (~10-50 Gbps)
- Defacement web
- Basic phishing (kit Grabify / Evilginx)

**Ce qu'il NE peut PAS faire** :

- Developper un nouveau exploit
- Reverse-engineer un binaire non trivial
- Compromettre une CA publique
- IMSI catcher, cable transatlantique tap
- Compromettre Google / GitHub / Cloudflare

## 3. Budget & timeline

- Materiel : PC gaming + VPS 5-20$/mois jetable
- Outils : principalement gratuits ou cracked
- Zero-days : **zero** (marche gris demarre a 50k$, hors budget)
- Temps : weekends, vacances scolaires
- Persistence : <1 semaine sur une meme cible (sauf trolling cible)

## 4. Motivations

- **Clout** : screenshot dans discord, bragging rights
- **Curiosite** : "je me demande ce qui se passe si..."
- **Vengeance** : harcelement d'un contributeur qui l'a humilie
- **Argent marginal** : ransomware petit ticket (~500$ Bitcoin),
  credential stuffing pour revendre comptes
- **Politique basique** : defacement de sites qu'il deteste

## 5. Tactiques typiques contre SBFB

| Attaque | Mecanique | Probabilite |
|---|---|---|
| Scan port loopback | Shodan + test localhost variants | Haute |
| Extension browser malveillante publiee | Chrome Web Store, promo discord | Haute |
| CSRF sur daemon `/deploy-from-repo` | HTML piege dans un blog | Moyenne |
| DNS rebind sur daemon | rebind.it service | Moyenne (tente Sprint 16 CVE-2025-49596 mitige) |
| Fake app "free V-Bucks" publiee sur SBFB | Deploy-from-repo abuse avant Keyoxide enforce | Moyenne |
| Flood iroh gossip avec fake announcements | Requires node_id Ed25519 — barrier | Basse |
| Defacement iframe via app exploit | CSP `connect-src 'none'` mitige | Basse |

## 6. Observable indicators

T1 laisse des empreintes grossieres :

- User-agent Python-requests / curl / Nmap dans les logs daemon
- Pattern de requetes sequentielles sans backoff (no rate limit awareness)
- Tentatives paths obvious (`/admin`, `/.env`, `/backup.sql`)
- Origin non-matchant + tentatives 5-10 requetes avant abandon
- IP residentielle ou VPS low-cost (AWS Lightsail, Hetzner)
- Commits GitHub fake avec dates mal forges

## 7. Mitigations SBFB actuelles

**Livre** :

- Bearer 256-bit + Host allowlist + Origin check (Sprint 16
  Phase A) : bloque la majorite des T1 qui tentent loopback.
- SO_PEERCRED UDS + DACL Named Pipes (Sprint 16 Phase B) :
  empeche process cohabitant avec autre user.
- CSP `connect-src 'none'` + iframe sandbox sans
  `allow-same-origin` (Sprint 12) : une app vulnerable ne peut
  pas pivoter vers le reseau local.
- Verified deploy Keyoxide (Sprint 14) : pas de zip upload direct,
  oblige repo public → barrier pour fake apps.
- Provenance signee Ed25519 (Sprint 14) : T1 ne peut pas forger
  une provenance valide sans la cle.

**Partiel** :

- Rate limiting loopback : pas de throttling par IP (loopback
  single-user, threat model local limite).
- No fail2ban-like ban apres N tentatives echouees (Sprint 18+).

**Absent** :

- Pas de CAPTCHA / rate-limit sur deploy-from-repo cote coordinator
  (Sprint 19 selon roadmap).
- Pas de reputation scoring pour nouveaux repos (Sprint 22+).

## 8. Priorisation

T1 **est deja bien couvert post-Sprint 16**. Les vecteurs
residuels sont :
- Rate-limit deploy-from-repo (Sprint 19 item)
- Fail2ban loopback pour le defense-in-depth (Sprint 18 Phase C)

## 9. Mitigations obligatoires par Gate

| Gate | Requirement T1 |
|---|---|
| 1 (DnD Forge) | Sprint 16 suffit |
| 2 (TransLingua) | + rate-limit deploy + banlist auto |
| 3 (PolitiScan) | + CSP report-uri + anomaly detection passive |
| 4 (LibanLive) | + all above + honeypot endpoints pour detecter T1 scans |

## 10. References

- Verizon DBIR 2024 (section "Basic Web Application Attacks")
- MITRE ATT&CK Initial Access TA0001
- "Script Kiddies vs Professionals" (HackerOne blog, 2023)
