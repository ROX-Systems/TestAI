# SSH Client + WebSSH Gateway — PRD

Version: v0.4  
Date: 2025-12-16  
Status: Production-ready draft

## Table of Contents

- [1. Product overview](#1-product-overview)
- [2. Goals and non-goals](#2-goals-and-non-goals)
- [3. Personas and primary use cases](#3-personas-and-primary-use-cases)
- [4. UX flows](#4-ux-flows)
- [5. Release scope](#5-release-scope)
- [6. Functional requirements](#6-functional-requirements)
- [7. Non-functional requirements](#7-non-functional-requirements)
- [8. Requirements traceability maps](#8-requirements-traceability-maps)
- [9. Acceptance criteria](#9-acceptance-criteria)
- [10. Open questions](#10-open-questions)

---

## 1. Product overview

### 1.1 Summary

A cross-platform SSH client for **Desktop (Windows/macOS/Linux)**, **Mobile (iOS/Android)**, and **Web (browser)** with:

- a unified **Host Catalog** (hosts, tags, favorites, connection profiles),
- secure management of **keys and secrets**,
- interactive **terminal sessions** (PTY),
- end-to-end encrypted (**E2EE**) **sync** of user data across devices.

Web sessions run via **WebSSH Gateway** (WSS ↔ SSH bridge):
`Browser (Web) ↔ WSS ↔ WebSSH Gateway ↔ SSH Server`.

### 1.2 Product principles

- **Security-by-default**: host key verification, secrets isolation, E2EE sync.
- **Single source of truth**: consistent data model and behavior across platforms.
- **Fast, familiar UX**: search-first navigation, minimal friction to connect.
- **Observability without leakage**: telemetry/logging never includes secrets or session content.

---

## 2. Goals and non-goals

### 2.1 Goals

| Goal ID | Goal | Success metric (v1 target) |
|---|---|---|
| G-01 | Unified host catalog across devices | Host edits appear on another device within 10s (p50), 60s (p95) |
| G-02 | Secure by default | 0 known secret leaks in logs/crash dumps in CI gates |
| G-03 | Reliable interactive terminal | PTY works with common interactive apps (top, vim basic) |
| G-04 | Web access without compromising keys | Web MVP never persists private keys; gateway key lifetime limited to session |

### 2.2 Non-goals (MVP/v1)

- Workspaces/organizational RBAC (consider v2).
- Persistent background SSH on mobile (platform constraints).
- Full OpenSSH config feature parity in MVP (incremental support).

---

## 3. Personas and primary use cases

### 3.1 Personas

- **P1: Individual developer** — connects to personal servers, uses keys, multiple devices.
- **P2: DevOps/SRE** — frequent connects, requires strict host key verification, needs quick reconnect and auditability.
- **P3: Support engineer** — often uses web access from locked-down machines; needs temporary key upload and clear UX warnings.

### 3.2 Primary use cases

1. Manage hosts (CRUD, tags, favorites), connect quickly.
2. Connect via password or key; host key verification must prevent MITM.
3. Sync hosts/snippets/settings across devices with E2EE (server never sees plaintext).
4. Web terminal via gateway with quotas and strong session controls.

---

## 4. UX flows

### 4.1 Flow: First-run onboarding (all platforms)

1. Create account / sign in.
2. Create or unlock Vault (master password).
3. (Desktop/Mobile) Import/generate SSH key (optional for MVP).
4. Add first Host or import minimal `~/.ssh/config` (Desktop MVP).

**Success**: user reaches a working terminal session in ≤ 90 seconds (first-time, p50).

### 4.2 Flow: Add host and connect (Desktop/Mobile)

1. Host List → “Add Host”
2. Fill `title`, `hostname`, `user`, optional `port`.
3. Choose Auth method: Password or Key.
4. Choose Host Key policy: default `accept-new`, allow `ask` and `strict`.
5. Connect → Terminal.

### 4.3 Flow: Host key verification prompt (policy=ask)

Trigger: unknown host key OR changed key.

1. Show fingerprint, reason (NEW/CHANGED), recommended action.
2. User chooses **Accept** or **Reject**.
3. On accept: persist known_hosts entry (vault).
4. On reject: abort connection.

### 4.4 Flow: Web connect with temporary key upload (Web MVP)

1. Web → select Host from synced catalog.
2. Choose Auth: Password or “Upload key (temporary)”.
3. If key upload: show explicit warning “Key lives only for this session; not stored in browser”.
4. Web opens WSS to Gateway and begins session.
5. On disconnect/timeout: gateway wipes key material.

---

## 5. Release scope

### 5.1 MVP scope

## Desktop MVP

- Host Catalog: CRUD, search, tags, favorites.
- SSH terminal: interactive PTY, tabs.
- Auth: password + key (Ed25519 required; RSA optional).
- Host key verification: `ask` + `accept-new` default; `strict` optional.
- Local vault: hosts/snippets/ui_settings/known_hosts.
- E2EE sync: hosts + snippets + ui_settings (private keys not synced by default).

## Web MVP

- Host catalog from sync + terminal via gateway.
- Auth: password OR temporary key upload (key stored only in gateway RAM).
- No SFTP, no port forwarding, no agent forwarding.
- Clear UX about limitations and policies.

## Mobile MVP

- Host catalog + terminal + sync.
- No SFTP/forwarding.

### 5.2 v1 scope (post-MVP)

- SFTP manager (Desktop full, Mobile/Web simplified).
- Port forwarding (Desktop: local + SOCKS; Mobile/Web limited by platform).
- ProxyJump 1 hop guaranteed; multi-hop optional.
- Improved conflict resolution (UI for manual merge).
- Expanded ssh_config import coverage.

---

## 6. Functional requirements

### 6.1 Requirement model and priorities

- IDs: `FR-###`  
- Priority: **P0 (MUST)**, **P1 (SHOULD)**, **P2 (COULD)**  
- Release: `MVP` or `v1`  

#### 6.1.1 Host catalog

| ID | Priority | Release | Requirement |
|---|---:|---:|---|
| FR-001 | P0 | MVP | CRUD Host records with tombstones (soft delete) |
| FR-002 | P0 | MVP | Search by title/hostname/username/tags |
| FR-003 | P0 | MVP | Tags, favorites, basic sorting |
| FR-004 | P0 | MVP | Connection params: host/port/user |
| FR-006 | P1 | v1 | Profiles/templates (prod/stage/dev) |
| FR-007 | P1 | v1 | Advanced SSH options (ciphers/KEX/MAC) |

#### 6.1.2 Keys and secrets

| ID | Priority | Release | Requirement |
|---|---:|---:|---|
| FR-020 | P0 | MVP | Import OpenSSH Ed25519 key; store private key via platform secure storage |
| FR-023 | P0 | MVP | Passphrase support for encrypted keys |
| FR-024 | P0 | MVP | No secrets in logs/telemetry/crash reports (best effort + tests) |
| FR-025 | P0 | MVP | Private keys are **not** synced by default |
| FR-026 | P2 | v1+ | Optional “Sync private keys inside E2EE vault” (explicit opt-in + re-auth) |

#### 6.1.3 Terminal / SSH sessions

| ID | Priority | Release | Requirement |
|---|---:|---:|---|
| FR-040 | P0 | MVP | Interactive PTY: stdin/stdout, resize, exit status |
| FR-041 | P0 | MVP | UTF-8 correctness |
| FR-042 | P0 | MVP | Clipboard copy/paste (desktop/web) incl. bracketed paste |
| FR-043 | P1 | v1 | Connection history and quick reconnect |
| FR-044 | P2 | v1 | Optional session recording (without secrets) |

#### 6.1.4 Host key verification (anti-MITM)

| ID | Priority | Release | Requirement |
|---|---:|---:|---|
| FR-060 | P0 | MVP | Policies: strict / accept-new / ask |
| FR-061 | P0 | MVP | Changed fingerprint blocks (strict) or prompts (ask) |
| FR-062 | P0 | MVP | Persist known_hosts entries in vault; support pinned fingerprints |

#### 6.1.5 Sync (E2EE)

| ID | Priority | Release | Requirement |
|---|---:|---:|---|
| FR-140 | P0 | MVP | E2EE for synced data (hosts/snippets/ui_settings) |
| FR-141 | P0 | MVP | Server stores only encrypted blobs + minimal metadata |
| FR-142 | P0 | MVP | Incremental sync using server cursors |
| FR-143 | P0 | MVP | Tombstones for deletions |
| FR-144 | P0 | MVP | Conflicts: auto-merge where safe; fallback LWW; manual merge UI in v1 |

#### 6.1.6 WebSSH Gateway

| ID | Priority | Release | Requirement |
|---|---:|---:|---|
| FR-160 | P0 | MVP | Browser connects to gateway via WSS |
| FR-161 | P0 | MVP | Gateway establishes SSH and streams I/O |
| FR-162 | P0 | MVP | Enforce quotas: duration/bytes/concurrency per user/device |
| FR-163 | P0 | MVP | Web key handling: key only in gateway RAM; wiped on end |
| FR-164 | P0 | MVP | Gateway auth via JWT from sync-api; bind WSS to userId/deviceId |
| FR-165 | P0 | MVP | Gateway rejects stdin/resize before READY and returns clear error codes |

---

## 7. Non-functional requirements

### 7.1 NFR catalog

IDs: `NFR-###`, with targets where meaningful.

| ID | Priority | Release | Requirement |
|---|---:|---:|---|
| NFR-001 | P0 | MVP | Secrets never appear in logs/telemetry/crash reports (gated) |
| NFR-003 | P0 | MVP | Reliability: keepalive + reconnect strategy (desktop/web) |
| NFR-004 | P0 | MVP | Secure storage for private keys and passwords (platform-native) |
| NFR-005 | P0 | MVP | Vault format versioning + migrations |
| NFR-007 | P0 | MVP | Rate limiting on sync-api and gateway with predictable errors |
| NFR-002 | P1 | v1 | Performance targets: Desktop start <2s; Web TTI <2.5s (p50, target) |
| NFR-006 | P1 | v1 | Accessibility: scaling and contrast compliance (baseline) |

---

## 8. Requirements traceability maps

### 8.1 FR → Platform/Module coverage (MVP)

Legend: ✅ implemented, ❌ not in MVP, ◐ partial.

| FR | Desktop | Mobile | Web | Gateway | Core SDK | Sync API |
|---|---|---|---|---|---|---|
| FR-001 | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ |
| FR-040 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| FR-060 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| FR-140 | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ |
| FR-163 | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ |
| FR-165 | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ |

### 8.2 FR → Implementation modules

| FR group | Primary module(s) |
|---|---|
| Host catalog | `core/vault`, `core/sync-client`, `apps/*` |
| Terminal/SSH | `core/ssh-core`, `apps/* terminal`, `server/webssh-gateway` (web) |
| Host key verification | `core/ssh-core`, `core/vault` |
| Sync (E2EE) | `core/vault`, `core/sync-client`, `server/sync-api` |
| Web Gateway | `server/webssh-gateway`, `apps/web` |

---

## 9. Acceptance criteria

**Note**: Detailed protocol/API acceptance tests live in **SRS.md**; this section defines product-level acceptance.

| AC ID | Scenario | Pass criteria |
|---|---|---|
| AC-001 | Import minimal `~/.ssh/config` (Desktop) | Creates Host entries (HostName/User/Port/IdentityFile/ProxyJump 1 hop) and connects successfully |
| AC-002 | Host key changed with `strict` | Connection is blocked; user sees explicit “host key changed” error; no auto-accept |
| AC-003 | E2EE sync between two devices | Host edit on device A appears on device B; server cannot decrypt blobs (validated via server DB inspection tests) |
| AC-004 | Web terminal via gateway | Interactive PTY works; gateway enforces quota and returns correct error codes |
| AC-005 | Secrets redaction | CI gate fails if secrets appear in logs/telemetry/crash payloads (automated tests) |
| AC-006 | Web temp key upload | Key is never persisted in browser; gateway wipes key after session end; verified by tests and code audit hooks |

### 9.1 Examples (acceptance snippets)

- If `hostKeyPolicy=strict` and server fingerprint differs, the UI must show:
  - Fingerprint old/new, reason CHANGED,
  - Blocked action path (no “Continue” without explicit override configured).
- Web terminal must refuse to send user keystrokes if gateway advertises `flow_control.windowBytes=0`.

---

## 10. Open questions

These are intentionally carried as tracked decisions (see `IMPLEMENTATION_PLAN.md` Decision Log).

1. Final SSH library choice (russh vs libssh2) and compatibility matrix.
2. Whether `known_hosts` should sync in MVP or remain local-only.
3. JWT transport for web (HttpOnly cookie vs header) to reduce XSS token theft risk.
