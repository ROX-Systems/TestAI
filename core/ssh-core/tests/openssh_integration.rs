use secrecy::SecretString;
use ssh_core::{
    HostKeyDecision, HostKeyPolicy, KnownHostEntry, Pty, SshErrorCode, SshEvent, SshSession,
    SshState,
};
use std::env;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

fn it_enabled() -> bool {
    env::var("SSH_IT_ENABLE").ok().as_deref() == Some("1")
}

fn runtime() -> String {
    if let Ok(v) = env::var("SSH_IT_RUNTIME") {
        return v;
    }

    if check_runtime("podman") {
        return "podman".to_string();
    }

    "docker".to_string()
}

fn player_cmd(cmd: &mut Command) -> Output {
    cmd.output().expect("failed to run container runtime")
}

fn check_runtime(bin: &str) -> bool {
    let mut cmd = Command::new(bin);
    cmd.arg("--version");
    let out = player_cmd(&mut cmd);
    out.status.success()
}

fn random_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    format!("{now}")
}

fn pick_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);
    port
}

fn wait_ssh_banner(host: &str, port: u16, max_wait: Duration) {
    let start = Instant::now();
    loop {
        if let Ok(mut stream) = TcpStream::connect((host, port)) {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
            let mut buf = [0u8; 64];

            if let Ok(n) = stream.read(&mut buf) {
                if n > 0 && buf[..n].starts_with(b"SSH-") {
                    return;
                }
            }
        }
        if start.elapsed() > max_wait {
            panic!("ssh banner not ready on {host}:{port} after {max_wait:?}");
        }
        thread::sleep(Duration::from_millis(200));
    }
}

async fn connect_with_retry(
    host: &str,
    port: u16,
    user: &str,
) -> Result<SshSession, ssh_core::SshError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);

    loop {
        let attempt_timeout_ms = 8_000;
        match SshSession::connect(host, port, user, attempt_timeout_ms).await {
            Ok(s) => return Ok(s),
            Err(e) => {
                if !e.retryable || e.code != SshErrorCode::ConnectFailed {
                    return Err(e);
                }

                if tokio::time::Instant::now() >= deadline {
                    return Err(e);
                }

                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
    }
}

struct Container {
    runtime: String,
    name: String,
}

impl Drop for Container {
    fn drop(&mut self) {
        let mut cmd = Command::new(&self.runtime);
        cmd.args(["rm", "-f", &self.name]);
        let _ = player_cmd(&mut cmd);
    }
}

fn start_openssh_container(host_port: u16, username: &str, password: &str) -> Container {
    let runtime = runtime();
    let name = format!("ssh-it-{}", random_suffix());

    let image = env::var("SSH_IT_IMAGE")
        .unwrap_or_else(|_| "lscr.io/linuxserver/openssh-server:latest".to_string());

    let port_arg = format!("{host_port}:2222");
    let user_arg = format!("USER_NAME={username}");
    let pass_arg = format!("USER_PASSWORD={password}");

    let mut cmd = Command::new(&runtime);
    cmd.args([
        "run",
        "-d",
        "--rm",
        "--name",
        &name,
        "-p",
        &port_arg,
        "-e",
        "PASSWORD_ACCESS=true",
        "-e",
        &user_arg,
        "-e",
        &pass_arg,
        "-e",
        "SUDO_ACCESS=false",
        &image,
    ]);
    let out = player_cmd(&mut cmd);

    if !out.status.success() {
        panic!(
            "container start failed (runtime={runtime}): {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    wait_ssh_banner("127.0.0.1", host_port, Duration::from_secs(30));

    Container { runtime, name }
}

async fn recv_stdout_contains(
    rx: &mut ssh_core::EventStream<SshEvent>,
    needle: &str,
    max_wait: Duration,
) {
    let mut buf = String::new();
    let deadline = Instant::now() + max_wait;

    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .unwrap_or(Duration::from_millis(1));

        let ev = timeout(remaining, rx.recv())
            .await
            .unwrap_or_else(|_| panic!("timeout waiting for stdout (needle={needle})"))
            .unwrap();

        if let SshEvent::Stdout { data } = ev {
            buf.push_str(&String::from_utf8_lossy(&data));
            if buf.contains(needle) {
                return;
            }
        }
    }
}

#[tokio::test]
async fn it_accept_new_accept_once_and_pty_resize() {
    if !it_enabled() {
        return;
    }

    let port = pick_free_port();
    let _c = start_openssh_container(port, "ituser", "itpass");

    let mut s1 = connect_with_retry("127.0.0.1", port, "ituser")
        .await
        .unwrap();

    let decision = s1
        .verify_host_key(HostKeyPolicy::AcceptNew, None)
        .await
        .unwrap();
    assert!(matches!(decision, HostKeyDecision::Accepted));
    assert!(s1.is_ready());

    let fp = s1.server_fingerprint().expect("fingerprint");

    s1.auth_password(SecretString::new("itpass".to_string()))
        .await
        .unwrap();

    let mut rx = s1.subscribe_events();

    s1.open_pty(Pty {
        cols: 80,
        rows: 24,
        term: "xterm-256color".to_string(),
    })
    .await
    .unwrap();

    s1.resize(100, 40).await.unwrap();
    s1.write_stdin(b"stty size\n").await.unwrap();

    recv_stdout_contains(&mut rx, "40 100", Duration::from_secs(10)).await;

    s1.disconnect().await.unwrap();

    let mut s2 = connect_with_retry("127.0.0.1", port, "ituser")
        .await
        .unwrap();

    let decision2 = s2
        .verify_host_key(
            HostKeyPolicy::AcceptNew,
            Some(KnownHostEntry {
                fingerprint: fp.clone(),
            }),
        )
        .await
        .unwrap();

    assert!(matches!(decision2, HostKeyDecision::Unchanged));
    assert!(s2.is_ready());

    s2.disconnect().await.unwrap();
}

#[tokio::test]
async fn it_strict_changed_hostkey_blocks() {
    if !it_enabled() {
        return;
    }

    let port = pick_free_port();
    let c1 = start_openssh_container(port, "ituser", "itpass");

    let mut s1 = connect_with_retry("127.0.0.1", port, "ituser")
        .await
        .unwrap();
    s1.verify_host_key(HostKeyPolicy::AcceptNew, None)
        .await
        .unwrap();
    let fp_a = s1.server_fingerprint().expect("fingerprint");
    s1.disconnect().await.unwrap();
    drop(c1);
    thread::sleep(Duration::from_millis(500));

    let _c2 = start_openssh_container(port, "ituser", "itpass");

    let mut s2 = connect_with_retry("127.0.0.1", port, "ituser")
        .await
        .unwrap();

    let fp_b = s2.server_fingerprint().expect("fingerprint");
    assert_ne!(fp_a, fp_b);

    let res = s2
        .verify_host_key(
            HostKeyPolicy::Strict,
            Some(KnownHostEntry { fingerprint: fp_a }),
        )
        .await;

    assert!(matches!(res, Err(e) if e.code == SshErrorCode::HostkeyChanged));
    assert_eq!(
        s2.subscribe_events().try_recv().ok(),
        Some(SshEvent::Status {
            state: SshState::Closed
        })
    );
}
