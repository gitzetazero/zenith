<template>
  <div class="market-project flex flex-col gap-4 p-6 lg:px-10" data-testid="market-project">
    <v-progress-linear class="absolute left-0 top-0 z-20" :active="loading" height="3" indeterminate />
    <div>
      <v-btn variant="text" prepend-icon="arrow_back" to="/market">{{ t('zenithMarket.name') }}</v-btn>
    </div>

    <v-alert v-if="error" type="error" variant="tonal">{{ error }}</v-alert>

    <template v-if="detail">
      <div class="flex flex-wrap items-start gap-4">
        <v-avatar rounded="lg" size="72" color="surface-variant">
          <v-img v-if="detail.project.icon" :src="detail.project.icon" cover />
          <v-icon v-else size="36">extension</v-icon>
        </v-avatar>
        <div class="min-w-0 flex-1">
          <h2 class="text-2xl font-bold">{{ detail.project.title }}</h2>
          <div class="text-sm opacity-70">
            {{ detail.project.author }} · {{ detail.project.source === 'zenith' ? t('zenithMarket.source.zenith') : 'Modrinth' }}
            · {{ t('zenithMarket.downloads', { count: detail.project.downloads }) }}
          </div>
          <p class="mt-2">{{ detail.project.summary }}</p>
          <div class="mt-2 flex flex-wrap items-center gap-2">
            <v-chip size="small" :color="badge.color" variant="flat">{{ t(badge.key) }}</v-chip>
            <v-chip v-if="detail.project.requiresRebuild" size="small" color="warning" variant="tonal">
              {{ t('zenithMarket.requiresRebuild') }}
            </v-chip>
            <span v-if="detail.project.support.reason" class="text-xs opacity-70">{{ detail.project.support.reason }}</span>
          </div>
        </div>
        <div class="flex flex-col gap-2">
          <v-menu v-if="detail.project.kind === 'datapack' && detail.project.installable">
            <template #activator="{ props: menu }">
              <v-btn v-bind="menu" color="primary" variant="flat" :loading="!!installing">{{ t('zenithMarket.install') }}</v-btn>
            </template>
            <v-list density="compact">
              <v-list-subheader>{{ t('zenithMarket.pickWorld') }}</v-list-subheader>
              <v-list-item v-for="w in worlds" :key="w.name" :title="w.title" @click="installProject(detail.project, w.name)" />
              <v-list-item v-if="worlds.length === 0" :title="t('zenithMarket.noWorlds')" disabled />
            </v-list>
          </v-menu>
          <v-btn
            v-else
            color="primary"
            variant="flat"
            :disabled="!detail.project.installable"
            :loading="!!installing"
            @click="installProject(detail.project)"
          >
            {{ t('zenithMarket.install') }}
          </v-btn>
          <v-btn variant="text" size="small" append-icon="open_in_new" :href="detail.project.page" target="_blank" rel="noopener noreferrer">
            {{ t('zenithMarket.openPage') }}
          </v-btn>
        </div>
      </div>

      <v-card variant="tonal" class="p-4">
        <ul class="list-disc pl-5 text-sm">
          <li v-for="n in detail.notes" :key="n">{{ n }}</li>
        </ul>
      </v-card>

      <v-card variant="tonal" class="market-description p-4">
        <div class="markdown-body" v-html="descriptionHtml" />
      </v-card>

      <section v-if="detail.versions.length">
        <h3 class="mb-2 text-lg font-bold">{{ t('zenithMarket.versions') }}</h3>
        <v-table density="compact">
          <tbody>
            <tr v-for="v in detail.versions" :key="v.id">
              <td class="font-bold">{{ v.number }} <v-chip v-if="v.picked" size="x-small" color="primary" class="ml-1">{{ t('zenithMarket.recommended') }}</v-chip></td>
              <td>{{ v.gameVersions.slice(-3).join(', ') }}</td>
              <td>{{ v.published.slice(0, 10) }}</td>
              <td>{{ v.fileName }}</td>
              <td class="text-error">{{ v.blocked }}</td>
            </tr>
          </tbody>
        </v-table>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { useService } from '@/composables'
import { supportBadge, useZenithMarketInstall } from '@/composables/zenithMarket'
import { ZenithMarketServiceKey, type MarketKind, type MarketProjectDetail, type MarketSource } from '@xmcl/runtime-api'
import MarkdownIt from 'markdown-it'

const props = defineProps<{ source: string; id: string }>()
const { t } = useI18n()
const route = useRoute()
const { getProject } = useService(ZenithMarketServiceKey)
const { installProject, installing, worlds } = useZenithMarketInstall()

// Descriptions come from third parties: raw HTML is escaped, never rendered.
const md = new MarkdownIt({ html: false, linkify: true, typographer: true })
const defaultLink = md.renderer.rules.link_open || ((tokens, idx, options, _env, self) => self.renderToken(tokens, idx, options))
md.renderer.rules.link_open = (tokens, idx, options, env, self) => {
  tokens[idx].attrSet('target', '_blank')
  tokens[idx].attrSet('rel', 'noopener noreferrer')
  return defaultLink(tokens, idx, options, env, self)
}

const detail = ref<MarketProjectDetail>()
const loading = ref(false)
const error = ref('')
const badge = computed(() => supportBadge(detail.value!.project.support))
const descriptionHtml = computed(() => md.render(detail.value?.description ?? ''))

watch(() => [props.source, props.id], async () => {
  loading.value = true
  error.value = ''
  try {
    detail.value = await getProject(props.source as MarketSource, props.id, (route.query.kind as MarketKind | undefined))
  } catch (e: any) {
    detail.value = undefined
    error.value = String(e?.message ?? e)
  } finally {
    loading.value = false
  }
}, { immediate: true })
</script>
