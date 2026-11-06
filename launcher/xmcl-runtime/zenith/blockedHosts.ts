// zenith: Zenith Launcher never talks to XMCL's hosted services or its telemetry. Their origins were replaced by
// `.invalid` names (never resolvable); this guard additionally refuses them before any DNS lookup happens.
export function isBlockedUpstreamUrl(input: unknown): boolean {
  try {
    const raw = typeof input === 'string' ? input : input instanceof URL ? input.href : (input as { url?: string })?.url
    if (!raw) return false
    const host = new URL(raw).hostname.toLowerCase()
    return host.endsWith('.invalid') || host.endsWith('.applicationinsights.azure.com') || host.endsWith('.livediagnostics.monitor.azure.com')
  } catch {
    return false
  }
}
