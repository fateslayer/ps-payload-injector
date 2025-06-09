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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;

    #[test]
    fn test_file_transfer_new() {
        let transfer = FileTransfer::new(
            "192.168.1.100".to_string(),
            "8080".to_string(),
            "/path/to/file.txt".to_string(),
        );

        assert_eq!(transfer.ip, "192.168.1.100");
        assert_eq!(transfer.port, "8080");
        assert_eq!(transfer.file_path, "/path/to/file.txt");
    }

    #[tokio::test]
    async fn test_send_file_nonexistent_file() {
        let transfer = FileTransfer::new(
            "127.0.0.1".to_string(),
            "8080".to_string(),
            "/nonexistent/file.txt".to_string(),
        );

        let result = transfer.send_file().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to open file"));
    }

    #[tokio::test]
    async fn test_send_file_invalid_address() {
        // Create a temporary file with some content
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, b"test data").expect("Failed to write test data");

        let transfer = FileTransfer::new(
            "invalid.address".to_string(),
            "8080".to_string(),
            temp_file.path().to_str().unwrap().to_string(),
        );

        let result = transfer.send_file().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to connect to"));
    }

    #[tokio::test]
    async fn test_send_file_connection_refused() {
        // Create a temporary file with some content
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, b"test data").expect("Failed to write test data");

        let transfer = FileTransfer::new(
            "127.0.0.1".to_string(),
            "65432".to_string(), // Use a port that's likely not in use
            temp_file.path().to_str().unwrap().to_string(),
        );

        let result = transfer.send_file().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to connect to"));
    }

    #[tokio::test]
    async fn test_send_file_successful() {
        // Create a temporary file with test content
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let test_data = b"Hello, this is test file content for network transfer!";
        fs::write(&temp_file, test_data).expect("Failed to write test data");

        // Start a test TCP server
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind listener");
        let local_addr = listener.local_addr().expect("Failed to get local address");

        // Spawn server task
        let server_handle = tokio::spawn(async move {
            let (mut socket, _) = listener
                .accept()
                .await
                .expect("Failed to accept connection");
            let mut buffer = Vec::new();
            socket
                .read_to_end(&mut buffer)
                .await
                .expect("Failed to read data");
            buffer
        });

        // Create file transfer and send
        let transfer = FileTransfer::new(
            local_addr.ip().to_string(),
            local_addr.port().to_string(),
            temp_file.path().to_str().unwrap().to_string(),
        );

        let result = transfer.send_file().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_data.len());

        // Verify server received the data
        let received_data = server_handle.await.expect("Server task failed");
        assert_eq!(received_data, test_data);
    }

    #[tokio::test]
    async fn test_send_empty_file() {
        // Create an empty temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");

        // Start a test TCP server
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind listener");
        let local_addr = listener.local_addr().expect("Failed to get local address");

        // Spawn server task
        let server_handle = tokio::spawn(async move {
            let (mut socket, _) = listener
                .accept()
                .await
                .expect("Failed to accept connection");
            let mut buffer = Vec::new();
            socket
                .read_to_end(&mut buffer)
                .await
                .expect("Failed to read data");
            buffer
        });

        // Create file transfer and send
        let transfer = FileTransfer::new(
            local_addr.ip().to_string(),
            local_addr.port().to_string(),
            temp_file.path().to_str().unwrap().to_string(),
        );

        let result = transfer.send_file().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Verify server received empty data
        let received_data = server_handle.await.expect("Server task failed");
        assert_eq!(received_data.len(), 0);
    }

    #[tokio::test]
    async fn test_send_large_file() {
        // Create a temporary file with larger content (1MB)
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let test_data = vec![0xAB; 1024 * 1024]; // 1MB of data
        fs::write(&temp_file, &test_data).expect("Failed to write test data");

        // Start a test TCP server
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind listener");
        let local_addr = listener.local_addr().expect("Failed to get local address");

        // Spawn server task
        let server_handle = tokio::spawn(async move {
            let (mut socket, _) = listener
                .accept()
                .await
                .expect("Failed to accept connection");
            let mut buffer = Vec::new();
            socket
                .read_to_end(&mut buffer)
                .await
                .expect("Failed to read data");
            buffer
        });

        // Create file transfer and send
        let transfer = FileTransfer::new(
            local_addr.ip().to_string(),
            local_addr.port().to_string(),
            temp_file.path().to_str().unwrap().to_string(),
        );

        let result = transfer.send_file().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_data.len());

        // Verify server received all the data
        let received_data = server_handle.await.expect("Server task failed");
        assert_eq!(received_data.len(), test_data.len());
        assert_eq!(received_data, test_data);
    }

    #[test]
    fn test_file_transfer_with_ipv6() {
        let transfer = FileTransfer::new(
            "::1".to_string(),
            "8080".to_string(),
            "/test/file.txt".to_string(),
        );

        assert_eq!(transfer.ip, "::1");
        assert_eq!(transfer.port, "8080");
    }

    #[test]
    fn test_file_transfer_with_special_characters_in_path() {
        let transfer = FileTransfer::new(
            "192.168.1.1".to_string(),
            "8080".to_string(),
            "/path with spaces/file-name_test.txt".to_string(),
        );

        assert_eq!(transfer.file_path, "/path with spaces/file-name_test.txt");
    }
}
