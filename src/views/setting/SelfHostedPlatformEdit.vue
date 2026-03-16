<script setup lang='ts'>
import { invoke } from '@tauri-apps/api/core'
import { useDebounceFn } from '@vueuse/core'
import { t } from '@/locales'

interface Props {
  platform: SelfHostedPlatform
}
const props = defineProps<Props>()
const emit = defineEmits<Emit>()
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
</script>

<template>
  <div>
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
  </div>
</template>
