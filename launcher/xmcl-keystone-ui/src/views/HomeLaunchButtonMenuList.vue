<template>
  <v-list
    ref="menuRoot"
    role="menu"
    :aria-label="t('baseSetting.title', 2)"
    min-width="300"
  >
    <v-list-item
      role="menuitem"
      :title="t('baseSetting.title', 2)"
      to="/base-setting"
    >
      <template #prepend>
        <v-icon size="20">
          settings
        </v-icon>
      </template>
    </v-list-item>
    <v-menu
      submenu
      :close-on-content-click="true"
      open-on-hover
      :open-on-focus="false"
      :open-delay="200"
      :close-delay="100"
    >
      <template #activator="{ props: menuProps }">
        <v-list-item
          v-bind="menuProps"
          role="menuitem"
          tabindex="-2"
          :title="t('shared.manage')"
          append-icon="chevron_right"
        >
          <template #prepend>
            <v-icon size="20">
              tune
            </v-icon>
          </template>
        </v-list-item>
      </template>
      <v-list
        role="menu"
        :aria-label="t('shared.manage')"
        min-width="220"
      >
        <v-list-item
          role="menuitem"
          :title="t('logsCrashes.title')"
          @click="showLogs"
        >
          <template #prepend>
            <v-icon size="20">
              subtitles
            </v-icon>
          </template>
        </v-list-item>
        <v-list-item
          role="menuitem"
          :title="t('instance.showInstance')"
          @click="showInstanceFolder"
        >
          <template #prepend>
            <v-icon size="20">
              folder
            </v-icon>
          </template>
        </v-list-item>
        <v-list-item
          v-if="env && env.os !== 'osx'"
          role="menuitem"
          :title="t('launch.createShortcut')"
          @click="onCreateShortcut"
        >
          <template #prepend>
            <v-icon size="20">
              rocket_launch
            </v-icon>
          </template>
        </v-list-item>
      </v-list>
    </v-menu>
    <v-divider role="presentation" class="my-1" />

    <v-menu
      v-if="otherUsers.length > 0"
      sub menu
      :close-on-content-click="true"
      open-on-hover      :open-on-focus="false"      :open-delay="200"
      :close-delay="100"
    >
      <template #activator="{ props: menuProps }">
        <v-list-item
          v-bind="menuProps"
          role="menuitem"
          tabindex="-2"
          :title="t('launch.launchAs')"
          append-icon="chevron_right"
        >
          <template #prepend>
            <v-icon size="20">
              person
            </v-icon>
          </template>
        </v-list-item>
      </template>
      <v-list
        role="menu"
        :aria-label="t('launch.launchAs')"
        min-width="220"
      >
        <v-list-item
          v-for="user in otherUsers"
          :key="user.id"
          role="menuitem"
          :title="isUserRunning(user) ? t('launch.kill') : getGameProfileName(user)"
          :subtitle="user.username"
          @click="onLaunchAs(user)"
        >
          <template #prepend>
            <PlayerAvatar
              class="overflow-hidden rounded-full mr-3"
              :src="user.profiles[user.selectedProfile]?.textures?.SKIN?.url"
              :dimension="28"
            />
          </template>
          <template v-if="isUserRunning(user)" #append>
            <v-icon size="16" color="error">cancel</v-icon>
          </template>
        </v-list-item>
      </v-list>
    </v-menu>
    <v-menu
      v-if="serverList.length > 0 || worldList.length > 0"
      submenu
      :close-on-content-click="true"
      open-on-hover
      :open-on-focus="false"
      :open-delay="200"
      :close-delay="100"
    >
      <template #activator="{ props: menuProps }">
        <v-list-item
          v-bind="menuProps"
          data-testid="launch-to-server"
          role="menuitem"
          tabindex="-2"
          :title="t('launch.launchTo')"
          append-icon="chevron_right"
        >
          <template #prepend>
            <v-icon size="20">
              dns
            </v-icon>
          </template>
        </v-list-item>
      </template>
      <v-list
        role="menu"
        :aria-label="t('launch.launchTo')"
        min-width="220"
      >
        <template v-if="serverList.length > 0">
          <v-list-subheader role="presentation">{{ t('server.serversListTitle') }}</v-list-subheader>
          <v-list-item
            v-for="server in serverList"
            :key="server.ip"
            role="menuitem"
            :title="server.name || server.ip"
            :subtitle="server.ip"
            @click="onLaunchToServer(server)"
          >
            <template #prepend>
              <img
                v-if="server.icon"
                class="overflow-hidden rounded mr-3"
                :src="server.icon"
                width="28"
                height="28"
              >
              <v-icon v-else size="20" class="mr-3">
                dns
              </v-icon>
            </template>
          </v-list-item>
        </template>
        <template v-if="worldList.length > 0">
          <v-list-subheader role="presentation">{{ t('save.name', 2) }}</v-list-subheader>
          <v-list-item
            v-for="world in worldList"
            :key="world.path"
            data-testid="launch-to-world"
            role="menuitem"
            :title="world.title"
            :subtitle="world.subtitle"
            @click="onLaunchToWorld(world)"
          >
            <template #prepend>
              <img
                v-if="world.icon"
                class="overflow-hidden rounded mr-3"
                :src="world.icon"
                width="28"
                height="28"
              >
              <v-icon v-else size="20" class="mr-3">
                map
              </v-icon>
            </template>
          </v-list-item>
        </template>
      </v-list>
    </v-menu>
  </v-list>
</template>
<script lang="ts" setup>
import PlayerAvatar from '@/components/PlayerAvatar.vue'
import { useService } from '@/composables';
import { useDialog } from '@/composables/dialog'
import { kEnvironment } from '@/composables/environment';
import { kInstance } from '@/composables/instance';
import { kInstanceLaunch } from '@/composables/instanceLaunch'
import { kInstanceSave } from '@/composables/instanceSave'
import { kInstanceServerInfo } from '@/composables/instanceServerInfo'
import { kServerStatusCache } from '@/composables/serverStatus'
import { kUserContext } from '@/composables/user';
import { join } from '@/util/basename';
import { getInstanceIcon } from '@/util/favicon';
import { injection } from '@/util/inject'
import { BaseServiceKey, LaunchServiceKey, parseServerAddress, UserProfile } from '@xmcl/runtime-api';

const { t } = useI18n()
defineProps<{}>()

// When the menu opens (this component mounts) Vuetify redirects focus to the
// overlay content wrapper, which sits *above* the roving-tabindex root, so
// arrow keys never reach the directive. Move focus onto the first item so the
// roving group is actually focused and Up/Down navigation works.
const menuRoot = ref<{ $el: HTMLElement }>()
onMounted(async () => {
  await nextTick()
  menuRoot.value?.$el.querySelector<HTMLElement>('[role="menuitem"]')?.focus()
})

const { serverCount, kill, launch, launchAs, killPid, gameProcesses } = injection(kInstanceLaunch)
const { users, userProfile } = injection(kUserContext)
const { servers } = injection(kInstanceServerInfo)
const { saves } = injection(kInstanceSave)
const { path, name, instance } = injection(kInstance)
const serverStatusCache = injection(kServerStatusCache)

const otherUsers = computed(() => users.value?.filter((u) => u.id !== userProfile.value.id) ?? [])

function isUserRunning(user: UserProfile) {
  return gameProcesses.value.some((p) => p.options.user.selectedProfile === user.selectedProfile)
}

function getGameProfileName(user: UserProfile) {
  return user.profiles[user.selectedProfile]?.name || user.username
}

function onLaunchAs(user: UserProfile) {
  if (isUserRunning(user)) {
    const proc = gameProcesses.value.find((p) => p.options.user.selectedProfile === user.selectedProfile)
    if (proc) {
      killPid(proc.pid)
    }
  } else {
    launchAs(user)
  }
}

// Only offer the "Launch to server" submenu for non-pinned instances. A
// pinned instance already auto-joins `instance.server` on a normal launch,
// so listing the same servers there would be redundant.
const serverList = computed(() => {
  if (instance.value?.server) return []
  return servers.value
    .map((s) => {
      const parsed = parseServerAddress(s.ip)
      if (!parsed) return undefined
      // Prefer the live favicon from the latest server-status ping (shared
      // host:port cache) and fall back to the static servers.dat icon.
      const cached = serverStatusCache.value[`${parsed.host}:${parsed.port ?? 25565}`]?.favicon
      const rawIcon = cached || s.icon
      return {
        ip: s.ip,
        name: s.name,
        host: parsed.host,
        port: parsed.port,
        icon: rawIcon
          ? (rawIcon.startsWith('data:') ? rawIcon : `data:image/png;base64,${rawIcon}`)
          : '',
      }
    })
    .filter((s): s is NonNullable<typeof s> => !!s)
})

function onLaunchToServer(server: { host: string; port?: number }) {
  launch('client', { server: { host: server.host, port: server.port } })
}

// Local worlds the user can jump straight into via quick play, most recently
// played first.
const worldList = computed(() => {
  return [...saves.value]
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))
    .map((s) => ({
      path: s.path,
      name: s.name,
      title: s.levelName || s.name,
      subtitle: s.gameVersion || '',
      icon: s.icon?.replace(/\\/g, '\\\\') || '',
    }))
})

function onLaunchToWorld(world: { name: string }) {
  launch('client', { world: world.name })
}


const { createLaunchShortcut } = useService(LaunchServiceKey)

const { getDesktopDirectory, openDirectory } = useService(BaseServiceKey)
const env = injection(kEnvironment)
const onCreateShortcut = async () => {
  const dir = await getDesktopDirectory()
  const { filePath } = await windowController.showSaveDialog({
    defaultPath: join(dir, name.value),
    filters: env.value?.os === 'windows' ? [
      {
        name: 'Shortcut',
        extensions: ['lnk']
      }
    ] : [{
      name: 'Shortcut',
      extensions: ['desktop']
    }],
    properties: ['createDirectory', 'showOverwriteConfirmation']
  })
  if (!filePath) {
    return
  }
  let icon = getInstanceIcon(instance.value, undefined)
  if (icon.endsWith('.webp')) {
    // render webp to png
    const canvas = document.createElement('canvas')
    const ctx = canvas.getContext('2d')
    const img = new Image()
    img.src = icon
    await new Promise((resolve) => {
      img.onload = () => {
        canvas.width = img.width
        canvas.height = img.height
        ctx?.drawImage(img, 0, 0)
        resolve(true)
      }
    })
    const dataUrl = canvas.toDataURL('image/png')
    icon = dataUrl
  }
  await createLaunchShortcut({
    instancePath: path.value,
    destination: filePath,
    userId: userProfile.value.id,
    icon,
  })
}

// Instance actions (settings / logs / folder)
const { show: showLogDialog } = useDialog('log')

function showInstanceFolder() {
  openDirectory(path.value)
}

function showLogs() {
  showLogDialog()
}
</script>
