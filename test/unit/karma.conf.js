'use strict'

const path = require('path')
const { merge } = require('webpack-merge')
const webpack = require('webpack')

const baseConfig = require('../../.electron-vue/webpack.renderer.config')
const projectRoot = path.resolve(__dirname, '../../src/renderer')

// Set BABEL_ENV to use proper preset config
process.env.BABEL_ENV = 'test'

let webpackConfig = merge(baseConfig, {
  devtool: 'inline-source-map', // Adjusted according to webpack 5 schema
  plugins: [
    new webpack.DefinePlugin({
      'process.env.NODE_ENV': '"testing"'
    })
  ]
})

// don't treat dependencies as externals
delete webpackConfig.entry
delete webpackConfig.externals
delete webpackConfig.output.libraryTarget

// Correctly apply babel-loader for js files within vue-loader
webpackConfig.module.rules = webpackConfig.module.rules.map(rule => {
  if (rule.loader === 'vue-loader') {
    rule.options.loaders = {
      ...rule.options.loaders,
      js: 'babel-loader'
    };
  } else if (rule.use && Array.isArray(rule.use)) {
    rule.use = rule.use.map(use => {
      if (use.loader === 'vue-loader') {
        return {
          loader: use.loader,
          options: {
            ...use.options,
            loaders: {
              js: 'babel-loader'
            }
          }
        };
      }
      return use;
    });
  }
  return rule;
});

module.exports = config => {
  config.set({
    browsers: ['visibleElectron'],
    client: {
      useIframe: false
    },
    coverageReporter: {
      dir: './coverage',
      reporters: [
        { type: 'lcov', subdir: '.' },
        { type: 'text-summary' }
      ]
    },
    customLaunchers: {
      'visibleElectron': {
        base: 'Electron',
        flags: ['--show']
      }
    },
    frameworks: ['mocha', 'chai'],
    files: ['./index.js'],
    preprocessors: {
      './index.js': ['webpack', 'sourcemap']
    },
    reporters: ['spec', 'coverage'],
    singleRun: true,
    webpack: webpackConfig,
    webpackMiddleware: {
      noInfo: true
    }
  })
}
