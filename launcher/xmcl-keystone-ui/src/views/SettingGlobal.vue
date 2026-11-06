<!-- src/components/SettingGlobal.vue -->
<template>
  <div class="flex flex-col gap-4">
    <!-- Quick Launch Settings Card -->
    <SettingCard :title="t('setting.quickLaunchSettings')" icon="flash_on">
      <SettingItemCheckbox v-model="fastLaunch" :title="t('instanceSetting.fastLaunch')"
        :description="t('instanceSetting.fastLaunchHint')" />
      <v-divider class="my-2" />
      <SettingItemCheckbox v-model="hideLauncher" :title="t('instanceSetting.hideLauncher')" />
      <v-divider class="my-2" />
      <SettingItemCheckbox v-model="showLog" :title="t('instanceSetting.showLog')"
        :description="t('instanceSetting.showLogHint')" />
    </SettingCard>

    <SettingCard :title="t('setting.authenticationSettings')" icon="security">
      <SettingItemCheckbox v-model="disableAuthlibInjector" :title="t('instanceSetting.disableAuthlibInjector')"
        :description="t('instanceSetting.disableAuthlibInjectorDescription')" />
      <v-divider class="my-2" />
      <SettingItemCheckbox v-model="disableElyByAuthlib" :title="t('instanceSetting.disableElyByAuthlib')"
        :description="t('instanceSetting.disableElyByAuthlibDescription')" />
    </SettingCard>

    <!-- Minecraft Options Card -->
    <SettingCard :title="t('setting.minecraftOptions')" icon="videogame_asset">
      <SettingItemCheckbox v-model="syncGameLanguage" :title="t('instanceSetting.syncGameLanguage')"
        :description="t('instanceSetting.syncGameLanguageDescription')" />
    </SettingCard>

    <!-- Game Window Resolution Card -->
    <SettingCard :title="t('instance.resolution')" icon="aspect_ratio">
      <div class="grid grid-cols-1 sm:grid-cols-4 gap-4 items-center">
        <v-text-field v-model="resolutionWidth" :label="t('instance.width')" type="number" variant="outlined" density="compact"
          hide-details />
        <v-text-field v-model="resolutionHeight" :label="t('instance.height')" type="number" variant="outlined" density="compact"
          hide-details />
        <v-switch v-model="resolutionFullscreen" data-testid="global-fullscreen-switch" :label="t('instance.fullscreen')" class="mt-0" color="primary" hide-details />
        <v-select v-model="selectedResolutionPreset" :items="resolutionPresets" item-title="text" item-value="value"
          :label="t('instance.resolutionPreset')" outlined filled hide-details dense />
        <v-select v-if="monitorItems.length > 1" v-model="resolutionMonitor" data-testid="global-monitor-select"
          :items="monitorItems" :label="t('instance.monitor')" :disabled="!resolutionFullscreen" variant="outlined"
          density="compact" hide-details class="sm:col-span-4" />
      </div>
    </SettingCard>
  </div>
</template>

<script setup lang="ts">
// zenith: no Java, memory, JVM arguments, pre-execute command or environment variable settings: Zenith is not run through Java.
// Saving writes them back as empty so a value left by an older configuration can never apply.
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useEventListener } from '@vueuse/core'
import { useGlobalSettings } from '@/composables/setting'
import { useResolutionPresets } from '@/composables/resolutionPresets'
import { useMonitorItems } from '@/composables/monitor'
import SettingItemCheckbox from '@/components/SettingItemCheckbox.vue'
import SettingCard from '@/components/SettingCard.vue'

const { t } = useI18n()
const {
  globalFastLaunch,
  globalHideLauncher,
  globalShowLog,
  globalDisableAuthlibInjector,
  globalDisableElyByAuthlib,
  globalSyncGameLanguage,
  globalResolution,
  setGlobalSettings,
} = useGlobalSettings()

const fastLaunch = ref(globalFastLaunch.value)
const hideLauncher = ref(globalHideLauncher.value)
const showLog = ref(globalShowLog.value)
const disableAuthlibInjector = ref(globalDisableAuthlibInjector.value)
const disableElyByAuthlib = ref(globalDisableElyByAuthlib.value)
const syncGameLanguage = ref(globalSyncGameLanguage.value)

// Resolution Settings
const resolutionFullscreen = ref(globalResolution.value?.fullscreen)
const resolutionWidth = ref(globalResolution.value?.width)
const resolutionHeight = ref(globalResolution.value?.height)
const resolutionMonitor = ref(globalResolution.value?.monitor ?? '')
const resolutionPresets = useResolutionPresets()
const monitorItems = useMonitorItems()

const selectedResolutionPreset = computed({
  get: () => {
    const width = resolutionWidth.value
    const height = resolutionHeight.value
    const preset = resolutionPresets.value.find(p => p.value.width === width && p.value.height === height)
    return preset ? preset.value : { width, height }
  },
  set: (value) => {
    resolutionWidth.value = value.width
    resolutionHeight.value = value.height
  }
})

onMounted(() => {
  fastLaunch.value = globalFastLaunch.value
  hideLauncher.value = globalHideLauncher.value
  showLog.value = globalShowLog.value
  disableAuthlibInjector.value = globalDisableAuthlibInjector.value
  disableElyByAuthlib.value = globalDisableElyByAuthlib.value
  syncGameLanguage.value = globalSyncGameLanguage.value
  if (globalResolution.value) {
    resolutionFullscreen.value = globalResolution.value.fullscreen
    resolutionWidth.value = globalResolution.value.width
    resolutionHeight.value = globalResolution.value.height
    resolutionMonitor.value = globalResolution.value.monitor ?? ''
  }
})

function save() {
  setGlobalSettings({
    globalAssignMemory: false,
    globalMaxMemory: 0,
    globalMinMemory: 0,
    globalJava: '',
    globalVmOptions: [],
    globalMcOptions: [],
    globalFastLaunch: fastLaunch.value,
    globalHideLauncher: hideLauncher.value,
    globalShowLog: showLog.value,
    globalDisableAuthlibInjector: disableAuthlibInjector.value,
    globalDisableElyByAuthlib: disableElyByAuthlib.value,
    globalSyncGameLanguage: syncGameLanguage.value,
    globalPrependCommand: '',
    globalPreExecuteCommand: '',
    globalEnv: {},
    globalResolution: {
      width: resolutionWidth.value,
      height: resolutionHeight.value,
      fullscreen: resolutionFullscreen.value,
      monitor: resolutionMonitor.value || undefined,
    },
  })
}

// Save settings when the component is unmounted or the user leaves the page
onUnmounted(save)
useEventListener('beforeunload', save)

</script>

<style scoped>
:deep(.transparent-list) {
  background: transparent !important;
}

.v-card {
  transition: all 0.2s ease;
}

.v-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}
</style>
