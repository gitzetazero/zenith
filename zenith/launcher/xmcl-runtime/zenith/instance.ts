// zenith: creates the "Zenith" instance whose `java` is a copy of the bundled Zenith bridge (the same trick as
// `zenith-bridge setup`, see zenith/bridge/src/setup.rs in gitzetazero/zenith). Implemented here because the bridge's
// own `setup` looks for XMCL's data root in %APPDATA%\xmcl, which Zenith Launcher does not use.
import { existsSync } from 'fs'
import { copyFile, mkdir, readFile, rename, rm, writeFile } from 'fs/promises'
import { dirname, join } from 'path'

/** Minecraft version of the Zenith game; the bridge pins the exact one on first play. Same fallback as the bridge. */
export const ZENITH_MINECRAFT = '26.3'
export const ZENITH_INSTANCE_NAME = 'Zenith'

/** Zenith's home, exactly like the bridge: ZENITH_HOME or %APPDATA%\zenith. */
export function zenithHome(env: NodeJS.ProcessEnv, appData: string) {
  return env.ZENITH_HOME?.trim() || join(appData, 'zenith')
}

async function sameBytes(a: string, b: string) {
  try {
    const [x, y] = await Promise.all([readFile(a), readFile(b)])
    return x.equals(y)
  } catch {
    return false
  }
}

/** Installs two independent copies (java.exe, javaw.exe; XMCL tells files apart by inode). Returns java.exe. */
export async function installJavaShim(bridgeExe: string, home: string): Promise<string> {
  const bin = join(home, 'java', 'bin')
  await mkdir(bin, { recursive: true })
  for (const name of ['java.exe', 'javaw.exe']) {
    const dst = join(bin, name)
    if (await sameBytes(bridgeExe, dst)) continue
    const tmp = dst + '.new'
    await copyFile(bridgeExe, tmp)
    await rename(tmp, dst).catch(async (e) => {
      await rm(tmp, { force: true })
      throw e
    })
  }
  return join(bin, 'java.exe')
}

export interface InstanceCreator {
  readonly instances: ReadonlyArray<{ name: string }>
  createInstance(options: Record<string, unknown>): Promise<string>
}

export interface EnsureZenithInstanceOptions {
  bridgeExe: string
  home: string
  markerPath: string
  iconPath?: string
  instances: InstanceCreator
  log: (message: string) => void
}

/**
 * Creates the Zenith instance once. Returns what happened so callers (and tests) can tell.
 * A marker file keeps a deliberately deleted instance from coming back on every start.
 */
export async function ensureZenithInstance(o: EnsureZenithInstanceOptions): Promise<'created' | 'exists' | 'skipped' | 'no-bridge'> {
  if (!existsSync(o.bridgeExe)) {
    o.log(`Zenith bridge not found at ${o.bridgeExe}; the Zenith instance is not created.`)
    return 'no-bridge'
  }
  // the shim is the "Java" of every Zenith instance: keep it current when the launcher (and so the bridge) was updated
  const java = await installJavaShim(o.bridgeExe, o.home)
  if (o.instances.instances.some((i) => i.name === ZENITH_INSTANCE_NAME)) return 'exists'
  if (existsSync(o.markerPath)) return 'skipped'
  await o.instances.createInstance({
    name: ZENITH_INSTANCE_NAME,
    description: 'Zenith (Minecraft in Rust), installed and updated by the Zenith bridge.',
    runtime: { minecraft: ZENITH_MINECRAFT },
    java,
    // an explicit empty string also overrides a global pre-launch command
    prependCommand: '',
    ...(o.iconPath ? { icon: o.iconPath } : {}),
  })
  await mkdir(dirname(o.markerPath), { recursive: true })
  await writeFile(o.markerPath, JSON.stringify({ created: new Date().toISOString() }))
  o.log('Created the Zenith instance.')
  return 'created'
}
