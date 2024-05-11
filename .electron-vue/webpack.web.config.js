'use strict';

process.env.BABEL_ENV = 'web';

const path = require('path');
const webpack = require('webpack');

const TerserPlugin = require('terser-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const { VueLoaderPlugin } = require('vue-loader');

let webConfig = {
  devtool: 'cheap-module-source-map',
  entry: {
    web: path.join(__dirname, '../src/renderer/main.js'),
  },
  module: {
    rules: [
      {
        test: /\.scss$/,
        use: ['vue-style-loader', 'css-loader', 'sass-loader'],
      },
      {
        test: /\.sass$/,
        use: ['vue-style-loader', 'css-loader', 'sass-loader?indentedSyntax'],
      },
      {
        test: /\.less$/,
        use: ['vue-style-loader', 'css-loader', 'less-loader'],
      },
      {
        test: /\.css$/,
        use: ['vue-style-loader', 'css-loader'],
      },
      {
        test: /\.js$/,
        use: 'babel-loader',
        include: [path.resolve(__dirname, '../src/renderer')],
        exclude: /node_modules/,
      },
      {
        test: /\.vue$/,
        use: 'vue-loader',
      },
      {
        test: /\.(png|jpe?g|gif|svg)(\?.*)?$/,
        type: 'asset/resource',
        generator: {
          filename: 'imgs/[name].[ext]',
        },
      },
      {
        test: /\.(woff2?|eot|ttf|otf)(\?.*)?$/,
        type: 'asset/resource',
        generator: {
          filename: 'fonts/[name].[ext]',
        },
      },
    ],
  },
  plugins: [
    new VueLoaderPlugin(),
    new MiniCssExtractPlugin({ filename: 'styles.css' }),
    new HtmlWebpackPlugin({
      filename: 'index.html',
      template: path.resolve(__dirname, '../src/index.ejs'),
      minify: {
        collapseWhitespace: true,
        removeAttributeQuotes: true,
        removeComments: true,
      },
      nodeModules: false,
    }),
    new webpack.DefinePlugin({
      'process.env.IS_WEB': 'true',
      'process.env.WEB_DEMO_MODE': `"${process.env.WEB_DEMO_MODE}"`,
    }),
  ],
  output: {
    filename: '[name].js',
    path: path.join(__dirname, '../dist/web'),
  },
  resolve: {
    alias: {
      '@': path.join(__dirname, '../src/renderer'),
      'vue$': 'vue/dist/vue.esm.js',
    },
    extensions: ['.js', '.vue', '.json', '.css'],
  },
  target: 'web',
};

if (process.env.NODE_ENV === 'production') {
  webConfig.devtool = '';

  webConfig.optimization = {
    minimize: true,
    minimizer: [new TerserPlugin()],
  };

  webConfig.plugins = [
    ...webConfig.plugins,
    new CopyWebpackPlugin({
      patterns: [
        {
          from: path.join(__dirname, '../static'),
          to: path.join(__dirname, '../dist/web/static'),
        },
      ],
    }),
    new webpack.DefinePlugin({
      'process.env.NODE_ENV': '"production"',
    }),
  ];
}

module.exports = webConfig;
