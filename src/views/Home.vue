<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import { useDebounceFn } from '@vueuse/core'
import HowToUse from './HowToUse.vue'
import { t } from '@/locales'
import SvgIcon from '@/components/SvgIcon.vue'
import { useIndexerStore } from '@/stores/indexer'

const extIcons = ['csv', 'doc', 'docx', 'html', 'json', 'mp3', 'mp4', 'pdf', 'ppt', 'pptx', 'psd', 'rar', 'txt', 'xls', 'xlsx']
const query = ref('')
const files = ref<FileInfo[]>([])
const searching = ref(false)
const selectedIndex = ref(-1)
const inputRef = ref<HTMLInputElement | null>(null)
const isFocused = ref(false)
const indexerStore = useIndexerStore()
const parsedContent = ref('')
const showContentModal = ref(false)

const focusInput = () => {
  inputRef.value?.focus()
}

const blurInput = () => {
  inputRef.value?.blur()
}

const quickSearch = useDebounceFn(async () => {
  if (query.value !== '') {
    invoke('quick_search', { query: query.value }).then((res) => {
      console.log(res)
    })
  }
}, 300)

function openFile(path = '') {
  openPath(path).then((res) => {
    console.log('openfile', res)
  })
}

async function loadFileDetail(id = 0) {
  showContentModal.value = true
  parsedContent.value = ''
  try {
    let fileInfo = await invoke<FileInfo>('load_file_detail', { fileId: id });
    if (fileInfo) {
      parsedContent.value = fileInfo.content
    }
  } catch (e) {
    console.log(e)
  }
}

function onClear() {
  query.value = ''
  files.value = []
  selectedIndex.value = -1
  focusInput()
}

async function search() {
  if (searching.value || !query.value) {
    window.$message.warning(t('message.inputEmpty'))
    return
  }
  selectedIndex.value = -1
  searching.value = true
  try {
    const res = await invoke('search', { query: query.value })
    files.value = res as FileInfo[]
    if (files.value.length === 0)
      window.$message.warning(t('common.noData'))

    // Test data
    // if (files.value.length > 0) {
    //   files.value[0].category = 2
    //   files.value[0].path = 'D:\\data\\test\\images\\屏幕截图_15-10-2025_162154.jpeg'
    //   files.value[1].category = 2
    //   files.value[1].path = 'D:\\data\\test\\images\\111.webp'
    // }
    files.value.forEach((file) => {
      if (file.category !== 2)
        return

      // Load image data for display
      invoke('read_file_data', { path: file.path }).then((resp) => {
        if (!resp)
          throw new Error('No image data received')

        console.log('imageData length:', resp)
        const mimeType = file.file_ext.toLowerCase() === 'png' ? 'image/png' : 'image/jpeg'
        const uint8Array = new Uint8Array(resp as ArrayBuffer)
        const blob = new Blob([uint8Array], { type: mimeType })
        const imageUrl = URL.createObjectURL(blob)
        file.file_data = imageUrl
      })
    })
  } catch (e) {
    console.log(e)
  } finally {
    searching.value = false
  }
}

const keyDown = (e: any) => {
  if (e.key === 'Enter') {
    if (!isFocused.value && selectedIndex.value > -1)
      openFile(files.value[selectedIndex.value].path)
    else
      search()
  } else if (e.key === 'ArrowUp') {
    if (selectedIndex.value === 0) {
      focusInput()
      selectedIndex.value = -1
      return
    } else if (selectedIndex.value === -1) {
      selectedIndex.value = files.value.length - 1
      return
    }
    blurInput()
    selectedIndex.value = Math.max(0, selectedIndex.value - 1)
  } else if (e.key === 'ArrowDown') {
    if (selectedIndex.value === files.value.length - 1) {
      focusInput()
      selectedIndex.value = -1
      return
    } else if (selectedIndex.value === -1) {
      blurInput()
      selectedIndex.value = 0
      return
    }
    blurInput()
    selectedIndex.value = Math.min(files.value.length - 1, selectedIndex.value + 1)
  } else if (e.key === 'Escape') {
    if (isFocused.value) {
      onClear()
    } else if (selectedIndex.value > -1) {
      selectedIndex.value = -1
      focusInput()
    }
  }
}

onMounted(async () => {
  let indexerSetting = await invoke<IndexerSetting>('load_indexer_setting')
  indexerStore.setIndexerSetting(indexerSetting)
  window.addEventListener('keydown', keyDown)
})
onUnmounted(() => {
  window.removeEventListener('keydown', keyDown, false)
})
</script>

<template>
  <div class="h-full flex flex-col items-center p-4 text-center">
    <div v-if="files.length === 0" class="mb-4 flex items-center">
      <NImage src="/mango-desk.png" alt="MangoDesk" width="100" height="100"
        class="transition-all duration-300 hover:scale-105" style="opacity: 0.8; filter: saturate(0.9)"
        preview-disabled />
      <div class="text-sm text-gray-400 mt-2">
        Awake your data
      </div>
    </div>
    <div class="flex w-full justify-center space-x-2 max-w-[80%]">
      <NInput ref="inputRef" v-model:value="query" class="flex-1 min-w-[100px] text-left" clearable
        :placeholder="t('common.sematicSearch')" @input="quickSearch" @focus="isFocused = true"
        @blur="isFocused = false" @clear="onClear" />
      <NButton :loading="searching" @click="search">
        {{ t('common.search') }}
      </NButton>
    </div>
    <div class="flex-1 flex flex-col w-full items-center justify-start mt-4"
      :class="files.length > 0 ? 'border-t border-(--border-color)' : ''">
      <div v-if="files.length === 0" class="flex flex-col mt-8 h-full space-y-4">
        <!-- Keyborad Shortcuts -->
        <div class="text text-sm text-gray-400 text-left">
          {{ t('common.searchKeyboradShortcuts') }}
        </div>
        <div class="flex space-x-2 text-sm text-gray-500">
          <div class="w-[50px] text-left">
            <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-300 rounded">Enter</kbd>
          </div>
          <span>{{ t('common.searchTip') }}</span>
        </div>
        <div class="flex space-x-2 text-sm text-gray-500">
          <div class="w-[50px] text-left">
            <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-300 rounded">Esc</kbd>
          </div>
          <span>{{ t('common.searchClearTip') }}</span>
        </div>
        <div class="text text-sm text-gray-400 text-left">
          {{ t('common.resultKeyboradShortcuts') }}
        </div>
        <div class="flex space-x-2 text-sm text-gray-500">
          <div class="w-[50px] text-left">
            <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-300 rounded">↑↓</kbd>
          </div>
          <span>{{ t('common.navigateTip') }}</span>
        </div>
        <div class="flex space-x-2 text-sm text-gray-500">
          <div class="w-[50px] text-left">
            <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-300 rounded">Enter</kbd>
          </div>
          <span>{{ t('common.openTip') }}</span>
        </div>
        <div class="flex space-x-2 text-sm text-gray-500">
          <div class="w-[50px] text-left">
            <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-300 rounded">Esc</kbd>
          </div>
          <span>{{ t('common.clearTip') }}</span>
        </div>
      </div>

      <NImageGroup v-else>
        <div v-for="(file, idx) in files" :key="file.path"
          class="flex w-full space-x-2 p-2 border-b border-(--border-color)"
          :style="selectedIndex === idx ? 'background-color: var(--secondary-bg-color);border: 1px solid var(--primary-color); box-sizing: border-box;border-radius: 0.25rem;' : ''">
          <div class="flex justify-center items-center">
            <NImage v-if="file.file_data" width="100" height="100" :src="file.file_data" />
            <div v-else-if="!file.file_data && !extIcons.includes(file.file_ext.toLowerCase())"
              class="w-[50px] h-[50px] flex justify-center items-center text-xl font-bold"
              style="opacity: 0.7;filter: saturate(0.5)">
              {{
                file.file_ext.toUpperCase()
              }}
            </div>
            <SvgIcon v-else :name="file.file_ext.toLowerCase()" width="50" height="50"
              style="opacity: 0.7;filter: saturate(0.5)" />
          </div>
          <div class="flex-1 flex flex-col text-left h-[50px]">
            <div class="cursor-pointer hover:underline hover:text-(--primary-color)" @click="openFile(file.path)">
              {{
                file.name }}
            </div>
            <div class="text-xs text-gray-500">
              {{ file.path }}
            </div>
          </div>
          <div style="width:100px" class="flex justify-center items-center">
            <div v-if="indexerStore.indexerSetting.save_parsed_content.document && file.category === 1">
              <n-button size="tiny" text @click="loadFileDetail(file.id)">
                {{ t('indexer.parsedContent') }}
              </n-button>
            </div>
            <div
              v-if="indexerStore.indexerSetting.save_parsed_content.image && file.category === 2 || (indexerStore.indexerSetting.save_parsed_content.audio && file.category === 3)">
              <n-button size="tiny" text @click="loadFileDetail(file.id)">
                {{ t('indexer.recognitionText') }}
              </n-button>
            </div>
          </div>
        </div>
      </NImageGroup>
    </div>
    <HowToUse />
    <NModal v-model:show="showContentModal" preset="card" :title="t('indexer.parsedContent')"
      style="width: 80%; height:80%;">
      <div style="max-height: 600px;overflow-y: auto;" class="select-text">
        <div v-if="parsedContent">
          {{ parsedContent }}
        </div>
        <div v-else>
          {{ t('common.noData') }}
        </div>
      </div>
    </NModal>
  </div>
</template>

<style scoped></style>
