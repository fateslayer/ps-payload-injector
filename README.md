# PS Payload Injector

A cross-platform GUI application built with Rust for network payload injection. Features an intuitive interface for configuring target connections and managing payload files.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-lightgrey.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## ✨ Features

- **Intuitive GUI**: Clean, modern interface built with egui
- **Cross-Platform**: Native support for Linux and Windows
- **Configuration Management**: Save and load connection configurations
- **Auto-Save**: Automatically save settings as you type
- **File Browser**: Built-in file picker for payload selection
- **Real-Time Status**: Live feedback on injection progress and results
- **Input Validation**: Comprehensive validation for IP addresses, ports, and files

## 📦 Download & Installation

### For End Users

Download the latest pre-built executables from the [Releases](../../releases) page:

- **Linux**: `ps-payload-injector-linux`
- **Windows**: `ps-payload-injector-windows.exe`

#### Linux Installation

```bash
# Download and make executable
chmod +x ps-payload-injector-linux
./ps-payload-injector-linux
```

#### Windows Installation

1. Download `ps-payload-injector-windows.exe`
2. Double-click to run

> **Windows Security Note**: Windows may show a security warning because the executable is not digitally signed. This is normal for open-source software. Click "More info" → "Run anyway" to proceed.

## 🚀 Quick Start

1. **Launch the application**
2. **Configure target**:
   - Enter target IP address
   - Specify port number
   - Select payload file using "Browse..." button
3. **Save configuration** (optional): Click "Save Config" to store settings
4. **Inject payload**: Click "Inject Payload" to begin transmission
5. **Monitor status**: Watch the status indicator for real-time feedback

## 🎛️ Interface Guide

### Main Controls

| Field          | Description              | Example                |
| -------------- | ------------------------ | ---------------------- |
| **IP Address** | Target server IP address | `192.168.1.100`        |
| **Port**       | Target server port       | `8080`                 |
| **File Path**  | Path to payload file     | `/path/to/payload.bin` |

### Buttons

- **Inject Payload**: Start the payload transmission
- **Save Config**: Save current settings to file
- **Load Config**: Load previously saved configuration
- **Browse...**: Open file picker to select payload file

### Settings

- **Autosave Config**: Automatically save configuration changes

## ⚙️ Configuration

The application supports two types of configuration:

### Auto-Save Configuration

When auto-save is enabled, the application automatically saves your settings to:

- `app_config.json` in the application directory

This file contains all settings including IP, port, file path, and auto-save preference.

### Manual Configuration

- **Save Config**: Opens a file dialog to save configuration to any location
- **Load Config**: Opens a file dialog to load configuration from any location
- Saved configurations are JSON files that can be shared or backed up

## 🛠️ Development

### Prerequisites

- **Rust** 1.70+ ([Install Rust](https://rustup.rs/))
- **Git**

#### Additional Requirements for Cross-Compilation

**For Windows builds on Linux:**

```bash
# Fedora/RHEL
sudo dnf install mingw64-gcc mingw64-gcc-c++

# Ubuntu/Debian
sudo apt install gcc-mingw-w64-x86-64

# Add Windows target
rustup target add x86_64-pc-windows-gnu
```

### Building from Source

#### Clone Repository

```bash
git clone https://github.com/yourusername/ps-payload-injector.git
cd ps-payload-injector
```

#### Development Build

```bash
# Debug build (with console output)
cargo run

# Release build
cargo build --release
```

#### Cross-Platform Build

Use our automated build script:

```bash
./scripts/build.sh
```

This creates optimized executables in the `dist/` folder:

- `ps-payload-injector-linux` (Linux)
- `ps-payload-injector-windows.exe` (Windows)

#### Manual Cross-Compilation

```bash
# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Windows
cargo build --release --target x86_64-pc-windows-gnu
```

### Project Structure

```
ps-payload-injector/
├── src/
│   ├── main.rs          # Application entry point
│   ├── ui.rs            # GUI implementation
│   ├── handlers.rs      # Business logic handlers
│   └── lib.rs           # Library exports
├── scripts/
│   └── build.sh         # Cross-platform build script
├── .cargo/
│   └── config.toml      # Cross-compilation configuration
├── tests/               # Integration tests
├── Cargo.toml           # Rust dependencies and metadata
└── README.md
```

### Dependencies

- **eframe**: GUI framework (egui + native backend)
- **rfd**: Native file dialogs
- **tokio**: Async runtime for network operations
- **serde**: Serialization for configuration files

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## 🔒 Security Considerations

- This tool is designed for legitimate network testing and educational purposes
- Always ensure you have proper authorization before testing on networks you don't own
- The application validates input but users are responsible for payload content
- Network communications are not encrypted by default

## 📋 System Requirements

### Minimum Requirements

- **OS**: Linux (x86_64) or Windows 10+ (x86_64)
- **RAM**: 50MB
- **Disk**: 20MB free space
- **Network**: TCP/IP connectivity

### Supported Platforms

- ✅ Linux x86_64
- ✅ Windows x86_64
- ❌ macOS (not currently supported)
- ❌ ARM architectures (not currently supported)

## 🐛 Troubleshooting

### Common Issues

**"Permission denied" on Linux:**

```bash
chmod +x ps-payload-injector-linux
```

**Windows SmartScreen warning:**

- Click "More info" → "Run anyway"
- Or add to Windows Defender exclusions

**File not found errors:**

- Ensure payload file exists and is accessible
- Check file permissions
- Try absolute file paths

**Network connection issues:**

- Verify target IP and port are correct
- Check firewall settings
- Ensure target service is running

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust naming conventions
- Add tests for new functionality
- Update documentation for API changes
- Ensure cross-platform compatibility
- Run `cargo fmt` and `cargo clippy` before committing

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [egui](https://github.com/emilkagaudi/egui) - Immediate mode GUI framework
- Cross-platform file dialogs by [rfd](https://github.com/PolyMeilex/rfd)
- Async runtime provided by [tokio](https://github.com/tokio-rs/tokio)

## 📞 Support

- **Issues**: [GitHub Issues](../../issues)
- **Discussions**: [GitHub Discussions](../../discussions)
- **Documentation**: [Wiki](../../wiki)

---

**⚠️ Disclaimer**: This tool is for educational and authorized testing purposes only. Users are responsible for ensuring compliance with applicable laws and regulations.
