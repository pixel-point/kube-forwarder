const dotEnv = require('dotenv-safe')
if (!process.env.CI) dotEnv.load()
