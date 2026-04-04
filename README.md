<div align="center">
  <img src="https://raw.githubusercontent.com/tauri-apps/tauri/HEAD/app-icon.png" width="100" />
  <h1>HyperHost</h1>
  <p><strong>Professional Local Virtual Domain & HTTPS Manager for Windows</strong></p>
</div>

---

**HyperHost** (trước đây là DevHost) là một công cụ mã nguồn mở dành riêng cho các lập trình viên trên hệ điều hành Windows. Công cụ này đóng vai trò như một bảng điều khiển Nginx siêu tốc, giúp bạn tự động định tuyến tên miền ảo (ví dụ: `my-project.test` hoặc `api.local`) tới các cổng `localhost` đang chạy với **đầy đủ chứng chỉ HTTPS (SSL) hợp lệ**.

Không còn cảnh phải sửa file `hosts` bằng tay, không còn phải loay hoay tạo chứng chỉ Self-Signed lỗi xanh lỗi đỏ trên trình duyệt. Chỉ cần 1 click là bạn có ngay một tên miền chuyên nghiệp ngay dưới máy tính nội bộ của mình!

## 🚀 Tính năng nổi bật

*   **⚡ Quản lý tên miền tức thì**: Giao diện UI/UX siêu mượt xây dựng bằng React. Tạo nhanh một tên miền và liên kết tới một cổng Upstream (VD: `localhost:3000`) chỉ trong 3 giây.
*   **🔒 HTTPS (SSL) tự động**: Tích hợp cực sâu công cụ chuyên dụng `mkcert`. Tự động đứng ra làm Tổ chức chứng thực (Certificate Authority - CA) của máy tính, cấp chứng chỉ xanh lá cây hợp lệ cho mọi tên miền ảo.
*   **💻 Text Editor cấp độ Pro**: Tích hợp lõi *Monaco Editor* (công nghệ đằng sau VSCode). Bạn có thể viết và chèn cấu hình Raw Nginx Server blocks/Directives tùy biến với Auto-format cực kì mạnh mẽ.
*   **👻 Chế độ Background / System Tray**: Hoạt động âm thầm không làm phiền bạn. Khi bấm dấu 'X' để tắt cửa sổ, HyperHost tự giấu mình vào System Tray dưới góc phải màn hình trong khi Nginx vẫn hoạt động mượt mà.
*   **🔄 Tự động Cập Nhật (Auto-Updater)**: Khi nhà phát triển ra mắt phiên bản mới trên GitHub Releases, HyperHost sẽ tự động mở hộp thoại thông báo tải xuống & cài đặt bản mới trực tiếp nhờ lõi Tauri Updater.

## 📦 Công nghệ sử dụng
*   **Backend**: [Rust](https://www.rust-lang.org/) & [Tauri v2](https://v2.tauri.app/).
*   **Frontend**: [React 18](https://react.dev/), [Vite](https://vitejs.dev/), và [TailwindCSS v4](https://tailwindcss.com/).
*   **Core Systems**: `nginx` (Máy chủ Proxy), `mkcert` (Quản lý HTTPS), SQLite (Dữ liệu).

## 📥 Tải xuống & Cài đặt

Ứng dụng HyperHost hiện cung cấp bộ cài `.exe` thông minh tự động (NSIS Installer) thông qua GitHub Actions CI/CD.

Bạn hãy truy cập vào mục **[Releases](../../releases/latest)** của kho lưu trữ này và tải về file `HyperHost_..._x64-setup.exe`. Trình cài đặt sẽ tự động lưu vào `C:\Program Files\HyperHost` và đổ biểu tượng ra màn hình Destkop cho bạn.

## 🛠 Hướng dẫn Dành cho Developer (Tự Build)

Nếu bạn muốn đóng góp code hoặc tự Build từ gốc, hãy làm theo các bước sau:

**Yêu cầu hệ thống:** NodeJS (v20+), Rust, và C++ Build Tools for Windows.

1.  **Clone mã nguồn**:
    ```bash
    git clone https://github.com/USERNAME/REPO.git
    cd REPO
    ```
    
2.  **Cài đặt thư viện Frontend**:
    ```bash
    npm install
    ```

3.  **Khởi chạy chế độ Phát triển (Dev Mode)**:
    ```bash
    npm run tauri dev
    ```

4.  **Đóng gói (Build) ra file Exe**:
    ```bash
    # (Tùy chọn) Nếu bạn muốn Build Updater, hãy nạp biến môi trường
    # $env:TAURI_PRIVATE_KEY="<Private-Key>" 
    
    npm run tauri build
    ```

*Lưu ý: Kho chứa không đẩy file `updater_keys` (Private Key) để bảo mật chống giả mạo cập nhật. Bạn phải sử dụng khóa của chính mình nếu tự build và muốn dùng Updater.*

## 📄 Giấy phép (License)
Dự án được mở mã nguồn dưới chứng chỉ MIT. Bạn hoàn toàn có thể tự do chỉnh sửa và sử dụng.
