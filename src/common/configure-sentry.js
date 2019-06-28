import * as Sentry from '@sentry/electron'

import packageJson from '../../package'

export default function configureSentry(configOverrides = {}) {
  if (process.env.NODE_ENV !== 'production') return

  Sentry.init({
    dsn: process.env.SENTRY_DSN,
    ...configOverrides
  })

  Sentry.configureScope(scope => {
    scope.setTag('app.version', packageJson.version)
    scope.setTag('app.build', process.env.BUILD)
    scope.setTag('app.env', process.env.NODE_ENV)
  })
}
