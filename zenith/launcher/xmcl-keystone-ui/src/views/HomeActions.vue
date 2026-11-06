<template>
  <div
    v-roving-tabindex
    role="toolbar"
    aria-orientation="horizontal"
    :aria-label="t('baseSetting.title', 2)"
    class="grid xl:gap-4 gap-1 home-actions"
    style="grid-template-columns: repeat(3, minmax(0, 1fr))"
  >
    <v-btn
      data-testid="home-screenshots-action"
      v-shared-tooltip="() => t('screenshots.manage')"
      icon
      variant="text"
      density="comfortable"
      :aria-label="t('screenshots.manage')"
      @click="showScreenshots()"
    >
      <v-icon> photo_camera </v-icon>
    </v-btn>
    <v-btn
      data-testid="home-log-action"
      v-shared-tooltip="() => t('logsCrashes.title')"
      icon
      variant="text"
      density="comfortable"
      :loading="isValidating"
      :aria-label="t('logsCrashes.title')"
      @click="showLogDialog()"
    >
      <v-icon> subtitles </v-icon>
    </v-btn>

    <v-btn
      data-testid="home-folder-action"
      v-shared-tooltip="() => t('instance.showInstance')"
      icon
      variant="text"
      density="comfortable"
      :loading="isValidating"
      :aria-label="t('instance.showInstance')"
      @click="openDirectory(path)"
    >
      <v-icon> folder </v-icon>
    </v-btn>
  </div>
</template>

<script lang="ts" setup>
import { useService } from "@/composables";
import { kInstance } from "@/composables/instance";
import { kInstances } from "@/composables/instances";
import { vRovingTabindex } from "@/directives/rovingTabindex";
import { vSharedTooltip } from "@/directives/sharedTooltip";
import { injection } from "@/util/inject";
import { BaseServiceKey } from "@xmcl/runtime-api";
import { useDialog } from "../composables/dialog";

const { show: showScreenshots } = useDialog('screenshots')
const { path } = injection(kInstance);
const { isValidating } = injection(kInstances);
const { openDirectory } = useService(BaseServiceKey);
const { show: showLogDialog } = useDialog("log");
const { t } = useI18n();
</script>
<style scoped>
.compact {
  background: rgba(0, 0, 0, 0.5);
}
</style>
