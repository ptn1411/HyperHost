<div align="center">
  <img src="https://raw.githubusercontent.com/ptn1411/HyperHost/refs/heads/main/src-tauri/icons/icon.png" width="100" />
  <h1>HyperHost</h1>
  <p><strong>Professional Local Virtual Domain & HTTPS Manager</strong></p>
  <p>
    <a href="../../releases/latest"><img src="https://img.shields.io/github/v/release/ptn1411/HyperHost?style=flat-square&color=blue" alt="Release" /></a>
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-0078d4?style=flat-square" alt="Cross-platform" />
    <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="MIT" />
  </p>
</div>

---

**HyperHost** là công cụ mã nguồn mở dành cho lập trình viên trên **Windows, macOS và Linux**. Hoạt động như một bảng điều khiển Nginx siêu tốc, tự động định tuyến tên miền ảo (ví dụ: `my-project.test`, `api.local`) tới các cổng `localhost` đang chạy với **chứng chỉ HTTPS hợp lệ**.

Không còn sửa file `hosts` bằng tay, không còn chứng chỉ Self-Signed lỗi đỏ trên trình duyệt. Chỉ 1 click là có ngay tên miền HTTPS chuyên nghiệp ngay trên máy cục bộ.

---

## 🚀 Tính năng nổi bật

### Cốt lõi
- **⚡ Quản lý tên miền tức thì** — Giao diện React mượt mà. Tạo domain và liên kết tới upstream (VD: `localhost:3000`) chỉ trong vài giây.
- **🔒 HTTPS tự động (365 ngày)** — CA nội bộ tự cấp chứng chỉ xanh lá hợp lệ. Hạn sử dụng 1 năm (browser-stable), tự động gia hạn khi gần hết hạn lúc khởi động app.
- **💻 Monaco Editor tích hợp** — Lõi editor của VSCode. Viết raw Nginx config tùy biến với syntax highlight và auto-format.
- **👻 System Tray** — Tắt cửa sổ mà nginx vẫn chạy ngầm, ẩn vào System Tray.
- **🔄 Auto-Updater** — Tự động thông báo và cài bản mới từ GitHub Releases.

### Mới trong v0.2.1
- **🌐 Cross-platform** — Hỗ trợ đầy đủ Windows, macOS (Homebrew nginx) và Linux. CA install/status hoạt động đúng trên mọi nền tảng.
- **🔀 CORS Toggle** — Bật/tắt CORS headers per-domain trực tiếp từ UI. Hỗ trợ `GET/POST/PUT/DELETE/PATCH/OPTIONS` và pre-flight requests.
- **📤 Export / Import domain** — Sao lưu toàn bộ cấu hình domain ra file JSON. Khôi phục sang máy khác hoặc sau khi cài lại hệ thống, chứng chỉ mới được tự động cấp lại.
- **📊 Traffic Stats** — Theo dõi số request và latency trung bình per-domain theo thời gian thực qua nginx access log.
- **🔑 CA Fingerprint** — Hiển thị SHA-256 fingerprint của CA certificate trong UI và CLI để xác minh tính toàn vẹn.
- **✅ nginx -t validation** — Tự động kiểm tra config bằng `nginx -t` trước khi reload, tránh downtime do config lỗi.
- **⚠️ Tunnel error handling** — Phát hiện và hiển thị lỗi khi Cloudflare Tunnel không khởi động được thay vì treo vô thời hạn.

---

## 📦 Công nghệ sử dụng

- **Backend**: [Rust](https://www.rust-lang.org/) & [Tauri v2](https://v2.tauri.app/)
- **Frontend**: [React 19](https://react.dev/), [Vite](https://vitejs.dev/), [TailwindCSS v4](https://tailwindcss.com/)
- **Core Systems**: `nginx` (Proxy), `mkcert` (HTTPS/NSS), `cloudflared` (Public Tunnel), SQLite (Database)
- **CLI**: `clap` (arg parser), `comfy-table` (table output), `rcgen` + `sha2` (cert issuance & fingerprint)

---

## 📥 Tải xuống & Cài đặt

Truy cập mục **[Releases](../../releases/latest)** và tải bản phù hợp với hệ điều hành của bạn:

| Nền tảng | File |
|----------|------|
| Windows x64 | `HyperHost_x64-setup.exe` |
| macOS Apple Silicon | `HyperHost_aarch64.dmg` |
| macOS Intel | `HyperHost_x64.dmg` |
| Linux x64 | `HyperHost_amd64.deb` / `.AppImage` |

### Yêu cầu bổ sung theo nền tảng

**Windows** — nginx bundled sẵn, không cần cài thêm.

**macOS**:
```bash
brew install nginx
```

**Linux (Debian/Ubuntu)**:
```bash
sudo apt install nginx
```

---

## ⌨️ HyperHost CLI (`hyh`)

Ngoài GUI, HyperHost đi kèm công cụ dòng lệnh **`hyh`** để quản lý toàn bộ hệ thống từ Terminal mà không cần mở ứng dụng chính.

> **Lưu ý:** CLI cần chạy với quyền **Administrator/sudo** vì phải ghi file `hosts` và quản lý nginx.

### Cài đặt CLI

Binary `hyh` được đóng gói sẵn trong installer. Nếu tự build từ source:

```bash
cd src-tauri

# Linux / macOS
cargo build --release --bin hyh --no-default-features
# Output: target/release/hyh

# Windows
cargo build --release --bin hyh --no-default-features
# Output: target/release/hyh.exe
```

### Tổng quan các lệnh

```
hyh <COMMAND> [OPTIONS]

COMMANDS:
  add       Thêm tên miền mới với HTTPS
  remove    Xóa một tên miền
  list      Liệt kê tất cả tên miền đã cấu hình
  toggle    Bật/tắt một tên miền
  nginx     Quản lý Nginx proxy
  ca        Quản lý Certificate Authority (CA)
  help      Hiển thị trợ giúp
```

---

### `hyh add` — Thêm tên miền

Tạo domain cục bộ mới kèm chứng chỉ HTTPS 365 ngày, tự động cập nhật file `hosts` và cấu hình Nginx.

```bash
hyh add <DOMAIN> <UPSTREAM>
```

| Tham số | Mô tả | Ví dụ |
|---------|-------|-------|
| `<DOMAIN>` | Tên miền cục bộ (kết thúc `.test` hoặc `.local`) | `myapp.test` |
| `<UPSTREAM>` | Địa chỉ server upstream | `http://127.0.0.1:3000` |

**Ví dụ:**

```bash
hyh add myapp.test http://127.0.0.1:3000
hyh add api.test http://127.0.0.1:8080
hyh add laravel.local http://127.0.0.1:8000
```

**Kết quả mẫu:**

```
🔐 Issuing certificate for myapp.test...
  ✓ Certificate issued (rcgen)
📦 Saved to database
📝 Hosts file updated
🔄 nginx config regenerated

✅ https://myapp.test → http://127.0.0.1:3000
```

---

### `hyh remove` — Xóa tên miền

```bash
hyh remove myapp.test
# ✅ Removed myapp.test
```

---

### `hyh list` — Liệt kê tên miền

```bash
hyh list
```

```
╭────────┬──────────────────┬────────────────────────────────┬───────────╮
│ Status │ Domain           │ Upstream                       │ Cert      │
├────────┼──────────────────┼────────────────────────────────┼───────────┤
│ 🟢     │ myapp.test       │ http://127.0.0.1:3000          │ ✓ valid   │
│ 🟢     │ api.test         │ http://127.0.0.1:8080          │ ✓ valid   │
│ ⚫     │ old-project.test │ http://127.0.0.1:4000          │ ✗ expired │
╰────────┴──────────────────┴────────────────────────────────┴───────────╯

  3 domain(s) total
```

---

### `hyh toggle` — Bật/tắt tên miền

```bash
hyh toggle old-project.test
# ✅ old-project.test → ⚫ disabled

hyh toggle old-project.test
# ✅ old-project.test → 🟢 enabled
```

---

### `hyh nginx` — Quản lý Nginx

```bash
hyh nginx start    # Khởi động nginx
hyh nginx stop     # Dừng nginx
hyh nginx reload   # Reload config (có nginx -t kiểm tra trước)
hyh nginx status   # Kiểm tra trạng thái
hyh nginx logs     # Xem error log (mặc định 20 dòng)
hyh nginx logs -n 50  # Xem 50 dòng gần nhất
```

```
# Ví dụ kết quả
nginx: 🟢 running
✅ nginx reloaded
```

---

### `hyh ca` — Quản lý Certificate Authority

Hỗ trợ Windows (certutil), macOS (security), và Linux (update-ca-certificates).

```bash
hyh ca install   # Cài CA vào system trust store
hyh ca status    # Kiểm tra trạng thái CA + SHA-256 fingerprint
```

**Kết quả mẫu:**

```bash
$ hyh ca install
# ✓ mkcert: Firefox NSS trusted
# ✅ CA installed successfully

$ hyh ca status
# CA: 🟢 installed & trusted
# SHA-256: 4A:F2:1B:...:9C:3D
```

---

### Quy trình sử dụng mẫu

```bash
# Bước 1: Cài CA vào hệ thống (chỉ cần làm 1 lần)
hyh ca install

# Bước 2: Khởi động nginx
hyh nginx start

# Bước 3: Thêm domain cho React dev server
hyh add myapp.test http://127.0.0.1:3000

# Bước 4: Mở trình duyệt → https://myapp.test  ✅ HTTPS xanh lá!

# Bước 5: Xem danh sách
hyh list

# Bước 6: Tắt domain khi không cần
hyh toggle myapp.test
```

---

## 🌐 Public Tunnel (Cloudflare)

Chia sẻ trang web cục bộ ra Internet qua Cloudflare Tunnel — không cần cấu hình Router, không cần mua domain.

### Cách sử dụng

1. Mở HyperHost GUI.
2. Nhấn nút **🌐** cạnh domain bất kỳ.
3. Chờ vài giây — URL dạng `https://xxx-yyy.trycloudflare.com` xuất hiện.
4. Gửi link cho bất kỳ ai.

> **v0.2.1**: Nếu tunnel không kết nối được, app hiển thị thông báo lỗi rõ ràng thay vì treo vô thời hạn.

### Lưu ý cho người dùng tại Việt Nam

Một số nhà mạng (VNPT, Viettel, FPT) chặn DNS `*.trycloudflare.com`. Nếu gặp lỗi `DNS_PROBE_FINISHED_NXDOMAIN`:

**Đổi DNS hệ thống (Windows PowerShell Admin):**

```powershell
netsh interface ip set dns "Wi-Fi" static 1.1.1.1
netsh interface ip add dns "Wi-Fi" 8.8.8.8 index=2
ipconfig /flushdns
```

**Bật Secure DNS trên trình duyệt:**

1. Chrome: `chrome://settings/security` → bật **Use secure DNS** → chọn Cloudflare.
2. Edge: `edge://settings/privacy` → bật **Use secure DNS**.

---

## 🛠 Hướng dẫn cho Developer (Tự Build)

### Yêu cầu hệ thống

| | Windows | macOS | Linux |
|-|---------|-------|-------|
| Runtime | Node.js 20+, Rust, C++ Build Tools | Node.js 20+, Rust, Xcode CLI | Node.js 20+, Rust, `libgtk-3-dev` |
| nginx | bundled | `brew install nginx` | `sudo apt install nginx` |

### Các bước

```bash
# 1. Clone
git clone https://github.com/ptn1411/HyperHost.git
cd HyperHost

# 2. Cài frontend deps
npm install

# 3. Dev mode (cần quyền admin/sudo)
npm run tauri dev

# 4. Build CLI riêng (không cần GUI/GTK)
cd src-tauri
cargo build --release --bin hyh --no-default-features

# 5. Build toàn bộ app
npm run tauri build
```

> **Lưu ý:** Private key của Updater không được commit vào repo. Nếu tự build và muốn dùng Auto-Updater, hãy tạo key riêng bằng `tauri signer generate`.

---

## 📄 Giấy phép

Dự án mở mã nguồn theo giấy phép MIT. Tự do sử dụng, chỉnh sửa và phân phối.
