// zenith: third-party identities that XMCL embedded are NOT carried over (see docs/ZENITH-LAUNCHER.md).
// Each value is optional and read, in order, from: the environment, <appData>/zenith-config.json, the build.
import { readFileSync } from 'fs'
import { join } from 'path'

export type ZenithConfigKey =
  | 'msaClientId'
  | 'curseforgeApiKey'
  | 'modrinthClientId'
  | 'littleskinClientId'
  | 'discordClientId'

const ENV: Record<ZenithConfigKey, string> = {
  msaClientId: 'ZENITH_MSA_CLIENT_ID',
  curseforgeApiKey: 'CURSEFORGE_API_KEY',
  modrinthClientId: 'ZENITH_MODRINTH_CLIENT_ID',
  littleskinClientId: 'ZENITH_LITTLESKIN_CLIENT_ID',
  discordClientId: 'ZENITH_DISCORD_CLIENT_ID',
}

// Set by esbuild `define` (see xmcl-electron-app/esbuild.config.ts); empty when the build did not provide them.
const BUILD: Partial<Record<ZenithConfigKey, string>> = {
  msaClientId: process.env.ZENITH_MSA_CLIENT_ID_BUILD,
  curseforgeApiKey: process.env.CURSEFORGE_API_KEY_BUILD,
  discordClientId: process.env.ZENITH_DISCORD_CLIENT_ID_BUILD,
}

let appDataDir = ''

/** Call once the app data path is known so `zenith-config.json` can be read. */
export function setZenithConfigDirectory(path: string) {
  appDataDir = path
}

function readFile(): Partial<Record<ZenithConfigKey, unknown>> {
  if (!appDataDir) return {}
  try {
    return JSON.parse(readFileSync(join(appDataDir, 'zenith-config.json'), 'utf-8'))
  } catch {
    return {}
  }
}

export function getZenithConfig(key: ZenithConfigKey): string {
  const fromEnv = process.env[ENV[key]]?.trim()
  if (fromEnv) return fromEnv
  const fromFile = readFile()[key]
  if (typeof fromFile === 'string' && fromFile.trim()) return fromFile.trim()
  return BUILD[key]?.trim() || ''
}

export const MSA_NOT_CONFIGURED_MESSAGE =
  'Zenith Launcher needs an Azure app registered and approved by Mojang to sign in with Microsoft; see docs/ZENITH-LAUNCHER.md.'
