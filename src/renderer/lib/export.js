import { promises as fs } from 'fs'
import { omit, pick, intersection } from 'lodash'
import Ajv from 'ajv'

import { serviceSchema } from '../store/modules/Services'
import { clusterSchema } from '../store/modules/Clusters'
import { CURRENT_STATE_VERSION } from '../store'
import * as configStoringMethods from '../lib/constants/config-storing-methods'

export function saveObjectToJsonFile(object, filename) {
  return fs.writeFile(filename, JSON.stringify(object))
}

export async function readObjectFromJsonFile(filename) {
  const data = await fs.readFile(filename, { encoding: 'utf8' })
  return JSON.parse(data)
}

const clusterFields = ['name']
const serviceFields = ['alias', 'namespace', 'workloadType', 'workloadName', 'forwards']

export async function exportCluster(state, clusterId, options = {}) {
  const { includeConfig = false } = options
  const cluster = state.Clusters.items[clusterId]

  const clusterObject = {
    ...pick(cluster, clusterFields),
    _services: Object.values(state.Services.items)
      .filter(service => service.clusterId === clusterId)
      .map(service => pick(service, serviceFields))
  }

  console.log(includeConfig, cluster)

  if (includeConfig) {
    const config = cluster.config

    if (config.storingMethod === configStoringMethods.PATH) {
      clusterObject.config = {
        ...omit(config, 'path'),
        storingMethod: configStoringMethods.CONTENT,
        content: await fs.readFile(config.path, { encoding: 'utf8' })
      }
    } else if (config.storingMethod === configStoringMethods.CONTENT) {
      clusterObject.config = config
    }
  }

  const exportObject = { version: CURRENT_STATE_VERSION, clusters: [clusterObject] }
  const valid = validate(exportObject)
  if (!valid) {
    throw new Error(
      'Sorry, something went wrong.' +
      `Feel free to create an issue here https://github.com/pixel-point/kube-forwarder/issues/new.` +
      `Error: ${JSON.stringify(validate.errors)}`
    )
  }

  return exportObject
}

const ajv = new Ajv()
const validate = ajv.compile({
  type: 'object',
  required: ['version', 'clusters'],
  properties: {
    version: { const: CURRENT_STATE_VERSION },
    clusters: {
      type: 'array',
      minItems: 1,
      maxItems: 1,
      items: {
        ...clusterSchema,
        required: [...clusterFields, '_services'],
        properties: {
          ...clusterSchema.properties,
          config: {
            ...clusterSchema.properties.config,
            properties: {
              ...clusterSchema.properties.config.properties,
              storingMethod: { const: configStoringMethods.CONTENT }
            }
          },
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
  if (object.version !== CURRENT_STATE_VERSION) {
    throw new Error(
      `Sorry, an export of version ${object.version || 1} isn't supported. Actual version is ${CURRENT_STATE_VERSION}`
    )
  }

  const valid = validate(object)
  if (!valid) throw new Error(`Data for import is invalid. Raw error message: ${JSON.stringify(validate.errors)}`)

  const clusterObject = object.clusters[0]
  const cluster = omit(clusterObject, '_services')
  const services = clusterObject._services

  return [cluster, services]
}
