!macro NSIS_HOOK_POSTINSTALL
  ; Add install directory to system PATH so devhost-cli works from anywhere
  nsExec::ExecToLog 'powershell -ExecutionPolicy Bypass -Command "\
    $installDir = \"$INSTDIR\"; \
    $currentPath = [Environment]::GetEnvironmentVariable(\"Path\", \"Machine\"); \
    if ($currentPath -notlike \"*$installDir*\") { \
      [Environment]::SetEnvironmentVariable(\"Path\", \"$currentPath;$installDir\", \"Machine\"); \
      Write-Host \"Added $installDir to system PATH\" \
    }"'

  ; Broadcast WM_SETTINGCHANGE so open terminals pick up the new PATH
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=1000
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Remove install directory from system PATH on uninstall
  nsExec::ExecToLog 'powershell -ExecutionPolicy Bypass -Command "\
    $installDir = \"$INSTDIR\"; \
    $currentPath = [Environment]::GetEnvironmentVariable(\"Path\", \"Machine\"); \
    $newPath = ($currentPath -split \";\" | Where-Object { $_ -ne $installDir }) -join \";\"; \
    [Environment]::SetEnvironmentVariable(\"Path\", $newPath, \"Machine\"); \
    Write-Host \"Removed $installDir from system PATH\""'

  ; Broadcast WM_SETTINGCHANGE
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=1000
!macroend
