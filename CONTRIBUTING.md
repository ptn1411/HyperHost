# Đóng góp cho HyperHost

Cảm ơn bạn đã quan tâm đến việc đóng góp cho **HyperHost** — công cụ quản lý HTTPS Domain cục bộ được xây dựng bằng Tauri v2, Rust, React và TypeScript.

## Mục lục

- [Bắt đầu](#bắt-đầu)
- [Thiết lập môi trường phát triển](#thiết-lập-môi-trường-phát-triển)
- [Cấu trúc dự án](#cấu-trúc-dự-án)
- [Cách đóng góp](#cách-đóng-góp)
- [Quy ước commit](#quy-ước-commit)
- [Hướng dẫn Pull Request](#hướng-dẫn-pull-request)
- [Báo cáo lỗi](#báo-cáo-lỗi)

## Bắt đầu

1. **Fork** repository trên GitHub
2. **Clone** fork về máy:
   ```bash
   git clone https://github.com/YOUR_USERNAME/HyperHost.git
   cd HyperHost
   ```
3. Thêm upstream remote:
   ```bash
   git remote add upstream https://github.com/ptn1411/HyperHost.git
   ```

## Thiết lập môi trường phát triển

### Yêu cầu

- [Node.js](https://nodejs.org/) >= 20
- [Rust](https://rustup.rs/) (stable toolchain)
- [Tauri CLI v2](https://tauri.app/start/prerequisites/)

Trên Windows, bạn cần thêm:
- Microsoft Visual Studio Build Tools (C++ workload)
- WebView2 Runtime

Trên macOS:
```bash
brew install nginx
```

Trên Linux (Debian/Ubuntu):
```bash
sudo apt install nginx libgtk-3-dev
```

### Cài đặt dependencies

```bash
npm install
```

### Chạy chế độ phát triển

> **Lưu ý:** Cần quyền Administrator / sudo vì app ghi file `hosts` và quản lý nginx.

```bash
npm run tauri dev
```

### Build bản production

```bash
npm run tauri build
```

### Chỉ chạy Frontend (Vite)

```bash
npm run dev
```

### Build CLI riêng (không cần GUI)

```bash
cd src-tauri
cargo build --release --bin hyh --no-default-features
```

## Cấu trúc dự án

```
HyperHost/
├── src/                    # React + TypeScript frontend
├── src-tauri/              # Rust backend (Tauri v2)
│   ├── src/
│   │   ├── bin/cli.rs      # CLI binary (hyh)
│   │   ├── cert/           # Quản lý chứng chỉ CA & HTTPS
│   │   ├── nginx/          # Sinh & quản lý config nginx
│   │   ├── dns/            # Quản lý hosts file
│   │   ├── cloudflare/     # Cloudflare Tunnel integration
│   │   ├── db/             # SQLite database
│   │   ├── ipc/            # Tauri IPC commands
│   │   └── ...
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── tsconfig.json
```

## Cách đóng góp

### Sửa lỗi

- Kiểm tra [các issue hiện có](https://github.com/ptn1411/HyperHost/issues) trước
- Tạo issue mô tả lỗi trước khi gửi bản sửa

### Thêm tính năng mới

- Mở issue thảo luận tính năng trước khi bắt đầu code
- Giữ thay đổi tập trung — mỗi PR chỉ một tính năng

### Cải thiện tài liệu

Luôn chào đón mọi cải thiện về tài liệu. Không cần issue cho các bản sửa lỗi chính tả hoặc làm rõ nhỏ.

## Quy ước commit

Sử dụng [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: thêm tự động gia hạn chứng chỉ mkcert
fix: sửa đường dẫn nginx config trên Windows
docs: cập nhật hướng dẫn thiết lập phát triển
refactor: đơn giản hóa logic kiểm tra domain
chore: nâng Tauri lên v2.x
```

## Hướng dẫn Pull Request

- Branch từ `main` và target về `main`
- Giữ PR nhỏ gọn và tập trung
- Mô tả rõ ràng những gì đã thay đổi và lý do
- Đảm bảo dự án build không lỗi:
  ```bash
  npm run build
  cargo check --manifest-path src-tauri/Cargo.toml
  ```
- Tham chiếu issue liên quan trong mô tả PR (`Closes #123`)

## Báo cáo lỗi

Sử dụng [GitHub Issues](https://github.com/ptn1411/HyperHost/issues) và bao gồm:

- Hệ điều hành và phiên bản (Windows 10/11, macOS, Linux distro)
- Phiên bản HyperHost
- Các bước tái hiện lỗi
- Kết quả mong đợi so với kết quả thực tế
- Log hoặc ảnh chụp màn hình (nếu có)

> ⚠️ Đối với **lỗ hổng bảo mật**, vui lòng **không** mở issue công khai. Xem [SECURITY.md](SECURITY.md) để biết cách báo cáo riêng tư.

## Giấy phép

Bằng việc đóng góp, bạn đồng ý rằng các đóng góp của bạn sẽ được cấp phép theo [Giấy phép MIT](LICENSE).
