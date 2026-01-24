<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { NConfigProvider, NIcon, darkTheme, dateEnUS, dateZhCN, enUS, zhCN } from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import { FileTrayStackedOutline, GitNetworkOutline, HomeOutline, SettingsOutline } from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import NaiveProvider from '@/components/NaiveProvider.vue'
import router from '@/router'
import { useAppStore } from '@/stores/app'
import { setLocale, t } from '@/locales'

const appStore = useAppStore()
const activeMenu = ref<string>('menu-home')
const menuOptions: MenuOption[] = [
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
]
function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

function gotoSetting() {
  activeMenu.value = 'menu-setting'
  router.push({ name: 'Setting' })
}
const isDark = computed(() => {
  console.log(appStore.getTheme)
  if (appStore.getTheme === 'dark') {
    document.body.classList.add('dark')
    return darkTheme
  } else {
    document.body.classList.remove('dark')
    return null
  }
})
onMounted(() => {
  if (import.meta.env.MODE === "production") {
    window.addEventListener("contextmenu", (e) => {
      e.preventDefault();
    }, false);
  }
  invoke('ui_mounted').then((resp) => {
    console.log('ui_mounted', resp)
  })
  invoke('load_active_locale').then((activeLocale) => {
    setLocale(activeLocale as 'en-US' | 'zh-CN')
  })
})
</script>

<template>
  <NConfigProvider class="h-full select-text" :locale="appStore.locale === 'en-US' ? enUS : zhCN"
    :date-locale="appStore.locale === 'en-US' ? dateEnUS : dateZhCN" :theme="isDark">
    <NaiveProvider>
      <NLayout class="h-full" has-sider>
        <NLayoutSider bordered :collapsed-width="48" collapse-mode="width" :collapsed="true" class="h-full">
          <div>
            <NMenu v-model:value="activeMenu" :options="menuOptions" />
            <div class="flex flex-col absolute bottom-0 ml-2 mb-2">
              <NTooltip trigger="hover" placement="right" style="margin-left: 1.5rem;">
                <template #trigger>
                  <NButton text style="font-size: 26px;" class="cursor-pointer" @click="gotoSetting">
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
              <component :is="Component" :key="route.fullPath" />
            </KeepAlive>
          </RouterView>
        </NLayout>
      </NLayout>
    </NaiveProvider>
  </NConfigProvider>
</template>
