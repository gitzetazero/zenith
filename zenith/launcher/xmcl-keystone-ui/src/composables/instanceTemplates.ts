import { DialogKey } from './dialog'

// zenith: the Add instance dialog creates only Zenith instances; any parameter other dialogs pass (drag and drop of
// modpacks, imports) is ignored, so the type stays permissive.
export type AddInstanceDialogParameter = unknown

export const AddInstanceDialogKey: DialogKey<AddInstanceDialogParameter> = 'add-instance-dialog'
