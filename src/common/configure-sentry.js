import * as Sentry from '@sentry/electron'
import { defaultIntegrations } from '@sentry/browser/esm/index.js'
import { Breadcrumbs } from '@sentry/browser/esm/integrations'

import packageJson from '../../package'

export default function configureSentry(configOverrides = {}) {
  if (!process.env.SENTRY_DSN) return

  Sentry.init({
    release: `${packageJson.name}@${packageJson.version}`,
    dsn: process.env.SENTRY_DSN,
    defaultIntegrations: false,
    async beforeSend(event) {
      // Ignore Errors with sentryIgnore == true
      if (event.extra && event.extra.Error && event.extra.Error.sentryIgnore) return null

      if (event.exception && event.exception.values && event.exception.values[0]) {
        const value = event.exception.values[0]

        // ignore Socket error.
        if (value.type === 'Error' && value.value === 'This socket has been ended by the other party') return null
      }

      // Send errors only in production mode.
      if (process.env.NODE_ENV === 'production') return event

      console.warn('Sentry send:', event)
      return null
    },

    ...configOverrides,

    integrations() {
      const integrations = defaultIntegrations.map(integration => {
        if (integration.name === 'Breadcrumbs') {
          // Disable navigation breadcrumbs to stop gathering paths to index.html file
          // which could contain private data (user names)
          // Use a custom navigation breadcrumb in src/renderer/router/index.js
          return new Breadcrumbs({ history: false })
        }

        return integration
      })

      if (configOverrides.integrations) {
        return configOverrides.integrations(integrations)
      }

      return integrations
    }
  })

  Sentry.configureScope(scope => {
    scope.setTag('app.version', packageJson.version)
    scope.setTag('app.build', process.env.BUILD)
    scope.setTag('app.env', process.env.NODE_ENV)
  })
}
