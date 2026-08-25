# macdaemon - macOS Launchd Daemon & Agent Manager

[![CI](https://github.com/manhavn/rust-macos-daemon/actions/workflows/ci.yml/badge.svg)](https://github.com/manhavn/rust-macos-daemon/actions/workflows/ci.yml)
[![Release](https://github.com/manhavn/rust-macos-daemon/actions/workflows/release.yml/badge.svg)](https://github.com/manhavn/rust-macos-daemon/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-macOS%20(Apple%20Silicon%20%26%20Intel)-lightgrey.svg)](https://apple.com/macos)
[![Language](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)

> 🌐 Language: **English** | [Tiếng Việt (Vietnamese README)](README_VI.md)

A high-performance, feature-rich Rust application for macOS that allows managing system auto-start daemons and user agents (**LaunchAgents** and **LaunchDaemons**) through both **CLI** and **Web UI** interfaces. Features built-in **Sudo / Privilege Escalation** to configure system-level root daemons, arbitrary raw file editing, file tools, safety countdown modals, and automatic pre-backup downloads.

---

## 🖥️ macOS Hardware & Architecture Compatibility

`macdaemon` provides native precompiled binaries and universal binaries supporting all popular current macOS hardware:

- 🍏 **Apple Silicon (M1 / M2 / M3 / M4 / M5, Pro, Max, Ultra):** Native `aarch64-apple-darwin` build for maximum performance and power efficiency.
- 💻 **Intel Macs (Core i5, Core i7, Core i9, Xeon):** Native `x86_64-apple-darwin` build supporting all Intel-based MacBooks, iMacs, Mac minis, and Mac Pros.
- 🌐 **Universal 2 macOS Binary:** Single fat binary (`universal-apple-darwin`) containing both ARM64 and x86_64 native code slices, running natively on all macOS machines without requiring Rosetta 2.

---

## 🚀 Installation

### 1. Via [mise](https://mise.jdx.dev/) (Recommended)

You can install precompiled binaries directly from GitHub Releases using `mise`:

```bash
# Install globally
mise use -g github:manhavn/rust-macos-daemon

# Or install for the current directory/project
mise use github:manhavn/rust-macos-daemon

# Or run directly on the fly without installing
mise x github:manhavn/rust-macos-daemon -- macdaemon list
mise x github:manhavn/rust-macos-daemon -- macdaemon web
```

Or add it to your `mise.toml` configuration:

```toml
[tools]
"github:manhavn/rust-macos-daemon" = "latest"
```

### 2. Via Cargo

If you have Rust and Cargo installed:

```bash
# Install directly from GitHub
cargo install --git https://github.com/manhavn/rust-macos-daemon.git

# Or install via mise using the cargo backend
mise use -g cargo:manhavn/rust-macos-daemon
```

### 3. Build from Source

```bash
# Clone the repository
git clone https://github.com/manhavn/rust-macos-daemon.git
cd rust-macos-daemon

# Build in release mode
cargo build --release

# Copy binary to your PATH
sudo cp target/release/macdaemon /usr/local/bin/
```

---

## 🌟 Key Features

1. **Multi-Scope Management:**
   - **User Agent (`~/Library/LaunchAgents`):** Personal user account agents (UID level).
   - **Global Agent (`/Library/LaunchAgents`):** Agents shared across all system users.
   - **System Daemon (`/Library/LaunchDaemons`):** Root system services starting at boot (Requires Root/Sudo).

2. **Privilege Escalation & Sudo Integration:**
   - Auto-detects root privileges (`uid == 0`).
   - If running in User Mode, operations targeting `SystemDaemon` automatically elevate privileges using `sudo` or native macOS Administrator authentication dialogs (`osascript`).

3. **Dual Mode (CLI + Remote Web UI):**
   - **CLI Mode:** Fast terminal execution and scripting integration.
   - **Web UI Mode:** Modern Dark Glassmorphism Single-Page Application (SPA) designed for remote management over local network or server environments.

4. **Form Wizard & Raw Plist XML Editors:**
   - **Form Editor:** Visual configuration for Executable path, Arguments, RunAtLoad, KeepAlive, StandardOutPath, StandardErrorPath, WorkingDirectory, and StartInterval.
   - **Raw Plist XML Editor:** Direct `.plist` XML editing with strict DTD validation before saving.

5. **Raw Arbitrary File Editor Mode:**
   - Read and edit any absolute file path on macOS directly in the Web UI (with optional Sudo elevation).

6. **File & Directory Management Tools:**
   - **Permissions & Ownership (`Chown` / `Chmod`):** Auto-fetches and pre-fills current ownership (`owner:group`) and octal mode (`chmod`) when loading path info.
   - **Copy (`cp -R`):** Copy files or directories with recursive & Sudo options.
   - **Move (`mv`):** Move or rename files and directories.
   - **Delete (`rm -rf`):** Safely delete files or folders.

7. **Safety Countdown Confirmation Modals:**
   - All submit and destructive operations trigger a compact confirmation modal with enforced countdown delays (**3 seconds** for User Mode, **5 seconds** for Sudo/Root Mode) before enabling the confirm button.

8. **Automatic Pre-Backup Downloads:**
   - Automatically triggers a browser file download of the original file content before performing file content overwrites or path deletions.
   - **Filename Format:** `(sanitized-directory-timestamp-filename)` with all slashes and special characters converted to hyphens.

9. **macOS System Notifications:**
   - Triggers native macOS Notification Center alerts when the server starts.

---

## 💻 CLI Usage Guide

### 1. List Services:
```bash
# List all services
macdaemon list

# Filter by scope (user, global, system)
macdaemon list --scope system

# Output JSON format for scripts
macdaemon list --json
```

### 2. View Service Detail & Plist XML:
```bash
macdaemon info com.example.mydaemon --scope user
```

### 3. Register a New User Auto-Start Service:
```bash
macdaemon add \
  --label com.user.myservice \
  --exec "/usr/local/bin/node" \
  --args "/app/server.js,--port,8080" \
  --scope user \
  --run-at-load true \
  --keep-alive true \
  --stdout "/tmp/myservice.stdout.log" \
  --stderr "/tmp/myservice.stderr.log"
```

### 4. Register a System Root LaunchDaemon:
```bash
sudo macdaemon add \
  --label com.system.mydaemon \
  --exec "/usr/local/bin/cloudflared" \
  --args "tunnel,run" \
  --scope system
```

### 5. Service Control Operations:
```bash
macdaemon start com.user.myservice
macdaemon stop com.user.myservice
macdaemon load com.user.myservice
macdaemon unload com.user.myservice
```

### 6. Remove Service:
```bash
macdaemon remove com.user.myservice --scope user
```

---

## 🌐 Web UI Usage (Remote Management)

Start the embedded Web UI server:

```bash
# Start server bound to 127.0.0.1:8990
macdaemon web

# Listen on all network interfaces for remote management
macdaemon web --host 0.0.0.0 --port 8990

# Run with Root privileges for full system control
sudo macdaemon web --host 0.0.0.0 --port 8990
```

---

## 🛠 Project Structure

```
rust-macos-daemon/
├── .github/
│   └── workflows/
│       ├── ci.yml             # CI: Formatting, Clippy, and Tests
│       └── release.yml        # Multi-architecture & Universal macOS Release builds
├── Cargo.toml
├── LICENSE                    # MIT License
├── README.md                  # Main Documentation (English)
├── README_VI.md               # Documentation in Vietnamese
└── src/
    ├── main.rs                # Entrypoint (CLI / Web UI dispatcher)
    ├── cli.rs                 # CLI argument parsing (clap) & commands logic
    ├── model.rs               # Data structures, Plist models, Raw XML validator
    ├── privilege.rs           # Sudo & Administrative privilege escalation helper
    ├── launchd.rs             # macOS launchctl wrapper & directory scanner
    └── web/
        ├── mod.rs             # Axum server setup & route definitions
        ├── api.rs             # REST API endpoints (Service CRUD, FS tools, Logs)
        └── static_assets.rs   # Embedded Dark Glassmorphic SPA (HTML/CSS/JS)
```

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).
