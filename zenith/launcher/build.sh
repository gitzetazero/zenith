#!/usr/bin/env bash
# Builds Zenith Launcher: the installer and the portable zip, from the official XMCL zip (checked against
# xmcl-pin.txt, fail closed), the bridge exe built in the same run and the licences. XMCL is bundled as-is.
#   zenith/launcher/build.sh <xmcl.zip> <zenith-bridge.exe> <outdir> <label> [--no-installer]
# Needs bash, 7z (or unzip), sha256sum and, unless --no-installer, makensis (NSIS 3) on PATH.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
zip=$1; bridge=$2; out=$3; label=$4; installer=${5:-yes}
pin() { sed -n "s/^$1=//p" "$here/xmcl-pin.txt" | tr -d '\r'; }
want=$(pin sha256); asset=$(pin asset)
[[ "$(basename "$zip")" == "$asset" ]] || { echo "::error::$zip is not the pinned asset $asset"; exit 1; }
got=$(sha256sum "$zip" | cut -d' ' -f1)
[[ "$got" == "$want" ]] || { echo "::error::SHA-256 of $asset is $got, the pin says $want"; exit 1; }
[[ -f "$bridge" ]] || { echo "::error::no bridge exe at $bridge"; exit 1; }

mkdir -p "$out"; out=$(cd "$out" && pwd)
work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
stage="$work/stage"; mkdir -p "$stage/licencias"
if command -v 7z > /dev/null; then 7z x -y -o"$stage" "$zip" > /dev/null; else unzip -q "$zip" -d "$stage"; fi
[[ -f "$stage/XMCL.exe" && -f "$stage/resources/app.asar" ]] || { echo "::error::XMCL.exe or resources/app.asar missing in the zip"; exit 1; }
cp "$bridge" "$stage/zenith-bridge.exe"
cp "$here/LICENSE.xmcl" "$stage/licencias/XMCL-LICENSE.txt"
cp "$repo/LICENSE" "$stage/licencias/ZENITH-LICENSE.txt"
cp "$repo/THIRD_PARTY_NOTICES.md" "$stage/licencias/THIRD_PARTY_NOTICES.md"
{ printf '\xEF\xBB\xBF'; cat "$here/AVISO.txt"; } > "$stage/licencias/AVISO.txt" # BOM: NSIS and Notepad read it as UTF-8

# Portable: one root folder, entry point `zenith-bridge.exe start`.
p="$work/portable/Zenith-Launcher-Portable"; mkdir -p "$p"
cp -r "$stage/." "$p/"
printf '@echo off\r\nstart "" "%%~dp0zenith-bridge.exe" start\r\n' > "$p/Zenith Launcher.cmd"
{ printf '\xEF\xBB\xBF'; sed 's/$/\r/' "$here/README-PORTABLE.txt"; } > "$p/README-PORTABLE.txt"
rm -f "$out/Zenith-Launcher-Portable-x64.zip"
(cd "$work/portable" && 7z a -tzip -mx=5 "$out/Zenith-Launcher-Portable-x64.zip" Zenith-Launcher-Portable > /dev/null)

if [[ "$installer" == --no-installer ]]; then ls -l "$out"; exit 0; fi
# The installer's file list and the uninstaller's come from the same staged tree, so the uninstaller deletes
# exactly the files the installer put there (and then only empty folders).
mkdir -p "$work/gen"
(cd "$stage" && find . -type f | LC_ALL=C sort) | while IFS= read -r f; do
  f=${f#./}; w=${f//\//\\}; d=$(dirname "$f"); wd=${d//\//\\}
  if [[ "$d" == . ]]; then printf 'SetOutPath "$INSTDIR"\n'; else printf 'SetOutPath "$INSTDIR\\%s"\n' "$wd"; fi
  printf 'File "${STAGE}\\%s"\n' "$w"
  printf 'Delete "$INSTDIR\\%s"\n' "$w" >> "$work/gen/delete.nsh"
done > "$work/gen/install.nsh"
(cd "$stage" && find . -mindepth 1 -type d | LC_ALL=C sort -r) | while IFS= read -r d; do
  d=${d#./}; printf 'RMDir "$INSTDIR\\%s"\n' "${d//\//\\}"
done > "$work/gen/rmdir.nsh"
win() { if command -v cygpath > /dev/null; then cygpath -w "$1"; else echo "$1"; fi; }
MSYS_NO_PATHCONV=1 makensis -V2 -INPUTCHARSET UTF8 \
  -DSTAGE="$(win "$stage")" -DGEN="$(win "$work/gen")" -DOUT="$(win "$out/Zenith-Launcher-Setup-x64.exe")" \
  -DVERSION="$label" "$(win "$here/installer.nsi")"
ls -l "$out"
