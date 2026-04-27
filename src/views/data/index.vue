<script setup lang="ts">
import type { DataTableColumns, PaginationInfo } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import { useWindowSize } from '@vueuse/core'
import { KeyboardArrowDownOutlined } from '@vicons/material'
import { getFileColumns } from './columns'

import { useIndexerStore } from '@/stores/indexer'
import { t } from '@/locales'

interface FileSortInfo {
  columnKey: string
  order: 'ascend' | 'descend'
}

const indexerStore = useIndexerStore()
const { height } = useWindowSize()
const page = ref(1)
const pageSize = ref(20)
const files = ref<FileInfo[]>([])
const selectedFileIds = ref<number[]>([])
const categoryFilter = ref<number | null>(null)
const searchKeyword = ref('')
const sortInfo = ref<FileSortInfo | null>(null)

const paginationReactive = reactive({
  page: 1,
  pageCount: 1,
  pageSize: 20,
  itemCount: 0,
  prefix({ itemCount }: PaginationInfo) {
    return `${t('common.total')}: ${itemCount} `
  },
})

const categoryCounts = computed(() => {
  const baseFiles = files.value.filter((f) => {
    if (!searchKeyword.value)
      return true
    return f.path.toLowerCase().includes(searchKeyword.value.toLowerCase())
  })
  return {
    all: baseFiles.length,
    document: baseFiles.filter(f => f.category === 1).length,
    image: baseFiles.filter(f => f.category === 2).length,
    audio: baseFiles.filter(f => f.category === 3).length,
    other: baseFiles.filter(f => f.category === 4 || f.category === 5).length,
  }
})

const filteredFiles = computed(() => {
  let result = files.value

  // Filter by search keyword
  if (searchKeyword.value) {
    result = result.filter(f =>
      f.path.toLowerCase().includes(searchKeyword.value.toLowerCase()),
    )
  }

  // Filter by category
  if (categoryFilter.value !== null) {
    if (categoryFilter.value === 5)
      result = result.filter(f => f.category === 4 || f.category === 5)
    else
      result = result.filter(f => f.category === categoryFilter.value)
  }

  // Sort
  if (sortInfo.value) {
    const { columnKey, order } = sortInfo.value
    result = [...result].sort((a, b) => {
      const aVal = a[columnKey as keyof FileInfo]
      const bVal = b[columnKey as keyof FileInfo]
      if (aVal === bVal)
        return 0
      if (aVal === null || aVal === undefined)
        return 1
      if (bVal === null || bVal === undefined)
        return -1
      const cmp = aVal < bVal ? -1 : 1
      return order === 'ascend' ? cmp : -cmp
    })
  }

  return result
})

function handleSortChange(options: { columnKey: string; order: 'ascend' | 'descend' | false }) {
  if (options.order === false)
    sortInfo.value = null

  else
    sortInfo.value = { columnKey: options.columnKey, order: options.order }
}

function handleCategoryFilter(category: number | null) {
  categoryFilter.value = category
  selectedFileIds.value = []
}

const handleOpenPath = (path: string) => {
  openPath(path)
}

async function handleDeleteSelected() {
  if (selectedFileIds.value.length === 0)
    return

  try {
    await invoke('delete_index_items', { fileIds: selectedFileIds.value })
    window.$message.success(t('common.operationSuccess'))
    selectedFileIds.value = []
    handlePageChange(1)
  } catch (e) {
    console.error('Failed to delete files:', e)
    window.$message.error(t('common.operationFailed'))
  }
}

function handleCheck(rowKeys: (string | number)[]) {
  selectedFileIds.value = rowKeys as number[]
}

const showClearConfirm = ref(false)
const clearConfirmType = ref<'selected' | 'all'>('selected')

const deleteDropdownOptions = computed(() => [
  {
    label: () => h('div', { style: { minWidth: '110px' } }, t('common.deleteAll')),
    key: 'all',
  },
])

function handleDeleteDropdownSelect(key: string) {
  if (key === 'all') {
    clearConfirmType.value = 'all'
    showClearConfirm.value = true
  }
}

async function handleConfirmClear() {
  if (clearConfirmType.value === 'selected')
    await handleDeleteSelected()

  else
    await clearIndex()

  showClearConfirm.value = false
}

const fileColumns: DataTableColumns<FileInfo> = getFileColumns(handleOpenPath)

async function handlePageChange(currentPage: number) {
  page.value = currentPage
  loadFiles()
}

async function clearIndex() {
  await invoke('clear_index')
  handlePageChange(1)
}

async function loadFiles() {
  const rows = await invoke('load_files', { page: page.value, pageSize: pageSize.value })
  files.value = rows as FileInfo[]
  if (files.value.length > 0)
    paginationReactive.page = page.value

  if (page.value === 1) {
    const totalResp = await invoke('count_files')
    const total = totalResp as number
    paginationReactive.pageCount = total / pageSize.value
    paginationReactive.itemCount = total
  }
}

watch(() => indexerStore.indexProcessing, (newVal) => {
  if (!newVal)
    handlePageChange(1)
})

watch(() => indexerStore.watcherProcessing, (newVal) => {
  if (!newVal)
    handlePageChange(1)
})

onMounted(() => {
  page.value = 1
  loadFiles()
})
</script>

<template>
  <div class="h-full m-auto p-4">
    <NCard :title="t('indexer.indexedFiles')" class="shadow-sm">
      <!-- Toolbar: Actions (left) | Search + Filter (right) -->
      <div class="flex flex-wrap items-center gap-2 mb-3 justify-between">
        <!-- Left: Actions -->
        <div class="flex items-center gap-2">
          <NButton ghost size="small" @click="handlePageChange(1)">
            {{ t('common.refresh') }}
          </NButton>
          <NButtonGroup v-if="selectedFileIds.length > 0">
            <NButton ghost type="error" size="small" @click="clearConfirmType = 'selected'; showClearConfirm = true">
              {{ t('common.deleteSelected') }} ({{ selectedFileIds.length }})
            </NButton>
            <NDropdown
              :options="deleteDropdownOptions"
              placement="bottom-end"
              @select="handleDeleteDropdownSelect"
            >
              <NButton ghost type="error" size="small">
                <template #icon>
                  <NIcon><KeyboardArrowDownOutlined /></NIcon>
                </template>
              </NButton>
            </NDropdown>
          </NButtonGroup>
        </div>
        <!-- Right: Search and Category Filter -->
        <div class="flex flex-wrap items-center gap-2 justify-end">
          <NButton
            :type="categoryFilter === null ? 'primary' : 'default'" :ghost="categoryFilter === null" size="small"
            @click="handleCategoryFilter(null)"
          >
            {{ t('cluster.pairing.all') }} ({{ categoryCounts.all }})
          </NButton>
          <NButton
            :type="categoryFilter === 1 ? 'primary' : 'default'" :ghost="categoryFilter === 1" size="small"
            @click="handleCategoryFilter(1)"
          >
            {{ t('common.document') }} ({{ categoryCounts.document }})
          </NButton>
          <NButton
            :type="categoryFilter === 2 ? 'primary' : 'default'" :ghost="categoryFilter === 2" size="small"
            @click="handleCategoryFilter(2)"
          >
            {{ t('common.image') }} ({{ categoryCounts.image }})
          </NButton>
          <NButton
            :type="categoryFilter === 3 ? 'primary' : 'default'" :ghost="categoryFilter === 3" size="small"
            @click="handleCategoryFilter(3)"
          >
            {{ t('common.audio') }} ({{ categoryCounts.audio }})
          </NButton>
          <NButton
            :type="categoryFilter === 5 ? 'primary' : 'default'" :ghost="categoryFilter === 5" size="small"
            @click="handleCategoryFilter(5)"
          >
            {{ t('common.other') }} ({{ categoryCounts.other }})
          </NButton>
          <NInput
            v-model:value="searchKeyword" :placeholder="t('indexer.searchPathPlaceholder')" clearable size="small"
            style="width: 200px" @update:value="selectedFileIds = []"
          />
        </div>
      </div>
      <NDataTable
        remote :columns="fileColumns" :data="filteredFiles" :pagination="paginationReactive" :bordered="false"
        striped scroll-x="1700" :max-height="height - 320" :row-key="(row: FileInfo) => row.id"
        :checked-row-keys="selectedFileIds" @update:page="handlePageChange" @update:checked-row-keys="handleCheck"
        @update:sorter="handleSortChange"
      />

      <!-- Delete Confirmation Modal -->
      <NModal v-model:show="showClearConfirm" preset="dialog" :title="t('common.warning')" :auto-focus="false">
        <template #default>
          <p v-if="clearConfirmType === 'selected'">
            {{ t('indexer.deleteSelectedConfirmation') }}
          </p>
          <p v-else>
            {{ t('indexer.clearIndexConfirmation') }}
          </p>
        </template>
        <template #action>
          <NButton @click="showClearConfirm = false">
            {{ t('common.cancel') }}
          </NButton>
          <NButton type="error" ghost @click="handleConfirmClear">
            {{ t('common.confirm') }}
          </NButton>
        </template>
      </NModal>
    </NCard>
  </div>
</template>
