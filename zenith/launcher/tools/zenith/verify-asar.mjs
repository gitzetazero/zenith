// zenith: checks a built app.asar (fail closed). Usage: node tools/zenith/verify-asar.mjs <path/to/app.asar>
// - nothing that talks to XMCL's services, its Microsoft application id, or the removed features (AI agent, Together/P2P)
// - Zenith's data folder `.minecraftz` and the product name are present
// - prints the asar's package.json fields
import asar from '@electron/asar'
import { mkdtempSync, readdirSync, readFileSync, rmSync, statSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, extname } from 'node:path'

/** Strings that must not appear in any text file of the app. */
export const FORBIDDEN = [
  { name: 'xmcl.app', re: /xmcl\.app\b/i },
  { name: 'XMCL Microsoft client id prefix', re: /1363d629/ },
  { name: 'XMCL Agent', re: /XMCL Agent/ },
  { name: 'Together (the P2P service)', re: /\bTogether\b/ },
  { name: 'signaling', re: /signaling/i },
  { name: 'XMCL Application Insights key', re: /InstrumentationKey=/ },
  { name: 'XMCL account service', re: /XmclAccountService/ },
  { name: 'XMCL AI agent service', re: /AgentService/ },
  { name: 'Peer service', re: /PeerService/ },
]
/** Strings that must appear. */
export const REQUIRED = [
  { name: '.minecraftz', re: /\.minecraftz/ },
  { name: 'Zenith Launcher', re: /Zenith Launcher/ },
  { name: 'ZenithMarketService', re: /ZenithMarketService/ },
  { name: 'Zenith-Launcher user agent', re: /Zenith-Launcher/ },
]
const TEXT = new Set(['.js', '.mjs', '.cjs', '.json', '.html', '.css', '.yaml', '.yml', '.txt', '.md', '.svg', '.map'])

function* walk(dir) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name)
    const st = statSync(p)
    if (st.isDirectory()) yield* walk(p)
    else yield p
  }
}

export function scan(root) {
  const forbidden = new Map()
  const required = new Map(REQUIRED.map((r) => [r.name, 0]))
  let files = 0
  for (const file of walk(root)) {
    if (!TEXT.has(extname(file).toLowerCase())) continue
    files++
    const text = readFileSync(file, 'utf-8')
    for (const f of FORBIDDEN) {
      const m = f.re.exec(text)
      if (m) forbidden.set(f.name, [...(forbidden.get(f.name) ?? []), `${file.slice(root.length + 1)}: ...${text.slice(Math.max(0, m.index - 40), m.index + 60).replace(/\s+/g, ' ')}...`])
    }
    for (const r of REQUIRED) if (r.re.test(text)) required.set(r.name, required.get(r.name) + 1)
  }
  return { files, forbidden, required }
}

if (process.argv[1]?.endsWith('verify-asar.mjs')) {
  const file = process.argv[2]
  if (!file) { console.error('usage: node verify-asar.mjs <app.asar>'); process.exit(2) }
  const dir = mkdtempSync(join(tmpdir(), 'zenith-asar-'))
  try {
    asar.extractAll(file, dir)
    const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'))
    console.log('package.json:', JSON.stringify({ name: pkg.name, productName: pkg.productName, version: pkg.version, author: pkg.author, description: pkg.description, homepage: pkg.homepage, main: pkg.main }, null, 2))
    const { files, forbidden, required } = scan(dir)
    console.log(`scanned ${files} text files`)
    let failed = false
    for (const f of FORBIDDEN) {
      const hits = forbidden.get(f.name)
      console.log(`${hits ? 'FOUND  ' : 'absent '} ${f.name}${hits ? ` (${hits.length} file(s))` : ''}`)
      for (const h of hits ?? []) console.log(`         ${h}`)
      if (hits) failed = true
    }
    for (const [name, n] of required) {
      console.log(`${n ? 'present' : 'MISSING'} ${name} (${n} file(s))`)
      if (!n) failed = true
    }
    process.exit(failed ? 1 : 0)
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
}
