# AGENT-QA — QA / Тест-инженер
Версия: 0.1  
Дата: 2025-12-17  
Статус: Active (Stage 0)

## 0. Мандат
- Определить тест-стратегию по модулям и минимальный набор обязательных suite’ов для MVP.
- Гарантировать проверяемость требований (особенно security invariants).

## 1. TODO History (append-only)
### Snapshot 2025-12-17
| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-QA-001 | TODO | HIGH | Сформировать список обязательных интеграционных тестов MVP (ssh-core, gateway, sync) | SRS.md §14; IMPLEMENTATION_PLAN.md §3 |
| M0-QA-002 | TODO | HIGH | Определить тесты на утечки секретов (логи/трейсы/краши) и формат fixtures | SRS.md §11/§14; THREAT_MODEL.md T-003 |
| M0-QA-003 | TODO | MEDIUM | Зафиксировать критерии DoD/no-go по этапам в виде чек-листов | IMPLEMENTATION_PLAN.md §3/§5 |

### Snapshot 2025-12-17 (updates)
| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-QA-001 | IN_PROGRESS | HIGH | Сформировать список обязательных интеграционных тестов MVP (ssh-core, gateway, sync) | `docs/traceability.md`; SRS.md §14 |
| M0-QA-002 | IN_PROGRESS | HIGH | Определить тесты на утечки секретов (логи/трейсы/краши) и формат fixtures | `docs/quality-gates.md`; SRS.md §11/§14 |
| M0-QA-003 | DONE | MEDIUM | Зафиксировать критерии DoD/no-go по этапам в виде чек-листов | `docs/quality-gates.md`; IMPLEMENTATION_PLAN.md §5.2 |
| M1-QA-001 | TODO | HIGH | Уточнить тест-кейсы для `core/ssh-core`: PTY+resize, hostkey policies, NOT_READY поведение | SRS.md §7.1.3; §14.1; FR-040; FR-060..062 |
| M1-QA-002 | TODO | HIGH | Определить критерии секрет-скана/fixtures для `ssh-core` (ключи/пароли/terminal I/O) | SEC-006; THREAT_MODEL.md T-003 |
