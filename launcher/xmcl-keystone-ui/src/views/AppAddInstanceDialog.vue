<template>
  <v-dialog
    v-model="isShown"
    data-testid="add-instance-dialog"
    width="520"
    :persistent="loading"
    transition="fade-transition"
    content-class="elevation-0"
  >
    <div class="surface-dialog-shell flex flex-col overflow-hidden">
      <div class="flex items-center gap-3 px-6 pt-6 pb-4">
        <div
          class="w-10 h-10 rounded-xl flex items-center justify-center flex-shrink-0"
          style="background-color: rgba(var(--v-theme-primary), 0.12)"
        >
          <v-icon size="22" color="primary">add</v-icon>
        </div>
        <div class="text-base font-bold tracking-tight">
          {{ t('zenith.create.title') }}
        </div>
      </div>
      <v-divider class="mx-6 opacity-20" />
      <div class="px-6 py-4 flex flex-col gap-3">
        <div class="text-body-2 opacity-80">{{ t('zenith.create.hint') }}</div>
        <v-text-field
          v-model="name"
          data-testid="add-instance-name"
          :label="t('instance.name')"
          variant="outlined"
          density="compact"
          hide-details="auto"
          :error-messages="nameError"
          autofocus
          @keydown.enter="onCreate"
        />
        <v-alert v-if="error" type="error" variant="tonal" density="compact" rounded="lg">
          {{ String((error as any)?.message ?? error) }}
        </v-alert>
      </div>
      <v-divider class="mx-6 opacity-20" />
      <div class="flex justify-end gap-2 px-6 pb-6 pt-4">
        <v-btn variant="text" rounded="pill" :disabled="loading" @click="hide()">
          {{ t('shared.cancel') }}
        </v-btn>
        <v-btn
          data-testid="add-instance-create"
          color="primary"
          variant="flat"
          rounded="pill"
          :loading="loading"
          :disabled="!!nameError || !name.trim()"
          @click="onCreate"
        >
          {{ t('zenith.create.action') }}
        </v-btn>
      </div>
    </div>
  </v-dialog>
</template>

<script lang="ts" setup>
// zenith: replaces XMCL's multi-step dialog (Minecraft version, mod loaders, Bedrock, modpack import).
// The runtime refuses anything but a Zenith instance (xmcl-runtime/zenith/only.ts), so this dialog only asks for a name.
import { useService } from '@/composables'
import { kInstance } from '@/composables/instance'
import { kInstances } from '@/composables/instances'
import { injection } from '@/util/inject'
import { InstanceServiceKey } from '@xmcl/runtime-api'
import { useDialog } from '../composables/dialog'
import { AddInstanceDialogKey } from '../composables/instanceTemplates'

const { t } = useI18n()
const router = useRouter()
const { instances } = injection(kInstances)
const { path } = injection(kInstance)
const { createInstance } = useService(InstanceServiceKey)

const name = ref('Zenith')
const loading = ref(false)
const error = ref<unknown>(undefined)

const nameError = computed(() => {
  const n = name.value.trim()
  return n && instances.value.some((i) => i.name === n) ? t('zenith.create.exists') : ''
})

const { isShown, hide } = useDialog(AddInstanceDialogKey, () => {
  error.value = undefined
  const taken = new Set(instances.value.map((i) => i.name))
  let candidate = 'Zenith'
  for (let i = 2; taken.has(candidate); i++) candidate = `Zenith ${i}`
  name.value = candidate
})

async function onCreate() {
  if (loading.value || nameError.value || !name.value.trim()) return
  loading.value = true
  error.value = undefined
  try {
    const created = await createInstance({ name: name.value.trim() })
    path.value = created
    if (router.currentRoute.value.path !== '/') router.push('/')
    hide()
  } catch (e) {
    error.value = e
  } finally {
    loading.value = false
  }
}
</script>
