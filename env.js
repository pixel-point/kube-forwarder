const dotEnv = require('dotenv-safe');

dotEnv.config({
  path: process.env.NODE_ENV === 'production' ? '.env.production' : '.env',
  allowEmptyValues: true
});
