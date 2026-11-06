import { createHash, generateKeyPairSync, sign } from 'crypto'
import { existsSync, mkdtempSync, readFileSync, readdirSync } from 'fs'
import { tmpdir } from 'os'
import { join } from 'path'
import { createHash as hash } from 'crypto'
import { blake2b512Js } from './blake2b'
import { describe, expect, test } from 'vitest'
import { destinationFor, downloadChecked, githubReleaseAsset, refuse, safeName } from './install'
import { parsePublicKey, parseSignature, verifyMinisign } from './minisign'
import { USER_AGENT_CONTACT, userAgent } from './userAgent'

const bytes = Buffer.from('zenith content mod')
const sha256 = createHash('sha256').update(bytes).digest('hex')
const okFetch = (body: Buffer) => async () => ({ ok: true, status: 200, arrayBuffer: async () => body.buffer.slice(body.byteOffset, body.byteOffset + body.length) as ArrayBuffer })
const REG = 'https://github.com/horizon-rust/horizon-mods/releases/download/compass_bar-0.1.0/compass_bar-0.1.0.hmod'

describe('safe downloads', () => {
  test('hosts: Modrinth CDN, the registry releases and project release assets only', () => {
    const base = { name: 'a.zip', sha256 }
    expect(refuse({ ...base, url: REG })).toBeUndefined()
    expect(refuse({ ...base, url: 'https://cdn.modrinth.com/data/x/versions/y/a.zip' })).toBeUndefined()
    expect(refuse({ ...base, url: 'https://github.com/o/r/releases/download/v1/a.zip' })).toBeUndefined()
    expect(refuse({ ...base, url: 'http://cdn.modrinth.com/a.zip' })).toMatch(/not on/)
    expect(refuse({ ...base, url: 'https://evil.example/a.zip' })).toMatch(/not on/)
    expect(refuse({ ...base, url: 'https://github.com/o/r/archive/main.zip' })).toMatch(/not on/)
    expect(githubReleaseAsset('https://github.com/o/r/releases/download/../a')).toBe(false)
  })

  test('plain file names and a checksum are required', () => {
    for (const name of ['../a.zip', 'a/b.zip', '.hidden', 'a:b', '']) expect(safeName(name)).toBeUndefined()
    expect(safeName('pack-1.0.zip')).toBe('pack-1.0.zip')
    expect(refuse({ url: REG, name: 'a.zip' })).toMatch(/no checksum/)
    expect(refuse({ url: REG, name: '../a.zip', sha256 })).toMatch(/plain file name/)
  })

  test('writes the file only when size and checksum match', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'zenith-dl-'))
    const done = await downloadChecked(okFetch(bytes), { url: REG, name: 'compass_bar-0.1.0.hmod', size: bytes.length, sha256 }, dir, userAgent('1'))
    expect(readFileSync(done.path)).toEqual(bytes)
    expect(readdirSync(dir)).toEqual(['compass_bar-0.1.0.hmod'])
  })

  test('a checksum or size mismatch installs nothing and leaves no part file', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'zenith-dl-'))
    await expect(downloadChecked(okFetch(bytes), { url: REG, name: 'a.hmod', sha256: 'f'.repeat(64) }, dir, 'ua')).rejects.toThrow(/does not match/)
    await expect(downloadChecked(okFetch(bytes), { url: REG, name: 'a.hmod', size: 3, sha256 }, dir, 'ua')).rejects.toThrow(/expected 3 bytes/)
    expect(existsSync(join(dir, 'a.hmod'))).toBe(false)
    expect(readdirSync(dir)).toEqual([])
  })

  test('the User-Agent names the launcher and the Zenith repository', () => {
    expect(userAgent('0.73.1-zenith.2')).toBe('Zenith-Launcher/0.73.1-zenith.2 (github.com/gitzetazero/zenith)')
    expect(USER_AGENT_CONTACT).toBe('github.com/gitzetazero/zenith')
  })

  test('destinations: horizon-mods, shaderpacks, resourcepacks, a world\'s datapacks; native mods and modpacks are refused', () => {
    const inst = join('x', 'instances', 'Zenith')
    expect(destinationFor('mod', inst)).toBe(join(inst, 'horizon-mods'))
    expect(destinationFor('shader', inst)).toBe(join(inst, 'shaderpacks'))
    expect(destinationFor('resourcepack', inst)).toBe(join(inst, 'resourcepacks'))
    expect(destinationFor('datapack', inst, 'My World')).toBe(join(inst, 'saves', 'My World', 'datapacks'))
    expect(() => destinationFor('datapack', inst)).toThrow(/world/)
    expect(() => destinationFor('datapack', inst, '../x')).toThrow(/world/)
    expect(() => destinationFor('native-mod', inst)).toThrow(/rebuild/)
    expect(() => destinationFor('modpack', inst)).toThrow(/modpack/)
  })
})

describe('blake2b (JS fallback)', () => {
  test('matches node:crypto on empty, short, block-sized and multi-block inputs', () => {
    for (const n of [0, 1, 127, 128, 129, 300, 1000]) {
      const data = Buffer.alloc(n, 7)
      for (let i = 0; i < n; i++) data[i] = (i * 31 + 5) & 0xff
      expect(Buffer.from(blake2b512Js(data)).toString('hex')).toBe(hash('blake2b512').update(data).digest('hex'))
    }
  })
})

describe('minisign', () => {
  // Builds a minisign key and signatures with node:crypto, in minisign's formats.
  const { publicKey, privateKey } = generateKeyPairSync('ed25519')
  const rawPub = (publicKey.export({ format: 'der', type: 'spki' }) as Buffer).subarray(-32)
  const keyId = Buffer.from('0123456789abcdef', 'hex')
  const keyText = Buffer.concat([Buffer.from('Ed'), keyId, rawPub]).toString('base64')
  const sigText = (alg: 'Ed' | 'ED', data: Buffer, id = keyId) => {
    const msg = alg === 'ED' ? Buffer.from(hash('blake2b512').update(data).digest()) : data
    const s = sign(null, msg, privateKey)
    return `untrusted comment: signature\n${Buffer.concat([Buffer.from(alg), id, s]).toString('base64')}\ntrusted comment: t\nxx\n`
  }

  test('verifies prehashed and legacy signatures', () => {
    expect(verifyMinisign(keyText, sigText('ED', bytes), bytes)).toBe(true)
    expect(verifyMinisign(keyText, sigText('Ed', bytes), bytes)).toBe(true)
  })

  test('refuses a changed file, another key id and garbage', () => {
    expect(verifyMinisign(keyText, sigText('ED', bytes), Buffer.from('changed'))).toBe(false)
    expect(verifyMinisign(keyText, sigText('ED', bytes, Buffer.from('ffffffffffffffff', 'hex')), bytes)).toBe(false)
    expect(verifyMinisign(keyText, 'nonsense', bytes)).toBe(false)
    expect(verifyMinisign('nonsense', sigText('ED', bytes), bytes)).toBe(false)
  })

  test('parses the real key and signature recorded in the registry index (same key id)', () => {
    const index = JSON.parse(readFileSync(join(__dirname, 'fixtures', 'registry-index.recorded.json'), 'utf-8'))
    const p = index.projects.find((p: any) => p.key)
    const key = parsePublicKey(p.key)
    const sig = parseSignature(p.versions[0].file.signature)
    expect(sig.algorithm).toBe('ED')
    expect(sig.keyId.equals(key.keyId)).toBe(true)
  })
})
