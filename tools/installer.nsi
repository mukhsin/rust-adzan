; Installer NSIS untuk Jadwal Sholat.
; Build dari root proyek (lihat recipe `just bundle`):
;   makensis /DEXE_SRC=target/release/rust-adzan.exe \
;            /DBOOTSTRAP_SRC=<path MicrosoftEdgeWebview2Setup.exe> \
;            /DOUTDIR=target/nsis tools/installer.nsi

Unicode true
ManifestDPIAware true
!include "MUI2.nsh"
!include "LogicLib.nsh"

!define APPNAME "Jadwal Sholat"
!define EXENAME "rust-adzan.exe"
!define VERSION "0.2.0"
!define UNINSTKEY "JadwalSholat"
!define ARP_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${UNINSTKEY}"
!define WEBVIEW2_KEY "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"

Name "${APPNAME}"
OutFile "${OUTDIR}\Jadwal Sholat ${VERSION} x64-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\Jadwal Sholat"
RequestExecutionLevel user

; Ikon installer & uninstaller dari aplikasi.
Icon "${ICON_SRC}"
UninstallIcon "${ICON_SRC}"

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "Indonesian"

Section "Install"
  ; WebView2 Runtime: pasang bootstrapper jika belum ada (cek HKLM/HKCU).
  ReadRegStr $0 HKLM "${WEBVIEW2_KEY}" "pv"
  ReadRegStr $1 HKCU "${WEBVIEW2_KEY}" "pv"
  ${If} $0 == ""
  ${AndIf} $1 == ""
    DetailPrint "Memasang WebView2 Runtime…"
    InitPluginsDir
    SetOutPath "$PLUGINSDIR"
    File "${BOOTSTRAP_SRC}"
    ExecWait '"$PLUGINSDIR\MicrosoftEdgeWebview2Setup.exe" /install'
  ${EndIf}

  SetOutPath "$INSTDIR"
  File "/oname=${EXENAME}" "${EXE_SRC}"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Shortcut Start Menu + Desktop.
  CreateDirectory "$SMPROGRAMS\${APPNAME}"
  CreateShortCut "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk" "$INSTDIR\${EXENAME}"
  CreateShortCut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\${EXENAME}"

  ; Entri Add/Remove Programs.
  WriteRegStr HKCU "${ARP_KEY}" "DisplayName" "${APPNAME}"
  WriteRegStr HKCU "${ARP_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${ARP_KEY}" "Publisher" "${APPNAME}"
  WriteRegStr HKCU "${ARP_KEY}" "DisplayIcon" "$INSTDIR\${EXENAME}"
  WriteRegStr HKCU "${ARP_KEY}" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegDWORD HKCU "${ARP_KEY}" "EstimatedSize" 12
  WriteRegStr HKCU "Software\${UNINSTKEY}" "InstallDir" "$INSTDIR"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\${EXENAME}"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$DESKTOP\${APPNAME}.lnk"
  Delete "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk"
  RMDir "$SMPROGRAMS\${APPNAME}"
  DeleteRegKey HKCU "${ARP_KEY}"
  DeleteRegKey HKCU "Software\${UNINSTKEY}"
SectionEnd
