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
    setDocumentParsedContent(value: boolean) {
      this.indexerSetting.save_parsed_content.document = value
    },
    setImageParsedContent(value: boolean) {
      this.indexerSetting.save_parsed_content.image = value
    },
    setAudioParsedContent(value: boolean) {
      this.indexerSetting.save_parsed_content.audio = value
    },
    setIndexProcessing(value: boolean) {
      this.indexProcessing = value
    },
  },
})
