<script setup lang="ts">
import type { DataTableColumns } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import { useWindowSize } from '@vueuse/core'
import { getFileColumns } from './columns'
import type { PaginationInfo } from 'naive-ui'
import { useIndexerStore } from '@/stores/indexer'
import { t } from '@/locales'

const indexerStore = useIndexerStore()
const { height } = useWindowSize()
const page = ref(1)
const pageSize = ref(20)
const files = ref<FileInfo[]>([])
const paginationReactive = reactive({
  page: 1,
  pageCount: 1,
  pageSize: 20,
  itemCount: 0,
  prefix({ itemCount }: PaginationInfo) {
    return `${t('common.total')}: ${itemCount} `
  }
})

const handleOpenPath = (path: string) => {
  openPath(path)
}

const handleDeleteItem = (id: number) => {
  invoke('delete_index_item', { fileId: id }).then(() => {
    handlePageChange(1)
  })
}
const fileColumns: DataTableColumns<FileInfo> = getFileColumns(handleOpenPath, handleDeleteItem)

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
  if (!newVal) {
    handlePageChange(1)
  }
})

onMounted(() => {
  page.value = 1
  loadFiles()
})
</script>

<template>
  <div class="h-full m-auto p-4">
    <NCard :title="t('indexer.indexedFiles')" class="shadow-sm">
      <div class="flex mb-2 justify-between">
        <NButton ghost size="small" @click="handlePageChange(1)" class="mr-2">
          {{ t('common.refresh')
          }}
        </NButton>
        <NPopconfirm :positive-text="t('common.confirm')" placement="left" :negative-text="t('common.cancel')"
          @positive-click="clearIndex">
          <template #trigger>
            <NButton text size="small" class="remove-all-button">
              {{ t('common.removeAll') }}
            </NButton>
          </template>
          {{ t('indexer.clearIndexConfirmation') }}
        </NPopconfirm>
      </div>
      <NDataTable remote :columns="fileColumns" :data="files" :pagination="paginationReactive" :bordered="false" striped
        scroll-x="1700" :max-height="height - 260" @update:page="handlePageChange" />
    </NCard>
  </div>
</template>

<style scoped>
.remove-all-button:hover {
  color: #cf3050;
}
</style>
