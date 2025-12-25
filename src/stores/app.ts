import { defineStore } from 'pinia'

function getInitialState(): AppState {
  const theme = localStorage.getItem('theme')
  return {
    theme: theme || 'light',
    locale: 'en-US',
  }
}

export const useAppStore = defineStore('app-store', {
  state: (): AppState => getInitialState(),
  getters: {
    getTheme: state => state.theme,
  },
  actions: {
    changeTheme() {
      this.theme = (this.getTheme === 'dark' ? 'light' : 'dark')
      localStorage.setItem('theme', this.getTheme)
    },
    setLocale(locale: string) {
      this.locale = locale
    },
  },
})
