import { expect, test } from 'vitest'
import { commitOfNotes, isNewBuild, pickLauncherRelease, versionOfTag, ZENITH_RELEASES_API, ZENITH_RELEASES_URL } from './release'

test('the release feed is the Zenith repository, not the fork', () => {
  expect(ZENITH_RELEASES_URL).toBe('https://github.com/gitzetazero/zenith/releases')
  expect(ZENITH_RELEASES_API).toContain('api.github.com/repos/gitzetazero/zenith/releases')
  expect(ZENITH_RELEASES_API).not.toContain('zenith-launcher')
})

test('picks the newest published release carrying the launcher installer, not a game release', () => {
  const releases = [
    { tag_name: 'nightly', assets: [{ name: 'zenith-bridge-windows-x64.exe' }] },
    { tag_name: 'v0.73.1-zenith.3', draft: true, assets: [{ name: 'Zenith-Launcher-Setup-x64.exe' }] },
    { tag_name: 'v0.73.1-zenith.2', assets: [{ name: 'Zenith-Launcher-Setup-x64.exe' }, { name: 'Zenith-Launcher-Portable-x64.exe' }] },
    { tag_name: 'v0.73.1-zenith.1', assets: [{ name: 'Zenith-Launcher-Setup-x64.exe' }] },
  ]
  expect(pickLauncherRelease(releases)?.tag_name).toBe('v0.73.1-zenith.2')
  expect(pickLauncherRelease([releases[0]])).toBeUndefined()
})

test('tags with and without a prefix map to the same version', () => {
  expect(versionOfTag('v0.73.1-zenith.2')).toBe('0.73.1-zenith.2')
  expect(versionOfTag('launcher-v0.73.1-zenith.2')).toBe('0.73.1-zenith.2')
  expect(versionOfTag('v0.74.0')).toBe('0.74.0')
})

test('a dated release is a new build only when its commit differs from the running one', () => {
  const notes = 'Zenith build of commit 07165e0.

Players: download...'
  expect(commitOfNotes(notes)).toBe('07165e0')
  expect(isNewBuild('07165e03f9a1', notes, false)).toBe(false)
  expect(isNewBuild('aaaaaaa1111', notes, true)).toBe(true)
  // a build without a commit (dev) or notes without one fall back to the version comparison
  expect(isNewBuild('', notes, true)).toBe(false)
  expect(isNewBuild('07165e0', 'no commit here', false)).toBe(true)
})
