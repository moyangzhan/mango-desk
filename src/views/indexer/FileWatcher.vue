<script lang="ts" setup>
import { AttachFileOutlined, DeleteOutlined, FolderOutlined } from '@vicons/material'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { emptyWatchSetting } from '@/utils/functions'
import { useIndexerStore } from '@/stores/indexer'
import { t } from '@/locales'

const emit = defineEmits<Emit>()
interface Emit {
  (ev: 'indexingFinish'): void
  (ev: 'indexingStop'): void
}

const indexerStore = useIndexerStore()
const watchSetting = ref<WatchSetting>(emptyWatchSetting())
const message = useMessage()
const indexingTitle = ref('')
const indexingMsg = ref('')

// Store unlisten functions for cleanup
const unlistenFns: UnlistenFn[] = []

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
  console.log('add path', path, isDirectory)
  const idStr = path
  if (isDirectory) {
    if (watchSetting.value.directories.find(item => item === idStr)) {
      message.warning(`${t('common.alreadyExists')}: ${path}`)
      return
    }
    watchSetting.value.directories.push(path)
  } else {
    if (watchSetting.value.files.find(item => item === idStr)) {
      message.warning(`${t('common.alreadyExists')}: ${path}`)
      return
    }
    watchSetting.value.files.push(path)
  }
  invoke('add_watch_path', { path }).then((resp) => {
    console.log('add path', resp)
  })
}

async function removePath(path: string) {
  const idx = watchSetting.value.directories.findIndex(item => item === path)
  if (idx !== -1)
    watchSetting.value.directories.splice(idx, 1)
  const idx2 = watchSetting.value.files.findIndex(item => item === path)
  if (idx2 !== -1)
    watchSetting.value.files.splice(idx2, 1)
  const resp = await invoke('remove_watch_path', { path })
  console.log('remove path', resp)
}

// Setup event listeners with cleanup
onMounted(async () => {
  // Load config
  invoke('load_config_value', { configName: 'fs_watcher_setting' }).then((resp) => {
    try {
      const str = resp as string
      const parsed = JSON.parse(str)
      watchSetting.value = parsed as WatchSetting
      console.log('load file watcher setting', watchSetting.value)
    } catch (e) {
      console.error('Failed to parse watcher setting:', e)
    }
  })

  // Setup event listener
  const unlisten = await listen<string>('watcher-indexing', (eventObj) => {
    const payload = JSON.parse(eventObj.payload) as IndexingEvent
    console.log('watcher-indexing', payload)
    indexingTitle.value = payload.event.toUpperCase()
    indexingMsg.value = payload.data.msg
    switch (payload.event) {
      case 'start':
        indexerStore.setWatcherProcessing(true)
        break
      case 'scan':
        break
      case 'embed':
        break
      case 'finish':
        emit('indexingFinish')
        indexerStore.setWatcherProcessing(false)
        break
      case 'stop':
        emit('indexingStop')
        indexerStore.setWatcherProcessing(false)
        break
    }
  })
  unlistenFns.push(unlisten)
})

// Cleanup event listeners on unmount
onUnmounted(() => {
  unlistenFns.forEach(unlisten => unlisten())
})

async function reindexing() {
  if (watchSetting.value.directories.length === 0 && watchSetting.value.files.length === 0) {
    message.warning(t('indexer.noFileSelected'))
    return
  }
  const paths = watchSetting.value.directories.concat(watchSetting.value.files)
  if (paths.length === 0)
    return

  try {
    indexingTitle.value = 'START'
    indexingMsg.value = ''
    const res = await invoke<CommandResult>('start_indexing', {
      paths,
      from: 'watcher',
    })
    if (!res.success && res.message) {
      indexingTitle.value = 'ERROR'
      indexingMsg.value = res.message
      if (res.code === 2)
        indexerStore.setWatcherProcessing(false)
    }
  } catch (e: any) {
    console.log(e)
    window.$message.error(e)
  }
}
</script>

<template>
  <div>
    <NCard :title="t('indexer.fileWatch')" class="mb-2" :subtitle="t('indexer.autoIndexWhenChanged')">
      <template #header-extra>
        <NText depth="3" class="text-xs">
          {{ t('indexer.autoIndexWhenChanged') }}
        </NText>
      </template>
      <div />
      <div
        v-if="watchSetting.directories.length !== 0 || watchSetting.files.length !== 0"
        class="mb-2 flex justify-between"
      >
        <div class="flex-1">
          <NButton ghost size="small" @click="openDirDialog">
            {{ t('common.selectFolder')
            }}
          </NButton>
          <NButton ghost size="small" style="margin-left: 8px" @click="openFileDialog">
            {{ t('common.selectFile')
            }}
          </NButton>
        </div>
        <div>
          <NButton
            ghost size="small"
            style="margin-right: 6px"
            :disabled="(watchSetting.directories.length === 0 && watchSetting.files.length === 0) || indexerStore.watcherProcessing"
            @click="reindexing"
          >
            {{ t('indexer.reindexing') }}
          </NButton>
        </div>
      </div>
      <div v-if="watchSetting.directories.length === 0 && watchSetting.files.length === 0" class="flex items-center">
        <span>
          {{ t('indexer.selectedFolderAndFileToWatch') }}
        </span>
        <div class="flex space-x-2">
          <div class="mx-2">
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
      <NList v-else bordered>
        <template v-for="item in watchSetting.directories" :key="item">
          <NListItem>
            <div class="flex items-center justify-between px-2 py-1">
              <div class="flex items-center gap-2">
                <NIcon :size="20">
                  <FolderOutlined />
                </NIcon>
                <span class="truncate max-w-xs" :title="item">{{ item }}</span>
              </div>
              <NButton ghost icon-placement="right" size="tiny" @click="removePath(item)">
                <template #icon>
                  <DeleteOutlined />
                </template>
                {{ t('indexer.unwatch') }}
              </NButton>
            </div>
          </NListItem>
        </template>
        <template v-for="item in watchSetting.files" :key="item">
          <NListItem>
            <div class="flex items-center justify-between px-2 py-1">
              <div class="flex items-center gap-2">
                <NIcon :size="20">
                  <AttachFileOutlined />
                </NIcon>
                <span class="truncate max-w-xs" :title="item">{{ item }}</span>
              </div>
              <NButton ghost icon-placement="right" size="tiny" @click="removePath(item)">
                <template #icon>
                  <DeleteOutlined />
                </template>
                {{ t('indexer.unwatch') }}
              </NButton>
            </div>
          </NListItem>
        </template>
      </NList>
      <NAlert v-if="indexingMsg" type="info" class="mt-4" :title="indexingTitle" closable @close="indexingMsg = ''">
        {{ indexingMsg }}
      </NAlert>
    </NCard>
  </div>
</template>

<style scoped></style>
