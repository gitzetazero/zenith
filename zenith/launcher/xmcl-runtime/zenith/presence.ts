// zenith: Discord Rich Presence. The Discord application (named "Zenith", with a large image asset called `zenith-logo`)
// is the maintainer's: its id comes from ZENITH_DISCORD_CLIENT_ID (environment, zenith-config.json or the build).
// Without an id the presence is disabled silently.
import type { SetActivity } from '@xmcl/discord-rpc'

export const ZENITH_REPOSITORY_URL = 'https://github.com/gitzetazero/zenith'
export const DISCORD_LOGO_ASSET = 'zenith-logo'

export function isPresenceConfigured(clientId: string | undefined): boolean {
  return !!clientId && /^\d{5,25}$/.test(clientId.trim())
}

/** What the launcher shows while it is open: "Playing Zenith", the page being viewed, and a link to the Zenith repository. */
export function buildZenithActivity(details: string, startTimestamp: number): SetActivity {
  return {
    details,
    state: 'Playing Zenith',
    largeImageKey: DISCORD_LOGO_ASSET,
    largeImageText: 'Zenith Launcher',
    startTimestamp,
    buttons: [{ label: 'Zenith on GitHub', url: ZENITH_REPOSITORY_URL }],
  }
}
