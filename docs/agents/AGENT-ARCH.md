# AGENT-ARCH — Архитектор / Tech Lead
Версия: 0.1  
Дата: 2025-12-17  
Статус: Active (Stage 0)

## 0. Мандат
- Декомпозиция системы на модули и границы ответственности.
- Проработка критического пути MVP (M1→M2→M3→M5).
- Архитектурные решения фиксировать через `docs/decision-log.md`.

## 1. TODO History (append-only)
### Snapshot 2025-12-17
| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-ARCH-001 | TODO | HIGH | Зафиксировать целевую структуру монорепо (core/apps/server/packages/docs) как конвенцию | IMPLEMENTATION_PLAN.md M0 |
| M0-ARCH-002 | TODO | HIGH | Подготовить ADR по выбору SSH-стека для `core/ssh-core` (варианты из D-001) | PRD.md §10; IMPLEMENTATION_PLAN.md §2.2; SRS.md §7.1 |
| M0-ARCH-003 | TODO | MEDIUM | Определить границы ответственности между `sync-api` и `webssh-gateway` (auth/binding/limits) | SRS.md §8/§9; SEC-008 |

### Snapshot 2025-12-17 (repo conventions)
| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-ARCH-001 | DONE | HIGH | Зафиксировать целевую структуру монорепо (core/apps/server/packages/docs) как конвенцию | `project-specs.md` §3.6 |
| M0-ARCH-002 | TODO | HIGH | Подготовить ADR по выбору SSH-стека для `core/ssh-core` (варианты из D-001) | `docs/decision-log.md` D-001 |
| M0-ARCH-003 | TODO | MEDIUM | Определить границы ответственности между `sync-api` и `webssh-gateway` (auth/binding/limits) | SRS.md §8/§9; SEC-008 |
