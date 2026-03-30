import { defineStore } from 'pinia'

function getInitialState(): AppState {
  const theme = localStorage.getItem('theme')
  return {
    theme: theme || 'light',
    locale: 'en-US',
    clusterPortError: null,
  }
}

export const useAppStore = defineStore('app-store', {
  state: (): AppState => getInitialState(),
  getters: {
    getTheme: state => state.theme,
    getClusterPortError: state => state.clusterPortError,
  },
  actions: {
    changeTheme() {
      this.theme = (this.getTheme === 'dark' ? 'light' : 'dark')
      localStorage.setItem('theme', this.getTheme)
    },
    setLocale(locale: string) {
      this.locale = locale
    },
    setClusterPortError(error: { port: number } | null) {
      this.clusterPortError = error
    },
    clearClusterPortError() {
      this.clusterPortError = null
    },
  },
})
