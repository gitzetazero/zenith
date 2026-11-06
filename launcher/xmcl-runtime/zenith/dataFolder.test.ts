import { expect, test } from 'vitest'
import { join } from 'path'
import { zenithDataPath, ZENITH_DATA_FOLDER } from './dataFolder'

test('Zenith data lives in .minecraftz under appData', () => {
  expect(ZENITH_DATA_FOLDER).toBe('.minecraftz')
  expect(zenithDataPath('/a', 'windows')).toBe(join('/a', '.minecraftz'))
  expect(zenithDataPath('/a', 'linux')).toBe(join('/a', '.minecraftz'))
  expect(zenithDataPath('/a', 'osx')).toBe(join('/a', 'minecraftz'))
})
