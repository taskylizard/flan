import { fileURLToPath, URL } from 'node:url'
import vue from '@vitejs/plugin-vue'
import RadixVue from 'radix-vue/resolver'
import AutoImport from 'unplugin-auto-import/vite'
import Fonts from 'unplugin-fonts/vite'
import IconsResolver from 'unplugin-icons/resolver'
import Icons from 'unplugin-icons/vite'
import Components from 'unplugin-vue-components/vite'
import { VueRouterAutoImports } from 'unplugin-vue-router'
import VueRouter from 'unplugin-vue-router/vite'
import { defineConfig } from 'vite'
import Terminal from 'vite-plugin-terminal'
import VueDevTools from 'vite-plugin-vue-devtools'

export default defineConfig(({ mode }) => {
  const API_HOST = 'http://localhost:8111'
  console.log(mode)
  return {
    plugins: [
      vue(),
      VueRouter({
        dts: true,
      }),
      Icons({ scale: 1.2, compiler: 'vue3', autoInstall: true }),
      AutoImport({
        imports: [
          'vue',
          '@vueuse/core',
          VueRouterAutoImports,
          'pinia',
          '@vueuse/core',
          '@vueuse/head',
        ],
        dts: true,
        dirs: ['./src/composables', './src/stores'],
        vueTemplate: true,
        resolvers: [
          RadixVue(),
          IconsResolver({ alias: { radix: 'radix-icons' } }),
        ],
      }),
      Components({
        dts: true,
        resolvers: [
          RadixVue(),
          IconsResolver({
            alias: {
              radix: 'radix-icons',
            },
          }),
        ],
      }),
      Fonts({
        fontsource: {
          families: ['Geist Sans'],
        },
      }),
      Terminal({ output: ['console', 'terminal'] }),
      VueDevTools(),
    ],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
      },
    },
    css: {
      preprocessorOptions: {
        scss: {
          api: 'modern-compiler',
        },
      },
    },
    ...(mode === 'development' && {
      server: {
        proxy: {
          '/api': {
            target: API_HOST,
            changeOrigin: true,
          },
          '/images': {
            target: API_HOST,
            changeOrigin: true,
          },
        },
      },
    }),
  }
})
