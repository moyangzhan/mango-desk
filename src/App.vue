<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { NBadge, NConfigProvider, NIcon, darkTheme, dateEnUS, dateZhCN, enUS, zhCN } from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import { FileTrayStackedOutline, GitNetworkOutline, HomeOutline, SettingsOutline } from '@vicons/ionicons5'
import { DeviceHubOutlined } from '@vicons/material'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import NaiveProvider from '@/components/NaiveProvider.vue'
import router from '@/router'
import { useAppStore } from '@/stores/app'
import { setLocale, t } from '@/locales'

const appStore = useAppStore()
const activeMenu = ref<string>('menu-home')

// Pending pairing request count for badge
const pendingPairingCount = ref(0)
const clusterEnabled = ref(false)

const menuOptions = computed<MenuOption[]>(() => [
  {
    key: 'menu-home',
    icon: renderIcon(HomeOutline),
    label: () =>
      h(
        RouterLink,
        {
          to: {
            name: 'Home',
          },
        },
        { default: () => t('menu.home') },
      ),
  },
  {
    key: 'menu-index',
    icon: renderIcon(GitNetworkOutline),
    label: () =>
      h(
        RouterLink,
        {
          to: {
            name: 'Indexer',
          },
        },
        { default: () => t('menu.indexer') },
      ),
  },
  {
    key: 'menu-data',
    icon: renderIcon(FileTrayStackedOutline),
    label: () =>
      h(
        RouterLink,
        {
          to: {
            name: 'Data',
          },
        },
        { default: () => t('menu.data') },
      ),
  },
  {
    key: 'menu-device',
    icon: () => (clusterEnabled.value && pendingPairingCount.value > 0)
      ? (
          h(NBadge, { value: pendingPairingCount.value, max: 99, type: 'info' }, {
            default: () => h(NIcon, null, { default: () => h(DeviceHubOutlined) }),
          })
        )
      : h(NIcon, null, { default: () => h(DeviceHubOutlined) }),
    label: () =>
      h(
        RouterLink,
        {
          to: {
            name: 'Device',
          },
        },
        { default: () => t('menu.deviceManagement') },
      ),
  },
])
function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

function gotoSetting() {
  activeMenu.value = 'menu-setting'
  router.push({ name: 'Setting' })
}
const isDark = computed(() => appStore.getTheme === 'dark' ? darkTheme : null)

watch(() => appStore.getTheme, (theme) => {
  if (theme === 'dark')
    document.body.classList.add('dark')
  else
    document.body.classList.remove('dark')
}, { immediate: true })

// Load pending pairing request count
async function loadPendingPairingCount() {
  try {
    const count = await invoke<number>('count_pending_pairing_requests')
    pendingPairingCount.value = count
  } catch (e) {
    console.error('Failed to load pairing requests count:', e)
  }
}

onMounted(() => {
  // Register event listeners first to avoid missing events
  // 优先注册事件监听器，避免错过事件
  listen<{ enabled: boolean }>('cluster-enabled-changed', (event) => {
    clusterEnabled.value = event.payload.enabled
    if (event.payload.enabled)
      loadPendingPairingCount()
    else
      pendingPairingCount.value = 0
  })

  listen('pairing-request-received', () => {
    if (clusterEnabled.value)
      loadPendingPairingCount()
  })

  listen('pairing-response-received', () => {
    if (clusterEnabled.value)
      loadPendingPairingCount()
  })

  listen<{ port: number }>('cluster-port-error', (event) => {
    console.log('Received cluster-port-error event:', event.payload)
    appStore.setClusterPortError(event.payload)
  })

  listen('cluster-setting-saved', () => {
    appStore.clearClusterPortError()
  })

  // Then invoke commands
  if (import.meta.env.MODE === 'production') {
    window.addEventListener('contextmenu', (e) => {
      e.preventDefault()
    }, false)
  }
  invoke('ui_mounted').then((resp) => {
    console.log('ui_mounted', resp)
  })
  invoke('load_active_locale').then((activeLocale) => {
    setLocale(activeLocale as 'en-US' | 'zh-CN')
  })

  // Load cluster setting
  invoke<{ enabled: boolean }>('load_cluster_setting').then((setting) => {
    clusterEnabled.value = setting?.enabled ?? false
    // Only load pairing count if cluster is enabled
    if (clusterEnabled.value)
      loadPendingPairingCount()
  })
})
</script>

<template>
  <NConfigProvider
    class="h-full select-text" :locale="appStore.locale === 'en-US' ? enUS : zhCN"
    :date-locale="appStore.locale === 'en-US' ? dateEnUS : dateZhCN" :theme="isDark"
  >
    <NaiveProvider>
      <NLayout class="h-full" has-sider>
        <NLayoutSider bordered :collapsed-width="48" collapse-mode="width" :collapsed="true" class="h-full">
          <div>
            <NMenu v-model:value="activeMenu" :options="menuOptions" />
            <div class="flex flex-col absolute bottom-0 ml-2 mb-2">
              <NTooltip trigger="hover" placement="right" style="margin-left: 1.5rem;">
                <template #trigger>
                  <NButton text type="primary" style="font-size: 26px;" class="text-link" @click="gotoSetting">
                    <NIcon>
                      <SettingsOutline />
                    </NIcon>
                  </NButton>
                </template>
                {{ t('menu.setting') }}
              </NTooltip>
            </div>
          </div>
        </NLayoutSider>
        <NLayout>
          <RouterView v-slot="{ Component, route }">
            <KeepAlive>
              <component :is="Component" />
            </KeepAlive>
          </RouterView>
        </NLayout>
      </NLayout>
    </NaiveProvider>
  </NConfigProvider>
</template>
