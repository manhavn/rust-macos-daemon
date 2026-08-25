# macdaemon - macOS Launchd Daemon & Agent Manager

[![CI](https://github.com/manhavn/rust-macos-daemon/actions/workflows/ci.yml/badge.svg)](https://github.com/manhavn/rust-macos-daemon/actions/workflows/ci.yml)
[![Release](https://github.com/manhavn/rust-macos-daemon/actions/workflows/release.yml/badge.svg)](https://github.com/manhavn/rust-macos-daemon/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-macOS%20(Apple%20Silicon%20%26%20Intel)-lightgrey.svg)](https://apple.com/macos)
[![Language](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)

> 🌐 Language: **Tiếng Việt** | [English README](README.md)

Một ứng dụng Rust mạnh mẽ dành cho macOS cho phép quản lý các dịch vụ tự khởi động (**LaunchAgents** và **LaunchDaemons**) qua cả giao diện **CLI** và **Web UI**. Hỗ trợ truy cập **Sudo / Privilege Escalation** để cấu hình các lệnh cấp hệ thống (Root System Daemon), chỉnh sửa raw file bất kỳ, bộ công cụ quản lý file, popup xác nhận đếm ngược an toàn và tự động tải file backup trước khi thao tác.

---

## 🖥️ Tương Thích Phần Cứng macOS

`macdaemon` hỗ trợ đầy đủ các dòng máy Mac phổ biến hiện nay thông qua bản dựng precompiled native và Universal Binary:

- 🍏 **Apple Silicon (M1 / M2 / M3 / M4 / M5, Pro, Max, Ultra):** Bản dựng Native `aarch64-apple-darwin` tối ưu hóa hiệu năng và tiết kiệm pin.
- 💻 **Intel Macs (Core i5, Core i7, Core i9, Xeon):** Bản dựng Native `x86_64-apple-darwin` tương thích toàn diện với tất cả dòng MacBook, iMac, Mac mini, Mac Pro chạy chip Intel.
- 🌐 **Universal 2 macOS Binary:** Bản nhị phân gộp (`universal-apple-darwin`) chứa cả mã máy ARM64 và x86_64, chạy trực tiếp trên mọi máy Mac mà không cần thông qua Rosetta 2.

---

## 🚀 Hướng Dẫn Cài Đặt

### 1. Cài đặt qua [mise](https://mise.jdx.dev/) (Khuyên dùng)

Bạn có thể cài đặt trực tiếp bản build nhị phân từ GitHub Releases qua `mise`:

```bash
# Cài đặt global vào hệ thống
mise use -g github:manhavn/rust-macos-daemon

# Hoặc cài đặt riêng cho thư mục/project hiện tại
mise use github:manhavn/rust-macos-daemon

# Hoặc chạy trực tiếp tức thì không cần cài đặt
mise x github:manhavn/rust-macos-daemon -- macdaemon list
mise x github:manhavn/rust-macos-daemon -- macdaemon web
```

Hoặc thêm trực tiếp vào cấu hình `mise.toml`:

```toml
[tools]
"github:manhavn/rust-macos-daemon" = "latest"
```

### 2. Cài đặt qua Cargo

Nếu máy bạn đã có môi trường Rust & Cargo:

```bash
# Cài đặt trực tiếp từ kho GitHub
cargo install --git https://github.com/manhavn/rust-macos-daemon.git

# Hoặc dùng mise qua cargo backend
mise use -g cargo:manhavn/rust-macos-daemon
```

### 3. Tự biên dịch từ mã nguồn (Build from Source)

```bash
# Clone repository
git clone https://github.com/manhavn/rust-macos-daemon.git
cd rust-macos-daemon

# Build ở chế độ release
cargo build --release

# Copy file thực thi vào PATH hệ thống
sudo cp target/release/macdaemon /usr/local/bin/
```

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
   - Chế độ nhập liệu trực quan: Executable, ProgramArguments, RunAtLoad, KeepAlive, StandardOutPath, StandardErrorPath, WorkingDirectory, StartInterval.
   - Chế độ **Raw XML Editor**: Sửa trực tiếp file `.plist` với trình kiểm tra cú pháp XML tự động trước khi lưu.

5. **Trình Chỉnh Sửa File Bất Kỳ (Raw File Editor):**
   - Nhập đường dẫn tuyệt đối bất kỳ để đọc và sửa file trực tiếp trên giao diện Web UI (hỗ trợ lưu với quyền Root Sudo).

6. **Bộ Công Cụ Quản Lý File & Thư Mục (File Tools):**
   - **Phân quyền & Chủ sở hữu (`Chown` / `Chmod`):** Tự động đọc và điền sẵn `chown` & `chmod` octal hiện tại khi bấm `Load Info`.
   - **Sao chép (`Copy`):** Sao chép file/thư mục (`cp -R`).
   - **Di chuyển (`Move`):** Di chuyển file/thư mục (`mv`).
   - **Xoá (`Delete`):** Xoá an toàn file/thư mục (`rm -rf`).

7. **Hộp Thoại Xác Nhận Đếm Ngược An Toàn (3s / 5s Countdown Modal):**
   - Tất cả các thao tác submit nguy hiểm đều trải qua Popup đếm ngược: **3 giây** cho User mode và **5 giây** cho Sudo/Root mode trước khi kích hoạt nút Confirm.

8. **Tự Động Tải File Backup Nguồn:**
   - Tự động kích hoạt tải file Backup về máy trước khi ghi đè nội dung file hoặc xoá file/thư mục (`duong-dan-thu-muc-thoi-gian-ten-file`).

9. **Thông Báo Hệ Thống macOS Native Notification:**
   - Phát thông báo macOS Notification Center khi ứng dụng và Web server khởi chạy.

---

## 💻 Hướng Dẫn Sử Dụng CLI

### 1. Xem danh sách các dịch vụ:
```bash
# Xem toàn bộ dịch vụ
macdaemon list

# Lọc theo phạm vi (user, global, system)
macdaemon list --scope system

# Xuất dữ liệu JSON cho script / pipeline
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

### 5. Điều khiển dịch vụ:
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

---

## 🌐 Hướng Dẫn Sử Dụng Web UI (Remote Management)

```bash
# Chạy Web Server tại localhost:8990
macdaemon web

# Chạy với IP mạng local để truy cập từ xa (Remote access)
macdaemon web --host 0.0.0.0 --port 8990

# Chạy với quyền Root
sudo macdaemon web --host 0.0.0.0 --port 8990
```

---

## 🛠 Cấu Trúc Dự Án

```
rust-macos-daemon/
├── .github/
│   └── workflows/
│       ├── ci.yml             # CI: Format, Clippy và Unit Test
│       └── release.yml        # Build tự động đa nền tảng và Universal macOS
├── Cargo.toml
├── LICENSE                    # Giấy phép MIT
├── README.md                  # Tài liệu Tiếng Anh (Chính)
├── README_VI.md               # Tài liệu Tiếng Việt
└── src/
    ├── main.rs                # Điểm khởi chạy (CLI / Web UI dispatcher)
    ├── cli.rs                 # Xử lý tham số dòng lệnh (clap) & commands logic
    ├── model.rs               # Cấu trúc dữ liệu, Plist models, Trình kiểm tra cú pháp XML
    ├── privilege.rs           # Module leo thang đặc quyền Sudo & macOS Admin
    ├── launchd.rs             # Wrapper tương tác macOS launchctl & quét thư mục
    └── web/
        ├── mod.rs             # Cấu hình Axum server & định tuyến routes
        ├── api.rs             # REST API endpoints (Service CRUD, Quản lý File, Logs)
        └── static_assets.rs   # Giao diện Web SPA Dark Glassmorphic (HTML/CSS/JS)
```

---

## 📜 License

Dự án được phát hành dưới giấy phép [MIT License](LICENSE).
