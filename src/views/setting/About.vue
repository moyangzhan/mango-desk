<script setup lang="ts">
import { openPath } from '@tauri-apps/plugin-opener'
import { LogoGithub } from '@vicons/ionicons5'
import HowToUse from '../HowToUse.vue'
import { t } from '@/locales'

function openUrl(path = '') {
  openPath(path).then((res) => {
    console.log('openUrl', res)
  })
}
</script>

<template>
  <div class="flex flex-col h-100vh p-4 text-left space-y-8 w-full">
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
    <div>
      <div class="font-semibold mb-2">
        {{ t('about.moreDetail') }}
      </div>
      <NButton text @click="openUrl('https://github.com/moyangzhan/mango-desk')">
        <template #icon>
          <NIcon>
            <LogoGithub />
          </NIcon>
        </template>
        <span class="hover:underline">MangoDesk on GitHub</span>
      </NButton>
    </div>
  </div>
</template>

<style scoped></style>
