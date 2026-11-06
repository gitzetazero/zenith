// zenith: the registry client. It reads the same public index Horizon's launcher reads (docs/api/mod-index.md of the engine):
// one https file, cached on disk, `schema: 1`. Unknown kinds are kept but never offered.
import { mkdir, readFile, writeFile } from 'fs/promises'
import { dirname } from 'path'
import type { MarketKind, MarketProject, MarketQuery, MarketVersion } from '@xmcl/runtime-api'
import { installable, registrySupport } from './support'

/** The registry's published index (GitHub Pages of the registry repository). */
export const REGISTRY_INDEX_URL = 'https://horizon-rust.github.io/horizon-mods/index.json'
export const REGISTRY_SITE = 'https://playhorizon.dev/mods'
/** Where the registry's own release assets live; a project's repository releases are accepted too (see install.ts). */
export const REGISTRY_FILE_HOST = 'https://github.com/horizon-rust/horizon-mods/releases/download/'
export const MAX_INDEX_BYTES = 16 << 20
export const INDEX_TTL_MS = 10 * 60 * 1000

export interface RegistryFile { name: string; url: string; size: number; sha256: string; signature?: string }
export interface RegistryVersion {
  version: string
  published: string
  changelog?: string
  game: string
  api?: string
  side?: string
  capabilities?: string[]
  file: RegistryFile
}
export interface RegistryProject {
  id: string
  kind: string
  name: string
  summary: string
  description: string
  authors: string[]
  owners?: string[]
  license: string
  source: string
  icon?: string
  key?: string
  downloads?: number
  stars?: number
  categories?: string[]
  tags?: string[]
  versions: RegistryVersion[]
}
export interface RegistryRevoked { id: string; sha256?: string; reason: string; date?: string }
export interface RegistryIndex { schema: number; generated?: string; projects: RegistryProject[]; revoked: RegistryRevoked[] }

const KINDS: Record<string, MarketKind> = {
  'mod': 'mod',
  'native-mod': 'native-mod',
  'modpack': 'modpack',
  'resourcepack': 'resourcepack',
  // Both go in shaderpacks/: Zenith reads `.hshader` packs and shaders/-folder zips.
  'shader': 'shader',
  'shaderpack': 'shader',
  'datapack': 'datapack',
}

export function kindOf(kind: string): MarketKind | undefined {
  return KINDS[kind]
}

/** True for what the launcher can write to disk today. */
export function canInstallKind(kind: MarketKind): boolean {
  return kind === 'mod' || kind === 'shader' || kind === 'resourcepack' || kind === 'datapack'
}

/** Parses and sanity-checks the index; refuses a newer schema rather than misreading it. */
export function parseIndex(text: string): RegistryIndex {
  const json = JSON.parse(text)
  if (!json || typeof json !== 'object' || !Array.isArray(json.projects)) {
    throw new Error('The mod registry answered something that is not an index.')
  }
  if (typeof json.schema !== 'number' || json.schema > 1) {
    throw new Error('The mod registry uses a newer format: update Zenith Launcher.')
  }
  const projects: RegistryProject[] = json.projects.filter((p: any) =>
    p && typeof p.id === 'string' && /^[a-z0-9_]{1,64}$/.test(p.id) && typeof p.kind === 'string' && typeof p.name === 'string' &&
    Array.isArray(p.versions) &&
    p.versions.every((v: any) => v && typeof v.game === 'string' && v.file && typeof v.file.url === 'string' && typeof v.file.sha256 === 'string'))
  return { schema: json.schema, generated: json.generated, projects, revoked: Array.isArray(json.revoked) ? json.revoked : [] }
}

export function revokedReason(index: RegistryIndex, id: string, sha256: string): string | undefined {
  const r = index.revoked.find((r) => r.id === id && (!r.sha256 || r.sha256 === sha256))
  return r ? `taken down: ${r.reason}` : undefined
}

export type FetchLike = (
  url: string,
  init?: { headers?: Record<string, string>; signal?: AbortSignal },
) => Promise<{ ok: boolean; status: number; text(): Promise<string> }>

export interface IndexLoaderOptions {
  fetch: FetchLike
  userAgent: string
  cacheFile: string
  url?: string
  ttlMs?: number
  now?: () => number
}

/** Fetches the index (at most every ten minutes), keeps it on disk, and falls back to the copy kept last time. */
export function createIndexLoader(o: IndexLoaderOptions) {
  const url = o.url ?? REGISTRY_INDEX_URL
  if (!url.startsWith('https://')) throw new Error('The mod registry must be reached over https.')
  const now = o.now ?? Date.now
  let loaded: { at: number; index: RegistryIndex } | undefined
  return async function load(force = false): Promise<RegistryIndex> {
    if (!force && loaded && now() - loaded.at < (o.ttlMs ?? INDEX_TTL_MS)) return loaded.index
    try {
      const res = await o.fetch(url, { headers: { 'user-agent': o.userAgent, accept: 'application/json' } })
      if (!res.ok) throw new Error(`The mod registry answered ${res.status}.`)
      const text = await res.text()
      if (text.length > MAX_INDEX_BYTES) throw new Error('The mod registry index is too large.')
      const index = parseIndex(text)
      await mkdir(dirname(o.cacheFile), { recursive: true }).then(() => writeFile(o.cacheFile, text)).catch(() => undefined)
      loaded = { at: now(), index }
      return index
    } catch (e) {
      const kept = await readFile(o.cacheFile, 'utf-8').then(parseIndex).catch(() => undefined)
      if (!kept) throw e
      loaded = { at: now(), index: kept }
      return kept
    }
  }
}

export function projectOf(p: RegistryProject, game: string): MarketProject | undefined {
  const kind = kindOf(p.kind)
  if (!kind) return undefined
  const gameVersions = [...new Set(p.versions.map((v) => v.game))]
  const support = registrySupport(kind, gameVersions, game)
  return {
    source: 'zenith',
    id: p.id,
    slug: p.id,
    title: p.name,
    summary: p.summary,
    author: p.authors.join(', '),
    downloads: p.downloads ?? 0,
    icon: p.icon,
    license: p.license,
    kind,
    page: `${REGISTRY_SITE}/${p.id}`,
    gameVersions,
    categories: [...(p.categories ?? []), ...(p.tags ?? [])],
    support,
    installable: installable(support) && canInstallKind(kind),
    ...(kind === 'native-mod' ? { requiresRebuild: true } : {}),
  }
}

function matches(p: RegistryProject, text: string) {
  const hay = [p.id, p.name, p.summary, p.authors.join(' '), (p.tags ?? []).join(' ')].join(' ').toLowerCase()
  return text.split(/\s+/).filter(Boolean).every((w) => hay.includes(w.toLowerCase()))
}

/** The projects of the query's kind whose words match, for this game unless `everything`; most downloaded first. */
export function searchIndex(index: RegistryIndex, query: MarketQuery, game: string): MarketProject[] {
  const day = (p: RegistryProject, newest: boolean) => {
    const days = p.versions.map((v) => v.published)
    return (newest ? days.reduce((a, b) => (a < b ? a : b)) : days.reduce((a, b) => (a > b ? a : b))) || ''
  }
  const downloads = (p: RegistryProject) => p.downloads ?? 0
  let found = index.projects
    .filter((p) => kindOf(p.kind) === query.kind && matches(p, query.text ?? ''))
    .filter((p) => query.everything || p.versions.some((v) => v.game === game))
  if (query.sort === 'newest') found = found.sort((a, b) => day(b, true).localeCompare(day(a, true)) || downloads(b) - downloads(a))
  else if (query.sort === 'updated') found = found.sort((a, b) => day(b, false).localeCompare(day(a, false)) || downloads(b) - downloads(a))
  else found = found.sort((a, b) => downloads(b) - downloads(a))
  const projects = found.map((p) => projectOf(p, game)!)
  const offset = query.offset ?? 0
  return projects.slice(offset, offset + (query.limit ?? 30))
}

export function versionsOf(index: RegistryIndex, p: RegistryProject, game: string): MarketVersion[] {
  const versions = p.versions.map((v): MarketVersion => ({
    id: v.version,
    number: v.version,
    published: v.published,
    gameVersions: [v.game],
    loaders: ['zenith'],
    fileName: v.file.name,
    size: v.file.size,
    blocked: revokedReason(index, p.id, v.file.sha256),
  }))
  const picked = versions.find((v) => !v.blocked && v.gameVersions.includes(game))
  if (picked) picked.picked = true
  return versions
}
