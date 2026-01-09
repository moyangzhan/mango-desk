<script lang="ts" setup>
import { AttachFileOutlined, DeleteOutlined, FolderOutlined } from '@vicons/material'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { emptyWatchSetting } from '@/utils/functions'
import { t } from '@/locales'

const watchSetting = ref<WatchSetting>(emptyWatchSetting())
const message = useMessage()

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

onMounted(async () => {
  invoke('load_config_value', { configName: 'fs_watcher_setting' }).then((resp) => {
    const str = resp as string
    const parsed = JSON.parse(str)
    watchSetting.value = parsed as WatchSetting
    console.log('load file watcher setting', watchSetting.value)
  })
})
</script>

<template>
  <div>
    <NCard :title="t('indexer.fileWatch')" class="mb-2" :subtitle="t('indexer.autoIndexWhenChanged')">
      <template #header-extra>
        <NText depth="3" class="text-xs">
          {{ t('indexer.autoIndexWhenChanged') }}
        </NText>
      </template>
      <div v-if="watchSetting.directories.length !== 0 || watchSetting.files.length !== 0" class="mb-2">
        <NButton ghost @click="openDirDialog">
          {{ t('common.selectFolder')
          }}
        </NButton>
        <NButton ghost style="margin-left: 8px" @click="openFileDialog">
          {{ t('common.selectFile')
          }}
        </NButton>
      </div>
      <div v-if="watchSetting.directories.length === 0 && watchSetting.files.length === 0" class="flex items-center">
        <span>
          {{ t('indexer.selectedFolderAndFileToWatch') }}
        </span>
        <div class="flex space-x-2">
          <div class="mx-2">
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
              <NButton quaternary icon-placement="right" size="tiny" @click="removePath(item)">
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
              <NButton quaternary icon-placement="right" size="tiny" @click="removePath(item)">
                <template #icon>
                  <DeleteOutlined />
                </template>
                {{ t('indexer.unwatch') }}
              </NButton>
            </div>
          </NListItem>
        </template>
      </NList>
    </NCard>
  </div>
</template>

<style scoped></style>
