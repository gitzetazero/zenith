// zenith: state of the Zenith Market page (kind tabs, sources, search, install). All the work happens in ZenithMarketService.
import { useService } from '@/composables'
import { kInstance } from '@/composables/instance'
import { kInstanceSave } from '@/composables/instanceSave'
import { useNotifier } from '@/composables/notifier'
import { injection } from '@/util/inject'
import {
  ZenithMarketServiceKey,
  type MarketInstallResult,
  type MarketKind,
  type MarketProject,
  type MarketSource,
  type MarketSupport,
} from '@xmcl/runtime-api'
import { useDebounce } from '@vueuse/core'
import type { Ref } from 'vue'

export const MARKET_KINDS: { kind: MarketKind; icon: string; key: string }[] = [
  { kind: 'mod', icon: 'extension', key: 'zenithMarket.kind.mod' },
  { kind: 'native-mod', icon: 'memory', key: 'zenithMarket.kind.nativeMod' },
  { kind: 'shader', icon: 'wb_twilight', key: 'zenithMarket.kind.shader' },
  { kind: 'resourcepack', icon: 'texture', key: 'zenithMarket.kind.resourcepack' },
  { kind: 'datapack', icon: 'dataset', key: 'zenithMarket.kind.datapack' },
  { kind: 'modpack', icon: 'folder_zip', key: 'zenithMarket.kind.modpack' },
]

/** The badge colour and i18n key of a support level (Works / Partly / Doesn't run). */
export function supportBadge(support: MarketSupport) {
  switch (support.level) {
    case 'works': return { color: 'success', key: 'zenithMarket.support.works' }
    case 'partial': return { color: 'warning', key: 'zenithMarket.support.partial' }
    default: return { color: 'error', key: 'zenithMarket.support.no' }
  }
}

/** Installing into the selected Zenith instance (used by the list and the project page). */
export function useZenithMarketInstall(everything: Ref<boolean> = ref(false)) {
  const { t } = useI18n()
  const { notify } = useNotifier()
  const { install } = useService(ZenithMarketServiceKey)
  const { path } = injection(kInstance)
  const { saves } = injection(kInstanceSave)
  const installing = ref('')
  const worlds = computed(() => saves.value.map((s) => ({ name: s.name, title: s.levelName || s.name })))

  async function installProject(project: MarketProject, world?: string, versionId?: string): Promise<MarketInstallResult | undefined> {
    if (!project.installable || installing.value) return undefined
    installing.value = `${project.source}:${project.id}`
    try {
      const result = await install({
        source: project.source,
        projectId: project.id,
        versionId,
        instancePath: path.value,
        world,
        everything: everything.value,
      })
      notify({ level: 'success', title: t('zenithMarket.installed', { name: project.title }), body: result.notes.join(' ') || result.fileName })
      return result
    } catch (e: any) {
      notify({ level: 'error', title: t('zenithMarket.installFailed', { name: project.title }), body: String(e?.message ?? e) })
      return undefined
    } finally {
      installing.value = ''
    }
  }

  return { installing, worlds, installProject }
}

export function useZenithMarket() {
  const { search } = useService(ZenithMarketServiceKey)

  const kind = ref<MarketKind>('mod')
  const keyword = ref('')
  const debounced = useDebounce(keyword, 300)
  const sources = ref<MarketSource[]>(['zenith', 'modrinth'])
  const sort = ref<'downloads' | 'newest' | 'updated'>('downloads')
  const everything = ref(false)
  const loading = ref(false)
  const projects = ref<MarketProject[]>([])
  const errors = ref<Partial<Record<MarketSource, string>>>({})
  let token = 0
  async function refresh() {
    const mine = ++token
    loading.value = true
    try {
      const result = await search({
        text: debounced.value.trim(),
        kind: kind.value,
        sources: sources.value,
        everything: everything.value,
        sort: sort.value,
        limit: 40,
      })
      if (mine !== token) return
      projects.value = result.projects
      errors.value = result.errors
    } catch (e: any) {
      if (mine !== token) return
      projects.value = []
      errors.value = { zenith: String(e?.message ?? e) }
    } finally {
      if (mine === token) loading.value = false
    }
  }

  watch([kind, debounced, sources, sort, everything], refresh, { immediate: true, deep: true })

  function toggleSource(source: MarketSource) {
    const has = sources.value.includes(source)
    if (has && sources.value.length === 1) return
    sources.value = has ? sources.value.filter((s) => s !== source) : [...sources.value, source]
  }

  return { kind, keyword, sources, sort, everything, loading, projects, errors, toggleSource, refresh }
}
