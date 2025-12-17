# Architecture Overview — SSH Client + WebSSH Gateway

Версия: 0.1  
Дата: 2025-12-17  
Статус: Active

## 0. Контекст

Система состоит из приложений (Desktop/Mobile/Web), общего Rust Core SDK и двух серверных компонентов (sync-api, webssh-gateway).

## 1. Высокоуровневая схема

- Desktop/Mobile/Web ↔ HTTPS ↔ `server/sync-api` ↔ Postgres (ciphertext + metadata)
- Web ↔ WSS ↔ `server/webssh-gateway` ↔ TCP ↔ SSH server
- Desktop/Mobile могут подключаться к SSH server напрямую по TCP.

## 2. Модули и ответственность

- `core/ssh-core`
  - SSH connect/auth, PTY, host key verification
  - (v1) SFTP/forwarding
  - (MVP) минимальный парсинг `~/.ssh/config` для импорта (Desktop)
- `core/vault`
  - E2EE, KDF+AEAD, версии форматов, миграции
- `core/sync-client`
  - oplog, push/pull по cursor, конфликтность (auto-merge + LWW by serverSeq)
- `server/sync-api`
  - auth (JWT), device binding, sync endpoints, rate limiting, audit events
- `server/webssh-gateway`
  - WSS протокол, state machine, идемпотентность, квоты, temp key lifecycle

## 3. Trust boundaries (критично)

- TB1: App ↔ Sync API (HTTPS)
- TB2: Web ↔ Gateway (WSS)
- TB3: Gateway ↔ SSH Server (TCP)
- TB4: Secure Storage (OS keychain/keystore)
- TB5: Browser JS (XSS boundary)

## 4. Критический путь MVP

`M1 (ssh-core) → M2 (vault) → M3 (sync-api) → M5 (gateway)`

## 5. Открытые архитектурные решения

См. `docs/decision-log.md`:

- D-001: выбор SSH реализации (`russh` vs `libssh2`)
- D-002: формат payload vault (CBOR vs JSON)
- D-003: JWT транспорт для Web (HttpOnly cookie vs header)
- D-004: синхронизация `known_hosts` в MVP
