# SSH Client — Project Specs (PRD+SRS)

Версия: 0.3  
Дата: 2025-12-16  
Статус: Draft for Implementation

## 0. Краткое описание

Кросс-платформенный SSH-клиент: Desktop (Windows/macOS/Linux), Mobile (iOS/Android), Web (браузер).

Функции:

- каталог хостов, ключи/секреты
- терминальные SSH-сессии
- SFTP (v1)
- туннели/форвардинги (v1)
- импорт OpenSSH config (MVP частично)
- синхронизация настроек между платформами с E2EE

Web-версия работает только через WebSSH Gateway:
браузер ↔ WSS ↔ gateway ↔ SSH-сервер

## 1. Цели и принципы

### 1.1 Цели продукта

- Единый каталог хостов и настроек на всех устройствах.
- Быстрое подключение и удобный UX: поиск/теги/избранное/шаблоны.
- Безопасность по умолчанию: проверка host key, защита секретов, E2EE-синхронизация.

### 1.2 Не цели (для MVP/v1)

- Организационные workspaces, RBAC (возможны v2).
- “24/7 фоновые” подключения на mobile (ограничения iOS/Android).
- Полная поддержка всех директив OpenSSH (MVP: минимальный набор, v1: расширение).

## 2. Security Invariants (MUST)

- Sync-сервер **никогда** не имеет доступа к plaintext данным vault (только encrypted blobs).
- Приватные ключи **не синхронизируются по умолчанию**.
- Web MVP: приватный ключ существует **только** как временная загрузка (RAM gateway) и не хранится персистентно в браузере.
- Gateway **не пишет** stdin/stdout/stderr в persistent storage.
- Любое изменение host key требует user-visible решения (ask/strict) или строгого поведения по policy.
- Секреты не должны попадать в логи/трейсы/телеметрию/краш-репорты (best effort + тесты).

## 3. Платформы и стек

### 3.1 Core SDK (общий)

- Rust: `ssh-core` (SSH PTY, SFTP, forwarding, host key verify, ssh_config parsing)
- SSH библиотека: `russh` (см. `docs/decision-log.md` D-001)
- Отпечаток ключа хоста: SHA256 (как в OpenSSH) — `sha2` + `base64`.
- Таймаут `SshSession::connect(..., timeout_ms)`: реализован через `tokio::time` (включена feature `tokio/time`).
- Поток событий `ssh-core`: при `subscribe_events()` новый подписчик получает snapshot текущего состояния (`Status{state}`), и если состояние `HostKeyPrompt`, то также текущий `HostKeyPrompt{fingerprint, reason}` (чтобы не терять prompt при поздней подписке).
- Rust: `vault` (E2EE, KDF, cipher suite, версия форматов)
- Rust: `sync-client` (oplog, pull/push, курсоры, конфликты)

### 3.2 Desktop

- Tauri 2 + React (TypeScript)
- Terminal UI: xterm.js
- Secure storage: OS Keychain (Windows DPAPI / macOS Keychain / Linux Secret Service)

### 3.3 Mobile

- Flutter + FFI bindings к Rust Core
- Secure storage: iOS Keychain / Android Keystore
- Terminal: Flutter terminal widget (MVP: корректная интерактивность для ANSI/VT100; v1: улучшение совместимости)

### 3.4 Web

- Next.js (React/TS) + xterm.js
- WebSSH Gateway: Rust (WSS ↔ SSH bridge), протокол: `gateway-protocol.md`
- Web vault: E2EE blobs в IndexedDB (ключ — master password; Passkeys как улучшение в v1+)

### 3.5 Backend Sync

- Rust (Axum) API + PostgreSQL
- На сервере: только encrypted blobs + метаданные версий/устройств/курсоров
- Rate limit: встроенно/через reverse proxy; Redis опционально

### 3.6 Структура репозитория (монорепо)

- `core/` — Rust Core SDK (ssh-core/vault/sync-client).
- `apps/` — приложения (desktop/web/mobile).
- `server/` — серверные компоненты (sync-api/webssh-gateway).
- `packages/` — общие пакеты (в т.ч. TS/shared UI/types при необходимости).
- `docs/` — управленческая и проектная документация.

### 3.7 Git / окончания строк (Windows)

- Окончания строк фиксируем через `.gitattributes` (в т.ч. `*.md`, `*.rs`, `*.toml`, `*.yml|*.yaml`, `Cargo.lock`) — в git всегда `LF`.
- Для читабельного `git diff` в Windows-консоли (без кракозябр Unicode) перед просмотром диффа переключать вывод в UTF-8 (`chcp 65001`; при необходимости также задать UTF-8 output encoding в PowerShell).
- Интеграционные контейнерные тесты (OpenSSH): локально Podman, в CI Docker (см. `docs/decision-log.md` D-005).
- Интеграционные тесты `ssh-core` (OpenSSH контейнер) включаются через `SSH_IT_ENABLE=1`.
- Рантайм контейнеров для тестов: `SSH_IT_RUNTIME=podman|docker` (по умолчанию: Podman если доступен, иначе Docker).
- Образ OpenSSH для тестов: `SSH_IT_IMAGE` (по умолчанию: `lscr.io/linuxserver/openssh-server:latest`).
- `cargo audit`: конфиг в `core/.cargo/audit.toml` (временно игнорируем `RUSTSEC-2023-0071`, т.к. для `rsa` нет fixed upgrade).

## 4. Релизные границы (очень важно)

### 4.1 MVP (обязательный минимум)

## Web MVP

- Каталог хостов (из синка) + терминал через gateway.
- В Web по умолчанию НЕ хранить приватные ключи персистентно.
- Аутентификация в сессию: password или “temporary key upload” (ключ только в RAM gateway до конца сессии).
- Без SFTP, без forwarding в MVP.

## Mobile MVP

- Каталог хостов + терминал + синк (hosts/snippets/ui_settings).
- Без SFTP/forwarding в MVP.

### 4.2 v1 (после MVP)

- SFTP manager (desktop полноценно, mobile/web упрощенно).
- Forwarding: local + dynamic SOCKS (desktop), ограниченно web, mobile по возможности.
- ProxyJump (1 hop) гарантированно; цепочки (N hops) опционально.
- Расширенный импорт `~/.ssh/config` (Include/Match/Host patterns по мере необходимости).
- Конфликты: улучшенный auto-merge по полям + UI для ручного merge.

## 5. Web Limitations (Explicit)

- Нет agent forwarding в MVP.
- Нет персистентных приватных ключей.
- Нет фоновых “вечных” сессий и background reconnect.
- Все SSH-сессии в Web идут только через WebSSH Gateway по WSS.

## 6. Функциональные требования (FR)

Нотация: FR-###, приоритет: MUST/SHOULD/COULD.

### 6.1 Каталог хостов

- FR-001 (MUST) CRUD Host (создать/редактировать/удалить с tombstone).
- FR-002 (MUST) Поиск по title/hostname/username/tags.
- FR-003 (MUST) Теги, избранное, сортировки.
- FR-004 (MUST) Параметры подключения: host/port/user.
- FR-005 (MUST) Auth: password и key.
- FR-006 (SHOULD) Профили/шаблоны (prod/stage/dev) — v1.
- FR-007 (SHOULD) Расширенные SSH options (ciphers/KEX/MAC) — v1.

### 6.2 Ключи и секреты

- FR-020 (MUST) Импорт ключей: OpenSSH (Ed25519), хранение приватного ключа в secure storage.
- FR-021 (SHOULD) Генерация Ed25519 (desktop).
- FR-022 (COULD) RSA/PEM/PPK импорт/конвертация (desktop) — v1.
- FR-023 (MUST) Passphrase поддержка (если ключ зашифрован).
- FR-024 (MUST) “Секреты не логируются” (password, key bytes, decrypted blobs).

## Политика синка ключей

- FR-025 (MUST) По умолчанию приватные ключи НЕ синхронизируются.
- FR-026 (COULD) Опция “Sync private keys inside E2EE vault” (v1/v2) — только при явном включении и повторной аутентификации (master password).

### 6.3 Терминал/SSH-сессии

- FR-040 (MUST) Поддержка интерактивного PTY: stdin/stdout, resize, exit status.
- FR-041 (MUST) UTF-8 корректно.
- FR-042 (MUST) Clipboard: copy/paste, bracketed paste (desktop/web).
- FR-043 (SHOULD) История подключений, quick reconnect — v1.
- FR-044 (COULD) Запись сессии без секретов (desktop) — v1.

### 6.4 Host key verification (anti-MITM)

- FR-060 (MUST) Режимы: strict / accept-new / ask.
- FR-061 (MUST) При смене fingerprint: блокировать (strict) или требовать явного подтверждения.
- FR-062 (MUST) Хранить known_hosts (в vault), поддержка pinned.

### 6.5 SFTP/SCP

- FR-080 (MUST v1) SFTP: list/get/put/mkdir/rm/rename/chmod.
- FR-081 (SHOULD v1) Resume/download progress.
- FR-082 (COULD) SCP совместимость — v2/опционально.

### 6.6 Forwarding/Tunnels

- FR-100 (MUST v1 desktop) Local forwarding.
- FR-101 (SHOULD v1 desktop) Dynamic SOCKS5.
- FR-102 (COULD) Remote forwarding — v1+ по мере ограничений.
- FR-103 (SHOULD v1) UI активных туннелей и автозапуск.

### 6.7 Импорт/экспорт

- FR-120 (MUST MVP) Импорт минимального набора `~/.ssh/config`:
  Host, HostName, User, Port, IdentityFile, ProxyJump (1 hop).
- FR-121 (SHOULD v1) Поддержка patterns и LocalForward/RemoteForward.
- FR-122 (COULD) Include/Match — v1/v2.
- FR-123 (MUST) Экспорт в JSON (backup) + encrypted backup.

### 6.8 Синхронизация (E2EE)

- FR-140 (MUST) E2EE по умолчанию для синхронизируемых данных (hosts/snippets/ui_settings/known_hosts).
- FR-141 (MUST) Сервер хранит только encrypted blobs и метаданные (не расшифровывает).
- FR-142 (MUST) Синк инкрементальный (pull/push по курсорам).
- FR-143 (MUST) Tombstones для удаления.
- FR-144 (MVP MUST) Конфликты: auto-merge по полям где возможно + fallback LWW; ручной merge UI — v1.

### 6.9 WebSSH Gateway

- FR-160 (MUST) WSS соединение браузера к gateway.
- FR-161 (MUST) Gateway устанавливает SSH и стримит I/O.
- FR-162 (MUST) Лимиты: длительность/байты/конкурентность на пользователя/девайс.
- FR-163 (MUST) В Web MVP ключи — только временно в RAM gateway (без персистентности).
- FR-164 (MUST) Gateway аутентифицируется JWT от sync-api и привязывает сессию к deviceId.
- FR-165 (MUST) Gateway не принимает `stdin` до состояния `READY` и даёт явные `error` коды при нарушении.

## 7. Нефункциональные требования (NFR)

- NFR-001 (MUST) Секреты не попадают в логи, дампы (best effort), telemetry.
- NFR-002 (SHOULD) Desktop старт < 2s (цель), Web TTI < 2.5s (цель).
- NFR-003 (MUST) Надёжность: keepalive, настраиваемый reconnect (desktop/web).
- NFR-004 (MUST) Хранилища: secure storage для приватных ключей и паролей.
- NFR-005 (MUST) Vault format versioning и миграции.
- NFR-006 (SHOULD) Accessibility: масштабирование, контраст.
- NFR-007 (MUST) Rate limiting на sync-api и gateway.

## 8. FR Traceability (MVP)

| FR      | Desktop | Mobile | Web | Gateway | Core |
|--------|--------|--------|-----|--------|------|
| FR-001 | ✅     | ✅     | ✅  | ❌     | ✅   |
| FR-040 | ✅     | ✅     | ✅  | ✅     | ✅   |
| FR-060 | ✅     | ✅     | ✅  | ✅     | ✅   |
| FR-120 | ✅     | ❌     | ❌  | ❌     | ✅   |
| FR-163 | ❌     | ❌     | ✅  | ✅     | ❌   |
| FR-165 | ❌     | ❌     | ✅  | ✅     | ✅   |

## 9. Данные и форматы

### 9.1 Логическая модель (минимум)

Host:

- id UUID
- title, tags[], favorite
- hostname, port, username
- auth: {type: password|key|agent, keyId?}
- proxy: {type: none|jump, jumpHostId?}
- options: {keepAliveSec, timeoutSec, compression, strictHostKeyMode}
- createdAt, updatedAt
- version (int), deleted (bool tombstone)

Key:

- id UUID
- name, type (ed25519|rsa)
- publicKey, fingerprint
- privateRef (handle в secure storage)
- createdAt

Snippet:

- id, title, body, tags[], scope, createdAt, updatedAt, version, deleted

KnownHost:

- id, hostPattern, keyType, publicKey, fingerprint
- pinned bool, addedAt, lastSeenAt

### 9.2 Vault криптосхема (минимум)

- KDF: Argon2id (параметры версионируются: kdfParamsVersion)
- Cipher suite: AES-256-GCM или ChaCha20-Poly1305 (cipherSuiteVersion)
- Структура blob:
  - header: {vaultVersion, kdfParamsVersion, cipherSuiteVersion, createdAt}
  - payload: encrypted(CBOR/JSON)
- Миграции: read old → write new; отказ при несовместимости с понятной ошибкой.

## 10. Sync протокол (высокий уровень)

Коллекции: hosts, snippets, known_hosts, ui_settings.

Сервер хранит:

- encrypted blobs
- метаданные: entityId, entityType, revision, deviceId, serverSeq, serverTime, tombstone

Курсоры:

- cursor = serverSeq per user (монотонный, серверный)
- клиент запоминает lastServerSeq и делает pull “после”

Конфликтность:

- Auto-merge по полям (если изменения не пересекаются)
- Иначе fallback LWW, где “последнее” определяется serverTime/serverSeq, а не часами устройства.
- Ручной merge UI — v1.

Эндпоинты:

- POST /auth/register|login|refresh
- GET /sync/state
- POST /sync/push (batch)
- POST /sync/pull (afterSeq)
- POST /sync/resolve (v1+)

## 11. Definition of Done (DoD)

- тесты (unit + интеграционные для core; smoke e2e для apps);
- отсутствие секретов в логах/трассировках;
- миграции данных и версионирование форматов;
- документация обновлена;
- сборки проходят CI на целевых платформах.

## 12. Acceptance Criteria (примеры)

- AC-001 Импорт `~/.ssh/config` создаёт Host записи (HostName/User/Port/IdentityFile/ProxyJump 1 hop) и позволяет подключиться.
- AC-002 При смене host key в strict режиме подключение блокируется до подтверждения.
- AC-003 Изменения хостов синхронизируются между двумя устройствами через E2EE (сервер не видит содержимого).
- AC-004 Web терминал работает через gateway, интерактивен, применяются лимиты сессий.
- AC-005 В Web MVP приватный ключ не сохраняется персистентно и не попадает в логи.

## 13. Документация и агентские логи

- Основная управленческая документация хранится в `docs/`.
- `docs/agent-orchestration.md` — дерево этапов, роли, критерии завершения и журнал этапов.
- `docs/decision-log.md` — журнал ключевых решений (append-only).
- `docs/architecture.md` — обзор архитектуры и границы модулей.
- `docs/traceability.md` — трассируемость требований (FR/NFR/SEC/Threat/AC) к этапам/модулям/проверкам.
- `docs/quality-gates.md` — merge/release gates (качество и безопасность).
- GitHub Actions: `.github/workflows/ci.yml` — минимальные merge gates и secret scan.
- `docs/agents/*.md` — логи субагентов; TODO ведётся как **append-only** история снапшотов (старые записи не переписываются).
