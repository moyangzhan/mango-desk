<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/app'
import { setLocale, t } from '@/locales'
import { emptyProxyInfo } from '@/utils/functions'

const appStore = useAppStore()
const activePlatform = ref('openai')
const activeTheme = ref(appStore.getTheme)
const activeLanguage = ref('en-US')
const activeTab = ref('openai')
const modelPlatformList = ref<ModelPlatform[]>([])
const proxy = ref<ProxyInfo>(emptyProxyInfo())

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

onMounted(async () => {
  console.log('common setting onMounted')
  const activePlatformName = await invoke('load_active_platform')
  activePlatform.value = activePlatformName as string
  const platforms = await invoke('load_model_platforms')
  modelPlatformList.value = platforms as ModelPlatform[]
  console.log('platforms', modelPlatformList.value)
  activeTab.value = activePlatform.value
  const activeLocale = await invoke('load_active_locale')
  console.log('active_locale', activeLocale)
  setLocale(activeLocale as 'en-US' | 'zh-CN')
  activeLanguage.value = activeLocale as string
  const proxyInfo = await invoke('load_proxy_info')
  proxy.value = proxyInfo as ProxyInfo
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
