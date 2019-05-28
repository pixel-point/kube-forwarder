const webpack = require('webpack')

const webConfig = require('./webpack.web.config')
const { startRenderer } = require('./dev-runner')

function startWeb() {
  return startRenderer({
    port: 9081,
    webpackConfig: {
      ...webConfig,
      plugins: [
        ...webConfig.plugins,
        new webpack.DefinePlugin({
          'process.env.NODE_ENV': '"test"'
        }),
      ]
    }
  })
}

async function init() {
  await startWeb()
  console.log('Web is started at 9081')
}

if (require.main === module) {
  init()
} else {
  module.exports = {
    startWeb
  }
}
