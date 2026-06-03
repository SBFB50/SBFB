### Livrable 1 : `THREAT_MODEL.md` section Operator

- Statut : CONFIRME
- Fichier(s) : [docs/security/THREAT_MODEL.md](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:701)
- Evidence :

```text
701:## 14. Operator surface (Sprint 72 Phase A)
722:### T-OPERATOR-CSRF — CSRF / DNS-rebinding sur surface write + spawn
745:### T-OPERATOR-SPAWN — Spawn agent autonome non gate
768:### Anticipation NetworkProvider (Sprint 72 ProviderRouter)
779:### Residual risks Operator
```

```text
729:Mitigation (S71 G7, `a0337c6`) : middleware `auth_required`
731:(1) `X-SBFB-Token` bearer per-boot compare en `constant_time_eq`
733:`Origin:` doit etre loopback ou absent
734:epingle a `is_loopback_origin` (`operator_server.rs:103`, plus de
```

Renumerotation et historique confirmes :

```text
795:## 15. Revue et evolution
825:- **v7 (Sprint 72 Phase A, 2026-05-31)** : ajout §14 Operator surface
826:  (T-OPERATOR-CSRF, T-OPERATOR-SPAWN + anticipation NetworkProvider),
827:  renommage §14→§15.
```

Cohérence : pas de section `## 14.` dupliquee. Le residual token local cite bien AD2 via le modele adversaire, et AD2 est bien defini comme malware user-mode abusant `auth_token` en [docs/security/THREAT_MODEL.md](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:74).

### Livrable 2 : `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` section Operator

- Statut : PARTIEL
- Fichier(s) : [docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md](C:/Users/FlowUP/Documents/Code/nexus/docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:3)
- Evidence :

```text
3:last_validated: 2026-05-31  # S72 Phase A : ajout §3.1 inventaire Operator (P2-H-1)
66:## 3.1 Serveur Operator (sbfb-factory, port `:3001`) — surface write + spawn
81:| `POST /api/artifacts/draft` (**write**) | off-sprint, durci S71 C | T0 | T0 |
82:| `GET /api/chat/{id}/stream` (**spawn**) | off-sprint, durci S71 C | T0 + gate `SENSITIVE_ACTIONS` |
```

```text
89:Gate **G7** (S71 Phase C `a0337c6`) : middleware `auth_required`
90:(`auth.rs:229`) sur chaque route data-bearing — `X-SBFB-Token`
91:(`constant_time_eq`) + `Host:` loopback + `Origin:` loopback/absent +
93:plus de `allow_origin(Any)`). Gate **G2** : `SENSITIVE_ACTIONS`
```

- GAP : reference croisee mineure incorrecte ligne 101. Le texte dit `cf. §8 AD2 « Malware user-mode »`, mais AD2 est defini dans `THREAT_MODEL.md §3`, pas en §8. Le label AD2 est correct, la section citee ne l’est pas.

La reference P35 est correcte : [docs/shell/PATTERNS.md](C:/Users/FlowUP/Documents/Code/nexus/docs/shell/PATTERNS.md:2158) contient bien `### P35 — Sprint 71 Phase C : Factory Operator server loopback hardening`, tandis que `docs/rust/PATTERNS.md §P35` est un autre sujet.

### Livrable 3 : verite terrain code

- Statut : CONFIRME
- Fichier(s) : [crates/sbfb-factory/src/auth.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-factory/src/auth.rs:229), [crates/sbfb-factory/src/operator_server.rs](C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-factory/src/operator_server.rs:34)
- Evidence :

```text
229:pub async fn auth_required(...)
232:    let host_ok = headers
235:        .map(is_loopback_host)
241:    if let Some(origin) = headers.get(header::ORIGIN) {
245:            .map(is_loopback_origin)
255:        .map(|t| constant_time_eq(t.as_bytes(), auth.token.as_bytes()))
```

```text
34:const SENSITIVE_ACTIONS: &[&str] = &["shell", "commit", "push", "PASS"];
99:    let cors = CorsLayer::new()
100:        .allow_origin(AllowOrigin::predicate(|origin, _parts| {
103:                .map(auth::is_loopback_origin)
```

```text
822:async fn handle_chat_stream(
866:    let is_sensitive = SENSITIVE_ACTIONS
869:    if is_sensitive {
876:        return sse_gate(
898:    let claude_stream = llm_bridge::spawn_claude_stream(&prompt, &model, &root);
```

Conclusion terrain : pas de defense fantome detectee. Le gate `SENSITIVE_ACTIONS` est bien avant `spawn_claude_stream`, le CORS n’utilise pas `allow_origin(Any)`, et `auth_required` impose Host loopback, Origin loopback/absent, et token `X-SBFB-Token` compare en constant-time. `git show a0337c6` confirme aussi que ces symboles existaient deja dans le commit S71 Phase C.

## Resume final

- Total livrables : 3
- Confirmes : 2
- Gaps : 0
- Partiels : 1

Aucun test runtime lance ; audit effectue par lecture source/doc et verification `git show a0337c6`.