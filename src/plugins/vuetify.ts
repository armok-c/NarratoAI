import 'vuetify/styles'
import '@mdi/font/css/materialdesignicons.css'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { zhHans } from 'vuetify/locale'

const vuetify = createVuetify({
  components,
  directives,
  locale: {
    locale: 'zhHans',
    messages: { zhHans },
  },
  icons: { defaultSet: 'mdi' },
  theme: {
    defaultTheme: 'light',
    themes: {
      light: {
        colors: {
          primary: '#2563EB',
          secondary: '#64748B',
          background: '#F8FAFC',
          surface: '#FFFFFF',
          'surface-variant': '#F1F5F9',
          error: '#DC2626',
          'on-background': '#334155',
          'on-surface': '#1E293B',
          info: '#0284C7',
          success: '#16A34A',
          warning: '#D97706',
        },
      },
      dark: {
        colors: {
          primary: '#3B82F6',
          secondary: '#94A3B8',
          background: '#0F172A',
          surface: '#1E293B',
          'surface-variant': '#334155',
          error: '#EF4444',
          'on-background': '#E2E8F0',
          'on-surface': '#F1F5F9',
          info: '#0EA5E9',
          success: '#22C55E',
          warning: '#F59E0B',
        },
      },
    },
  },
})

export default vuetify
