/* eslint-disable no-unused-vars */
import yaml from 'js-yaml'

const migrations = [
  {
    from: 1,
    to: 2,
    mutate(state) {
      for (const cluster of Object.values(state.Clusters.items)) {
        let currentContext = null
        try {
          currentContext = yaml.safeLoad(cluster.config)['current-context']
        } catch (error) {
        }

        cluster.config = {
          currentContext,
          storingMethod: 'content',
          content: cluster.config,
          path: null
        }
      }
    }
  }
]

export default function(state) {
  for (const migration of migrations) {
    if (migration.from === state.version) {
      migration.mutate(state)
      state.version = migration.to
    }
  }
}
