# Security Policy

## Reporting a vulnerability

If you discover a security vulnerability in nexus-grid:

1. **DO NOT** open a public issue
2. Use [GitHub Security Advisories](https://github.com/SBFB50/SBFB/security/advisories/new)
   to report privately
3. Email `security@sbfb.network` (PGP key below) as a fallback
4. Describe the vulnerability, reproduction steps, and potential impact
5. We will acknowledge within **48 hours** and provide an initial
   assessment within **7 calendar days**

## PGP key

```
Fingerprint: (to be published at first external audit engagement)
```

## Response timeline

| Stage | SLA |
|---|---|
| Acknowledgement | 48 hours |
| Initial assessment (severity + affected components) | 7 days |
| Fix development (critical/high) | 14 days |
| Fix development (medium/low) | 30 days |
| Public disclosure | 90 days after report, or at fix release |

We follow coordinated disclosure. If you need an extension before
public disclosure, let us know in the initial report.

## Scope

nexus-grid is a P2P network. Security-relevant components include:

- **Ed25519 signatures** — task, result, claim, curator list, kudos
  signing and verification (`crates/nexus-core-rs/src/crypto.rs`)
- **iroh QUIC transport** — peer-to-peer communication
  (`crates/nexus-core-rs/src/`, iroh 0.97 pinned)
- **Coordinator loopback** — FastAPI on `127.0.0.1:8765`, triple
  validation bearer + Host + Origin (Sprint 16)
- **Shell daemon** — loopback HTTP surface, accessed exclusively
  through the coordinator proxy (Pattern P9), peer creds
  UDS/Named Pipe (Sprint 16)
- **Blob-serve** — iframe content server, CSP `connect-src 'none'`,
  sandbox `allow-scripts` without `allow-same-origin`
- **Worker** — GPU compute engine, consent 4 levels, caps enforced
- **Deploy-from-repo** — Keyoxide Ed25519 + SLSA L1 provenance
  (Sprint 14)
- **postMessage bridge** — 3-method whitelist iframe communication
- **CAS file storage** — SHA256 content-addressed, magic bytes
  validation on upload, max_size_bytes enforcement
- **Migration runner** — SHA256 tamper detection on applied migrations

### Out of scope

- OS kernel, drivers, CUDA runtime vulnerabilities
- Ollama runtime vulnerabilities (report to ollama/ollama)
- iroh protocol vulnerabilities (report to n0-computer/iroh)
- Social engineering attacks on curator list operators

## Trust model

nexus-grid operates on a **zero-trust P2P model**:

- No central server, no admin, no moderation
- Workers choose which projects they serve (consent levels L1-L4)
- Curator lists are Ed25519-signed and propagated via gossip
- The coordinator is loopback-only by default — exposing it to the
  network is the operator's responsibility
- Threat model: `docs/security/THREAT_MODEL.md`

## Severity classification

We use CVSS v3.1 for severity scoring:

| CVSS | Severity | Example |
|---|---|---|
| 9.0-10.0 | Critical | Remote keypair exfiltration, unsigned code execution |
| 7.0-8.9 | High | Loopback auth bypass, consent override |
| 4.0-6.9 | Medium | Information disclosure, rate-limit bypass |
| 0.1-3.9 | Low | UI-only, requires physical access |

## Bug bounty

No formal bug bounty program at this time. We will credit
reporters in the advisory and CHANGELOG unless they prefer
anonymity.

## License

AGPL-3.0-or-later — see [LICENSE](LICENSE)
