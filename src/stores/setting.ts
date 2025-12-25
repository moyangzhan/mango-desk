import { defineStore } from 'pinia'

function getState(): SettingState {
  return {
    activeTab: 'common',
  }
}

export const useSettingStore = defineStore('setting-store', {
  state: (): SettingState => getState(),
  getters: {
    getActiveTab: state => state.activeTab,
  },
  actions: {
    changeTab(tab: string) {
      this.activeTab = tab
    },
  },
})
