#!/bin/sh
# Reproduces the test of section 10 of SPIKE.md. Usage: build.sh <XMCL clone> <work folder> <26.3.json>
# Needs rustc, node 24 and esbuild (ESBUILD_DIR=<node_modules/esbuild> if it is not installed). Writes nothing outside <work folder>.
set -e
CLONE=$1; WORK=$2; VJSON=$3; HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$WORK/h" "$WORK/w/xmcl/versions/26.3" "$WORK/w/xmcl/instances/Zenith" "$WORK/jre/bin" "$WORK/jre2/bin" "$WORK/jre3/bin" "$WORK/jre21/bin"
cp "$VJSON" "$WORK/w/xmcl/versions/26.3/26.3.json"
cp "$HERE"/*.cjs "$WORK/h/"; cp "$HERE/../xmcl-args.txt" "$WORK/h/"
node "$HERE/bundle.cjs" "$CLONE" "$WORK"
rustc -O "$HERE/fakejava.rs" -o "$WORK/jre/bin/java.exe"
rustc -O --cfg winsub "$HERE/fakejava.rs" -o "$WORK/jre/bin/javaw.exe"
rustc -O --cfg old "$HERE/fakejava.rs" -o "$WORK/jre21/bin/java.exe"
cp "$WORK/jre/bin/java.exe" "$WORK/jre2/bin/java.exe"
cp "$WORK/jre/bin/javaw.exe" "$WORK/jre3/bin/javaw.exe"; cp "$WORK/jre/bin/javaw.exe" "$WORK/jre3/bin/java.exe"
cd "$WORK/h" && node e2e.cjs && node validate.cjs && node probe.cjs jre3
