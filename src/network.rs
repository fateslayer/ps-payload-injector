use std::fs::File;
use std::io::Read;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub struct FileTransfer {
    pub ip: String,
    pub port: String,
    pub file_path: String,
}

impl FileTransfer {
    pub fn new(ip: String, port: String, file_path: String) -> Self {
        Self {
            ip,
            port,
            file_path,
        }
    }

    pub async fn send_file(&self) -> Result<usize, String> {
        let address = format!("{}:{}", self.ip, self.port);

        let mut file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open file '{}': {}", self.file_path, e))?;

        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file '{}': {}", self.file_path, e))?;

        let mut stream =
            tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(&address))
                .await
                .map_err(|_| format!("Connection timeout to {}", address))?
                .map_err(|e| format!("Failed to connect to {}: {}", address, e))?;

        stream
            .write_all(&buffer)
            .await
            .map_err(|e| format!("Failed to send data: {}", e))?;

        stream
            .flush()
            .await
            .map_err(|e| format!("Failed to flush data: {}", e))?;

        Ok(buffer.len())
    }
}
