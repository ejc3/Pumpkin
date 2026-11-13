use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use tokio::{io::AsyncReadExt, net::TcpStream, time::timeout};

/// The harness server's login timeout: long enough for every bot test's own login.
pub const LOGIN_TIMEOUT_SECS: u64 = 10;

/// A connection that sends nothing is closed once the login timeout runs out, not held open.
pub async fn run_login_timeout_test(
    addr: SocketAddr,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let start = Instant::now();
    let mut stream = TcpStream::connect(addr)
        .await
        .map_err(|err| format!("login timeout: connect failed: {err}"))?;
    let mut buf = [0u8; 64];
    let limit = Duration::from_secs(LOGIN_TIMEOUT_SECS + 10);
    loop {
        match timeout(limit.saturating_sub(start.elapsed()), stream.read(&mut buf)).await {
            Err(_) => {
                return Err(format!(
                    "login timeout: a silent connection was still open after {:?}",
                    start.elapsed()
                ));
            }
            Ok(Ok(0) | Err(_)) => break,
            Ok(Ok(_)) => {}
        }
    }
    let held = start.elapsed();
    if held < Duration::from_secs(LOGIN_TIMEOUT_SECS) {
        return Err(format!(
            "login timeout: the silent connection closed after {held:?}, before the {LOGIN_TIMEOUT_SECS} s timeout"
        ));
    }
    summary_lines.push(format!(
        "login timeout: a silent connection was closed after {:.1} s",
        held.as_secs_f64()
    ));
    Ok(())
}
