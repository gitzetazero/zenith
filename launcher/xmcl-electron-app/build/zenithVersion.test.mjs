import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { expect, test } from 'vitest'
import { stampPackageJson, stampedVersion } from '../../tools/zenith/stamp-version.mjs'

test('a release advances the version over the XMCL base', () => {
  expect(stampedVersion('0.73.1', 25)).toBe('0.73.1-zenith0.1.25')
  expect(stampedVersion('0.73.1', 26)).not.toBe(stampedVersion('0.73.1', 25))
  expect(stampedVersion('0.80.0', 26)).toBe('0.80.0-zenith0.1.26')
  expect(() => stampedVersion('0.73', 1)).toThrow(/base/)
  expect(() => stampedVersion('0.73.1', 'x')).toThrow(/release number/)
})

test('stamping the app package keeps the base and writes the file', () => {
  const dir = mkdtempSync(join(tmpdir(), 'zenith-version-'))
  const file = join(dir, 'package.json')
  writeFileSync(file, JSON.stringify({ name: 'zenith-launcher', version: '0.73.1-zenith.2' }, null, 2))
  expect(stampPackageJson(file, 25)).toBe('0.73.1-zenith0.1.25')
  expect(JSON.parse(readFileSync(file, 'utf-8')).version).toBe('0.73.1-zenith0.1.25')
})
