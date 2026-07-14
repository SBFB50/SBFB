# Sprint 82 Phase I — Preflight (G8)

Date : 2026-07-14. Phase I ferme la doc-dette sécurité S81 : Track H
(S81-H-1/H-2/H-3), VALIDATED_BLUEPRINT iroh stale (G-D5-1), K-R-7
qualificatifs sur-larges, K-2 prose résiduelle, claim cargo-audit
tranché, catalog_len=0 seeder consigné (PO-8, décision fermante §6.2.1).
Preflight ultracode = Workflow 12 agents (6 scans S2-trackH / S2-KR7-K2 /
S4-blueprint / S1b-cargo-audit / S3-catalog-len / S2-decisions-gates
+ 6 vérifications adversariales par scan, pipeline sans barrière,
opus-4-8[1m]). Toutes les ancres ci-dessous re-vérifiées au disque le
2026-07-14 par DEUX passes indépendantes (arbre propre, tip `32a23f6`).
Une fabrication attrapée par la vérif adversariale (citation
`check-frontier-contracts.sh:205` inventée — retirée) + 5 corrections
d'ancrage intégrées (off-by-one review K :194→:195 ; rust/PATTERNS
:3350→:3351 ; HARDENING cargo-deny :3→:28 ; frontier :34→:36 ;
shell/PATTERNS §P36→§P37).

## Verdict: PLAN-ADAPT

Le plan est exécutable, aucune décision Day-0/PO n'est contredite
(aucun DESIGN-CONFLICT : PO-8 accept-and-document est appliqué tel
quel, la subsomption cargo-deny honore la décision gelée S18 D3, le
recadrage HARDENING §3 S73 reste intangible). Mais **sept faits du
plan sont incomplets ou inexacts** et imposent une exécution corrigée :

1. **La prose exacte de S81-H-1/H-2/H-3 est INTROUVABLE — la consigne
   « Ré-extraire le texte exact H-1/H-2 des reviews » (`sprint82_plan.md:232`)
   repose sur une prémisse fausse.** Les trois IDs n'existent que comme
   identifiants sans corps : `sprint81_audit_findings.md:180-181`
   (« Findings (tous PRE-EXISTANTS, aucun cause par S81) : S81-H-1 (P2),
   S81-H-2 (P2), S81-H-3 (P3). ») + tables :224/:225 ; grep
   `S81-H-[123]` sur TOUT `.planning/` (y compris les 30+ phase-reviews
   archivées v2.1) = **0 hit de prose descriptive** (IDs, routage ledger,
   désambiguïsation homonyme Sprint 9 uniquement — vérifié par grep
   indépendant du vérificateur adversarial). La seule section détaillée
   de l'audit S81 est « ### Les 4 P1 en detail » (:227). Même classe
   exactement que Track C en Phase H (audit diff-first anti-anchoring,
   sortie brute jamais persistée). Le mapping H-1→THREAT_MODEL /
   H-2→LOOPBACK / H-3→HARDENING_ROADMAP (`sprint82_plan.md:225-226`)
   est une INFÉRENCE de l'auteur du plan, non ancrée per-item.
   → **Exécution : consigner INTROUVABLE (preuves ci-dessus), puis
   RE-DÉRIVER une passe de fidélité hardening-drift sur les 3 docs
   (corpus entier, le mapping ne sert que de cadrage), étiquetée
   « ré-audités Phase I » — jamais « ré-extraits ». Si la passe ne
   révèle aucun drift vivant, REQUALIFIER H-n CLOSED-requalifié
   (les 3 docs ont été revalidés en S81 : HARDENING last_validated
   2026-07-08, LOOPBACK 2026-07-11, THREAT §1.1 iroh =1.0.1 — un
   résultat « 0 drift » est plausible et acceptable). Désambiguïsation
   à écrire noir sur blanc : S80-H-1/2/3/4 (LOT-LOOPBACK-DOC, CLOSED
   S81 Phase G) ≠ S81-H-1/2/3 (Track H audit) ; homonymes Sprint 9
   H-3 wheel-drift et SEC-H-1 (phase-h review S81). NE PAS re-fabriquer
   le candidat « hickory DoS-class » : `HARDENING_ROADMAP.md:28`
   distingue DÉJÀ les classes (0119/0104 = DoS, 0098/0099 = laxité
   name-constraints) — le GAP Codex Phase G a été corrigé in-phase.**

2. **SIGNAL MÉTA (2 PLAN-ADAPT consécutifs, même classe).** Phase H
   (Track C) et Phase I (Track H + K-2) butent sur la MÊME prémisse
   fausse du plan : « ré-extraire » une prose d'audit qui n'a jamais
   été persistée. Cause racine : l'audit gate S81 n'a détaillé que les
   4 P1 ; tous ses P2/P3 sont des IDs nus. Conséquence prospective :
   les phases restantes qui « ré-extraient » (Phase J, Tracks F/I/J)
   doivent d'emblée planifier une RE-DÉRIVATION, pas une ré-extraction.
   À consigner dans le commit body et à porter au wrap-up (amendement
   process candidat : l'audit gate persiste désormais une ligne de
   prose par P2/P3, pas seulement les P1).

3. **K-2 « prose résiduelle » n'est PAS un finding persisté — c'est un
   raccourci de plan** (`sprint82_plan.md:228` + covers-ID
   S82-DC-S81-K2-PROSE:234 ; aucun « K-2 » dans la review K — les
   findings review sont S81-K-R-1..R-21). Les deux candidats concrets
   sont les résiduels Codex round 3 (`sprint81_phase_k_codex_review.md:37-40`) :
   (a) P2 provenance « operator-corroborated » sur 4 surfaces ;
   (b) P3 commentaire de test all-zero echo (`shard.rs`). L'addendum
   review (:423-452, span 430-431) les déclare **corrigés post-verdict
   dans la même fenêtre pré-commit**.
   → **Exécution : VÉRIFIER-et-fermer (0 fabrication) : confirmer au
   disque que les 4 surfaces portent le qualificatif de provenance et
   qu'aucune prose DOC obsolète ne subsiste ; consigner K-2 = raccourci
   de plan, prose déjà close S81. Le commentaire shard.rs est du CODE →
   hors périmètre docs-only, noter sans éditer.**

4. **K-R-7 : 4 sites confirmés ouverts, dont un site ADJACENT non cité
   par le finding.** Texte réel du finding : `sprint81_phase_k_review.md:282-286`
   (+ corroboration :195 — l'off-by-one :194 du scan corrigé). Sites :
   (a) `SHARD_PROTOCOL_SPEC.md` §5.2 :273 « (byte-identical to S77
   Phase B) » — à borner : vaut pour le chemin echo/transport côté
   DRIVER (`model_digest == 0`) ; `ShardProtocol::accept` intercepte
   pour TOUTE session de l'ALPN partagé (miroir correct : §P73(1),
   `PATTERNS.md:4227-4238`). NB : :255 contient DÉJÀ « of a real
   session » — la correction SPEC porte sur byte-identical SEUL, pas
   d'ajout redondant. (b) `THREAT_MODEL.md:148` (sommaire §5
   attestation, plage :144-149) — qualificatif « d'une session réelle »
   absent (grep « session réelle » THREAT = 0 hit). (c)
   `THREAT_MODEL.md:1558` §16 « le chemin echo (digest zeros) est
   exempte byte-identique » — MÊME classe sur-large, NON citée par
   K-R-7 (qui ne nomme byte-identical qu'à SPEC §5.2) : borner par
   COHÉRENCE sous le même covers, en signalant honnêtement « non cité
   par le finding » (les 2 autres « byte-identique » de THREAT :993
   /nodes-S75 et :1228 convergence-E3 sont HORS classe — ne pas
   toucher ; idem LOOPBACK :85 /nodes-S75). (d) `LOOPBACK_ENDPOINTS_TRUST_TIERS.md:92`
   row /generate — « attestation loaded-stage fail-closed à chaque
   stage-link » sans « d'une session réelle » (grep = 0 hit).
   → **Exécution : 1 phrase d'honnêteté par site, 0 code. Ancrer par
   SYMBOLE (`### 5.2 Stage attestation`, « ATTESTATION du stage
   charge », row `/generate`), re-relire chaque plage au disque au
   moment de l'édit (les :148/:92 sont des numéros S81 stables mais
   non garantis). Bump `last_validated` LOOPBACK → 2026-07-14 avec
   entrée datée (édit de prose de posture §3) ; THREAT_MODEL n'a pas
   de front-matter trigger.**

5. **G-D5-1 : 3 occurrences textuelles « 0.97 », pas 2** (ligne 156 en
   contient DEUX : cellule version « 0.97 (1.0 pas encore) » + note
   « SBFB deja pinne 0.97 » ; ligne 157 UNE : « iroh-gossip 0.97
   native »). Cibles RE-PROUVÉES au lock : iroh **1.0.1**
   (`Cargo.toml:48` `=1.0.1`, `Cargo.lock:3920`), iroh-gossip
   **0.101.0** (`Cargo.toml:50`, lock :4096) ; blobs 0.103.0, docs
   0.101.0. `VALIDATED_BLUEPRINT.md` n'a AUCUN front-matter
   (`last_validated`/`triggers_revalidate` absents — vérifié) : édit
   pur-corps, **ne pas fabriquer un champ de métadonnées**. Secondaires
   du MÊME doc surfacés par le scan (cellules snapshot-écosystème
   dérivées au lock) : blake3 1.8.3→1.8.5 (:218, lock :717),
   ed25519-dalek 2.1.x→2.2.0 (:71, lock :2102) + URL docs.rs 2.1.0
   (:667).
   → **Exécution : corriger les 3 occurrences 0.97 (critère machine
   `grep '0.97'` = 0 hit) ; décision d'exécution motivée ici : corriger
   AUSSI les 2 cellules secondaires + URL (le critère du plan « aucune
   contradiction code↔doc résiduelle » couvre le même doc ; extension
   bornée à 3 lignes, consignée au commit body — pas de re-validation
   crate-par-crate au-delà). Les crates aspirationnelles (wasmtime 43,
   arti 2.2, nym…) restent INTACTES.**

6. **Claim cargo-audit TRANCHÉ : NOTE DE SUBSOMPTION, ne PAS câbler.**
   État réel re-prouvé live : `cargo audit` → « no such command » ;
   cargo-deny **0.19.2 installé** ; gate advisisory réel CÂBLÉ =
   `cargo-deny check` (GHA `.github/workflows/supply-chain.yml:70-74`
   EmbarkStudios/cargo-deny-action@v2 + smoke local
   `scripts/ci-smoke/supply-chain-green.sh:38`) ; `deny.toml:41-45`
   `[advisories]` version 2, MÊME base `rustsec/advisory-db` que
   cargo-audit + `yanked = "deny"` (ferme le seul avantage résiduel de
   cargo-audit) ; 6 ignores motivés (4 hickory-0.24 → Phase K + 2
   quick-xml via iroh). Décision GELÉE S18 D3 (`sprint18_kickoff.md:238-247`
   « cargo-deny comme seule CI step Rust supply-chain ») re-confirmée
   S81-G F6. Câbler cargo-audit = binaire redondant, zéro couverture
   advisory nette, contredit D3. Claims à corriger (tous relus
   verbatim) : `THREAT_MODEL.md:331` « DIFFERE S17+ » (FAUX : livré
   S18 via cargo-deny), :248 « scope cut S17+ » (résiduel H→L
   justifié), :366 roadmap « Sprint 17+ ajoute cargo-audit… » (livré
   S18) ; `VALIDATED_BLUEPRINT.md:378` (renommer cargo-audit→cargo-deny,
   laisser cargo-vet/osv-scanner :380/:382 comme futur réel) ;
   `POST_CHATONS.md:411` (littéralement vrai mais trompeur par
   omission — remplacer par la note de subsomption, édit chirurgical
   d'une puce, le doc snapshot n'est pas rafraîchi au-delà) ;
   `HARDENING_ROADMAP.md:889` annotation légère « LIVRE via cargo-deny
   (D3) » (:181 = prose de plan S18 historique, INTACTE — passé
   immuable). `security_posture.md:709` (`.planning/codebase/`,
   artefact GÉNÉRÉ /gsd:map-codebase, hors livrables nommés du plan) :
   NE PAS éditer, consigner hors-périmètre au commit body.
   **INTERDIT : affirmer « supply-chain green »** (statut de run GHA
   non vérifiable au disque) — écrire « câblé bloquant par config ».

7. **catalog_len=0 (PO-8) : consignation due, comportement CONFORME au
   design.** Mécanisme re-prouvé au code : le catalogue signé est bâti
   EXCLUSIVEMENT depuis `own_entries(&my_node_id)`
   (`build_sign_announce_directory`, `http.rs:1310/1321-1328` ;
   `own_entries` `browse.rs:605` filtre `node_id == my_node_id`) ; une
   app volontairement seedée garde le node_id de l'AUTEUR et n'est
   jamais un direct-entry (test `seed_voluntary_directory_only_app`,
   `http.rs:5770/5797`) → un pur-seeder a `catalog_len:0` par
   construction (verrou-4, `node_directory.rs:44-52`). Nature : trou de
   DÉCOUVRABILITÉ borné, PAS de sécurité (disponibilité des octets ≠
   découvrabilité de l'index ; si l'auteur disparaît, un pair frais
   perd le chemin de découverte via un pur-seeder même si les octets
   restent servis). Chaîne de reports PERSISTÉE verbatim :
   S75-G (origine, `sprint75_verification.md:150-152`) → S76 (2/3) →
   S77 (3/3) → S78 (« pas reporté », arbitrage requis) → S81-G-3
   (re-observé live flip H `bd5d680`). Compteur §6.2.1 #4 déjà statué
   au ledger (`sprint82_phase_e_ledger_reconciliation.md:242-243`).
   → **Exécution : consignation dans `THREAT_MODEL.md` §15.1 (bloc
   Résiduel FR nommé, après « Residuals S75 » :1005-1009) + nouveau
   sous-bloc EN dans `docs/rust/PATTERNS.md` §P59 (découverte PULL —
   home plus apte que §P58) : constat + décision PO-8 datée S82
   Phase I 2026-07 + rationale verrou-4 + risque résiduel borné +
   compteur de reports + conditions de réouverture ((a) section
   « seeded » distincte NON-autoritaire dans NodeDirectoryEntry =
   changement wire, ou (b) SearchManifest opt-in post-launch). Citer
   la date/phase, PAS le chemin planning (éphémère). Ancres corrigées
   par la vérif adversariale : rust/PATTERNS « is_own = false » à
   :3351 (pas :3350) ; le bloc shell est sous §P37 (pas §P36) — pas de
   3e duplication shell, le plan dit THREAT/PATTERNS.**

## Baselines et critères machine (avant-édit, 2026-07-14)

- `grep -c '0.97' docs/security/VALIDATED_BLUEPRINT.md` = **2 lignes /
  3 occurrences textuelles** (cible : 0).
- `bash scripts/check-sharding-docs.sh` = **exit 0** ; les 4 ancres
  SPEC gatées (`sbfb/shard/1`, `ShardGroupMintRequest`,
  `MountSessionRequest`, `ShardGenerateRequest`) vivent HORS §5.2 —
  les édits K-R-7 restent verts tant qu'elles survivent.
- `bash scripts/check-frontier-contracts.sh` = **exit 0** (anti-promise
  = crates/+web/src SEULEMENT : les docs sont tenus au présent-vrai
  par CONVENTION ; census DOMAIN 25 frozen — docs-only, aucun risque).
- Baseline wire S4 : **13 constantes** `*_VERSION` dans
  `nexus-core-rs/src`, toutes = 1 (`BLOB_VERSION = 0x01`). La regex du
  plan `_VERSION\s*[:=]\s*[0-9]+` est INSUFFISANTE (0 déclaration
  réelle matchée — le type `u16` suit le `:`) ; regex corrigée :
  `const\s+[A-Z0-9_]*VERSION[A-Z0-9_]*\s*:` (13 hits reproduits par 2
  passes indépendantes). Phase docs-only → delta attendu : 0.
- `PATTERNS.md:850-853` « routed to S82 Phase I » : flip présent-vrai
  → « closed S82 Phase I » à la livraison (dans cette phase).
- Note datée Track H à poser dans `sprint81_audit_findings.md` (miroir
  de la note Track C posée en Phase H :103-121).

## Contraintes intangibles vérifiées

- HARDENING_ROADMAP §3 (:156-174) recadrage S73
  P2-HARDENING-ROADMAP-META-STALE : enregistrement historique —
  INTANGIBLE, ne pas re-promettre un sprint ouvert.
- Passé immuable / présent-vrai (anti STALE-PHASE-K) : aucune promesse
  future dans les docs ; « corrigé S82 Phase I » = passé immuable une
  fois fait.
- Ancres SYMBOLE partout (root-cause « pointeur-qui-pourrit » Phase H).
- 0 wire bump, 0 dep, docs-only (les seuls fichiers touchés :
  docs/security/*.md, docs/protocol/SHARD_PROTOCOL_SPEC.md,
  docs/rust/PATTERNS.md, docs/community/POST_CHATONS.md,
  .planning/active/*.md).
- Invariant cardinal « héberger ≠ publier, seeder ≠ auteur » : la
  consignation catalog_len=0 le RENFORCE (aucune recommandation ne le
  contredit).
