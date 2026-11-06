// zenith: where Zenith Launcher releases are published. The launcher lives in the Zenith repository
// (launcher/) and is released by its workflow; nothing here depends on the fork's repository name.
export const ZENITH_REPOSITORY = 'gitzetazero/zenith'
export const ZENITH_REPOSITORY_URL = `https://github.com/${ZENITH_REPOSITORY}`
export const ZENITH_RELEASES_URL = `${ZENITH_REPOSITORY_URL}/releases`
export const ZENITH_RELEASES_API = `https://api.github.com/repos/${ZENITH_REPOSITORY}/releases?per_page=30`
export const ZENITH_AUTHOR_URL = 'https://github.com/gitzetazero'
/** The installer a launcher release carries; the Zenith repository also publishes the game, so releases are told apart by it. */
export const LAUNCHER_ASSETS = ['Zenith-Launcher-Setup-x64.exe', 'Zenith-Launcher-Portable-x64.zip'] as const
export const LAUNCHER_USER_AGENT_CONTACT = 'github.com/gitzetazero/zenith'

interface ReleaseLike {
  tag_name?: string
  draft?: boolean
  assets?: Array<{ name?: string }>
}

/** Newest published release (the API lists newest first) that carries the launcher installer. */
export function pickLauncherRelease<T extends ReleaseLike>(releases: T[]): T | undefined {
  return releases.find((r) => !r.draft && r.assets?.some((a) => a.name === LAUNCHER_ASSETS[0]))
}

/** `v0.73.1-zenith.2` and `launcher-v0.73.1-zenith.2` both mean `0.73.1-zenith.2`. */
export function versionOfTag(tag: string): string {
  return /(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$/.exec(tag)?.[1] ?? tag.replace(/^v/, '')
}

/** The commit a release was built from: its notes start with "Zenith build of commit <sha>". */
export function commitOfNotes(body: string | undefined | null): string | undefined {
  return /Zenith build of commit ([0-9a-f]{7,40})/i.exec(body ?? '')?.[1]?.toLowerCase()
}

/**
 * Whether `release` is a different build from the running one. Releases are tagged by date (`nightly-2026-11-06`),
 * which says nothing about the app version, so the build's own commit is compared; without it (a dev build)
 * the versions decide.
 */
export function isNewBuild(builtCommit: string, notes: string | undefined | null, sameVersion: boolean): boolean {
  const released = commitOfNotes(notes)
  const built = builtCommit.trim().toLowerCase()
  if (!built || !released) return !sameVersion
  return !(built.startsWith(released) || released.startsWith(built))
}
