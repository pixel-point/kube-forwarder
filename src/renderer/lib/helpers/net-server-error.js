
export function netServerPrettyError(error) {
  if (error.code === 'EADDRINUSE') {
    return new Error(`Port ${error.port} already in use`)
  }

  if (error.code === 'EACCES') {
    return new Error('Application haven\'t enough privileges to use local ports below 1024.')
  }

  return error
}
