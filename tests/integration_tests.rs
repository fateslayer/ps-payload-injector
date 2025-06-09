use ps_payload_injector::config::Config;
use ps_payload_injector::handlers::{
    create_inject_fn, create_load_config_fn, create_save_config_fn,
};
use ps_payload_injector::network::FileTransfer;
use ps_payload_injector::ui::InjectionStatus;

use std::fs;
use std::sync::mpsc;

use tempfile::NamedTempFile;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

#[test]
fn test_config_roundtrip_integration() {
    // Test complete config save and load cycle
    let original_config = Config::new(
        "172.16.0.1".to_string(),
        "3000".to_string(),
        "/home/user/payload.bin".to_string(),
    );

    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let temp_path = temp_file.path();

    // Save config
    assert!(original_config.save_to_file(temp_path).is_ok());

    // Load config
    let loaded_config = Config::load_from_file(temp_path).expect("Failed to load config");

    // Verify all fields match
    assert_eq!(loaded_config.ip, original_config.ip);
    assert_eq!(loaded_config.port, original_config.port);
    assert_eq!(loaded_config.file_path, original_config.file_path);

    // Verify file content is valid JSON
    let file_content = fs::read_to_string(temp_path).expect("Failed to read file");
    assert!(serde_json::from_str::<Config>(&file_content).is_ok());
}

#[tokio::test]
async fn test_file_transfer_integration() {
    // Create a test file with specific content
    let test_content = b"This is a test payload for integration testing";
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(&temp_file, test_content).expect("Failed to write test data");

    // Start a test server
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind listener");
    let local_addr = listener.local_addr().expect("Failed to get local address");

    // Server task
    let server_handle = tokio::spawn(async move {
        let (mut socket, _) = listener
            .accept()
            .await
            .expect("Failed to accept connection");
        let mut received_data = Vec::new();
        socket
            .read_to_end(&mut received_data)
            .await
            .expect("Failed to read data");
        received_data
    });

    // Create FileTransfer and send file
    let file_transfer = FileTransfer::new(
        local_addr.ip().to_string(),
        local_addr.port().to_string(),
        temp_file.path().to_str().unwrap().to_string(),
    );

    let bytes_sent = file_transfer
        .send_file()
        .await
        .expect("Failed to send file");
    assert_eq!(bytes_sent, test_content.len());

    // Verify server received correct data
    let received_data = server_handle.await.expect("Server task failed");
    assert_eq!(received_data, test_content);
}

#[test]
fn test_config_validation_scenarios() {
    // Test various config validation scenarios that would occur in real usage

    // Valid config
    let valid_temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(
        &valid_temp_file,
        r#"{
        "ip": "192.168.1.100",
        "port": "8080",
        "file_path": "/path/to/payload.bin"
    }"#,
    )
    .expect("Failed to write config");

    let config = Config::load_from_file(valid_temp_file.path()).expect("Should load valid config");
    assert_eq!(config.ip, "192.168.1.100");
    assert_eq!(config.port, "8080");
    assert_eq!(config.file_path, "/path/to/payload.bin");

    // Invalid JSON structure
    let invalid_temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(
        &invalid_temp_file,
        r#"{
        "ip": "192.168.1.100",
        "invalid_field": "value"
    }"#,
    )
    .expect("Failed to write config");

    assert!(Config::load_from_file(invalid_temp_file.path()).is_err());

    // Malformed JSON
    let malformed_temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(&malformed_temp_file, "{ invalid json").expect("Failed to write config");

    assert!(Config::load_from_file(malformed_temp_file.path()).is_err());

    // Empty file
    let empty_temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(&empty_temp_file, "").expect("Failed to write config");

    assert!(Config::load_from_file(empty_temp_file.path()).is_err());
}

#[test]
fn test_simple_file_error() {
    // Test simple file error scenario (fast and reliable)
    let file_transfer = FileTransfer::new(
        "127.0.0.1".to_string(),
        "8080".to_string(),
        "/nonexistent/file.txt".to_string(),
    );

    // We test creation but not execution to avoid network timeouts
    assert_eq!(file_transfer.ip, "127.0.0.1");
    assert_eq!(file_transfer.port, "8080");
    assert_eq!(file_transfer.file_path, "/nonexistent/file.txt");
}

#[test]
fn test_status_enum_behavior() {
    // Test InjectionStatus enum in various scenarios

    let statuses = vec![
        InjectionStatus::Idle,
        InjectionStatus::InProgress("Loading...".to_string()),
        InjectionStatus::Success(1024),
        InjectionStatus::Error("Network error".to_string()),
        InjectionStatus::ConfigLoaded(
            "10.0.0.1".to_string(),
            "9000".to_string(),
            "/test.bin".to_string(),
        ),
    ];

    // Test Debug trait
    for status in &statuses {
        let debug_str = format!("{:?}", status);
        assert!(!debug_str.is_empty());
    }

    // Test pattern matching
    for status in statuses {
        match status {
            InjectionStatus::Idle => assert!(true),
            InjectionStatus::InProgress(msg) => assert!(!msg.is_empty()),
            InjectionStatus::Success(_bytes) => assert!(true),
            InjectionStatus::Error(msg) => assert!(!msg.is_empty()),
            InjectionStatus::ConfigLoaded(ip, port, path) => {
                assert!(!ip.is_empty());
                assert!(!port.is_empty());
                assert!(!path.is_empty());
            }
        }
    }
}

#[test]
fn test_handler_functions_creation() {
    // Test that handler functions can be created without panicking
    let inject_fn = create_inject_fn();
    let save_config_fn = create_save_config_fn();
    let load_config_fn = create_load_config_fn();

    // Test that functions have the correct signatures by creating them
    // Note: We don't call save_config_fn and load_config_fn here because they open file dialogs
    // which require manual user interaction and can't run in automated tests.

    // Only test inject_fn with a non-existent file to avoid actual network operations
    let (tx1, _rx1) = mpsc::channel();
    inject_fn("127.0.0.1", "8080", "/nonexistent/test/path", tx1);

    // Test that the other functions exist with proper types
    let _ = save_config_fn;
    let _ = load_config_fn;

    // If we get here without panicking, the functions were created successfully
    assert!(true);
}

#[test]
fn test_multiple_file_transfer_creation() {
    // Test creating multiple FileTransfer instances (simple and fast)
    let mut transfers = Vec::new();

    for i in 0..3 {
        let transfer = FileTransfer::new(
            format!("192.168.1.{}", i + 1),
            "8080".to_string(),
            format!("/test/file{}.txt", i),
        );
        transfers.push(transfer);
    }

    // Verify all transfers were created correctly
    assert_eq!(transfers.len(), 3);
    assert_eq!(transfers[0].ip, "192.168.1.1");
    assert_eq!(transfers[1].ip, "192.168.1.2");
    assert_eq!(transfers[2].ip, "192.168.1.3");
}

#[test]
fn test_edge_case_configs() {
    // Test edge cases in config handling

    // Config with empty values
    let empty_config = Config::new("".to_string(), "".to_string(), "".to_string());
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");

    assert!(empty_config.save_to_file(temp_file.path()).is_ok());
    let loaded_config =
        Config::load_from_file(temp_file.path()).expect("Failed to load empty config");

    assert_eq!(loaded_config.ip, "");
    assert_eq!(loaded_config.port, "");
    assert_eq!(loaded_config.file_path, "");

    // Config with special characters
    let special_config = Config::new(
        "192.168.1.1".to_string(),
        "8080".to_string(),
        "/path with spaces/file-name_test.txt".to_string(),
    );
    let temp_file2 = NamedTempFile::new().expect("Failed to create temp file");

    assert!(special_config.save_to_file(temp_file2.path()).is_ok());
    let loaded_special =
        Config::load_from_file(temp_file2.path()).expect("Failed to load special config");

    assert_eq!(
        loaded_special.file_path,
        "/path with spaces/file-name_test.txt"
    );

    // Config with unicode characters
    let unicode_config = Config::new(
        "192.168.1.1".to_string(),
        "8080".to_string(),
        "/path/файл.txt".to_string(),
    );
    let temp_file3 = NamedTempFile::new().expect("Failed to create temp file");

    assert!(unicode_config.save_to_file(temp_file3.path()).is_ok());
    let loaded_unicode =
        Config::load_from_file(temp_file3.path()).expect("Failed to load unicode config");

    assert_eq!(loaded_unicode.file_path, "/path/файл.txt");
}
