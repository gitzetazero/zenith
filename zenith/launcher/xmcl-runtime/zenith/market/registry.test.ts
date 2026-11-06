import { mkdtempSync, readFileSync, writeFileSync } from 'fs'
import { tmpdir } from 'os'
import { join } from 'path'
import { describe, expect, test, vi } from 'vitest'
import { ZENITH_MINECRAFT } from '../instance'
import { createIndexLoader, kindOf, parseIndex, projectOf, revokedReason, searchIndex, versionsOf, REGISTRY_INDEX_URL } from './registry'
import { userAgent } from './userAgent'

// Recorded from https://horizon-rust.github.io/horizon-mods/index.json (4 projects, one of each kind, 1 revocation).
const recorded = readFileSync(join(__dirname, 'fixtures', 'registry-index.recorded.json'), 'utf-8')
const GAME = ZENITH_MINECRAFT

describe('registry index', () => {
  const index = parseIndex(recorded)

  test('parses the recorded index', () => {
    expect(index.schema).toBe(1)
    expect(index.projects.map((p) => p.kind).sort()).toEqual(['mod', 'native-mod', 'resourcepack', 'shader'])
    expect(index.revoked[0].id).toBe('crossversion')
  })

  test('refuses a newer schema and a non-index', () => {
    expect(() => parseIndex(JSON.stringify({ schema: 2, projects: [] }))).toThrow(/update Zenith Launcher/)
    expect(() => parseIndex('{"nope":1}')).toThrow(/not an index/)
  })

  test('drops malformed entries, keeps unknown kinds but never offers them', () => {
    const synthetic = JSON.stringify({
      schema: 1,
      projects: [
        { id: 'BAD ID', kind: 'mod', name: 'x', versions: [] },
        { id: 'theme_x', kind: 'theme', name: 'A theme', summary: '', description: '', authors: [], license: 'MIT', source: 'https://x', versions: [{ version: '1', published: '2026-01-01', game: GAME, file: { name: 'a.zip', url: 'https://github.com/a/b/releases/download/t/a.zip', size: 1, sha256: 'a'.repeat(64) } }] },
      ],
    })
    const parsed = parseIndex(synthetic)
    expect(parsed.projects.map((p) => p.id)).toEqual(['theme_x'])
    expect(kindOf('theme')).toBeUndefined()
    expect(searchIndex(parsed, { text: '', kind: 'mod', everything: true }, GAME)).toEqual([])
    expect(projectOf(parsed.projects[0], GAME)).toBeUndefined()
  })

  test('types: content mod, native mod (requires rebuild), shader, resource pack', () => {
    const byKind = (kind: any) => searchIndex(index, { text: '', kind }, GAME)
    const mod = byKind('mod')[0]
    expect(mod).toMatchObject({ id: 'compass_bar', source: 'zenith', installable: true, support: { level: 'works' } })
    const native = byKind('native-mod')[0]
    expect(native).toMatchObject({ requiresRebuild: true, installable: false, support: { level: 'works' } })
    expect(native.support.reason).toMatch(/rebuild/)
    expect(byKind('shader')[0]).toMatchObject({ id: 'daybreak', installable: true })
    expect(byKind('resourcepack')[0]).toMatchObject({ id: 'menu_panorama', installable: true })
    expect(byKind('datapack')).toEqual([])
  })

  test('search by words and hides projects not made for the Zenith game unless everything', () => {
    expect(searchIndex(index, { text: 'compass', kind: 'mod' }, GAME).map((p) => p.id)).toEqual(['compass_bar'])
    expect(searchIndex(index, { text: 'zzz-no-match', kind: 'mod' }, GAME)).toEqual([])
    expect(searchIndex(index, { text: '', kind: 'mod' }, '99.9')).toEqual([])
    const all = searchIndex(index, { text: '', kind: 'mod', everything: true }, '99.9')
    expect(all[0].support).toMatchObject({ level: 'no' })
    expect(all[0].installable).toBe(false)
  })

  test('versions: the one an install would take, and takedowns', () => {
    const p = index.projects.find((p) => p.id === 'compass_bar')!
    const v = versionsOf(index, p, GAME)
    expect(v[0]).toMatchObject({ picked: true, fileName: 'compass_bar-0.1.0.hmod' })
    const sha = p.versions[0].file.sha256
    const revoked = { ...index, revoked: [{ id: 'compass_bar', sha256: sha, reason: 'bad' }] }
    expect(revokedReason(revoked, 'compass_bar', sha)).toBe('taken down: bad')
    expect(versionsOf(revoked, p, GAME)[0]).toMatchObject({ blocked: 'taken down: bad' })
    expect(versionsOf(revoked, p, GAME)[0].picked).toBeUndefined()
  })
})

describe('index loader', () => {
  const ok = (text: string) => ({ ok: true, status: 200, text: async () => text })

  test('fetches over https with the descriptive User-Agent, caches in memory and on disk', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'zenith-market-'))
    const cacheFile = join(dir, 'c', 'registry-index.json')
    const fetch = vi.fn(async (_url: string, _init?: any) => ok(recorded))
    let now = 1000
    const load = createIndexLoader({ fetch, userAgent: userAgent('0.73.1-zenith.2'), cacheFile, now: () => now })
    await load()
    await load()
    expect(fetch).toHaveBeenCalledTimes(1)
    expect(fetch.mock.calls[0][0]).toBe(REGISTRY_INDEX_URL)
    expect(fetch.mock.calls[0][1].headers['user-agent']).toBe('Zenith-Launcher/0.73.1-zenith.2 (github.com/gitzetazero/zenith)')
    expect(readFileSync(cacheFile, 'utf-8')).toBe(recorded)
    now += 11 * 60 * 1000
    await load()
    expect(fetch).toHaveBeenCalledTimes(2)
  })

  test('falls back to the copy kept last time when the registry is down', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'zenith-market-'))
    const cacheFile = join(dir, 'registry-index.json')
    writeFileSync(cacheFile, recorded)
    const load = createIndexLoader({ fetch: async () => ({ ok: false, status: 503, text: async () => '' }), userAgent: 'x', cacheFile })
    expect((await load()).projects).toHaveLength(4)
    const none = createIndexLoader({ fetch: async () => ({ ok: false, status: 503, text: async () => '' }), userAgent: 'x', cacheFile: join(dir, 'missing.json') })
    await expect(none()).rejects.toThrow(/503/)
  })

  test('refuses a registry address that is not https', () => {
    expect(() => createIndexLoader({ fetch: async () => ok(''), userAgent: 'x', cacheFile: 'x', url: 'http://example.com/index.json' })).toThrow(/https/)
  })
})
