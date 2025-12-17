# Traceability — SSH Client + WebSSH Gateway
Версия: 0.1  
Дата: 2025-12-17  
Статус: Active

## 0. Назначение
Этот документ связывает требования (FR/NFR/SEC/Threat/AC) с модулями и этапами реализации (M0–M7).

Источники:
- `project-specs.md`
- `PRD.md`
- `SRS.md`
- `THREAT_MODEL.md`
- `IMPLEMENTATION_PLAN.md`

## 1. FR → Milestones / Modules (MVP)
| Требование / группа | Milestone(ы) | Основные модули |
|---|---|---|
| Host catalog (FR-001..FR-004) | M2, M3.5, M4/M6 | `core/vault`, `core/sync-client`, `apps/*` |
| Auth methods (FR-005) | M1, M2, M4/M6/M5 | `core/ssh-core`, `core/vault`, `apps/*` |
| Keys/secrets (FR-020, FR-023..FR-026) | M2, M4/M6 | `core/vault`, `apps/desktop`, `apps/mobile`, OS secure storage |
| Terminal/PTy (FR-040..FR-042) | M1, M4/M5/M6 | `core/ssh-core`, `apps/* terminal` |
| Host key verification (FR-060..FR-062) | M1, M4/M5/M6 | `core/ssh-core`, `core/vault` |
| SSH config import minimal (FR-120) | M1, M4 | `core/ssh-core` (parser), `apps/desktop` |
| Sync E2EE (FR-140..FR-144) | M2, M3, M3.5 | `core/vault`, `core/sync-client`, `server/sync-api` |
| WebSSH Gateway (FR-160..FR-165) | M5 | `server/webssh-gateway`, `apps/web` |

## 2. NFR/SEC → проверки (gates/tests)
| ID | Требование | Где реализуется | Как проверяется (минимум) |
|---|---|---|---|
| NFR-001 / SEC-006 | Секреты не попадают в логи/телеметрию/краши | все модули | secret-scan gate + redaction/unit tests + аудит логирования |
| NFR-003 | Надёжность: keepalive/reconnect | `core/ssh-core`, `apps/*` | интеграционные тесты + e2e smoke (позже) |
| NFR-004 | Secure storage для ключей/паролей | `apps/desktop`, `apps/mobile` | интеграционные/смоук тесты + ручная валидация |
| NFR-005 | Версии форматов + миграции | `core/vault` | migration tests, cross-platform compat tests |
| NFR-007 | Rate limiting на sync-api/gateway | `server/sync-api`, `server/webssh-gateway` | контракт/интеграционные тесты + 429 с retryAfterMs |
| SEC-003 | Web: ключ только в RAM gateway | `server/webssh-gateway` | тесты “wipe on exit/timeout/error”; запрет логирования |
| SEC-005 | Host key policy enforced | `core/ssh-core`, `apps/*` | интеграционные тесты: strict block, accept-new accept-once, ask prompt |
| SEC-007 | stdin/resize только в READY | `core/ssh-core`, `server/webssh-gateway` | state machine tests; error `NOT_READY` |
| SEC-008 | JWT exp + device binding | `server/sync-api`, `server/webssh-gateway` | тесты валидации токена и привязки userId/deviceId |

## 3. Threat → Controls (приоритетные)
| Threat ID | Риск | Контрмеры | Проверка |
|---|---|---|---|
| T-003 | утечки секретов | SEC-006, NFR-001 | secret-scan gate, тестовые ключи/энтропия |
| T-005 | XSS в web | CSP, token strategy (D-003), sanitization | security review + web tests (baseline) |
| T-002 | MITM/host key substitution | SEC-005, FR-060/061/062 | host key tests + UX prompt/strict behavior |

## 4. AC → Milestones (MVP)
| AC | Milestone(ы) |
|---|---|
| AC-001 | M1 + M4 |
| AC-002 | M1 |
| AC-003 | M2 + M3 + M3.5 |
| AC-004 | M5 |
| AC-005 | M0..M6 (как gate), критично для M1/M5 |
