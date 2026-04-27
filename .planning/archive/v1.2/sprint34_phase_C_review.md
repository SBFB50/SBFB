# Phase Review — Sprint 34 Phase C

## Verdict : PASS (1 P2 + 1 P3)

Rigor signal : 2 findings documentés / >=1 requis pour PASS.

## Staging check
- Phase fichiers : 7 (Info.plist NEW, .desktop NEW, bundle-macos.sh NEW,
  nexus-launcher.png NEW, install-node.sh modified, preflight, review)
- Preflight + review : docs planning séparés ✅

## Suites
- Rust nextest : 902/902 pass ✅
- Rust fmt/clippy : clean ✅
- Python : 195 + 409+36f + 46 ✅ (pré-existant)
- Frontend : 267 + 42+2f + size 7/7 + en-strings ✅ (pré-existant)

## Delta tests
- 0 (config/script phase, no code changes)

## Scope cuts verification
- Code signing macOS : ✅ non touché (right-click bypass documenté)
- .deb/.rpm packages : ✅ non touché
- Tous les autres §7 : ✅

## Findings

### P2-C-1 : .icns absent, fallback .png dans .app bundle

macOS .app bundle utilise le .png comme fallback car `iconutil`
(conversion PNG→ICNS) n'est disponible que sur macOS. Le script
bundle-macos.sh documente ce fallback. L'icône fonctionne dans
Finder mais n'est pas optimale (pas de multi-résolution macOS).

**Action** : créer le .icns sur une machine macOS, ou utiliser
un outil tiers cross-platform. Carry S35.

### P3-C-1 : .desktop Exec path hardcodé /opt/nexus-grid

Le .desktop template a `Exec=/opt/nexus-grid/...` mais install-node.sh
le remplace dynamiquement via sed. Le path hardcodé est un fallback
raisonnable si l'utilisateur copie le fichier manuellement.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S35 : P2-C-1 .icns macOS
