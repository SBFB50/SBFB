# Sprint 59 — Design Review Board

**Date** : 2026-05-11
**Reviewer** : agent Explore independant (session fraiche)

## Scoring

D1 : ⚠️ — Source S21 (22j) bonne, mais choix log2 vs ln non documente, alpha EMA=0.97 absente de S21 (S21 suggere 0.95), pas de recherche P2P post-S21
D2 : ⚠️ — Source deploy.rs stable, mais SBFB.json schema inexistant et verification d'identite (node_id seul) qualifiee insuffisante pour "verified deploy"
D3 : ⚠️ — Source context7 windows-rs tres frais (2026-05-11), mais pas d'analyse UTF-16 encoding UB, windows-sys alternative non comparee
D4 : ⚠️ — Pattern GCRA existant (BrowseRequestLimiter), mais alternatives (token bucket, adaptive) non etudiees et quota 10/min sans justification empirique

Rigor signal G4 : 4 ⚠️ sur 4

## Detail par decision

### D1 — LT-1 Kudos-v2

- **Fraicheur source** : S21 (2026-04-19) = 22j. Acceptable (< 90j).
- **Angle mort 1** : log2() choisi mais S21 recommande formule generique `K * log(1 + x/K0)` sans specifier base. Litterature academique (Mo & Walrand, Kelly) utilise ln (naturel). Impact : multiplicatif pur, mathematiquement equivalent.
- **Angle mort 2** : EMA alpha=0.97 (half-life ~23j) absent de S21. S21 suggere alpha ≈ 0.95 (half-life ~14j a 1 tache/jour). Divergence non expliquee.
- **Angle mort 3** : Pas de verification si systemes P2P compute plus recents (BOINC CreditNew, Filecoin, Render Network) ont emerge post-S21.

### D2 — Verified deploy E2E

- **Source** : deploy.rs (S42, 679 LOC) complet et stable.
- **Angle mort 1** : SBFB.json schema inexistant (pas de JSON Schema ni struct documentee). Quels champs obligatoires? Comment node_id est rempli?
- **Angle mort 2** : node_id seul est une convention de nommage, pas une verification d'identite forte. La chaine de verification (SBFB.json → provenance) n'est pas documentee bout en bout.
- **Angle mort 3** : Alternatives intermediaires a Keyoxide non etudiees (Git GPG tag, did:key, DNS CNAME).

### D3 — Launcher MessageBoxW

- **Source** : context7 windows-rs consulte 2026-05-11 (tres frais).
- **Angle mort 1** : UTF-16 encoding non couvert. MessageBoxW requiert LPCWSTR (UTF-16 LE). Rust String = UTF-8. Conversion requise.
- **Angle mort 2** : HWND NULL vs invalid handle — que passe le launcher?
- **Angle mort 3** : windows-sys vs raw extern pas compare qualitativement (windows deja dans workspace daemon).

### D4 — Storage carries

- **Source** : S58 audit + BrowseRequestLimiter pattern (S56).
- **Angle mort 1** : REPLICATED_APPS hardcode (`["sbfb-ideas"]`). Plan post-v1.0 non documente pour ajout apps.
- **Angle mort 2** : Quota 10/min sans mesure empirique de frequence reelle Ideas Hub.
- **Angle mort 3** : GCRA vs token bucket vs adaptive pas compare. Choix pragmatique (reuse) mais pas justifie formellement.
