// ==========================================
// Vietnamese (vi) — Ngôn ngữ mặc định
// ==========================================

// Header
export const headerSubtitle = "Quản lý tên miền HTTPS cục bộ cho lập trình viên";

// Nginx status
export const nginxRunning = "nginx: ĐANG CHẠY";
export const nginxStopped = "nginx: ĐÃ DỪNG";

// CA
export const caInstall = "Cài đặt CA";
export const caTrusted = "CA đã tin cậy";
export const caNotTrustedTitle = "CA Certificate chưa được trust";
export const caNotTrustedDesc = "Trình duyệt sẽ hiển thị khóa đỏ cho các domain HTTPS. Cài CA certificate để fix.";
export const caInstallNow = "Cài ngay";

// Tabs
export const tabDomains = "Tên Miền & Proxy";
export const tabTraffic = "Lưu lượng trực tiếp";
export const tabNamedTunnel = "Named Tunnel";
export const tabSettings = "⚙ Cài đặt";

// Quick Add Form
export const quickAddTitle = "Thêm Route Nhanh";
export const codeEditorMode = "Chế độ Code Editor";
export const labelLocalDomain = "Tên miền cục bộ";
export const labelUpstream = "Máy chủ Upstream";
export const btnQuickCreate = "Tạo Nhanh";
export const btnCreating = "...";

// Domain List
export const domainListTitle = "Danh sách Route";
export const btnImport = "Nhập";
export const btnExport = "Xuất";
export const importTitle = "Nhập domain từ file JSON";
export const exportTitle = "Xuất tất cả domain ra file JSON";
export const errorLogShow = "Nhật ký lỗi Nginx";
export const errorLogHide = "Ẩn nhật ký lỗi";
export const emptyDomainTitle = "Chưa có tên miền nào";
export const emptyDomainDesc = "Thêm route proxy cục bộ đầu tiên ở phía trên.";
export const importNoValid = "Không tìm thấy domain hợp lệ trong file import.";

// Domain Card
export const sslValid = "SSL hợp lệ";
export const sslInvalid = "SSL lỗi";
export const corsEnableTitle = "Nhấn để bật CORS headers";
export const corsDisableTitle = "CORS đang bật — nhấn để tắt";
export const btnFolder = "Thư mục";
export const btnRun = "Chạy";
export const btnEdit = "Sửa";
export const tooltipEditConfig = "Sửa cấu hình";
export const tooltipCopyUrl = "Sao chép URL";
export const tooltipRemoveRoute = "Xóa Route";
export const tooltipShareTunnel = "Chia sẻ qua Tunnel công khai";
export const tunnelStarting = "Đang khởi tạo Tunnel...";
export const tooltipStopTunnel = "Dừng Tunnel";

// Delete Confirmation
export const deleteTitle = "Xác nhận xóa";
export const deleteMessage = "Bạn có chắc muốn xóa domain này?";
export const btnCancel = "Hủy";
export const btnDeleteDomain = "Xóa domain";

// Log Viewer
export const logTitle = "nginx error.log";
export const logRefresh = "↻ Làm mới";
export const logEmpty = "Hiện tại không có gì đáng chú ý.";

// Settings
export const settingsTitle = "Cài đặt ứng dụng";
export const settingsAutostart = "Khởi động cùng Windows";
export const settingsAutostartDesc = "Tự động chạy HyperHost khi đăng nhập Windows";
export const settingsStartHidden = "Chỉ chạy icon khay, không mở cửa sổ";
export const settingsStartHiddenDesc = "Khi Windows khởi động, HyperHost chạy ngầm — icon xuất hiện ở góc phải màn hình";
export const settingsLanguage = "Ngôn ngữ";
export const settingsLanguageDesc = "Đổi ngôn ngữ giao diện ứng dụng";

// Update Dialog
export const updateTitle = "Có bản cập nhật mới!";
export const updateReady = "đã sẵn sàng cài đặt.";
export const updateDownloading = "Đang tải...";
export const updateLater = "Để sau";
export const updateInstall = "Cài đặt & Khởi động lại";
export const updateUpdating = "Đang cập nhật...";

// Traffic Inspector
export const trafficTitle = "Lưu lượng HTTP trực tiếp";
export const trafficListening = "Đang lắng nghe...";
export const trafficColMethod = "Phương thức";
export const trafficColStatus = "Trạng thái";
export const trafficColDomain = "Tên miền";
export const trafficColUri = "URI";
export const trafficColTime = "Thời gian";
export const trafficEmpty = "Chưa có lưu lượng nào được ghi nhận.";
export const trafficDetails = "Chi tiết Request";
export const trafficLatency = "Độ trễ";
export const trafficBody = "Nội dung Request";
export const trafficNoBody = "Không có nội dung";

// Nginx Editor Mode
export const editorTitleNew = "Tạo cấu hình Proxy Mới";
export const editorTitleEdit = "Chỉnh sửa Domain: %%domain%%";
export const editorLabelDomain = "Tên miền cục bộ (ví dụ: myapp.test)";
export const editorLabelUpstream = "Máy chủ đích (ví dụ: http://127.0.0.1:8080)";
export const editorLabelProjectPath = "Thư mục dự án (tùy chọn)";
export const editorLabelRunCommand = "Lệnh Run (tùy chọn)";
export const editorBtnImport = "Nhập từ prod";
export const editorBtnValidate = "Kiểm tra (nginx -t)";
export const editorBtnExport = "Xuất sang project";
export const editorBtnCancelEdit = "Hủy biên tập";
export const editorBtnUpdate = "Cập nhật Config";
export const editorBtnCreate = "Tạo Route";
export const editorBtnClear = "Làm mới (Clear)";
export const editorImportTitle = "Nhập nginx config từ prod";
export const editorImportDesc = "Dán nội dung file .conf của bạn. Tool sẽ tự strip SSL/listen/server_name và rewrite proxy_pass thành $UPSTREAM.";
export const editorImportBtn = "Convert & Áp dụng";
export const editorImportProcessing = "Đang xử lý…";
export const editorExportTitle = "Xuất sang thư mục dự án";
export const editorExportLabelDomain = "Tên miền Prod";
export const editorExportLabelUpstream = "Upstream Prod";
export const editorExportBtn = "Xuất file";
export const editorExportWriting = "Đang ghi…";
export const editorBtnClose = "Đóng";
export const editorValidateEmpty = "Config trống — không có gì để validate.";

// Quick Start Panel
export const quickStartTitle = "Khởi đầu nhanh";
export const quickStartSubtitle = "Mẫu cấu hình · Quét port · Quét dự án";
export const quickStartTabTemplates = "Mẫu cấu hình";
export const quickStartTabPorts = "Ports đang mở";
export const quickStartTabProjects = "Quét dự án";
export const quickStartTemplateDesc = "Chọn preset để tự điền upstream. Các stack có HMR/WebSocket sẽ mở sẵn editor với nginx snippet phù hợp.";
export const quickStartLoading = "Đang tải…";
export const quickStartPortDesc = "Liệt kê tất cả TCP port đang lắng nghe trên 127.0.0.1 cùng tên process.";
export const quickStartHideSystem = "Ẩn system / nginx";
export const quickStartScanNow = "Quét ngay";
export const quickStartRescan = "Quét lại";
export const quickStartScanning = "Đang quét…";
export const quickStartNoPort = "Không có port nào đang lắng nghe.";
export const quickStartAllHidden = "Tất cả port đều bị ẩn — bỏ chọn lọc để xem.";
export const quickStartUse = "Dùng →";
export const quickStartProjectDesc = "Quét thư mục để tìm các dự án Node / Rust / Go / Django / Laravel / Rails… Nhận diện port mặc định theo framework.";
export const quickStartScan = "Quét";
export const quickStartNoProject = "Không tìm thấy dự án nào trong thư mục này.";

// Named Tunnel Panel
export const namedTunnelTitle = "Named Tunnel";
export const namedTunnelDesc = "Dùng domain cố định của bạn qua Cloudflare";
export const namedTunnelLogin = "Login Cloudflare";
export const namedTunnelConnected = "Cloudflare: Đã kết nối";
export const namedTunnelRequirements = "Yêu cầu:";
export const namedTunnelReqOwn = "Bạn sở hữu domain và đã thêm vào Cloudflare";
export const namedTunnelReqLogin = "Đăng nhập Cloudflare một lần duy nhất (tạo cert.pem)";
export const namedTunnelReqEach = "Mỗi tunnel = 1 hostname cố định → upstream local";
export const namedTunnelAdd = "Thêm Named Tunnel";
export const namedTunnelAddNew = "Thêm Named Tunnel mới";
export const namedTunnelLabelName = "Tên Tunnel";
export const namedTunnelLabelHostname = "Hostname (domain bạn sở hữu)";
export const namedTunnelLabelUpstream = "Upstream (local server)";
export const namedTunnelBtnAdd = "Thêm";
export const namedTunnelBtnCancel = "Hủy";
export const namedTunnelEmpty = "Chưa có Named Tunnel nào";
export const namedTunnelEmptyDesc = "Thêm tunnel để dùng domain cố định.";
export const namedTunnelProvisioned = "Đã tạo";
export const namedTunnelNotProvisioned = "Chưa tạo";
export const namedTunnelRunning = "Đang chạy";
export const namedTunnelProvision = "Provision";
export const namedTunnelStop = "Dừng";
export const namedTunnelStart = "Chạy";
export const namedTunnelNeedLogin = "Cần đăng nhập Cloudflare trước";
export const namedTunnelProvisionTooltip = "Tạo tunnel trên Cloudflare";
