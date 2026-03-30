<script lang="ts" setup>
import { AttachFileOutlined, DeleteOutlined, DoneOutlineRound, FileOpenOutlined, FolderOpenOutlined, FolderOutlined, StopCircleOutlined } from '@vicons/material'
import { open } from '@tauri-apps/plugin-dialog'
import { TauriEvent, listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import type { Event } from '@tauri-apps/api/event'
import { useIndexerStore } from '@/stores/indexer'
import router from '@/router'
import { t } from '@/locales'

const emit = defineEmits<Emit>()
interface Emit {
  (ev: 'indexingFinish'): void
  (ev: 'indexingStop'): void
}

const indexerStore = useIndexerStore()
const isDragOver = ref(false)
const selectedList = ref<SelectedItem[]>([])
const message = useMessage()
const btnDisabled = ref(false)
const indexingTitle = ref('')
const indexingMsg = ref('')

// Store unlisten functions for cleanup
const unlistenFns: UnlistenFn[] = []

interface DragPayload {
  paths: string[]
  position: { x: number; y: number }
}

async function openDirDialog() {
  // Replace browser's native input with Tauri dialog
  // This prevents the default file upload confirmation dialog such as ("Do you want to upload [number] files to this site?")
  const paths = await open({ directory: true, multiple: true })
  if (typeof paths === 'string') {
    addPath(paths, true)
  } else if (Array.isArray(paths)) {
    paths.forEach((path) => {
      addPath(path, true)
    })
  }
}

async function openFileDialog() {
  const paths = await open({ directory: false, multiple: true })
  if (typeof paths === 'string') {
    addPath(paths, false)
  } else if (Array.isArray(paths)) {
    paths.forEach((path) => {
      addPath(path, false)
    })
  }
}

function addPath(path: string, isDirectory: boolean) {
  const idStr = path
  if (selectedList.value.find(item => item.id === idStr)) {
    message.warning(`${t('common.alreadyExists')}: ${path}`)
    return
  }
  selectedList.value.push({
    id: idStr,
    name: path,
    type: isDirectory ? 'directory' : 'file',
    raw: null,
    path,
    done: false,
  })
}

function removePath(id: string) {
  const idx = selectedList.value.findIndex(item => item.id === id)
  if (idx !== -1)
    selectedList.value.splice(idx, 1)
}

function clearAllPaths() {
  selectedList.value = []
  indexingMsg.value = ''
}

// Setup event listeners with cleanup
onMounted(async () => {
  const unlisten1 = await listen(TauriEvent.DRAG_DROP, async (e: Event<DragPayload>) => {
    if (router.currentRoute.value.name !== 'Indexer')
      return

    isDragOver.value = false
    console.log('Dropped files:', e)
    const paths = e.payload.paths
    for (const path of paths) {
      const pathType = await invoke('check_path_type', { path })
      addPath(path, pathType === 'directory')
    }
  })
  unlistenFns.push(unlisten1)

  const unlisten2 = await listen(TauriEvent.DRAG_LEAVE, (e) => {
    if (router.currentRoute.value.name !== 'Indexer')
      return

    isDragOver.value = false
    console.log('Drag leave', e)
  })
  unlistenFns.push(unlisten2)

  const unlisten3 = await listen(TauriEvent.DRAG_ENTER, (e) => {
    if (router.currentRoute.value.name !== 'Indexer')
      return

    isDragOver.value = true
    console.log('Drag enter', e)
  })
  unlistenFns.push(unlisten3)

  const unlisten4 = await listen<string>('selector-indexing', (eventObj) => {
    const payload = JSON.parse(eventObj.payload) as IndexingEvent
    indexingTitle.value = payload.event.toUpperCase()
    indexingMsg.value = payload.data.msg
    switch (payload.event) {
      case 'start':
        indexerStore.setIndexProcessing(true)
        break
      case 'scan':
        break
      case 'embed':
        break
      case 'finish':
        emit('indexingFinish')
        indexerStore.setIndexProcessing(false)
        selectedList.value.forEach((item) => {
          item.done = true
        })
        break
      case 'stop':
        emit('indexingStop')
        indexerStore.setIndexProcessing(false)
        break
    }
  })
  unlistenFns.push(unlisten4)
})

// Cleanup event listeners on unmount
onUnmounted(() => {
  unlistenFns.forEach(unlisten => unlisten())
})

async function startIndexing() {
  if (selectedList.value.length === 0) {
    message.warning(t('indexer.noFileSelected'))
    return
  }
  const undonePaths = selectedList.value.filter(item => !item.done).map(item => item.path)
  if (undonePaths.length === 0) {
    window.$message.info(t('indexer.allFilesIndexed'))
    return
  }
  btnDisabled.value = true
  setTimeout(() => {
    btnDisabled.value = false
  }, 3000)
  try {
    indexingTitle.value = 'START'
    indexingMsg.value = ''
    const res = await invoke<CommandResult>('start_indexing', {
      paths: undonePaths,
      from: 'selector',
    })
    if (!res.success && res.message) {
      indexingTitle.value = 'ERROR'
      indexingMsg.value = res.message
      if (res.code === 2)
        indexerStore.setIndexProcessing(true)
    }
  } catch (e: any) {
    console.log(e)
    window.$message.error(e)
  }
}

watch(indexingMsg, (newVal) => {
  if (newVal === 'done') {
    setTimeout(() => {
      indexingTitle.value = ''
      indexingMsg.value = ''
    }, 3000)
  }
})

async function stopIndexing() {
  try {
    await invoke('stop_indexing')
  } catch (e) {
    console.log(e)
  } finally {
    indexerStore.setIndexProcessing(false)
  }
}
</script>

<template>
  <div>
    <NCard
      size="small" :bordered="true" content-style="padding: 10px; " class="mb-2"
      :content-class="isDragOver ? 'bg-gray-200 dark:text-white dark:bg-white' : ''"
    >
      <div class="flex flex-col items-center justify-center space-y-2 mb-2">
        <NIcon size="32">
          <FolderOpenOutlined v-if="isDragOver" />
          <FileOpenOutlined v-else />
        </NIcon>
        <div class="flex items-center">
          <span>
            {{ t('common.dragDropTip') }}
          </span>
          <div class="flex space-x-2">
            <span class="mx-2">{{ t('common.or') }}</span>
            <div class="mr-2">
              <NButton text type="primary" class="text-link" @click="openDirDialog">
                {{ t('common.selectFolder')
                }}
              </NButton>
            </div>
            <NButton text type="primary" class="text-link" @click="openFileDialog">
              {{ t('common.selectFile')
              }}
            </NButton>
          </div>
        </div>
      </div>
    </NCard>

    <NList bordered>
      <template #header>
        <div class="flex justify-between items-center mb-2">
          <div class="font-semibold">
            {{ t('common.selectedFileAndFolder') }}
          </div>
          <div>
            <NButton size="tiny" ghost @click="clearAllPaths">
              {{ t('indexer.clearSelected') }}
            </NButton>
          </div>
        </div>
      </template>
      <template v-for="item in selectedList" :key="item.id">
        <NListItem>
          <div class="flex items-center px-2 py-1">
            <div class="flex-1 flex">
              <div
                class="mr-2 w-5 items-center flex"
                :class="item.done ? 'text-green-500' : 'text-gray-300 dark:text-gray-800'"
              >
                <NIcon :size="20">
                  <DoneOutlineRound />
                </NIcon>
              </div>
              <div
                class="flex items-center gap-2"
                :class="item.done ? 'text-green-500' : 'text-gray-800 dark:text-gray-300'"
              >
                <NIcon :size="20">
                  <FolderOutlined v-if="item.type === 'directory'" />
                  <AttachFileOutlined v-else />
                </NIcon>
                <span class="truncate" :title="item.name">{{ item.name }}</span>
              </div>
            </div>
            <NButton
              v-if="!indexerStore.indexProcessing" quaternary type="error" size="small"
              @click="removePath(item.id)"
            >
              <template #icon>
                <DeleteOutlined />
              </template>
            </NButton>
          </div>
        </NListItem>
      </template>
      <template #footer>
        <div class="text-xs text-gray-400 pl-2">
          {{ t('common.total') }}: {{ selectedList.length }}
        </div>
      </template>
    </NList>

    <div class="flex mt-2">
      <NButton
        v-if="!indexerStore.indexProcessing" type="primary" ghost style="margin-right: 6px"
        :disabled="selectedList.length === 0 || indexerStore.indexProcessing" :loading="indexerStore.indexProcessing"
        @click="startIndexing"
      >
        {{ t('indexer.startIndexing') }}
      </NButton>
      <NPopconfirm
        v-if="indexerStore.indexProcessing" :positive-text="t('common.confirm')"
        :negative-text="t('common.cancel')" @positive-click="stopIndexing"
      >
        <template #trigger>
          <NButton ghost type="error">
            <template #icon>
              <NIcon>
                <StopCircleOutlined />
              </NIcon>
            </template>
            {{ t('indexer.stopIndexing') }}
          </NButton>
        </template>
        {{ t('indexer.stopIndexingConfirm') }}
      </NPopconfirm>
    </div>

    <NAlert v-if="indexingMsg" type="info" class="mt-4" :title="indexingTitle" closable @close="indexingMsg = ''">
      {{ indexingMsg }}
    </NAlert>
  </div>
</template>

<style scoped></style>
