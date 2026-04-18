// ==========================================
// English (en)
// ==========================================

// Header
export const headerSubtitle = "Local HTTPS domains for development";

// Nginx status
export const nginxRunning = "nginx: RUNNING";
export const nginxStopped = "nginx: STOPPED";

// CA
export const caInstall = "Install CA";
export const caTrusted = "CA Trusted";
export const caNotTrustedTitle = "CA Certificate is not trusted";
export const caNotTrustedDesc = "Browsers will show a red lock for HTTPS domains. Install the CA certificate to fix this.";
export const caInstallNow = "Install now";

// Tabs
export const tabDomains = "Domains & Proxy";
export const tabTraffic = "Live Traffic";
export const tabNamedTunnel = "Named Tunnel";
export const tabSettings = "⚙ Settings";

// Quick Add Form
export const quickAddTitle = "Quick Add Route";
export const codeEditorMode = "Code Editor Mode";
export const labelLocalDomain = "Local Domain";
export const labelUpstream = "Upstream Server";
export const btnQuickCreate = "Quick Create";
export const btnCreating = "...";

// Domain List
export const domainListTitle = "Active Routes";
export const btnImport = "Import";
export const btnExport = "Export";
export const importTitle = "Import domains from JSON";
export const exportTitle = "Export all domains to JSON";
export const errorLogShow = "Nginx Error Log";
export const errorLogHide = "Hide Error Log";
export const emptyDomainTitle = "No domains configured";
export const emptyDomainDesc = "Add your first local proxy route above.";
export const importNoValid = "No valid domains found in import file.";

// Domain Card
export const sslValid = "Valid SSL";
export const sslInvalid = "Invalid SSL";
export const corsEnableTitle = "Click to enable CORS headers";
export const corsDisableTitle = "CORS enabled — click to disable";
export const btnFolder = "Folder";
export const btnRun = "Run";
export const btnEdit = "Edit";
export const tooltipEditConfig = "Edit Configuration";
export const tooltipCopyUrl = "Copy URL";
export const tooltipRemoveRoute = "Remove Route";
export const tooltipShareTunnel = "Share Public Tunnel";
export const tunnelStarting = "Starting Tunnel...";
export const tooltipStopTunnel = "Stop Tunnel";

// Delete Confirmation
export const deleteTitle = "Confirm deletion";
export const deleteMessage = "Are you sure you want to delete this domain?";
export const btnCancel = "Cancel";
export const btnDeleteDomain = "Delete domain";

// Log Viewer
export const logTitle = "nginx error.log";
export const logRefresh = "↻ Refresh";
export const logEmpty = "Nothing interesting is happening right now.";

// Settings
export const settingsTitle = "App Settings";
export const settingsAutostart = "Start with Windows";
export const settingsAutostartDesc = "Automatically run HyperHost on Windows login";
export const settingsStartHidden = "Start minimized to tray only";
export const settingsStartHiddenDesc = "When Windows starts, HyperHost runs in background — icon appears in the system tray";
export const settingsLanguage = "Language";
export const settingsLanguageDesc = "Change the application display language";

// Update Dialog
export const updateTitle = "Update Available!";
export const updateReady = "is ready to install.";
export const updateDownloading = "Downloading...";
export const updateLater = "Later";
export const updateInstall = "Install & Relaunch";
export const updateUpdating = "Updating...";

// Traffic Inspector
export const trafficTitle = "Live HTTP Traffic";
export const trafficListening = "Listening...";
export const trafficColMethod = "Method";
export const trafficColStatus = "Status";
export const trafficColDomain = "Domain";
export const trafficColUri = "URI";
export const trafficColTime = "Time";
export const trafficEmpty = "No traffic recorded yet.";
export const trafficDetails = "Request Details";
export const trafficLatency = "Latency";
export const trafficBody = "Request Body";
export const trafficNoBody = "No body";

// Nginx Editor Mode
export const editorTitleNew = "New Proxy Configuration";
export const editorTitleEdit = "Editing Domain: %%domain%%";
export const editorLabelDomain = "Local Domain (e.g. myapp.test)";
export const editorLabelUpstream = "Upstream Target (e.g. http://127.0.0.1:8080)";
export const editorLabelProjectPath = "Project Directory (optional)";
export const editorLabelRunCommand = "Run Command (optional)";
export const editorBtnImport = "Import from prod";
export const editorBtnValidate = "Validate (nginx -t)";
export const editorBtnExport = "Export to project";
export const editorBtnCancelEdit = "Cancel editing";
export const editorBtnUpdate = "Update Config";
export const editorBtnCreate = "Create Route";
export const editorBtnClear = "Clear";
export const editorImportTitle = "Import nginx config from prod";
export const editorImportDesc = "Paste your .conf file contents. The tool will automatically strip SSL/listen/server_name and rewrite proxy_pass to $UPSTREAM.";
export const editorImportBtn = "Convert & Apply";
export const editorImportProcessing = "Processing…";
export const editorExportTitle = "Export to project folder";
export const editorExportLabelDomain = "Prod Domain";
export const editorExportLabelUpstream = "Prod Upstream";
export const editorExportBtn = "Export file";
export const editorExportWriting = "Writing…";
export const editorBtnClose = "Close";
export const editorValidateEmpty = "Config is empty — nothing to validate.";

// Quick Start Panel
export const quickStartTitle = "Quick Start";
export const quickStartSubtitle = "Templates · Port scan · Project scan";
export const quickStartTabTemplates = "Templates";
export const quickStartTabPorts = "Open Ports";
export const quickStartTabProjects = "Scan Projects";
export const quickStartTemplateDesc = "Choose a preset to auto-fill upstream. Stacks with HMR/WebSocket will open the editor with a matching nginx snippet.";
export const quickStartLoading = "Loading…";
export const quickStartPortDesc = "List all TCP ports listening on 127.0.0.1 with process name.";
export const quickStartHideSystem = "Hide system / nginx";
export const quickStartScanNow = "Scan now";
export const quickStartRescan = "Rescan";
export const quickStartScanning = "Scanning…";
export const quickStartNoPort = "No ports are listening.";
export const quickStartAllHidden = "All ports are hidden — uncheck filter to see.";
export const quickStartUse = "Use →";
export const quickStartProjectDesc = "Scan directory for Node / Rust / Go / Django / Laravel / Rails projects. Auto-detect default ports by framework.";
export const quickStartScan = "Scan";
export const quickStartNoProject = "No projects found in this directory.";

// Named Tunnel Panel
export const namedTunnelTitle = "Named Tunnel";
export const namedTunnelDesc = "Use your own fixed domain via Cloudflare";
export const namedTunnelLogin = "Login Cloudflare";
export const namedTunnelConnected = "Cloudflare: Connected";
export const namedTunnelRequirements = "Requirements:";
export const namedTunnelReqOwn = "You own a domain and have added it to Cloudflare";
export const namedTunnelReqLogin = "Log in to Cloudflare once (creates cert.pem)";
export const namedTunnelReqEach = "Each tunnel = 1 fixed hostname → local upstream";
export const namedTunnelAdd = "Add Named Tunnel";
export const namedTunnelAddNew = "Add New Named Tunnel";
export const namedTunnelLabelName = "Tunnel Name";
export const namedTunnelLabelHostname = "Hostname (domain you own)";
export const namedTunnelLabelUpstream = "Upstream (local server)";
export const namedTunnelBtnAdd = "Add";
export const namedTunnelBtnCancel = "Cancel";
export const namedTunnelEmpty = "No Named Tunnels yet";
export const namedTunnelEmptyDesc = "Add a tunnel to use a fixed domain.";
export const namedTunnelProvisioned = "Provisioned";
export const namedTunnelNotProvisioned = "Not Provisioned";
export const namedTunnelRunning = "Running";
export const namedTunnelProvision = "Provision";
export const namedTunnelStop = "Stop";
export const namedTunnelStart = "Start";
export const namedTunnelNeedLogin = "You need to login to Cloudflare first";
export const namedTunnelProvisionTooltip = "Create tunnel on Cloudflare";
