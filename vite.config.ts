import path from 'path'
import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'
import tailwindcss from '@tailwindcss/vite'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'
import svgLoader from 'vite-svg-loader';

const host = process.env.TAURI_DEV_HOST

export default defineConfig(async ({ command, mode }) => {
  const env = loadEnv(mode, process.cwd(), 'TAURI_ENV_')
  const port = parseInt(env.PORT || '5173', 10)

  return {
    // Env variables starting with the item of `envPrefix` will be exposed in tauri's source code through `import.meta.env`.
    envPrefix: ['VITE_', 'TAURI_ENV_*'],
    plugins: [
      vue(),
      vueDevTools(),
      tailwindcss(),
      svgLoader(),
      AutoImport({
        imports: [
          'vue',
          'vue-router',
          {
            'naive-ui': [
              'useDialog',
              'useMessage',
              'useNotification',
              'useLoadingBar',
            ],
          },
        ],
      }),
      Components({
        resolvers: [NaiveUiResolver()],
      }),
    ],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, 'src'),
      },
    },
    clearScreen: false,
    server: {
      // make sure this port matches the devUrl port in tauri.conf.json file
      port,
      // Tauri expects a fixed port, fail if that port is not available
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: 'ws',
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        ignored: ['**/src-tauri/**'],
      },
    },
  }
})
