import { BaseService } from '@xmcl/runtime/app'
import { AuthlibInjectorService } from '@xmcl/runtime/authlibInjector'
import { CollectionService } from '@xmcl/runtime/collection'
import { ExternalCredentialService } from '@xmcl/runtime/credential/ExternalCredentialService'
import { ElyByService } from '@xmcl/runtime/elyby'
import { VersionInstallService, VersionMetadataService } from '@xmcl/runtime/install'
import {
  InstanceLogService,
  InstanceModsService,
  InstanceOptionsService,
  InstanceSavesService,
  InstanceResourcePackService,
  InstanceScreenshotService,
  InstanceServerInfoService,
  InstanceService,
  InstanceShaderPacksService,
  InstanceThemeService,
  InstanceModsGroupService,
  InstanceBlueprintsService,
} from '@xmcl/runtime/instance'
import {
  InstanceIOService,
  InstanceInstallService,
  InstanceManifestService,
} from '@xmcl/runtime/instanceIO'
import { JavaService } from '@xmcl/runtime/java'
import { LaunchService, VersionService } from '@xmcl/runtime/launch'
import { ProjectMappingService } from '@xmcl/runtime/moddb'
import { ModMetadataService } from '@xmcl/runtime/moddb/ModMetadataService'
import { BlueprintMarketService } from '@xmcl/runtime/market'
import { ModpackService } from '@xmcl/runtime/modpack'
import { RemoteServerService } from '@xmcl/runtime/remoteServer'
import { PresenceService } from '@xmcl/runtime/presence'
import { ResourcePackPreviewService } from '@xmcl/runtime/resourcePack'
import { ServerStatusService } from '@xmcl/runtime/serverStatus'
import { ThemeService } from '@xmcl/runtime/theme'
import { BackupService } from '@xmcl/runtime/backup'
import { LocalSkinService, OfficialUserService, UserService } from '@xmcl/runtime/user'
import { ZenithMarketService } from '@xmcl/runtime/zenith'
import { ServerService } from './ServerService'

export const definedServices = [
  // Main-process-only: it deliberately has no service key, so token access
  // cannot be invoked through renderer service IPC.
  ExternalCredentialService,
  VersionMetadataService,
  BaseService,
  AuthlibInjectorService,
  CollectionService,
  VersionInstallService,
  ProjectMappingService,
  InstanceIOService,
  InstanceLogService,
  ElyByService,
  InstanceModsService,
  InstanceModsGroupService,
  InstanceOptionsService,
  InstanceResourcePackService,
  InstanceSavesService,
  InstanceService,
  InstanceScreenshotService,
  InstanceShaderPacksService,
  InstanceBlueprintsService,
  BlueprintMarketService,
  PresenceService,
  JavaService,
  LaunchService,
  ModpackService,
  InstanceServerInfoService,
  RemoteServerService,
  ServerService,
  ResourcePackPreviewService,
  InstanceManifestService,
  ServerStatusService,
  OfficialUserService,
  UserService,
  LocalSkinService,
  VersionService,
  InstanceInstallService,
  ModMetadataService,
  ThemeService,
  InstanceThemeService,
  BackupService,
  ZenithMarketService, // zenith
]
