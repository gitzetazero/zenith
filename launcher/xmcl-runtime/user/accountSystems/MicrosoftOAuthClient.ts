import { AccountInfo, DeviceCodeResponse, INativeBrokerPlugin } from '@azure/msal-common'
import { AuthenticationResult, LogLevel, PublicClientApplication } from '@azure/msal-node'
import { AnyError } from '@xmcl/utils'
import {
  MINECRAFT_LAUNCHER_PUBLIC_REDIRECT_URI,
  MINECRAFT_LAUNCHER_PUBLIC_SCOPE,
  MSA_NOT_CONFIGURED_MESSAGE,
  isMinecraftLauncherPublicClientId,
} from '~/zenith/config'
import { randomUUID } from 'crypto'
import { copyFileSync, mkdirSync } from 'fs'
import { tmpdir } from 'os'
import { join } from 'path'
import type { MicrosoftAuthTelemetryEvent } from '~/app'
import { SecretStorage } from '~/app/SecretStorage'
import { Logger } from '~/infra'
import { createPlugin } from '../credentialPlugin'
import { isNetworkError, isUserCanceledError } from './MicrosoftAuthErrors'
import { createNetworkClient } from './OAuthNetworkClient'

export const MICROSOFT_GRAPH_USER_READ_SCOPE = 'User.Read'
// zenith: the scopes of the standard Minecraft sign-in (wiki.vg "Microsoft Authentication Scheme"). `offline_access`
// is what gives MSAL a refresh token; `XboxLive.offline_access` is not a scope a newly registered app can rely on.
export const MICROSOFT_XBOX_LAUNCHER_SCOPES = [
  'XboxLive.signin',
  'offline_access',
]

let nativeBrokerRuntimePrepared = false
let nativeBrokerPlugin: INativeBrokerPlugin | undefined
let nativeBrokerPluginInitialized = false
let nativeBrokerPluginPromise: Promise<INativeBrokerPlugin | undefined> | undefined

interface MsalDiagnostic {
  status?: string
  errorCode?: string
}

interface Authentication {
  result: AuthenticationResult
  extra?: AuthenticationResult
}

interface SilentAuthentication {
  account?: AccountInfo
  authentication?: Authentication
}

/**
 * Tokens issued by Live Connect (v1). The public Minecraft client id is not a
 * Microsoft Entra application, so MSAL (which only talks to
 * login.microsoftonline.com, v2) cannot be used with it.
 */
interface LegacyTokenResponse {
  access_token?: string
  refresh_token?: string
  expires_in?: number
  user_id?: string
  error?: string
  error_description?: string
}

interface LegacyTokenRecord {
  version: 1
  clientId: string
  username: string
  userId: string
  accessToken: string
  refreshToken: string
  expiresAt: number
}

const LEGACY_TOKEN_SERVICE = 'xmcl-oauth-legacy'

interface AuthTelemetryState {
  attemptId: string
  startedAt: number
  routeUsed: string
  fallbackUsed: boolean
  cachedAccount: boolean
  completed: boolean
}

interface AuthenticateOptions {
  signal?: AbortSignal
  useDeviceCode?: boolean
  code?: string
  slientOnly?: boolean
  extraScopes?: string[]
  directRedirectToLauncher?: boolean
  useNativeBroker?: boolean
}

type TrackAuthentication = (
  name: MicrosoftAuthTelemetryEvent['name'],
  properties: Record<string, string | number | boolean>,
  measurements?: Record<string, number>,
) => void

function parseMsalDiagnostic(message: string): MsalDiagnostic {
  return {
    status: /StatusInternal::([^,\s]+)/.exec(message)?.[1],
    errorCode: /Error Code ([0-9]+)/.exec(message)?.[1],
  }
}

function getSafeErrorProperties(error: unknown) {
  const result: Record<string, string | number | boolean> = {}
  const toSafeCode = (value: unknown) => {
    const text = typeof value === 'string' || typeof value === 'number' ? String(value) : ''
    return /^[a-z0-9_.-]{1,128}$/i.test(text) ? text : undefined
  }
  const errorName = error instanceof Error ? toSafeCode(error.name) : undefined
  if (errorName) {
    result.errorName = errorName
  }
  const candidate = error as { errorCode?: unknown; subError?: unknown } | undefined
  const errorCode = toSafeCode(candidate?.errorCode)
  if (errorCode) {
    result.errorCode = errorCode
  }
  const subError = toSafeCode(candidate?.subError)
  if (subError) {
    result.subError = subError
  }
  return result
}

export class MicrosoftOAuthClient {
  constructor(
    private fetch: typeof global.fetch,
    private logger: Logger,
    readonly clientId: string,
    private getCode: (url: string, redirectUri: string, signal?: AbortSignal, authAttemptId?: string) => Promise<string>,
    private getRedirectUrl: (preferLocalHost: boolean) => Promise<string>,
    private deviceCodeCallback: (deviceCodeResponse: DeviceCodeResponse) => void,
    private storage: SecretStorage,
    private getWindowHandle?: () => Buffer | undefined,
    private emitTelemetry?: (event: MicrosoftAuthTelemetryEvent) => void,
  ) {
  }

  private async getNativeBrokerPlugin() {
    if (process.platform !== 'win32') {
      return undefined
    }
    if (nativeBrokerPluginInitialized) {
      return nativeBrokerPlugin
    }
    if (nativeBrokerPluginPromise) {
      return nativeBrokerPluginPromise
    }
    const operation = (async () => {
      try {
        this.prepareNativeBrokerRuntime()
        const { NativeBrokerPlugin } = await import('@azure/msal-node-extensions')
        const plugin = new NativeBrokerPlugin()
        return plugin.isBrokerAvailable ? plugin : undefined
      } catch (e) {
        this.logger.warn('Unable to load the Windows native broker plugin')
        this.logger.warn(e)
        return undefined
      }
    })()
    nativeBrokerPluginPromise = operation
    try {
      nativeBrokerPlugin = await operation
      nativeBrokerPluginInitialized = true
      return nativeBrokerPlugin
    } finally {
      if (nativeBrokerPluginPromise === operation) {
        nativeBrokerPluginPromise = undefined
      }
    }
  }

  private prepareNativeBrokerRuntime() {
    if (nativeBrokerRuntimePrepared) return

    const arch = process.arch === 'ia32' ? 'ia32' : 'x64'
    const target = join(tmpdir(), 'xmcl-msal-node-runtime', String(process.pid), arch)
    mkdirSync(target, { recursive: true })
    const targetNode = join(target, 'msal-node-runtime.node')
    copyFileSync(join(__dirname, `msal-node-runtime-${arch}.node`), targetNode)
    copyFileSync(join(__dirname, `msalruntime-${arch}.dll`), join(target, 'msalruntime.dll'))
    process.env.XMCL_MSAL_NODE_RUNTIME_PATH = targetNode
    nativeBrokerRuntimePrepared = true
  }

  protected async getOAuthApp(
    signal?: AbortSignal,
    nativeBrokerPlugin?: INativeBrokerPlugin,
    onDiagnostic?: (diagnostic: MsalDiagnostic) => void,
  ) {
    if (!this.clientId) throw new AnyError('ZenithMsaNotConfigured', MSA_NOT_CONFIGURED_MESSAGE)
    return new PublicClientApplication({
      auth: {
        authority: 'https://login.microsoftonline.com/consumers/',
        clientId: this.clientId,
      },
      broker: nativeBrokerPlugin ? { nativeBrokerPlugin } : undefined,
      cache: {
        cachePlugin: createPlugin('xmcl-oauth', this.logger, this.storage),
      },
      system: {
        loggerOptions: {
          logLevel: LogLevel.Error,
          loggerCallback: (level, message) => {
            if (level === LogLevel.Error) {
              const diagnostic = parseMsalDiagnostic(message)
              onDiagnostic?.(diagnostic)
              const status = diagnostic.status ? `: ${diagnostic.status}` : ''
              const errorCode = diagnostic.errorCode ? ` (${diagnostic.errorCode})` : ''
              this.logger.warn(`Microsoft authentication diagnostic${status}${errorCode}`)
            }
          },
        },
        networkClient: createNetworkClient(this.fetch, signal),
      },
    })
  }

  private async acquireSilently(
    app: PublicClientApplication,
    username: string,
    scopes: string[],
    extraScopes: string[] | undefined,
  ): Promise<SilentAuthentication> {
    const accounts = await app.getAllAccounts().catch((error) => {
      this.logger.warn(`Microsoft account lookup failed: ${JSON.stringify(getSafeErrorProperties(error))}`)
      return []
    })
    const account = accounts.find(value => value.username.toLowerCase() === username.toLowerCase())
    if (!account) {
      this.logger.warn(`Microsoft silent token acquisition missed: no matching account (accountCount=${accounts.length}).`)
      return {}
    }

    const result = await app.acquireTokenSilent({
      scopes,
      account,
      forceRefresh: false,
    }).catch((error) => {
      this.logger.warn(`Microsoft silent token acquisition missed: ${JSON.stringify(getSafeErrorProperties(error))}`)
      return null
    })
    if (!result) return { account }

    const extra = extraScopes
      ? await app.acquireTokenSilent({ scopes: extraScopes, account }).catch((error) => {
        this.logger.warn(`Microsoft silent extra-scope acquisition missed: ${JSON.stringify(getSafeErrorProperties(error))}`)
        return undefined
      }) ?? undefined
      : undefined
    return { account, authentication: { result, extra } }
  }

  private async acquireInteractively(
    app: PublicClientApplication,
    nativeBrokerPlugin: INativeBrokerPlugin | undefined,
    brokerDiagnostic: MsalDiagnostic,
    account: AccountInfo | undefined,
    username: string,
    scopes: string[],
    options: AuthenticateOptions,
    windowHandle: Buffer | undefined,
    authAttemptId: string,
    track: TrackAuthentication,
  ): Promise<Authentication & { routeUsed: string; fallbackUsed: boolean }> {
    let result: AuthenticationResult | null = null
    let routeUsed = 'device_code'
    let fallbackUsed = false

    if (options.useDeviceCode) {
      if (options.signal) {
        options.signal.addEventListener('abort', () => {
          app.acquireTokenByDeviceCode({
            scopes,
            deviceCodeCallback: this.deviceCodeCallback,
            cancel: true,
          })
        })
      }
      result = await app.acquireTokenByDeviceCode({
        scopes,
        deviceCodeCallback: this.deviceCodeCallback,
      })
    } else {
      if (nativeBrokerPlugin) {
        routeUsed = 'wam'
        const brokerStartedAt = Date.now()
        try {
          result = await app.acquireTokenInteractive({
            scopes,
            extraScopesToConsent: options.extraScopes,
            account,
            loginHint: username || undefined,
            prompt: 'select_account',
            openBrowser: async () => {},
            windowHandle,
          })
          if (account && result.account?.homeAccountId !== account.homeAccountId) {
            track('microsoft-auth-broker-result', {
              outcome: 'account_mismatch',
              fallbackReason: 'different_account',
              ...(brokerDiagnostic.status ? { msalStatus: brokerDiagnostic.status } : {}),
              ...(brokerDiagnostic.errorCode ? { msalInternalErrorCode: brokerDiagnostic.errorCode } : {}),
            }, { durationMs: Date.now() - brokerStartedAt })
            this.logger.warn('Microsoft broker returned a different account; falling back to WebView.')
            result = null
          } else {
            track('microsoft-auth-broker-result', {
              outcome: 'success',
              ...(brokerDiagnostic.status ? { msalStatus: brokerDiagnostic.status } : {}),
              ...(brokerDiagnostic.errorCode ? { msalInternalErrorCode: brokerDiagnostic.errorCode } : {}),
            }, { durationMs: Date.now() - brokerStartedAt })
          }
        } catch (error) {
          const canceled = isUserCanceledError(error)
          track('microsoft-auth-broker-result', {
            outcome: canceled ? 'user_cancelled' : 'error',
            ...getSafeErrorProperties(error),
            ...(brokerDiagnostic.status ? { msalStatus: brokerDiagnostic.status } : {}),
            ...(brokerDiagnostic.errorCode ? { msalInternalErrorCode: brokerDiagnostic.errorCode } : {}),
          }, { durationMs: Date.now() - brokerStartedAt })
          if (canceled) throw error
          this.logger.warn('Microsoft broker authentication failed; falling back to WebView.')
        }
      }

      if (!result) {
        routeUsed = options.code ? 'authorization_code' : 'webview'
        fallbackUsed = Boolean(nativeBrokerPlugin)
        const redirectUri = await this.getRedirectUrl(options.directRedirectToLauncher ?? false)
        let code = options.code
        if (!code) {
          const url = await app.getAuthCodeUrl({
            redirectUri,
            scopes,
            extraScopesToConsent: options.extraScopes,
            loginHint: username,
            prompt: 'select_account',
          })
          code = await this.getCode(url, redirectUri, options.signal, authAttemptId)
        }
        result = await app.acquireTokenByCode({ code, scopes, redirectUri })
      }
    }

    if (!result) {
      throw new AnyError('MicrosoftOAuthEmptyResult', 'Microsoft authentication returned no result.')
    }
    if (account && result.account?.homeAccountId !== account.homeAccountId) {
      throw new AnyError(
        'MicrosoftOAuthAccountMismatch',
        `Microsoft authentication returned a different account than ${username}.`,
      )
    }
    let extra: AuthenticationResult | undefined
    if (options.extraScopes && result.account) {
      extra = await app.acquireTokenSilent({
        account: result.account,
        scopes: options.extraScopes,
      }).catch((error) => {
        this.logger.warn(`Microsoft silent extra-scope acquisition missed: ${JSON.stringify(getSafeErrorProperties(error))}`)
        return undefined
      }) ?? undefined
    }
    return { result, extra, routeUsed, fallbackUsed }
  }

  /** True when the configured id is the public Minecraft client id (Live Connect v1). */
  get usesLegacyMinecraftClient() {
    return isMinecraftLauncherPublicClientId(this.clientId)
  }

  private legacyAccountKey(username: string, userId: string) {
    return (username.trim() || userId).toLowerCase()
  }

  private legacyAuthentication(record: LegacyTokenRecord): Authentication {
    const account: AccountInfo = {
      homeAccountId: record.userId,
      localAccountId: record.userId,
      environment: 'login.live.com',
      tenantId: 'consumers',
      username: record.username,
    }
    return {
      result: {
        accessToken: record.accessToken,
        expiresOn: new Date(record.expiresAt),
        account,
        scopes: [MINECRAFT_LAUNCHER_PUBLIC_SCOPE],
        tenantId: 'consumers',
        uniqueId: record.userId,
        fromCache: false,
      } as unknown as AuthenticationResult,
    }
  }

  private async readLegacyToken(username: string): Promise<LegacyTokenRecord | undefined> {
    const key = username.trim().toLowerCase()
    if (!key) return undefined
    try {
      const raw = await this.storage.get(LEGACY_TOKEN_SERVICE, key)
      if (!raw) return undefined
      const record = JSON.parse(raw) as LegacyTokenRecord
      if (record.version !== 1 || record.clientId !== this.clientId || !record.refreshToken) {
        return undefined
      }
      return record
    } catch (error) {
      this.logger.warn(`Microsoft legacy token cache unreadable: ${JSON.stringify(getSafeErrorProperties(error))}`)
      return undefined
    }
  }

  private async requestLegacyToken(fields: Record<string, string>, signal?: AbortSignal): Promise<LegacyTokenResponse> {
    const response = await this.fetch('https://login.live.com/oauth20_token.srf', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams(fields).toString(),
      signal,
    })
    let payload: LegacyTokenResponse = {}
    try {
      payload = await response.json() as LegacyTokenResponse
    } catch {
      // The error below carries the HTTP status.
    }
    if (!response.ok || payload.error || !payload.access_token) {
      const message = payload.error_description || payload.error ||
        `Live Connect token request failed (HTTP ${response.status})`
      const error = new AnyError('MicrosoftLegacyOAuthTokenFailed', message)
      ;(error as any).error = payload.error
      ;(error as any).status = response.status
      throw error
    }
    return payload
  }

  private async saveLegacyToken(
    username: string,
    token: LegacyTokenResponse,
    previous?: LegacyTokenRecord,
  ): Promise<Authentication> {
    const userId = token.user_id || previous?.userId || this.legacyAccountKey(username, '')
    const record: LegacyTokenRecord = {
      version: 1,
      clientId: this.clientId,
      username: username.trim() || userId,
      userId,
      accessToken: token.access_token!,
      refreshToken: token.refresh_token || previous?.refreshToken || '',
      expiresAt: Date.now() + (token.expires_in ?? 3600) * 1000,
    }
    await this.storage.put(
      LEGACY_TOKEN_SERVICE,
      this.legacyAccountKey(record.username, record.userId),
      JSON.stringify(record),
    )
    return this.legacyAuthentication(record)
  }

  private async acquireLegacySilently(username: string, signal?: AbortSignal): Promise<SilentAuthentication> {
    const record = await this.readLegacyToken(username)
    if (!record) return {}
    if (record.expiresAt > Date.now() + 60_000) {
      const authentication = this.legacyAuthentication(record)
      return { account: authentication.result.account ?? undefined, authentication }
    }
    const token = await this.requestLegacyToken({
      client_id: this.clientId,
      grant_type: 'refresh_token',
      refresh_token: record.refreshToken,
      redirect_uri: MINECRAFT_LAUNCHER_PUBLIC_REDIRECT_URI,
      scope: MINECRAFT_LAUNCHER_PUBLIC_SCOPE,
    }, signal)
    const authentication = await this.saveLegacyToken(record.username, token, record)
    return { account: authentication.result.account ?? undefined, authentication }
  }

  private async acquireLegacyInteractively(
    username: string,
    options: AuthenticateOptions,
    authAttemptId: string,
  ): Promise<Authentication> {
    const redirectUri = MINECRAFT_LAUNCHER_PUBLIC_REDIRECT_URI
    let code = options.code
    if (!code) {
      const params = new URLSearchParams({
        client_id: this.clientId,
        response_type: 'code',
        redirect_uri: redirectUri,
        scope: MINECRAFT_LAUNCHER_PUBLIC_SCOPE,
        prompt: 'select_account',
      })
      if (username.trim()) params.set('login_hint', username.trim())
      const url = `https://login.live.com/oauth20_authorize.srf?${params.toString()}`
      code = await this.getCode(url, redirectUri, options.signal, authAttemptId)
    }
    const token = await this.requestLegacyToken({
      client_id: this.clientId,
      grant_type: 'authorization_code',
      code,
      redirect_uri: redirectUri,
      scope: MINECRAFT_LAUNCHER_PUBLIC_SCOPE,
    }, options.signal)
    return this.saveLegacyToken(username, token)
  }

  private async authenticateLegacy(
    username: string,
    options: AuthenticateOptions,
    telemetry: AuthTelemetryState,
    complete: (outcome: string, error?: unknown) => void,
  ): Promise<Authentication> {
    telemetry.routeUsed = 'silent'
    if (!options.code && !options.useDeviceCode && username.trim()) {
      try {
        const silent = await this.acquireLegacySilently(username, options.signal)
        telemetry.cachedAccount = Boolean(silent.account)
        if (silent.authentication) {
          complete('success')
          return silent.authentication
        }
      } catch (error) {
        this.logger.warn(`Microsoft legacy silent token acquisition missed: ${JSON.stringify(getSafeErrorProperties(error))}`)
        if (options.slientOnly) throw error
      }
    }
    if (options.slientOnly) {
      throw new AnyError('MicrosoftOAuthSlientFailed', 'Fail to acquire Microsoft token silently.')
    }
    telemetry.routeUsed = 'authorization_code'
    const authentication = await this.acquireLegacyInteractively(username, options, telemetry.attemptId)
    complete('success')
    return authentication
  }

  async authenticate(username: string, scopes: string[], options: AuthenticateOptions = {}) {
    const telemetry = {
      attemptId: randomUUID(),
      startedAt: Date.now(),
      routeUsed: 'silent',
      fallbackUsed: false,
      cachedAccount: false,
      completed: false,
    }
    const track: TrackAuthentication = (name, properties, measurements) => {
      this.emitTelemetry?.({
        name,
        properties: {
          authAttemptId: telemetry.attemptId,
          ...properties,
        },
        measurements,
      })
    }
    const complete = (outcome: string, error?: unknown) => {
      if (telemetry.completed) return
      telemetry.completed = true
      track('microsoft-auth-complete', {
        outcome,
        routeUsed: telemetry.routeUsed,
        fallbackUsed: telemetry.fallbackUsed,
        cachedAccount: telemetry.cachedAccount,
        ...getSafeErrorProperties(error),
      }, {
        durationMs: Date.now() - telemetry.startedAt,
      })
    }

    try {
      if (this.usesLegacyMinecraftClient) {
        // The public Minecraft client id only exists on Live Connect v1; MSAL,
        // the native broker and the localhost redirect cannot be used with it.
        track('microsoft-auth-start', {
          preferredFlow: options.slientOnly ? 'silent' : 'authorization_code',
          brokerRequested: false,
          brokerAvailable: false,
          windowHandleAvailable: false,
          existingAccount: Boolean(username),
        })
        return await this.authenticateLegacy(username, options, telemetry, complete)
      }

      const nativeBrokerPlugin = options.useNativeBroker
        ? await this.getNativeBrokerPlugin()
        : undefined
      const windowHandle = nativeBrokerPlugin && !options.slientOnly ? this.getWindowHandle?.() : undefined
      track('microsoft-auth-start', {
        preferredFlow: options.slientOnly ? 'silent' : options.useDeviceCode ? 'device_code' : options.useNativeBroker ? 'wam' : 'webview',
        brokerRequested: Boolean(options.useNativeBroker),
        brokerAvailable: Boolean(nativeBrokerPlugin),
        windowHandleAvailable: Boolean(windowHandle),
        existingAccount: Boolean(username),
      })

      let brokerDiagnostic: MsalDiagnostic = {}
      const app = await this.getOAuthApp(
        options.signal,
        options.useDeviceCode ? undefined : nativeBrokerPlugin,
        diagnostic => { brokerDiagnostic = diagnostic },
      )
      let account: AccountInfo | undefined
      const allowSilentReuse = options.slientOnly || !options.useDeviceCode
      if (username && !options.code && allowSilentReuse) {
        const silent = await this.acquireSilently(
          app,
          username,
          scopes,
          options.extraScopes,
        )
        account = silent.account
        telemetry.cachedAccount = Boolean(account)
        telemetry.routeUsed = nativeBrokerPlugin && account?.nativeAccountId ? 'wam' : 'silent'
        if (silent.authentication) {
          complete('success')
          return silent.authentication
        }
      }

      if (options.slientOnly) {
        throw new AnyError('MicrosoftOAuthSlientFailed', 'Fail to acquire Microsoft token silently.')
      }

      telemetry.routeUsed = options.useDeviceCode
        ? 'device_code'
        : nativeBrokerPlugin
          ? 'wam'
          : options.code
            ? 'authorization_code'
            : 'webview'
      const authentication = await this.acquireInteractively(
        app,
        nativeBrokerPlugin,
        brokerDiagnostic,
        account,
        username,
        scopes,
        options,
        windowHandle,
        telemetry.attemptId,
        track,
      )
      telemetry.routeUsed = authentication.routeUsed
      telemetry.fallbackUsed = authentication.fallbackUsed

      complete('success')
      return {
        result: authentication.result,
        extra: authentication.extra,
      }
    } catch (e) {
      complete(
        options.signal?.aborted
          ? 'aborted'
          : (e as Error | undefined)?.name === 'MicrosoftOAuthSlientFailed'
            ? 'silent_miss'
          : isUserCanceledError(e)
            ? 'user_cancelled'
            : isNetworkError(e)
              ? 'network_error'
              : 'error',
        e,
      )
      throw e
    }
  }
}
