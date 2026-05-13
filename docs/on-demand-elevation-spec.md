# Technical Spec: On-Demand Elevation for HyperHost

## 1. Objective
Refactor HyperHost (GUI and CLI) to run with standard user privileges (`asInvoker`) by default, and only request Administrator elevation when performing privileged operations (editing `hosts` file, managing `nginx` services).

## 2. Manifest Changes
Modify `src-tauri/app.manifest` to prevent Windows from forcing a UAC prompt on startup.

```xml
<!-- From -->
<requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>

<!-- To -->
<requestedExecutionLevel level="asInvoker" uiAccess="false"/>
```

## 3. Privileged Operations Identification
The following operations must be wrapped in an elevation check:
- **DNS Module**: `sync_hosts` (Writing to `C:\Windows\System32\drivers\etc\hosts`).
- **Nginx Module**: `start`, `stop`, `reload` (Managing services/binding to port 80/443).
- **CA Module**: `ca install` (Writing to the Windows Trust Store).

## 4. Implementation Strategy (Rust)

### 4.1. Elevation Check Utility
Add a helper function to determine if the current process is running with administrative privileges.

```rust
pub fn is_admin() -> bool {
    #[cfg(windows)]
    {
        use winapi::um::shellapi::IsUserAnAdmin;
        unsafe { IsUserAnAdmin() != 0 }
    }
    #[cfg(not(windows))]
    {
        unsafe { libc::getuid() == 0 }
    }
}
```

### 4.2. On-Demand Relaunch Logic
When a privileged command is called and `is_admin()` is false, the app should relaunch itself using the `runas` verb.

```rust
pub fn relaunch_as_admin() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let exe = std::env::current_exe()?;
        let args: Vec<String> = std::env::args().skip(1).collect();
        let args_str: Vec<u16> = args.join(" ").encode_utf16().chain(Some(0)).collect();
        let exe_str: Vec<u16> = exe.as_os_str().encode_utf16().chain(Some(0)).collect();

        unsafe {
            winapi::um::shellapi::ShellExecuteW(
                std::ptr::null_mut(),
                "runas".encode_utf16().chain(Some(0)).collect::<Vec<u16>>().as_ptr(),
                exe_str.as_ptr(),
                args_str.as_ptr(),
                std::ptr::null_mut(),
                winapi::um::winuser::SW_SHOWNORMAL,
            );
        }
        std::process::exit(0);
    }
    Ok(())
}
```

## 5. CLI Refactoring (`cli.rs`)
The `main()` function should be updated to handle commands selectively:

- **Standard Commands** (`list`, `status`, `doctor`, `mcp serve`): Execute immediately.
- **Privileged Commands** (`add`, `remove`, `nginx start`, etc.): 
    - Check `is_admin()`.
    - If false, call `relaunch_as_admin()`.

## 6. Benefits
- **Improved UX**: No UAC prompt for simply viewing domain lists.
- **AI Integration**: AI agents can query the state of HyperHost without being blocked by UAC.
- **Standard Compliance**: Follows the principle of least privilege.

---
*Created by Antigravity AI on 2026-04-30*
