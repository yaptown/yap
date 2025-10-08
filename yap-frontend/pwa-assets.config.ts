import {
  defineConfig,
  minimal2023Preset as preset
} from '@vite-pwa/assets-generator/config'

export default defineConfig({
  headLinkOptions: {
    preset: '2023'
  },
  preset: {
    ...preset,
    apple: {
      sizes: [180],
      padding: 0.3,
      resizeOptions: {
        background: '#0e0e0a',
        fit: 'contain'
      }
    }
  },
  images: ['public/yap.svg']
})
