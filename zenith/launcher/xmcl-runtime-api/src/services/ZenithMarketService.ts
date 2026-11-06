// zenith: the Zenith Market. Sources are Zenith's registry (the same index Horizon's launcher reads, see
// docs/api/mod-index.md of the engine) and Modrinth. Only what can run in Zenith is installable.
import { ServiceKey } from './Service'

/** Registry kinds (`.hmod`, `.hnmod`, `.hshader`/shader zips, resource packs, data packs, `.hpack`). */
export type MarketKind = 'mod' | 'native-mod' | 'shader' | 'resourcepack' | 'datapack' | 'modpack'
export type MarketSource = 'zenith' | 'modrinth'

/** How well a project runs in Zenith, as Horizon words it: Works / Partly / Doesn't run. */
export interface MarketSupport {
  level: 'works' | 'partial' | 'no'
  reason?: string
}

export interface MarketProject {
  source: MarketSource
  id: string
  slug: string
  title: string
  summary: string
  author: string
  downloads: number
  icon?: string
  license: string
  kind: MarketKind
  /** The project's page on its source. */
  page: string
  gameVersions: string[]
  categories: string[]
  support: MarketSupport
  /** False for what the launcher cannot install (Java mods, native mods until the game is rebuilt, modpacks). */
  installable: boolean
  /** Native mods are compiled into the game: they need a Zenith rebuild, which this launcher does not do yet. */
  requiresRebuild?: boolean
}

export interface MarketQuery {
  text: string
  kind: MarketKind
  /** Default: every source that has the kind. */
  sources?: MarketSource[]
  /** Also list what does not run, each marked with its badge (never installable). */
  everything?: boolean
  sort?: 'downloads' | 'newest' | 'updated'
  offset?: number
  limit?: number
}

export interface MarketSearchResult {
  projects: MarketProject[]
  /** A failing source never hides the other one. */
  errors: Partial<Record<MarketSource, string>>
}

export interface MarketVersion {
  id: string
  number: string
  published: string
  gameVersions: string[]
  loaders: string[]
  fileName: string
  size: number
  /** Why this version cannot be installed (taken down, ...). */
  blocked?: string
  /** The version an install would take. */
  picked?: boolean
}

export interface MarketProjectDetail {
  project: MarketProject
  /** Markdown (registry) or the summary (Modrinth). */
  description: string
  versions: MarketVersion[]
  /** Who made it, whether it is signed, what it may do. */
  notes: string[]
}

export interface MarketInstallOptions {
  source: MarketSource
  projectId: string
  versionId?: string
  /** The Zenith instance folder. */
  instancePath: string
  /** Data packs go into `saves/<world>/datapacks`. */
  world?: string
  everything?: boolean
}

export interface MarketInstallResult {
  path: string
  fileName: string
  kind: MarketKind
  notes: string[]
}

export interface ZenithMarketService {
  search(query: MarketQuery): Promise<MarketSearchResult>
  getProject(source: MarketSource, id: string, kind?: MarketKind): Promise<MarketProjectDetail>
  install(options: MarketInstallOptions): Promise<MarketInstallResult>
}

export const ZenithMarketServiceKey: ServiceKey<ZenithMarketService> = 'ZenithMarketService'
