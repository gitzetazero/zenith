<template>
  <v-dialog :model-value="modelValue" width="520" :persistent="loading" @update:model-value="emit('update:modelValue', $event)">
    <v-card class="p-4">
      <v-card-title>{{ t('zenith.create.title') }}</v-card-title>
      <v-card-text class="flex flex-col gap-3">
        <div class="text-body-2 opacity-80">{{ t('zenith.create.hint') }}</div>
        <v-text-field
          v-model="name"
          :label="t('instance.name')"
          variant="outlined"
          density="compact"
          hide-details="auto"
          :error-messages="nameError"
        />
        <v-alert v-if="error" type="error" variant="tonal" density="compact">
          {{ String((error as any)?.message ?? error) }}
        </v-alert>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" :disabled="loading" @click="emit('update:modelValue', false)">{{ t('shared.cancel') }}</v-btn>
        <v-btn color="primary" variant="flat" :loading="loading" :disabled="!!nameError || !name.trim()" @click="onCreate">
          {{ t('zenith.create.action') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
// zenith: Big Picture only creates Zenith instances (the runtime refuses anything else).
import { useService } from '@/composables'
import { kInstance } from '@/composables/instance'
import { kInstances } from '@/composables/instances'
import { injection } from '@/util/inject'
import { InstanceServiceKey } from '@xmcl/runtime-api'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', value: boolean): void }>()
const { t } = useI18n()
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
watch(() => props.modelValue, (shown) => {
  if (!shown) return
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
    path.value = await createInstance({ name: name.value.trim() })
    emit('update:modelValue', false)
  } catch (e) {
    error.value = e
  } finally {
    loading.value = false
  }
}
</script>
