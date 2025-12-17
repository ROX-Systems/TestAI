# AGENT-MANAGER — Управляющий агент

Версия: 0.1  
Дата: 2025-12-17  
Статус: Active

## 0. Мандат

- Координировать этапы, роли и артефакты проекта.
- Подтверждать завершение этапов и фиксировать результаты в `docs/agent-orchestration.md`.
- Следить за трассируемостью решений через `docs/decision-log.md`.

## 1. TODO History (append-only)

### Snapshot 2025-12-17

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-MGR-001 | IN_PROGRESS | HIGH | Зафиксировать дерево этапов/ролей и правила работы | IMPLEMENTATION_PLAN.md M0; `docs/agent-orchestration.md` |
| M0-MGR-002 | TODO | HIGH | Создать логи активных субагентов Этапа 0 в `docs/agents/` | User request; `docs/agent-orchestration.md` |
| M0-MGR-003 | TODO | HIGH | Обновить `project-specs.md` (конвенции по `docs/*` и агентским логам) | User rule; `project-specs.md` |
| M0-MGR-004 | TODO | MEDIUM | Закрыть Этап 0 и открыть Этап 1 (M1) в `docs/agent-orchestration.md` | IMPLEMENTATION_PLAN.md M1 |

### Snapshot 2025-12-17 (Stage 0 completion / Stage 1 kickoff)

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-MGR-001 | DONE | HIGH | Зафиксировать дерево этапов/ролей и правила работы | `docs/agent-orchestration.md` |
| M0-MGR-002 | DONE | HIGH | Создать логи активных субагентов Этапа 0 в `docs/agents/` | `docs/agents/*` |
| M0-MGR-003 | DONE | HIGH | Обновить `project-specs.md` (конвенции по `docs/*` и агентским логам) | `project-specs.md` §13 |
| M0-MGR-004 | DONE | MEDIUM | Закрыть Этап 0 и открыть Этап 1 (M1) в `docs/agent-orchestration.md` | `docs/agent-orchestration.md` §6 |
| M1-MGR-001 | IN_PROGRESS | HIGH | Скоординировать старт Этапа 1 (M1) и контроль решения D-001 | IMPLEMENTATION_PLAN.md M1; `docs/decision-log.md` D-001 |

### Snapshot 2025-12-17 (M0 repo/CI)

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-MGR-006 | IN_PROGRESS | HIGH | Закрыть M0 (repo/CI) после настройки required checks / branch protection в GitHub | IMPLEMENTATION_PLAN.md M0 Exit criteria |
| M0-MGR-005 | DONE | HIGH | Добавить GitHub Actions workflow и зафиксировать его в документации | `.github/workflows/ci.yml`; `project-specs.md` §13 |
| M1-MGR-001 | TODO | HIGH | Скоординировать старт Этапа 1 (M1) и контроль решения D-001 | `docs/decision-log.md` D-001 |

### Snapshot 2025-12-17 (M0 repo/CI completion)

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-MGR-006 | DONE | HIGH | Закрыть M0 (repo/CI) после настройки required checks / branch protection в GitHub | IMPLEMENTATION_PLAN.md M0 Exit criteria |
| M1-MGR-001 | IN_PROGRESS | HIGH | Скоординировать старт Этапа 1 (M1) и контроль решения D-001 | IMPLEMENTATION_PLAN.md M1; `docs/decision-log.md` D-001 |

### Snapshot 2025-12-17 (D-001 decided + M1 scaffold)

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M1-MGR-001 | DONE | HIGH | Скоординировать старт Этапа 1 (M1) и контроль решения D-001 | `docs/decision-log.md` D-001; `core/ssh-core` |
