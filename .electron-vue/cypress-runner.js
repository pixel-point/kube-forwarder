const { spawn } = require('child_process')

const { startWeb } = require('./web-runner')

function runCypress() {
  return new Promise((resolve, reject) => {
    const cypressProcess = spawn(`node_modules/.bin/cypress`, ['run'])
    console.log('Cypress is spawned.')

    cypressProcess.stdout.on('data', function(data) {
      console.log(data.toString())
    })

    cypressProcess.on('close', () => {
      resolve()
    })

    cypressProcess.on('error', (error) => {
      reject(error)
    })

    cypressProcess.on('disconnect', () => {
      reject(new Error('Cypress was disconnected'))
    })
  })
}

async function init() {
  const webpackServer = await startWeb()
  console.log('Renderer is started at 9081')

  await runCypress()

  webpackServer.close()
}

init()
