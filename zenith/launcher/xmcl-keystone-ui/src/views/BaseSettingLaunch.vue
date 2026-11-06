<template>
  <SettingCard :title="t('setting.minecraftOptions')" icon="videogame_asset">
    <SettingItemCheckbox
      :model-value="!!isGameOptionsLinkedCache"
      :disabled="isGameOptionsLinkedCache === undefined"
      :title="t('instance.useSharedOptions')"
      :description="t('instance.useSharedOptionsDesc')"
      @update:model-value="
        !isGameOptionsLinkedCache
          ? show('options.txt')
          : unlinkGameOptions(path).then(() => mutateOptions())
      "
    />

    <v-divider class="my-3" />

    <SettingItemCheckbox
      v-model="syncGameLanguage"
      :title="t('instanceSetting.syncGameLanguage')"
      :description="t('instanceSetting.syncGameLanguageDescription')"
    >
      <BaseSettingGlobalLabel
        :global="isGlobalSyncGameLanguage"
        @clear="resetSyncGameLanguage"
      />
    </SettingItemCheckbox>

    <v-divider class="my-4" />

    <BaseSettingResolution />
  </SettingCard>

  <v-dialog v-model="model" width="440">
    <v-card>
      <v-card-item>
        <template #prepend>
          <v-avatar color="primary" variant="tonal">
            <v-icon>link</v-icon>
          </v-avatar>
        </template>
        <v-card-title class="text-base font-medium">
          {{ t('instance.linkFileTitle', { file: target }) }}
        </v-card-title>
      </v-card-item>
      <v-card-text class="pt-0 text-sm opacity-80">
        {{ t('instance.linkFileDesc', { file: target }) }}
      </v-card-text>
      <v-card-actions class="px-4 pb-4">
        <v-btn variant="text" @click="cancel">
          {{ t('shared.cancel') }}
        </v-btn>
        <v-spacer />
        <v-btn
          color="primary"
          variant="flat"
          rounded="pill"
          prepend-icon="link"
          @click="confirm"
        >
          {{ t('shared.yes') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
// zenith: the launch page keeps only what Zenith supports (shared options.txt, game language, resolution / fullscreen).
// JVM arguments, Java, pre-execute commands and the launch-arguments preview are gone: Zenith is not run through Java.
import SettingCard from '@/components/SettingCard.vue'
import SettingItemCheckbox from '@/components/SettingItemCheckbox.vue'
import { useService } from '@/composables'
import { useSimpleDialog } from '@/composables/dialog'
import { kInstance } from '@/composables/instance'
import { injection } from '@/util/inject'
import { InstanceOptionsServiceKey } from '@xmcl/runtime-api'
import useSWRV from 'swrv'
import { InstanceEditInjectionKey } from '../composables/instanceEdit'
import BaseSettingGlobalLabel from '@/components/BaseSettingGlobalLabel.vue'
import BaseSettingResolution from './BaseSettingResolution.vue'

const { t } = useI18n()
const {
  syncGameLanguage,
  isGlobalSyncGameLanguage,
  resetSyncGameLanguage,
} = injection(InstanceEditInjectionKey)

const { path } = injection(kInstance)
const { isGameOptionsLinked, linkGameOptions, unlinkGameOptions } =
  useService(InstanceOptionsServiceKey)

const { data: isGameOptionsLinkedCache, mutate: mutateOptions } = useSWRV(
  computed(() => `${path.value}/options.txt`),
  (key) => isGameOptionsLinked(key.substring(0, key.lastIndexOf('/'))),
)

const { model, show, target, confirm, cancel } = useSimpleDialog<'options.txt'>(
  (type) => {
    if (type === 'options.txt') {
      linkGameOptions(path.value).then(() => mutateOptions())
    }
  },
)
</script>
