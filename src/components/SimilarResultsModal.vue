<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { NImage, NModal, NSpin, NTag, NTooltip } from 'naive-ui'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import SvgIcon from './SvgIcon.vue'

const props = defineProps<{
  fileId: number | null
}>()

const emit = defineEmits<{
  'open-file': [path: string]
}>()

const { t } = useI18n()

const showModal = ref(false)
const loading = ref(false)
const sourceFile = ref<FileInfo | null>(null)
const results = ref<SearchResult[]>([])

const similarityTypeLabels = computed<Record<string, string>>(() => ({
  imageHash: t('common.similarityType.imageHash'),
  imageSemantic: t('common.similarityType.imageSemantic'),
  documentSemantic: t('common.similarityType.documentSemantic'),
  audioFingerprint: t('common.similarityType.audioFingerprint'),
  audioTranscription: t('common.similarityType.audioTranscription'),
}))

/**
 * Find similar files by file ID
 * 根据文件 ID 查找相似文件
 */
async function findSimilars(fileInfo: FileInfo) {
  showModal.value = true
  loading.value = true
  sourceFile.value = fileInfo
  results.value = []

  try {
    const searchResults = await invoke<SearchResult[]>('find_similars_by_file_id', {
      fileId: fileInfo.id,
      limit: 20,
    })

    // Load image data for image files (category === 2)
    const imagePromises = searchResults
      .filter(item => item.file_info.category === 2)
      .map((item) => {
        return invoke<ArrayBuffer>('read_file_data', { path: item.file_info.path }).then((resp) => {
          if (resp) {
            const mimeType = item.file_info.file_ext.toLowerCase() === 'png' ? 'image/png' : 'image/jpeg'
            const uint8Array = new Uint8Array(resp)
            const blob = new Blob([uint8Array], { type: mimeType })
            item.file_info.file_data = URL.createObjectURL(blob)
          }
        }).catch((e) => {
          console.warn('Failed to load image:', item.file_info.path, e)
        })
      })

    await Promise.all(imagePromises)
    results.value = searchResults
  }
  catch (e) {
    console.error('Failed to find similar files:', e)
  }
  finally {
    loading.value = false
  }
}

/**
 * Open file in system
 * 在系统中打开文件
 */
function openFile(path: string) {
  emit('open-file', path)
}

/**
 * Close modal
 * 关闭弹窗
 */
function close() {
  showModal.value = false
}

// Expose findSimilars for parent component
defineExpose({
  findSimilars,
})
</script>

<template>
  <NModal v-model:show="showModal" preset="card" :title="t('common.similarFiles')"
    style="width: 80%; height: 80%;">
    <!-- Source file info -->
    <div v-if="sourceFile" class="mb-4 p-2 bg-gray-50 dark:bg-gray-800 rounded">
      <div class="text-sm text-gray-500">{{ t('common.sourceFile') }}:</div>
      <div class="font-medium">{{ sourceFile.name }}</div>
      <div class="text-xs text-gray-400 truncate">{{ sourceFile.path }}</div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="flex justify-center p-4">
      <NSpin />
    </div>

    <!-- Results list -->
    <div v-else-if="results.length > 0" style="max-height: 500px; overflow-y: auto;">
      <div v-for="item in results" :key="item.file_info.path"
        class="group w-full p-2 border-b border-(--border-color) hover:bg-gray-50 dark:hover:bg-gray-800">
        <div class="flex space-x-2">
          <!-- Image preview -->
          <div v-if="item.file_info.file_data && item.file_info.category === 2"
            class="flex justify-center items-start shrink-0 pt-0.5">
            <NImage width="100" :src="item.file_info.file_data" />
          </div>
          <!-- File icon -->
          <div v-else class="flex justify-center items-center shrink-0">
            <SvgIcon :name="item.file_info.file_ext.toLowerCase()" width="40" height="40"
              style="opacity: 0.7; filter: saturate(0.5)" />
          </div>

          <!-- File info -->
          <div class="flex-1 flex flex-col justify-between text-left min-w-0">
            <div>
              <div class="cursor-pointer hover:underline hover:text-(--primary-color) truncate"
                style="font-weight: 550" @click="openFile(item.file_info.path)">
                {{ item.file_info.name }}
              </div>
              <div class="text-xs text-gray-400 truncate">
                {{ item.file_info.path }}
              </div>
            </div>
            <!-- Similarity type and score -->
            <div class="flex justify-end items-center text-xs text-gray-400 gap-2">
              <NTooltip v-if="item.similarity_type">
                <template #trigger>
                  <NTag size="tiny" :bordered="false" type="info">
                    {{ similarityTypeLabels[item.similarity_type] || item.similarity_type }}
                  </NTag>
                </template>
                {{ t('common.similarityTypeTip') }}
              </NTooltip>
              <NTag size="tiny" :bordered="false">
                {{ t('common.similarityScore') }}: {{ item.score }}%
              </NTag>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="p-4 text-gray-400 text-center">
      {{ t('common.noSimilarFiles') }}
    </div>
  </NModal>
</template>
