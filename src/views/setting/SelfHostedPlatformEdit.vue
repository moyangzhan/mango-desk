<script setup lang='ts'>
import { invoke } from '@tauri-apps/api/core'
import { useDebounceFn } from '@vueuse/core'
import { t } from '@/locales'
import { useAppStore } from '@/stores/app'

const props = defineProps<Props>()
const emit = defineEmits<Emit>()
const appStore = useAppStore()
const labelWidth = computed(() => appStore.locale === 'zh-CN' ? 60 : 100)

interface Props {
  platform: SelfHostedPlatform
}
interface Emit {
  (ev: 'saved', updatedPlatform: SelfHostedPlatform): void
}

const tmpPlatform = ref<SelfHostedPlatform>({
  id: 0,
  name: '',
  title: '',
  host: '127.0.0.1',
  port: 11434,
  remark: '',
})
tmpPlatform.value = { ...props.platform }

const debounceSave = useDebounceFn(async () => {
  await invoke('update_self_hosted_platform', { platform: tmpPlatform.value })
  emit('saved', tmpPlatform.value)
}, 1000)

const checking = ref(false)
async function checkConnection() {
  checking.value = true
  try {
    await invoke('check_self_hosted_platform', { platform: tmpPlatform.value })
    window.$message.success(t('common.checkConnectionSuccess'))
  } catch (e: any) {
    window.$message.error(`${t('common.checkConnectionFailed')}: ${e}`)
  } finally {
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
      <NFormItem label="Host">
        <NInput v-model:value="tmpPlatform.host" @update:value="debounceSave" />
      </NFormItem>
      <NFormItem label="Port">
        <NInputNumber v-model:value="tmpPlatform.port" :min="1" :max="65535" @update:value="debounceSave" />
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
