import { LauncherApp } from '~/app'
import { AnyError } from '@xmcl/utils'
import { UserService } from '../UserService'
import { ExternalCredentialService } from '~/credential/ExternalCredentialService'

export function formatModrinthAuthorization(accessToken: string, tokenType = 'Bearer') {
  if (/^[A-Za-z][\w-]*\s+/.test(accessToken)) return accessToken
  return `${tokenType || 'Bearer'} ${accessToken}`
}

export async function getModrinthAccessToken(
  app: LauncherApp,
  credentials?: ExternalCredentialService,
) {
  const service = credentials ?? (await app.registry.getOrCreate(ExternalCredentialService))
  const result = await service.getValidAccessToken('modrinth')
  return result.status === 'valid' ? result.accessToken : undefined
}

export function createModrinthAuthenticatedFetch(app: LauncherApp): typeof fetch {
  return async (input, init) => {
    const headers = new Headers(input instanceof Request ? input.headers : undefined)
    new Headers(init?.headers).forEach((value, key) => headers.set(key, value))
    const token = await getModrinthAccessToken(app)
    if (token) headers.set('Authorization', formatModrinthAuthorization(token))
    return await app.fetch(input, { ...init, headers })
  }
}

export async function loginModrinth(
  app: LauncherApp,
  _userService: UserService,
  _scopes: string[],
  invalidate: boolean,
  _signal?: AbortSignal,
  credentials?: ExternalCredentialService,
) {
  const credentialService =
    credentials ?? (await app.registry.getOrCreate(ExternalCredentialService))
  const token = invalidate ? undefined : await getModrinthAccessToken(app, credentialService)

  if (token) return

  // zenith: XMCL's Modrinth OAuth app and its token-exchange backend are not carried over, and Zenith has no backend
  // holding a client secret. The Market uses Modrinth anonymously; signing in is unavailable.
  throw new AnyError(
    'ZenithModrinthLoginUnavailable',
    'Modrinth sign-in is not available in Zenith Launcher. The Zenith Market works without signing in.',
  )
}
