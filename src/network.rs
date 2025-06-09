use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

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

    pub fn send_file(&self) -> Result<usize, String> {
        let address = format!("{}:{}", self.ip, self.port);

        let mut file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open file '{}': {}", self.file_path, e))?;

        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file '{}': {}", self.file_path, e))?;

        let mut stream = TcpStream::connect_timeout(
            &address
                .parse()
                .map_err(|e| format!("Invalid address '{}': {}", address, e))?,
            Duration::from_secs(10),
        )
        .map_err(|e| format!("Failed to connect to {}: {}", address, e))?;

        stream
            .set_write_timeout(Some(Duration::from_secs(30)))
            .map_err(|e| format!("Failed to set write timeout: {}", e))?;

        stream
            .write_all(&buffer)
            .map_err(|e| format!("Failed to send data: {}", e))?;

        stream
            .flush()
            .map_err(|e| format!("Failed to flush data: {}", e))?;

        Ok(buffer.len())
    }
}
