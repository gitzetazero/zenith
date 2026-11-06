// zenith: Zenith Launcher creates only Zenith instances. Every instance creation (the Add instance dialog, drag and drop,
// modpack import, commands) goes through InstanceService.createInstance, which calls this guard first.
import { join } from 'path'
import { zenithHome, ZENITH_MINECRAFT } from './instance'

/** Runtime keys that would make an instance something other than Zenith (mod loaders, OptiFine, LabyMod, ...). */
const FOREIGN_RUNTIME_KEYS = [
  'forge', 'neoForged', 'fabricLoader', 'quiltLoader', 'optifine', 'labyMod', 'liteloader', 'liteloaderOptiFine',
] as const

export class ZenithOnlyError extends Error {
  constructor(reason: string) {
    super(`Zenith Launcher can only create Zenith instances: ${reason}`)
    this.name = 'ZenithOnlyError'
  }
}

export interface ZenithGuardEnv {
  env: NodeJS.ProcessEnv
  appData: string
  platform: NodeJS.Platform
}

/** The java.exe the bridge was installed as (the instance's invisible "Java"); only on Windows. */
export function zenithJavaPath({ env, appData, platform }: ZenithGuardEnv): string | undefined {
  return platform === 'win32' ? join(zenithHome(env, appData), 'java', 'bin', 'java.exe') : undefined
}

/**
 * Turns the creation payload into a Zenith instance or throws. The Minecraft version is always Zenith's pinned one,
 * `java` is the bridge shim and nothing Java-specific (arguments, memory, command) is accepted.
 */
export function enforceZenithInstance<T extends Record<string, any>>(payload: T, guard: ZenithGuardEnv): T & Record<string, any> {
  if (payload.edition && payload.edition !== 'java') throw new ZenithOnlyError(`edition "${payload.edition}"`)
  if (payload.upstream) throw new ZenithOnlyError('modpack upstreams are not supported')
  if (payload.server) throw new ZenithOnlyError('server presets are not supported')
  const runtime = (payload.runtime ?? {}) as Record<string, unknown>
  for (const key of FOREIGN_RUNTIME_KEYS) {
    if (runtime[key]) throw new ZenithOnlyError(`${key} is not supported`)
  }
  for (const key of Object.keys(runtime)) {
    if (key !== 'minecraft' && runtime[key]) throw new ZenithOnlyError(`runtime "${key}" is not supported`)
  }
  // keep a pinned patch version of Zenith's own line (e.g. 26.3.1 from the bridge), never anything else
  const minecraft = typeof runtime.minecraft === 'string' && runtime.minecraft.startsWith(ZENITH_MINECRAFT) ? runtime.minecraft : ZENITH_MINECRAFT
  const java = zenithJavaPath(guard)
  return {
    ...payload,
    runtime: { minecraft },
    ...(java ? { java } : {}),
    // an explicit empty string also overrides a global pre-launch command
    prependCommand: '',
    vmOptions: [],
    mcOptions: [],
    env: {},
    preExecuteCommand: '',
    minMemory: 0,
    maxMemory: 0,
    assignMemory: false,
  }
}
