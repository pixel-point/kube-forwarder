'use strict'

process.env.BABEL_ENV = 'main'

const path = require('path')
const webpack = require('webpack')

const packageJson = require('../package.json')
const sentryWebpackPlugin = require('./plugins/sentry-webpack')

let mainConfig = {
  entry: {
    main: path.join(__dirname, '../src/main/index.js')
  },
  externals: [
    ...Object.keys(packageJson.dependencies || {})
  ],
  module: {
    rules: [
      {
        test: /\.js$/,
        use: 'babel-loader',
        exclude: /node_modules/
      },
      {
        test: /\.node$/,
        use: 'node-loader'
      }
    ]
  },
  node: {
    __dirname: process.env.NODE_ENV !== 'production',
    __filename: process.env.NODE_ENV !== 'production'
  },
  output: {
    filename: '[name].js',
    libraryTarget: 'commonjs2',
    path: path.join(__dirname, '../dist/electron')
  },
  plugins: [
    new webpack.NoEmitOnErrorsPlugin()
  ],
  resolve: {
    extensions: ['.js', '.json', '.node']
  },
  target: 'electron-main'
}

/**
 * Adjust mainConfig for development settings
 */
if (process.env.NODE_ENV !== 'production') {
  mainConfig.plugins.push(
    new webpack.DefinePlugin({
      '__static': `"${path.join(__dirname, '../static').replace(/\\/g, '\\\\')}"`
    })
  )
}

/**
 * Adjust mainConfig for production settings
 */
if (process.env.NODE_ENV === 'production') {
  mainConfig.devtool = 'source-map'

  mainConfig.plugins = [
    ...mainConfig.plugins,
    new webpack.DefinePlugin({
      'process.env.NODE_ENV': '"production"',
      'process.env.SENTRY_DSN': `"${process.env.SENTRY_DSN}"`
    }),
    process.env.RELEASE === 'true' ? sentryWebpackPlugin : null
  ].filter(Boolean)
}

module.exports = mainConfig
