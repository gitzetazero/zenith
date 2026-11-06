// zenith: downloads that reach the instance folder only when their checksum matches the source's index
// (same rules as Horizon's launcher/src/browser/download.rs): https, known hosts, a plain file name, size and hash.
import { createHash } from 'crypto'
import { mkdir, rename, rm, writeFile } from 'fs/promises'
import { basename, join } from 'path'
import type { MarketKind } from '@xmcl/runtime-api'
import { MODRINTH_FILE_HOST } from './modrinth'
import { REGISTRY_FILE_HOST } from './registry'

export interface RemoteFile {
  url: string
  name: string
  size?: number
  sha256?: string
  sha512?: string
  sha1?: string
}

/** Bigger than any pack these sources serve (Modrinth caps uploads at 500 MiB); the registry caps files at 64 MiB. */
export const MAX_FILE_BYTES = 1 << 30
export const MAX_REGISTRY_FILE_BYTES = 64 << 20

/** `https://github.com/<owner>/<repo>/releases/download/<tag>/<name>`: a release asset of a project's own repository. */
export function githubReleaseAsset(url: string): boolean {
  const rest = url.startsWith('https://github.com/') ? url.slice('https://github.com/'.length) : undefined
  if (!rest) return false
  const parts = rest.split('/')
  const plain = (p: string) => /^[A-Za-z0-9\-_.+%]+$/.test(p) && p !== '.' && p !== '..'
  return parts.length === 6 && parts[2] === 'releases' && parts[3] === 'download' && [parts[0], parts[1], parts[4], parts[5]].every(plain)
}

/** One plain file name: no folders, `..`, hidden names or reserved characters. */
export function safeName(name: string): string | undefined {
  if (!name || name.length > 200 || basename(name) !== name || name.startsWith('.') || /[\\/:*?"<>|\x00-\x1f]/.test(name)) return undefined
  return name
}

/** Why a file cannot be downloaded at all, before any byte is read. */
export function refuse(file: RemoteFile): string | undefined {
  if (!file.url.startsWith(MODRINTH_FILE_HOST) && !file.url.startsWith(REGISTRY_FILE_HOST) && !githubReleaseAsset(file.url)) {
    return `${file.url} is not on Modrinth's or Zenith's file servers, or a GitHub release.`
  }
  if (!safeName(file.name)) return `${JSON.stringify(file.name)} is not a plain file name.`
  if (!file.sha256 && !file.sha512 && !file.sha1) return `${file.name} lists no checksum.`
  if (file.size !== undefined && file.size > MAX_FILE_BYTES) return `${file.name} is too large.`
  return undefined
}

export type FetchBytes = (url: string, init?: { headers?: Record<string, string> }) => Promise<{
  ok: boolean
  status: number
  arrayBuffer(): Promise<ArrayBuffer>
}>

function digest(algo: 'sha256' | 'sha512' | 'sha1', bytes: Uint8Array) {
  return createHash(algo).update(bytes).digest('hex')
}

/** Downloads, checks size and the strongest checksum listed, then renames into place. Returns the final path and bytes. */
export async function downloadChecked(
  fetchBytes: FetchBytes,
  file: RemoteFile,
  directory: string,
  userAgent: string,
  maxBytes = MAX_FILE_BYTES,
): Promise<{ path: string; bytes: Uint8Array }> {
  const refusal = refuse(file)
  if (refusal) throw new Error(refusal)
  const res = await fetchBytes(file.url, { headers: { 'user-agent': userAgent } })
  if (!res.ok) throw new Error(`${file.name}: the server answered ${res.status}.`)
  const bytes = new Uint8Array(await res.arrayBuffer())
  if (bytes.length > maxBytes) throw new Error(`${file.name} is too large.`)
  if (file.size !== undefined && file.size > 0 && bytes.length !== file.size) {
    throw new Error(`${file.name}: expected ${file.size} bytes, got ${bytes.length}.`)
  }
  const wanted: Array<['sha256' | 'sha512' | 'sha1', string | undefined]> = [['sha256', file.sha256], ['sha512', file.sha512], ['sha1', file.sha1]]
  const [algo, expected] = wanted.find(([, h]) => h) as ['sha256' | 'sha512' | 'sha1', string]
  if (digest(algo, bytes) !== expected.toLowerCase()) throw new Error(`${file.name}: the ${algo.toUpperCase()} does not match the source's.`)
  await mkdir(directory, { recursive: true })
  const final = join(directory, file.name)
  const part = join(directory, `.${file.name}.part`)
  try {
    await writeFile(part, bytes)
    await rename(part, final)
  } catch (e) {
    await rm(part, { force: true })
    throw e
  }
  return { path: final, bytes }
}

/** Where a kind goes inside a Zenith instance folder. Native mods and modpacks cannot be installed yet. */
export function destinationFor(kind: MarketKind, instancePath: string, world?: string): string {
  switch (kind) {
    case 'mod': return join(instancePath, 'horizon-mods')
    case 'shader': return join(instancePath, 'shaderpacks')
    case 'resourcepack': return join(instancePath, 'resourcepacks')
    case 'datapack': {
      if (!world || !safeName(world)) throw new Error('Pick the world to install the data pack into.')
      return join(instancePath, 'saves', world, 'datapacks')
    }
    case 'native-mod':
      throw new Error('Native mods are compiled into the game and need a Zenith rebuild: this launcher cannot install them yet.')
    default:
      throw new Error('This launcher cannot install modpacks yet.')
  }
}
