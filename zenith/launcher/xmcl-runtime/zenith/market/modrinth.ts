// zenith: Modrinth's public API (api.modrinth.com/v2, no key). Only the kinds Zenith can use are searched:
// resource packs, data packs and shaders; Java mods appear only with "everything" and never install.
import type { MarketKind, MarketProject, MarketQuery, MarketVersion } from '@xmcl/runtime-api'
import { installable, JAVA_LOADERS, modrinthSupport } from './support'

export const MODRINTH_API = 'https://api.modrinth.com/v2'
export const MODRINTH_SITE = 'https://modrinth.com'
export const MODRINTH_FILE_HOST = 'https://cdn.modrinth.com/'

/** Modrinth's project type for a kind (a data pack says it is a `mod` with the `datapack` category). */
function projectType(kind: MarketKind): string {
  switch (kind) {
    case 'resourcepack': return 'resourcepack'
    case 'shader': return 'shader'
    case 'modpack': return 'modpack'
    default: return 'mod'
  }
}

/** The kinds Modrinth is searched for. `mod` is Java mods: only listed with "everything", as Doesn't run. */
export function modrinthHasKind(kind: MarketKind, everything?: boolean): boolean {
  if (kind === 'resourcepack' || kind === 'datapack' || kind === 'shader') return true
  return kind === 'mod' && !!everything
}

/** The search's facets (AND of ORs). */
export function facets(query: MarketQuery, game: string): string {
  const all: string[][] = [[`project_type:${projectType(query.kind)}`]]
  if (query.kind === 'datapack') all.push(['categories:datapack'])
  if (query.kind === 'mod') all.push(JAVA_LOADERS.slice(0, 4).map((l) => `categories:${l}`))
  if (!query.everything) all.push([`versions:${game}`])
  return JSON.stringify(all)
}

const INDEX: Record<NonNullable<MarketQuery['sort']> | 'relevance', string> = {
  downloads: 'downloads',
  newest: 'newest',
  updated: 'updated',
  relevance: 'relevance',
}

export function searchUrl(query: MarketQuery, game: string): string {
  const params = new URLSearchParams({
    facets: facets(query, game),
    index: INDEX[query.sort ?? (query.text ? 'relevance' : 'downloads')],
    offset: String(query.offset ?? 0),
    limit: String(Math.min(Math.max(query.limit ?? 30, 1), 100)),
  })
  if (query.text) params.set('query', query.text)
  return `${MODRINTH_API}/search?${params}`
}

const strings = (v: unknown): string[] => (Array.isArray(v) ? v.filter((s): s is string => typeof s === 'string') : [])

/** The kind of a project: a `mod` carrying the `datapack` category is a data pack. */
function kindFrom(type: string, categories: string[], wanted: MarketKind): MarketKind | undefined {
  if (type === 'resourcepack') return 'resourcepack'
  if (type === 'shader') return 'shader'
  if (type === 'modpack') return 'modpack'
  if (type === 'mod') return wanted === 'datapack' ? 'datapack' : 'mod'
  return undefined
}

/** A search hit. Its `categories` hold the loaders too. */
export function parseHit(hit: any, wanted: MarketKind, game: string): MarketProject | undefined {
  const kind = kindFrom(String(hit?.project_type), strings(hit?.categories), wanted)
  if (!kind || typeof hit.project_id !== 'string') return undefined
  const gameVersions = strings(hit.versions)
  const loaders = strings(hit.categories)
  const support = modrinthSupport(kind, gameVersions, loaders, game)
  return {
    source: 'modrinth',
    id: hit.project_id,
    slug: String(hit.slug ?? ''),
    title: String(hit.title ?? ''),
    summary: String(hit.description ?? ''),
    author: String(hit.author ?? ''),
    downloads: Number(hit.downloads) || 0,
    icon: hit.icon_url || undefined,
    license: String(hit.license ?? ''),
    kind,
    page: `${MODRINTH_SITE}/${kind === 'datapack' ? 'datapack' : projectType(kind)}/${hit.slug}`,
    gameVersions,
    categories: loaders.filter((c) => !JAVA_LOADERS.includes(c) && c !== 'minecraft' && c !== 'datapack'),
    support,
    installable: installable(support),
  }
}

export interface ModrinthFile { url: string; filename: string; size: number; primary: boolean; sha512?: string; sha1?: string }
export interface ModrinthVersion {
  id: string
  number: string
  published: string
  status?: string
  gameVersions: string[]
  loaders: string[]
  files: ModrinthFile[]
}

export function parseVersion(v: any): ModrinthVersion | undefined {
  if (!v || typeof v.id !== 'string' || !Array.isArray(v.files)) return undefined
  return {
    id: v.id,
    number: String(v.version_number ?? ''),
    published: String(v.date_published ?? ''),
    status: v.status,
    gameVersions: strings(v.game_versions),
    loaders: strings(v.loaders),
    files: v.files
      .filter((f: any) => typeof f?.url === 'string' && typeof f?.filename === 'string')
      .map((f: any): ModrinthFile => ({
        url: f.url,
        filename: f.filename,
        size: Number(f.size) || 0,
        primary: !!f.primary,
        sha512: typeof f.hashes?.sha512 === 'string' ? f.hashes.sha512.toLowerCase() : undefined,
        sha1: typeof f.hashes?.sha1 === 'string' ? f.hashes.sha1.toLowerCase() : undefined,
      })),
  }
}

export const primaryFile = (v: ModrinthVersion): ModrinthFile | undefined => v.files.find((f) => f.primary) ?? v.files[0]

/**
 * The version to install, newest first in `versions`: for Zenith's game, else (with `everything`) the newest that is still
 * a resource pack / data pack / shader. Data pack versions must carry the `datapack` loader (most data packs also ship a mod build).
 */
export function pickVersion(kind: MarketKind, versions: ModrinthVersion[], game: string, everything?: boolean): ModrinthVersion | undefined {
  const usable = versions.filter((v) => (!v.status || v.status === 'listed') && primaryFile(v))
  const ofKind = kind === 'datapack' ? usable.filter((v) => v.loaders.includes('datapack')) : usable
  return ofKind.find((v) => v.gameVersions.includes(game)) ?? (everything || kind === 'resourcepack' ? ofKind[0] : undefined)
}

export function toMarketVersion(v: ModrinthVersion, picked: boolean): MarketVersion {
  const file = primaryFile(v)
  return {
    id: v.id,
    number: v.number,
    published: v.published,
    gameVersions: v.gameVersions,
    loaders: v.loaders,
    fileName: file?.filename ?? '',
    size: file?.size ?? 0,
    picked,
  }
}
