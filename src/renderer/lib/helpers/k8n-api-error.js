const postfixes = {
  notFound: 'not found.',
  forbidden: 'forbidden.',
  default: 'can\'t be fetched.'
}

function buildMessage(error, messages, messageKey) {
  // Return custom messages if exists
  if (messages[messageKey]) return messages[messageKey]
  if (messages._object) return `${messages._object} ${postfixes[messageKey]}`

  // Return error message from body.
  if (error.body && error.body.message) return error.body.message
  if (error.response && error.response.body && error.response.body.message) return error.response.body.message

  // Return default messages
  return postfixes[messageKey]
}

function getMessageKey(error) {
  if (error.response) {
    if (error.response.statusCode === 404) return 'notFound'
    if (error.response.statusCode === 403) return 'forbidden'
  }

  return 'default'
}

export function k8nApiPrettyError(error, messages = {}) {
  const messageKey = getMessageKey(error)
  const prettyError = new Error(buildMessage(error, messages, messageKey))
  prettyError.originError = error

  if (
    error.response ||
    error.code || (
      error.message && (
        error.message.match(/\s(aws: command not found)/) ||
        error.message.match(/\W(gcloud: No such file or directory)/)
      )
    )
  ) {
    prettyError.sentryIgnore = true
  }

  return prettyError
}
