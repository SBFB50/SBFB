# Security Policy

## Reporting a vulnerability

If you discover a security vulnerability in nexus-grid:

1. **DO NOT** open a public issue
2. Use [GitHub Security Advisories](https://github.com/SBFB50/SBFB/security/advisories/new)
   to report privately
3. Describe the vulnerability, reproduction steps, and potential impact
4. We will respond within 48 hours

## Scope

nexus-grid is a P2P network. Security-relevant components include:

- **Ed25519 signatures** — task, result, claim, curator list, kudos
  signing and verification (`crates/nexus-core-rs/src/crypto.rs`)
- **iroh QUIC transport** — peer-to-peer communication
- **Coordinator loopback** — FastAPI runs on `127.0.0.1:8765` by
  default, CORS restricted to loopback origins only
- **Shell daemon** — loopback HTTP surface, accessed exclusively
  through the coordinator proxy (Pattern P9)
- **CAS file storage** — SHA256 content-addressed, magic bytes
  validation on upload, max_size_bytes enforcement
- **Migration runner** — SHA256 tamper detection on applied migrations

## Trust model

nexus-grid operates on a **zero-trust P2P model**:

- No central server, no admin, no moderation
- Workers choose which projects they serve (allowlist)
- Curator lists are Ed25519-signed and propagated via gossip
- The coordinator is loopback-only by default — exposing it to the
  network is the operator's responsibility

## License

AGPL-3.0-or-later — see [LICENSE](LICENSE)
