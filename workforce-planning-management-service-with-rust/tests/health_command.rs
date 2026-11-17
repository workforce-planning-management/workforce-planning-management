//! `workforce-planning-management health` (WPM-T524, `axa` 0.16): it asks the server on this machine and exits by its answer.
//! The server here is a stub that answers one request, so the test needs no database and no running service.

use std::process::Command;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// A listener that answers its first request with `status`, and the path it was asked for.
async fn stub(status: &'static str) -> (u16, tokio::task::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let asked = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut head = [0_u8; 256];
        let read = stream.read(&mut head).await.expect("read");
        let reply = format!("HTTP/1.1 {status}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n");
        stream.write_all(reply.as_bytes()).await.expect("write");
        String::from_utf8_lossy(&head[..read]).into_owned()
    });
    (port, asked)
}

/// Runs the binary's `health` with these arguments, off the async runtime, and gives its exit status and output.
async fn health(args: Vec<String>) -> (bool, String) {
    tokio::task::spawn_blocking(move || {
        let out = Command::new(env!("CARGO_BIN_EXE_workforce-planning-management"))
            .arg("health")
            .args(&args)
            .env_remove("INTERNAL_PORT")
            .output()
            .expect("run the binary");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        (out.status.success(), text)
    })
    .await
    .expect("join")
}

#[tokio::test]
async fn a_live_server_passes_and_is_asked_for_the_liveness_probe() {
    let (port, asked) = stub("200 OK").await;
    let (ok, text) = health(vec!["--port".into(), port.to_string()]).await;
    assert!(ok, "health should pass on a 200: {text}");
    assert!(asked.await.expect("stub").starts_with("GET /_health/live "));
}

#[tokio::test]
async fn ready_asks_the_readiness_probe_and_a_503_fails() {
    let (port, asked) = stub("503 Service Unavailable").await;
    let (ok, text) = health(vec!["--ready".into(), "--port".into(), port.to_string()]).await;
    assert!(!ok, "health should fail on a 503: {text}");
    assert!(
        asked
            .await
            .expect("stub")
            .starts_with("GET /_health/ready ")
    );
}

#[tokio::test]
async fn nothing_listening_fails() {
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        listener.local_addr().expect("addr").port()
    };
    let (ok, _) = health(vec!["--port".into(), port.to_string()]).await;
    assert!(!ok);
}
