<div align="center">
  <img src="https://raw.githubusercontent.com/ptn1411/HyperHost/refs/heads/main/src-tauri/icons/icon.png" width="100" />
  <h1>HyperHost</h1>
  <p><strong>Professional Local Virtual Domain & HTTPS Manager for Windows</strong></p>
  <p>
    <a href="../../releases/latest"><img src="https://img.shields.io/github/v/release/ptn1411/HyperHost?style=flat-square&color=blue" alt="Release" /></a>
    <img src="https://img.shields.io/badge/platform-Windows-0078d4?style=flat-square" alt="Windows" />
    <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="MIT" />
  </p>
</div>

---

**HyperHost** là một công cụ mã nguồn mở dành riêng cho các lập trình viên trên hệ điều hành Windows. Công cụ này đóng vai trò như một bảng điều khiển Nginx siêu tốc, giúp bạn tự động định tuyến tên miền ảo (ví dụ: `my-project.test` hoặc `api.local`) tới các cổng `localhost` đang chạy với **đầy đủ chứng chỉ HTTPS (SSL) hợp lệ**.

Không còn cảnh phải sửa file `hosts` bằng tay, không còn phải loay hoay tạo chứng chỉ Self-Signed lỗi xanh lỗi đỏ trên trình duyệt. Chỉ cần 1 click là bạn có ngay một tên miền chuyên nghiệp ngay dưới máy tính nội bộ của mình!

## 🚀 Tính năng nổi bật

- **⚡ Quản lý tên miền tức thì**: Giao diện UI/UX siêu mượt xây dựng bằng React. Tạo nhanh một tên miền và liên kết tới một cổng Upstream (VD: `localhost:3000`) chỉ trong 3 giây.
- **🔒 HTTPS (SSL) tự động**: Tích hợp cực sâu công cụ chuyên dụng `mkcert`. Tự động đứng ra làm Tổ chức chứng thực (Certificate Authority - CA) của máy tính, cấp chứng chỉ xanh lá cây hợp lệ cho mọi tên miền ảo.
- **💻 Text Editor cấp độ Pro**: Tích hợp lõi _Monaco Editor_ (công nghệ đằng sau VSCode). Bạn có thể viết và chèn cấu hình Raw Nginx Server blocks/Directives tùy biến với Auto-format cực kì mạnh mẽ.
- **👻 Chế độ Background / System Tray**: Hoạt động âm thầm không làm phiền bạn. Khi bấm dấu 'X' để tắt cửa sổ, HyperHost tự giấu mình vào System Tray dưới góc phải màn hình trong khi Nginx vẫn hoạt động mượt mà.
- **🔄 Tự động Cập Nhật (Auto-Updater)**: Khi nhà phát triển ra mắt phiên bản mới trên GitHub Releases, HyperHost sẽ tự động mở hộp thoại thông báo tải xuống & cài đặt bản mới trực tiếp nhờ lõi Tauri Updater.
- **🌐 Public Tunnel (Cloudflare)**: Chia sẻ trang web của bạn ra ngoài Internet qua Cloudflare Tunnel. Nhấn 1 nút và nhận đường link `*.trycloudflare.com` tạm thời.
- **📊 Live Traffic Inspector**: Theo dõi lưu lượng HTTP đi qua Nginx theo thời gian thực với giao diện dạng bảng trực quan.

## 📦 Công nghệ sử dụng

- **Backend**: [Rust](https://www.rust-lang.org/) & [Tauri v2](https://v2.tauri.app/).
- **Frontend**: [React 18](https://react.dev/), [Vite](https://vitejs.dev/), và [TailwindCSS v4](https://tailwindcss.com/).
- **Core Systems**: `nginx` (Máy chủ Proxy), `mkcert` (Quản lý HTTPS), `cloudflared` (Public Tunnel), SQLite (Dữ liệu).

## 📥 Tải xuống & Cài đặt

Ứng dụng HyperHost hiện cung cấp bộ cài `.exe` thông minh tự động (NSIS Installer) thông qua GitHub Actions CI/CD.

Bạn hãy truy cập vào mục **[Releases](../../releases/latest)** của kho lưu trữ này và tải về file `HyperHost_..._x64-setup.exe`. Trình cài đặt sẽ tự động lưu vào `C:\Program Files\HyperHost` và đổ biểu tượng ra màn hình Destkop cho bạn.

---

## ⌨️ HyperHost CLI (`hyh`)

Ngoài giao diện đồ hoạ (GUI), HyperHost còn đi kèm công cụ dòng lệnh **`hyh`** cho phép bạn quản lý toàn bộ hệ thống ngay từ Terminal/PowerShell mà không cần mở ứng dụng chính.

> **Lưu ý:** CLI cần chạy dưới quyền **Administrator** vì phải ghi file `hosts` hệ thống và quản lý tiến trình Nginx.

### Cài đặt CLI

CLI đã được tự động cài sẵn khi bạn cài HyperHost qua Installer. Binary `hyh.exe` nằm trong thư mục cài đặt và đã được đăng ký vào `PATH` hệ thống.

Nếu tự build từ source:

```bash
cd src-tauri
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

Tạo một tên miền cục bộ mới kèm chứng chỉ HTTPS, tự động cập nhật file `hosts` và cấu hình Nginx.

```bash
hyh add <DOMAIN> <UPSTREAM>
```

| Tham số      | Mô tả                                          | Ví dụ                       |
| ------------ | ----------------------------------------------- | --------------------------- |
| `<DOMAIN>`   | Tên miền cục bộ (bắt buộc kết thúc `.test` hoặc `.local`) | `myapp.test`                |
| `<UPSTREAM>` | Địa chỉ server upstream kèm protocol            | `http://127.0.0.1:3000`     |

**Ví dụ mẫu:**

```bash
# Tạo domain đơn giản trỏ về React dev server
hyh add myapp.test http://127.0.0.1:3000

# Tạo domain cho API backend đang chạy ở port 8080
hyh add api.test http://127.0.0.1:8080

# Tạo domain cho Laravel (Herd/Valet)
hyh add laravel.local http://127.0.0.1:8000

# Tạo domain cho Next.js
hyh add shop.test https://127.0.0.1:3001
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

Xóa tên miền khỏi hệ thống, bao gồm chứng chỉ SSL, bản ghi hosts, và cấu hình Nginx.

```bash
hyh remove <DOMAIN>
```

**Ví dụ:**

```bash
hyh remove myapp.test
# ✅ Removed myapp.test
```

---

### `hyh list` — Liệt kê tên miền

Hiển thị bảng tất cả tên miền đã cấu hình kèm trạng thái hoạt động và chứng chỉ SSL.

```bash
hyh list
```

**Kết quả mẫu:**

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

Chuyển đổi trạng thái hoạt động của tên miền. Khi tắt, Nginx sẽ không proxy cho domain đó nữa nhưng cấu hình vẫn được giữ nguyên.

```bash
hyh toggle <DOMAIN>
```

**Ví dụ:**

```bash
# Tắt tạm thời một domain
hyh toggle old-project.test
# ✅ old-project.test → ⚫ disabled

# Bật lại
hyh toggle old-project.test
# ✅ old-project.test → 🟢 enabled
```

---

### `hyh nginx` — Quản lý Nginx

Điều khiển tiến trình Nginx proxy server.

```bash
hyh nginx <ACTION>

ACTIONS:
  start    Khởi động Nginx
  stop     Dừng Nginx
  reload   Tải lại cấu hình (không cần restart)
  status   Kiểm tra trạng thái Nginx
  logs     Xem log lỗi gần nhất
```

**Ví dụ mẫu:**

```bash
# Khởi động Nginx
hyh nginx start
# ✅ nginx started

# Kiểm tra trạng thái
hyh nginx status
# nginx: 🟢 running

# Tải lại sau khi sửa cấu hình
hyh nginx reload
# ✅ nginx reloaded

# Xem 50 dòng log lỗi gần nhất
hyh nginx logs -n 50

# Dừng Nginx
hyh nginx stop
# ✅ nginx stopped
```

---

### `hyh ca` — Quản lý Certificate Authority

Quản lý CA nội bộ dùng để cấp chứng chỉ SSL hợp lệ cho các tên miền cục bộ.

```bash
hyh ca <ACTION>

ACTIONS:
  install  Cài đặt CA vào Windows Trust Store
  status   Kiểm tra trạng thái CA
```

**Ví dụ mẫu:**

```bash
# Cài đặt CA (cần Admin) — trình duyệt sẽ tin tưởng SSL sau bước này
hyh ca install
# 🔐 Installing CA to Windows trust store...
#   ✓ certutil: Chrome/Edge trusted
#   ✓ mkcert: Firefox NSS trusted
#
# ✅ CA installed successfully

# Kiểm tra CA đã được cài đặt chưa
hyh ca status
# CA: 🟢 installed & trusted
```

---

### Quy trình sử dụng mẫu (Workflow)

Dưới đây là quy trình hoàn chỉnh từ lúc cài đặt tới khi có HTTPS tên miền cục bộ:

```bash
# Bước 1: Cài đặt CA vào hệ thống (chỉ cần làm 1 lần)
hyh ca install

# Bước 2: Khởi động Nginx
hyh nginx start

# Bước 3: Thêm tên miền cho React app đang chạy tại port 3000
hyh add myapp.test http://127.0.0.1:3000

# Bước 4: Mở trình duyệt và truy cập
# → https://myapp.test  ✅ Xanh lá, HTTPS hợp lệ!

# Bước 5: Xem danh sách domain hiện có
hyh list

# Bước 6: Tắt domain khi không cần nữa
hyh toggle myapp.test
```

---

## 🌐 Public Tunnel (Cloudflare)

HyperHost tích hợp sẵn `cloudflared` để chia sẻ trang web cục bộ ra ngoài Internet mà không cần cấu hình Router hay mua tên miền.

### Cách sử dụng

1. Mở HyperHost GUI.
2. Nhấn nút **🌐** (Share Public Tunnel) cạnh domain bất kỳ.
3. Chờ vài giây — một URL dạng `https://xxx-yyy.trycloudflare.com` sẽ xuất hiện.
4. Gửi link này cho bất kỳ ai trên thế giới.

### Lưu ý quan trọng cho người dùng tại Việt Nam

Một số nhà mạng Việt Nam (VNPT, Viettel, FPT) **chặn DNS** của `*.trycloudflare.com`. Nếu bạn hoặc người nhận link gặp lỗi `DNS_PROBE_FINISHED_NXDOMAIN`, hãy đổi DNS:

**Cách 1 — Đổi DNS hệ thống (PowerShell Admin):**

```powershell
netsh interface ip set dns "Wi-Fi" static 1.1.1.1
netsh interface ip add dns "Wi-Fi" 8.8.8.8 index=2
ipconfig /flushdns
```

**Cách 2 — Bật Secure DNS trên trình duyệt:**

1. Mở `chrome://settings/security` (Chrome) hoặc `edge://settings/privacy` (Edge).
2. Bật **"Use secure DNS"**.
3. Chọn **Cloudflare (1.1.1.1)** hoặc **Google (Public DNS)**.

---

## 🛠 Hướng dẫn Dành cho Developer (Tự Build)

Nếu bạn muốn đóng góp code hoặc tự Build từ gốc, hãy làm theo các bước sau:

**Yêu cầu hệ thống:** NodeJS (v20+), Rust, và C++ Build Tools for Windows.

1.  **Clone mã nguồn**:
    ```bash
    git clone https://github.com/ptn1411/HyperHost.git
    cd HyperHost
    ```
2.  **Cài đặt thư viện Frontend**:

    ```bash
    npm install
    ```

3.  **Khởi chạy chế độ Phát triển (Dev Mode)**:

    ```bash
    # Cần chạy ở Terminal quyền Administrator
    npm run tauri dev
    ```

4.  **Build CLI riêng (không cần GUI)**:

    ```bash
    cd src-tauri
    cargo build --release --bin hyh --no-default-features
    # Output: target/release/hyh.exe
    ```

5.  **Đóng gói (Build) ra file Exe**:

    ```bash
    # (Tùy chọn) Nếu bạn muốn Build Updater, hãy nạp biến môi trường
    # $env:TAURI_PRIVATE_KEY="<Private-Key>"

    npm run tauri build
    ```

_Lưu ý: Kho chứa không đẩy file `updater_keys` (Private Key) để bảo mật chống giả mạo cập nhật. Bạn phải sử dụng khóa của chính mình nếu tự build và muốn dùng Updater._

## 📄 Giấy phép (License)

Dự án được mở mã nguồn dưới chứng chỉ MIT. Bạn hoàn toàn có thể tự do chỉnh sửa và sử dụng.
