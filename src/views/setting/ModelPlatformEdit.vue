<script setup lang='ts'>
import { invoke } from '@tauri-apps/api/core'
import { useDebounceFn } from '@vueuse/core'
import { emptyModelPlatform } from '@/utils/functions'
import { useSettingStore } from '@/stores/setting'
import { t } from '@/locales'
import { useAppStore } from '@/stores/app'

const appStore = useAppStore()
const labelWidth = computed(() => appStore.locale === 'zh-CN' ? 80 : 100)

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

const debounceSave = useDebounceFn(async () => {
  await invoke('update_model_platform', { platform: tmpPlatform.value })
  emit('saved', tmpPlatform.value)
}, 1000)

const checking = ref(false)
async function checkConnection() {
  checking.value = true
  try {
    await invoke('check_model_platform', { platform: tmpPlatform.value })
    window.$message.success(t('common.checkConnectionSuccess'))
  }
  catch (e: any) {
    window.$message.error(`${t('common.checkConnectionFailed')}: ${e}`)
  }
  finally {
    checking.value = false
  }
}
</script>

<template>
  <div class="w-full">
    <NForm label-placement="left" :label-width="labelWidth" class="w-full">
      <NFormItem :label="t('common.name')">
        <NInput v-model:value="tmpPlatform.name" disabled />
      </NFormItem>
      <NFormItem :label="t('common.title')">
        <NInput v-model:value="tmpPlatform.title" @update:value="debounceSave" />
      </NFormItem>
      <NFormItem label="Base Url">
        <NInput v-model:value="tmpPlatform.base_url" @update:value="debounceSave" />
      </NFormItem>
      <NFormItem label="Api Key">
        <NInput
          v-model:value="tmpPlatform.api_key" type="password" show-password-on="click"
          @update:value="debounceSave"
        />
      </NFormItem>
      <NFormItem :label="t('proxy.enable')">
        <NSwitch v-model:value="tmpPlatform.is_proxy_enable" class="mr-6" @update:value="debounceSave" />
        <NButton text tag="a" target="_blank" type="primary" class="text-link" @click="gotoProxySetting">
          {{
            t('common.setting').toLowerCase()
          }}
        </NButton>
      </NFormItem>
      <NFormItem :label="t('common.description')">
        <NInput v-model:value="tmpPlatform.remark" @update:value="debounceSave" />
      </NFormItem>
    </NForm>
    <div class="w-full">
      <NButton type="primary" :loading="checking" @click="checkConnection">
        {{ t('common.checkConnection') }}
      </NButton>
    </div>
  </div>
</template>

<style scoped>
:deep(.n-form-item-label) {
  align-items: center !important;
}
</style>