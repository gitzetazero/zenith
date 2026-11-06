import { existsSync, mkdtempSync, readFileSync, writeFileSync } from 'fs'
import { tmpdir } from 'os'
import { join } from 'path'
import { describe, expect, it } from 'vitest'
import { ensureZenithInstance, zenithHome } from './instance'

function setup() {
  const dir = mkdtempSync(join(tmpdir(), 'zenith-instance-'))
  const bridgeExe = join(dir, 'bridge.exe')
  writeFileSync(bridgeExe, 'bridge-bytes')
  const created: any[] = []
  const instances = {
    instances: [] as { name: string }[],
    async createInstance(o: Record<string, unknown>) {
      created.push(o)
      instances.instances.push({ name: String(o.name) })
      return join(dir, 'instances', String(o.name))
    },
  }
  return { dir, bridgeExe, created, instances, markerPath: join(dir, 'marker.json'), home: join(dir, 'zenith') }
}

describe('ensureZenithInstance', () => {
  it('creates the instance once, with the shim as java', async () => {
    const s = setup()
    const run = () => ensureZenithInstance({ ...s, log: () => {} })
    expect(await run()).toBe('created')
    expect(s.created).toHaveLength(1)
    expect(s.created[0].name).toBe('Zenith')
    expect(s.created[0].java).toBe(join(s.home, 'java', 'bin', 'java.exe'))
    expect(readFileSync(join(s.home, 'java', 'bin', 'javaw.exe'), 'utf-8')).toBe('bridge-bytes')
    expect(await run()).toBe('exists')
    expect(s.created).toHaveLength(1)
  })
  it('does not recreate a deleted instance', async () => {
    const s = setup()
    await ensureZenithInstance({ ...s, log: () => {} })
    s.instances.instances.length = 0
    expect(await ensureZenithInstance({ ...s, log: () => {} })).toBe('skipped')
    expect(s.created).toHaveLength(1)
  })
  it('does nothing without the bridge', async () => {
    const s = setup()
    expect(await ensureZenithInstance({ ...s, bridgeExe: join(s.dir, 'missing.exe'), log: () => {} })).toBe('no-bridge')
    expect(existsSync(s.markerPath)).toBe(false)
  })
  it('resolves the home like the bridge', () => {
    expect(zenithHome({ ZENITH_HOME: 'X' } as any, 'A')).toBe('X')
    expect(zenithHome({} as any, 'A')).toBe(join('A', 'zenith'))
  })
})
