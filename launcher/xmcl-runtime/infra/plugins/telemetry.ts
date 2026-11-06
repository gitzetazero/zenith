// zenith: telemetry is hard-off. There is no exporter, no key, no setting and no toggle: this plugin only
// makes sure the in-process telemetry stays disabled and answers the renderer's telemetry IPC with no-ops.
import { LauncherAppPlugin } from '~/app'
import { setRuntimeTelemetryEnabled } from '~/infra'

export const pluginTelemetry: LauncherAppPlugin = async (app) => {
  setRuntimeTelemetryEnabled(false)
  app.controller.handle('renderer-telemetry-exception', async () => {})
  app.controller.handle('renderer-telemetry-flush', async () => {})
  app.controller.handle('renderer-telemetry-action-start', async () => undefined)
  app.controller.handle('renderer-telemetry-action-end', async () => false)
}
