<script setup lang="ts">
import { openPath } from '@tauri-apps/plugin-opener'
import { LogoGithub, DocumentTextOutline } from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { resourceDir } from '@tauri-apps/api/path'
import { readTextFile } from '@tauri-apps/plugin-fs'
import HowToUse from '../HowToUse.vue'
import { t } from '@/locales'

const appVersion = ref('')
const appClientId = ref('')

function openUrl(path = '') {
  openPath(path).then((res) => {
    console.log('openUrl', res)
  })
}

async function openCliDoc() {
  try {
    const resDir = await resourceDir()
    // 资源目录路径（Tauri 会自动处理路径分隔符）
    const docPath = `${resDir}docs/cli.md`
    const content = await readTextFile(docPath)
    // 创建一个临时 HTML 页面显示文档
    const htmlContent = `
      <!DOCTYPE html>
      <html>
      <head>
        <title>Mango Finder CLI Documentation</title>
        <style>
          body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; line-height: 1.6; }
          pre { background: #f5f5f5; padding: 12px; border-radius: 4px; overflow-x: auto; }
          code { background: #f5f5f5; padding: 2px 4px; border-radius: 2px; }
          h1, h2, h3 { color: #333; }
        </style>
      </head>
      <body>
        <pre>${content}</pre>
      </body>
      </html>
    `
    // 使用 data URL 打开
    const dataUrl = `data:text/html;charset=utf-8,${encodeURIComponent(htmlContent)}`
    openPath(dataUrl)
  } catch (error) {
    console.error('Failed to open CLI doc:', error)
  }
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
      <NImage src="/mango-desk.png" alt="MangoFinder" width="200" height="200" preview-disabled />
      <p class="text-gray-700 dark:text-gray-300 mb-4">
        {{ t('about.description') }}
      </p>
    </div>
    <HowToUse :show-steps="true">
      <template #tip>
        <div class="font-semibold mb-2">
          {{ t('common.usageGaide') }}
        </div>
      </template>
    </HowToUse>
    <div class="flex-1" />
    <div class="flex flex-col w-full space-y-2 text-left">
      <div class="font-semibold">
        {{ t('about.moreDetail') }}
      </div>
      <div>
        <NButton text type="primary" class="text-link" @click="openUrl('https://github.com/moyangzhan/mango-finder')">
          <template #icon>
            <NIcon>
              <LogoGithub />
            </NIcon>
          </template>
          <span class="text-xs">MangoFinder on GitHub</span>
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
      <div class="mt-4">
        <NButton text type="primary" class="text-link" @click="openCliDoc">
          <template #icon>
            <NIcon>
              <DocumentTextOutline />
            </NIcon>
          </template>
          <span class="text-xs">CLI Documentation</span>
        </NButton>
      </div>
    </div>
  </div>
</template>

<style scoped></style>
