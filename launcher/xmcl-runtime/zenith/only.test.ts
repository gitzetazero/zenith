import { describe, expect, test } from 'vitest'
import { join } from 'path'
import { enforceZenithInstance, ZenithOnlyError, zenithJavaPath } from './only'
import { ZENITH_MINECRAFT } from './instance'

const guard = { env: {} as NodeJS.ProcessEnv, appData: 'C:\AppData', platform: 'win32' as const }

describe('Zenith-only instance creation guard', () => {
  test('pins the Zenith Minecraft version and the bridge as java', () => {
    const out = enforceZenithInstance({ name: 'X', runtime: { minecraft: '1.12.2' } }, guard)
    expect(out.runtime).toEqual({ minecraft: ZENITH_MINECRAFT })
    expect(out.java).toBe(zenithJavaPath(guard))
    expect(out.java).toBe(join('C:\AppData', 'zenith', 'java', 'bin', 'java.exe'))
    expect(out.prependCommand).toBe('')
    expect(out.vmOptions).toEqual([])
    expect(out.maxMemory).toBe(0)
  })

  test('ZENITH_HOME is honoured', () => {
    expect(zenithJavaPath({ ...guard, env: { ZENITH_HOME: 'D:\Z' } })).toBe(join('D:\Z', 'java', 'bin', 'java.exe'))
  })

  test('refuses mod loaders, OptiFine, LabyMod, Bedrock and modpack upstreams', () => {
    for (const runtime of [{ forge: '1' }, { neoForged: '1' }, { fabricLoader: '0.1' }, { quiltLoader: '1' }, { optifine: 'x' }, { labyMod: '1' }]) {
      expect(() => enforceZenithInstance({ name: 'a', runtime }, guard)).toThrow(ZenithOnlyError)
    }
    expect(() => enforceZenithInstance({ name: 'a', edition: 'bedrock' }, guard)).toThrow(ZenithOnlyError)
    expect(() => enforceZenithInstance({ name: 'a', upstream: { type: 'modrinth-modpack' } }, guard)).toThrow(ZenithOnlyError)
  })

  test('no java path off Windows', () => {
    const out = enforceZenithInstance({ name: 'a' }, { ...guard, platform: 'linux' })
    expect('java' in out).toBe(false)
  })
})
