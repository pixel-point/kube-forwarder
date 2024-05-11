'use strict'

process.env.NODE_ENV = 'production'

require('../env')
const fs = require('fs').promises
const path = require('path')
const chalk = require('chalk')
const del = require('del')
const { exec } = require('child_process')
const webpack = require('webpack')
const Multispinner = require('multispinner')

const mainConfig = require('./webpack.main.config')
const rendererConfig = require('./webpack.renderer.config')
const webConfig = require('./webpack.web.config')
const packageJson = require('../package')

const doneLog = chalk.bgGreen.white(' DONE ') + ' '
const errorLog = chalk.bgRed.white(' ERROR ') + ' '
const okayLog = chalk.bgBlue.white(' OKAY ') + ' '
const isCI = process.env.CI || false

let dateFormat; // Placeholder for the dateFormat module

// Dynamically import the dateFormat module
import('dateformat').then(module => {
  dateFormat = module.default;
  if (process.env.BUILD_TARGET === 'clean') clean()
  else if (process.env.BUILD_TARGET === 'web') web()
  else build()
}).catch(error => console.error(`Failed to load module: ${error}`));

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
  const buildNumber = dateFormat(new Date(), 'yyyymmdd-HHMMss', true)

  await fs.mkdir(path.resolve(__dirname, '../build'), { recursive: true })
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
	try {
	  await fs.mkdir(path.resolve(__dirname, '../build/icons'), { recursive: true });
	  console.log(`${doneLog}Created icons directory.`);
	} catch (err) {
	  console.error(`${errorLog}Failed to create icons directory: ${err}`);
	  throw err; // Rethrow to be caught by the caller
	}

	try {
	  const generateIconsCmd = 'bash .electron-vue/generate-icons.sh src/app-icon.png build';
	  await execPromise(generateIconsCmd);
	  console.log(`${doneLog}Icons generated successfully.`);
	} catch (err) {
	  console.error(`${errorLog}Failed to generate icons: ${err}`);
	  throw err; // Rethrow to be caught by the caller
	}
  }

  // Utility function to promisify exec
  function execPromise(command) {
	return new Promise((resolve, reject) => {
	  exec(command, (error, stdout, stderr) => {
		if (error) {
		  console.error(`exec error: ${error}`);
		  reject(error);
		}
		if (stderr) {
		  console.error(`stderr: ${stderr}`);
		}
		console.log(`stdout: ${stdout}`);
		resolve(stdout);
	  });
	});
  }
