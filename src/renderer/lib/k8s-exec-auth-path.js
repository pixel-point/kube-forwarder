import { ExecAuth } from '@kubernetes/client-node/dist/exec_auth'
import { execSync } from 'child_process'
import { KubeConfig } from '@kubernetes/client-node'
import { CloudAuth } from '@kubernetes/client-node/dist/cloud_auth'

// Use `child_process` instead of shelljs
class PatchedExecAuth extends ExecAuth {
  getToken(user) {
    // TODO: Handle client cert auth here, requires auth refactor.
    // See https://kubernetes.io/docs/reference/access-authn-authz/authentication/#input-and-output-formats
    // for details on this protocol.
    // TODO: Add a unit test for token caching.
    const cachedToken = this.tokenCache[user.name]
    if (cachedToken) {
      const date = Date.parse(cachedToken.status.expirationTimestamp)
      if (date > Date.now()) {
        return `Bearer ${cachedToken.status.token}`
      }
      this.tokenCache[user.name] = null
    }
    let exec = null
    if (user.authProvider && user.authProvider.config) {
      exec = user.authProvider.config.exec
    }
    if (user.exec) {
      exec = user.exec
    }
    if (!exec) {
      return null
    }
    if (!exec.command) {
      throw new Error('No command was specified for exec authProvider!')
    }
    let cmd = exec.command
    if (exec.args) {
      cmd = `${cmd} ${exec.args.join(' ')}`
    }
    let opts = { silent: true }
    if (exec.env) {
      const env = process.env
      exec.env.forEach((elt) => (env[elt.name] = elt.value))
      opts = Object.assign({}, opts, { env })
    }

    // =====================
    // Patch starts here
    // =====================

    const stdout = execSync(cmd, opts, { encoding: 'utf-8' })
    const result = { code: 0, stdout }

    // =====================
    // Patch ends here
    // =====================

    if (result.code === 0) {
      const obj = JSON.parse(result.stdout)
      this.tokenCache[user.name] = obj
      return `Bearer ${obj.status.token}`
    }
    throw new Error(result.stderr)
  }
}

KubeConfig.authenticators = [new CloudAuth(), new PatchedExecAuth()]
