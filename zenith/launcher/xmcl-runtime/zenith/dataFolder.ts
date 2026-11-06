// zenith: Zenith's own game data folder. It never touches the official `.minecraft`.
import { join } from 'path'

/** Directory name under the OS application-data folder (`%APPDATA%`, `~/.config`, ...). */
export const ZENITH_DATA_FOLDER = '.minecraftz'

/** `<appData>/.minecraftz` (macOS keeps the visible `minecraftz`, like the official layout did). */
export function zenithDataPath(appData: string, os: 'windows' | 'osx' | 'linux' | string): string {
  return join(appData, os === 'osx' ? ZENITH_DATA_FOLDER.slice(1) : ZENITH_DATA_FOLDER)
}
