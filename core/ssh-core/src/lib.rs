#![allow(clippy::unused_async)]

#[cfg(test)]
use base64::engine::general_purpose::STANDARD_NO_PAD;
#[cfg(test)]
use base64::Engine as _;
use russh::client;
use russh::{Channel, CryptoVec, Disconnect};
use secrecy::{ExposeSecret, SecretString};
#[cfg(test)]
use sha2::{Digest, Sha256};
use ssh_key::HashAlg;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::time::{timeout_at, Duration, Instant};

pub struct EventStream<T> {
    snapshot: VecDeque<T>,
    rx: broadcast::Receiver<T>,
}

impl<T: Clone> EventStream<T> {
    pub async fn recv(&mut self) -> Result<T, broadcast::error::RecvError> {
        if let Some(v) = self.snapshot.pop_front() {
            return Ok(v);
        }
        self.rx.recv().await
    }

    pub fn try_recv(&mut self) -> Result<T, broadcast::error::TryRecvError> {
        if let Some(v) = self.snapshot.pop_front() {
            return Ok(v);
        }
        self.rx.try_recv()
    }
}

struct ClientHandler {
    events_tx: broadcast::Sender<SshEvent>,
    host_key_fingerprint_tx: Mutex<Option<oneshot::Sender<String>>>,
}

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    fn check_server_key<'life0, 'life1, 'async_trait>(
        &'life0 mut self,
        server_public_key: &'life1 ssh_key::PublicKey,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<bool, Self::Error>> + Send + 'async_trait>,
    >
    where
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
    {
        Box::pin(async move {
            let fingerprint = server_public_key.fingerprint(HashAlg::Sha256).to_string();
            if let Some(tx) = self
                .host_key_fingerprint_tx
                .lock()
                .expect("poisoned")
                .take()
            {
                let _ = tx.send(fingerprint);
            }
            Ok(true)
        })
    }

    fn data<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 mut self,
        _channel: russh::ChannelId,
        data: &'life1 [u8],
        _session: &'life2 mut client::Session,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'async_trait>,
    >
    where
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
    {
        let events_tx = self.events_tx.clone();
        Box::pin(async move {
            let _ = events_tx.send(SshEvent::Stdout {
                data: data.to_vec(),
            });
            Ok(())
        })
    }

    fn extended_data<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 mut self,
        _channel: russh::ChannelId,
        ext: u32,
        data: &'life1 [u8],
        _session: &'life2 mut client::Session,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'async_trait>,
    >
    where
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
    {
        let events_tx = self.events_tx.clone();
        Box::pin(async move {
            if ext == 1 {
                let _ = events_tx.send(SshEvent::Stderr {
                    data: data.to_vec(),
                });
            } else {
                let _ = events_tx.send(SshEvent::Stdout {
                    data: data.to_vec(),
                });
            }
            Ok(())
        })
    }

    fn exit_status<'life0, 'life1, 'async_trait>(
        &'life0 mut self,
        _channel: russh::ChannelId,
        exit_status: u32,
        _session: &'life1 mut client::Session,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'async_trait>,
    >
    where
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
    {
        let events_tx = self.events_tx.clone();
        Box::pin(async move {
            let _ = events_tx.send(SshEvent::Exit {
                exit_code: exit_status as i32,
                signal: None,
            });
            Ok(())
        })
    }

    fn exit_signal<'life0, 'life1, 'life2, 'life3, 'async_trait>(
        &'life0 mut self,
        _channel: russh::ChannelId,
        signal_name: russh::Sig,
        _core_dumped: bool,
        _error_message: &'life1 str,
        _lang_tag: &'life2 str,
        _session: &'life3 mut client::Session,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'async_trait>,
    >
    where
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        'life3: 'async_trait,
    {
        let events_tx = self.events_tx.clone();
        Box::pin(async move {
            let _ = events_tx.send(SshEvent::Exit {
                exit_code: 128,
                signal: Some(format!("{signal_name:?}")),
            });
            Ok(())
        })
    }
}

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
    server_fingerprint: Option<String>,
    handle: Option<client::Handle<ClientHandler>>,
    channel: Option<Channel<client::Msg>>,
    username: Option<String>,
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
            server_fingerprint: None,
            handle: None,
            channel: None,
            username: None,
        }
    }

    pub fn server_fingerprint(&self) -> Option<String> {
        self.server_fingerprint.clone()
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

    pub async fn connect(
        host: &str,
        port: u16,
        user: &str,
        timeout_ms: u32,
    ) -> Result<Self, SshError> {
        let mut session = SshSession::new();
        session.transition(SshState::Connecting)?;

        let deadline = Instant::now() + Duration::from_millis(timeout_ms.into());

        let (tx, rx) = oneshot::channel::<String>();
        let handler = ClientHandler {
            events_tx: session.events_tx.clone(),
            host_key_fingerprint_tx: Mutex::new(Some(tx)),
        };

        let config = Arc::new(client::Config::default());
        let handle = timeout_at(deadline, client::connect(config, (host, port), handler))
            .await
            .map_err(|_| SshError::new(SshErrorCode::Timeout, "Connect timeout", true))?
            .map_err(|e| {
                SshError::new(
                    SshErrorCode::ConnectFailed,
                    format!("russh connect failed: {e:?}"),
                    true,
                )
            })?;

        let fingerprint = match timeout_at(deadline, rx).await {
            Ok(Ok(f)) => f,
            Ok(Err(_)) => {
                let _ = handle.disconnect(Disconnect::ByApplication, "", "").await;
                return Err(SshError::new(
                    SshErrorCode::InternalError,
                    "Missing server host key",
                    false,
                ));
            }
            Err(_) => {
                let _ = handle.disconnect(Disconnect::ByApplication, "", "").await;
                return Err(SshError::new(
                    SshErrorCode::Timeout,
                    "Connect timeout",
                    true,
                ));
            }
        };

        session.handle = Some(handle);
        session.username = Some(user.to_string());
        session.server_fingerprint = Some(fingerprint.clone());
        session.pending_host_key = Some(HostKeyPromptEvent {
            fingerprint,
            reason: HostKeyReason::New,
        });
        Ok(session)
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
                self.pending_host_key = None;
                self.transition(SshState::Ready)?;
                return Ok(HostKeyDecision::Unchanged);
            }
            Some(_) => HostKeyReason::Changed,
        };

        match policy {
            HostKeyPolicy::Strict => match reason {
                HostKeyReason::New => {
                    let err = SshError::new(
                        SshErrorCode::HostkeyUnknown,
                        "Host key unknown (policy=strict)",
                        false,
                    );
                    let _ = self.disconnect().await;
                    Err(err)
                }
                HostKeyReason::Changed => {
                    let err = SshError::new(
                        SshErrorCode::HostkeyChanged,
                        "Host key changed (policy=strict)",
                        false,
                    );
                    let _ = self.disconnect().await;
                    Err(err)
                }
            },
            HostKeyPolicy::AcceptNew => match reason {
                HostKeyReason::New => {
                    self.pending_host_key = None;
                    self.transition(SshState::Ready)?;
                    Ok(HostKeyDecision::Accepted)
                }
                HostKeyReason::Changed => {
                    let err = SshError::new(
                        SshErrorCode::HostkeyChanged,
                        "Host key changed (policy=accept-new)",
                        false,
                    );
                    let _ = self.disconnect().await;
                    Err(err)
                }
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
        if !self.is_ready() {
            return Err(SshError::not_ready());
        }
        let handle = self.handle.as_ref().ok_or_else(SshError::not_ready)?;
        let channel = self
            .channel
            .as_ref()
            .ok_or_else(|| SshError::new(SshErrorCode::NotReady, "PTY not open", true))?;

        handle
            .data(channel.id(), CryptoVec::from(data))
            .await
            .map_err(|_| SshError::new(SshErrorCode::InternalError, "Send failed", true))?;
        Ok(())
    }

    pub async fn auth_password(&mut self, password: SecretString) -> Result<(), SshError> {
        if self.state != SshState::Ready {
            return Err(SshError::invalid_state());
        }
        if self.pending_host_key.is_some() {
            return Err(SshError::invalid_state());
        }
        let username = self
            .username
            .as_ref()
            .ok_or_else(|| SshError::new(SshErrorCode::InternalError, "Missing username", false))?
            .clone();
        let handle = self
            .handle
            .as_mut()
            .ok_or_else(|| SshError::new(SshErrorCode::InternalError, "Missing handle", false))?;

        let ok = handle
            .authenticate_password(username, password.expose_secret().to_string())
            .await
            .map_err(|e| {
                SshError::new(
                    SshErrorCode::AuthFailed,
                    format!("auth_password failed: {e:?}"),
                    true,
                )
            })?;
        if !ok {
            return Err(SshError::new(
                SshErrorCode::AuthFailed,
                "Authentication failed",
                false,
            ));
        }
        Ok(())
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
        self.disconnect().await
    }

    pub async fn open_pty(&mut self, _pty: Pty) -> Result<(), SshError> {
        if self.state != SshState::Ready {
            return Err(SshError::invalid_state());
        }
        if self.pending_host_key.is_some() {
            return Err(SshError::invalid_state());
        }

        let pty = _pty;
        let handle = self
            .handle
            .as_ref()
            .ok_or_else(|| SshError::new(SshErrorCode::InternalError, "Missing handle", false))?;
        let channel = handle.channel_open_session().await.map_err(|e| {
            SshError::new(
                SshErrorCode::InternalError,
                format!("channel_open_session failed: {e:?}"),
                true,
            )
        })?;

        channel
            .request_pty(true, &pty.term, pty.cols as u32, pty.rows as u32, 0, 0, &[])
            .await
            .map_err(|e| {
                SshError::new(
                    SshErrorCode::InternalError,
                    format!("request_pty failed: {e:?}"),
                    true,
                )
            })?;
        channel.request_shell(true).await.map_err(|e| {
            SshError::new(
                SshErrorCode::InternalError,
                format!("request_shell failed: {e:?}"),
                true,
            )
        })?;

        self.channel = Some(channel);
        Ok(())
    }

    pub async fn resize(&mut self, _cols: u16, _rows: u16) -> Result<(), SshError> {
        if !self.is_ready() {
            return Err(SshError::not_ready());
        }
        let channel = self
            .channel
            .as_ref()
            .ok_or_else(|| SshError::new(SshErrorCode::NotReady, "PTY not open", true))?;
        channel
            .window_change(_cols as u32, _rows as u32, 0, 0)
            .await
            .map_err(|e| {
                SshError::new(
                    SshErrorCode::InternalError,
                    format!("window_change failed: {e:?}"),
                    true,
                )
            })?;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), SshError> {
        if let Some(channel) = self.channel.take() {
            let _ = channel.close().await;
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.disconnect(Disconnect::ByApplication, "", "").await;
        }
        self.transition(SshState::Closing)?;
        self.transition(SshState::Closed)?;
        self.pending_host_key = None;
        self.server_fingerprint = None;
        self.username = None;
        Ok(())
    }

    pub fn subscribe_events(&self) -> EventStream<SshEvent> {
        let mut snapshot = VecDeque::new();
        snapshot.push_back(SshEvent::Status { state: self.state });
        if self.state == SshState::HostKeyPrompt {
            if let Some(pending) = self.pending_host_key.as_ref() {
                snapshot.push_back(SshEvent::HostKeyPrompt {
                    fingerprint: pending.fingerprint.clone(),
                    reason: pending.reason,
                });
            }
        }

        EventStream {
            snapshot,
            rx: self.events_tx.subscribe(),
        }
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

    fn seed_pending_host_key(session: &mut SshSession, seed: &[u8]) {
        session.transition(SshState::Connecting).unwrap();
        session.pending_host_key = Some(HostKeyPromptEvent {
            fingerprint: format!("SHA256:{}", STANDARD_NO_PAD.encode(Sha256::digest(seed))),
            reason: HostKeyReason::New,
        });
    }

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

        assert!(matches!(
            rx.try_recv(),
            Ok(SshEvent::Status {
                state: SshState::Init
            })
        ));

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
        seed_pending_host_key(&mut session, b"example");

        let result = session.verify_host_key(HostKeyPolicy::Strict, None).await;
        assert!(matches!(result, Err(e) if e.code == SshErrorCode::HostkeyUnknown));
        assert_eq!(session.state, SshState::Closed);
    }

    #[tokio::test]
    async fn test_verify_host_key_accept_new_unknown_accepts() {
        let mut session = SshSession::new();
        seed_pending_host_key(&mut session, b"example");

        let result = session
            .verify_host_key(HostKeyPolicy::AcceptNew, None)
            .await;
        assert!(matches!(result, Ok(HostKeyDecision::Accepted)));
        assert!(session.is_ready());
    }

    #[tokio::test]
    async fn test_verify_host_key_accept_new_changed_disconnects() {
        let mut session = SshSession::new();
        seed_pending_host_key(&mut session, b"example");

        let known = KnownHostEntry {
            fingerprint: "SHA256:DIFFERENT".to_string(),
        };
        let result = session
            .verify_host_key(HostKeyPolicy::AcceptNew, Some(known))
            .await;
        assert!(matches!(result, Err(e) if e.code == SshErrorCode::HostkeyChanged));
        assert_eq!(session.state, SshState::Closed);
    }

    #[tokio::test]
    async fn test_verify_host_key_ask_emits_prompt_and_accepts() {
        let mut session = SshSession::new();
        let mut rx = session.subscribe_events();
        seed_pending_host_key(&mut session, b"example");

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
