import { DialogKey } from './dialog'

export type LaunchStatusParam = {
  isKill?: boolean
}

export const LaunchStatusDialogKey: DialogKey<LaunchStatusParam> = 'launch-status'
