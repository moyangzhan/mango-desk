<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { join } from '@tauri-apps/api/path'
import type { DataTableColumns } from 'naive-ui'
import ModelPlatformEdit from './ModelPlatformEdit.vue'
import { useIndexerStore } from '@/stores/indexer'
import { useAppStore } from '@/stores/app'
import { t } from '@/locales'

interface RowData {
  name: string
  key: string
  status: string
  children?: RowData[]
  render?: (row: RowData) => VNode
}

const indexerStore = useIndexerStore()
const appStore = useAppStore()
const activePlatform = ref('openai')
const activeTab = ref('openai')
const modelPlatformList = ref<ModelPlatform[]>([])
const dataRef = ref<RowData[]>([])
const imageParserDesc = ref('')
const audioParserDesc = ref('')

const modelPath = ref('')
const dbPath = ref('')
initStatusData()

async function initStatusData() {
  imageParserDesc.value = t('common.local')
  audioParserDesc.value = t('indexer.notSupportByPrivateMode')
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
        imageParserDesc.value = `${platformInfo.title} => ${aiModel.title}`

      const audioModel = await invoke<AiModel>('load_model_by_type', {
        platform: activePlatform.value,
        oneType: 'asr',
      })
      if (audioModel && audioModel.name)
        audioParserDesc.value = `${platformInfo.title} => ${audioModel.title}`
      else audioParserDesc.value = t('common.disable')
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
          status: imageParserDesc.value,
          key: 'index-image-parser',
        },
        {
          name: t('indexer.audioParser'),
          status: audioParserDesc.value,
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
          status: indexerStore.indexerSetting.ignore_exts.length === 0 ? t('common.none') : indexerStore.indexerSetting.ignore_exts.join(', '),
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
          status: indexerStore.indexerSetting.ignore_dirs.length === 0 ? t('common.none') : indexerStore.indexerSetting.ignore_dirs.join(', '),
          key: 'ignore-folders',
        },
        {
          name: t('indexer.ignoreHiddenFolders'),
          status: t('common.yes'),
          key: 'ignore-hidden-folders',
        },
      ],
    },
  ]
}

const columns = computed<DataTableColumns<RowData>>(() => [
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
  },
]);

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

function onModelPlatformSaved(updatedPlatform: ModelPlatform) {
  const index = modelPlatformList.value.findIndex(
    (p) => p.name === updatedPlatform.name,
  )
  if (index !== -1) {
    modelPlatformList.value[index] = { ...updatedPlatform }
  }
}

watch(() => appStore.locale, (newVal) => {
  if (newVal) {
    initStatusData()
  }
})

onMounted(async () => {
  console.log('IndexerSetting onMounted')
  try {
    indexerStore.indexerSetting = await invoke<IndexerSetting>('load_indexer_setting')
    let indexerSetting = await invoke<IndexerSetting>('load_indexer_setting')
    indexerStore.setIndexerSetting(indexerSetting)

    activePlatform.value = await invoke<string>('load_active_platform')
    modelPlatformList.value = await invoke<ModelPlatform[]>(
      'load_model_platforms',
    )
    const userDataPath = await invoke<string>('get_data_path')
    console.log('userDataPath', userDataPath)
    modelPath.value = await join(userDataPath, 'model')
    console.log('modelPath', modelPath.value)
    dbPath.value = await join(userDataPath, 'storage')
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
    <NCard :title="t('common.privacy')" class="mb-4" size="small" :bordered="true" hoverable>
      <div class="flex flex-col">
        <div class="flex flex-col space-y-1 mb-2">
          <NRadioGroup :value="indexerStore.indexerSetting.is_private" @update:value="doPrivateModeChanged">
            <NRadio :label="t('indexer.privateMode')" :value="true" />
            <NRadio :label="t('indexer.cloudMode')" :value="false" />
          </NRadioGroup>
          <NAlert :show-icon="false" class="text-xs" size="small" >
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
        <NTable size="small" class="mb-2">
          <tbody class="text-xs">
            <tr>
              <td>{{ t('indexer.documentParser') }}</td>
              <td>{{ t('common.local') }}</td>
            </tr>
            <tr>
              <td>{{ t('indexer.imageParser') }}</td>
              <td>{{ imageParserDesc }}</td>
            </tr>
            <tr>
              <td>{{ t('indexer.audioParser') }}</td>
              <td>{{ audioParserDesc }}</td>
            </tr>
          </tbody>
        </NTable>
        <NCard v-if="!indexerStore.indexerSetting.is_private" :title="t('indexer.cloudModeSetting')" size="small"
          :bordered="true">
          <NFormItem :label="t('model.selectForActivePlatform')">
            <NRadioGroup :value="activePlatform" @update:value="doActivePlatformChanged">
              <NRadio v-for="platform in modelPlatformList" :key="platform.id" :label="platform.title"
                :value="platform.name" />
            </NRadioGroup>
          </NFormItem>
          <NFormItem :label="t('indexer.detailConfig')">
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
