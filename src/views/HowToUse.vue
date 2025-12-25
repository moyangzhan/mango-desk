<script setup lang="ts">
import { t } from '@/locales'

interface Props {
  showSteps?: boolean
}
const props = withDefaults(defineProps<Props>(), {
  showSteps: false,
})
const router = useRouter()
const showGuideStep = ref(props.showSteps)
</script>

<template>
  <div class="flex flex-col space-x-2 mt-2 items-start justify-start w-full">
    <slot name="tip">
      <div
        class="text-left hover:underline cursor-pointer hover:text-(--primary-color)"
        @click="showGuideStep = !showGuideStep"
      >
        {{ t('common.howToUse') }} ？
      </div>
    </slot>
    <NSteps v-if="showGuideStep" size="small" class="pt-2" style="filter: saturate(0.5)">
      <NStep title="Setting" status="process">
        <div class="flex flex-col">
          <div class="n-step-description text-left">
            {{ t('common.useStep1Desc') }}
          </div>
          <div
            class="hover:underline cursor-pointer hover:text-(--primary-color) flex w-12"
            @click="router.push({ name: 'Setting', query: { tab: 'indexer' } })"
          >
            GO
            <SvgIcon name="right-top-arrow" />
          </div>
        </div>
      </NStep>
      <NStep title="Indexing" status="process">
        <div class="flex flex-col">
          <div class="n-step-description text-left">
            {{ t('common.useStep2Desc') }}
          </div>
          <div
            class="hover:underline cursor-pointer hover:text-(--primary-color) flex w-12"
            @click="router.push({ name: 'Indexer' })"
          >
            GO
            <SvgIcon name="right-top-arrow" />
          </div>
        </div>
      </NStep>
      <NStep title="Search" status="process">
        <div class="n-step-description text-left">
          {{ t('common.useStep3Desc') }}
        </div>
      </NStep>
    </NSteps>
  </div>
</template>
