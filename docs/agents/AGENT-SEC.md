# AGENT-SEC — Security инженер

Версия: 0.1  
Дата: 2025-12-17  
Статус: Active

## 0. Мандат

- Перевод threat model в конкретные контрмеры и тесты.
- Контроль Security Invariants (SEC-001..SEC-008) в реализации и CI.
- Критические решения (JWT транспорт, known_hosts sync, key handling) фиксировать в `docs/decision-log.md`.

## 1. TODO History (append-only)

### Snapshot 2025-12-17

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-SEC-001 | TODO | HIGH | Уточнить и формализовать «no-go» условия и способы проверки (секреты в логах, stdin до READY, host key policy) | IMPLEMENTATION_PLAN.md §5.2; SRS.md §9.4/§11 |
| M0-SEC-002 | TODO | HIGH | Подготовить решение по Web JWT стратегии (cookie vs header) (D-003) | PRD.md §10; THREAT_MODEL.md T-005; SRS.md §6.4/§9.5 |
| M0-SEC-003 | TODO | MEDIUM | Подготовить решение по `known_hosts` sync (D-004) и последствия | PRD.md §10; SRS.md §5.1.3/§7.3 |

### Snapshot 2025-12-17 (Stage 0 фиксация + Stage 1 kickoff)

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-SEC-001 | DONE | HIGH | Уточнить и формализовать «no-go» условия и способы проверки (секреты в логах, stdin до READY, host key policy) | `docs/quality-gates.md`; IMPLEMENTATION_PLAN.md §5.2 |
| M0-SEC-002 | TODO | HIGH | Подготовить решение по Web JWT стратегии (cookie vs header) (D-003) | `docs/decision-log.md` D-003 |
| M0-SEC-003 | TODO | MEDIUM | Подготовить решение по `known_hosts` sync (D-004) и последствия | `docs/decision-log.md` D-004 |
| M1-SEC-001 | IN_PROGRESS | HIGH | Определить security checklist для `core/ssh-core` (host key policy, redaction, state machine) | SRS.md §6.2/§7.1.3; SEC-005/006/007 |
