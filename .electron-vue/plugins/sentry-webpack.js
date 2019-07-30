const packageJson = require('../../package.json')

const SentryWebpackPlugin = require('@sentry/webpack-plugin')

module.exports = new SentryWebpackPlugin({
  release: `${packageJson.name}@${packageJson.version}`,
  silent: true,
  include: ['.'],
  ignore: ['node_modules', '.electron-vue', 'build', 'cypress', 'test', 'src'],
  configFile: 'sentry.properties'
})
