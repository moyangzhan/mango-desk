<script setup lang='ts'>
import { invoke } from '@tauri-apps/api/core'
import { emptyModelPlatform } from '@/utils/functions'
import { useSettingStore } from '@/stores/setting'
import { t } from '@/locales'

interface Props {
  modelPlatform: ModelPlatform
}
const props = defineProps<Props>()
const emit = defineEmits<Emit>()
interface Emit {
  (ev: 'saved', updatedPlatform: ModelPlatform): void
}
const settingStore = useSettingStore()
const tmpPlatform = ref<ModelPlatform>(emptyModelPlatform())
tmpPlatform.value = { ...props.modelPlatform }

function gotoProxySetting() {
  settingStore.changeTab('common')
}

async function onSave() {
  await invoke('update_model_platform', { platform: tmpPlatform })
  emit('saved', tmpPlatform.value)
}
</script>

<template>
  <div>
    <NFormItem :label="t('common.name')">
      <NInput v-model:value="tmpPlatform.name" disabled />
    </NFormItem>
    <NFormItem :label="t('common.title')">
      <NInput v-model:value="tmpPlatform.title" />
    </NFormItem>
    <NFormItem label="Base Url">
      <NInput v-model:value="tmpPlatform.base_url" />
    </NFormItem>
    <NFormItem label="Api Key">
      <NInput v-model:value="tmpPlatform.api_key" type="password" show-password-on="click" />
    </NFormItem>
    <NFormItem :label="t('proxy.enable')">
      <NSwitch v-model:value="tmpPlatform.is_proxy_enable" class="mr-6" />
      <NButton text tag="a" target="_blank" type="primary" @click="gotoProxySetting">
        {{
          t('common.setting').toLowerCase()
        }}
      </NButton>
    </NFormItem>
    <NFormItem :label="t('common.description')">
      <NInput v-model:value="tmpPlatform.remark" />
    </NFormItem>
    <div>
      <NButton @click="onSave">
        {{ t('common.save') }}
      </NButton>
    </div>
  </div>
</template>
