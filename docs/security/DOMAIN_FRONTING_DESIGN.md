# Domain Fronting — Design Outline

**Status** : design-only (Sprint 24 Phase E). Implementation
deferred S25+ pending legal review.

**Last updated** : 2026-04-21

---

## 1. Problem statement

Censorship-capable adversaries (nation-state ISPs, corporate
firewalls) can block SBFB traffic by:

1. **IP blocking** — blocking the IP addresses of known pkarr
   relays and iroh relay servers.
2. **DPI (Deep Packet Inspection)** — fingerprinting the iroh
   QUIC protocol and dropping matching flows.
3. **DNS poisoning** — returning NXDOMAIN for SBFB-related
   domains.

Sprint 24 Phase E addresses item 3 via encrypted DNS (DoH/DoT
fallback). Items 1 and 2 require transport-layer obfuscation,
of which domain fronting is one approach.

---

## 2. What is domain fronting

Domain fronting exploits the gap between the SNI (Server Name
Indication) in the TLS ClientHello and the HTTP Host header
inside the encrypted tunnel. The client connects to a CDN edge
IP (e.g. `cdn.example.com` in SNI) but sends the actual request
to a different origin (`hidden.sbfb.net` in the Host header).
The CDN routes the request to the hidden origin, and the censor
sees only traffic to the CDN IP + SNI.

---

## 3. Current landscape (April 2026)

Major CDN providers have **closed** the domain fronting vector:

| Provider | Status | Date closed |
|---|---|---|
| Google Cloud / GFE | Blocked | April 2018 |
| Amazon CloudFront | Blocked | April 2018 |
| Microsoft Azure CDN | Blocked | 2019 |
| Cloudflare | Blocked | 2018 |
| Fastly | Blocked | 2020 |

**Remaining options** :

- **Encrypted Client Hello (ECH)** — TLS 1.3 extension (RFC 9579,
  finalized 2024) that encrypts the SNI. Supported by Cloudflare
  (opt-in) and Firefox. Does not require CDN cooperation for
  routing — it simply hides the target domain from the censor.
  Adoption is growing but not universal.

- **Snowflake WebRTC** — Tor pluggable transport using WebRTC
  data channels through a volunteer proxy network. Traffic
  resembles a video call. Used by Tor Browser since 2021.

- **Minor / regional CDNs** — some smaller CDNs (Akamai partial,
  regional providers) may still allow fronting, but relying on
  them is fragile and legally uncertain.

---

## 4. Recommended approach for SBFB

### 4.1 Short-term (S25)

- **ECH support** in the iroh relay client (requires iroh
  upstream support or a custom `rustls::ClientConfig` with ECH
  keys). Hides the relay SNI from DPI.
- **Tor integration** via Arti (Tor client library in Rust).
  Sprint 26 roadmap item. Provides full anonymity + censorship
  resistance through the Tor network.

### 4.2 Medium-term (S26+)

- **Snowflake bridge** as a pluggable transport option for the
  iroh relay connection. Requires volunteer proxies.
- **Obfs4 / Lyrebird** obfuscation layer — makes traffic look
  random. Already researched Sprint 25 (HARDENING_ROADMAP §3).

### 4.3 Explicitly NOT pursued

- **Classic domain fronting** on major CDNs — blocked since 2018,
  legally risky (violates CDN ToS), unreliable.
- **Custom CDN deployment** — requires partnership and funding,
  incompatible with the solo-maintainer model (cf. vision_model).

---

## 5. Legal considerations

Domain fronting (where still possible) may violate:

- CDN Terms of Service (traffic routing fraud)
- Local anti-circumvention laws (varies by jurisdiction)
- ISP acceptable use policies

**SBFB policy** : the project will only implement censorship
resistance techniques that do not require ToS violations. ECH
and Tor are legitimate, standards-based protocols.

---

## 6. Prerequisites for implementation

1. Legal review of ECH + Tor in target jurisdictions
2. iroh upstream ECH support (or custom rustls config path)
3. Arti library API stability (Sprint 26 research item)
4. Volunteer proxy infrastructure for Snowflake (community)

---

## 7. References

- RFC 9579 — TLS Encrypted Client Hello
- Tor Project Snowflake — https://snowflake.torproject.org/
- Arti — https://gitlab.torproject.org/tpo/core/arti
- Fifield et al. 2015 — "Blocking-resistant communication
  through domain fronting" (PETS 2015)
- HARDENING_ROADMAP.md §3 Sprint 25-26 transport hardening
