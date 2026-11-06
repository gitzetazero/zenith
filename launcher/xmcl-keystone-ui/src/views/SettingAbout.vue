<template>
  <section class="about">
    <SettingCard>
      <div class="pa-4">
        <!-- zenith: Zenith Launcher header, author, links, licence and credits -->
        <div class="d-flex align-center flex-wrap gap-4 mb-6">
          <v-img
            :src="logo"
            alt="Zenith Launcher Logo"
            width="64"
            height="64"
            class="mr-4 rounded-lg flex-grow-0 flex-shrink-0"
          ></v-img>
          <div>
            <div class="text-h5 font-weight-bold" data-testid="about-name">Zenith Launcher</div>
            <div class="text-caption opacity-80 mt-1" data-testid="about-version">
              {{ t('zenith.about.version') }} {{ version }}
            </div>
          </div>
        </div>

        <div class="d-flex flex-column gap-2 mb-6 text-body-2">
          <div data-testid="about-author">
            <span class="opacity-70">{{ t('zenith.about.author') }}:</span>
            <a :href="AUTHOR_URL" target="_blank" rel="noopener noreferrer" class="ml-1 font-weight-medium">gitzetazero</a>
          </div>
          <div data-testid="about-repository">
            <span class="opacity-70">{{ t('zenith.about.repository') }}:</span>
            <a :href="REPOSITORY_URL" target="_blank" rel="noopener noreferrer" class="ml-1">github.com/gitzetazero/zenith</a>
          </div>
          <div data-testid="about-license">
            <span class="opacity-70">{{ t('zenith.about.license') }}:</span>
            <span class="ml-1">MIT</span>
          </div>
        </div>

        <div class="text-subtitle-2 font-weight-bold mb-1 opacity-80">{{ t('zenith.about.credits') }}</div>
        <p class="text-caption opacity-80 mb-6" data-testid="zenith-xmcl-credit">
          {{ t('zenith.about.xmcl') }}
          (<a href="https://github.com/Voxelum/x-minecraft-launcher" target="_blank" rel="noopener noreferrer">github.com/Voxelum/x-minecraft-launcher</a>)
          {{ t('zenith.about.horizon') }}
        </p>

        <div class="text-caption opacity-70 mb-6">
          NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.
        </div>

        <!-- Debug Info Box -->
        <div>
          <div class="text-subtitle-2 font-weight-bold mb-2 opacity-80">
            {{ t('setting.about') }}
          </div>
          <pre
            class="debug-info-code pa-4 text-caption font-mono"
          ><code>{{ debugInfo }}</code></pre>
        </div>
      </div>
    </SettingCard>
  </section>
</template>

<script lang="ts" setup>
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SettingCard from '@/components/SettingCard.vue'
import { kEnvironment } from '@/composables/environment'
import { injection } from '@/util/inject'
import logo from '../assets/logo.webp'

const AUTHOR_URL = 'https://github.com/gitzetazero'
const REPOSITORY_URL = 'https://github.com/gitzetazero/zenith'

const env = injection(kEnvironment)

const debugInfo = computed(() => {
  return JSON.stringify({ ...env.value }, null, 2)
})

const { t } = useI18n()
const version = computed(() => env.value?.version ?? '')
</script>

<style scoped>
.debug-info-code {
  background: rgba(0, 0, 0, 0.25);
  border: var(--card-subsection-border);
  border-radius: var(--card-item-radius);
  color: rgba(var(--v-theme-on-surface), 0.85);
  max-height: 180px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
