import { readFile } from 'fs-extra'
import { join } from 'path'
import { LauncherAppPlugin } from '~/app'
import { kFlights } from '../flights'

const BUILTIN_FLIGHTS = {
}

export const DEVELOPMENT_XMCL_BILLING_API_BASE_URL = 'https://api-staging.xmcl.disabled.invalid'

export function applyDevelopmentApiFlight(
  flights: Record<string, any>,
  environment = process.env.NODE_ENV,
) {
  if (environment === 'development') {
    flights.xmclBillingApiBaseUrl = DEVELOPMENT_XMCL_BILLING_API_BASE_URL
  }
  return flights
}

export function applyRemoteFlights(
  flights: Record<string, any>,
  remote: Record<string, any>,
  environment = process.env.NODE_ENV,
) {
  Object.assign(flights, remote)
  const persisted = { ...flights }
  if (
    environment === 'development' &&
    !Object.hasOwn(remote, 'xmclBillingApiBaseUrl') &&
    persisted.xmclBillingApiBaseUrl === DEVELOPMENT_XMCL_BILLING_API_BASE_URL
  ) {
    delete persisted.xmclBillingApiBaseUrl
  }
  applyDevelopmentApiFlight(flights, environment)
  return persisted
}

export const pluginFlights: LauncherAppPlugin = async (app) => {
  const logger = app.getLogger('Flights')
  // zenith: remote flights came from XMCL's servers; Zenith Launcher never fetches them (only a cached flights.json is read)
  const readCachedFlights = async (output: Record<string, any>, cachedPath: string) => {
    try {
      const cached = JSON.parse(await readFile(cachedPath, 'utf-8'))
      for (const [k, v] of Object.entries(cached)) {
        output[k] = v
      }
      applyDevelopmentApiFlight(output)
      return false
    } catch {
      return true
    }
  }
  const readFlights = async (output: Record<string, any>, cachedPath: string) => {
    await readCachedFlights(output, cachedPath)
  }
  try {
    const filtered = applyDevelopmentApiFlight({ ...BUILTIN_FLIGHTS }) as Record<string, string>
    const cachedPath = join(app.appDataPath, 'flights.json')
    const promise = readFlights(filtered, cachedPath).then(() => {
      logger.log('Flights loaded', JSON.stringify(filtered))
    })

    app.protocol.registerHandler('http', async ({ request, response }) => {
      if (request.url.host === 'launcher' && request.url.pathname === '/flights') {
        await promise
        response.status = 200
        const jsContent = `window.flights = ${JSON.stringify(filtered)}`
        response.headers = {
          'content-type': 'application/javascript',
        }
        response.body = jsContent
      }
    })

    app.registry.register(kFlights, filtered)
  } catch {
    app.registry.register(kFlights, { ...BUILTIN_FLIGHTS })
  }
}
