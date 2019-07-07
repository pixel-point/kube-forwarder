const dotEnv = require('dotenv-safe')
dotEnv.load({
  path: process.env.NODE_ENV === 'production' ? '.env.production' : '.env',
  allowEmptyValues: true
})
