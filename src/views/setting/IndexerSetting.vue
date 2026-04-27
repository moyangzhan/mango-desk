<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { join } from '@tauri-apps/api/path'
import type { DataTableColumns } from 'naive-ui'
import ModelPlatformEdit from './ModelPlatformEdit.vue'
import SelfHostedPlatformEdit from './SelfHostedPlatformEdit.vue'
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

type ParserMode = 'local' | 'selfhosted' | 'remote' | 'mixed'

const indexerStore = useIndexerStore()
const appStore = useAppStore()
const activePlatform = ref('openai')
const modelPlatformList = ref<ModelPlatform[]>([])
const selfHostedPlatformList = ref<SelfHostedPlatform[]>([])
const activeSelfHostedPlatform = ref('')
const dataRef = ref<RowData[]>([])
const imageParserDesc = ref('')
const audioParserDesc = ref('')

const modelPath = ref('')
const dbPath = ref('')

// Parser mode state
const parserMode = ref<ParserMode>('local')

// Self-hosted model state
const selfHostedVisionModel = ref<AiModel | null>(null)
const showModelEditModal = ref(false)
const editingModel = ref<AiModel | null>(null)
const showSelfHostedModal = ref(false)
const showCloudModal = ref(false)

initStatusData()

async function initStatusData() {
  // Detect parser mode based on image and audio parser modes
  const setting = indexerStore.indexerSetting
  const imageMode = setting.image_parser_mode
  const audioMode = setting.audio_parser_mode

  // Determine mode: local | selfhosted | remote | mixed
  if (imageMode === 'local' && audioMode === 'local')
    parserMode.value = 'local'
  else if (imageMode === 'selfhosted' && audioMode === 'local')
    parserMode.value = 'selfhosted'
  else if (imageMode === 'remote' && audioMode === 'remote')
    parserMode.value = 'remote'
  else
    parserMode.value = 'mixed'

  imageParserDesc.value = t('common.local')
  audioParserDesc.value = t('common.local')

  if (parserMode.value === 'remote' || imageMode === 'remote') {
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
    }
  } else if (imageMode === 'selfhosted') {
    const platformInfo = selfHostedPlatformList.value.find(
      p => p.name === activeSelfHostedPlatform.value,
    )
    if (platformInfo) {
      const aiModel = await invoke<AiModel>('load_model_by_type', {
        platform: activeSelfHostedPlatform.value,
        oneType: 'vision',
      })
      selfHostedVisionModel.value = aiModel
      imageParserDesc.value = `${platformInfo.title} => ${aiModel?.title || aiModel?.name}`
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
          status: t('common.local'),
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
          name: t('indexer.ignorePathPrefixes'),
          status: indexerStore.indexerSetting.ignore_path_prefixes?.length === 0 ? t('common.none') : indexerStore.indexerSetting.ignore_path_prefixes?.join(', '),
          key: 'ignore-path-prefixes',
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
])

async function doActivePlatformChanged(selectedName: string) {
  activePlatform.value = selectedName
  try {
    await invoke('set_active_platform', {
      platformName: selectedName,
    })
    window.$message.success(t('common.updateSuccess'))
  } catch (error) {
    console.error('set active tab error', error)
  }
  initStatusData()
}

async function doActiveSelfHostedPlatformChanged(selectedName: string) {
  activeSelfHostedPlatform.value = selectedName
  try {
    await invoke('set_active_self_hosted_platform', {
      platformName: selectedName,
    })
    window.$message.success(t('common.updateSuccess'))
  } catch (error) {
    console.error('set active self-hosted platform error', error)
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

async function doParserModeChanged(mode: ParserMode) {
  // Mixed mode cannot be set by user directly
  if (mode === 'mixed')
    return

  parserMode.value = mode
  indexerStore.indexerSetting.parser_mode = mode

  if (mode === 'local') {
    // 本地模式：image 和 audio 都使用 local
    indexerStore.indexerSetting.image_parser_mode = 'local'
    indexerStore.indexerSetting.audio_parser_mode = 'local'
  } else if (mode === 'selfhosted') {
    // 自托管模式：image 使用 selfhosted，audio 使用 local（自托管平台不支持 ASR）
    indexerStore.indexerSetting.image_parser_mode = 'selfhosted'
    indexerStore.indexerSetting.audio_parser_mode = 'local'
  } else if (mode === 'remote') {
    // 云端模式：image 和 audio 都使用 remote
    indexerStore.indexerSetting.image_parser_mode = 'remote'
    indexerStore.indexerSetting.audio_parser_mode = 'remote'
  }

  updateIndexerSetting()
}

// Handle image parser mode change
function onImageParserModeChanged(mode: string) {
  indexerStore.indexerSetting.image_parser_mode = mode
  updateParserModeFromSettings()
  updateIndexerSetting()
}

// Handle audio parser mode change
function onAudioParserModeChanged(mode: string) {
  indexerStore.indexerSetting.audio_parser_mode = mode
  updateParserModeFromSettings()
  updateIndexerSetting()
}

// Update parserMode based on current settings
function updateParserModeFromSettings() {
  const imageMode = indexerStore.indexerSetting.image_parser_mode
  const audioMode = indexerStore.indexerSetting.audio_parser_mode

  if (imageMode === 'local' && audioMode === 'local') {
    parserMode.value = 'local'
    indexerStore.indexerSetting.parser_mode = 'local'
  } else if (imageMode === 'selfhosted' && audioMode === 'local') {
    parserMode.value = 'selfhosted'
    indexerStore.indexerSetting.parser_mode = 'selfhosted'
  } else if (imageMode === 'remote' && audioMode === 'remote') {
    parserMode.value = 'remote'
    indexerStore.indexerSetting.parser_mode = 'remote'
  } else {
    parserMode.value = 'mixed'
    indexerStore.indexerSetting.parser_mode = 'mixed'
  }
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
    p => p.name === updatedPlatform.name,
  )
  if (index !== -1)
    modelPlatformList.value[index] = { ...updatedPlatform }
}

function onSelfHostedPlatformSaved(updatedPlatform: SelfHostedPlatform) {
  const index = selfHostedPlatformList.value.findIndex(
    p => p.name === updatedPlatform.name,
  )
  if (index !== -1)
    selfHostedPlatformList.value[index] = { ...updatedPlatform }
}

function openModelEditModal() {
  if (selfHostedVisionModel.value) {
    editingModel.value = { ...selfHostedVisionModel.value }
    showModelEditModal.value = true
  }
}

async function saveModel() {
  if (!editingModel.value)
    return
  try {
    let modelName = editingModel.value.name
    if (activeSelfHostedPlatform.value === 'ollama' && modelName && !modelName.includes(':')) {
      modelName = `${modelName}:latest`
      editingModel.value.name = modelName
    }
    await invoke('update_ai_model', {
      id: editingModel.value.id,
      name: modelName,
      title: editingModel.value.title,
      remark: editingModel.value.remark,
    })
    selfHostedVisionModel.value = { ...editingModel.value }
    showModelEditModal.value = false
    initStatusData()
  } catch (error) {
    console.error('save model error', error)
  }
}

watch(() => appStore.locale, () => {
  initStatusData()
})

onMounted(async () => {
  console.log('IndexerSetting onMounted')
  try {
    const indexerSetting = await invoke<IndexerSetting>('load_indexer_setting')
    indexerStore.setIndexerSetting(indexerSetting)

    activePlatform.value = await invoke<string>('load_active_platform')
    modelPlatformList.value = await invoke<ModelPlatform[]>(
      'load_model_platforms',
    )

    // Load self-hosted platforms
    selfHostedPlatformList.value = await invoke<SelfHostedPlatform[]>(
      'load_self_hosted_platforms',
    )
    activeSelfHostedPlatform.value = await invoke<string>(
      'load_active_self_hosted_platform',
    )

    const userDataPath = await invoke<string>('get_data_path')
    console.log('userDataPath', userDataPath)
    modelPath.value = await join(userDataPath, 'model')
    console.log('modelPath', modelPath.value)
    dbPath.value = await join(userDataPath, 'storage')
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
    <NCard :title="t('indexer.parserSetting')" class="mb-4" size="small" :bordered="true" hoverable>
      <div class="flex flex-col">
        <div class="flex flex-col space-y-1 mb-2">
          <NRadioGroup :value="parserMode" @update:value="doParserModeChanged">
            <NSpace>
              <NRadio value="local">
                {{ t('indexer.localMode') }}
                <AppTag v-if="parserMode === 'local'" type="success" size="small" class="ml-1">
                  {{ t('indexer.fullyPrivate') }}
                </AppTag>
              </NRadio>
              <NRadio value="selfhosted">
                {{ t('indexer.selfHostedMode') }}
              </NRadio>
              <NRadio value="remote">
                {{ t('indexer.cloudMode') }}
              </NRadio>
              <NRadio v-if="parserMode === 'mixed'" value="mixed" disabled>
                {{ t('indexer.mixedMode') }}
              </NRadio>
            </NSpace>
          </NRadioGroup>
          <NAlert :show-icon="false" class="text-xs" size="small">
            <div v-if="parserMode === 'local'">
              {{ t('indexer.localModeDesc') }}
            </div>
            <div v-if="parserMode === 'selfhosted'">
              {{
                t('indexer.selfHostedModeDescDynamic', {
                  platform: selfHostedPlatformList.find((p) => p.name === activeSelfHostedPlatform)?.title,
                  model: selfHostedVisionModel?.title || selfHostedVisionModel?.name || 'LLaVA',
                })
              }}
            </div>
            <div v-if="parserMode === 'selfhosted'" class="mt-1">
              <NButton text type="primary" size="tiny" class="text-link" @click="showSelfHostedModal = true">
                {{ t('indexer.selfHostedSetting') }}
              </NButton>
            </div>
            <div v-if="parserMode === 'remote'">
              {{
                t('indexer.cloudModeDescDynamic', {
                  modelPlatform: modelPlatformList.find(
                    (p) => p.name === activePlatform,
                  )?.title,
                })
              }}
            </div>
            <div v-if="parserMode === 'remote'" class="mt-1">
              <NButton text type="primary" size="tiny" class="text-link" @click="showCloudModal = true">
                {{ t('indexer.cloudModeSetting') }}
              </NButton>
            </div>
            <div v-if="parserMode === 'mixed'">
              {{ t('indexer.mixedModeDesc') }}
            </div>
          </NAlert>
        </div>
        <NTable size="small" class="mb-2">
          <tbody class="text-xs">
            <tr>
              <td class="w-32">
                {{ t('indexer.documentParser') }}
              </td>
              <td>{{ t('common.local') }}</td>
            </tr>
            <tr>
              <td>{{ t('indexer.imageParser') }}</td>
              <td>
                <NRadioGroup
                  :value="indexerStore.indexerSetting.image_parser_mode"
                  size="small"
                  @update:value="onImageParserModeChanged"
                >
                  <NRadio value="local">
                    {{ t('common.local') }}
                  </NRadio>
                  <NRadio value="selfhosted">
                    {{ t('common.selfHosted') }}
                  </NRadio>
                  <NRadio value="remote">
                    {{ t('common.cloud') }}
                  </NRadio>
                </NRadioGroup>
              </td>
            </tr>
            <tr>
              <td>{{ t('indexer.audioParser') }}</td>
              <td>
                <NRadioGroup
                  :value="indexerStore.indexerSetting.audio_parser_mode"
                  size="small"
                  @update:value="onAudioParserModeChanged"
                >
                  <NRadio value="local">
                    {{ t('common.local') }}
                  </NRadio>
                  <NTooltip>
                    <template #trigger>
                      <NRadio value="selfhosted" disabled>
                        {{ t('common.selfHosted') }}
                      </NRadio>
                    </template>
                    {{ t('common.selfHostedNotSupportASR') }}
                  </NTooltip>
                  <NRadio value="remote">
                    {{ t('common.cloud') }}
                  </NRadio>
                </NRadioGroup>
              </td>
            </tr>
          </tbody>
        </NTable>

        <!-- Self-hosted platform modal -->
        <NModal v-model:show="showSelfHostedModal" preset="card" :title="t('indexer.selfHostedSetting')" style="width: 600px">
          <NScrollbar style="max-height: 70vh">
            <NFormItem :label="t('indexer.selectActivePlatform')">
              <NRadioGroup :value="activeSelfHostedPlatform" @update:value="doActiveSelfHostedPlatformChanged">
                <NRadio
                  v-for="platform in selfHostedPlatformList" :key="platform.id" :label="platform.title"
                  :value="platform.name"
                />
              </NRadioGroup>
            </NFormItem>
            <NCard size="small" :bordered="true">
              <NFormItem :label="t('common.visionModel')">
                <span class="selectable mr-4">{{ selfHostedVisionModel?.title || selfHostedVisionModel?.name || '-' }}</span>
                <NButton text type="primary" size="tiny" class="text-link ml-2" @click="openModelEditModal">
                  {{ t('indexer.editModel') }}
                </NButton>
              </NFormItem>
              <NFormItem :label="`${selfHostedPlatformList.find(p => p.name === activeSelfHostedPlatform)?.title} ${t('common.setting')}`">
                <SelfHostedPlatformEdit
                  v-for="platform in selfHostedPlatformList"
                  v-show="platform.name === activeSelfHostedPlatform"
                  :key="platform.name"
                  :platform="platform"
                  @saved="onSelfHostedPlatformSaved"
                />
              </NFormItem>
            </NCard>
          </NScrollbar>
        </NModal>

        <!-- Cloud platform modal -->
        <NModal v-model:show="showCloudModal" preset="card" :title="t('indexer.cloudModeSetting')" style="width: 600px">
          <NScrollbar style="max-height: 70vh">
            <NFormItem :label="t('model.selectForActivePlatform')">
              <NRadioGroup :value="activePlatform" @update:value="doActivePlatformChanged">
                <NRadio
                  v-for="platform in modelPlatformList" :key="platform.id" :label="platform.title"
                  :value="platform.name"
                />
              </NRadioGroup>
            </NFormItem>
            <NCard size="small" :bordered="true">
              <div>
                <ModelPlatformEdit
                  v-for="platform in modelPlatformList"
                  v-show="platform.name === activePlatform"
                  :key="platform.name"
                  :model-platform="platform"
                  @saved="onModelPlatformSaved"
                />
              </div>
            </NCard>
          </NScrollbar>
        </NModal>
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
            <NSwitch
              size="small" :value="indexerStore.indexerSetting.save_parsed_content.document"
              @update:value="doParsedContentChange1"
            />
          </div>
          <div>
            <div>{{ t('indexer.saveImageParsedContent') }}</div>
            <NSwitch
              size="small" :value="indexerStore.indexerSetting.save_parsed_content.image"
              @update:value="doParsedContentChange2"
            />
          </div>
          <div>
            <div>{{ t('indexer.saveAudioParsedContent') }}</div>
            <NSwitch
              size="small" :value="indexerStore.indexerSetting.save_parsed_content.audio"
              @update:value="doParsedContentChange3"
            />
          </div>
        </div>
      </div>
    </NCard>

    <!-- Model Edit Modal -->
    <NModal v-model:show="showModelEditModal" preset="card" :title="t('indexer.editModel')" style="width: 500px">
      <NForm v-if="editingModel" ref="modelEditFormRef" label-placement="left" label-width="80">
        <NFormItem :label="t('common.name')" :rule="{ required: true, message: t('common.nameRequired') }">
          <NInput v-model:value="editingModel.name" />
          <template #feedback>
            <span class="text-xs text-gray-500">{{ t('indexer.modelNameHint', { platform: selfHostedPlatformList.find((p) => p.name === activeSelfHostedPlatform)?.title || 'Ollama' }) }}</span>
          </template>
        </NFormItem>
        <NFormItem :label="t('common.title')" :rule="{ required: true, message: t('common.titleRequired') }">
          <NInput v-model:value="editingModel.title" />
        </NFormItem>
        <NFormItem :label="t('common.description')">
          <NInput v-model:value="editingModel.remark" type="textarea" :rows="3" />
        </NFormItem>
      </NForm>
      <template #footer>
        <NSpace justify="end">
          <NButton ghost @click="showModelEditModal = false">
            {{ t('common.cancel') }}
          </NButton>
          <NButton type="primary" ghost :disabled="!editingModel?.name || !editingModel?.title" @click="saveModel">
            {{ t('common.save') }}
          </NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>
