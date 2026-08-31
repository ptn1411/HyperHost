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

// Domain List & Cards
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

export const sslValid = "Valid SSL";
export const sslInvalid = "Invalid SSL";
export const corsEnableTitle = "Click to enable CORS headers";
export const corsDisableTitle = "CORS enabled — click to disable";
export const btnFolder = "Folder";
export const btnTerminal = "Terminal";
export const btnDocker = "Docker";
export const btnAiSkill = "AI Skill";
export const btnRun = "Run";
export const btnEdit = "Edit";
export const tooltipOpenFolder = "Open folder: %%path%%";
export const tooltipOpenTerminal = "Open terminal in project folder";
export const tooltipDocker = "Manage project docker compose";
export const tooltipAiSkill = "Install AI skills (Claude, Gemini, Codex) into project";
export const tooltipRunCommand = "Run: %%command%%";
export const tooltipEditConfig = "Edit Configuration";
export const tooltipCopyUrl = "Copy URL";
export const tooltipRemoveRoute = "Remove Route";
export const tooltipShareTunnel = "Share Public Tunnel";
export const tunnelStarting = "Starting Tunnel...";
export const tooltipStopTunnel = "Stop Tunnel";
export const tunnelDnsHint = "⚠ If unreachable, change DNS to 1.1.1.1 or enable Secure DNS";
export const tunnelError = "Tunnel [%%domain%%]: %%error%%";
export const aiSkillsInstalled = "AI skills installed into %%path%%";
export const aiSkillsFailed = "Some skills failed: %%failed%%";

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
export const editorPlaceholderProjectPath = "Project folder (optional — enables quick folder/terminal/docker actions)";
export const editorPlaceholderRunCommand = "Run command (optional — e.g. npm run dev, cargo run...)";
export const editorBtnImport = "Import from prod";
export const editorBtnValidate = "Validate (nginx -t)";
export const editorBtnExport = "Export to project";
export const editorBtnCancelEdit = "Cancel editing";
export const editorBtnUpdate = "Update Config";
export const editorBtnCreate = "Create Route";
export const editorBtnClear = "Clear";
export const editorSubtext = "Direct Nginx configuration editor. Supports variables $DOMAIN, $UPSTREAM, $CERT_PATH, $KEY_PATH.";
export const editorImportTooltip = "Paste production .conf content to auto-convert to dev configuration";
export const editorValidateTooltip = "Validate nginx syntax with nginx -t";
export const editorExportTooltip = "Export nginx snippet to project folder (ready for production deploy)";
export const editorCancelTooltip = "Cancel editing";
export const editorUpdateTooltip = "Update Nginx configuration for this domain";
export const editorCreateTooltip = "Create new domain with this configuration";
export const editorClearTooltip = "Reset configuration";
export const editorClearConfirm = "Clear all Nginx configuration?";
export const editorValidateEmpty = "Config is empty — nothing to validate.";
export const editorValidateSyntaxOk = "nginx -t: syntax OK";
export const editorImportValidateOk = "Import passed nginx -t ✓";
export const editorImportValidateFail = "Import: nginx -t failed\n%%error%%";
export const editorImportTitle = "Import nginx config from prod";
export const editorImportDesc = "Paste your .conf file contents. The tool will automatically strip SSL/listen/server_name and rewrite proxy_pass to $UPSTREAM.";
export const editorImportBtn = "Convert & Apply";
export const editorImportProcessing = "Processing…";
export const editorExportTitle = "Export to project folder";
export const editorExportLabelDomain = "Prod Domain";
export const editorExportLabelUpstream = "Prod Upstream";
export const editorExportPlaceholderProdDomain = "Prod domain (e.g. myapp.com)";
export const editorExportPlaceholderProdUpstream = "Prod upstream (e.g. http://127.0.0.1:8080 or unix:/tmp/app.sock)";
export const editorExportBtn = "Export file";
export const editorExportWriting = "Writing…";
export const editorExportSaved = "Configuration saved to: %%path%%";
export const editorBtnClose = "Close";

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
export const quickStartProjectCreateDomainTooltip = "Create domain for this project";
export const quickStartProjectOpenTerminalTooltip = "Open terminal in project directory";
export const quickStartProjectRunCommandTooltip = "Open terminal and run: %%command%%";
export const quickStartProjectCreateTooltip = "Create domain";

// Named Tunnel Panel
export const namedTunnelTitle = "Named Tunnel";
export const namedTunnelDesc = "Use your own fixed domain via Cloudflare";
export const namedTunnelLogin = "Login Cloudflare";
export const namedTunnelConnected = "Cloudflare: Connected";
export const namedTunnelLoginSuccess = "Cloudflare login successful";
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
export const namedTunnelProvisionSuccess = "Tunnel \"%%name%%\" created successfully on Cloudflare";
export const namedTunnelStop = "Stop";
export const namedTunnelStart = "Start";
export const namedTunnelNeedLogin = "You need to login to Cloudflare first";
export const namedTunnelProvisionTooltip = "Create tunnel on Cloudflare";

// Docker Panel
export const dockerTitle = "Docker Compose · %%domain%%";
export const dockerClose = "Close (Esc)";
export const dockerStatusTitle = "Docker Status";
export const dockerCli = "Docker CLI";
export const dockerDaemon = "Docker Daemon";
export const dockerNotInstalled = "Not installed / Not found";
export const dockerRunning = "Running";
export const dockerStopped = "Stopped";
export const dockerRefresh = "Refresh";
export const dockerChecking = "Checking...";
export const dockerFilesTitle = "Compose Files in Project";
export const dockerNoFiles = "No docker-compose*.yml files found in project directory.";
export const dockerCreateWithAi = "Create Compose with AI";
export const dockerServices = "Services (%%count%%):";
export const dockerNoServicesRunning = "No services running currently.";
export const dockerBtnRun = "Start (up -d)";
export const dockerBtnStop = "Stop (down)";
export const dockerBtnRestart = "Restart";
export const dockerBtnLogs = "Logs";
export const dockerOutput = "Output";
export const dockerClear = "Clear";
export const dockerPromptTitle = "Generate Docker Compose Prompt for AI";
export const dockerPromptDesc = "Select databases / services your project needs:";
export const dockerPromptPort = "Port:";
export const dockerPromptPass = "Pass:";
export const dockerPromptExtraReq = "Additional requirements (optional):";
export const dockerPromptExtraPlaceholder = "e.g. add phpMyAdmin for MySQL, custom network, persistent volumes...";
export const dockerPromptGenerated = "Generated AI Prompt (Cursor, Claude, Copilot, ChatGPT...):";
export const dockerPromptCopy = "Copy Prompt";
export const dockerPromptCopied = "Copied!";
export const dockerPasteTitle = "Save generated YAML to project";
export const dockerPasteFileName = "File name";
export const dockerPasteContent = "YAML Content:";
export const dockerPastePlaceholder = "Paste the AI generated docker-compose.yml content here...";
export const dockerPasteSave = "Save file to project";
export const dockerPasteSaving = "Saving...";
export const dockerSaveValidation = "File name and YAML content are required.";
export const dockerSaveOverwriteConfirm = "File %%fileName%% already exists. Overwrite?";
export const dockerSaveSuccess = "Saved: %%path%%";
