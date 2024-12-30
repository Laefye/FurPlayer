!define VC_REG_KEY "HKLM\\SOFTWARE\\Microsoft\\VisualStudio\\14.0\\VC\\Runtimes\\x64"
!define VC_REG_VALUE "Installed"

!macro NSIS_HOOK_POSTINSTALL
  ReadRegDWORD $0 "HKLM" "${VC_REG_KEY}" "${VC_REG_VALUE}"
  StrCmp $0 1 +2
  Goto InstallVC

  MessageBox MB_OK "Visual C++ Redistributable already installed."
  Return

  InstallVC:
  ExecWait '"$PLUGINSDIR\\vc_redist.x64.exe" /quiet /norestart'
!macroend

