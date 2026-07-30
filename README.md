# macdaemon - macOS Launchd Daemon & Agent Manager

Một ứng dụng Rust mạnh mẽ dành cho macOS cho phép quản lý các dịch vụ tự khởi động (**LaunchAgents** và **LaunchDaemons**) qua cả giao diện **CLI** và **Web UI**. Hỗ trợ truy cập **Sudo / Privilege Escalation** để cấu hình các lệnh cấp hệ thống (Root System Daemon).

---

## 🌟 Tính Năng Nổi Bật

1. **Quản lý đa cấp độ (Scope):**
   - **User Agent (`~/Library/LaunchAgents`):** Dành cho tài khoản cá nhân.
   - **Global Agent (`/Library/LaunchAgents`):** Dành cho tất cả người dùng.
   - **System Daemon (`/Library/LaunchDaemons`):** Khởi chạy cùng hệ thống (Yêu cầu quyền Root/Sudo).

2. **Truy cập Sudo & Privilege Escalation:**
   - Tự động phát hiện quyền `Root` (`uid == 0`).
   - Nếu chạy ở chế độ User nhưng thao tác vào `SystemDaemon`, tự động dùng `sudo` hoặc hộp thoại xác thực `osascript` (macOS Administrator privileges).

3. **Chế độ kép (CLI + Web UI):**
   - **CLI Mode:** Tiện lợi khi thao tác nhanh qua Terminal hoặc Script.
   - **Web UI Mode:** Giao diện web hiện đại (Dark Glassmorphic UI), hỗ trợ quản lý từ xa (Remote access).

4. **Chỉnh sửa Raw Plist XML & Form Wizard:**
   - Chế độ nhập liệu trực quan: Executable, ProgramArguments, RunAtLoad, KeepAlive, StandardOutPath, StandardErrorPath, WorkingDirectory, StartInterval, EnvironmentVariables.
   - Chế độ **Raw XML Editor**: Sửa trực tiếp file `.plist` với trình kiểm tra cú pháp XML tự động trước khi lưu.

5. **Điều khiển dịch vụ qua `launchctl`:**
   - Hỗ trợ đầy đủ các lệnh: `load` / `unload`, `enable` / `disable`, `start` / `stop`.

6. **Live Log Viewer:**
   - Xem log stdout / stderr trực tiếp trên giao diện Web UI.

---

## 🚀 Hướng Dẫn Cài Đặt & Biên Dịch

### Yêu cầu:
- macOS (Apple Silicon / Intel).
- Rust & Cargo (`rustc` >= 1.70).

### Build ứng dụng:
```bash
cargo build --release
```
File nhị phân sau khi build nằm tại: `./target/release/macdaemon`

---

## 💻 Hướng Dẫn Sử Dụng CLI

### 1. Xem danh sách các dịch vụ:
```bash
# Xem tất cả
macdaemon list

# Lọc theo scope (user, global, system)
macdaemon list --scope system

# Xuất dạng JSON (Dành cho việc tích hợp script)
macdaemon list --json
```

### 2. Xem chi tiết thông tin và nội dung plist:
```bash
macdaemon info com.example.mydaemon --scope user
```

### 3. Đăng ký lệnh tự khởi động mới (Add Service):
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

### 4. Đăng ký lệnh tự khởi chạy cấp hệ thống System Root (LaunchDaemon):
```bash
sudo macdaemon add \
  --label com.system.mydaemon \
  --exec "/usr/local/bin/cloudflared" \
  --args "tunnel,run" \
  --scope system
```

### 5. Khởi chạy / Dừng / Bật / Tắt dịch vụ:
```bash
macdaemon start com.user.myservice
macdaemon stop com.user.myservice
macdaemon load com.user.myservice
macdaemon unload com.user.myservice
```

### 6. Xoá lệnh đã đăng ký:
```bash
macdaemon remove com.user.myservice --scope user
```

### 7. Sửa file Plist trực tiếp dạng Raw XML:
```bash
macdaemon raw-edit com.user.myservice --scope user
```

---

## 🌐 Hướng Dẫn Sử Dụng Web UI (Remote Management)

Để mở giao diện quản lý trên trình duyệt:

```bash
# Chạy Web Server tại localhost:8990 và tự mở trình duyệt
macdaemon web

# Hoặc lắng nghe trên tất cả IP mạng local để truy cập từ xa (Remote access)
macdaemon web --host 0.0.0.0 --port 8990
```

Nếu chạy với quyền `sudo`:
```bash
sudo macdaemon web --host 0.0.0.0 --port 8990
```
Web UI sẽ có đầy đủ quyền thao tác trực tiếp trên tất cả các `LaunchDaemons` của toàn bộ hệ thống!

---

## 🛠 Project Structure

```
rust-macos-daemon/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs            # Entrypoint (CLI / Web UI dispatcher)
    ├── cli.rs             # CLI argument parsing (clap) & commands logic
    ├── model.rs           # Data structures, Plist models, Raw XML validator
    ├── privilege.rs       # Sudo & Administrative privilege escalation helper
    ├── launchd.rs         # macOS launchctl wrapper & directory scanner
    └── web/
        ├── mod.rs         # Axum server setup & route definitions
        ├── api.rs         # REST API endpoints
        └── static_assets.rs # Embedded Glassmorphism SPA Web UI (HTML/CSS/JS)
```

---

## 📜 License
MIT License
