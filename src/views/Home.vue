<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import { useDebounceFn } from '@vueuse/core'
import HowToUse from './HowToUse.vue'
import { t } from '@/locales'
import SvgIcon from '@/components/SvgIcon.vue'
import { useIndexerStore } from '@/stores/indexer'

const SEMANTIC_SEARCH = 1
const KEYWORD_SEARCH = 2
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

const hitTypeLabels = computed<Record<string, string>>(() => ({
  pathKeyword: t('common.hitType.pathKeyword'),
  contentKeyword: t('common.hitType.contentKeyword'),
  contentSemantic: t('common.hitType.contentSemantic'),
  metaSemantic: t('common.hitType.metaSemantic'),
}))

const focusInput = () => {
  inputRef.value?.focus()
}

const blurInput = () => {
  inputRef.value?.blur()
}

let debounceSearch = useDebounceFn(async () => {
  search()
}, 600)

watch(searchType, (newVal) => {
  if (newVal === SEMANTIC_SEARCH) {
    console.log('switch to semantic search')
    debounceSearch = useDebounceFn(async () => {
      search()
    }, 300)
  } else if (newVal === KEYWORD_SEARCH) {
    console.log('switch to keyword search')
    //Sematic search is slower, so we use a longer debounce time
    debounceSearch = useDebounceFn(async () => {
      search()
    }, 600)
  }
  debounceSearch()
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

async function loadChunks(ids: number[], keywords: string[]) {
  showChunksModal.value = true
  matchChunks.value = []
  try {
    let chunks = await invoke<string[]>('load_chunks', { ids });
    if (chunks) {
      if (keywords && keywords.length > 0) {
        matchChunks.value = chunks.map((chunk) => {
          return highlightText(chunk, keywords)
        })
      } else {
        matchChunks.value = chunks
      }
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

function highlightText(text: string, keywords: string[]) {
  if (!text || !keywords || keywords.length === 0) return text;
  return keywords.reduce((html, keyword) => {
    const safeKeyword = escapeRegExp(keyword);
    const regex = new RegExp(`(${safeKeyword})`, 'gi');
    return html.replace(
      regex,
      '<span class="font-bold text-(--match-word-color)">$1</span>'
    );
  }, text);
}

async function search() {
  if (searching.value || !query.value) {
    searchResults.value = []
    return
  }
  try {
    let query_txt = query.value.trim()
    let search_name = 'semantic_search'
    if (searchType.value === KEYWORD_SEARCH) {
      search_name = 'keyword_search'
      query_txt = query.value.trim()
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
      if (item.hit_types.includes('pathKeyword') && item.matched_keywords.length > 0) {
        item.file_info.html_path = highlightText(item.file_info.path, item.matched_keywords)
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
    searchType.value = searchType.value === SEMANTIC_SEARCH ? KEYWORD_SEARCH : SEMANTIC_SEARCH
    query.value = query.value.trim()
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
      <NImage src="/mango-desk.png" alt="MangoFinder" width="100" height="100"
        class="transition-all duration-300 hover:scale-105" style="opacity: 0.8; filter: saturate(0.9)"
        preview-disabled />
      <div class="text-sm text-gray-400 mt-2">
        Awake your data
      </div>
    </div>
    <div class="flex flex-col w-full justify-center space-x-2 max-w-[80%]">
      <div class="flex flex-row items-center justify-center space-x-2">
        <NInput ref="inputRef" v-model:value="query" class="flex-1 min-w-[100px] text-left" clearable
          :placeholder="searchType == KEYWORD_SEARCH ? t('common.keywordSearchTip.description') : t('common.semanticSearchTip.description')"
          @input="debounceSearch" @focus="isFocused = true" @blur="isFocused = false" @clear="onClear">
          <template #prefix>
            <NButton type="primary" text
              @click="searchType = searchType === SEMANTIC_SEARCH ? KEYWORD_SEARCH : SEMANTIC_SEARCH">
              <span v-if="searchType === SEMANTIC_SEARCH" class="pr-2">
                {{ t('common.semantic') }}
              </span>
              <span v-else class="pr-2">
                {{ t('common.keyword') }}
              </span>
            </NButton>
          </template>
        </NInput>
      </div>
      <div v-if="searchResults.length === 0" class="mt-2 text-xs text-gray-400 w-full text-left">
        <div>{{ t('common.semanticSearchTip.title') }}：{{ t('common.semanticSearchTip.description') }}</div>
        <div>{{ t('common.keywordSearchTip.title') }}：{{ t('common.keywordSearchTip.description') }}, {{
          t('common.keywordSearchTip.example') }}</div>
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
          class="group w-full p-2 border-b border-(--border-color)"
          :style="selectedIndex === idx ? 'background-color: var(--secondary-bg-color);border: 1px solid var(--primary-color); box-sizing: border-box;border-radius: 0.25rem;' : ''">
          <!-- Icon + File info -->
          <div class="flex space-x-2">
            <!-- Large image: top aligned -->
            <div v-if="item.file_info.file_data && item.file_info.category === 2" class="flex justify-center items-start shrink-0 pt-0.5">
              <NImage width="100" :src="item.file_info.file_data" />
            </div>
            <!-- Small icon: vertically centered -->
            <div v-else class="flex justify-center items-center shrink-0">
              <NImage v-if="item.file_info.file_data" width="40" height="40" :src="item.file_info.file_data" />
              <div v-else-if="!item.file_info.file_data && !extIcons.includes(item.file_info.file_ext.toLowerCase())"
                class="w-10 h-10 flex justify-center items-center text-sm font-bold"
                style="opacity: 0.7;filter: saturate(0.5)">
                {{ item.file_info.file_ext.toUpperCase() }}
              </div>
              <SvgIcon v-else :name="item.file_info.file_ext.toLowerCase()" width="40" height="40"
                style="opacity: 0.7;filter: saturate(0.5)" />
            </div>
            <div class="flex-1 flex flex-col justify-between text-left min-w-0 min-h-14">
              <div class="min-h-11">
                <div class="cursor-pointer hover:underline hover:text-(--primary-color) truncate"
                  style="font-weight: 550"
                  @click="openFile(item.file_info.path)">
                  {{ item.file_info.name }}
                </div>
                <div class="text-xs truncate">
                  <div v-html="item.file_info.html_path"></div>
                </div>
              </div>
              <!-- Third row: Actions + Metadata -->
              <div class="flex justify-between items-center text-xs text-gray-400">
                <div class="flex items-center gap-2">
                  <NButton v-if="indexerStore.indexerSetting.save_parsed_content.document && item.file_info.category === 1"
                    size="tiny" @click="loadFileDetail(item.file_info.id)">
                    {{ t('indexer.parsedContent') }}
                  </NButton>
                  <NButton
                    v-if="indexerStore.indexerSetting.save_parsed_content.image && item.file_info.category === 2 || (indexerStore.indexerSetting.save_parsed_content.audio && item.file_info.category === 3)"
                    size="tiny" @click="loadFileDetail(item.file_info.id)">
                    {{ t('indexer.recognitionText') }}
                  </NButton>
                  <NButton v-if="item.matched_chunk_ids && item.matched_chunk_ids.length > 0" size="tiny"
                    @click="loadChunks(item.matched_chunk_ids, item.matched_keywords)">
                    {{ t('common.matchedSegments', { count: item.matched_chunk_ids.length }) }}
                  </NButton>
                </div>
                <div class="flex items-center gap-2">
                  <NTooltip v-if="item.hit_types && item.hit_types.length > 0">
                    <template #trigger>
                      <span class="flex gap-1">
                        <NTag v-for="hitType in item.hit_types" :key="hitType" size="tiny" :bordered="false">
                          {{ hitTypeLabels[hitType] || hitType }}
                        </NTag>
                      </span>
                    </template>
                    {{ t('common.hitTypeTip') }}
                  </NTooltip>
                  <NTooltip v-if="item.score">
                    <template #trigger>
                      <NTag size="tiny" :bordered="false">
                        {{ item.score }}%
                      </NTag>
                    </template>
                    {{ t('common.score') }}: {{ item.score }}%
                  </NTooltip>
                </div>
              </div>
            </div>
          </div>
        </div>
      </NImageGroup>
    </div>
    <HowToUse v-if="searchResults.length === 0" />
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
            <div v-html="chunk"></div>
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
