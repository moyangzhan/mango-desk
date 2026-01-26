import { defineStore } from 'pinia'
import { emptyIndexerSetting } from '@/utils/functions'

export const useIndexerStore = defineStore('indexer-store', {
  state: () => ({
    indexerSetting: emptyIndexerSetting(),
    indexProcessing: false,
  }),
  getters: {},
  actions: {
    setIndexerSetting(setting: IndexerSetting) {
      this.indexerSetting = setting
    },
    setDocumentParsedContent(value: boolean) {
      this.indexerSetting.save_parsed_content.document = value
    },
    setImageParsedContent(value: boolean) {
      this.indexerSetting.save_parsed_content.image = value
    },
    setAudioParsedContent(value: boolean) {
      this.indexerSetting.save_parsed_content.video = value
    },
    setIndexProcessing(value: boolean) {
      this.indexProcessing = value
    }
  },
})
