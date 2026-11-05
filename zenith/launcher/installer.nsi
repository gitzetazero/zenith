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
BrandingText "${FULL}: proyecto independiente, no afiliado a XMCL, Mojang ni Microsoft"

!define MUI_WELCOMEPAGE_TEXT "Este asistente instala ${FULL}: el lanzador oficial XMCL, sin modificar, junto con el puente de Zenith.$\r$\n$\r$\nZenith Launcher es un proyecto independiente: no está afiliado ni respaldado por XMCL (Voxelum), Mojang ni Microsoft.$\r$\n$\r$\nSe instala solo para tu usuario y no necesita permisos de administrador."
!define MUI_LICENSEPAGE_CHECKBOX
!define MUI_FINISHPAGE_RUN "$INSTDIR\zenith-bridge.exe"
!define MUI_FINISHPAGE_RUN_PARAMETERS "start"
!define MUI_FINISHPAGE_RUN_TEXT "Ejecutar ${APP}"
!define MUI_UNCONFIRMPAGE_TEXT_TOP "Se borrarán solo los archivos que instaló ${FULL} en su carpeta. Nunca se tocan los datos de XMCL (%APPDATA%\xmcl y su carpeta de datos), tus mundos ni los datos de Zenith (%APPDATA%\zenith)."

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${STAGE}\licencias\AVISO.txt"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "Spanish"

Section "${APP} (obligatorio)" SecMain
  SectionIn RO
  SetOutPath "$INSTDIR"
  !include "${GEN}\install.nsh"
  SetOutPath "$INSTDIR"
  WriteUninstaller "$INSTDIR\Desinstalar.exe"
  CreateShortcut "$SMPROGRAMS\Zenith Launcher.lnk" "$INSTDIR\zenith-bridge.exe" "start" "" 0 SW_SHOWNORMAL "" "Abre XMCL con la instancia Zenith"
  WriteRegStr HKCU "Software\ZenithLauncher" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNKEY}" "DisplayName" "${FULL}"
  WriteRegStr HKCU "${UNKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNKEY}" "Publisher" "Zenith (proyecto independiente)"
  WriteRegStr HKCU "${UNKEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNKEY}" "UninstallString" '"$INSTDIR\Desinstalar.exe"'
  WriteRegStr HKCU "${UNKEY}" "QuietUninstallString" '"$INSTDIR\Desinstalar.exe" /S'
  WriteRegStr HKCU "${UNKEY}" "URLInfoAbout" "https://github.com/gitzetazero/zenith"
  WriteRegDWORD HKCU "${UNKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNKEY}" "NoRepair" 1
SectionEnd

Section /o "Acceso directo en el escritorio" SecDesktop
  SetOutPath "$INSTDIR"
  CreateShortcut "$DESKTOP\Zenith Launcher.lnk" "$INSTDIR\zenith-bridge.exe" "start" "" 0 SW_SHOWNORMAL "" "Abre XMCL con la instancia Zenith"
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecMain} "XMCL sin modificar y el puente de Zenith, en la carpeta elegida."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "Crea un acceso directo «Zenith Launcher» en el escritorio."
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; Only what this installer put in its own folder, its two shortcuts and its two registry keys.
Section "Uninstall"
  Delete "$SMPROGRAMS\Zenith Launcher.lnk"
  Delete "$DESKTOP\Zenith Launcher.lnk"
  !include "${GEN}\delete.nsh"
  Delete "$INSTDIR\Desinstalar.exe"
  !include "${GEN}\rmdir.nsh"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${UNKEY}"
  DeleteRegKey HKCU "Software\ZenithLauncher"
  IfFileExists "$INSTDIR\*.*" 0 +2
    MessageBox MB_OK|MB_ICONINFORMATION "Quedan archivos en $INSTDIR (¿XMCL abierto, o archivos que no son de Zenith?). No se han borrado." /SD IDOK
SectionEnd
