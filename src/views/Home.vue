<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import { useDebounceFn } from '@vueuse/core'
import HowToUse from './HowToUse.vue'
import { t } from '@/locales'
import SvgIcon from '@/components/SvgIcon.vue'
import { useIndexerStore } from '@/stores/indexer'

const SEMANTIC_SEARCH = 1
const PATH_SEARCH = 2
const extIcons = ['csv', 'doc', 'docx', 'html', 'json', 'mp3', 'mp4', 'pdf', 'ppt', 'pptx', 'psd', 'rar', 'txt', 'xls', 'xlsx']
const query = ref('')
const searchResults = ref<SearchResult[]>([])
const searching = ref(false)
const selectedIndex = ref(-1)
const inputRef = ref<HTMLInputElement | null>(null)
const isFocused = ref(false)
const indexerStore = useIndexerStore()
const parsedContent = ref('')
const showContentModal = ref(false)
const showChunksModal = ref(false)
const matchChunks = ref<string[]>([])
const searchType = ref(SEMANTIC_SEARCH) // 1: semantic search, 2: path search

const focusInput = () => {
  inputRef.value?.focus()
}

const blurInput = () => {
  inputRef.value?.blur()
}

let debounceSearch = useDebounceFn(async () => {
  search()
}, 600)

watch(query, () => {
  let query_txt = query.value.trimStart()
  if (query_txt.startsWith('/') && searchType.value === SEMANTIC_SEARCH) {
    console.log('switch to path search')
    searchType.value = PATH_SEARCH
    debounceSearch = useDebounceFn(async () => {
      search()
    }, 300)
  } else if (!query_txt.startsWith('/') && searchType.value === PATH_SEARCH) {
    console.log('switch to semantic search')
    searchType.value = SEMANTIC_SEARCH
    //Sematic search is slower, so we use a longer debounce time
    debounceSearch = useDebounceFn(async () => {
      search()
    }, 600)
  }
})


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

async function loadChunks(ids: number[]) {
  showChunksModal.value = true
  matchChunks.value = []
  try {
    let chunks = await invoke<string[]>('load_chunks', { ids });
    if (chunks) {
      matchChunks.value = chunks
    }
  } catch (e) {
    console.log(e)
  }
}

function onClear() {
  query.value = ''
  searchResults.value = []
  selectedIndex.value = -1
  focusInput()
}

function escapeRegExp(str: string) {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function highlightPath(path: string, keywords: string[]) {
  if (!path || !keywords || keywords.length === 0) return path;
  return keywords.reduce((html, keyword) => {
    const safeKeyword = escapeRegExp(keyword);
    const regex = new RegExp(`(${safeKeyword})`, 'gi');
    return html.replace(
      regex,
      '<span class="font-bold text-gray-600 dark:text-gray-400">$1</span>'
    );
  }, path);
}

async function search() {
  if (searching.value || !query.value) {
    searchResults.value = []
    return
  }
  try {
    let query_txt = query.value.trim()
    let search_name = 'semantic_search'
    if (query.value.startsWith('/')) {
      search_name = 'path_search'
      query_txt = query.value.substring(1)
    }
    if (query_txt.length < 2) {
      window.$message.warning(t('common.queryTooShort'))
      return
    }
    selectedIndex.value = -1
    searching.value = true
    const res = await invoke<SearchResult[]>(search_name, { query: query_txt })
    if (res.length === 0) {
      window.$message.warning(t('common.noData'))
      searchResults.value = []
      return
    }
    searchResults.value = res
    searchResults.value.forEach((item) => {
      if (item.source === 'path' && item.matched_keywords.length > 0) {
        item.file_info.html_path = highlightPath(item.file_info.path, item.matched_keywords)
      } else {
        item.file_info.html_path = item.file_info.path
      }
      if (item.file_info.category !== 2)
        return

      // Load image data for display
      invoke('read_file_data', { path: item.file_info.path }).then((resp) => {
        if (!resp)
          throw new Error('No image data received')
        const mimeType = item.file_info.file_ext.toLowerCase() === 'png' ? 'image/png' : 'image/jpeg'
        const uint8Array = new Uint8Array(resp as ArrayBuffer)
        const blob = new Blob([uint8Array], { type: mimeType })
        const imageUrl = URL.createObjectURL(blob)
        item.file_info.file_data = imageUrl
      })
    })
  } catch (e) {
    console.log(e)
  } finally {
    searching.value = false
  }
}

const keyDown = (e: any) => {
  if (e.ctrlKey && e.key === 'Tab') {
    query.value = query.value.trim()
    if (query.value.startsWith('/')) {
      query.value = query.value.substring(1)
    } else {
      query.value = '/' + query.value
    }
    focusInput()
    debounceSearch()
    return
  } else if (e.key === 'Enter') {
    if (!isFocused.value && selectedIndex.value > -1)
      openFile(searchResults.value[selectedIndex.value].file_info.path)
    else
      debounceSearch()
  } else if (e.key === 'ArrowUp') {
    if (selectedIndex.value === 0) {
      focusInput()
      selectedIndex.value = -1
      return
    } else if (selectedIndex.value === -1) {
      selectedIndex.value = searchResults.value.length - 1
      return
    }
    blurInput()
    selectedIndex.value = Math.max(0, selectedIndex.value - 1)
  } else if (e.key === 'ArrowDown') {
    if (selectedIndex.value === searchResults.value.length - 1) {
      focusInput()
      selectedIndex.value = -1
      return
    } else if (selectedIndex.value === -1) {
      blurInput()
      selectedIndex.value = 0
      return
    }
    blurInput()
    selectedIndex.value = Math.min(searchResults.value.length - 1, selectedIndex.value + 1)
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
    <div v-if="searchResults.length === 0" class="mb-4 flex items-center">
      <NImage src="/mango-desk.png" alt="MangoDesk" width="100" height="100"
        class="transition-all duration-300 hover:scale-105" style="opacity: 0.8; filter: saturate(0.9)"
        preview-disabled />
      <div class="text-sm text-gray-400 mt-2">
        Awake your data
      </div>
    </div>
    <div class="flex flex-col w-full justify-center space-x-2 max-w-[80%]">
      <NInput ref="inputRef" v-model:value="query" class="flex-1 min-w-[100px] text-left" clearable
        :placeholder="searchType == PATH_SEARCH ? t('common.pathSearchTip.title') : t('common.semanticSearch')"
        @input="debounceSearch" @focus="isFocused = true" @blur="isFocused = false" @clear="onClear">
        <template #prefix>
          <span v-if="!query.startsWith('/')" class="text-gray-300 dark:text-gray-600">
            {{ t('common.content') }}
          </span>
          <span v-else class="text-gray-300 dark:text-gray-600">
            {{ t('common.path') }}
          </span>
        </template>
      </NInput>
      <div v-if="searchResults.length === 0" class="mt-2 text-xs text-gray-400 w-full text-left">
        <div>{{ t('common.semanticSearchTip.title') }}：{{ t('common.semanticSearchTip.description') }}</div>
        <div>{{ t('common.pathSearchTip.title') }}：{{ t('common.pathSearchTip.description') }}, {{
          t('common.pathSearchTip.example') }}</div>
      </div>
    </div>
    <div class="flex-1 flex flex-col w-full items-center justify-start mt-4"
      :class="searchResults.length > 0 ? 'border-t border-(--border-color)' : ''">
      <div v-if="searchResults.length === 0" class="flex flex-col mt-8 h-full space-y-4">
        <!-- Keyborad Shortcuts -->
        <div class="text text-sm text-gray-400 text-left">
          {{ t('common.searchKeyboradShortcuts') }}
        </div>
        <div class="flex space-x-2 text-sm text-gray-500">
          <div class="w-[65px] text-left">
            <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-300 rounded">Ctrl+Tab</kbd>
          </div>
          <span>{{ t('common.switchSearchMode') }}</span>
        </div>
        <div class="flex space-x-2 text-sm text-gray-500">
          <div class="w-[65px] text-left">
            <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-300 rounded">Esc</kbd>
          </div>
          <span>{{ t('common.searchClearTip') }}</span>
        </div>
        <div class="text text-sm text-gray-400 text-left">
          {{ t('common.resultKeyboradShortcuts') }}
        </div>
        <div class="flex space-x-2 text-sm text-gray-500">
          <div class="w-[65px] text-left">
            <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-300 rounded">↑↓</kbd>
          </div>
          <span>{{ t('common.navigateTip') }}</span>
        </div>
        <div class="flex space-x-2 text-sm text-gray-500">
          <div class="w-[65px] text-left">
            <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-300 rounded">Enter</kbd>
          </div>
          <span>{{ t('common.openTip') }}</span>
        </div>
        <div class="flex space-x-2 text-sm text-gray-500">
          <div class="w-[65px] text-left">
            <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-300 rounded">Esc</kbd>
          </div>
          <span>{{ t('common.clearTip') }}</span>
        </div>
      </div>

      <NImageGroup v-else>
        <div v-for="(item, idx) in searchResults" :key="item.file_info.path"
          class="flex w-full space-x-2 p-2 border-b border-(--border-color)"
          :style="selectedIndex === idx ? 'background-color: var(--secondary-bg-color);border: 1px solid var(--primary-color); box-sizing: border-box;border-radius: 0.25rem;' : ''">
          <div class="flex justify-center items-center">
            <NImage v-if="item.file_info.file_data" width="100" height="100" :src="item.file_info.file_data" />
            <div v-else-if="!item.file_info.file_data && !extIcons.includes(item.file_info.file_ext.toLowerCase())"
              class="w-[50px] h-[50px] flex justify-center items-center text-xl font-bold"
              style="opacity: 0.7;filter: saturate(0.5)">
              {{
                item.file_info.file_ext.toUpperCase()
              }}
            </div>
            <SvgIcon v-else :name="item.file_info.file_ext.toLowerCase()" width="50" height="50"
              style="opacity: 0.7;filter: saturate(0.5)" />
          </div>
          <div class="flex-1 flex flex-col text-left h-[50px]">
            <div class="cursor-pointer hover:underline hover:text-(--primary-color)"
              @click="openFile(item.file_info.path)">
              {{
                item.file_info.name }}
            </div>
            <div class="text-xs text-gray-500">
              <div v-html="item.file_info.html_path"></div>
            </div>
          </div>
          <div class="flex justify-center items-center">
            <div v-if="indexerStore.indexerSetting.save_parsed_content.document && item.file_info.category === 1">
              <NButton size="tiny" text @click="loadFileDetail(item.file_info.id)">
                {{ t('indexer.parsedContent') }}
              </NButton>
            </div>
            <div
              v-if="indexerStore.indexerSetting.save_parsed_content.image && item.file_info.category === 2 || (indexerStore.indexerSetting.save_parsed_content.audio && item.file_info.category === 3)">
              <NButton size="tiny" text @click="loadFileDetail(item.file_info.id)">
                {{ t('indexer.recognitionText') }}
              </NButton>
            </div>
            <div v-if="item.matched_chunk_ids && item.matched_chunk_ids.length > 0" class="ml-2">
              <NButton size="tiny" text @click="loadChunks(item.matched_chunk_ids)">
                {{ t('common.matchedSegments', { count: item.matched_chunk_ids.length }) }}
              </NButton>
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
    <NModal v-model:show="showChunksModal" preset="card" :title="t('common.matchedSegments')"
      style="width: 80%; height:80%;">
      <div style="max-height: 600px;overflow-y: auto;" class="select-text">
        <div v-if="matchChunks.length > 0">
          <div v-for="(chunk, index) in matchChunks" :key="index" class="mb-4">
            <div class="mb-2">
              <strong>{{ t('common.segment') }} {{ index + 1 }}:</strong>
            </div>
            <div>
              {{ chunk }}
            </div>
          </div>
        </div>
        <div v-else>
          {{ t('common.noData') }}
        </div>
      </div>
    </NModal>
  </div>
</template>

<style scoped></style>
