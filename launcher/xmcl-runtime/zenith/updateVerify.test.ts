import { describe, expect, test } from 'vitest'
import { INSTALLER_ASSET, assertInstallerHash, canSelfUpdate, installerSilentArgs, isAllowedInstallerUrl, isInstalledBuild, sha256FromNotes } from './updateVerify'

const H = 'a'.repeat(64)
const notes = ['Zenith build of commit 1234567.', '', 'SHA-256:', '', '```', `${H}  ${INSTALLER_ASSET}`, `${'b'.repeat(64)}  Zenith-Launcher-Portable-x64.zip`, '```', ''].join('\n')
const url = `https://github.com/gitzetazero/zenith/releases/download/nightly-2026-11-06/${INSTALLER_ASSET}`
const base = { os: 'windows', env: 'raw', installed: true, devServer: false, builtCommit: 'abcdef0', notes, files: [{ name: INSTALLER_ASSET, url }] }

test('extracts the hash of one file from the notes', () => {
  expect(sha256FromNotes(notes, INSTALLER_ASSET)).toBe(H)
  expect(sha256FromNotes(notes, 'Zenith-Launcher-Portable-x64.zip')).toBe('b'.repeat(64))
  expect(sha256FromNotes(notes, 'other.exe')).toBeUndefined()
  expect(sha256FromNotes('', INSTALLER_ASSET)).toBeUndefined()
  expect(sha256FromNotes(`${'a'.repeat(63)}  ${INSTALLER_ASSET}`, INSTALLER_ASSET)).toBeUndefined()
})

test('host allow-list accepts only the Zenith release downloads on github.com', () => {
  expect(isAllowedInstallerUrl(url)).toBe(true)
  for (const bad of [
    url.replace('https', 'http'),
    'https://evil.example/gitzetazero/zenith/releases/download/x/a.exe',
    'https://github.com.evil.example/gitzetazero/zenith/releases/download/x/a.exe',
    'https://github.com/other/zenith/releases/download/x/a.exe',
    'https://github.com/gitzetazero/zenith/releases/download/../../x.exe',
    'https://user@github.com/gitzetazero/zenith/releases/download/x/a.exe',
    'not a url',
  ]) expect(isAllowedInstallerUrl(bad), bad).toBe(false)
})

test('a hash mismatch or a missing hash is refused', () => {
  expect(() => assertInstallerHash(H.toUpperCase(), H)).not.toThrow()
  expect(() => assertInstallerHash('c'.repeat(64), H)).toThrow(/does not match/)
  expect(() => assertInstallerHash(H, undefined)).toThrow(/no SHA-256/)
})

test('silent args restart the app', () => {
  expect(installerSilentArgs()).toEqual(['/S', '--updated', '--force-run'])
})

test('installed vs portable', () => {
  expect(isInstalledBuild(undefined, ['Zenith Launcher.exe', 'Uninstall Zenith Launcher.exe'])).toBe(true)
  expect(isInstalledBuild(undefined, ['Zenith Launcher.exe'])).toBe(false)
  expect(isInstalledBuild('C:\\x\\portable.exe', ['Uninstall Zenith Launcher.exe'])).toBe(false)
})

describe('canSelfUpdate', () => {
  test('a different commit on an installed Windows build', () => expect(canSelfUpdate(base)).toBe(true))
  test('the same commit never updates (loop guard)', () => expect(canSelfUpdate({ ...base, builtCommit: '1234567' })).toBe(false))
  test('unknown commits never update', () => {
    expect(canSelfUpdate({ ...base, builtCommit: '' })).toBe(false)
    expect(canSelfUpdate({ ...base, notes: `${H}  ${INSTALLER_ASSET}` })).toBe(false)
  })
  test('portable, dev, non-Windows, appx stay manual', () => {
    expect(canSelfUpdate({ ...base, installed: false })).toBe(false)
    expect(canSelfUpdate({ ...base, devServer: true })).toBe(false)
    expect(canSelfUpdate({ ...base, os: 'linux' })).toBe(false)
    expect(canSelfUpdate({ ...base, env: 'appx' })).toBe(false)
  })
  test('no hash, wrong host or no asset stay manual', () => {
    expect(canSelfUpdate({ ...base, notes: 'Zenith build of commit 1234567.' })).toBe(false)
    expect(canSelfUpdate({ ...base, files: [{ name: INSTALLER_ASSET, url: 'https://evil.example/a.exe' }] })).toBe(false)
    expect(canSelfUpdate({ ...base, files: [] })).toBe(false)
  })
})
