import { setRendererTelemetryEnabled } from '@/telemetry'
import type { Settings, SharedState } from '@xmcl/runtime-api'
import type { Ref } from 'vue'

// zenith: telemetry is hard-off; there is no setting to turn it on.
export function useTelemetryTrack(_settings: Ref<SharedState<Settings> | undefined>) {
  setRendererTelemetryEnabled(false)
}
