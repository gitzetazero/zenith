import { describe, expect, it, vi } from 'vitest'
import { ExternalCredentialService } from '~/credential/ExternalCredentialService'
import { loginModrinth } from './loginModrinth'
import { createModrinthAuthenticatedFetch, formatModrinthAuthorization } from './loginModrinth'

function createStorage() {
  const values = new Map<string, string>()
  return {
    values,
    get: vi.fn(async (service: string, account: string) => values.get(`${service}/${account}`)),
    put: vi.fn(async (service: string, account: string, value: string) => {
      values.set(`${service}/${account}`, value)
    }),
  }
}

describe('loginModrinth', () => {
  it('refuses to sign in: Zenith has no Modrinth OAuth exchange service', async () => {
    await expect(loginModrinth({ registry: { getOrCreate: async () => ({ getValidAccessToken: async () => ({ status: 'missing' }) }) } } as any, {} as any, [], false))
      .rejects.toMatchObject({ name: 'ZenithModrinthLoginUnavailable' })
  })
})
describe('formatModrinthAuthorization', () => {
  it('uses the Bearer scheme for OAuth access tokens', () => {
    expect(formatModrinthAuthorization('mrp_oauth_token')).toBe('Bearer mrp_oauth_token')
  })

  it('preserves an explicitly formatted authorization value', () => {
    expect(formatModrinthAuthorization('Bearer mrp_oauth_token')).toBe('Bearer mrp_oauth_token')
  })

  it('injects the latest OAuth token into runtime fetch requests', async () => {
    let requestHeaders: Headers | undefined
    const app = {
      secretStorage: {
        get: async () => JSON.stringify({
          access_token: 'mrp_oauth_token',
          token_type: 'Bearer',
          expires_in: 3600,
          issued_at: Date.now(),
        }),
      },
      registry: {
        getOrCreate: async () => ({
          getValidAccessToken: async () => ({
            status: 'valid',
            accessToken: 'mrp_oauth_token',
          }),
        }),
      },
      fetch: async (_input: RequestInfo | URL, init?: RequestInit) => {
        requestHeaders = new Headers(init?.headers)
        return new Response('{}', { status: 200 })
      },
    }

    await createModrinthAuthenticatedFetch(app as any)('https://api.modrinth.com/v2/project', {
      headers: { 'x-test': 'preserved' },
    })

    expect(requestHeaders?.get('authorization')).toBe('Bearer mrp_oauth_token')
    expect(requestHeaders?.get('x-test')).toBe('preserved')
  })
})
