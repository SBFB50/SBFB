# Sprint 22 — Carry summary (cap G7 + LT-2 reclassification + wire-up debts + items deferrés)

**Écrit** : 2026-04-19 (ouverture Sprint 22).
**Source** : audit gate S21 findings `.planning/archive/v1.2/
sprint21_audit_findings.md` §5 + kickoff S22 §4.5 G1
acknowledgements.
**Règle cap** : `docs/claude/README.md §6.2.1` (G7 cap max 2
carry-overs par sprint).

---

## 1. Cap G7 — 1/2 slots utilisés

### Slot 1 — T-NN+2 iframe Rust-wasm (hors cap formel PATTERNS §P34)

**Origine** : S21 Phase E `49f0d32` tech debt batch closeout
PATTERNS §P34.

**Titre** : Iframe PII SDK Rust-wasm realignement Option G (vision
initiale D2 S21 rejetée factuellement).

**Rationale rejet Rust-first iframe S21** :
- tract 0.22.1 teste opset 9-18 vs GLiNER export opset 19
  (DisentangledSelfAttention DeBERTa-v3 non documenté)
- tract `wasm32-unknown-unknown` browser non documenté officiellement
  (seul wasm32-wasi wasmtime supporté)
- zero precedent production tract + GLiNER + wasm-bindgen browser
- `gline-rs v1.0.1` (Rust GLiNER mainstream 2026-01) a choisi `ort`
  pas tract

**Triggers re-activation S22+** :
- (a) tract opset 19 coverage (`sonos/tract` GitHub tag release
  notes), OR
- (b) `ort` wasm32-browser target stable release (Microsoft
  `onnxruntime` stable npm + wasm32 compat ≠ node-only), OR
- (c) `gline-rs` wasm-bindgen target availability (fbilhaut/gline-rs
  roadmap item), OR
- (d) NVIDIA Open Model License vs AGPL-3.0 clarification
  (affecte `nvidia/gliner-PII` adoption)

**Status S22** : **hors cap G7 formel** (PATTERNS.md tech debt
tracking, pas carry-over). Tracé PATTERNS §P34, revu au Phase 0
audit chaque sprint.

---

### Slot 2 — LIBRE

Réservé aux findings carry-overs éventuels Phase F S22 (audit
findings P2/P3 non-résolus in-phase).

Remplissage attendu post-audit gate S22 Phase 0 Sprint 23.

---

## 2. LT-2 Meta-1 Radicle-v1.0 — RECLASSIFICATION régularisée

### Timeline carry-over

| Sprint | Status carry G7 | Notes |
|---|---|---|
| S18 Phase E3 `95807b1` | Open | Pivot Codeberg mirror (Radicle P2P public-only incompat pre-launch repo privé) |
| S19 Phase F | Re-carry G7 | 1re consécutive |
| S20 Phase F | Re-carry G7 | 2e consécutive |
| S21 Phase F `7887471` | Re-carry G7 (4e cumul) | 3e consécutive — **règle §6.2.1 trigger auto-promote → LT mais oublié Phase F S21** |
| **S22 kickoff (ce commit)** | **Rattrapage LT-2 reclassification** | Section `docs/release/ROADMAP_COMMITMENTS.md §LT-2` créée + sort cap G7 formel |

### Règle §6.2.1 appliquée (rattrapage)

> « Reclassification automatique : un carry-over present dans 3
> carry_summary consecutifs (cf. §6.2.1) est promu long-term
> commitment en Phase F wrap-up du sprint N+2 et sort du cap G7 »

Meta-1 Radicle = 4e consécutif en entrée S22. Rattrapage via
kickoff §4.5 P3-G1-7 acknowledgement (S21 audit gate PASS confirmé
`96a953b`) + création §LT-2 dans le même commit d'ouverture S22.

### Condition de déclenchement LT-2 (hors cap G7 formel)

**Tag `v1.0` go-live posé sur master** : déclencheur unique. Au
moment du tag, réouvrir Meta-1 comme carry actif dans le sprint
courant (reintegration G7 cap du sprint qui pose le tag).

### Runbook pointer

`docs/release/MIRROR_FALLBACK.md §3 "Flip sequence Codeberg →
Radicle"` — procédure self-contained documentée depuis S18.

### Status S22

**Sort cap G7 formel**. Suivi via registre LT-2 revu annuellement
ou post-trigger v1.0. Pas d'action requise pendant S22 sauf
trigger v1.0 inattendu.

---

## 3. Wire-up debts absorbés en phases S22 dédiées (PAS carry-overs formels G7)

Distinction workflow `docs/claude/README.md §6.2.1` :

- **Carry-over formel G7** : item non-livré fonctionnellement que
  l'on porte au sprint suivant en re-engagement explicite.
- **Wire-up debt** : fonctionnalité livrée mais primitive non-câblée
  au chemin critique runtime. Absorption en phase dédiée S+1 est un
  **completement** pas un re-engagement.

**Précédent validé** : S20 Phase C `16b94ba` PoW runtime wire
carry S19 A-2 absorbé sans compter slot G7.

### Wire-up debts S21 → S22 absorbés

| Finding | Phase S22 | LOC | Status |
|---|---|---|---|
| **P2-S21-1** RateLimiter primitive non-câblée engine | Phase A | ~150 Rust wire | Absorbé, pas slot G7 |
| **P2-S21-2** RateLimitPolicy hot-reload incomplet | Phase A | ~80 Rust Arc swap | Absorbé couplé P2-S21-1 |
| **P2-S21-3** OnnxModelHandle scaffold returns [] | Phase B | ~350 TS decoder | Absorbé, pas slot G7 |
| **P2-S21-6** HARDENING §3 S21 wording fix | Phase A chore planning | Docs | Absorbé chore opening S22 |
| **P3-S21-4** rate_limit_policy.toml.sample absent | Phase A | ~20 TOML config | Absorbé Phase A |

### Process fixes Phase F S22

| Finding | Phase S22 | LOC | Status |
|---|---|---|---|
| **P2-S21-4** Phase F review → audit_plan carry règle | Phase F | README §4.X ~30 LOC docs | Absorbé |
| **P2-S21-5** Hook coverage Phase D bypass audit trail | Phase F | GHA workflow ~80 LOC + log | Absorbé |

---

## 4. Items deferrés S23 (chore planning HARDENING §3 updates)

**NON comptés carry-overs G7** car ce sont des scope-cuts documentés,
pas des re-engagements. Chore planning dédié dans le commit
d'ouverture S22 met à jour HARDENING §3 S22 + S23 + S24.

### Redundancy voting 3-worker majority (item §3 S22 ligne 268-269)

**Raison deferr S22→S23** : mitigue C-ResultSpoof tier max **T5**
(§1 threat matrix). Surdimensionné 3 gates au-dessus du Gate 2
cible (T0-T2 TransLingua/FamilyScan). BOINC/Folding@home ont
opéré 1-worker production 20 ans. Gate 3 track explicite §7 ligne
554 (S27 Sybil mature → S29 audit → Gate 3).

**Co-deferrer dependency S24 ligne 311** : `S22 redundancy voting`
→ `S23 redundancy voting`.

**Scope-cut S23** : drop Exponential cooldown (redondant avec
Couche 1 age gate) OU reporter Traffic padding design doc S28
(aligné Nym mixnet).

### Sandbox tool-calling allow-list (item §3 S22 ligne 267)

**Raison deferr post-S25** : pas de surface tool-call live
aujourd'hui (seul S20 structured output existe, pas de tool-
registry ouvert). OWASP LLM06:2025 Excessive Agency ne se
déclenche PAS sans tool-call live. Pattern « allow-list seul
insuffisant » (SoK arxiv 2601.17548, bypass ceiling ~85%)
implique que allow-list doit venir avec container-sandbox
(blob-serve iframe CSP déjà en place S12, mais scope différent
LLM tool-call).

**Re-évaluation trigger** : quand tool-registry LLM ouvert
(estimation S25 RAG ou S28+).

---

## 5. S27 pivot roadmap (meta-track chore planning)

**Item §3 S27 ligne 371-372 original** : « Sybil kudos-weighted
mature : trust-web bootstrapped par Amnesty-class ONG pour Gate 4 ».

**Flag FAIRNESS_VISION §7 implicite** : même conflit que S22 item 1
original. « Kudos-weighted » câble Matthew effect.

**Pivot documenté chore planning S22** : ligne 371-372 réécrite
« Couche 3 mature (multi-forge cross-validate + trust-web Amnesty
integration, remplace kudos-weighted flag FAIRNESS) ».

**Cohérent timeline** :
- S23 design doc finalisé Couche 3 (RFC émis S22 Phase C)
- S25-S26 implem parser `git log --show-signature` offline + cache
  LRU SQLite
- S27 finalisation + trust-web Amnesty integration (~700 LOC)
- S29 audit externe Cure53/ToB stress-teste composition 3 couches
  → Gate 3 unlock

**Dependency preserve** : S27 dep `S22 Sybil base` + `S26 Tor
complete` respectée ; S29 dep `S27 Sybil mature` respectée.

---

## 6. Cap G7 final S22

| Slot | Item | Status | Cap consumé |
|---|---|---|---|
| 1 | T-NN+2 iframe Rust-wasm | Hors cap formel PATTERNS §P34 | 0 |
| 2 | LIBRE (reserve findings post-audit S22) | — | 0 |
| hors cap | LT-2 Meta-1 Radicle-v1.0 reclassification | Régulier §6.2.1 | 0 |

**Cap G7 S22 consommé : 0/2 slots formels** (+1 hors cap PATTERNS
§P34, +1 LT-2 reclassification).

Sprint 22 peut absorber jusqu'à 2 nouveaux carry-overs fin-de-
sprint si audit findings S22 identifient P2+ non-résolvables
in-phase.

---

**Fin carry summary S22**. Liste finale validée post-G1
acknowledgement kickoff §4.5 + chore planning opening S22.
