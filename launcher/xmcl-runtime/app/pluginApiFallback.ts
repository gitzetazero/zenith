import { isBlockedUpstreamUrl } from '~/zenith/blockedHosts'
import { getZenithConfig } from '~/zenith/config'
import type { LauncherAppPlugin } from './LauncherAppPlugin'
import type { Handler } from './LauncherProtocolHandler'

export const pluginApiFallback: LauncherAppPlugin = (app) => {
  const handler: Handler = async ({ request, response }) => {
    if (request.url.host === 'api.curseforge.com') {
      request.headers['x-api-key'] = getZenithConfig('curseforgeApiKey')
    }
    // zenith: refuse XMCL-hosted services and telemetry, also from the renderer
    if (isBlockedUpstreamUrl(request.url.href)) {
      response.status = 403
      response.handled = true
    }
  }
  app.protocol.registerHandler('https', handler)
}
