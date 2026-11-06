import { mkdtempSync, writeFileSync } from 'fs'
import { tmpdir } from 'os'
import { join } from 'path'
import { afterEach, describe, expect, it } from 'vitest'
import { getZenithConfig, setZenithConfigDirectory } from './config'

describe('zenith config', () => {
  afterEach(() => {
    delete process.env.ZENITH_MSA_CLIENT_ID
    setZenithConfigDirectory('')
  })
  it('is empty by default (no XMCL identity is embedded)', () => {
    expect(getZenithConfig('msaClientId')).toBe('')
  })
  it('prefers the environment over the config file', () => {
    const dir = mkdtempSync(join(tmpdir(), 'zenith-config-'))
    writeFileSync(join(dir, 'zenith-config.json'), JSON.stringify({ msaClientId: 'from-file' }))
    setZenithConfigDirectory(dir)
    expect(getZenithConfig('msaClientId')).toBe('from-file')
    process.env.ZENITH_MSA_CLIENT_ID = ' from-env '
    expect(getZenithConfig('msaClientId')).toBe('from-env')
  })
})
