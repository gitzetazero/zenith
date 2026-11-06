export const IS_DEV = process.env.NODE_ENV === 'development'
// zenith: data lives in %APPDATA%/zenith-launcher, never mixed with XMCL's
export const LAUNCHER_NAME = 'zenith-launcher'
export const LAUNCHER_PROTOCOL = 'zenith-launcher'
export const RESOURCE_FILE_VERSION = 2
export const HAS_DEV_SERVER = !!process.env.HAS_DEV_SERVER
