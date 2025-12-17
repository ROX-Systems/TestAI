# SSH Client + WebSSH Gateway — SRS

Version: v0.4  
Date: 2025-12-16  
Status: Production-ready draft

## Table of Contents

- [1. Introduction](#1-introduction)
- [2. Glossary](#2-glossary)
- [3. Assumptions and constraints](#3-assumptions-and-constraints)
- [4. System context and architecture](#4-system-context-and-architecture)
- [5. Data model and storage](#5-data-model-and-storage)
- [6. Security requirements](#6-security-requirements)
- [7. Core SDK interfaces](#7-core-sdk-interfaces)
- [8. Sync API](#8-sync-api)
- [9. WebSSH Gateway protocol (WSS)](#9-webssh-gateway-protocol-wss)
- [10. Platform-specific constraints](#10-platform-specific-constraints)
- [11. Logging, telemetry, and audit](#11-logging-telemetry-and-audit)
- [12. Error model](#12-error-model)
- [13. Versioning and migrations](#13-versioning-and-migrations)
- [14. Testability and compliance](#14-testability-and-compliance)

---

## 1. Introduction

### 1.1 Purpose

This document defines **software requirements** for:

- Cross-platform SSH client apps (Desktop/Mobile/Web)
- Shared Core SDK (Rust)
- Sync backend (sync-api)
- WebSSH Gateway (WSS ↔ SSH bridge)

Product-level scope and priorities are defined in **PRD.md**.

### 1.2 Scope

MVP must deliver:

- Interactive SSH terminal on all platforms
- E2EE sync for catalog data (hosts/snippets/ui_settings)
- Web access via gateway with strict security invariants

---

## 2. Glossary

| Term | Definition |
|---|---|
| Vault | Encrypted local store for user data (hosts/snippets/settings/known_hosts) |
| E2EE | End-to-end encryption where server stores only ciphertext |
| Device | A logical client installation identified by `deviceId` |
| Host Catalog | Collection of Host entities and related metadata |
| Known Hosts | Store of known host keys / fingerprints |
| Host Key Policy | `strict`, `accept-new`, or `ask` |
| PTY | Pseudoterminal used by SSH for interactive terminal behavior |
| WSS | WebSocket Secure |
| Gateway | WebSSH gateway bridging WSS messages to SSH sessions |
| Cursor | Monotonic server sequence used for incremental sync |
| Tombstone | Soft-deleted record preserved for sync convergence |

---

## 3. Assumptions and constraints

### 3.1 Assumptions

- Client devices and browsers can be compromised; system reduces risk but cannot fully eliminate it.
- Server components are **zero-trust for plaintext**: sync-api never decrypts vault data.
- Web clients are subject to XSS risk; CSP and token handling must reduce impact.

### 3.2 Constraints

- Web client cannot open raw TCP to SSH servers; **must** use gateway.
- Mobile background session persistence is limited by OS.
- Cryptographic parameters must be versioned to allow migrations and future suite upgrades.

---

## 4. System context and architecture

### 4.1 High-level diagram

```plaintext
Desktop App ─┐
Mobile App  ─┼── HTTPS ──► Sync API ──► Postgres (ciphertext + metadata)
Web App     ─┘

Web App ── WSS ──► WebSSH Gateway ── TCP ──► SSH Server
Desktop/Mobile may also connect directly to SSH Server over TCP.
```

### 4.2 Modules

- `core/ssh-core`: SSH connection, auth, PTY, SFTP (v1), forwarding (v1), host key verification
- `core/vault`: E2EE vault encryption/decryption, format versioning, migrations
- `core/sync-client`: oplog, push/pull, conflict handling
- `server/sync-api`: auth (JWT), device registration, sync endpoints, quotas/rate limits
- `server/webssh-gateway`: WSS protocol server, session limits, streaming bridge

---

## 5. Data model and storage

### 5.1 Entities (logical)

#### 5.1.1 Host

```yaml
Host:
  id: UUID
  title: string
  tags: string[]
  favorite: bool
  hostname: string
  port: int
  username: string
  auth:
    type: password|key
    keyId?: UUID
  proxy:
    type: none|jump
    jumpHostId?: UUID
  options:
    keepAliveSec?: int
    timeoutSec?: int
    compression?: bool
    strictHostKeyMode?: strict|accept-new|ask
  version: int
  deleted: bool  # tombstone
  createdAt: RFC3339
  updatedAt: RFC3339
```

#### 5.1.2 Key (metadata only in vault; private material in secure storage)

```yaml
Key:
  id: UUID
  name: string
  type: ed25519|rsa
  publicKey: string
  fingerprint: string
  privateRef: string  # handle in secure storage
  createdAt: RFC3339
```

#### 5.1.3 KnownHost

```yaml
KnownHost:
  id: UUID
  hostPattern: string
  keyType: string
  publicKey: string
  fingerprint: string
  pinned: bool
  addedAt: RFC3339
  lastSeenAt: RFC3339
```

### 5.2 Storage responsibilities

- Desktop/Mobile: local DB (SQLite) holds encrypted vault blobs and indexes.
- Web: IndexedDB holds encrypted vault blobs; master key never persists in plaintext.
- Private keys: **secure storage only** (Desktop/Mobile). Web MVP: no persistent storage for keys.

---

## 6. Security requirements

### 6.1 Security invariants (MUST)

| SEC ID | Invariant |
|---|---|
| SEC-001 | Sync server must never access plaintext vault contents |
| SEC-002 | Private keys are not synced by default |
| SEC-003 | Web MVP: private key exists only in gateway RAM and is wiped on session end |
| SEC-004 | Gateway must not persist stdin/stdout/stderr, and must not log secrets |
| SEC-005 | Host key verification is enforced per policy; changed keys require explicit block/prompt |
| SEC-006 | Secrets never appear in logs/traces/telemetry/crash reports (best effort + gating tests) |
| SEC-007 | Gateway enforces state machine: stdin/resize only in READY |
| SEC-008 | JWT validation includes exp; device binding is enforced (userId, deviceId) |

### 6.2 Host key verification requirements

- Fingerprint algorithm: **SHA256** fingerprints (OpenSSH style).
- Policies:
  - `strict`: if unknown or changed → fail with `HOSTKEY_CHANGED` or `HOSTKEY_UNKNOWN`
  - `accept-new`: unknown accepted once; changed fails
  - `ask`: unknown/changed prompts user and awaits decision
- Known hosts are stored in vault (`KnownHost` collection), with optional `pinned=true` (treat as strict for that hostPattern).

### 6.3 Key handling requirements

- Decrypted private key bytes must:
  - be held in memory for minimal time (best effort),
  - be wiped after use where feasible (secure zeroization libraries),
  - never be copied to logs, crash reports, telemetry, or persistence.

### 6.4 Web security controls (baseline)

- Strict CSP (no inline scripts; allow-list origins).
- Avoid storing JWT in JS-readable storage if possible.
- Sanitize any user-supplied strings rendered in DOM (tags, titles, hostnames).

---

## 7. Core SDK interfaces

### 7.1 `ssh-core` API (Rust)

Normative interface (language bindings must preserve semantics).

#### 7.1.1 Types

```rust
pub enum HostKeyPolicy { Strict, AcceptNew, Ask }
pub struct Pty { pub cols: u16, pub rows: u16, pub term: String }

pub enum SshEvent {
  Status { state: SshState },
  Stdout { data: Vec<u8> },
  Stderr { data: Vec<u8> },
  HostKeyPrompt { fingerprint: String, reason: HostKeyReason },
  Exit { exit_code: i32, signal: Option<String> },
  Error { code: SshErrorCode, message: String, retryable: bool }
}

pub enum SshState { Init, Connecting, HostKeyPrompt, Ready, Closing, Closed }
```

#### 7.1.2 Session API

```rust
pub struct SshSession;

impl SshSession {
  pub async fn connect(host: &str, port: u16, user: &str, timeout_ms: u32) -> Result<Self, SshError>;
  pub async fn auth_password(&mut self, password: SecretString) -> Result<(), SshError>;
  pub async fn auth_key(&mut self, key: PrivateKeyRef, passphrase: Option<SecretString>) -> Result<(), SshError>;

  pub async fn verify_host_key(&mut self, policy: HostKeyPolicy, known: Option<KnownHostEntry>)
      -> Result<HostKeyDecision, SshError>;
  pub async fn host_key_accept(&mut self) -> Result<(), SshError>;
  pub async fn host_key_reject(&mut self) -> Result<(), SshError>;

  pub async fn open_pty(&mut self, pty: Pty) -> Result<(), SshError>;
  pub async fn write_stdin(&mut self, data: &[u8]) -> Result<(), SshError>;
  pub async fn resize(&mut self, cols: u16, rows: u16) -> Result<(), SshError>;
  pub async fn disconnect(&mut self) -> Result<(), SshError>;

  pub fn subscribe_events(&self) -> EventStream<SshEvent>;
}
```

#### 7.1.3 Behavioral requirements

- `write_stdin` and `resize` MUST fail with `NOT_READY` unless state is `Ready`.
- `verify_host_key` MUST yield deterministic results given same known_hosts and server key.
- `HostKeyPrompt` event MUST contain fingerprint and reason (NEW/CHANGED).
- Errors MUST be stable and machine-readable (see [12](#12-error-model)).

### 7.2 `vault` API (Rust)

#### 7.2.1 Vault header

```yaml
VaultHeader:
  vaultVersion: int
  kdfParamsVersion: int
  cipherSuiteVersion: int
  createdAt: RFC3339
```

#### 7.2.2 Normative operations

```rust
pub struct Vault;

impl Vault {
  pub fn create(master_password: SecretString) -> Result<Vault, VaultError>;
  pub fn unlock(master_password: SecretString) -> Result<VaultSession, VaultError>;
  pub fn rotate_master_password(session: &mut VaultSession, new_password: SecretString) -> Result<(), VaultError>;

  pub fn encrypt_entity<T: Serialize>(session: &VaultSession, entity: &T) -> Result<Vec<u8>, VaultError>;
  pub fn decrypt_entity<T: DeserializeOwned>(session: &VaultSession, blob: &[u8]) -> Result<T, VaultError>;

  pub fn migrate(blob: &[u8]) -> Result<Vec<u8>, VaultError>; // read old → write new
}
```

#### 7.2.3 Cryptography requirements

- KDF: Argon2id; parameters MUST be versioned and included in header.
- AEAD: AES-256-GCM or ChaCha20-Poly1305; suite selection MUST be versioned.
- Payload serialization: CBOR preferred (JSON allowed only if explicitly versioned).

### 7.3 `sync-client` API

#### 7.3.1 Oplog

- Client records operations as:

```yaml
SyncOp:
  entityType: hosts|snippets|ui_settings|known_hosts
  entityId: UUID
  revision: int
  tombstone: bool
  encryptedBlob: bytes
  deviceId: string
  clientTime?: RFC3339
```

#### 7.3.2 Push/Pull algorithm (normative)

```Pseudo-code:
push():
  batch = oplog.pending(limit)
  resp = POST /sync/push { batch }
  mark_acked(resp.ackSeq)

pull():
  resp = POST /sync/pull { afterSeq: lastSeq, limit }
  apply_ops_in_order(resp.ops)
  lastSeq = resp.lastSeq
```

#### 7.3.3 Conflict rules (MVP)

- Prefer safe **field-level auto-merge** when changes do not overlap.
- Otherwise use **LWW** based on server ordering (`serverSeq`), not device clocks.
- v1 adds manual resolution UI and `/sync/resolve`.

---

## 8. Sync API

### 8.1 Authentication

- Access token: JWT with `userId`, `deviceId`, `exp` and SHOULD include `jti`.
- Refresh token: separate, long-lived, stored in secure storage.

### 8.2 Endpoints (MVP)

**Note**: This is a normative contract; OpenAPI should be generated to match.

#### 8.2.1 `POST /auth/register`

Request:

```json
{ "email":"...", "password":"...", "deviceName":"..." }
```

Response:

```json
{ "accessToken":"...", "refreshToken":"...", "deviceId":"..." }
```

#### 8.2.2 `POST /auth/login`

Same shape as register.

#### 8.2.3 `POST /auth/refresh`

Request:

```json
{ "refreshToken":"...", "deviceId":"..." }
```

Response:

```json
{ "accessToken":"..." }
```

#### 8.2.4 `GET /sync/state`

Response:

```json
{ "userId":"...", "deviceId":"...", "lastServerSeq": 12345 }
```

#### 8.2.5 `POST /sync/push`

Request:

```json
{ "ops": [ /* SyncOp */ ] }
```

Response:

```json
{ "ackSeq": 12345, "rejected": [] }
```

#### 8.2.6 `POST /sync/pull`

Request:

```json
{ "afterSeq": 12345, "limit": 500 }
```

Response:

```json
{ "ops": [ /* server ops */ ], "lastSeq": 12500 }
```

### 8.3 Storage schema (minimum)

- `users`
- `devices (userId, deviceId, deviceName, createdAt, revokedAt?)`
- `sync_ops (userId, entityType, entityId, revision, tombstone, encryptedBlob, deviceId, serverSeq, serverTime)`
- `device_state (userId, deviceId, lastAckSeq)`

### 8.4 Rate limiting requirements

- Return HTTP 429 with structured retry info:

```json
{ "code":"RATE_LIMITED", "retryAfterMs": 1000 }
```

---

## 9. WebSSH Gateway protocol (WSS)

### 9.1 Transport

- Transport: WebSocket Secure (WSS)
- Encoding: JSON (MVP). Binary data in base64 fields named `dataBase64`.

### 9.2 Common envelope

All messages MUST include:

- `type: string`
- `requestId: string` (UUID) — correlation + idempotency
- `protocolVersion: string` (e.g. "0.4")
- `payload: object`

### 9.3 Idempotency rules

Gateway MUST cache results per `requestId` within the WSS session lifetime:

- Repeat with same `requestId` → return identical response without re-execution.
- For `stdin`: at-least-once safe; repeats must not duplicate input (track `lastAppliedStdinRequestId` per `sessionId`).

### 9.4 Session state machine (per `sessionId`)

States:

- `INIT`, `CONNECTING`, `HOSTKEY_PROMPT`, `READY`, `CLOSING`, `CLOSED`

Transitions:
`INIT` → connect → `CONNECTING`  
`CONNECTING` → hostkey_prompt → `HOSTKEY_PROMPT`  
`HOSTKEY_PROMPT` → accept/reject → `CONNECTING` / `CLOSED`  
`CONNECTING` → READY → `READY`  
`READY` → disconnect/timeout/error/exit → `CLOSING` → `CLOSED`

Validation:

- `connect` forbidden before successful auth (if auth uses a message).
- `stdin` and `resize` allowed only in `READY`.
- Unknown `sessionId` → `error` with `BAD_REQUEST`.
- Repeated `connect` for active `sessionId`:
  - identical params → return current `status` (idempotent replay)
  - different params → `error` `SESSION_CONFLICT`

### 9.5 Authentication

Client provides JWT access token:

- Prefer header: `Authorization: Bearer <token>`, or
- First message: `auth`

Token MUST include `userId`, `deviceId`, `exp` (SHOULD include `jti`).
Gateway binds WSS session to `(userId, deviceId)` after validation.

### 9.6 Heartbeat

- client → gateway: `ping`
- gateway → client: `pong`
- If no activity for `idleTimeoutSec` → close WSS.

### 9.7 Message definitions (normative)

#### 9.7.1 `auth` (optional)

Client → gateway:

```json
{ "type":"auth", "requestId":"...", "protocolVersion":"0.4", "payload": { "token":"..." } }
```

Gateway → client:

```json
{ "type":"auth_ok", "requestId":"...", "protocolVersion":"0.4", "payload": { "userId":"...", "deviceId":"..." } }
```

Or error (see `error`).

#### 9.7.2 `ping` / `pong`

Client → gateway:

```json
{ "type":"ping", "requestId":"...", "protocolVersion":"0.4", "payload": {} }
```

Gateway → client:

```json
{ "type":"pong", "requestId":"...", "protocolVersion":"0.4", "payload": { "serverTime":"2025-12-16T03:00:00Z" } }
```

#### 9.7.3 `connect`

Client → gateway:

```json
{
  "type":"connect",
  "requestId":"...",
  "protocolVersion":"0.4",
  "payload":{
    "sessionId":"uuid",
    "host":"example.com",
    "port":22,
    "user":"root",
    "pty":{"cols":120,"rows":30,"term":"xterm-256color"},
    "hostKeyPolicy":"ask|accept-new|strict",
    "knownHost":{"fingerprint":"optional"},
    "auth":{
      "type":"password|temp_key",
      "password":"...",
      "tempKey":{
        "privateKeyPemBase64":"...",
        "passphrase":"optional"
      }
    }
  }
}
```

Gateway lifecycle responses:

```json
{ "type":"status", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"...", "state":"CONNECTING" } }
```

```json
{ "type":"status", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"...", "state":"READY" } }
```

#### 9.7.4 `stdout` / `stderr`

Gateway → client:

```json
{ "type":"stdout", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"...", "dataBase64":"..." } }
```

```json
{ "type":"stderr", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"...", "dataBase64":"..." } }
```

#### 9.7.5 `stdin`

Client → gateway:

```json
{ "type":"stdin", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"...", "dataBase64":"..." } }
```

If sent before `READY` → `error` with code `NOT_READY`.

#### 9.7.6 `resize`

Client → gateway:

```json
{ "type":"resize", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"...", "cols":120, "rows":30 } }
```

#### 9.7.7 `disconnect`

Client → gateway:

```json
{ "type":"disconnect", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"..." } }
```

#### 9.7.8 `exit`

Gateway → client:

```json
{ "type":"exit", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"...", "exitCode":0, "signal":null } }
```

#### 9.7.9 `error`

Gateway → client:

```json
{
  "type":"error",
  "requestId":"...",
  "protocolVersion":"0.4",
  "payload":{
    "sessionId":"optional",
    "code":"HOSTKEY_CHANGED",
    "message":"...",
    "retryable":false,
    "retryAfterMs":0
  }
}
```

- `retryAfterMs` MUST be set for `RATE_LIMITED` and MAY be set for `TIMEOUT`.
- `retryable` guides UX (e.g., show “Reconnect”).

#### 9.7.10 `flow_control` (MVP+)

Gateway → client:

```json
{
  "type":"flow_control",
  "requestId":"...",
  "protocolVersion":"0.4",
  "payload":{
    "sessionId":"...",
    "windowBytes":65536
  }
}
```

If `windowBytes=0`, client MUST pause stdin sending.

### 9.8 Host key prompt flow

If policy requires user decision:
Gateway → client:

```json
{ "type":"hostkey_prompt", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"...", "fingerprint":"...", "reason":"NEW|CHANGED" } }
```

Client → gateway:

```json
{ "type":"hostkey_accept", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"..." } }
```

Client → gateway:

```json
{ "type":"hostkey_reject", "requestId":"...", "protocolVersion":"0.4", "payload": { "sessionId":"..." } }
```

### 9.9 Quotas and limits

Configurable minimum limits:

- `maxConcurrentSessionsPerUser`
- `maxSessionDurationSec`
- `maxBytesPerSession`
- `maxMessageSizeBytes`
- `idleTimeoutSec`

Exceeding MUST return `error` with `QUOTA_EXCEEDED` or `RATE_LIMITED`.

### 9.10 Gateway secret handling (temp_key)

- `tempKey` material MUST exist only in gateway RAM.
- Must be wiped on `disconnect`, `exit`, timeout, error, or heartbeat timeout.
- Must not be logged; must not be serialized in panic dumps (best effort).

---

## 10. Platform-specific constraints

### 10.1 Web

- No persistent private key storage in MVP.
- No agent forwarding (MVP).
- No background reconnect guarantee.
- All SSH sessions only via gateway.

### 10.2 Mobile

- Background execution constraints; sessions may be suspended.
- Secure storage must use platform keychain/keystore.

### 10.3 Desktop

- Must integrate with OS secure storage APIs.
- Should support xterm.js terminal behavior consistently.

---

## 11. Logging, telemetry, and audit

### 11.1 Logging levels and redaction

- All logs must be structured.
- Must not log:
  - passwords, private key material, decrypted vault blobs, terminal I/O.
- Allowed metadata:
  - userId/deviceId (hashed if required), host alias (not raw hostname if sensitive), durations, error codes.

### 11.2 Audit events (minimum)

| Event | Fields |
|---|---|
| user_login | userId, deviceId, time |
| device_registered | userId, deviceId, deviceName |
| sync_push | userId, deviceId, opsCount, sizeBytes |
| sync_pull | userId, deviceId, opsCount |
| gateway_session_start | userId, deviceId, sessionId, host, policy |
| gateway_session_end | userId, deviceId, sessionId, duration, exitCode, errorCode |

---

## 12. Error model

### 12.1 Canonical error codes

#### 12.1.1 Gateway error codes (minimum)

- AUTH_FAILED
- UNSUPPORTED_PROTOCOL
- BAD_REQUEST
- NOT_READY
- SESSION_CONFLICT
- CONNECT_FAILED
- DNS_FAILED
- TIMEOUT
- HOSTKEY_CHANGED
- HOSTKEY_UNKNOWN
- HOSTKEY_REJECTED
- QUOTA_EXCEEDED
- RATE_LIMITED
- INTERNAL_ERROR

#### 12.1.2 Sync API error codes (minimum)

- AUTH_FAILED
- DEVICE_REVOKED
- RATE_LIMITED
- BAD_REQUEST
- INTERNAL_ERROR

### 12.2 HTTP status mapping (sync-api)

- 400 → BAD_REQUEST
- 401 → AUTH_FAILED
- 403 → DEVICE_REVOKED (or AUTH_FAILED depending on threat model)
- 429 → RATE_LIMITED (include `retryAfterMs`)
- 500 → INTERNAL_ERROR

---

## 13. Versioning and migrations

### 13.1 Vault versioning

- `vaultVersion` increments on format changes.
- `kdfParamsVersion` increments on KDF parameter changes.
- `cipherSuiteVersion` increments on cipher suite changes.

### 13.2 Protocol versioning

- Gateway protocol uses `protocolVersion` string.
- Client MUST fail closed on unsupported versions with a clear UX error.

---

## 14. Testability and compliance

### 14.1 Required test suites (MVP)

- Integration tests with dockerized OpenSSH server for:
  - interactive PTY + resize,
  - host key accept-new and strict changed-key block.
- Gateway state machine tests:
  - reject stdin before READY,
  - idempotent replay of connect,
  - quota enforcement.
- Secret leakage tests:
  - grep logs for high-entropy secrets / known test keys,
  - block CI if detected.
- Dependency audits:
  - `cargo audit`, `npm audit` gating (policy in Implementation Plan).

### 14.2 Release “no-go” conditions

See `IMPLEMENTATION_PLAN.md` for operational checklist and gates.
