<script setup lang="ts">
import { openPath } from '@tauri-apps/plugin-opener'
import { LogoGithub } from '@vicons/ionicons5'
import HowToUse from '../HowToUse.vue'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app';
import { t } from '@/locales'

const appVersion = ref('')
const appClientId = ref('')

function openUrl(path = '') {
  openPath(path).then((res) => {
    console.log('openUrl', res)
  })
}

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
    appClientId.value = await invoke('get_client_id')
  } catch (error) {
    console.error(`getVersion error: ${error}`)
  }
})
</script>

<template>
  <div class="flex flex-col h-full p-4 text-left space-y-8 w-full">
    <div class="flex flex-col w-full items-center justify-center pr-8">
      <NImage src="/mango-desk.png" alt="MangoDesk" width="200" height="200" preview-disabled />
      <p class="text-gray-700 dark:text-gray-300 mb-4">
        {{ t('about.description') }}
      </p>
    </div>
    <HowToUse :show-steps="true">
      <template #tip>
        <div class="font-semibold mb-2">
          {{ t('common.howToUse') }}
        </div>
      </template>
    </HowToUse>
    <div class="flex-1"></div>
    <div class="flex flex-col w-full space-y-2 text-left">
      <div class="font-semibold">
        {{ t('about.moreDetail') }}
      </div>
      <div>
        <NButton text @click="openUrl('https://github.com/moyangzhan/mango-desk')">
          <template #icon>
            <NIcon>
              <LogoGithub />
            </NIcon>
          </template>
          <span class="hover:underline text-xs">MangoDesk on GitHub</span>
        </NButton>
      </div>
      <div class="text-xs mt-2 flex flex-col space-y-2">
        <div class="text-sm font-semibold">
          {{ t('about.appInfo') }}
        </div>
        <div>
          {{ t('about.currentVersion') }}: v{{ appVersion }}
        </div>
        <div>
          {{ t('about.appClientId') }}: {{ appClientId }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped></style>
