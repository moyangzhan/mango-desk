import { defineStore } from 'pinia'

export const useIndexerStore = defineStore('indexer-store', {
  state: () => ({
    indexPaths: [] as string[],
  }),
  getters: {
    indexPaths: state => state.indexPaths,
  },
  actions: {
    add(path: string) {
      this.indexPaths.push(path)
    },
  },
})
