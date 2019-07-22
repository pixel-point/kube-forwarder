const postfixes = {
  notFound: 'not found.',
  default: 'can\'t be fetched.'
}

function buildMessage(error, messages, messageKey) {
  if (messages[messageKey]) return messages[messageKey]
  if (messages._object) return `${messages._object} ${postfixes[messageKey]}`
  if (error.body) return error.body.message
  return postfixes[messageKey]
}

function getMessageKey(error) {
  if (error.response && error.response.statusCode === 404) {
    return 'notFound'
  }

  return 'default'
}

export function k8nApiPrettyError(error, messages = {}) {
  const messageKey = getMessageKey(error)
  const prettyError = new Error(buildMessage(error, messages, messageKey))

  prettyError.parentError = error
  prettyError.details = error.message

  if (error.response || error.code) {
    prettyError.sentryIgnore = true
  }

  return prettyError
}
