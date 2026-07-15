Verdict global : 4 livrables confirmés, 2 partiels. Les écarts concernent les assertions TLS des tests et les identifiants RUSTSEC encore présents dans un commentaire de `deny.toml`.

Contrôle réalisé sur `master`, exclusivement depuis `HEAD` et le working tree. Aucun build/test écrivant dans `target/` n’a été lancé.

### Livrable 1 : pin workspace hickory-resolver 0.26

- Statut : CONFIRME
- Fichier(s) : [Cargo.toml:443](C:/Users/FlowUP/Documents/Code/nexus/Cargo.toml:443)–459 ; [Cargo.lock:7173](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:7173), [Cargo.lock:10458](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:10458)
- Evidence :

```toml
455:hickory-resolver = { version = "0.26", features = [
456:    "tls-ring",
457:    "https-ring",
458:    "webpki-roots",
459:] }
```

```text
448:# deny.toml ignores. Features: `tls-ring` (DoT) + `https-ring`
449:# (DoH) keep rustls on the `ring` backend already pulled by the
450:# iroh stack — no aws-lc-rs, no new TLS backend in the binary
451:# (re-validated against Cargo.lock at the S82 K bump).
```

Le commentaire ne contient pas de promesse future : il décrit le bump et la validation effectuée. `cargo tree --locked --offline -e features` confirme les trois features et `rustls feature "ring"`. `ring 0.17.14` et `webpki-roots 1.0.7` sont présents ; `aws-lc-rs` et `aws-lc-sys` sont absents.

Ce constat porte sur le provider rustls/hickory, comme l’écrit le commentaire. Le lock contient par ailleurs des piles `native-tls/openssl` préexistantes pour d’autres dépendances ; aucune n’est ajoutée par cette phase.

### Livrable 2 : migration du resolver vers l’API 0.26

- Statut : CONFIRME
- Fichier(s) : [dns_fallback.rs:163](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/dns_fallback.rs:163)–176, [dns_fallback.rs:201](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/dns_fallback.rs:201)–238, [dns_fallback.rs:252](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/dns_fallback.rs:252)–272
- Evidence :

```rust
219:                DnsTransport::Doh => ConnectionConfig::https(server_name, None),
220:                DnsTransport::Dot => ConnectionConfig::tls(server_name),
221:            };
222:            conn.port = ep.port;
223:            // trust_negative_responses=false must stay EXPLICIT
```

```rust
264:        for record in lookup.answers() {
265:            if let RData::TXT(txt) = &record.data {
266:                for segment in txt.txt_data.iter() {
267:                    data.extend_from_slice(segment);
268:                }
```

Confirmations :

- Alias `Resolver<TokioRuntimeProvider>` aux lignes 163–165.
- Enum fermé `DnsTransport::{Doh, Dot}` aux lignes 167–176.
- Garde endpoints vides aux lignes 206–210.
- TLS name construit depuis `ep.tls_name` pour chaque itération, lignes 213–220.
- Port configurable conservé ligne 222.
- `NameServerConfig::new(ep.ip, false, vec![conn])` explicite ligne 226.
- `options_mut()`, timeout et `attempts = 2` lignes 229–234.
- `build()` faillible mappé vers `NexusError::Endpoint`, lignes 236–238.
- L’API a été recoupée contre les sources installées de hickory 0.26.1 : signatures et champs utilisés sont réels et publics.

Le `git diff --unified=0` confirme l’absence d’édition des invariants demandés : threat model [lignes 16–31](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/dns_fallback.rs:16), constantes [46–71](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/dns_fallback.rs:46), trait public [82–92](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/dns_fallback.rs:82), `build_query_name` [241–249](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/dns_fallback.rs:241), course `tokio::select!` [289–336](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/dns_fallback.rs:289), chargement env et concaténation [346–384](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/dns_fallback.rs:346).

### Livrable 3 : adaptation honnête des tests

- Statut : PARTIEL
- Fichier(s) : [dns_fallback.rs:529](C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/dns_fallback.rs:529)–583
- Evidence :

```rust
573:    // `build_resolver_rejects_unsupported_protocol` was removed with
574:    // the hickory 0.26 bump (S82 Phase K): the local `DnsTransport`
575:    // enum only has Doh/Dot variants, so an unsupported protocol is
576:    // unrepresentable and the runtime guard it exercised is gone.
```

```rust
579:    fn build_resolver_rejects_empty_endpoints() {
580:        let cfg = DnsFallbackConfig::default();
581:        let err = DnsFallbackResolver::build_resolver(&[], DnsTransport::Doh, &cfg).unwrap_err();
582:        let msg = format!("{err}");
583:        assert!(msg.contains("no DNS endpoints"), "got: {msg}");
```

La comparaison structurée donne :

- `HEAD` : 14 tests.
- Working tree : 13 tests.
- Retiré uniquement : `build_resolver_rejects_unsupported_protocol`.
- Ajout : aucun.
- Delta source exact : `-1`. Les autres tests ne présentent aucune modification, hormis l’adaptation `Protocol::Https` → `DnsTransport::Doh`.

GAP interne : les tests `per_endpoint_tls_name_used_doh` et `_dot` ne vérifient pas leur invariant annoncé. Ils construisent un resolver avec des noms personnalisés, puis testent uniquement :

```rust
546:        let resolver =
547:            DnsFallbackResolver::new(&cfg).expect("build resolver with per-endpoint TLS");
548:        assert_eq!(resolver.label(), "dns-fallback-doh-dot");
```

Cette assertion resterait verte même si les TLS names étaient ignorés ou globalisés. Conformément à la contrainte donnée, ces tests ne constituent pas une assertion utile de P2-E-1.

Les totaux 2099 Windows / 2103 Docker sont cohérents arithmétiquement avec le `-1`, mais n’ont pas été exécutés pendant cet audit read-only.

### Livrable 4 : retrait des ignores dans deny.toml

- Statut : PARTIEL
- Fichier(s) : [deny.toml:41](C:/Users/FlowUP/Documents/Code/nexus/deny.toml:41)–86, [deny.toml:101](C:/Users/FlowUP/Documents/Code/nexus/deny.toml:101), [deny.toml:127](C:/Users/FlowUP/Documents/Code/nexus/deny.toml:127)–137, [deny.toml:167](C:/Users/FlowUP/Documents/Code/nexus/deny.toml:167)
- Evidence :

```toml
62:# Removed S82 Phase K: the four hickory-root ignores
63:# (RUSTSEC-2026-0119 hickory-proto O(n^2) name-compression, DoS
64:# class; RUSTSEC-2026-0098/0099 rustls-webpki name-constraint
65:# laxity, authentication class; RUSTSEC-2026-0104 rustls-webpki
66:# CRL-parsing panic, DoS/availability class) — all four closed by
```

```toml
83:    # in HARDENING_ROADMAP front-matter).
84:    { id = "RUSTSEC-2026-0194", reason = "quick-xml 0.39.4 quadratic duplicate-attribute check; ..." },
85:    { id = "RUSTSEC-2026-0195", reason = "quick-xml 0.39.4 unbounded NsReader namespace allocation; ..." },
```

Le TOML parsé confirme que le tableau `advisories.ignore` ne contient plus que les deux ignores quick-xml. `yanked = "deny"`, le commentaire rand, `[licenses]`, `[bans]`, `multiple-versions = "warn"` et `[sources]` sont intacts.

GAP : l’exigence « plus aucune occurrence » n’est pas satisfaite. Les quatre advisories sont encore mentionnées lignes 63–65, dont `0099` sous la forme abrégée `0098/0099`. Les entrées d’ignore sont bien retirées et la note est factuelle, mais l’absence textuelle demandée est fausse.

### Livrable 5 : résolution et assainissement de Cargo.lock

- Statut : CONFIRME
- Fichier(s) : [Cargo.lock:3277](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:3277)–3299, [Cargo.lock:7305](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:7305)–7405, [Cargo.lock:8135](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:8135)–8146, [Cargo.lock:10458](C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:10458)
- Evidence :

```toml
3276:[[package]]
3277:name = "hickory-proto"
3278:version = "0.26.1"
3279:source = "registry+https://github.com/rust-lang/crates.io-index"
```

```toml
3296:[[package]]
3297:name = "hickory-resolver"
3298:version = "0.26.1"
3299:source = "registry+https://github.com/rust-lang/crates.io-index"
```

```toml
7402:[[package]]
7403:name = "rustls-webpki"
7404:version = "0.103.13"
7405:source = "registry+https://github.com/rust-lang/crates.io-index"
```

Comparaison TOML structurée `HEAD` → working tree :

- Enregistrements : `1002 → 990`, soit `-12`.
- Identités ajoutées : uniquement `spin 0.9.9` et `spin 0.10.1`.
- Identités retirées : `spin 0.9.8`, `spin 0.10.0` et les 12 éléments de l’arbre legacy : `hickory-{resolver,proto} 0.24.4`, `h2 0.3.27`, `http 0.2.12`, `rustls 0.21.12`, `rustls-webpki 0.101.7`, `tokio-rustls 0.24.1`, `enum-as-inner`, `linked-hash-map`, `lru-cache`, `rustls-pemfile 1.0.4`, `sct 0.7.1`.
- Nouveaux noms de crate : aucun.
- Versions restantes : `rustls 0.23.40`, `tokio-rustls 0.26.4`, `h2 0.4.14`, `http 1.4.0`.
- `ring 0.17.14` et `webpki-roots 1.0.7` présents ; aucun `aws-lc-rs`/`aws-lc-sys`.

Les seuils annoncés sont exacts : hickory-proto est corrigé à `>= 0.26.1` selon [RUSTSEC-2026-0119](https://rustsec.org/advisories/RUSTSEC-2026-0119.html) ; rustls-webpki à `>= 0.103.12` pour [0098](https://rustsec.org/advisories/RUSTSEC-2026-0098.html) et [0099](https://rustsec.org/advisories/RUSTSEC-2026-0099.html), et à `>= 0.103.13` pour [0104](https://rustsec.org/advisories/RUSTSEC-2026-0104.html). Le lock à `0.103.13` ferme donc les trois.

L’[index officiel crates.io de spin](https://github.com/rust-lang/crates.io-index/blob/master/sp/in/spin#L378-L381) marque bien `0.9.8` et `0.10.0` comme yanked ; `0.9.9` et `0.10.1` ne le sont pas.

### Livrable 6 : documentation sécurité

- Statut : CONFIRME
- Fichier(s) : [HARDENING_ROADMAP.md:3](C:/Users/FlowUP/Documents/Code/nexus/docs/security/HARDENING_ROADMAP.md:3), [HARDENING_ROADMAP.md:17](C:/Users/FlowUP/Documents/Code/nexus/docs/security/HARDENING_ROADMAP.md:17), [HARDENING_ROADMAP.md:29](C:/Users/FlowUP/Documents/Code/nexus/docs/security/HARDENING_ROADMAP.md:29)–30 ; [THREAT_MODEL.md:1827](C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1827)–1878
- Evidence :

```yaml
16:  - "microsoft/sudo elevation mode release beyond Windows 11 24H2 inbox (...)"
17:  - "hickory-resolver breaking release > 0.26 OU nouvelle RUSTSEC sur la chaine rustls 0.23 / rustls-webpki / hickory-proto (...)"
18:audited_findings:
```

```markdown
1853:- **v18 (Sprint 82 Phase K, 2026-07-15)** : supply-chain DNS fallback —
1854:  bump hickory-resolver 0.24 → 0.26 (construction resolver reecrite
1855:  `dns_fallback.rs` : Resolver/builder + `NameServerConfig::new` +
1856:  `ConnectionConfig` ; per-endpoint TLS name P2-E-1 PRESERVE ;
1857:  `trust_negative_responses=false` rendu EXPLICITE
```

Confirmations :

- `last_validated` est daté `2026-07-15` ligne 3.
- Le contenu daté S81-G précédent est reproduit exactement en queue de cette ligne : comparaison exacte du corps de 889 caractères.
- L’entrée `audited_findings` S81-G, désormais ligne 29, est byte-identique à `HEAD`.
- Nouvelle entrée S82-K présente ligne 30.
- Le trigger standing `> 0.26` est présent ligne 17.
- Le diff de `THREAT_MODEL.md` est exclusivement additif : ajout de v18 aux lignes 1853–1878. Les blocs v15, v16 et v17 n’ont aucune modification.

## Anomalie de périmètre hors décompte

Le working tree contient aussi `.planning/active/sprint82_phase_k_review.md` en untracked. Ce fichier n’est ni dans la liste exacte de phase fournie, ni dans les trois exclusions de recherche. Il a été laissé hors audit et n’a pas été modifié.

## Résumé final

- Total livrables : 6
- Confirmés : 4
- Gaps : 0
- Partiels : 2

Causes des deux statuts PARTIEL :

- Les tests P2-E-1 n’assertent pas réellement les TLS names par endpoint.
- `deny.toml` conserve encore les identifiants des quatre advisories dans son commentaire de retrait.