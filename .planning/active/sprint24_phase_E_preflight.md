# Sprint 24 Phase E — preflight G8

Date : 2026-04-21 | HEAD : `1fcf7ec` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : "context7 obligatoire pour nouvelle dep" + "OSS prior art avant code" — applique S1a+S1b
- feedback_context7_systematic.md : hickory-resolver = nouvelle dep Rust, context7 query effectuee

## Scans (all clean)
- S1a OSS prior art : 5 projets recherches (libp2p/IPFS dnsaddr TXT bootstrap, pubky/pkdns pkarr DNS server, pubky/pkarr mainline DHT, KadNode P2P DNS, AdguardTeam/dnsproxy DoH+DoT fallback), APPROACH-ALIGNED — DNS comme fallback transport P2P = pattern standard (IPFS `/dnsaddr` bootstrap via TXT records). pkarr nativement DNS-compatible (packets DNS <= 1000 bytes). pkdns = serveur-side existant, notre code = client-side DoH/DoT resolver. Pas de lib cliente prete qui combine pkarr+DoH+DoT (pkdns est un serveur, pas un resolver client).
- S1b deps : hickory-resolver scanne via context7 (`/websites/rs_hickory-resolver`), API DoH confirmee (`NameServerConfigGroup::from_ips_https`), API DoT confirmee (`from_ips_tls`), `TxtLookup` type pour TXT records. 0 CVE rustsec 2026. RUSTSEC-2026-0098 rustls-webpki (CVSS 2.2 minimal, URI name constraints) = non-bloquant.
- S2 historiques : 5 fichiers cibles scannes (`lib.rs`, `Cargo.toml`, `browse_aggregator.rs`), 0 commit DEVIATION/rejected sur zone DNS/fallback/transport. Archives v1.0-v1.2 : 0 mention DNS rejected. Memory feedback : 0 contrainte DNS/transport.
- S3 threat model : fast-path verified, HARDENING_ROADMAP S24 mentionne "DNS fallback" + "domain fronting legal" — Phase E alignee.
- S4 wire format : fast-path verified, `_VERSION = 1` (schemas/mod.rs), Phase E = transport additionnel, 0 impact wire format, Day 0 D4 DNS fallback preserved.

## Action
Proceder code Phase E.
