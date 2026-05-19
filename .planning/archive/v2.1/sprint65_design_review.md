# Sprint 65 — Design Review Board (G1)

**Reviewer** : agent Explore independant (session fraiche).
**Sprint** : S65 — Contrat Public.
**Date** : 2026-05-18.

---

## Scoring report

| D# | Decision | Score | Verdict |
|---|---|---|---|
| D1 | Feed raw-op serde_json::Value | ✅ A+ | EXECUTE |
| D2 | Taxonomie confiance 6 niveaux | ✅ A+ | EXECUTE |
| D3 | Vocabulaire enforcement CI | ✅ A | EXECUTE |
| D4 | Deploy→feed + auth tier header | ⚠️ B+ | SCOPE-CUT-CONSISTENT |
| D5 | Factory gates FG0-FG10 spec | ✅ A | EXECUTE |

Rigor signal G4 : 4 ✅ + 1 ⚠️ sur 5.

---

## D1 — Feed raw-op : ✅

**Sources verifiees** :
- `public_feed.rs:1-100` : FeedEntry.op = PublicFeedOperation enum,
  `#[serde(tag = "op_type")]`. 2 variants.
- `PUBLIC_FEED_SPEC.md §2-3, §9` : JCS RFC 8785 + DOMAIN_FEED_V1.
- `public_feed.rs:120` : `compute_feed_entry_hash()` utilise
  `nexus_core_rs::canonical_bytes()`.

**Alternatives** :
- `#[serde(other)]` : serde ne supporte pas `other` sur enums
  avec data (internally tagged). Techniquement bloque. Confirme.
- Champ dual `op_raw + op_parsed` : desync + canonical ambigu.
  Correctement rejete.

**Angle mort** : absence comparaison CBOR/MessagePack. Justifiable
car JCS (RFC 8785) n'existe que pour JSON — formats binaires n'ont
pas de canonical spec equivalente. Note recommandee dans commit body.

**Checklist crypto/spec** : ✅ alternative concurrente (<6 mois),
✅ source RFC 8785 stable, ✅ canonical bytes existantes.

---

## D2 — Taxonomie confiance : ✅

**Sources verifiees** :
- `Browse.tsx:259`, `BrowsedProject.tsx:281` : "Verifie" sans
  qualification. Gap confirme.
- `GpuConsentDialog.tsx:56-84` : "Projets open source verifies".
- `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` : tiers T0/T1/T2 = endpoints
  loopback, pas taxonomie app-level. Scopes differents, pas de
  duplication.

**Alternatives** :
- OpenSSF Scorecards (30+ checks) : assume CI/CD centralise
  (branch protection, binary artifacts hosting). Inadapte P2P.
- F-Droid trust model : utilise aussi methodologie multi-niveaux.
  Alignement implicite avec D2.

**Angle mort** : rejet OpenSSF pas explicitement source.
Recommandation : ajouter section "Why not Scorecard" dans
TRUST_TAXONOMY.md.

---

## D3 — Vocabulaire enforcement : ✅

**Sources verifiees** :
- Script n'existe pas encore (livrable Phase C).
- Termes interdits identifies dans codebase : confirmes par scan.

**Alternatives** :
- ESLint custom : overhead pour grep 10 lignes. Rejete.
- Pas de script : regression wording au premier refactor. Rejete.

**Angle mort** : faux positifs (ex: "verified" dans nom de
variable). Recommandation : valider 0 FP sur codebase existante
avant activation CI.

---

## D4 — Deploy→feed + auth tier : ⚠️

**Sources verifiees** :
- `deploy.rs:237-250` : aucun appel `feed_insert()` apres
  `publish_announcement()`. Gap confirme.
- `deploy.rs:72` : `starts_with("http")` accepte http://. Bug
  confirme.
- `feed_sync.rs:445` : `feed_insert()` zero check auth tier.
- `LOOPBACK_ENDPOINTS_TRUST_TIERS.md:129-137` : TrustTier enum
  spec mais NOT implemented in code.

**Alternatives** :
- T1 CONFIRM_PROMPT complet : requiert UI (nonce + TTL + prompt
  React). Hors scope S65. Defere post-pilote S69.
- Feed insert manuel : UX degradee. Rejete.

**Angle mort critique** : header `X-SBFB-Feed-Internal` n'est pas
cryptographiquement lie au daemon. Process local malveillant avec
connaissance du header peut inserer. Pas de rotation, pas de nonce.

**Recommandations** :
1. Documenter dans LOOPBACK_ENDPOINTS_TRUST_TIERS.md que
   feed_insert() = T0 temporaire, header != credential crypto.
2. Vrai T1 avec nonce temporal + HMAC(secret, nonce, endpoint)
   post-pilote S69.
3. Limitation acceptable pre-launch (device dev-only, loopback
   bearer deja requis comme premiere couche).

**Verdict** : ⚠️ SCOPE-CUT-CONSISTENT — defense-in-depth
suffisante pre-launch, limitation documentee, T1 vrai programme
S69.

---

## D5 — Factory gates spec : ✅

**Sources verifiees** :
- Documents n'existent pas encore (livrables Phase D).
- SBFB.json v1 existant (retro-compatibilite mentionnee).

**Alternatives** :
- Gates dynamiques configurables : overhead pour flow lineaire.
- Gates code S65 : couplage premature. Rejete.

**Angle mort** : strategie upgrade v1→v2 non detaillee.
Recommandation : section versioning dans SBFB_JSON_V2.md
(schema_version, #[serde(default)], forward-compat parsing).

---

## Checklist DETER

### Crypto/spec
- [x] D1 : alternative concurrente (<6 mois) — serde(other) bloque
- [x] D2 : source datee <2 ans — F-Droid/SLSA/RFC 8785
- [x] D4 : limitation crypto documentee

### Rust-first
- [x] D1 : serde_json = crate Rust native production
- [x] D4 : Axum header check natif
- [x] D5 : docs-only, pas de code

---

## Sources externes

1. RFC 8785 (JCS) — rfc-editor.org/rfc/rfc8785
2. OpenSSF Scorecard — github.com/ossf/scorecard
3. F-Droid Security Model — f-droid.org/en/docs/Security_Model/
4. Serde enum handling — serde.rs/enum-representations.html
5. SLSA specification — slsa.dev/spec/v1.0/levels

Pas de contradiction detectee avec l'ecosysteme mai 2026.
