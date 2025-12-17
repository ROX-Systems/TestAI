# SSH Client + WebSSH Gateway — Implementation Plan
Version: v0.4  
Date: 2025-12-16  
Status: Execution plan (MVP → v1)

## Table of Contents
- [1. Milestones overview (M0–M7)](#1-milestones-overview-m0m7)
- [2. Dependencies and critical path](#2-dependencies-and-critical-path)
- [3. Definition of Done (DoD) by module](#3-definition-of-done-dod-by-module)
- [4. CI/CD and quality gates](#4-cicd-and-quality-gates)
- [5. Security audits and release gates](#5-security-audits-and-release-gates)
- [6. Operational constraints](#6-operational-constraints)
- [7. Decision log (required)](#7-decision-log-required)

---

## 1. Milestones overview (M0–M7)

### M0 — Repo, CI, docs
**Deliverables**
- Monorepo skeleton: `core/`, `apps/`, `server/`, `packages/`, `docs/`
- CI pipelines:
  - Rust: `fmt`, `clippy`, `test`, `cargo audit`
  - TS: `eslint`, `typecheck`, `test`, `npm audit`
  - Flutter: `analyze`, `test`
- Documentation set: `PRD.md`, `SRS.md`, `THREAT_MODEL.md`, `IMPLEMENTATION_PLAN.md`

**Exit criteria**
- Main branch green on PRs (required checks).
- Dependency pinning + lockfiles enforced.

### M1 — `core/ssh-core` (interactive terminal + host key policy)
**Deliverables**
- SSH session API + event stream
- Host key verification policies and fingerprinting
- Docker integration tests with OpenSSH server

**Exit criteria**
- PTY interactive + resize works in docker tests.
- Strict policy blocks on host key change.
- Secret leakage tests for ssh-core pass.

### M2 — `core/vault` + local stores + KeyStore abstraction
**Deliverables**
- Vault format + crypto suite + migrations
- Local encrypted storage:
  - Desktop: SQLite
  - Mobile: SQLite
  - Web: IndexedDB
- KeyStore interface + desktop implementation

**Exit criteria**
- Vault created/unlocked/migrated across at least Desktop + Web.
- No plaintext persisted.
- Secure storage integration passes basic tests.

### M3 — `server/sync-api`
**Deliverables**
- Auth: register/login/refresh (JWT)
- Device registration + device revocation hooks (API placeholder ok for MVP)
- Sync endpoints: state/push/pull
- Postgres schema + migrations
- Rate limiting + audit logs

**Exit criteria**
- Two devices can push/pull encrypted blobs end-to-end.
- Contract tests for API pass (OpenAPI).

### M3.5 — `core/sync-client`
**Deliverables**
- Local oplog and deterministic application of operations
- Conflict handling MVP (auto-merge + LWW by serverSeq)
- Integration tests with two simulated devices

**Exit criteria**
- Conflict scenario is reproducible and passes tests.
- Tombstones converge correctly.

### M4 — Desktop MVP (Tauri)
**Deliverables**
- Hosts UI + editor
- Key import and secure storage
- Terminal UI (xterm.js) bridged to core
- Sync login/unlock/push/pull

**Exit criteria**
- User can add host → connect → interactive terminal.
- Host edits sync between two desktop profiles.

### M5 — WebSSH Gateway + Web MVP
**Deliverables**
- Gateway WSS server implementing SRS §9
- JWT auth bound to deviceId
- Quotas (duration/bytes/concurrency), message size limits, flow control
- Web app: login/unlock vault, host list, terminal via WSS
- UX warnings for temporary key upload and limitations

**Exit criteria**
- Web terminal stable; gateway returns correct error codes.
- Gateway refuses stdin before READY (automated tests).
- Temp keys wiped on all termination paths.

### M6 — Mobile MVP (Flutter)
**Deliverables**
- Host list/editor + sync
- Terminal widget + Rust FFI integration
- Document background limitations

**Exit criteria**
- Basic interactive terminal works (input/output/resize).
- Sync works for catalog data.

### M7 — v1 features
- SFTP: core + desktop full UI; mobile/web simplified
- Forwarding: desktop local + SOCKS; UI for active tunnels
- Improved conflict resolution (manual merge UI)
- Expanded ssh_config import

---

## 2. Dependencies and critical path

### 2.1 Critical path (MVP)
`M1 (ssh-core) → M2 (vault) → M3 (sync-api) → M5 (gateway)`

Web MVP is blocked until **M5**.

### 2.2 Key dependencies
- SSH library selection must be finalized during early M1 spike (record in decision log).
- Crypto suite (AES-GCM vs ChaCha20-Poly1305) must be fixed before M2 format freeze.
- Token strategy for web (cookie vs header) must be decided before M5 production hardening.

---

## 3. Definition of Done (DoD) by module

### 3.1 Global DoD (applies to all modules)
- Unit tests + integration tests where applicable.
- No secrets in logs/telemetry/crash artifacts (CI gate).
- Versioned formats and migrations (where data is persisted).
- Docs updated (PRD/SRS references remain consistent).
- Reproducible builds in CI.

### 3.2 Module-specific DoD
#### `core/ssh-core`
- PTY, resize, exit are deterministic across test SSH server.
- Host key verification policies behave exactly as SRS §6.2.
- Fuzz/property tests for protocol parsing (if applicable).
- Secret redaction tests pass.

#### `core/vault`
- Cross-platform compatibility test: Desktop encrypt → Web decrypt (and reverse).
- Migration tests: old version blobs migrate to new version.
- KDF and cipher suite versions enforced.

#### `server/sync-api`
- OpenAPI contract tests pass.
- Rate limiting is enabled and returns structured retry.
- Audit events emitted without secrets.
- DB migrations are idempotent and reviewed.

#### `server/webssh-gateway`
- Implements SRS §9 exactly (message schemas, state machine, idempotency).
- Quotas and message size enforcement tested.
- Temp key lifecycle: wipe on disconnect/exit/timeout/error.
- No terminal I/O persistence; log policy enforced by tests.

#### `apps/*`
- UX flows match PRD.
- Host key prompts are user-visible and unambiguous.
- Secure storage usage verified (Desktop/Mobile).
- Web limitations and warnings are visible.

---

## 4. CI/CD and quality gates

### 4.1 Required CI checks (merge gates)
- Rust: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo audit`
- Web/TS: `eslint`, `tsc --noEmit`, `test`, `npm audit` (policy-defined thresholds)
- Flutter: `flutter analyze`, `flutter test`
- Secret scan:
  - static grep for known test private keys,
  - high-entropy detection for logs/fixtures (policy),
  - block merge on findings.

### 4.2 Artifact builds
- Desktop: signed builds (later stage; MVP can be unsigned in CI artifacts).
- Web: deterministic build; SRI hashes for critical bundles (v1 hardening).
- Server: container images with pinned base images.

---

## 5. Security audits and release gates

### 5.1 Minimum audits before MVP release
- Dependency audit: cargo/npm audits green (policy-defined).
- Gateway protocol security review:
  - state machine correctness,
  - idempotency semantics,
  - quota enforcement,
  - token binding.
- Basic web security review:
  - CSP present and strict,
  - no inline scripts,
  - DOM sanitization for user strings.

### 5.2 Release “no-go” conditions (MVP)
- Secrets appear in logs/traces/crash artifacts.
- Host key changes not detected or not blocked/prompted per policy.
- Gateway accepts stdin before READY.
- Quota bypass is reproducible.
- Unsupported protocol versions fail open (must fail closed).

---

## 6. Operational constraints

### 6.1 Gateway operational requirements
- Hard limits configurable at runtime:
  - concurrent sessions, bytes per session, session duration, max message size, idle timeout
- Observability:
  - metrics: active sessions, bytes in/out, auth failures, quota events, error codes
  - logs: metadata only
- Deployment:
  - must run behind TLS termination if not terminating TLS itself
  - must enforce origin policy for web clients (where applicable)

### 6.2 Sync API operational requirements
- DB migrations tracked and reversible.
- Backups protect ciphertext + metadata only.
- Device revocation should invalidate access (v1: hard requirement).

---

## 7. Decision log (required)
Create `docs/decision-log.md` with entries:
- SSH implementation choice + compatibility notes
- Vault payload encoding choice (CBOR vs JSON) and rationale
- Web token handling choice (cookie vs header) and rationale
- Known_hosts sync decision (MVP vs v1)
