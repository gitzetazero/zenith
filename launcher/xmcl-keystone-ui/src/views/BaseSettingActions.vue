<template>
  <div
    v-roving-tabindex
    role="toolbar"
    aria-orientation="horizontal"
    :aria-label="t('baseSetting.title', 2)"
    class="grid xl:gap-4 gap-1 home-actions"
    style="grid-template-columns: repeat(2, minmax(0, 1fr))"
  >
    <v-btn
      data-testid="base-setting-log-action"
      v-shared-tooltip.left="() => t('logsCrashes.title')"
      variant="text"
      icon
      :loading="isValidating"
      @click="showLogDialog()"
    >
      <v-icon> subtitles </v-icon>
    </v-btn>
    <v-btn
      data-testid="base-setting-folder-action"
      v-shared-tooltip.left="() => t('instance.showInstance')"
      variant="text"
      icon
      :loading="isValidating"
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

const { path } = injection(kInstance);
const { isValidating } = injection(kInstances);
const { openDirectory } = useService(BaseServiceKey);
const { show: showLogDialog } = useDialog("log");
const { t } = useI18n();
</script>
