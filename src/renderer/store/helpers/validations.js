/* eslint-disable no-unused-vars */
import Ajv from 'ajv'

const ajv = new Ajv()

export function commitIfValid(commit, mutationName, item, validate) {
  const valid = validate(item)

  if (valid) {
    commit(mutationName, item)
  }

  return { success: valid, item, errors: validate.errors }
}

export function createValidate(schema) {
  return ajv.compile(schema)
}

export function createPick(schema) {
  const keys = Object.keys(schema.properties)

  return function(object) {
    const result = {}

    for (const key of keys) {
      result[key] = object[key]
    }

    return result
  }
}

export function createToolset(schema) {
  return {
    validate: createValidate(schema),
    pick: createPick(schema)
  }
}
