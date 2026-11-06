<template>
  <v-dialog
    v-model="isShown"
    :width="380"
  >
    <v-card class="h-full flex select-none flex-col launch-status-card">
      <v-card-item v-if="exiting">
        <v-card-title>
          {{ t('launchStatus.exit') }}
        </v-card-title>
      </v-card-item>
      <AppLoadingCircular
        v-if="launching || !windowReady"
        class="mb-4"
        :texts="launchingSteps"
        :secondary-hint="launchingStatus !== ''"
        :hint="hint"
      >
        <template v-if="true">
          <div class="mx-10 my-3 flex flex-col items-center justify-center gap-1">
            <VTypical
              class="blink"
              :steps="launchingSteps"
            />
            <div
              v-if="hint"
              class="transition-all"
              :class="{ 'text-gray-500': launchingStatus !== '', 'text-transparent': !launchingStatus }"
            >
              {{ hint + '...' }}
            </div>
          </div>
        </template>
      </AppLoadingCircular>

      <v-alert
        v-if="showElyByWarning"
        type="warning"
        dense
        outlined
        class="mx-3 mb-3"
      >
        <div class="text-sm">
          {{ t('launchStatus.elyByWarning') }}
        </div>
      </v-alert>

      <div
        class="flex p-3 gap-3"
      >
        <v-btn
          v-if="exiting"
          @click="onCancel"
         variant="text">
          {{ t('shared.cancel') }}
        </v-btn>
        <div class="flex-grow" />
        <v-btn
          v-if="exiting"
          color="red"
          @click="onKill"
         variant="text">
          <v-icon start>
            exit_to_app
          </v-icon>
          {{ t('shared.yes') }}
        </v-btn>
        <v-btn
          v-if="exiting"
          color="red darken-2"
          @click="onForceKill"
         variant="text">
          <v-icon start>
            dangerous
          </v-icon>
          {{ t('launch.forceKill') }}
        </v-btn>
        <v-btn
          v-if="refreshUserTimeout"
          color="primary"
          @click="skipRefresh()"
         variant="text">
          <v-icon>
            skip_next
          </v-icon>
          {{ t('shared.skipForNow') }}
        </v-btn>
        <v-btn
          v-if="authLibTimeout"
          color="primary"
          @click="skipAuthLib()"
         variant="text">
          {{ t('shared.skipForNow') }}
        </v-btn>
      </div>
    </v-card>
  </v-dialog>
</template>

<script lang=ts setup>
import AppLoadingCircular from '@/components/AppLoadingCircular.vue'
import VTypical from '@/components/VTyping.vue'
import { kInstance } from '@/composables/instance'
import { kInstanceLaunch } from '@/composables/instanceLaunch'
import { injection } from '@/util/inject'
import { useDialog } from '../composables/dialog'
import { LaunchStatusDialogKey } from '../composables/launch'
import { kUserContext } from '@/composables/user'
import { kSettingsState } from '@/composables/setting'

const { t } = useI18n()
const { launching, windowReady, kill, launchingStatus, skipRefresh, skipAuthLib } = injection(kInstanceLaunch)
const { userProfile } = injection(kUserContext)
const { state: settings } = injection(kSettingsState)
const { instance } = injection(kInstance)

const exiting = ref(false)

const isElyBy = computed(() => userProfile.value.authority.indexOf('authserver.ely.by') !== -1)
const disableElyByAuthlib = computed(() => instance.value.disableElybyAuthlib ?? settings.value?.globalDisableElyByAuthlib ?? false)
const showElyByWarning = computed(() => isElyBy.value && !disableElyByAuthlib.value && launching.value)

const { isShown, show, hide } = useDialog(LaunchStatusDialogKey, (param) => {
  exiting.value = !!param?.isKill
}, () => {
  exiting.value = false
  refreshUserTimeout.value = false
  authLibTimeout.value = false
})

const refreshUserTimeout = ref(false)
const authLibTimeout = ref(false)
watch(launchingStatus, (newStat) => {
  if (newStat === 'refreshing-user') {
    setTimeout(() => {
      if (launchingStatus.value === 'refreshing-user') {
        refreshUserTimeout.value = true
      }
    }, 5000)
  } else {
    refreshUserTimeout.value = false
  }
  if (newStat === 'preparing-authlib') {
    setTimeout(() => {
      if (launchingStatus.value === 'preparing-authlib') {
        authLibTimeout.value = true
      }
    }, 5000)
  } else {
    authLibTimeout.value = false
  }
})

const hint = computed(() => launchingStatus.value === 'preparing-authlib'
  ? t('launchStatus.injectingAuthLib')
  : launchingStatus.value === 'assigning-memory'
    ? t('launchStatus.assigningMemory')
    : launchingStatus.value === 'refreshing-user'
      ? t('launchStatus.refreshingUser')
      : launchingStatus.value === 'spawning-process'
        ? t('launchStatus.spawningProcess')
        : '')

const launchingSteps = computed(() => [
  t('launchStatus.launching'),
  4000,
  t('launchStatus.launchingSlow'),
])

const onKill = () => {
  kill()
  hide()
}
const onForceKill = () => {
  // gh #1395 — force-terminate the JVM process tree when graceful kill fails.
  kill('client', true)
  hide()
}
const onCancel = () => hide()

watch(windowReady, (ready) => {
  if (ready && isShown.value) {
    hide()
  }
})
</script>

<style scoped="true">
.blink::after {
  content: '|';
  animation: blink 1s infinite step-start;
}

@keyframes blink {
  50% {
    opacity: 0;
  }
}
</style>
