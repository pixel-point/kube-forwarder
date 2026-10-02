// Minimal form validation, replacing vuelidate (which the forms used only for
// required/integer/between). Each validator returns true when the
// value is VALID — matching the boolean shape ValidationErrors.vue reads
// (attribute.required/integer/between === false means "failed").

export const required = value => {
  if (value == null) return false
  if (typeof value === 'string') return value.trim().length > 0
  if (Array.isArray(value)) return value.length > 0
  return true
}

export const integer = value => {
  if (value === '' || value == null) return false
  return Number.isInteger(Number(value))
}

export const between = (min, max) => value => {
  const number = Number(value)
  return number >= min && number <= max
}

// Builds a per-field validation object: { $error, $invalid, <ruleName>: bool }.
// `$error` is only true once the field (or form) has been touched, so errors
// don't flash before the user interacts — same behaviour vuelidate's $touch gave.
export function field(value, rules, touched) {
  const result = { $invalid: false, $error: false }

  for (const [name, test] of Object.entries(rules)) {
    const valid = test(value)
    result[name] = valid
    if (!valid) result.$invalid = true
  }

  result.$error = touched && result.$invalid
  return result
}
