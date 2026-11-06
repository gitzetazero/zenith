import { describe, expect, it, vi } from 'vitest'
import { MicrosoftOAuthClient } from '~/user/accountSystems/MicrosoftOAuthClient'
import { MSA_NOT_CONFIGURED_MESSAGE } from './config'

describe('Microsoft sign-in without a configured client id', () => {
  it('fails with the honest message and never contacts Microsoft', async () => {
    const fetch = vi.fn()
    const client = new MicrosoftOAuthClient(
      fetch as any,
      { log() {}, warn() {}, error() {} } as any,
      '',
      async () => '',
      async () => '',
      () => {},
      {} as any,
    )
    await expect(client.authenticate('', ['XboxLive.signin'])).rejects.toThrow(MSA_NOT_CONFIGURED_MESSAGE)
    expect(fetch).not.toHaveBeenCalled()
    expect(MSA_NOT_CONFIGURED_MESSAGE).toContain('docs/ZENITH-LAUNCHER.md')
  })
})
