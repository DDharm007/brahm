# BRAHM Installation & Build Guide

This document details the build prerequisites, environment configuration, and verification steps for **BRAHM Stage 01**.

---

## 1. Prerequisites

### Operating System
- **Windows 10 / 11** (x86_64), Linux, or macOS.

### Toolchain Components
- **Rust Stable** (version 1.80.0 or later).
- **Cargo** package manager.
- **Linker & Runtime**:
  - Windows: Either Microsoft C++ Build Tools (`link.exe` via Visual Studio) or MinGW-w64 (`gcc` / `ld` / `WinLibs`).
  - Linux: `build-essential` (`gcc`, `ld`).
  - macOS: Xcode Command Line Tools.

---

## 2. Windows 11 Configuration Notice

If developing on Windows 11:
1. **Developer Mode**: Enabled in **Settings → System → For developers → Developer Mode (On)** to allow Cargo to execute build scripts and test binaries locally.
2. **Smart App Control**: If Smart App Control is active, ensure it is set to **Off** or **Evaluation** in **Windows Security → App & browser control → Smart App Control**.

---

## 3. Cloning & Building from Source

```powershell
# Clone the repository
git clone https://github.com/DDharm007/brahm.git
cd brahm

# Check compilation without producing binaries
cargo check

# Run the complete automated test suite
cargo test

# Build optimized release binary
cargo build --release
```

The resulting release binary will be available at `target/release/brahm.exe` (Windows) or `target/release/brahm` (Unix).

---

## 4. Verification & Testing

To execute formatting, linting, and security regression checks:

```powershell
# Check code formatting
cargo fmt --check

# Strict Clippy lint verification
cargo clippy --all-targets --all-features -- -D warnings

# Execute all integration and fixture tests
cargo test
```

---

## 5. Basic Usage Verification

```powershell
# Display version and CLI help
./target/release/brahm --version
./target/release/brahm --help

# Initialize configuration
./target/release/brahm init

# Analyze a repository
./target/release/brahm scan ./fixtures/fixture_c_multilingual --format all --output ./reports
```
