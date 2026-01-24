<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useWindowSize } from '@vueuse/core'
import FileSelector from './FileSelector.vue'
import FileWatcher from './FileWatcher.vue'
import { getTaskColumns } from './columns'
import type { PaginationInfo } from 'naive-ui'
import { t } from '@/locales'

const { height } = useWindowSize()
const taskPageReactive = reactive({
  page: 1,
  pageSize: 10,
  itemCount: 0,
  prefix({ itemCount }: PaginationInfo) {
    return `${t('common.total')}: ${itemCount} `
  }
})
const tasks = ref<IndexingTask[]>([])
const showTasks = ref(false)

const taskColumns = getTaskColumns((deleteId: number) => {
  console.log('deleteId', deleteId)
  invoke('delete_indexing_task', { taskId: deleteId }).then(() => {
    handleTaskPageChange(1)
  })
})

async function handleTaskPageChange(currentPage: number) {
  taskPageReactive.page = currentPage
  loadIndexingTask()
}

async function loadIndexingTask() {
  const { page, pageSize } = taskPageReactive
  const rows = await invoke<IndexingTask[]>('load_indexing_tasks', { page, pageSize })
  tasks.value = rows
  if (taskPageReactive.page === 1 || taskPageReactive.itemCount === 0) {
    const total = await invoke<number>('count_indexing_tasks')
    console.log('total', total)
    taskPageReactive.itemCount = total
  }
}

function indexingFinish() {
  loadIndexingTask()
}

// watch([width, height], ([newWidth, newHeight]) => {
//   console.log(`Window size: ${newWidth}x${newHeight}`)
// })

onMounted(() => {
  loadIndexingTask()
})
</script>

<template>
  <div class="h-full m-auto p-4">
    <NCard :title="t('indexer.indexer')" class="mb-2">
      <FileSelector @indexing-finish="indexingFinish" />
      <div class="flex justify-end mt-4">
        <NButton type="primary" text @click="showTasks = true">
          <span class="hover:underline">{{ t('indexer.indexingTaskHistory') }}</span>
        </NButton>
      </div>
    </NCard>
    <FileWatcher />
    <NModal v-model:show="showTasks" preset="card" :title="t('indexer.indexingTaskHistory')"
      style="width: 80%; height:80%; max-width: 1200px;">
      <NDataTable remote :columns="taskColumns" :data="tasks" :pagination="taskPageReactive" :bordered="true" striped
        scroll-x="1300" :max-height="height - 250" @update:page="handleTaskPageChange" />
    </NModal>
  </div>
</template>

<style scoped></style>
