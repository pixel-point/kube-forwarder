'use strict'

process.env.NODE_ENV = 'production'

require('../env')
const fs = require('fs').promises
const dateFormat = require('dateformat')
const path = require('path')
const mkdirp = require('mkdirp')
const { say } = require('cfonts')
const chalk = require('chalk')
const del = require('del')
const { spawn } = require('child_process')
const webpack = require('webpack')
const Multispinner = require('multispinner')
const gm = require('gm').subClass({ imageMagick: true })

const mainConfig = require('./webpack.main.config')
const rendererConfig = require('./webpack.renderer.config')
const webConfig = require('./webpack.web.config')
const packageJson = require('../package')

const doneLog = chalk.bgGreen.white(' DONE ') + ' '
const errorLog = chalk.bgRed.white(' ERROR ') + ' '
const okayLog = chalk.bgBlue.white(' OKAY ') + ' '
const isCI = process.env.CI || false

if (process.env.BUILD_TARGET === 'clean') clean()
else if (process.env.BUILD_TARGET === 'web') web()
else build()

function clean() {
  del.sync(['build/*', '!build/icons', '!build/icons/icon.*'])
  console.log(`\n${doneLog}\n`)
  process.exit()
}

async function build() {
  const [ver, buildNumber] = await setBuildVersionAndNumber()

  del.sync(['dist/electron/*', '!.gitkeep'])

  const tasks = ['main', 'renderer', 'logo']
  const m = new Multispinner(tasks, {
    preText: 'building',
    postText: 'process'
  })

  let results = ''

  m.on('success', () => {
    process.stdout.write('\x1B[2J\x1B[0f')
    console.log(`\n\n${results}`)
    console.log(`${okayLog}take it away ${chalk.yellow('`electron-builder`')}\n`)
    process.exit()
  })

  pack({
    ...mainConfig,
    plugins: [
      ...mainConfig.plugins,
      new webpack.DefinePlugin({
        'process.env.BUILD': `"${buildNumber}"`
      }),
    ]
  }).then(result => {
    results += result + '\n\n'
    m.success('main')
  }).catch(err => {
    m.error('main')
    console.log(`\n  ${errorLog}failed to build main process`)
    console.error(`\n${err}\n`)
    process.exit(1)
  })

  pack({
    ...rendererConfig,
    plugins: [
      ...rendererConfig.plugins,
      new webpack.DefinePlugin({
        'process.env.BUILD': `"${buildNumber}"`
      }),
    ]
  }).then(result => {
    results += result + '\n\n'
    m.success('renderer')
  }).catch(err => {
    m.error('renderer')
    console.log(`\n  ${errorLog}failed to build renderer process`)
    console.error(`\n${err}\n`)
    process.exit(1)
  })

  prepareLogo().then(() => {
    m.success('logo')
  }).catch(err => {
    m.error('renderer')
    console.error(err)
    process.exit(1)
  })
}

async function setBuildVersionAndNumber() {
  const { version } = packageJson
  const buildNumber = dateFormat(new Date(), 'yyyyddmm-HHMMss', true)

  await mkdirp(path.resolve(__dirname, '../build'))
  await Promise.all([
    fs.writeFile(path.resolve(__dirname, '../build/.number'), buildNumber),
    fs.writeFile(path.resolve(__dirname, '../build/.version'), version),
  ])

  return [version, buildNumber]
}

function pack(config) {
  return new Promise((resolve, reject) => {
    config.mode = 'production'
    webpack(config, (err, stats) => {
      if (err) reject(err.stack || err)
      else if (stats.hasErrors()) {
        let err = ''

        stats.toString({
          chunks: false,
          colors: true
        })
          .split(/\r?\n/)
          .forEach(line => {
            err += `    ${line}\n`
          })

        reject(err)
      } else {
        resolve(stats.toString({
          chunks: false,
          colors: true
        }))
      }
    })
  })
}

function web() {
  del.sync(['dist/web/*', '!.gitkeep'])
  webConfig.mode = 'production'
  webpack(webConfig, (err, stats) => {
    if (err || stats.hasErrors()) console.log(err)

    console.log(stats.toString({
      chunks: false,
      colors: true
    }))

    process.exit()
  })
}

async function prepareLogo() {
  const createIconsDir = new Promise((resolve, reject) => {
    mkdirp(path.resolve(__dirname, '../build/icons'), (err) => {
      if (!err) resolve()
      else reject(err)
    })
  })

  const x256 = new Promise((resolve, reject) => {
    gm(path.resolve(__dirname, '../src/renderer/assets/logo.svg'))
      .background('none')
      .rotate('none', 45)
      .resize(512, 512)
      .write(path.resolve(__dirname, '../build/icons/256x256.png'), function(err) {
        if (err) reject(err)
        else resolve()
      })
  })

  return createIconsDir.then(() => Promise.all([x256]))
}
