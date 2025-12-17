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

## D-002 — Формат payload в Vault (CBOR vs JSON)
- Дата: 2025-12-17
- Статус: OPEN
- Контекст: SRS.md §7.2.3, IMPLEMENTATION_PLAN.md §7
- Варианты:
  - CBOR
  - JSON (только при явном versioning)
- Решение: TBD
- Последствия: TBD

## D-003 — Транспорт JWT для Web (HttpOnly cookie vs Authorization header)
- Дата: 2025-12-17
- Статус: OPEN
- Контекст: PRD.md §10 (Open questions #3), SRS.md §6.4/§9.5, THREAT_MODEL.md T-005
- Варианты:
  - HttpOnly cookie
  - Authorization header (Bearer)
- Решение: TBD
- Последствия: TBD

## D-004 — Синхронизация `known_hosts` в MVP
- Дата: 2025-12-17
- Статус: OPEN
- Контекст: PRD.md §10 (Open questions #2), SRS.md §5.1.3/§7.3.1, IMPLEMENTATION_PLAN.md §7
- Варианты:
  - Не синхронизировать в MVP (локально), рассмотреть v1
  - Синхронизировать в MVP (в составе E2EE vault)
- Решение: TBD
- Последствия: TBD
