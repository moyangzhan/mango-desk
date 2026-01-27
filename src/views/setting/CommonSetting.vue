<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/app'
import { setLocale, t } from '@/locales'
import { emptyProxyInfo } from '@/utils/functions'
import { open } from '@tauri-apps/plugin-dialog'
import { relaunch } from '@tauri-apps/plugin-process';

const dataPath = ref('')
const appStore = useAppStore()
const activePlatform = ref('openai')
const activeTheme = ref(appStore.getTheme)
const activeLanguage = ref('en-US')
const activeTab = ref('openai')
const proxy = ref<ProxyInfo>(emptyProxyInfo())
const needRestart = ref(false)
const dataCopying = ref(false)

function handleLanguageChanged(newLang: string) {
  console.log('languageChange', newLang)
  activeLanguage.value = newLang
  appStore.setLocale(newLang)
  setLocale(newLang as 'en-US' | 'zh-CN')
  invoke('set_active_locale', { locale: newLang })
}

function handleThemeChanged(selectedTheme: string) {
  console.log('themeChanged', selectedTheme)
  activeTheme.value = selectedTheme
  appStore.changeTheme()
}

function handleSaveProxy() {
  console.log('onSaveProxy', proxy.value)
  invoke('update_proxy_info', { proxyInfo: proxy.value }).then((res) => {
    console.log('update_proxy_info result', res)
  }).catch((err) => {
    console.error('update_proxy_info error', err)
  })
}

async function openDirDialog() {
  // Replace browser's native input with Tauri dialog
  // This prevents the default file upload confirmation dialog such as ("Do you want to upload [number] files to this site?")
  const paths = await open({ directory: true, multiple: false })
  if (Array.isArray(paths)) {
    console.log('openDirDialog', paths)
    return;
  }
  let res = await invoke<string>('set_data_path', { path: paths, force: false })
  if (res.indexOf('exist') === 0) {
    let existPath = res.split(':')[1]
    window.$dialog.warning({
      title: t('common.warning'),
      content: t('common.existFile') + ': ' + existPath + t('common.forceChange'),
      positiveText: t('common.confirm'),
      onPositiveClick: async () => {
        dataCopying.value = true
        try {
          res = await invoke<string>('set_data_path', { path: paths, force: true })
          if (res.indexOf('success') === 0) {
            const userDataPath = await invoke<string>('get_data_path')
            dataPath.value = userDataPath
            window.$message.success(t('common.restartAppForChange'))
            needRestart.value = true
          } else {
            window.$message.error(res)
          }
        } catch (err) {
          console.error('set_data_path error', err)
        } finally {
          dataCopying.value = false
        }
      },
    })
    return;
  } else if (res.indexOf('same') === 0) {
    window.$message.error(t('common.changeSamePathError'))
    return;
  }

  const userDataPath = await invoke<string>('get_data_path')
  dataPath.value = userDataPath
  window.$message.success(t('common.restartAppForChange'))
  needRestart.value = true
}

async function resetDataPath() {
  let res = await invoke<string>('reset_data_path', { force: false })
  if (res.indexOf('exist') === 0) {
    let existPath = res.split(':')[1]
    window.$dialog.warning({
      title: t('common.warning'),
      content: t('common.existFile') + ': ' + existPath + t('common.forceChange'),
      positiveText: t('common.confirm'),
      onPositiveClick: async () => {
        dataCopying.value = true
        try {
          res = await invoke<string>('reset_data_path', { force: true })
          if (res.indexOf('success') === 0) {
            const userDataPath = await invoke<string>('get_data_path')
            dataPath.value = userDataPath
            window.$message.success(t('common.restartAppForChange'))
            needRestart.value = true
          } else {
            window.$message.error(res)
          }
        } catch (err) {
          console.log(err)
        } finally {
          dataCopying.value = false
        }
      }
    })
    return;
  } else if (res.indexOf('same') === 0) {
    window.$message.error(t('common.changeSamePathError'))
    return;
  }
  const userDataPath = await invoke<string>('get_data_path')
  dataPath.value = userDataPath
  window.$message.success(t('common.restartAppForChange'))
  needRestart.value = true
}

async function restart() {
  try {
    await relaunch();
  } catch (e) {
    console.error('relaunch failed', e);
  }
}

onMounted(async () => {
  console.log('common setting onMounted')
  const activePlatformName = await invoke('load_active_platform')
  activePlatform.value = activePlatformName as string
  activeTab.value = activePlatform.value
  const activeLocale = await invoke('load_active_locale')
  console.log('active_locale', activeLocale)
  setLocale(activeLocale as 'en-US' | 'zh-CN')
  activeLanguage.value = activeLocale as string
  const proxyInfo = await invoke('load_proxy_info')
  proxy.value = proxyInfo as ProxyInfo
  const userDataPath = await invoke<string>('get_data_path')
  dataPath.value = userDataPath
})
</script>

<template>
  <div>
    <NCard :title="t('common.display')" class="mb-4" :bordered="true">
      <div class="flex flex-col">
        <NFormItem :label="t('common.language')">
          <div class="flex flex-col space-y-1">
            <NTag type="warning">
              {{ t('common.languageSettingWarning') }}
            </NTag>
            <NRadioGroup :value="activeLanguage" @update:value="handleLanguageChanged">
              <NRadio :label="t('common.english')" value="en-US" class="mr-2" />
              <NRadio :label="t('common.chinese')" value="zh-CN" />
            </NRadioGroup>
          </div>
        </NFormItem>

        <NFormItem :label="t('common.theme')">
          <NRadioGroup v-model:value="activeTheme" @update:value="handleThemeChanged">
            <NRadio :label="`☀️${t('common.light')}`" value="light" class="mr-2" />
            <NRadio :label="`🌙${t('common.dark')}`" value="dark" />
          </NRadioGroup>
        </NFormItem>
      </div>
    </NCard>
    <NCard :title="t('common.storage')" class="mb-4" size="small" :bordered="true">
      <div class="flex flex-col space-y-2">
        <div>{{ t('indexer.dataPath') }}: {{ dataPath }}</div>
        <NAlert v-if="needRestart" type="warning">{{ t('common.restartAppForChange') }}
          <NButton type="primary" text @click="restart">{{
            t('common.clickToRestart') }}</NButton>
        </NAlert>
        <div class="flex space-x-2">
          <div class="mr-2">
            <NButton @click="openDirDialog" :disabled="dataCopying" :loading="dataCopying">
              <span>{{ t('common.change') }}</span>
            </NButton>
          </div>
          <NButton @click="resetDataPath" :disabled="dataCopying" :loading="dataCopying">
            {{ t('common.reset') }}
          </NButton>
        </div>
      </div>
    </NCard>
    <NCard :title="t('proxy.setting')" class="mb-4" size="small" :bordered="true">
      <div class="flex flex-col">
        <NFormItem :label="t('proxy.protocal')">
          <NInput v-model:value="proxy.protocal" />
        </NFormItem>

        <NFormItem :label="t('proxy.host')">
          <NInput v-model:value="proxy.host" />
        </NFormItem>

        <NFormItem :label="t('proxy.port')">
          <NInputNumber v-model:value="proxy.port" />
        </NFormItem>
        <div>
          <NButton @click="handleSaveProxy">
            {{ t('common.save') }}
          </NButton>
        </div>
      </div>
    </NCard>
  </div>
</template>
