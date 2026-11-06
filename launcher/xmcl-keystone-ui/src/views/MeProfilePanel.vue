<template>
  <div class="me-profile-panel workspace-side-panel flex flex-col h-full overflow-hidden select-none">
    <UserAccountSwitcher class="profile-header px-3 pt-3 pb-2 flex-shrink-0" show-inline-delete />

    <!-- Skin & Cape Card -->
    <div class="skin-cape-card surface-panel mx-3 mt-1 mb-2 flex-shrink-0 overflow-hidden">
      <!-- 3D Model with edit controls -->
      <div
        class="flex items-center justify-center py-3 cursor-default"
      >
        <UserSkin
          :user="userProfile"
          :profile="gameProfile"
          :inspect="false"
          :hide-controls="true"
        />
      </div>

      <!-- Skin Library Button -->
      <div v-if="canUploadSkin" class="skin-row border-t px-3 py-2"
        style="border-color: rgba(var(--v-theme-on-surface), 0.06);"
      >
        <v-btn
          variant="tonal"
          size="small"
          block
          color="primary"
          class="rounded-lg font-medium text-xs tracking-normal"
          @click="isSkinLibraryOpen = true"
        >
          <v-icon start size="16">accessibility</v-icon>
          {{ t('userSkin.libraryTitle') }}
        </v-btn>
      </div>

      <!-- Cape row -->
      <div v-if="capes.length > 0" class="cape-row border-t px-3 py-2.5"
        style="border-color: rgba(var(--v-theme-on-surface), 0.06);"
      >
        <div class="text-[10px] font-semibold uppercase tracking-widest opacity-50 mb-1.5">
          {{ t('userCape.changeTitle') }}
        </div>
        <div
          ref="capeScroller"
          v-roving-tabindex
          role="radiogroup"
          :aria-label="t('userCape.changeTitle')"
          class="cape-scroll flex gap-1.5 overflow-x-auto"
          @wheel.prevent="onCapeWheel"
        >
          <!-- No cape -->
          <div
            v-shared-tooltip.top="() => t('userCape.noCape')"
            class="cape-thumb flex-shrink-0 cursor-pointer rounded-lg border transition-all flex items-center justify-center hover:scale-110 hover:-translate-y-0.5"
            :class="!skinModel.cape.value
              ? 'border-primary bg-primary/15 shadow-sm shadow-primary/20'
              : 'border-transparent hover:border-[rgba(var(--v-theme-on-surface),0.15)] hover:bg-[rgba(var(--v-theme-on-surface),0.05)]'"
            role="radio"
            tabindex="0"
            :aria-checked="!skinModel.cape.value"
            :aria-label="t('userCape.noCape')"
            @click="selectCape(undefined)"
            @keydown.enter.prevent="selectCape(undefined)"
            @keydown.space.prevent="selectCape(undefined)"
          >
            <div class="w-full h-full border border-dashed border-current rounded opacity-30 flex items-center justify-center">
              <v-icon size="12" aria-hidden="true">block</v-icon>
            </div>
          </div>
          <!-- Capes -->
          <div
            v-for="c of capes"
            :key="c.id"
            v-shared-tooltip.top="() => c.alias || c.id"
            class="cape-thumb flex-shrink-0 cursor-pointer rounded-lg border transition-all overflow-hidden hover:scale-110 hover:-translate-y-0.5"
            :class="skinModel.cape.value === c.url
              ? 'border-primary bg-primary/15 shadow-sm shadow-primary/20'
              : 'border-transparent hover:border-[rgba(var(--v-theme-on-surface),0.15)] hover:bg-[rgba(var(--v-theme-on-surface),0.05)]'"
            role="radio"
            tabindex="0"
            :aria-checked="skinModel.cape.value === c.url"
            :aria-label="c.alias || c.id"
            @click="selectCape(c.url)"
            @keydown.enter.prevent="selectCape(c.url)"
            @keydown.space.prevent="selectCape(c.url)"
          >
            <div class="cape-scale-wrapper">
              <PlayerCape :src="c.url" />
            </div>
          </div>
        </div>
      </div>
    </div>
    <!-- Master Skin Library Dialog -->
    <UserSkinLibraryDialog
      v-if="canUploadSkin"
      v-model="isSkinLibraryOpen"
      :user="userProfile"
      :profile="gameProfile"
    />
  </div>
</template>

<script lang="ts" setup>
import PlayerCape from '@/components/PlayerCape.vue'
import SkinView from '@/components/SkinView.vue'
import UserAccountSwitcher from '@/components/UserAccountSwitcher.vue'
import UserSkin from '@/components/UserSkin.vue'
import UserSkinLibraryDialog from '@/components/UserSkinLibraryDialog.vue'
import { kUserContext } from '@/composables/user'
import { UserSkinModel, UserSkinRenderPaused, useUserSkin } from '@/composables/userSkin'
import { vRovingTabindex } from '@/directives/rovingTabindex'
import { vSharedTooltip } from '@/directives/sharedTooltip'
import { injection } from '@/util/inject'

const { t } = useI18n()

const { userProfile, gameProfile } = injection(kUserContext)
const paused = inject(UserSkinRenderPaused, ref(false))

const skinModel = useUserSkin(
  computed(() => userProfile.value.id),
  gameProfile,
  computed(() => userProfile.value),
)
provide(UserSkinModel, skinModel)
const { canUploadSkin } = skinModel

const isSkinLibraryOpen = ref(false)

const capes = computed(() => gameProfile.value?.capes ?? [])
const capeScroller = ref<HTMLElement | null>(null)

function selectCape(url: string | undefined) {
  skinModel.cape.value = url
  skinModel.save()
}

function onCapeWheel(e: WheelEvent) {
  if (capeScroller.value) {
    capeScroller.value.scrollLeft += e.deltaY
  }
}

</script>

<style scoped>
.me-profile-panel {
  --workspace-side-panel-width: 280px;
}

.skin-thumb,
.cape-thumb {
  width: 36px;
  height: 52px;
  padding: 3px;
}

.cape-scale-wrapper {
  width: 80px;
  height: 120px;
  transform: scale(0.375);
  transform-origin: top left;
}

.skin-scroll,
.cape-scroll {
  scrollbar-width: none;
}

.skin-scroll::-webkit-scrollbar,
.cape-scroll::-webkit-scrollbar {
  height: 0;
  display: none;
}

.profile-friends-menu-list {
  max-height: min(360px, calc(100vh - 220px));
  min-height: 56px;
  overflow-y: auto;
}

.incoming-requests-pill {
  background: rgba(var(--v-theme-primary), 0.08);
  color: inherit;
  font: inherit;
  appearance: none;
}

.incoming-requests-pill:focus-visible {
  outline: 2px solid rgb(var(--v-theme-primary));
  outline-offset: 2px;
}
</style>

<style>
.profile-friends-menu-card {
  width: 300px;
  background: rgba(var(--v-theme-surface), 0.96);
  backdrop-filter: blur(12px);
}
</style>
