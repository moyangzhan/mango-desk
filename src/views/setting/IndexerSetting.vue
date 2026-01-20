<script setup lang="ts">
import { Channel, invoke } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import { join } from '@tauri-apps/api/path'
import type { DataTableColumns, ProgressStatus } from 'naive-ui'
import ModelPlatformEdit from './ModelPlatformEdit.vue'
import { useIndexerStore } from '@/stores/indexer'
import { t } from '@/locales'
import { emptyDownloadState } from '@/utils/functions'
type DownloadEvent =
  | {
    event: 'start'
    data: {
      downloadId: string
      url: string
    }
  }
  | {
    event: 'progress'
    data: {
      downloadId: string
      progress: number
    }
  }
  | {
    event: 'finish'
    data: {
      downloadId: string
    }
  }
  | {
    event: 'error'
    data: {
      downloadId: string
      error: string
    }
  }

interface RowData {
  name: string
  key: string
  status: string
  children?: RowData[]
  render?: (row: RowData) => VNode
}

const indexerStore = useIndexerStore()
const activePlatform = ref('openai')
const activeTab = ref('openai')
const modelPlatformList = ref<ModelPlatform[]>([])
const embeddingModels = ref<Map<string, boolean>>(new Map<string, boolean>())
const embeddingModelChanged = ref(false)
const dataRef = ref<RowData[]>([])

const modelPath = ref('')
const downloading = ref(false)
const downloadProxy = ref(false)
const downloadFailed = ref(false)
const modelDownloadState = ref<DownloadState>(emptyDownloadState())
const tokenizerDownloadState = ref<DownloadState>(emptyDownloadState())
initStatusData()

async function initStatusData() {
  let imageParserDesc = t('indexer.disabledByPrivateMode')
  let audioParserDesc = t('indexer.disabledByPrivateMode')
  const contentLaugnage
    = indexerStore.indexerSetting.file_content_language === 'en'
      ? t('common.english')
      : t('common.multilingual')
  if (!indexerStore.indexerSetting.is_private) {
    const platformInfo = modelPlatformList.value.find(
      p => p.name === activePlatform.value,
    )
    if (platformInfo) {
      const aiModel = await invoke<AiModel>('load_model_by_type', {
        platform: activePlatform.value,
        oneType: 'vision',
      })
      if (aiModel && aiModel.name)
        imageParserDesc = `${platformInfo.title} => ${aiModel.title}`
      else imageParserDesc = t('common.disable')

      const audioModel = await invoke<AiModel>('load_model_by_type', {
        platform: activePlatform.value,
        oneType: 'asr',
      })
      if (audioModel && audioModel.name)
        audioParserDesc = `${platformInfo.title} => ${audioModel.title}`
      else audioParserDesc = t('common.disable')
    }
  }
  dataRef.value = [
    {
      name: t('indexer.indexFileMetadata'),
      status: t('common.enable'),
      key: 'index-metadata',
    },
    {
      name: t('indexer.indexFileContent'),
      status: t('common.enable'),
      key: 'index-content',
      children: [
        {
          name: t('indexer.documentParser'),
          status: `${t('common.enable')} (${t('indexer.localParser')})`,
          key: 'index-document-parser',
        },
        {
          name: t('indexer.imageParser'),
          status: imageParserDesc,
          key: 'index-image-parser',
        },
        {
          name: t('indexer.audioParser'),
          status: audioParserDesc,
          key: 'index-audio-parser',
        },
      ],
    },
    {
      name: t('indexer.ignoreFilesAndFolders'),
      status: '',
      key: 'ignore-files-and-folders',
      children: [
        {
          name: t('indexer.ignoreFileExtensions'),
          status: indexerStore.indexerSetting.ignore_exts.join(', '),
          key: 'ignore-file-extensions',
        },
        {
          name: t('indexer.ignoreFiles'),
          status:
            indexerStore.indexerSetting.ignore_files.length === 0
              ? t('common.none')
              : indexerStore.indexerSetting.ignore_files.join(', '),
          key: 'ignore-files',
        },
        {
          name: t('indexer.ignoreHiddenFiles'),
          status: t('common.yes'),
          key: 'ignore-hidden-files',
        },
        {
          name: t('indexer.ignoreFolders'),
          status: indexerStore.indexerSetting.ignore_dirs.join(', '),
          key: 'ignore-folders',
        },
        {
          name: t('indexer.ignoreHiddenFolders'),
          status: t('common.yes'),
          key: 'ignore-hidden-folders',
        },
      ],
    },
    {
      name: t('indexer.fileContentLanguage'),
      status: contentLaugnage,
      key: 'file-content-language',
    },
    {
      name: t('indexer.embeddingModel'),
      status: '',
      key: 'embedding-model',
    },
  ]
}

const columns: DataTableColumns<RowData> = [
  {
    title: t('common.setting'),
    key: 'name',
    width: '220px',
    render(row) {
      const childCount = row.children?.length || 0
      if (childCount > 0) {
        return h(
          'span',
          {
            class: '',
            onClick: (e: PointerEvent) => {
              const target = e.target as HTMLElement
              if (target.parentElement) {
                const trigger = target.parentElement.querySelector(
                  '.n-data-table-expand-trigger',
                )
                if (trigger)
                  (trigger as HTMLElement).click()
                else console.log('trigger not found')
              } else {
                console.log('no target')
              }
            },
          },
          {
            default: () => row.name,
          },
        )
      } else {
        return row.name
      }
    },
  },
  {
    title: t('common.status'),
    key: 'status',
    render(row) {
      if (row.name === t('indexer.embeddingModel')) {
        return h(
          'div',
          { class: 'flex flex-col' },
          {
            default: () => [
              h(
                'div',
                {},
                { default: () => `${t('common.english')}: all-minilm-l6-v2` },
              ),
              h(
                'div',
                {},
                {
                  default: () =>
                    `${t(
                      'common.multilingual',
                    )}: paraphrase-multilingual-MiniLM-L12-v2`,
                },
              ),
            ],
          },
        )
      } else {
        return row.status
      }
    },
  },
]

async function doActivePlatformChanged(selectedName: string) {
  activePlatform.value = selectedName
  try {
    const res = await invoke('set_active_platform', {
      platformName: selectedName,
    })
    console.log('set_active_platform result', res)
  } catch (error) {
    console.error('set active tab error', error)
  }
  initStatusData()
}

async function doParsedContentChange1(value: boolean) {
  indexerStore.setDocumentParsedContent(value)
  updateIndexerSetting()
}

async function doParsedContentChange2(value: boolean) {
  indexerStore.setImageParsedContent(value)
  updateIndexerSetting()
}

async function doParsedContentChange3(value: boolean) {
  indexerStore.setAudioParsedContent(value)
  updateIndexerSetting()
}

async function doPrivateModeChanged(enabled: boolean) {
  indexerStore.indexerSetting.is_private = enabled
  await updateIndexerSetting()
}

async function doContentLauguageChanged(lang: string) {
  indexerStore.indexerSetting.file_content_language = lang
  await updateIndexerSetting()
  embeddingModelChanged.value = await invoke('is_embedding_model_changed')
  if (embeddingModelChanged.value)
    window.$message.warning(t('indexer.embeddingModelChanged'))
}

async function download() {
  console.log('download')
  if (downloading.value) {
    window.$message.warning(t('indexer.downloading'))
    return
  }
  downloading.value = true
  try {
    const onEvent = new Channel<DownloadEvent>()
    onEvent.onmessage = (message) => {
      console.log(`got download event ${message.event}`)
      switch (message.event) {
        case 'start': {
          downloading.value = true
          if (message.data.url.includes('model.onnx')) {
            modelDownloadState.value.url = message.data.url
            modelDownloadState.value.downloadId = message.data.downloadId
          } else {
            tokenizerDownloadState.value.url = message.data.url
            tokenizerDownloadState.value.downloadId = message.data.downloadId
          }
          break
        }
        case 'progress':
          getDownloadState(message.data.downloadId).progress
            = message.data.progress
          break
        case 'finish': {
          getDownloadState(message.data.downloadId).progress = 100
          if (
            modelDownloadState.value.progress === 100
            && tokenizerDownloadState.value.progress === 100
          ) {
            downloading.value = false
            loadEmbeddingModels()
          }
          break
        }
        case 'error': {
          console.log('receive error event', message.data.error)
          const downloadState = getDownloadState(message.data.downloadId)
          downloadState.status = 'error'
          window.$message.error(message.data.error)
          downloading.value = false
          break
        }
      }
    }
    await invoke('download_multilingual_model', {
      proxy: downloadProxy.value,
      onEvent,
    })
  } catch (error) {
    console.error('download error', error)
    window.$message.error(t('message.downloadFailed'))
  }
}

function getDownloadState(downloadId: string) {
  if (downloadId === modelDownloadState.value.downloadId)
    return modelDownloadState.value
  else return tokenizerDownloadState.value
}

async function updateIndexerSetting() {
  try {
    const res = await invoke('update_indexer_setting', {
      indexerSetting: indexerStore.indexerSetting,
    })
    console.log('update_indexer_setting result', res)
  } catch (error) {
    console.error('update indexer setting error', error)
  }
  initStatusData()
}

async function loadEmbeddingModels() {
  const models = await invoke<Map<string, boolean>>('load_embedding_models')
  embeddingModels.value = new Map(Object.entries(models))
}

function closeEmbeddingChangedTip() {
  embeddingModelChanged.value = false
}

function openUrl(path = '') {
  openPath(path).then((res) => {
    console.log('openUrl', res)
  })
}

function onModelPlatformSaved(updatedPlatform: ModelPlatform) {
  const index = modelPlatformList.value.findIndex(
    (p) => p.name === updatedPlatform.name,
  )
  if (index !== -1) {
    modelPlatformList.value[index] = { ...updatedPlatform }
  }
}

onMounted(async () => {
  console.log('IndexerSetting onMounted')
  try {
    if (!indexerStore.indexerSetting) {
      indexerStore.indexerSetting = await invoke<IndexerSetting>('load_indexer_setting')
      let indexerSetting = await invoke<IndexerSetting>('load_indexer_setting')
      indexerStore.setIndexerSetting(indexerSetting)
    }
    activePlatform.value = await invoke<string>('load_active_platform')
    modelPlatformList.value = await invoke<ModelPlatform[]>(
      'load_model_platforms',
    )
    embeddingModelChanged.value = await invoke<boolean>(
      'is_embedding_model_changed',
    )
    const homePath = await invoke<string>('get_app_dir')
    console.log('homePath', homePath)
    modelPath.value = await join(homePath, 'assets', 'model')
    console.log('modelPath', modelPath.value)
    await loadEmbeddingModels()
    activeTab.value = activePlatform.value
    initStatusData()
  } catch (e) {
    console.error('IndexerSetting onMounted error', e)
  }
})
</script>

<template>
  <div>
    <NCard :title="t('common.overview')" class="mb-4 p-0" size="small" :bordered="true" hoverable>
      <div>
        <NDataTable size="small" :columns="columns" :data="dataRef" :default-expanded-row-keys="['index-content']" />
      </div>
    </NCard>
    <NCard :title="t('indexer.fileContentLanguage')" class="mb-4 px-0" size="small" :bordered="true" hoverable>
      <div class="flex flex-col space-y-1 mb-4">
        <NRadioGroup :value="indexerStore.indexerSetting.file_content_language"
          @update:value="doContentLauguageChanged">
          <NRadio :label="t('common.english')" value="en" />
          <NRadio :label="t('common.multilingual')" value="multilingual" />
        </NRadioGroup>
        <NAlert v-if="!embeddingModelChanged" :show-icon="false">
          {{ t('indexer.fileContentLanguageDesc') }}
        </NAlert>
        <NAlert v-if="embeddingModelChanged" type="warning" closable @close="closeEmbeddingChangedTip">
          {{ t('indexer.embeddingModelChangedTip') }}
        </NAlert>
        <div v-if="
          indexerStore.indexerSetting.file_content_language === 'multilingual'
          && !embeddingModels.get('paraphrase-multilingual-MiniLM-L12-v2')
        " class="flex flex-col space-y-2">
          <div class="flex flex-col space-y-2 justify-start items-start">
            <div v-html="$t('indexer.multilingualEmbeddingModeSettingTip')" />
            <NButton :type="downloadFailed ? 'error' : 'primary'" size="small" ghost tag="a" target="_blank"
              :loading="downloading" @click="download">
              <span v-if="!downloadFailed">
                {{ t('indexer.downloadMultilingualEmbeddingModelBtn') }}
              </span>
              <span v-if="downloadFailed">{{
                t('indexer.retryDownload')
              }}</span>
            </NButton>
          </div>
          <NProgress type="line" :percentage="modelDownloadState.progress" class="px-8"
            :status="modelDownloadState.status as ProgressStatus" indicator-placement="inside">
            {{
              `${t(
                'model.embeddingModel',
              )} ${modelDownloadState.progress.toFixed(2)}%`
            }}
          </NProgress>
          <NProgress type="line" :percentage="tokenizerDownloadState.progress" class="px-8"
            :status="tokenizerDownloadState.status as ProgressStatus" indicator-placement="inside">
            {{
              `${t(
                'model.tokenizer',
              )} ${tokenizerDownloadState.progress.toFixed(2)}%`
            }}
          </NProgress>
          <NAlert v-if="downloadFailed" type="error" :show-icon="false">
            {{ t('message.downloadFailed') }}
          </NAlert>
          <NAlert type="warning" :show-icon="false">
            <div>
              {{ t('message.downloadModelManualTip') }}
            </div>
            <div>1. {{ t('message.downloadLinks') }}</div>
            <div>
              • {{ t('common.model') }}:
              <NText type="primary" class="cursor-pointer" @click="
                openUrl(
                  'https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/onnx/model.onnx',
                )
                ">
                https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/onnx/model.onnx
              </NText>
            </div>
            <div>
              • {{ t('common.tokenizer') }}:
              <NText type="primary" class="cursor-pointer" @click="
                openUrl(
                  'https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/tokenizer.json',
                )
                ">
                https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/tokenizer.json
              </NText>
            </div>
            <div>
              2. {{ t('message.moveModelTip', { modelPath }) }}
            </div>
          </NAlert>
        </div>
      </div>
    </NCard>
    <NCard :title="t('common.privacy')" class="mb-4" size="small" :bordered="true" hoverable>
      <div class="flex flex-col">
        <div class="flex flex-col space-y-1 mb-4">
          <NRadioGroup :value="indexerStore.indexerSetting.is_private" @update:value="doPrivateModeChanged">
            <NRadio :label="t('indexer.privateMode')" :value="true" />
            <NRadio :label="t('indexer.cloudMode')" :value="false" />
          </NRadioGroup>
          <NAlert :show-icon="false">
            <div v-if="indexerStore.indexerSetting.is_private">
              {{ t('indexer.privateModeDesc') }}
            </div>
            <div v-if="!indexerStore.indexerSetting.is_private">
              {{
                t('indexer.cloudModeDescDynamic', {
                  modelPlatform: modelPlatformList.find(
                    (p) => p.name === activePlatform,
                  )?.title,
                })
              }}
            </div>
          </NAlert>
        </div>
        <NCard v-if="!indexerStore.indexerSetting.is_private" :title="t('indexer.cloudModeSetting')" size="small"
          :bordered="true">
          <NFormItem :label="t('model.activePlatform')">
            <NRadioGroup :value="activePlatform" @update:value="doActivePlatformChanged">
              <NRadio v-for="platform in modelPlatformList" :key="platform.id" :label="platform.title"
                :value="platform.name" />
            </NRadioGroup>
          </NFormItem>
          <NFormItem :label="t('model.platformInfo')">
            <NTabs v-model:value="activeTab" type="line" animated placement="left">
              <NTabPane v-for="platform in modelPlatformList" :key="platform.name" :name="platform.name"
                :tab="platform.title">
                <ModelPlatformEdit :model-platform="platform" @saved="onModelPlatformSaved" />
              </NTabPane>
            </NTabs>
          </NFormItem>
        </NCard>
      </div>
    </NCard>
    <NCard :title="t('common.storage')" class="mb-4 px-0" size="small" :bordered="true" hoverable>
      <div class="flex flex-col">
        <NAlert :show-icon="false">
          <div>
            {{ t('indexer.saveParsedContentTip') }}
          </div>
          <div>
            {{ t('indexer.saveParsedContentWarn') }}
          </div>
        </NAlert>
        <div class="flex flex-col space-y-2 my-4">
          <div>
            <div>{{ t('indexer.saveDocumentParsedContent') }}</div>
            <n-switch size="small" :value="indexerStore.indexerSetting.save_parsed_content.document"
              @update:value="doParsedContentChange1"></n-switch>
          </div>
          <div>
            <div>{{ t('indexer.saveImageParsedContent') }}</div>
            <n-switch size="small" :value="indexerStore.indexerSetting.save_parsed_content.image"
              @update:value="doParsedContentChange2"></n-switch>
          </div>
          <div>
            <div>{{ t('indexer.saveAudioParsedContent') }}</div>
            <n-switch size="small" :value="indexerStore.indexerSetting.save_parsed_content.audio"
              @update:value="doParsedContentChange3"></n-switch>
          </div>
        </div>
      </div>
    </NCard>
  </div>
</template>
