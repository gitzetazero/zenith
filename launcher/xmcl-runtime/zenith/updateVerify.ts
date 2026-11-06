import { ZENITH_REPOSITORY, commitOfNotes } from './release'

/** The per-user NSIS installer the launcher updates itself with. */
export const INSTALLER_ASSET = 'Zenith-Launcher-Setup-x64.exe'

/**
 * The SHA-256 the release notes list for `filename` (a `<64 hex>  <filename>` line, as ci/zenith_publish.sh writes it).
 * Undefined when absent: callers must then refuse to run the file.
 */
export function sha256FromNotes(notes: string | undefined | null, filename: string): string | undefined {
  for (const line of (notes ?? '').split(/\r?\n/)) {
    const m = /^\s*([0-9a-fA-F]{64})\s+\*?(.+?)\s*$/.exec(line)
    if (m && m[2] === filename) return m[1].toLowerCase()
  }
  return undefined
}

/** Only HTTPS release downloads of the Zenith repository on github.com; anything else is refused (redirects are then GitHub's own CDN). */
export function isAllowedInstallerUrl(url: string): boolean {
  try {
    const u = new URL(url)
    return u.protocol === 'https:' && u.hostname === 'github.com' && !u.username && !u.password && !u.port
      && u.pathname.startsWith(`/${ZENITH_REPOSITORY}/releases/download/`) && !u.pathname.includes('..')
  } catch {
    return false
  }
}

/** Throws unless the downloaded file matches the hash in the notes (fail closed: a missing hash is a refusal). */
export function assertInstallerHash(actual: string, expected: string | undefined): void {
  if (!expected) throw new Error('The release notes carry no SHA-256 for the installer; it will not be run')
  if (actual.toLowerCase() !== expected.toLowerCase()) throw new Error('The downloaded installer does not match the SHA-256 of the release; it will not be run')
}

/** NSIS: silent, flagged as an update, and restart the app afterwards (electron-builder's own flags). */
export function installerSilentArgs(): string[] {
  return ['/S', '--updated', '--force-run']
}

/** The NSIS installer drops an "Uninstall <name>.exe" next to the app; the portable build has none (or sets PORTABLE_EXECUTABLE_FILE). */
export function isInstalledBuild(portableExecutableFile: string | undefined, filesNextToExe: string[]): boolean {
  return !portableExecutableFile && filesNextToExe.some((f) => /^uninstall .*\.exe$/i.test(f))
}

/**
 * Whether this build can replace itself with the release's installer. Never for the same commit (loop guard) and never
 * when either commit is unknown: dated tags do not tell versions apart, so without commits it could repeat forever.
 */
export function canSelfUpdate(o: {
  os: string
  env: string
  installed: boolean
  devServer: boolean
  builtCommit: string
  notes: string | undefined | null
  files: Array<{ name: string; url: string }>
}): boolean {
  const released = commitOfNotes(o.notes)
  const built = o.builtCommit.trim().toLowerCase()
  if (o.devServer || o.os !== 'windows' || o.env === 'appx' || !o.installed || !built || !released) return false
  if (built.startsWith(released) || released.startsWith(built)) return false
  const asset = o.files.find((f) => f.name === INSTALLER_ASSET)
  return !!asset && isAllowedInstallerUrl(asset.url) && !!sha256FromNotes(o.notes, INSTALLER_ASSET)
}
