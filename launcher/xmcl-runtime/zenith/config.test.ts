import { mkdtempSync, writeFileSync } from 'fs'
import { tmpdir } from 'os'
import { join } from 'path'
import { afterEach, describe, expect, it } from 'vitest'
import {
  MINECRAFT_LAUNCHER_PUBLIC_CLIENT_ID,
  getZenithConfig,
  isMinecraftLauncherPublicClientId,
  setZenithConfigDirectory,
} from './config'

describe('zenith config', () => {
  afterEach(() => {
    delete process.env.ZENITH_MSA_CLIENT_ID
    setZenithConfigDirectory('')
  })
  it('defaults Microsoft sign-in to the public Minecraft launcher client id', () => {
    expect(getZenithConfig('msaClientId')).toBe(MINECRAFT_LAUNCHER_PUBLIC_CLIENT_ID)
    expect(getZenithConfig('msaClientId')).toBe('00000000402b5328')
  })
  it('recognizes the public id with braces or different case', () => {
    expect(isMinecraftLauncherPublicClientId('{00000000-402b-4cd3-a82b-c45ab2f1d3f7}')).toBe(false)
    expect(isMinecraftLauncherPublicClientId('{00000000402B5328}')).toBe(true)
    expect(isMinecraftLauncherPublicClientId('c36a9fb6-4f2a-41ff-90bd-ae7cc92031eb')).toBe(false)
  })
  it('prefers the environment over the config file and the built-in id', () => {
    const dir = mkdtempSync(join(tmpdir(), 'zenith-config-'))
    writeFileSync(join(dir, 'zenith-config.json'), JSON.stringify({ msaClientId: 'from-file' }))
    setZenithConfigDirectory(dir)
    expect(getZenithConfig('msaClientId')).toBe('from-file')
    process.env.ZENITH_MSA_CLIENT_ID = ' from-env '
    expect(getZenithConfig('msaClientId')).toBe('from-env')
  })
})
