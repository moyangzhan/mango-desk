<script setup lang="ts">
import { useRoute } from 'vue-router'
import CommonSetting from './CommonSetting.vue'
import IndexerSetting from './IndexerSetting.vue'
import About from './About.vue'
import { useSettingStore } from '@/stores/setting'
import { t } from '@/locales'

const route = useRoute()
const settingStore = useSettingStore()
const activeTab = computed(() => {
  return settingStore.activeTab
})

if (route.query.tab) {
  console.log('route.query.tab', route.query.tab)
  settingStore.changeTab(route.query.tab as string)
}

function onUpdateTab(tabName: string) {
  console.log('onUpdateTab', tabName)
  settingStore.changeTab(tabName)
}

onMounted(async () => {
  console.log('Settings onMounted')
})
</script>

<template>
  <div class="h-full mx-auto">
    <NTabs :value="activeTab" type="line" size="large" :tabs-padding="20"
      pane-style=" height: 100%; overflow-y: auto;padding: 20px;" style="height: 100%;" @update:value="onUpdateTab">
      <NTabPane name="common" :tab="t('common.commonSetting')">
        <CommonSetting />
      </NTabPane>
      <NTabPane name="indexer" :tab="t('indexer.setting')">
        <IndexerSetting />
      </NTabPane>
      <NTabPane name="about" :tab="t('menu.about')">
        <About />
      </NTabPane>
    </NTabs>
  </div>
</template>

<style scoped></style>
