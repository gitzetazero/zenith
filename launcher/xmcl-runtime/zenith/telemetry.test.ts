import { expect, test, vi } from 'vitest'
import { pluginTelemetry } from '../infra/plugins/telemetry'
import { isRuntimeTelemetryEnabled, setRuntimeTelemetryEnabled } from '../infra/telemetry_context'

test('telemetry is hard-off: enabled flag is forced off and IPC answers with no-ops', async () => {
  setRuntimeTelemetryEnabled(true)
  const handlers = new Map<string, (...a: any[]) => any>()
  const app: any = { controller: { handle: vi.fn((c: string, h: any) => handlers.set(c, h)) } }
  await pluginTelemetry(app, {} as any)
  expect(isRuntimeTelemetryEnabled()).toBe(false)
  expect(await handlers.get('renderer-telemetry-action-start')!()).toBeUndefined()
  expect(await handlers.get('renderer-telemetry-action-end')!()).toBe(false)
  expect(handlers.size).toBe(4)
})
