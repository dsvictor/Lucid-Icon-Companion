use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, thiserror::Error)]
pub enum FigCraftError {
    #[error("Could not launch FigCraft: {0}")]
    Launch(String),
    #[error("Could not connect to FigCraft: {0}")]
    Connect(String),
    #[error("Control socket I/O failed: {0}")]
    Io(String),
    #[error("FigCraft rejected the command: {0}")]
    Rejected(String),
    #[error("FigCraft returned an invalid response: {0}")]
    Response(String),
}

pub struct FigCraftClient {
    address: SocketAddr,
}

impl FigCraftClient {
    pub fn new(port: u16) -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], port)),
        }
    }

    /// Launch the macOS application with FigCraft's loopback control listener enabled.
    pub fn launch(&self) -> Result<(), FigCraftError> {
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .args(["-a", "FigCraft", "--args", "--control"])
                .arg(self.address.port().to_string())
                .spawn()
                .map(|_| ())
                .map_err(|e| FigCraftError::Launch(e.to_string()))
        }
        #[cfg(not(target_os = "macos"))]
        {
            Command::new("figcraft")
                .arg("--control")
                .arg(self.address.port().to_string())
                .spawn()
                .map(|_| ())
                .map_err(|e| FigCraftError::Launch(e.to_string()))
        }
    }

    /// Insert an SVG as editable geometry. With no x/y, FigCraft uses the current
    /// viewport centre; center=true centres the imported frame on that point.
    pub fn insert_svg(&self, name: &str, svg: &str) -> Result<String, FigCraftError> {
        let request_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let data = STANDARD.encode(svg.as_bytes());
        let request = json!({
            "id": request_id,
            "method": "engine.execute",
            "params": {
                "command": "file.import",
                "params": {
                    "data": data,
                    "name": format!("{name}.svg"),
                    "center": true
                }
            }
        });

        let response = self.send(&request)?;
        Ok(response.to_string())
    }

    fn send(&self, request: &Value) -> Result<Value, FigCraftError> {
        let mut stream = TcpStream::connect_timeout(&self.address, Duration::from_secs(3))
            .map_err(|e| FigCraftError::Connect(e.to_string()))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(60)))
            .map_err(|e| FigCraftError::Io(e.to_string()))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(60)))
            .map_err(|e| FigCraftError::Io(e.to_string()))?;
        stream
            .write_all(format!("{request}\n").as_bytes())
            .map_err(|e| FigCraftError::Io(e.to_string()))?;

        let mut line = String::new();
        BufReader::new(stream)
            .read_line(&mut line)
            .map_err(|e| FigCraftError::Io(e.to_string()))?;
        if line.is_empty() {
            return Err(FigCraftError::Response("empty reply".into()));
        }

        let reply: Value = serde_json::from_str(&line)
            .map_err(|e| FigCraftError::Response(e.to_string()))?;
        if reply.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(FigCraftError::Rejected(
                reply
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown control-channel error")
                    .to_owned(),
            ));
        }

        Ok(reply.get("result").cloned().unwrap_or(Value::Null))
    }
}
