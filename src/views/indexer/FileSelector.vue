<script lang="ts" setup>
import { AttachFileOutlined, DeleteOutlined, FileOpenOutlined, FolderOpenOutlined, FolderOutlined, StopCircleOutlined } from '@vicons/material'
import { open } from '@tauri-apps/plugin-dialog'
import { TauriEvent, listen } from '@tauri-apps/api/event'
import { Channel, invoke } from '@tauri-apps/api/core'
import type { Event } from '@tauri-apps/api/event'
import router from '@/router'
import { t } from '@/locales'

const emit = defineEmits<Emit>()
interface Emit {
  (ev: 'indexingFinish'): void
}
const isDragOver = ref(false)
const selectedList = ref<SelectedItem[]>([])
const message = useMessage()
const indexProcessing = ref(false)
const btnDisabled = ref(false)
const indexingTitle = ref('')
const indexingMsg = ref('')

//  Start { task_id: i64 },
//  Scan { task_id: i64, msg: String },
//  Stop { task_id: i64, msg: String },
//  Embed { task_id: i64, msg: String },
//  Finish { task_id: i64, msg: String },

interface IndexingEvent {
  event: string
  data: {
    taskId: number
    msg: string
  }
}

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
  })
}

function removePath(id: string) {
  const idx = selectedList.value.findIndex(item => item.id === id)
  if (idx !== -1)
    selectedList.value.splice(idx, 1)
}

listen(TauriEvent.DRAG_DROP, async (e: Event<DragPayload>) => {
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
listen(TauriEvent.DRAG_LEAVE, (e) => {
  if (router.currentRoute.value.name !== 'Indexer')
    return

  isDragOver.value = false
  console.log('Drag leave', e)
})
listen(TauriEvent.DRAG_ENTER, (e) => {
  if (router.currentRoute.value.name !== 'Indexer')
    return

  isDragOver.value = true
  console.log('Drag enter', e)
})

async function startIndexing() {
  if (selectedList.value.length === 0) {
    message.warning(t('indexer.noFileSelected'))
    return
  }
  btnDisabled.value = true
  setTimeout(() => {
    btnDisabled.value = false
  }, 3000)
  try {
    const onEvent = new Channel<IndexingEvent>()
    onEvent.onmessage = (eventObj) => {
      console.log(`got indexing event ${eventObj.event}`)
      indexingTitle.value = eventObj.event.toUpperCase()
      indexingMsg.value = eventObj.data.msg
      switch (eventObj.event) {
        case 'start':
          indexProcessing.value = true
          break
        case 'scan':
          break
        case 'embed':
          break
        case 'finish':
        case 'stop':
          indexProcessing.value = false
          setTimeout(() => {
            emit('indexingFinish')
            selectedList.value = []
          }, 1000)
          break
      }
    }
    const result = await invoke('start_indexing', {
      paths: selectedList.value.map(item => item.path),
      onEvent,
    })
    const res = result as CommandResult
    if (!res.success && res.message) {
      indexingTitle.value = 'ERROR'
      indexingMsg.value = res.message
      if (res.code === 2)
        indexProcessing.value = true
    }
  } catch (e: any) {
    console.log(e)
    window.$message.error(e)
  }
}

async function stopIndexing() {
  try {
    await invoke('stop_indexing')
  } catch (e) {
    console.log(e)
  } finally {
    indexProcessing.value = false
  }
}
</script>

<template>
  <div>
    <NCard :title="t('indexer.indexer')" class="mb-2">
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
                <NButton text @click="openDirDialog">
                  <NText type="success" underline>
                    {{ t('common.selectFolder')
                    }}
                  </NText>
                </NButton>
              </div>
              <NButton text @click="openFileDialog">
                <NText type="success" underline>
                  {{ t('common.selectFile')
                  }}
                </NText>
              </NButton>
            </div>
          </div>
        </div>
      </NCard>

      <NList bordered>
        <template #header>
          <div class="font-semibold">
            {{ t('common.selctedFileAndFolder') }}
          </div>
        </template>
        <template v-for="item in selectedList" :key="item.id">
          <NListItem>
            <div class="flex items-center justify-between px-2 py-1">
              <div class="flex items-center gap-2">
                <NIcon :size="20">
                  <FolderOutlined v-if="item.type === 'directory'" />
                  <AttachFileOutlined v-else />
                </NIcon>
                <span class="truncate max-w-xs" :title="item.name">{{ item.name }}</span>
              </div>
              <NButton quaternary type="error" size="small" @click="removePath(item.id)">
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
          v-if="!indexProcessing" type="primary" style="margin-right: 6px"
          :disabled="selectedList.length === 0 || indexProcessing" :loading="indexProcessing" @click="startIndexing"
        >
          {{ t('indexer.startIndexing') }}
        </NButton>
        <NPopconfirm
          v-if="indexProcessing" :positive-text="t('common.confirm')" :negative-text="t('common.cancel')"
          @positive-click="stopIndexing"
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

      <NAlert v-if="indexingMsg" type="info" class="mt-4" :title="indexingTitle">
        {{ indexingMsg }}
      </NAlert>
    </NCard>
  </div>
</template>

<style scoped></style>
