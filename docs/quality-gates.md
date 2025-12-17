# Quality & Security Gates — SSH Client + WebSSH Gateway

Версия: 0.1  
Дата: 2025-12-17  
Статус: Active

## 0. Назначение

Этот документ фиксирует минимальные quality/security gates, которые должны быть включены как merge-gates и release-gates.

Реализация (GitHub Actions): `.github/workflows/ci.yml`.

Источники: `IMPLEMENTATION_PLAN.md` §4–§5, `SRS.md` §11/§14.

## 1. Merge gates (минимум)

### 1.1 Rust (core + server)

- `cargo fmt --check`
- `cargo clippy -D warnings`
- `cargo test`
- `cargo audit`

### 1.2 Web/TS

- `eslint`
- `tsc --noEmit`
- `test`
- `npm audit` (порог — policy-defined)

### 1.3 Flutter

- `flutter analyze`
- `flutter test`

## 2. Secret leakage gate (обязательный)

### 2.1 Запрещено

- пароли, приватные ключи, passphrase, decrypted vault blobs
- terminal I/O (stdin/stdout/stderr)

### 2.2 Минимальные проверки

- статический поиск по репозиторию и тестовым артефактам на известные тестовые приватные ключи
- high-entropy detection (policy)
- запрет попадания секретов в логи/трейсы/краши (best effort + тесты)

## 3. Dependency policy

- lockfiles обязательны
- зависимости должны быть закреплены (pinning) согласно инструментам экосистемы
- аудиты (`cargo audit`, `npm audit`) — merge-gate

## 4. Release “no-go” условия (MVP)

(Нельзя релизить при наличии любого пункта)

- секреты появляются в логах/трейсах/краш-артефактах
- host key changes не обнаруживаются или не блокируются/не требуют подтверждения per policy
- gateway принимает stdin до READY
- воспроизводим обход квот
- unsupported protocol versions fail open (должно быть fail closed)
