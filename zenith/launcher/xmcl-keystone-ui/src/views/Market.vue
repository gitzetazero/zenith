<template>
  <div
    data-testid="market-page"
    class="market visible-scroll relative flex flex-col overflow-auto"
  >
    <header class="market-banner">
      <div class="market-banner__stars" aria-hidden="true">
        <svg viewBox="0 0 64 64" width="64" height="64">
          <path d="M32 2 C34 22 42 30 62 32 C42 34 34 42 32 62 C30 42 22 34 2 32 C22 30 30 22 32 2 Z" fill="var(--zenith-accent)" />
        </svg>
      </div>
      <h1 class="market-banner__title">{{ t('zenithMarket.name') }}</h1>
      <p class="market-banner__subtitle">{{ t('zenithMarket.tagline') }}</p>
    </header>
    <router-view v-slot="{ Component }">
      <Transition name="slide-y-transition" mode="out-in" @after-enter="end" @leave="start">
        <component :is="Component" class="flex flex-col" />
      </Transition>
    </router-view>
  </div>
</template>

<script setup lang="ts">
import { useBlockSharedTooltip } from '@/composables/sharedTooltip'

const { t } = useI18n()
const { start, end } = useBlockSharedTooltip()
</script>

<style>
.market {
  --zenith-night: #0E1030;
  --zenith-primary: #5B4BFF;
  --zenith-primary-deep: #231B8F;
  --zenith-accent: #FFB938;
}

.market-banner {
  position: relative;
  flex: none;
  padding: 28px 40px 24px;
  color: #F2F1FF;
  background:
    radial-gradient(circle at 92% 20%, rgba(255, 185, 56, 0.28), transparent 38%),
    linear-gradient(115deg, var(--zenith-night) 0%, var(--zenith-primary-deep) 62%, var(--zenith-primary) 100%);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.market-banner__stars {
  position: absolute;
  right: 36px;
  top: 14px;
  opacity: 0.9;
}

.market-banner__title {
  font-size: 1.9rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  margin: 0;
}

.market-banner__subtitle {
  margin: 4px 0 0;
  max-width: 46rem;
  color: #A9A9D6;
}
</style>
