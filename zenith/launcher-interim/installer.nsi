Unicode true
; Zenith Launcher (powered by XMCL): per-user installer of the unmodified XMCL plus Zenith's bridge.
; Inputs (-D): STAGE (the staged tree), GEN (install/delete/rmdir lists made by build.sh), OUT, VERSION.
!include "MUI2.nsh"

!define APP "Zenith Launcher"
!define FULL "Zenith Launcher (powered by XMCL)"
!define UNKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\ZenithLauncher"

Name "${FULL}"
OutFile "${OUT}"
RequestExecutionLevel user
InstallDir "$LOCALAPPDATA\Programs\Zenith Launcher"
InstallDirRegKey HKCU "Software\ZenithLauncher" "InstallDir"
SetCompressor /SOLID lzma
SetCompressorDictSize 64
BrandingText "${FULL}: independent project, not affiliated with XMCL, Mojang or Microsoft"

!define MUI_WELCOMEPAGE_TEXT "This wizard installs ${FULL}: the official XMCL launcher, unmodified, together with the Zenith bridge.$\r$\n$\r$\nZenith Launcher is an independent project: it is not affiliated with or endorsed by XMCL (Voxelum), Mojang or Microsoft.$\r$\n$\r$\nIt installs for your user only and needs no administrator rights."
!define MUI_LICENSEPAGE_CHECKBOX
!define MUI_FINISHPAGE_RUN "$INSTDIR\zenith-bridge.exe"
!define MUI_FINISHPAGE_RUN_PARAMETERS "start"
!define MUI_FINISHPAGE_RUN_TEXT "Run ${APP}"
!define MUI_UNCONFIRMPAGE_TEXT_TOP "Only the files that ${FULL} installed in its folder will be deleted. XMCL data (%APPDATA%\xmcl and its data folder), your worlds and Zenith data (%APPDATA%\zenith) are never touched."

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${STAGE}\licenses\NOTICE.txt"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "${APP} (required)" SecMain
  SectionIn RO
  SetOutPath "$INSTDIR"
  !include "${GEN}\install.nsh"
  SetOutPath "$INSTDIR"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateShortcut "$SMPROGRAMS\Zenith Launcher.lnk" "$INSTDIR\zenith-bridge.exe" "start" "" 0 SW_SHOWNORMAL "" "Opens XMCL with the Zenith instance"
  WriteRegStr HKCU "Software\ZenithLauncher" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNKEY}" "DisplayName" "${FULL}"
  WriteRegStr HKCU "${UNKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNKEY}" "Publisher" "Zenith (independent project)"
  WriteRegStr HKCU "${UNKEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNKEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKCU "${UNKEY}" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
  WriteRegStr HKCU "${UNKEY}" "URLInfoAbout" "https://github.com/gitzetazero/zenith"
  WriteRegDWORD HKCU "${UNKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNKEY}" "NoRepair" 1
SectionEnd

Section /o "Desktop shortcut" SecDesktop
  SetOutPath "$INSTDIR"
  CreateShortcut "$DESKTOP\Zenith Launcher.lnk" "$INSTDIR\zenith-bridge.exe" "start" "" 0 SW_SHOWNORMAL "" "Opens XMCL with the Zenith instance"
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecMain} "Unmodified XMCL and the Zenith bridge, in the chosen folder."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "Creates a "Zenith Launcher" shortcut on the desktop."
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; Only what this installer put in its own folder, its two shortcuts and its two registry keys.
Section "Uninstall"
  Delete "$SMPROGRAMS\Zenith Launcher.lnk"
  Delete "$DESKTOP\Zenith Launcher.lnk"
  !include "${GEN}\delete.nsh"
  Delete "$INSTDIR\Uninstall.exe"
  !include "${GEN}\rmdir.nsh"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${UNKEY}"
  DeleteRegKey HKCU "Software\ZenithLauncher"
  IfFileExists "$INSTDIR\*.*" 0 +2
    MessageBox MB_OK|MB_ICONINFORMATION "Files remain in $INSTDIR (is XMCL open, or are they files that are not Zenith files?). They were not deleted." /SD IDOK
SectionEnd
