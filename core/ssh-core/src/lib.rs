#![allow(clippy::unused_async)]

use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine as _;
use russh as _;
use russh_keys as _;
use secrecy::SecretString;
use sha2::{Digest, Sha256};
use tokio::sync::broadcast;

pub type EventStream<T> = broadcast::Receiver<T>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostKeyPolicy {
    Strict,
    AcceptNew,
    Ask,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pty {
    pub cols: u16,
    pub rows: u16,
    pub term: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostKeyReason {
    New,
    Changed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownHostEntry {
    pub fingerprint: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostKeyDecision {
    Accepted,
    Rejected,
    Unchanged,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SshEvent {
    Status {
        state: SshState,
    },
    Stdout {
        data: Vec<u8>,
    },
    Stderr {
        data: Vec<u8>,
    },
    HostKeyPrompt {
        fingerprint: String,
        reason: HostKeyReason,
    },
    Exit {
        exit_code: i32,
        signal: Option<String>,
    },
    Error {
        code: SshErrorCode,
        message: String,
        retryable: bool,
    },
}

/// Состояния SSH-сессии согласно SRS §7.1.3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SshState {
    Init,
    Connecting,
    HostKeyPrompt,
    Ready,
    Closing,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SshErrorCode {
    AuthFailed,
    UnsupportedProtocol,
    BadRequest,
    NotReady,
    SessionConflict,
    ConnectFailed,
    DnsFailed,
    Timeout,
    HostkeyChanged,
    HostkeyUnknown,
    HostkeyRejected,
    QuotaExceeded,
    RateLimited,
    InternalError,
    InvalidState,
}

impl SshErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SshErrorCode::AuthFailed => "AUTH_FAILED",
            SshErrorCode::UnsupportedProtocol => "UNSUPPORTED_PROTOCOL",
            SshErrorCode::BadRequest => "BAD_REQUEST",
            SshErrorCode::NotReady => "NOT_READY",
            SshErrorCode::SessionConflict => "SESSION_CONFLICT",
            SshErrorCode::ConnectFailed => "CONNECT_FAILED",
            SshErrorCode::DnsFailed => "DNS_FAILED",
            SshErrorCode::Timeout => "TIMEOUT",
            SshErrorCode::HostkeyChanged => "HOSTKEY_CHANGED",
            SshErrorCode::HostkeyUnknown => "HOSTKEY_UNKNOWN",
            SshErrorCode::HostkeyRejected => "HOSTKEY_REJECTED",
            SshErrorCode::QuotaExceeded => "QUOTA_EXCEEDED",
            SshErrorCode::RateLimited => "RATE_LIMITED",
            SshErrorCode::InternalError => "INTERNAL_ERROR",
            SshErrorCode::InvalidState => "INVALID_STATE",
        }
    }
}

impl std::fmt::Display for SshErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshError {
    pub code: SshErrorCode,
    pub message: String,
    pub retryable: bool,
}

impl SshError {
    pub fn new(code: SshErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code,
            message: message.into(),
            retryable,
        }
    }

    fn not_ready() -> Self {
        Self::new(
            SshErrorCode::NotReady,
            "Operation requires READY state",
            true,
        )
    }

    fn not_implemented(op: &'static str) -> Self {
        Self::new(
            SshErrorCode::InternalError,
            format!("Not implemented: {op}"),
            false,
        )
    }

    fn invalid_state() -> Self {
        Self::new(SshErrorCode::InvalidState, "Invalid state", false)
    }
}

impl std::fmt::Display for SshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SshError {}

#[derive(Clone, Debug)]
pub struct PrivateKeyRef {
    pub pem: SecretString,
}

/// Основная структура SSH-сессии
pub struct SshSession {
    state: SshState,
    events_tx: broadcast::Sender<SshEvent>,
    pending_host_key: Option<HostKeyPromptEvent>,
}

impl SshSession {
    fn can_transition(from: SshState, to: SshState) -> bool {
        if from == to {
            return true;
        }

        if from == SshState::Closed {
            return false;
        }

        matches!(
            (from, to),
            (SshState::Init, SshState::Connecting)
                | (SshState::Connecting, SshState::HostKeyPrompt)
                | (SshState::Connecting, SshState::Ready)
                | (SshState::HostKeyPrompt, SshState::Ready)
                | (SshState::Ready, SshState::Closing)
                | (SshState::Closing, SshState::Closed)
                | (_, SshState::Closing)
                | (_, SshState::Closed)
        )
    }

    pub fn new() -> Self {
        SshSession {
            state: SshState::Init,
            events_tx: broadcast::Sender::new(256),
            pending_host_key: None,
        }
    }

    /// Обновление состояния сессии
    pub fn transition(&mut self, new_state: SshState) -> Result<(), SshError> {
        if !Self::can_transition(self.state, new_state) {
            return Err(SshError::invalid_state());
        }

        if new_state == self.state {
            return Ok(());
        }

        self.state = new_state;
        let _ = self.events_tx.send(SshEvent::Status { state: new_state });
        Ok(())
    }

    /// Проверка, разрешены ли операции ввода
    pub fn is_ready(&self) -> bool {
        self.state == SshState::Ready
    }

    pub async fn connect(&mut self, host: &str, port: u16, user: &str) -> Result<(), SshError> {
        let _ = (host, port, user); // Заглушка для неиспользуемых переменных
        self.transition(SshState::Connecting)?;
        // Логика установки соединения
        // ...

        self.pending_host_key = Some(HostKeyPromptEvent {
            fingerprint: format!(
                "SHA256:{}",
                STANDARD_NO_PAD.encode(Sha256::digest(host.as_bytes()))
            ),
            reason: HostKeyReason::New,
        });
        Ok(())
    }

    pub async fn verify_host_key(
        &mut self,
        policy: HostKeyPolicy,
        known: Option<KnownHostEntry>,
    ) -> Result<HostKeyDecision, SshError> {
        if self.state != SshState::Connecting && self.state != SshState::HostKeyPrompt {
            return Err(SshError::invalid_state());
        }

        let server_fingerprint = match self.pending_host_key.as_ref() {
            Some(p) => p.fingerprint.as_str(),
            None => {
                return Err(SshError::new(
                    SshErrorCode::InternalError,
                    "Missing pending host key",
                    false,
                ))
            }
        };

        let reason = match known.as_ref() {
            None => HostKeyReason::New,
            Some(k) if k.fingerprint == server_fingerprint => {
                self.transition(SshState::Ready)?;
                return Ok(HostKeyDecision::Unchanged);
            }
            Some(_) => HostKeyReason::Changed,
        };

        match policy {
            HostKeyPolicy::Strict => match reason {
                HostKeyReason::New => Err(SshError::new(
                    SshErrorCode::HostkeyUnknown,
                    "Host key unknown (policy=strict)",
                    false,
                )),
                HostKeyReason::Changed => Err(SshError::new(
                    SshErrorCode::HostkeyChanged,
                    "Host key changed (policy=strict)",
                    false,
                )),
            },
            HostKeyPolicy::AcceptNew => match reason {
                HostKeyReason::New => {
                    self.transition(SshState::Ready)?;
                    Ok(HostKeyDecision::Accepted)
                }
                HostKeyReason::Changed => Err(SshError::new(
                    SshErrorCode::HostkeyChanged,
                    "Host key changed (policy=accept-new)",
                    false,
                )),
            },
            HostKeyPolicy::Ask => {
                self.pending_host_key = Some(HostKeyPromptEvent {
                    fingerprint: server_fingerprint.to_string(),
                    reason,
                });
                self.transition(SshState::HostKeyPrompt)?;
                let pending = self.pending_host_key.as_ref().expect("just set");
                let _ = self.events_tx.send(SshEvent::HostKeyPrompt {
                    fingerprint: pending.fingerprint.clone(),
                    reason: pending.reason,
                });
                Ok(HostKeyDecision::Unchanged)
            }
        }
    }

    pub async fn write_stdin(&mut self, data: &[u8]) -> Result<(), SshError> {
        let _ = data; // Заглушка
        if !self.is_ready() {
            return Err(SshError::not_ready());
        }
        // Логика отправки данных
        // ...
        Ok(())
    }

    pub async fn auth_password(&mut self, _password: SecretString) -> Result<(), SshError> {
        Err(SshError::not_implemented("auth_password"))
    }

    pub async fn auth_key(
        &mut self,
        _key: PrivateKeyRef,
        _passphrase: Option<SecretString>,
    ) -> Result<(), SshError> {
        Err(SshError::not_implemented("auth_key"))
    }

    pub async fn host_key_accept(&mut self) -> Result<(), SshError> {
        if self.state != SshState::HostKeyPrompt {
            return Err(SshError::invalid_state());
        }
        self.pending_host_key = None;
        self.transition(SshState::Ready)?;
        Ok(())
    }

    pub async fn host_key_reject(&mut self) -> Result<(), SshError> {
        if self.state != SshState::HostKeyPrompt {
            return Err(SshError::invalid_state());
        }
        self.pending_host_key = None;
        self.transition(SshState::Closing)?;
        self.transition(SshState::Closed)?;
        Ok(())
    }

    pub async fn open_pty(&mut self, _pty: Pty) -> Result<(), SshError> {
        Err(SshError::not_implemented("open_pty"))
    }

    pub async fn resize(&mut self, _cols: u16, _rows: u16) -> Result<(), SshError> {
        if !self.is_ready() {
            return Err(SshError::not_ready());
        }
        Err(SshError::not_implemented("resize"))
    }

    pub async fn disconnect(&mut self) -> Result<(), SshError> {
        self.transition(SshState::Closing)?;
        self.transition(SshState::Closed)?;
        Ok(())
    }

    pub fn subscribe_events(&self) -> EventStream<SshEvent> {
        self.events_tx.subscribe()
    }
}

impl Default for SshSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Событие запроса проверки host key
#[derive(Debug)]
pub struct HostKeyPromptEvent {
    pub fingerprint: String,
    pub reason: HostKeyReason,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_state_transitions() {
        let mut session = SshSession::new();
        assert_eq!(session.state, SshState::Init);

        assert!(matches!(
            session.transition(SshState::Ready),
            Err(e) if e.code == SshErrorCode::InvalidState
        ));

        session.transition(SshState::Connecting).unwrap();
        assert_eq!(session.state, SshState::Connecting);

        session.transition(SshState::Ready).unwrap();
        assert_eq!(session.state, SshState::Ready);
    }

    #[tokio::test]
    async fn test_transition_emits_status_event() {
        let mut session = SshSession::new();
        let mut rx = session.subscribe_events();

        session.transition(SshState::Connecting).unwrap();
        assert!(matches!(
            rx.try_recv(),
            Ok(SshEvent::Status {
                state: SshState::Connecting
            })
        ));
    }

    #[tokio::test]
    async fn test_verify_host_key_errors() {
        let mut session = SshSession::new();

        // Попытка верификации в неверном состоянии
        let result = session.verify_host_key(HostKeyPolicy::Strict, None).await;
        assert!(matches!(result, Err(e) if e.code == SshErrorCode::InvalidState));
    }

    #[tokio::test]
    async fn test_verify_host_key_strict_unknown_fails() {
        let mut session = SshSession::new();
        session.connect("example", 22, "user").await.unwrap();

        let result = session.verify_host_key(HostKeyPolicy::Strict, None).await;
        assert!(matches!(result, Err(e) if e.code == SshErrorCode::HostkeyUnknown));
    }

    #[tokio::test]
    async fn test_verify_host_key_accept_new_unknown_accepts() {
        let mut session = SshSession::new();
        session.connect("example", 22, "user").await.unwrap();

        let result = session
            .verify_host_key(HostKeyPolicy::AcceptNew, None)
            .await;
        assert!(matches!(result, Ok(HostKeyDecision::Accepted)));
        assert!(session.is_ready());
    }

    #[tokio::test]
    async fn test_verify_host_key_ask_emits_prompt_and_accepts() {
        let mut session = SshSession::new();
        let mut rx = session.subscribe_events();
        session.connect("example", 22, "user").await.unwrap();

        let result = session.verify_host_key(HostKeyPolicy::Ask, None).await;
        assert!(matches!(result, Ok(HostKeyDecision::Unchanged)));

        loop {
            match rx.try_recv() {
                Ok(SshEvent::HostKeyPrompt { .. }) => break,
                Ok(_) => continue,
                Err(_) => panic!("missing HostKeyPrompt event"),
            }
        }

        session.host_key_accept().await.unwrap();
        assert!(session.is_ready());
    }
}
