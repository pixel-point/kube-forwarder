const { spawn } = require('child_process')

const { startWeb } = require('./web-runner')

function runCypress(args) {
  return new Promise((resolve, reject) => {
    const cypressProcess = spawn(`node_modules/.bin/cypress`, ['run', ...args], {
      stdio: [process.stdin, process.stdout, process.stderr]
    })
    console.log('Cypress is spawned.')

    cypressProcess.on('close', (code) => {
      code === 0 ? resolve() : reject(new Error(`Cypress exited with code: ${code}`))
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
  const args = process.argv.slice(2)

  const webpackServer = await startWeb()
  console.log('Renderer is started at 9081')

  let error
  try {
    await runCypress(args)
  } catch(err) {
    error = err
  }
  console.log('Cypress completed.')

  webpackServer.close()
  console.log('Renderer stopped.')

  if (error) {
    console.error(error)
    process.exit(1)
  }
}

init()
