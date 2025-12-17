# Decision Log — SSH Client + WebSSH Gateway

Версия: 0.4  
Дата: 2025-12-17  
Статус: Active

## Правила ведения

- Каждое решение имеет стабильный ID вида `D-###`.
- Записи добавляются *append-only*; ранее принятые решения не переписываются (только новые записи/уточнения).
- Каждая запись должна ссылаться на исходные требования (PRD/SRS/THREAT_MODEL) и/или этап (см. `docs/agent-orchestration.md`).

---

## D-001 — Выбор SSH реализации для `core/ssh-core`

- Дата: 2025-12-17
- Статус: OPEN
- Контекст: PRD.md §10 (Open questions #1), IMPLEMENTATION_PLAN.md §2.2
- Варианты:
  - `russh`
  - `libssh2` (+ биндинги)
- Решение: TBD
- Последствия: TBD

- Update 2025-12-17:
  - Статус: DECIDED
  - Решение: `russh`
  - Последствия: `core/ssh-core` реализуем на базе `russh` (Tokio async); `libssh2` не используем в M1.

## D-002 — Формат payload в Vault (CBOR vs JSON)

- Дата: 2025-12-17
- Статус: OPEN
- Контекст: SRS.md §7.2.3, IMPLEMENTATION_PLAN.md §7
- Варианты:
  - CBOR
  - JSON (только при явном versioning)
- Решение: TBD
- Последствия: TBD

- Update 2025-12-17:
  - Статус: DECIDED
  - Решение: JSON (явное versioning) для payload в Vault (MVP).
  - Последствия: Payload в vault сериализуется как JSON; CBOR может быть добавлен позже при сохранении совместимости.

## D-003 — Транспорт JWT для Web (HttpOnly cookie vs Authorization header)

- Дата: 2025-12-17
- Статус: OPEN
- Контекст: PRD.md §10 (Open questions #3), SRS.md §6.4/§9.5, THREAT_MODEL.md T-005
- Варианты:
  - HttpOnly cookie
  - Authorization header (Bearer)
- Решение: TBD
- Последствия: TBD

- Update 2025-12-17:
  - Статус: DECIDED
  - Решение: HttpOnly cookie для Web (access token недоступен JS).
  - Последствия: `sync-api` и `webssh-gateway` должны поддержать cookie-based auth; в Web избегать JS-readable хранилищ для JWT.

## D-004 — Синхронизация `known_hosts` в MVP

- Дата: 2025-12-17
- Статус: OPEN
- Контекст: PRD.md §10 (Open questions #2), SRS.md §5.1.3/§7.3.1, IMPLEMENTATION_PLAN.md §7
- Варианты:
  - Не синхронизировать в MVP (локально), рассмотреть v1
  - Синхронизировать в MVP (в составе E2EE vault)
- Решение: TBD
- Последствия: TBD

- Update 2025-12-17:
  - Статус: DECIDED
  - Решение: синхронизировать `known_hosts` в MVP как часть E2EE vault.
  - Последствия: коллекция `known_hosts` включается в E2EE sync (ciphertext) и обрабатывается общими правилами конфликтов (auto-merge/LWW по serverSeq).

## D-005 — Container runtime для интеграционных тестов (Podman vs Docker)

- Дата: 2025-12-17
- Статус: DECIDED
- Контекст: SRS.md §14.1, IMPLEMENTATION_PLAN.md M1
- Варианты:
  - Docker
  - Podman
- Решение: Podman.
- Последствия: интеграционные тесты с OpenSSH выполняются в контейнерах через Podman; документация и скрипты должны использовать Podman (docker-совместимый CLI).

- Update 2025-12-17:
  - Уточнение: гибридный runtime.
  - Локально: Podman.
  - CI (GitHub Actions): Docker (для совместимости с `ubuntu-latest` без установки Podman).
