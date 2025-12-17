# AGENT-RUST-CORE — Rust Core разработчик

Версия: 0.1  
Дата: 2025-12-17  
Статус: Active (Stage 1)

## 0. Мандат

- Реализация и тестирование Rust Core SDK модулей.
- В рамках Stage 1: `core/ssh-core` (SSH connect/auth, PTY, host key verification) согласно `SRS.md`.

## 1. TODO History (append-only)

### Snapshot 2025-12-17 (Stage 1 kickoff)

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M1-RUST-001 | TODO | HIGH | Spike: сравнить `russh` vs `libssh2` под требования PTY/hostkey/auth/совместимость, оформить рекомендацию | PRD.md §10; IMPLEMENTATION_PLAN.md §2.2; `docs/decision-log.md` D-001 |
| M1-RUST-002 | TODO | HIGH | Зафиксировать целевые публичные типы/ошибки/события `ssh-core` (SRS §7.1) | SRS.md §7.1 |
| M1-RUST-003 | TODO | HIGH | Спроектировать и реализовать state machine `SshState` и запрет `write_stdin/resize` до READY | SRS.md §7.1.3; SEC-007 |
| M1-RUST-004 | TODO | HIGH | Реализовать host key verification policy (strict/accept-new/ask) и fingerprinting (SHA256) | SRS.md §6.2; FR-060..062 |
| M1-RUST-005 | TODO | HIGH | Добавить docker integration tests (OpenSSH): PTY+resize; strict changed-key block; accept-new accept-once | SRS.md §14.1; IMPLEMENTATION_PLAN.md M1 |
| M1-RUST-006 | TODO | HIGH | Добавить secret leakage tests/гарантии redaction в `ssh-core` | SEC-006; THREAT_MODEL.md T-003 |

### Snapshot 2025-12-17 (D-001 decided + scaffold)

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M1-RUST-001 | DONE | HIGH | Spike: сравнить `russh` vs `libssh2` под требования PTY/hostkey/auth/совместимость, оформить рекомендацию | `docs/decision-log.md` D-001 |
| M1-RUST-002 | DONE | HIGH | Зафиксировать целевые публичные типы/ошибки/события `ssh-core` (SRS §7.1) | `core/ssh-core/src/lib.rs`; SRS.md §7.1 |
| M1-RUST-003 | IN_PROGRESS | HIGH | Спроектировать и реализовать state machine `SshState` и запрет `write_stdin/resize` до READY | SRS.md §7.1.3; SEC-007 |
| M1-RUST-004 | TODO | HIGH | Реализовать host key verification policy (strict/accept-new/ask) и fingerprinting (SHA256) | SRS.md §6.2; FR-060..062 |
| M1-RUST-005 | TODO | HIGH | Добавить docker integration tests (OpenSSH): PTY+resize; strict changed-key block; accept-new accept-once | SRS.md §14.1; IMPLEMENTATION_PLAN.md M1 |
| M1-RUST-006 | TODO | HIGH | Добавить secret leakage tests/гарантии redaction в `ssh-core` | SEC-006; THREAT_MODEL.md T-003 |
