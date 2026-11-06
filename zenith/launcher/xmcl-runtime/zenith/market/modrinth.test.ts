import { readFileSync } from 'fs'
import { join } from 'path'
import { describe, expect, test } from 'vitest'
import { ZENITH_MINECRAFT } from '../instance'
import { facets, modrinthHasKind, parseHit, parseVersion, pickVersion, primaryFile, searchUrl } from './modrinth'
import { modrinthSupport, newestRelease } from './support'

// Recorded from https://api.modrinth.com/v2 (search for resource packs and Java mods, data packs, and one project's versions).
const recorded = JSON.parse(readFileSync(join(__dirname, 'fixtures', 'modrinth.recorded.json'), 'utf-8'))
const GAME = ZENITH_MINECRAFT

describe('modrinth client', () => {
  test('search facets: kind, data pack category, Zenith version unless everything', () => {
    expect(JSON.parse(facets({ text: '', kind: 'resourcepack' }, GAME))).toEqual([['project_type:resourcepack'], [`versions:${GAME}`]])
    expect(JSON.parse(facets({ text: '', kind: 'datapack' }, GAME))).toEqual([['project_type:mod'], ['categories:datapack'], [`versions:${GAME}`]])
    expect(JSON.parse(facets({ text: '', kind: 'shader', everything: true }, GAME))).toEqual([['project_type:shader']])
  })

  test('search url is https against api.modrinth.com and carries the query', () => {
    const url = new URL(searchUrl({ text: 'fresh animations', kind: 'resourcepack', limit: 500 }, GAME))
    expect(url.origin + url.pathname).toBe('https://api.modrinth.com/v2/search')
    expect(url.searchParams.get('query')).toBe('fresh animations')
    expect(url.searchParams.get('limit')).toBe('100')
    expect(url.searchParams.get('index')).toBe('relevance')
  })

  test('Java mods are only searched with everything', () => {
    expect(modrinthHasKind('mod')).toBe(false)
    expect(modrinthHasKind('mod', true)).toBe(true)
    expect(modrinthHasKind('resourcepack')).toBe(true)
    expect(modrinthHasKind('native-mod', true)).toBe(false)
  })

  test('resource pack hits: Works for 26.3, Partly for older packs', () => {
    const hits = recorded.search.rp.hits.map((h: any) => parseHit(h, 'resourcepack', GAME))
    expect(hits.every((h: any) => h && h.source === 'modrinth' && h.kind === 'resourcepack')).toBe(true)
    for (const h of hits) {
      expect(h.support.level).toBe(h.gameVersions.includes(GAME) ? 'works' : 'partial')
      expect(h.installable).toBe(true)
    }
  })

  test('Java mod hits do not run and are not installable', () => {
    const hit = parseHit(recorded.search.mod.hits[0], 'mod', GAME)!
    expect(hit.support.level).toBe('no')
    expect(hit.support.reason).toMatch(/Java/)
    expect(hit.installable).toBe(false)
  })

  test('data pack hits are Partly with what Zenith runs said', () => {
    const hit = parseHit(recorded.search.dp.hits[0], 'datapack', GAME)!
    expect(hit.kind).toBe('datapack')
    expect(hit.support.level).toBe('partial')
    expect(hit.support.reason).toMatch(/functions/)
    expect(hit.page).toContain('/datapack/')
  })

  test('versions parse with SHA-512 and the version to install is for Zenith', () => {
    const versions = recorded.versions.map(parseVersion)
    expect(versions[0].files[0].sha512).toMatch(/^[0-9a-f]{128}$/)
    expect(primaryFile(versions[0])?.url.startsWith('https://cdn.modrinth.com/')).toBe(true)
    const picked = pickVersion('resourcepack', versions, GAME)!
    expect(picked.gameVersions).toContain(GAME)
    // data pack versions must carry the datapack loader
    expect(pickVersion('datapack', versions, GAME)).toBeUndefined()
  })

  test('support helpers', () => {
    expect(newestRelease(['1.21.4', '26.2', '26.10', '1.8.9', '26.3-pre1'])).toBe('26.10')
    expect(modrinthSupport('shader', [GAME], ['minecraft'], GAME).level).toBe('partial')
    expect(modrinthSupport('mod', [GAME], ['fabric'], GAME).level).toBe('no')
  })
})
