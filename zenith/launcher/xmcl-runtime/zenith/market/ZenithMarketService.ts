// zenith: the Zenith Market service. See xmcl-runtime-api/src/services/ZenithMarketService.ts for the contract.
import type {
  MarketInstallOptions, MarketInstallResult, MarketKind, MarketProject, MarketProjectDetail, MarketQuery, MarketSearchResult,
  MarketSource, MarketVersion, ZenithMarketService as IZenithMarketService,
} from '@xmcl/runtime-api'
import { ZenithMarketServiceKey } from '@xmcl/runtime-api'
import { existsSync } from 'fs'
import { join } from 'path'
import { Inject, LauncherAppKey } from '~/app'
import { InstanceService } from '~/instance'
import { AbstractService, ExposeServiceKey } from '~/service'
import { LauncherApp } from '../../app/LauncherApp'
import { ZENITH_MINECRAFT } from '../instance'
import { downloadChecked, destinationFor, MAX_REGISTRY_FILE_BYTES, safeName } from './install'
import { verifyMinisign } from './minisign'
import {
  MODRINTH_API, modrinthHasKind, parseHit, parseVersion, pickVersion, primaryFile, searchUrl, toMarketVersion,
} from './modrinth'
import {
  canInstallKind, createIndexLoader, kindOf, projectOf, revokedReason, searchIndex, versionsOf, type RegistryIndex,
} from './registry'
import { USER_AGENT_PRODUCT, userAgent } from './userAgent'

export { USER_AGENT_PRODUCT }

@ExposeServiceKey(ZenithMarketServiceKey)
export class ZenithMarketService extends AbstractService implements IZenithMarketService {
  readonly game = ZENITH_MINECRAFT
  private loadIndex: (force?: boolean) => Promise<RegistryIndex>

  constructor(@Inject(LauncherAppKey) app: LauncherApp) {
    super(app)
    this.loadIndex = createIndexLoader({
      fetch: (url, init) => this.app.fetch(url, init as any),
      userAgent: this.userAgent,
      cacheFile: this.getAppDataPath('zenith-market', 'registry-index.json'),
      url: process.env.ZENITH_MOD_INDEX?.startsWith('https://') ? process.env.ZENITH_MOD_INDEX : undefined,
    })
  }

  get userAgent() {
    return userAgent(this.app.version)
  }

  private async json(url: string): Promise<any> {
    const res = await this.app.fetch(url, { headers: { 'user-agent': this.userAgent, accept: 'application/json' } })
    if (!res.ok) throw new Error(`Modrinth answered ${res.status}.`)
    return res.json()
  }

  async search(query: MarketQuery): Promise<MarketSearchResult> {
    const sources = query.sources ?? (['zenith', 'modrinth'] as MarketSource[])
    const result: MarketSearchResult = { projects: [], errors: {} }
    const tasks: Promise<void>[] = []
    if (sources.includes('zenith')) {
      tasks.push(this.loadIndex().then((index) => {
        result.projects.push(...searchIndex(index, query, this.game))
      }).catch((e) => { result.errors.zenith = String(e?.message ?? e) }))
    }
    if (sources.includes('modrinth') && modrinthHasKind(query.kind, query.everything)) {
      tasks.push(this.json(searchUrl(query, this.game)).then((found) => {
        for (const hit of found.hits ?? []) {
          const project = parseHit(hit, query.kind, this.game)
          if (project && project.kind === query.kind && (query.everything || project.support.level !== 'no')) result.projects.push(project)
        }
      }).catch((e) => { result.errors.modrinth = String(e?.message ?? e) }))
    }
    await Promise.all(tasks)
    return result
  }

  async getProject(source: MarketSource, id: string, kind?: MarketKind): Promise<MarketProjectDetail> {
    if (source === 'zenith') {
      const index = await this.loadIndex()
      const p = index.projects.find((p) => p.id === id)
      const project = p && projectOf(p, this.game)
      if (!p || !project) throw new Error(`${id} is not in the Zenith registry.`)
      const notes = [
        project.author ? `Made by ${project.author}.` : '',
        p.key ? 'Signed by its author: the launcher checks the signature.' : 'Unsigned: the launcher checks its checksum only.',
        p.kind === 'native-mod' ? 'Native code: it is compiled into the game and runs with full access to your computer.' : '',
        `Licence: ${p.license}. Source: ${p.source}`,
      ].filter(Boolean)
      return { project, description: p.description, versions: versionsOf(index, p, this.game), notes }
    }
    const p = await this.json(`${MODRINTH_API}/project/${encodeURIComponent(id)}`)
    const wantedKind = kind ?? (p.project_type === 'resourcepack' ? 'resourcepack' : p.project_type === 'shader' ? 'shader' : (p.categories ?? []).includes('datapack') ? 'datapack' : 'mod')
    const project = parseHit({ ...p, project_id: p.id, versions: p.game_versions, categories: [...(p.categories ?? []), ...(p.loaders ?? [])], author: '', license: p.license?.id }, wantedKind, this.game)
    if (!project) throw new Error(`${id} is not a pack Zenith can use.`)
    const list = await this.json(`${MODRINTH_API}/project/${encodeURIComponent(id)}/version`)
    const versions = (Array.isArray(list) ? list : []).map(parseVersion).filter((v) => !!v)
    const picked = pickVersion(project.kind, versions, this.game)
    return {
      project,
      description: String(p.body || p.description || ''),
      versions: versions.map((v) => toMarketVersion(v, v === picked)),
      notes: [`Licence: ${p.license?.id ?? 'unknown'}.`, 'From Modrinth: the launcher checks the file against Modrinth\'s SHA-512.'],
    }
  }

  private async assertInstance(instancePath: string) {
    const instances = await this.app.registry.getOrCreate(InstanceService)
    await instances.initialize()
    if (!instances.state.instances.some((i) => i.path === instancePath) || !existsSync(join(instancePath, 'instance.json'))) {
      throw new Error('That is not one of your Zenith instances.')
    }
  }

  async install(options: MarketInstallOptions): Promise<MarketInstallResult> {
    await this.assertInstance(options.instancePath)
    const fetchBytes = (url: string, init?: any) => this.app.fetch(url, init) as any
    if (options.source === 'zenith') {
      const index = await this.loadIndex()
      const p = index.projects.find((p) => p.id === options.projectId)
      const kind = p && kindOf(p.kind)
      if (!p || !kind) throw new Error(`${options.projectId} is not in the Zenith registry.`)
      if (!canInstallKind(kind)) {
        throw new Error(kind === 'native-mod'
          ? 'Native mods are compiled into the game and need a Zenith rebuild: this launcher cannot install them yet.'
          : 'This launcher cannot install this kind of project yet.')
      }
      const versions = versionsOf(index, p, this.game)
      const chosen = options.versionId ? versions.find((v) => v.id === options.versionId) : versions.find((v) => v.picked)
      if (!chosen) throw new Error(`${p.name} has no version for Zenith ${this.game}.`)
      if (chosen.blocked) throw new Error(`${p.name} ${chosen.number} is ${chosen.blocked}`)
      const v = p.versions.find((v) => v.version === chosen.id)!
      const revoked = revokedReason(index, p.id, v.file.sha256)
      if (revoked) throw new Error(`${p.name} ${v.version} is ${revoked}`)
      const dir = destinationFor(kind, options.instancePath, options.world)
      const { path, bytes } = await downloadChecked(fetchBytes, { url: v.file.url, name: v.file.name, size: v.file.size, sha256: v.file.sha256 }, dir, this.userAgent, MAX_REGISTRY_FILE_BYTES)
      if (p.key) {
        if (!v.file.signature || !verifyMinisign(p.key, v.file.signature, bytes)) {
          const { rm } = await import('fs/promises')
          await rm(path, { force: true })
          throw new Error(`${v.file.name}: the author's signature does not verify.`)
        }
      }
      return { path, fileName: v.file.name, kind, notes: kind === 'mod' ? ['Mods start turned off: enable it in the game, Options > Mods.'] : [] }
    }
    // Modrinth
    const detail = await this.getProject('modrinth', options.projectId)
    const project = detail.project
    if (!project.installable) throw new Error(project.support.reason ?? 'This project does not run in Zenith.')
    const raw = await this.json(`${MODRINTH_API}/project/${encodeURIComponent(options.projectId)}/version`)
    const versions = (Array.isArray(raw) ? raw : []).map(parseVersion).filter((v) => !!v)
    const version = options.versionId ? versions.find((v) => v.id === options.versionId) : pickVersion(project.kind, versions, this.game, options.everything)
    const file = version && primaryFile(version)
    if (!version || !file) throw new Error(`${project.title} has no file for Zenith ${this.game}.`)
    if (project.kind === 'datapack' && !version.loaders.includes('datapack')) throw new Error(`${project.title}: this version is not a data pack.`)
    if (!safeName(file.filename)) throw new Error(`${file.filename} is not a plain file name.`)
    const dir = destinationFor(project.kind, options.instancePath, options.world)
    const { path } = await downloadChecked(fetchBytes, { url: file.url, name: file.filename, size: file.size, sha512: file.sha512, sha1: file.sha1 }, dir, this.userAgent)
    return { path, fileName: file.filename, kind: project.kind, notes: project.support.reason ? [project.support.reason] : [] }
  }
}

export type { MarketProject, MarketVersion }
