# SSH Client + WebSSH Gateway — Threat Model (STRIDE/DREAD)

Version: v0.4  
Date: 2025-12-16  
Status: Production-ready draft

## Table of Contents

- [1. Scope](#1-scope)
- [2. Assets and trust boundaries](#2-assets-and-trust-boundaries)
- [3. Attackers and entry points](#3-attackers-and-entry-points)
- [4. STRIDE analysis](#4-stride-analysis)
- [5. DREAD scoring](#5-dread-scoring)
- [6. Security requirements mapping](#6-security-requirements-mapping)
- [7. Architectural rationale](#7-architectural-rationale)
- [8. Residual risk and follow-ups](#8-residual-risk-and-follow-ups)

---

## 1. Scope

Components:

- Apps: Desktop / Mobile / Web
- Core: `ssh-core`, `vault`, `sync-client`
- Server: `sync-api`, `webssh-gateway`

Out of scope:

- Compromise of SSH server itself
- Full OS compromise mitigation (only risk reduction)

---

## 2. Assets and trust boundaries

### 2.1 Critical assets

| Asset | Description |
|---|---|
| A1 | Private keys / passphrases / passwords |
| A2 | Decrypted vault contents (hosts/snippets/settings) |
| A3 | JWT access/refresh tokens |
| A4 | Known hosts + pinned fingerprints |
| A5 | Terminal I/O (stdin/stdout/stderr) |
| A6 | Device identity (deviceId), server cursor state |

### 2.2 Trust boundaries

- TB1: App ↔ Sync API over HTTPS
- TB2: Web App ↔ WebSSH Gateway over WSS
- TB3: Gateway ↔ SSH Server over TCP
- TB4: Secure Storage boundary (OS keychain/keystore)
- TB5: Browser JS environment (XSS boundary)

---

## 3. Attackers and entry points

### 3.1 Attacker profiles

- ATK-1: Network attacker (MITM, replay)
- ATK-2: Web attacker (XSS, supply chain in JS deps)
- ATK-3: Server attacker (DB exfiltration, sync-api compromise)
- ATK-4: Local malware on client device
- ATK-5: Abusive client (quota bypass, protocol fuzzing)

### 3.2 Entry points

- EP1: Sync API endpoints
- EP2: Gateway WSS protocol messages
- EP3: Web UI inputs rendered in DOM
- EP4: Dependency supply chain (crates/npm/flutter)
- EP5: Logging/telemetry/crash pipelines

---

## 4. STRIDE analysis

### 4.1 Threat catalog (normative IDs)

Legend: STRIDE category — **S**poofing, **T**ampering, **R**epudiation, **I**nformation disclosure, **D**oS, **E**levation.

| Threat ID | STRIDE | Description | Primary assets | Primary mitigations (refs) |
|---|---|---|---|---|
| T-001 | S | Token theft/reuse to impersonate user/device | A3, A6 | JWT exp+jti, device binding (SEC-008), rate limits (NFR-007) |
| T-002 | T | Host key substitution / MITM | A4, A5 | Host key policy strict/ask/accept-new (SEC-005), prompts |
| T-003 | I | Secrets leaked via logs/telemetry/crashes | A1, A2, A5 | Redaction + CI gates (SEC-006), no terminal logging (SEC-004) |
| T-004 | I | Sync DB compromise reveals user data | A2 | E2EE invariant (SEC-001), minimal metadata |
| T-005 | I | Web XSS steals decrypted data or tokens | A2, A3 | CSP, token strategy, sanitize, minimize key lifetime (SEC-003/6.4) |
| T-006 | D | Gateway resource exhaustion / quota bypass | A6 | Session quotas + flow control + message size limits (FR-162, SRS 9.9) |
| T-007 | T | Replay of WSS messages (connect/stdin) | A5, A6 | requestId idempotency + session state machine (SEC-007) |
| T-008 | E | Privilege escalation across users in gateway | A1-A6 | Strict auth binding per WSS, per-session userId checks, isolation |
| T-009 | R | Lack of auditability for sensitive actions | A6 | Audit events (SRS 11.2), immutable server logs |
| T-010 | T/I | Supply chain compromise | A1-A6 | lockfiles, audits, SBOM policy (Implementation Plan) |

---

## 5. DREAD scoring

Scale: 0–10 per dimension; Risk = average.

| Threat ID | Damage | Repro | Exploit | Affected | Discover | Risk |
|---|---:|---:|---:|---:|---:|---:|
| T-002 | 9 | 7 | 7 | 8 | 6 | 7.4 |
| T-003 | 10 | 8 | 8 | 9 | 7 | 8.4 |
| T-005 | 9 | 6 | 7 | 8 | 8 | 7.6 |
| T-004 | 8 | 8 | 6 | 8 | 7 | 7.4 |
| T-006 | 6 | 8 | 7 | 7 | 7 | 7.0 |
| T-007 | 7 | 7 | 6 | 6 | 6 | 6.4 |

Highest priority risks: **T-003, T-005, T-002**.

---

## 6. Security requirements mapping

### 6.1 Threat → Control mapping

| Threat | Controls |
|---|---|
| T-002 | SEC-005; FR-060/061/062; gateway hostkey_prompt flow |
| T-003 | SEC-006; SRS 11; Implementation CI secret-leak gates |
| T-005 | SRS 6.4; CSP; token strategy; Web MVP key policy (SEC-003) |
| T-004 | SEC-001; FR-140/141; server schema stores ciphertext only |
| T-006 | FR-162; flow_control; maxMessageSizeBytes; rate limits |
| T-007 | SRS 9.3/9.4; SEC-007 |
| T-010 | CI audits; pinned deps; optional SBOM |

### 6.2 Security test requirements (minimum)

- Host key changed behavior blocks/prompted as per policy.
- Gateway refuses stdin before READY.
- Secret scanning of logs/telemetry/crash artifacts in CI.
- Basic web security tests: CSP presence, no inline scripts, DOM sanitization smoke tests.
- Token replay tests: short-lived access token; device binding enforcement.

---

## 7. Architectural rationale

### 7.1 Why E2EE sync

- Primary risk: server compromise (T-004).
- Design choice: server stores encrypted blobs; decryption only on client.
- Tradeoff: server cannot do plaintext search; clients maintain indexes.

### 7.2 Why Web requires gateway

- Browsers cannot open raw TCP to SSH.
- Gateway provides controlled bridge with strong limits and state validation.
- Tradeoff: gateway becomes a high-value component; must be hardened and quota-protected.

### 7.3 Why “no persistent keys in web MVP”

- Web threat model includes XSS and weaker local isolation.
- Temporary key upload into gateway RAM limits exposure window (SEC-003).
- Tradeoff: some UX friction; mitigated by clear warnings and optional password auth.

---

## 8. Residual risk and follow-ups

- Local malware (ATK-4) remains partially unmitigated; consider:
  - device revocation and “wipe device” in v1,
  - optional screenshot prevention on mobile,
  - optional passkeys for web unlock.
- Web token handling decision is pending (cookie vs header); must be resolved before production.
