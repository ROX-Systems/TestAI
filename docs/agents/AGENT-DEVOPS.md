# AGENT-DEVOPS — DevOps/CI

Версия: 0.1  
Дата: 2025-12-17  
Статус: Active

## 0. Мандат

- Настроить CI/CD и quality gates, которые блокируют регрессии безопасности и качества.
- Обеспечить воспроизводимые проверки (fmt/clippy/test/audit, eslint/typecheck/test/audit, flutter analyze/test, secret scan).

## 1. TODO History (append-only)

### Snapshot 2025-12-17

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-CI-001 | TODO | HIGH | Зафиксировать список merge-gates и критерии failure (Rust/TS/Flutter/audit) | IMPLEMENTATION_PLAN.md §4 |
| M0-CI-002 | TODO | HIGH | Добавить secret-scan gate (ключи/энтропия) и правила redaction тестов | SRS.md §11; THREAT_MODEL.md T-003; IMPLEMENTATION_PLAN.md §4.1 |
| M0-CI-003 | TODO | MEDIUM | Определить политику dependency pinning/lockfiles и SBOM (если нужно) | THREAT_MODEL.md T-010; IMPLEMENTATION_PLAN.md M0 |

### Snapshot 2025-12-17 (updates)

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-CI-001 | DONE | HIGH | Зафиксировать список merge-gates и критерии failure (Rust/TS/Flutter/audit) | `docs/quality-gates.md` |
| M0-CI-002 | TODO | HIGH | Добавить secret-scan gate (ключи/энтропия) и правила redaction тестов | `docs/quality-gates.md`; SRS.md §11 |
| M0-CI-003 | TODO | MEDIUM | Определить политику dependency pinning/lockfiles и SBOM (если нужно) | THREAT_MODEL.md T-010; IMPLEMENTATION_PLAN.md M0 |

### Snapshot 2025-12-17 (GitHub Actions CI)

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-CI-001 | DONE | HIGH | Зафиксировать список merge-gates и критерии failure (Rust/TS/Flutter/audit) | `docs/quality-gates.md` |
| M0-CI-002 | DONE | HIGH | Добавить secret-scan gate (ключи/энтропия) и правила redaction тестов | `.github/workflows/ci.yml`; `docs/quality-gates.md` |
| M0-CI-003 | TODO | MEDIUM | Определить политику dependency pinning/lockfiles и SBOM (если нужно) | THREAT_MODEL.md T-010; IMPLEMENTATION_PLAN.md M0 |
| M0-CI-004 | TODO | HIGH | Настроить required checks/branch protection на GitHub (main) под имена job’ов workflow | IMPLEMENTATION_PLAN.md M0 Exit criteria |

### Snapshot 2025-12-17 (branch protection)

| ID | Статус | Приоритет | Задача | Трассировка |
|---|---|---|---|---|
| M0-CI-004 | DONE | HIGH | Настроить required checks/branch protection на GitHub (main) под имена job’ов workflow | IMPLEMENTATION_PLAN.md M0 Exit criteria |
| M0-CI-003 | TODO | MEDIUM | Определить политику dependency pinning/lockfiles и SBOM (если нужно) | THREAT_MODEL.md T-010; IMPLEMENTATION_PLAN.md M0 |
