'use strict';

process.env.BABEL_ENV = 'main';

const path = require('path');
const webpack = require('webpack');

const packageJson = require('../package.json');

let mainConfig = {
  entry: {
    main: path.join(__dirname, '../src/main/index.js'),
  },
  externals: [
    ...Object.keys(packageJson.dependencies || {}),
  ],
  module: {
    rules: [
      {
        test: /\.js$/,
        use: 'babel-loader',
        exclude: /node_modules/,
      },
      {
        test: /\.node$/,
        use: 'node-loader',
      },
    ],
  },
  resolve: {
    extensions: ['.js', '.json', '.node'],
    fallback: { "path": require.resolve("path-browserify") },
  },
  output: {
    filename: '[name].js',
    libraryTarget: 'commonjs2',
    path: path.join(__dirname, '../dist/electron'),
  },
  plugins: [
    new webpack.NoEmitOnErrorsPlugin(),
  ],
  target: 'electron-main',
};

if (process.env.NODE_ENV !== 'production') {
  mainConfig.plugins.push(
    new webpack.DefinePlugin({
      '__static': `"${path.join(__dirname, '../static').replace(/\\/g, '\\\\')}"`,
    }),
  );
}

if (process.env.NODE_ENV === 'production') {
  mainConfig.devtool = 'source-map';

  mainConfig.plugins = [
    ...mainConfig.plugins,
    new webpack.DefinePlugin({
      'process.env.NODE_ENV': '"production"',
    }),
  ].filter(Boolean);
}

module.exports = mainConfig;
