// Async version of https://www.npmjs.com/package/fix-path

import shellPath from 'shell-path'

export default async function() {
  if (process.platform !== 'darwin') {
    return
  }

  const path = await shellPath()

  process.env.PATH = path || [
    './node_modules/.bin',
    '/.nodebrew/current/bin',
    '/usr/local/bin',
    process.env.PATH
  ].join(':')
};
