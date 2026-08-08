import { defineConfig, mergeConfig } from 'vitest/config'
import { playwright } from '@vitest/browser-playwright'
import viteConfig from './vite.config'

export default mergeConfig(viteConfig, defineConfig({
  test: {
    browser: {
      enabled: true,
      instances: [{ browser: 'chromium', provider: playwright() }],
    },
  },
}))
