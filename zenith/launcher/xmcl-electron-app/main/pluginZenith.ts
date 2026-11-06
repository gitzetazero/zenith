// zenith: creates the Zenith instance on startup (see xmcl-runtime/zenith/instance.ts)
import { LauncherAppPlugin, kGameDataPath } from '@xmcl/runtime/app'
import { InstanceService } from '@xmcl/runtime/instance'
import { toMediaIconUrl } from '@xmcl/runtime/instance/instanceIcon'
import { ensureZenithInstance, zenithHome } from '@xmcl/runtime/zenith/instance'
import { join } from 'path'
import logoDark from '../icons/dark@256x256.png'

export const pluginZenith: LauncherAppPlugin = (app) => {
  if (process.platform !== 'win32' || process.env.XMCL_E2E) return
  const logger = app.getLogger('Zenith')
  // runs in the background once the data root is known (the first-run wizard may take a while)
  app.registry.get(kGameDataPath).then(async () => {
    const instances = await app.registry.getOrCreate(InstanceService)
    await instances.initialize()
    await ensureZenithInstance({
      bridgeExe: join(process.resourcesPath, 'zenith', 'zenith-bridge-windows-x64.exe'),
      home: zenithHome(process.env, app.host.getPath('appData')),
      markerPath: join(app.appDataPath, 'zenith-instance.json'),
      iconPath: toMediaIconUrl(logoDark),
      instances: {
        get instances() { return instances.state.instances },
        createInstance: (o) => instances.createInstance(o as any),
      },
      log: (m) => logger.log(m),
    })
  }).catch((e) => logger.warn(e))
}
