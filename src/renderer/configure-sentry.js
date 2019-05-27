import { getCurrentHub } from '@sentry/electron'
import * as Integrations from '@sentry/integrations'
import configureSentryDefault from '../common/configure-sentry'

export default function configureSentry(overrides = {}) {
  // This integrations prevent an appearing of errors to console, so it is enabled only for production
  const enableVueIntegration = process.env.NODE_ENV === 'production'

  const vueIntegration = new Integrations.Vue({ Vue: overrides.Vue, attachProps: true })

  // > Electron SDK hasn't been updated to use 5.x yet
  // So we have to patch integration
  // https://github.com/getsentry/sentry-javascript/issues/2033
  vueIntegration.setupOnce = vueIntegration.setupOnce.bind(vueIntegration, null, getCurrentHub)

  configureSentryDefault({
    ...overrides,
    Vue: null,
    integrations: [enableVueIntegration && vueIntegration].filter(Boolean)
  })
}
