/* eslint-disable */
import querystring from 'querystring'
import { WebSocketHandler } from '@kubernetes/client-node/dist/web-socket-handler'
import WebSocket from 'isomorphic-ws'

WebSocketHandler.restartableHandleStandardInput = async function (createWS, stdin, streamNum = 0) {
  const tryLimit = 3;
  let queue = Promise.resolve()
  let ws = null

  async function processData(data) {
    const buff = Buffer.alloc(data.length + 1);

    buff.writeInt8(streamNum, 0);
    if (data instanceof Buffer) {
      data.copy(buff, 1);
    } else {
      buff.write(data, 1);
    }

    let i = 0;
    for (; i < tryLimit; ++i) {
      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(buff);
        break;
      } else {
        ws = await createWS()
      }
    }

    if (i >= tryLimit) {
      throw new Error("can't send data to ws")
    }
  }

  stdin.on('data', (data) => {
    queue = queue.then(processData(data))
  })

  stdin.on('end', () => {
    ws.close();
  });
}

export function patchForward (forward) {
  forward.portForward = async function (namespace, podName, targetPorts, output, err, input) {
    if (targetPorts.length === 0) {
      throw new Error('You must provide at least one port to forward to.')
    }
    if (targetPorts.length > 1) {
      throw(new Error('Only one port is currently supported for port-forward'))
    }
    const query = {
      ports: targetPorts[0]
    }
    const queryStr = querystring.stringify(query)
    const path = `/api/v1/namespaces/${namespace}/pods/${podName}/portforward?${queryStr}`

    const createWebSocket = async () => {
      const needsToReadPortNumber = []
      targetPorts.forEach((value, index) => {
        needsToReadPortNumber[index * 2] = true
        needsToReadPortNumber[index * 2 + 1] = true
      })

      return await this.handler.connect(path, null, (streamNum, buff) => {
        if (streamNum >= targetPorts.length * 2) {
          return !this.disconnectOnErr
        }
        // First two bytes of each stream are the port number
        if (needsToReadPortNumber[streamNum]) {
          buff = buff.slice(2)
          needsToReadPortNumber[streamNum] = false
        }
        if (streamNum % 2 === 1) {
          if (err) {
            err.write(buff)
          }
        } else {
          output.write(buff)
        }
        return true
      })
    }

    await WebSocketHandler.restartableHandleStandardInput(createWebSocket, input, 0)
  }
}
/* eslint-enable */
