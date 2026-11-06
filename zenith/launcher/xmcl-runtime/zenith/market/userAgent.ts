// zenith: every request of the Market names the launcher and the Zenith repository.
export const USER_AGENT_PRODUCT = 'Zenith-Launcher'
export const USER_AGENT_CONTACT = 'github.com/gitzetazero/zenith'

/** `Zenith-Launcher/<version> (github.com/gitzetazero/zenith)` */
export function userAgent(version: string): string {
  return `${USER_AGENT_PRODUCT}/${version} (${USER_AGENT_CONTACT})`
}
