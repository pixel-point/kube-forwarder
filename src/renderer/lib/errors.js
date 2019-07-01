export function buildSentryIgnoredError(message) {
  const error = new Error(message)
  error.sentryIgnore = true
  return error
}
