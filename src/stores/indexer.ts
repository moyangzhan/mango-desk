import { defineStore } from 'pinia'
import { emptyIndexerSetting } from '@/utils/functions'

export const useIndexerStore = defineStore('indexer-store', {
  state: () => ({
    indexerSetting: emptyIndexerSetting(),
    indexProcessing: false,
    watcherProcessing: false,
  }),
  getters: {},
  actions: {
    setIndexerSetting(setting: IndexerSetting) {
      this.indexerSetting = setting
    },
    setWatcherProcessing(value: boolean) {
      this.watcherProcessing = value
    },
    setIndexProcessing(value: boolean) {
      this.indexProcessing = value
    },
  },
})
