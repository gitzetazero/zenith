// zenith: third-party identities that XMCL embedded are NOT carried over (see docs/ZENITH-LAUNCHER.md).
// Each value is optional and read, in order, from: the environment, <appData>/zenith-config.json, the build;
// except the Microsoft client id, which is public and built in below.
import { readFileSync } from 'fs'
import { join } from 'path'

export type ZenithConfigKey =
  | 'msaClientId'
  | 'curseforgeApiKey'
  | 'modrinthClientId'
  | 'littleskinClientId'
  | 'discordClientId'

/**
 * The public client id the Minecraft launcher and the open-source launcher
 * ecosystem have used for years. It is not a secret: Microsoft embeds it in
 * the authorization URL of every signed-in client.
 *
 * It only speaks Live Connect (v1), not the Entra ID (v2) endpoints MSAL
 * uses, so `MicrosoftOAuthClient` selects its legacy flow from this value.
 */
export const MINECRAFT_LAUNCHER_PUBLIC_CLIENT_ID = '00000000402b5328'
export const MINECRAFT_LAUNCHER_PUBLIC_SCOPE = 'service::user.auth.xboxlive.com::MBI_SSL'
export const MINECRAFT_LAUNCHER_PUBLIC_REDIRECT_URI = 'https://login.live.com/oauth20_desktop.srf'

export function isMinecraftLauncherPublicClientId(clientId: string) {
  return clientId.trim().replace(/[{}]/g, '').toLowerCase() === MINECRAFT_LAUNCHER_PUBLIC_CLIENT_ID
}

const ENV: Record<ZenithConfigKey, string> = {
  msaClientId: 'ZENITH_MSA_CLIENT_ID',
  curseforgeApiKey: 'CURSEFORGE_API_KEY',
  modrinthClientId: 'ZENITH_MODRINTH_CLIENT_ID',
  littleskinClientId: 'ZENITH_LITTLESKIN_CLIENT_ID',
  discordClientId: 'ZENITH_DISCORD_CLIENT_ID',
}

// Set by esbuild `define` (see xmcl-electron-app/esbuild.config.ts); empty when the build did not provide them.
const BUILD: Partial<Record<ZenithConfigKey, string>> = {
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
  const fromBuild = BUILD[key]?.trim()
  if (fromBuild) return fromBuild
  // The Microsoft client id is public and built into every launcher, so it needs
  // no repository variable or build-time value. The environment and the config
  // file still win above, so a developer can point a launcher at another app.
  return key === 'msaClientId' ? MINECRAFT_LAUNCHER_PUBLIC_CLIENT_ID : ''
}

export const MSA_NOT_CONFIGURED_MESSAGE =
  'Zenith Launcher needs an Azure app registered and approved by Mojang to sign in with Microsoft; see docs/ZENITH-LAUNCHER.md.'
