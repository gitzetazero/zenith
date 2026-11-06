import { describe, expect, it } from 'vitest'
import { isBlockedUpstreamUrl } from './blockedHosts'

describe('isBlockedUpstreamUrl', () => {
  it('blocks disabled origins and telemetry hosts', () => {
    expect(isBlockedUpstreamUrl('https://api.xmcl.disabled.invalid/flights')).toBe(true)
    expect(isBlockedUpstreamUrl(new URL('https://eastasia-0.in.applicationinsights.azure.com/v2/track'))).toBe(true)
  })
  it('lets normal traffic through', () => {
    expect(isBlockedUpstreamUrl('https://github.com/gitzetazero/zenith')).toBe(false)
    expect(isBlockedUpstreamUrl('not a url')).toBe(false)
  })
})
