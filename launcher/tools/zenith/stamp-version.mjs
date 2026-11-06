#!/usr/bin/env node
// zenith: stamps the release's version into the Electron app's package.json, so Settings > About and
// the executable's metadata name the build. XMCL's base version alone never changes between releases.
// Format: <base>-zenith0.1.<run>: 0.73.1-zenith0.1.25 is XMCL 0.73.1, the Zenith fork's own line 0.1,
// and the release workflow's run number (one per release, so it always advances).
import { readFileSync, writeFileSync } from 'node:fs'
import { pathToFileURL } from 'node:url'

/** `base` is XMCL's numeric version; `run` is the release workflow's run number. */
export function stampedVersion(base, run) {
  if (!/^\d+\.\d+\.\d+$/.test(base)) throw new Error(`not an XMCL base version: ${base}`)
  if (!/^\d+$/.test(String(run))) throw new Error(`not a release number: ${run}`)
  return `${base}-zenith0.1.${run}`
}

/** Rewrites `file`'s `version`, keeping the numeric part it has now. Returns the new version. */
export function stampPackageJson(file, run) {
  const pkg = JSON.parse(readFileSync(file, 'utf-8'))
  const base = String(pkg.version).replace(/-.*$/, '')
  pkg.version = stampedVersion(base, run)
  writeFileSync(file, `${JSON.stringify(pkg, null, 2)}\n`)
  return pkg.version
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [file, run] = process.argv.slice(2)
  if (!file || !run) {
    console.error('usage: stamp-version.mjs <package.json> <run-number>')
    process.exit(2)
  }
  console.log(`version ${stampPackageJson(file, run)}`)
}
