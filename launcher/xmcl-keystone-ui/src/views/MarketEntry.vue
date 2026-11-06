<template>
  <div class="market-entry flex flex-col gap-4 p-6 lg:px-10">
    <v-progress-linear class="absolute left-0 top-0 z-20" :active="loading" height="3" indeterminate />

    <div class="flex flex-wrap items-center gap-3">
      <v-btn-toggle v-model="kind" mandatory divided density="comfortable" color="primary" variant="outlined" rounded="lg">
        <v-btn v-for="k in MARKET_KINDS" :key="k.kind" :value="k.kind" :data-testid="`market-kind-${k.kind}`" size="small">
          <v-icon start size="18">{{ k.icon }}</v-icon>
          {{ t(k.key) }}
        </v-btn>
      </v-btn-toggle>
    </div>

    <div class="flex flex-wrap items-center gap-3">
      <v-text-field
        v-model="keyword"
        data-testid="market-search"
        class="max-w-xl"
        variant="solo"
        flat
        hide-details
        rounded="xl"
        density="comfortable"
        :placeholder="t('zenithMarket.searchHint')"
        prepend-inner-icon="search"
        clearable
      />
      <v-chip-group :model-value="sources" multiple selected-class="text-primary">
        <v-chip filter value="zenith" @click.stop="toggleSource('zenith')">{{ t('zenithMarket.source.zenith') }}</v-chip>
        <v-chip filter value="modrinth" @click.stop="toggleSource('modrinth')">Modrinth</v-chip>
      </v-chip-group>
      <v-select
        v-model="sort"
        :items="sortItems"
        item-title="text"
        item-value="value"
        density="compact"
        variant="solo"
        rounded="lg"
        hide-details
        class="max-w-44"
        :aria-label="t('modrinth.sort.title')"
      />
      <v-switch
        v-model="everything"
        color="primary"
        density="compact"
        hide-details
        :label="t('zenithMarket.showEverything')"
      />
    </div>

    <v-alert v-for="(message, source) in errors" :key="source" type="warning" variant="tonal" density="compact">
      {{ t('zenithMarket.sourceError', { source: source === 'zenith' ? t('zenithMarket.source.zenith') : 'Modrinth' }) }}
      <span class="opacity-70"> {{ message }}</span>
    </v-alert>

    <v-alert v-if="kind === 'native-mod'" type="info" variant="tonal" density="compact">
      {{ t('zenithMarket.nativeNote') }}
    </v-alert>

    <div
      v-if="projects.length > 0"
      class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3"
      role="list"
    >
      <v-card
        v-for="p in projects"
        :key="`${p.source}:${p.id}`"
        role="listitem"
        class="market-card flex flex-col gap-2 p-4"
        :data-testid="`market-item-${p.source}-${p.id}`"
        variant="tonal"
      >
        <div class="flex items-start gap-3">
          <v-avatar rounded="lg" size="52" color="surface-variant">
            <v-img v-if="p.icon" :src="p.icon" cover />
            <v-icon v-else>{{ iconOf(p.kind) }}</v-icon>
          </v-avatar>
          <div class="min-w-0 flex-1">
            <router-link
              class="block truncate text-base font-bold"
              :to="{ path: `/market/${p.source}/${encodeURIComponent(p.id)}`, query: { kind: p.kind } }"
            >
              {{ p.title }}
            </router-link>
            <div class="truncate text-xs opacity-70">{{ p.author }}</div>
          </div>
          <v-chip size="x-small" variant="outlined">{{ p.source === 'zenith' ? t('zenithMarket.source.zenith') : 'Modrinth' }}</v-chip>
        </div>
        <p class="line-clamp-3 text-sm opacity-80">{{ p.summary }}</p>
        <div class="flex flex-wrap items-center gap-2">
          <v-chip
            size="small"
            :color="supportBadge(p.support).color"
            variant="flat"
            :title="p.support.reason"
          >
            {{ t(supportBadge(p.support).key) }}
          </v-chip>
          <v-chip v-if="p.requiresRebuild" size="small" color="warning" variant="tonal">
            {{ t('zenithMarket.requiresRebuild') }}
          </v-chip>
          <span class="text-xs opacity-60">{{ t('zenithMarket.downloads', { count: p.downloads }) }}</span>
          <v-spacer />
          <v-menu v-if="p.kind === 'datapack' && p.installable">
            <template #activator="{ props: menu }">
              <v-btn v-bind="menu" size="small" color="primary" variant="flat" :loading="installing === `${p.source}:${p.id}`">
                {{ t('zenithMarket.install') }}
              </v-btn>
            </template>
            <v-list density="compact">
              <v-list-subheader>{{ t('zenithMarket.pickWorld') }}</v-list-subheader>
              <v-list-item v-for="w in worlds" :key="w.name" :title="w.title" @click="installProject(p, w.name)" />
              <v-list-item v-if="worlds.length === 0" :title="t('zenithMarket.noWorlds')" disabled />
            </v-list>
          </v-menu>
          <v-btn
            v-else
            size="small"
            color="primary"
            variant="flat"
            :disabled="!p.installable"
            :loading="installing === `${p.source}:${p.id}`"
            @click="installProject(p)"
          >
            {{ t('zenithMarket.install') }}
          </v-btn>
        </div>
      </v-card>
    </div>
    <div v-else-if="!loading" class="py-16 text-center opacity-70">
      <v-icon size="48">search_off</v-icon>
      <div class="mt-2 text-lg font-bold">{{ t('zenithMarket.empty') }}</div>
      <div class="text-sm">{{ t('zenithMarket.emptyHint') }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { MARKET_KINDS, supportBadge, useZenithMarket, useZenithMarketInstall } from '@/composables/zenithMarket'
import type { MarketKind } from '@xmcl/runtime-api'

const { t } = useI18n()
const { kind, keyword, sources, sort, everything, loading, projects, errors, toggleSource } = useZenithMarket()
const { installing, worlds, installProject } = useZenithMarketInstall(everything)

const sortItems = computed(() => [
  { text: t('modrinth.sort.downloads'), value: 'downloads' },
  { text: t('modrinth.sort.newest'), value: 'newest' },
  { text: t('modrinth.sort.updated'), value: 'updated' },
])

const iconOf = (k: MarketKind) => MARKET_KINDS.find((m) => m.kind === k)?.icon ?? 'extension'
</script>

<style scoped>
.market-card {
  min-height: 190px;
}
</style>
