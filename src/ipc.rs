use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug)]
pub struct IpcRequest {
    pub line: String,
    pub reply_tx: Sender<String>,
}

#[derive(Debug)]
pub struct IpcServer {
    shutdown_tx: Sender<()>,
    join: Option<JoinHandle<()>>,
    socket_path: PathBuf,
}

impl IpcServer {
    pub fn shutdown(mut self) {
        let _ = self.shutdown_tx.send(());
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        let _ = fs::remove_file(&self.socket_path);
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        let _ = fs::remove_file(&self.socket_path);
    }
}

pub fn default_socket_path() -> PathBuf {
    let uid = unsafe { libc::geteuid() };
    let runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| format!("/run/user/{uid}"));
    let wayland_display =
        std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_string());

    PathBuf::from(runtime_dir)
        .join("nighterrors")
        .join(format!("{wayland_display}.sock"))
}

pub fn start_server(
    socket_path: &Path,
    request_tx: Sender<IpcRequest>,
) -> Result<IpcServer, String> {
    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed to create socket directory {}: {e}",
                parent.display()
            )
        })?;
    }

    if socket_path.exists() {
        match UnixStream::connect(socket_path) {
            Ok(_) => {
                return Err(format!(
                    "daemon already running (socket is active at {})",
                    socket_path.display()
                ));
            }
            Err(_) => {
                fs::remove_file(socket_path).map_err(|e| {
                    format!(
                        "failed to remove stale socket {}: {e}",
                        socket_path.display()
                    )
                })?;
            }
        }
    }

    let listener = UnixListener::bind(socket_path)
        .map_err(|e| format!("failed to bind socket {}: {e}", socket_path.display()))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("failed to mark listener nonblocking: {e}"))?;

    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();
    let path = socket_path.to_path_buf();

    let join = thread::spawn(move || run_listener_loop(listener, request_tx, shutdown_rx, path));

    Ok(IpcServer {
        shutdown_tx,
        join: Some(join),
        socket_path: socket_path.to_path_buf(),
    })
}

fn run_listener_loop(
    listener: UnixListener,
    request_tx: Sender<IpcRequest>,
    shutdown_rx: Receiver<()>,
    socket_path: PathBuf,
) {
    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        match listener.accept() {
            Ok((stream, _addr)) => {
                let _ = handle_client(stream, &request_tx);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    let _ = fs::remove_file(socket_path);
}

fn handle_client(mut stream: UnixStream, request_tx: &Sender<IpcRequest>) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|e| format!("failed to clone socket stream: {e}"))?,
    );

    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .map_err(|e| format!("failed to read request: {e}"))?;

    if bytes == 0 {
        return Ok(());
    }

    let request = line.trim_end_matches(['\r', '\n']).to_string();

    let (reply_tx, reply_rx) = mpsc::channel::<String>();
    request_tx
        .send(IpcRequest {
            line: request,
            reply_tx,
        })
        .map_err(|_| "daemon request channel closed".to_string())?;

    let response = reply_rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap_or_else(|_| "error: daemon timed out while handling request".to_string());

    stream
        .write_all(response.as_bytes())
        .map_err(|e| format!("failed to write response body: {e}"))?;
    stream
        .write_all(b"\n")
        .map_err(|e| format!("failed to write response newline: {e}"))?;
    stream
        .flush()
        .map_err(|e| format!("failed to flush response: {e}"))?;

    Ok(())
}

pub fn send_request(socket_path: &Path, request: &str) -> Result<String, String> {
    let mut stream = UnixStream::connect(socket_path)
        .map_err(|e| format!("failed to connect to {}: {e}", socket_path.display()))?;

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("failed to write request: {e}"))?;
    stream
        .write_all(b"\n")
        .map_err(|e| format!("failed to write request terminator: {e}"))?;
    stream
        .flush()
        .map_err(|e| format!("failed to flush request: {e}"))?;
    stream
        .shutdown(Shutdown::Write)
        .map_err(|e| format!("failed to shutdown request stream: {e}"))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("failed to read response: {e}"))?;

    let response = response.trim_end_matches(['\r', '\n']).to_string();

    if response.is_empty() {
        return Err("error: daemon returned an empty response".to_string());
    }

    if response.starts_with("ok") {
        Ok(response)
    } else {
        Err(response)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn unique_socket_path(suffix: &str) -> PathBuf {
        let pid = std::process::id();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("nighterrors-test-{suffix}-{pid}-{now}.sock"))
    }

    #[test]
    fn default_path_contains_nighterrors() {
        let path = default_socket_path();
        assert!(path.to_string_lossy().contains("nighterrors"));
    }

    #[test]
    fn request_roundtrip() {
        let path = unique_socket_path("roundtrip");
        let (tx, rx) = mpsc::channel::<IpcRequest>();
        let server = start_server(&path, tx).expect("server should start");

        let worker = thread::spawn(move || {
            let req = rx.recv().expect("request should be forwarded");
            assert_eq!(req.line, "ping");
            req.reply_tx
                .send("ok pong".to_string())
                .expect("reply should be sent");
        });

        let response = send_request(&path, "ping").expect("request should succeed");
        assert_eq!(response, "ok pong");

        worker.join().expect("worker thread should finish");
        server.shutdown();
    }
}
