# Security Policy

## Phiên bản được hỗ trợ

| Phiên bản | Hỗ trợ bảo mật     |
| --------- | ------------------- |
| 0.3.x     | :white_check_mark:  |
| 0.2.x     | :white_check_mark:  |
| < 0.2     | :x:                 |

> Chỉ **hai phiên bản minor mới nhất** nhận được bản vá bảo mật. Vui lòng nâng cấp lên phiên bản mới nhất qua Auto-Updater hoặc tải từ [Releases](../../releases/latest).

---

## Báo cáo lỗ hổng bảo mật

> **Không** mở Issue công khai cho lỗ hổng bảo mật.

Nếu bạn phát hiện lỗ hổng trong HyperHost, vui lòng báo cáo **riêng tư** qua một trong các kênh sau:

| Kênh | Chi tiết |
|------|----------|
| **GitHub Security Advisories** | [Tạo báo cáo riêng tư](../../security/advisories/new) (khuyến nghị) |
| **Email** | Gửi email tới maintainer qua GitHub profile của [ptn1411](https://github.com/ptn1411) |

### Quy trình xử lý

1. **Xác nhận nhận báo cáo** — trong vòng **48 giờ**.
2. **Đánh giá & phân loại** — xác định mức độ nghiêm trọng (Critical / High / Medium / Low) trong vòng **7 ngày**.
3. **Phát triển bản vá** — fix sẽ được phát triển trên nhánh riêng, không công khai cho đến khi sẵn sàng.
4. **Phát hành & thông báo** — bản vá được release kèm Security Advisory trên GitHub. Người dùng nhận được thông báo qua Auto-Updater.

Nếu lỗ hổng bị từ chối (false positive hoặc ngoài phạm vi), chúng tôi sẽ giải thích lý do cụ thể.

---

## Kiến trúc bảo mật

HyperHost là ứng dụng **desktop** (Tauri v2 / Rust + React) chạy hoàn toàn trên máy cục bộ. Dưới đây là các thành phần bảo mật quan trọng:

### 🔐 Certificate Authority (CA) nội bộ

- CA root certificate được tạo bằng `rcgen` (Rust) và cài vào system trust store qua `mkcert`.
- Chứng chỉ cho domain cục bộ có hiệu lực **365 ngày** và tự động gia hạn khi gần hết hạn.
- SHA-256 fingerprint của CA hiển thị trong UI và CLI (`hyh ca status`) để người dùng xác minh tính toàn vẹn.
- **Private key của CA chỉ lưu trên máy cục bộ**, không bao giờ được truyền qua mạng.

### 🌐 Nginx Proxy

- Config nginx được **tự động sinh** từ dữ liệu trong SQLite, không cho phép inject trực tiếp.
- Trước mỗi lần reload, config được validate bằng `nginx -t` để tránh downtime do cấu hình lỗi.
- Custom config qua Monaco Editor được **sandbox** trong thư mục riêng cho từng domain.

### 📂 Hosts File

- App yêu cầu **quyền Administrator / sudo** để ghi file `hosts` hệ thống.
- Chỉ thêm/xóa các entry trỏ về `127.0.0.1` — không chỉnh sửa entry có sẵn của hệ thống.

### 🔄 Auto-Updater

- Sử dụng **Tauri Updater v2** với chữ ký số (minisign public key).
- Chỉ tải bản cập nhật từ endpoint GitHub Releases chính thức (`github.com/ptn1411/HyperHost`).
- **Private key** của updater **không được commit** vào repository.

### ☁️ Cloudflare Tunnel

- Tunnel sử dụng binary `cloudflared` chính thức, tạo URL tạm thời dạng `*.trycloudflare.com`.
- **Không yêu cầu đăng nhập** Cloudflare — sử dụng túnel nhanh (Quick Tunnel).
- Lưu ý: khi bật tunnel, ứng dụng cục bộ sẽ **tạm thời truy cập được từ Internet** cho đến khi tắt tunnel.

### 💾 SQLite Database

- Dữ liệu domain, upstream, trạng thái chứng chỉ lưu trong file SQLite cục bộ.
- Không chứa thông tin nhạy cảm của người dùng (không có tài khoản, mật khẩu).

### 🛡️ Tauri IPC & Capabilities

- Frontend giao tiếp với Rust backend qua **Tauri IPC** — không có HTTP API mở.
- Capabilities được giới hạn tối thiểu: `core:default`, `opener:default`, `updater:default`, `process:allow-restart`.
- CSP (Content Security Policy) có thể được cấu hình trong `tauri.conf.json`.

---

## Lưu ý bảo mật cho người dùng

| Rủi ro | Mô tả | Khuyến nghị |
|--------|--------|-------------|
| **CA root trong trust store** | CA của HyperHost được cài vào hệ thống để trình duyệt tin tưởng chứng chỉ `.test`/`.local`. Nếu private key bị lộ, kẻ tấn công có thể tạo chứng chỉ giả cho domain bất kỳ. | Không chia sẻ thư mục data/CA key. Gỡ CA khi không cần (`hyh ca` hoặc xóa thủ công). |
| **Quyền Administrator** | App cần quyền cao để sửa hosts file và quản lý nginx. | Chỉ cấp quyền khi tin tưởng nguồn cài đặt. Tải từ GitHub Releases chính thức. |
| **Cloudflare Tunnel** | Expose ứng dụng cục bộ ra Internet. | Chỉ bật tunnel khi cần thiết, tắt ngay sau khi dùng xong. Không expose dịch vụ chứa dữ liệu nhạy cảm. |
| **Updater key** | File `updater_keys.pub` chứa public key — đây là thông tin công khai, không phải bí mật. | Đảm bảo private key được giữ an toàn và **không commit** vào repo. |

---

## Phạm vi bảo mật (Scope)

### Trong phạm vi ✅

- Lỗ hổng trong mã nguồn Rust/TypeScript của HyperHost
- Vấn đề với quy trình tạo/quản lý chứng chỉ CA
- Lỗi leo thang đặc quyền (privilege escalation)
- Lỗ hổng trong Tauri IPC commands
- Vấn đề với auto-updater (bypass chữ ký, MITM)
- Injection trong nginx config generation
- Path traversal hoặc arbitrary file write

### Ngoài phạm vi ❌

- Lỗ hổng trong nginx, mkcert, cloudflared (báo cáo cho upstream project tương ứng)
- Lỗ hổng trong Tauri framework (báo cáo tại [tauri-apps/tauri](https://github.com/tauri-apps/tauri/security))
- Tấn công yêu cầu physical access vào máy đã mở khóa
- Social engineering
- Denial of Service trên máy cục bộ (app chỉ chạy local)

---

## Cảm ơn

Cảm ơn bạn đã giúp giữ HyperHost an toàn cho cộng đồng! 🛡️
