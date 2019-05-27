import { promises as fs } from 'fs'
import { omit, pick, intersection } from 'lodash'
import Ajv from 'ajv'

import { serviceSchema } from '../store/modules/Services'
import { clusterSchema } from '../store/modules/Clusters'

export function saveObjectToJsonFile(object, filename) {
  return fs.writeFile(filename, JSON.stringify(object))
}

export async function readObjectFromJsonFile(filename) {
  const data = await fs.readFile(filename, { encoding: 'utf8' })
  return JSON.parse(data)
}

const clusterFields = ['name']
const serviceFields = ['alias', 'namespace', 'workloadType', 'workloadName', 'forwards']

export function exportCluster(state, clusterId, options = {}) {
  const { includeConfig = false } = options

  const cFields = [...clusterFields]
  if (includeConfig) cFields.push('config')

  return {
    _clusters: [
      {
        ...pick(state.Clusters.items[clusterId], cFields),
        _services: Object.values(state.Services.items)
          .filter(service => service.clusterId === clusterId)
          .map(service => pick(service, serviceFields))
      }
    ]
  }
}

const ajv = new Ajv()
const validate = ajv.compile({
  type: 'object',
  required: ['_clusters'],
  properties: {
    _clusters: {
      type: 'array',
      minItems: 1,
      maxItems: 1,
      items: {
        ...clusterSchema,
        required: [...clusterFields, '_services'],
        properties: {
          ...clusterSchema.properties,
          _services: {
            type: 'array',
            items: {
              ...serviceSchema,
              required: intersection(serviceSchema.required, serviceFields)
            }
          }
        }
      }
    }
  }
})

export function importCluster(object) {
  const valid = validate(object)
  if (!valid) throw new Error(`Data for import is invalid. Raw error message: ${JSON.stringify(validate.errors)}`)

  const clusterObject = object._clusters[0]
  const cluster = omit(clusterObject, '_services')
  const services = clusterObject._services

  return [cluster, services]
}
