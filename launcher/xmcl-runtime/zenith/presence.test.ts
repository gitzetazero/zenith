import { expect, test } from 'vitest'
import { buildZenithActivity, DISCORD_LOGO_ASSET, isPresenceConfigured, ZENITH_REPOSITORY_URL } from './presence'

test('presence says Playing Zenith with the Zenith logo asset and the repository button', () => {
  const a = buildZenithActivity('Browsing the Zenith Market', 1)
  expect(a.state).toBe('Playing Zenith')
  expect(a.details).toBe('Browsing the Zenith Market')
  expect(a.largeImageKey).toBe(DISCORD_LOGO_ASSET)
  expect(DISCORD_LOGO_ASSET).toBe('zenith-logo')
  expect(a.buttons).toEqual([{ label: 'Zenith on GitHub', url: ZENITH_REPOSITORY_URL }])
  expect(ZENITH_REPOSITORY_URL).toBe('https://github.com/gitzetazero/zenith')
})

test('without a valid application id the presence is disabled silently', () => {
  expect(isPresenceConfigured('')).toBe(false)
  expect(isPresenceConfigured(undefined)).toBe(false)
  expect(isPresenceConfigured('not-an-id')).toBe(false)
  expect(isPresenceConfigured('123456789012345678')).toBe(true)
})
