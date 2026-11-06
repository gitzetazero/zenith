import { MicrosoftMinecraftXboxLoginError } from '@xmcl/user'

export function isAccountSuspendedError(error: unknown): error is MicrosoftMinecraftXboxLoginError {
  if (!(error instanceof MicrosoftMinecraftXboxLoginError) || error.status !== 403) return false
  try {
    return JSON.parse(error.body)?.details?.reason === 'ACCOUNT_SUSPENDED'
  } catch {
    return false
  }
}

export function isUserCanceledError(error: unknown) {
  const message = (error as Error | undefined)?.message
  return typeof message === 'string' &&
    (/\buser_cancel(?:led|ed)\b/.test(message) ||
      message === 'Microsoft authorization window was closed.')
}

export function isNetworkError(error: unknown) {
  const candidate = error as { errorCode?: unknown; message?: unknown } | undefined
  return candidate?.errorCode === 'network_error' ||
    (typeof candidate?.message === 'string' && /\bnetwork_error\b/.test(candidate.message))
}

/**
 * Why the Microsoft sign-in cannot even start or finish, when the cause is how the Azure app is registered (not the
 * user or the network). Matches MSAL / login.live.com error codes and messages.
 */
export type MicrosoftAppSetupProblem = 'APP_NOT_FOUND' | 'REDIRECT_URI_NOT_REGISTERED' | 'INVALID_SCOPE'

export function classifyMicrosoftAppSetupProblem(error: unknown): MicrosoftAppSetupProblem | undefined {
  const candidate = error as { errorCode?: unknown; error?: unknown; message?: unknown; errorMessage?: unknown } | undefined
  const text = [candidate?.errorCode, candidate?.error, candidate?.message, candidate?.errorMessage]
    .filter((v): v is string => typeof v === 'string')
    .join(' ')
  if (!text) return undefined
  if (/unauthorized_client|AADSTS700016|AADSTS700025|client does not exist|not enabled for consumers/i.test(text)) return 'APP_NOT_FOUND'
  if (/redirect_uri|redirect uri|AADSTS50011|AADSTS900971/i.test(text) && /not valid|does not match|not registered|invalid/i.test(text)) return 'REDIRECT_URI_NOT_REGISTERED'
  if (/AADSTS70011|AADSTS65001|invalid_scope|scope[^.]*not valid/i.test(text)) return 'INVALID_SCOPE'
  return undefined
}
