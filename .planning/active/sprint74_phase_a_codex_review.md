**Verdict Global**
Audit du working tree effectué. Résultat : **GAP partiel**, pas PASS strict. La Phase A est bien front-only et les tests ciblés passent, mais le livrable 4 ne respecte pas exactement le mapping demandé pour `reachable` sur une app locale.

Tests exécutés : `npx vitest run ...AvailabilitySheet... Deploy... BrowsedProject... AppShell... AddCoordinatorDialog...` → **5 fichiers passés, 36 tests passés**. Environnement : Node `v22.22.0`, npm `10.9.4`.

**Livrables**
1. **CONFIRME** — Carte succès publish : `web/src/pages/Deploy.tsx:212`, `:216`, `:224`, `:227-230`, `:234-240`, `:241-258`, hashes cachés sous toggle `:257-281`. Aucun champ hôte/cible visible dans le formulaire, seulement repo/projet/description `:99-141`.

2. **CONFIRME** — Ligne de vérité avant clic : `web/src/pages/Deploy.tsx:143-176`, avec texte exact `:174-175`; elle est avant le rendu conditionnel succès `:179`.

3. **CONFIRME** — Top-bar `Disponibilite` remplace `blob:<hash>` : `web/src/pages/BrowsedProject.tsx:350-376`; test anti-blob `web/src/pages/__tests__/BrowsedProject.test.tsx:521-537`.

4. **GAP partiel** — `AvailabilitySheet` existe et couvre les sections : auteur `web/src/components/AvailabilitySheet.tsx:112-130`, état `:137-180`, seeders `:187-260`, copies `:268-285`. Mais le mapping `reachable` n’est pas strictement celui demandé : pour `isOwn`, il affiche `En ligne (vu de ton noeud)` `:152-155`, pas `En ligne — joignable par tous`. Pour distant, le libellé demandé est bien présent `:155`.

5. **CONFIRME** — Toggle `Garder en ligne` lecture seule : `pressed`, `disabled`, aucun handler réseau `web/src/components/AvailabilitySheet.tsx:217-226`; test fetch non appelé `web/src/components/__tests__/AvailabilitySheet.test.tsx:129-147`. `rg keep-online/keep_online` sur API/Rust ne trouve aucune route.

6. **CONFIRME** — App distante : CTA présentationnel inerte, rendu en `<div>` avec badge `Bientôt`, pas bouton actif : `web/src/components/AvailabilitySheet.tsx:249-259`; test remote `:176-183`.

7. **CONFIRME** — Rappel hors-ligne own-only : `isOwn`, `isOffline`, clé session par app `web/src/pages/BrowsedProject.tsx:197-225`; bandeau dismissible `:458-480`; test own/remote/dismiss `web/src/pages/__tests__/BrowsedProject.test.tsx:483-519`.

8. **CONFIRME** — Placeholder app tombée + prefill `/deploy` : condition distant sans archive + `https` `web/src/pages/BrowsedProject.tsx:538`; lien encodé `:555-561`; tests `:546-568` et rejet scheme non-https `:570-591`.

9. **CONFIRME** — Rename UI visible : exemples vérifiés dans Deploy `web/src/pages/Deploy.tsx:39-43`, Browse `web/src/pages/Browse.tsx:35-39`, Network `web/src/pages/Network.tsx:42-53`, Curators `web/src/pages/Curators.tsx:35-39`, Projects `web/src/pages/Projects.tsx:27-28`, ProjectDetail `web/src/pages/ProjectDetail.tsx:56-65`, Onboarding `web/src/pages/OnboardingEmpty.tsx:62-63`, AppShell nav/picker `web/src/components/AppShell.tsx:59-63`, `:243`, `:263`, `:306`, AddCoordinator `web/src/components/AddCoordinatorDialog.tsx:125-136`, Invites `web/src/components/project/InvitesTab.tsx:76-78`, Overview `web/src/components/project/OverviewTab.tsx:48`, CommandPalette `web/src/components/command-palette/CommandPalette.tsx:174-186`, `:234`. Identifiants/persistence conservés : `knownCoordinators`, `activeCoordinatorUrl` `web/src/stores/projectStore.ts:49-50`, clé `nexus-grid:shell:v1` `:146`.

10. **CONFIRME pour les nouveaux usages Phase A** — Greffe D protégée par helper local `isHttpsUrl` `web/src/pages/BrowsedProject.tsx:853-860`, utilisé avant le lien `:538`, `:555-561`. Note résiduelle hors Phase A : des ancres `repo_url` préexistantes restent non gardées dans Browse/BrowsedProject (`web/src/pages/BrowsedProject.tsx:430-436`, `web/src/pages/Browse.tsx:469-476`), mais le diff montre qu’elles ne sont pas introduites par cette greffe.

11. **CONFIRME** — Les 6 tests demandés existent : `availability_sheet_renders_author_state_seeders` `web/src/components/__tests__/AvailabilitySheet.test.tsx:67`, `availability_state_maps_reachable_unreachable_unknown` `:93`, `keep_online_toggle_readonly_in_phase_a` `:129`, `publish_success_card_folds_hashes` `web/src/pages/__tests__/Deploy.test.tsx:119`, `offline_reminder_only_for_own_apps_dismissible` `web/src/pages/__tests__/BrowsedProject.test.tsx:483`, `coordinator_renamed_to_node_in_shell` `web/src/components/__tests__/AppShell.test.tsx:52`.

**Invariants**
- Aucun `.rs` modifié : `git diff --name-only -- '*.rs'` vide.
- Aucun changement de `web/src/api/daemon.ts`, bridge protocol, schema tabview, Cargo/package files dans le diff ciblé.
- Aucun `/api/daemon/keep-online` réel trouvé ; le toggle ne déclenche aucun fetch.
- `rg -i "coordinateur"` hors tests ne renvoie aucune chaîne UI visible. Les occurrences restantes sont identifiants/commentaires/tests.

**GAP**
- Livrable 4 uniquement : mapping `reachable` non strict pour app locale (`En ligne (vu de ton noeud)` au lieu de `En ligne — joignable par tous`).
