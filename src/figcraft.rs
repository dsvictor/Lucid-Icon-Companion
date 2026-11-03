use std::{io::{BufRead, BufReader, Write}, net::{SocketAddr, TcpStream}, path::Path, process::Command, time::Duration};

#[derive(Debug, thiserror::Error)]
pub enum FigCraftError {
    #[error("Could not launch FigCraft: {0}")]
    Launch(String),
    #[error("Could not connect to FigCraft control socket: {0}")]
    Connect(String),
    #[error("Could not send command: {0}")]
    Io(String),
    #[error("FigCraft returned an unexpected response: {0}")]
    Response(String),
}

pub struct FigCraftClient { address: SocketAddr }

impl FigCraftClient {
    pub fn new(port: u16) -> Self {
        Self { address: SocketAddr::from(([127, 0, 0, 1], port)) }
    }

    pub fn launch(&self, app_path: Option<&Path>) -> Result<(), FigCraftError> {
        let mut command = if cfg!(target_os = "macos") {
            let mut c = Command::new("open");
            c.arg("-a").arg(app_path.unwrap_or(Path::new("FigCraft")));
            c.arg("--args").arg("--control").arg(self.address.port().to_string());
            c
        } else {
            let mut c = Command::new(app_path.unwrap_or(Path::new("figcraft")));
            c.arg("--control").arg(self.address.port().to_string());
            c
        };
        command.spawn().map(|_| ()).map_err(|e| FigCraftError::Launch(e.to_string()))
    }

    /// Sends one newline-delimited JSON command and returns the response line.
    /// The exact method and payload must be validated against the installed FigCraft
    /// control-protocol version before release.
    pub fn send_json(&self, command: &serde_json::Value) -> Result<String, FigCraftError> {
        let mut stream = TcpStream::connect_timeout(&self.address, Duration::from_secs(2))
            .map_err(|e| FigCraftError::Connect(e.to_string()))?;
        stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
        stream.write_all(format!("{}\n", command).as_bytes())
            .map_err(|e| FigCraftError::Io(e.to_string()))?;
        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).map_err(|e| FigCraftError::Io(e.to_string()))?;
        if response.is_empty() { return Err(FigCraftError::Response("empty response".into())); }
        Ok(response)
    }
}
