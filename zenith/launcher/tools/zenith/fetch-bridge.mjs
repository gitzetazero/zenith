// zenith: downloads the Zenith bridge from the latest `nightly` release of gitzetazero/zenith and verifies it
// (fail closed): the Ed25519 signature of SHA256SUMS against the pinned Zenith release key, then the exe's SHA-256
// against that signed list. Usage: node tools/zenith/fetch-bridge.mjs <outDir>
// Same format as crates/horizon-release-sign in gitzetazero/zenith: signed message = "horizon release <tag>\n" + SHA256SUMS.
import { createHash, createPublicKey, verify } from 'node:crypto'
import { mkdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

// zenith/bridge/zenith-release-keys.txt in gitzetazero/zenith (base64 of the raw 32-byte Ed25519 public key)
export const RELEASE_KEY = 'eutn+UZqdlJIEeLkP2SSkiNS9IDkAF3TImYT81fdWTs='
export const TAG = 'nightly'
export const ASSET = 'zenith-bridge-windows-x64.exe'
const BASE = `https://github.com/gitzetazero/zenith/releases/download/${TAG}`

export function verifySums(sums, signatureText, keyBase64 = RELEASE_KEY, tag = TAG) {
  const spki = Buffer.concat([Buffer.from('302a300506032b6570032100', 'hex'), Buffer.from(keyBase64, 'base64')])
  const key = createPublicKey({ key: spki, format: 'der', type: 'spki' })
  const message = Buffer.concat([Buffer.from(`horizon release ${tag}\n`), sums])
  const ok = signatureText.split(/\r?\n/).map((l) => l.trim()).filter(Boolean)
    .some((line) => verify(null, message, key, Buffer.from(line, 'base64')))
  if (!ok) throw new Error('SHA256SUMS signature does not match the Zenith release key')
}

export function expectedHash(sums, name) {
  const hits = sums.toString('utf-8').split(/\r?\n/)
    .map((l) => /^([0-9a-fA-F]{64}) [ *](.+)$/.exec(l)).filter((m) => m && m[2] === name)
  if (hits.length !== 1) throw new Error(`expected exactly one ${name} in SHA256SUMS, found ${hits.length}`)
  return hits[0][1].toLowerCase()
}

async function get(name) {
  const r = await fetch(`${BASE}/${name}`)
  if (!r.ok) throw new Error(`${name}: HTTP ${r.status}`)
  return Buffer.from(await r.arrayBuffer())
}

export async function main(outDir) {
  const [sums, sig] = await Promise.all([get('SHA256SUMS'), get('SHA256SUMS.sig')])
  verifySums(sums, sig.toString('utf-8'))
  const want = expectedHash(sums, ASSET)
  const exe = await get(ASSET)
  const got = createHash('sha256').update(exe).digest('hex')
  if (got !== want) throw new Error(`${ASSET}: SHA-256 ${got} does not match the signed list (${want})`)
  mkdirSync(outDir, { recursive: true })
  writeFileSync(join(outDir, ASSET), exe)
  console.log(`${ASSET} ${exe.length} bytes sha256=${got} (signature of SHA256SUMS verified)`)
}

if (process.argv[1]?.endsWith('fetch-bridge.mjs')) {
  const out = process.argv[2]
  if (!out) { console.error('usage: node fetch-bridge.mjs <outDir>'); process.exit(2) }
  main(out).catch((e) => { console.error(`::error::${e.message}`); process.exit(1) })
}
