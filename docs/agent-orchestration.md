# Agent Orchestration — SSH Client + WebSSH Gateway
Версия: 0.1  
Дата: 2025-12-17  
Статус: Active

## 0. Источники истины
- `project-specs.md` — главный документ по целям/границам/стеку/FR/NFR.
- `PRD.md`, `SRS.md`, `THREAT_MODEL.md`, `IMPLEMENTATION_PLAN.md` — нормативные требования и план.
- `docs/decision-log.md` — журнал решений (append-only).

## 1. Конденсат целей, функционала и ограничений (MVP)
### 1.1 Цели
- Единый каталог хостов/настроек и E2EE-синхронизация (сервер хранит только ciphertext).
- Надёжный интерактивный терминал (PTY, resize, UTF-8, clipboard).
- Security-by-default: host key verification (strict/accept-new/ask), отсутствие секретов в логах.
- Web-доступ только через WebSSH Gateway (WSS ↔ SSH) без персистентного хранения приватных ключей в браузере.

### 1.2 Неподдерживаемое в MVP
- SFTP/forwarding/agent-forwarding (переходит в v1 или позже).
- Полная совместимость с OpenSSH config (в MVP — минимальный импорт).
- RBAC/workspaces.

### 1.3 Security invariants (MUST)
- Sync API не имеет доступа к plaintext данным vault.
- Приватные ключи не синхронизируются по умолчанию.
- Web MVP: ключ только в RAM gateway и должен быть стёрт на всех путях завершения.
- Gateway не пишет stdin/stdout/stderr в persistent storage и не логирует секреты.

## 2. Реестр субагентов (агентские объекты)
Все субагенты ведут собственные файлы в `docs/agents/`.

| Agent ID | Роль | Основной фокус |
|---|---|---|
| AGENT-MANAGER | Управляющий агент | Координация, этапы, подтверждения, общая документация |
| AGENT-REQ | Аналитик требований | Трассируемость FR/NFR/SEC → модули/тесты, контроль scope |
| AGENT-ARCH | Архитектор/Tech Lead | Архитектура, границы модулей, критический путь, интеграции |
| AGENT-DEVOPS | DevOps/CI | CI/CD, quality gates, секрет-скан, зависимостные аудиты |
| AGENT-SEC | Security инженер | Threat-model → controls/tests, web token strategy, hardening |
| AGENT-QA | QA/Тест-инженер | Тест-стратегия, интеграционные тесты, DoD/no-go условия |
| AGENT-RUST-CORE | Rust Core dev | `core/ssh-core`, `core/vault`, `core/sync-client` |
| AGENT-BE | Backend dev | `server/sync-api`, Postgres schema/migrations, rate limiting |
| AGENT-GW | Gateway dev | `server/webssh-gateway`, WSS протокол/квоты/идемпотентность |
| AGENT-FE-DESKTOP | Desktop dev | Tauri+React, xterm.js, мост к core, UX flows |
| AGENT-FE-WEB | Web dev | Next.js+React, xterm.js, WSS клиент, web vault UX |
| AGENT-MOBILE | Mobile dev | Flutter UI, Rust FFI, ограничения background |

## 3. Правила работы (трассируемость и история)
### 3.1 TODO-листы и история
- Каждый агент ведёт **TODO History** как набор *snapshot* секций (append-only).
- Обновление статусов делается через добавление нового snapshot, а не переписывание старого.
- Каждая задача должна иметь ссылку на источник (FR/NFR/SEC/AC, SRS раздел, Threat ID или Milestone).

### 3.2 Завершение этапа
Этап считается завершённым, когда:
- все задачи этапа в активных агентских TODO помечены как `DONE` (в новом snapshot);
- управляющий агент добавил запись “Stage Completion” в этот документ;
- обновлён `docs/decision-log.md` для принятых решений (если применимо).

## 4. Дерево этапов (MVP → v1)
Нумерация согласована с `IMPLEMENTATION_PLAN.md`.

1. **Этап 0 (M0) — Repo/CI/Docs/Decisions**
2. **Этап 1 (M1) — `core/ssh-core` (PTY + host key policy)**
3. **Этап 2 (M2) — `core/vault` + local stores + KeyStore**
4. **Этап 3 (M3) — `server/sync-api`**
5. **Этап 3.5 (M3.5) — `core/sync-client`**
6. **Этап 4 (M4) — Desktop MVP (Tauri)**
7. **Этап 5 (M5) — WebSSH Gateway + Web MVP**
8. **Этап 6 (M6) — Mobile MVP (Flutter)**
9. **Этап 7 (M7) — v1 features**

## 5. Этапы (цели, роли, агенты, задачи, критерии)
### 5.1 Этап 0 (M0) — Repo/CI/Docs/Decisions
- Цель: подготовить репозиторий и процессы, чтобы дальнейшая разработка шла с quality/security gates и трассируемостью.
- Требуемые роли:
  - AGENT-ARCH, AGENT-DEVOPS, AGENT-SEC, AGENT-QA, AGENT-REQ
- Назначенные субагенты:
  - AGENT-MANAGER (координация)
  - AGENT-ARCH (структура модулей/репозитория)
  - AGENT-DEVOPS (CI gates)
  - AGENT-SEC (security gates)
  - AGENT-QA (test strategy)
  - AGENT-REQ (traceability)
- Задачи этапа:
  - Зафиксировать структуру документации и агентских логов.
  - Создать/поддерживать decision log.
  - Определить минимальный набор quality gates (fmt/clippy/test/audit, eslint/typecheck/test/audit, flutter analyze/test, secret scan).
- Критерии завершения:
  - Репозиторий готов к работе с PR-мердж-гейтами (как минимум описано в документации).
  - Есть decision log и правила его ведения.

### 5.2 Этап 1 (M1) — `core/ssh-core`
- Цель: интерактивный SSH (PTY) + host key verification per policy, + интеграционные тесты.
- Требуемые роли: AGENT-RUST-CORE, AGENT-SEC, AGENT-QA.
- Назначенные субагенты: AGENT-RUST-CORE, AGENT-SEC, AGENT-QA.
- Критерии завершения (кратко): PTY+resize в docker tests; strict блокирует changed host key; secret leakage tests проходят.

### 5.3 Этап 2 (M2) — `core/vault` + local stores + KeyStore
- Цель: E2EE vault, версии/миграции, локальное хранение (Desktop/Mobile/Web) и secure storage интеграция.
- Требуемые роли: AGENT-RUST-CORE, AGENT-SEC, AGENT-QA, AGENT-FE-WEB (IndexedDB UX).
- Критерии завершения (кратко): Desktop encrypt ↔ Web decrypt; no plaintext persisted; базовые тесты secure storage.

### 5.4 Этап 3 (M3) — `server/sync-api`
- Цель: auth (JWT), device binding, sync state/push/pull, Postgres schema, rate limits и audit events.
- Требуемые роли: AGENT-BE, AGENT-DEVOPS, AGENT-SEC, AGENT-QA.
- Критерии завершения (кратко): 2 девайса пуш/пулл ciphertext end-to-end; контракт (OpenAPI) и rate limiting тесты.

### 5.5 Этап 3.5 (M3.5) — `core/sync-client`
- Цель: oplog + детерминированный apply, конфликты (field auto-merge + LWW by serverSeq), tombstones.
- Требуемые роли: AGENT-RUST-CORE, AGENT-QA.
- Критерии завершения (кратко): конфликтные сценарии воспроизводимы и проходят; tombstones сходятся.

### 5.6 Этап 4 (M4) — Desktop MVP
- Цель: Tauri UI для hosts/keys/terminal + синк.
- Требуемые роли: AGENT-FE-DESKTOP, AGENT-RUST-CORE, AGENT-QA, (опционально дизайн).
- Критерии завершения (кратко): add host → connect → interactive terminal; синк хостов между профилями.

### 5.7 Этап 5 (M5) — WebSSH Gateway + Web MVP
- Цель: gateway WSS протокол (SRS §9), квоты, flow control, temp key lifecycle; web app terminal.
- Требуемые роли: AGENT-GW, AGENT-FE-WEB, AGENT-SEC, AGENT-QA, AGENT-BE (JWT/device binding).
- Критерии завершения (кратко): gateway error codes корректны; stdin до READY запрещён (тесты); temp key wiping на всех путях.

### 5.8 Этап 6 (M6) — Mobile MVP
- Цель: Flutter UI + Rust FFI, базовый интерактивный терминал и синк.
- Требуемые роли: AGENT-MOBILE, AGENT-RUST-CORE, AGENT-QA.
- Критерии завершения (кратко): input/output/resize работает; синк каталога работает.

### 5.9 Этап 7 (M7) — v1 features
- Цель: SFTP, forwarding, улучшенные конфликты, расширенный ssh_config import.
- Требуемые роли: все профильные.
- Критерии завершения: определяются по каждому подпакету (SFTP/forwarding/import).

## 6. Журнал этапов (append-only)
### Update 2025-12-17
- Этап 0: инициализирован decision log (`docs/decision-log.md`).

### Update 2025-12-17 (Stage 0 docs)
- Создан `docs/agent-orchestration.md` (дерево этапов/ролей/критериев).
- Созданы логи субагентов Этапа 0 в `docs/agents/`.
- Добавлены базовые документы: `docs/architecture.md`, `docs/traceability.md`, `docs/quality-gates.md`.
- Этап 0 (документация/координация) зафиксирован; переход к Этапу 1 (M1) выполняется после подтверждения готовности M0 (repo/CI).

### Update 2025-12-17 (Stage 0 repo/CI)
- Создан монорепо-скелет: `core/`, `apps/`, `server/`, `packages/`.
- Добавлен GitHub Actions workflow: `.github/workflows/ci.yml` (условные Rust/Node/Flutter gates + secret scan).
- M0 не закрыт до настройки required checks / branch protection в GitHub и появления первых реальных манифестов (Cargo.toml/package.json/pubspec.yaml).
