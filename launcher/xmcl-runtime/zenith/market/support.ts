// zenith: "Works / Partly / Doesn't run", the way Horizon's launcher (launcher/src/browser.rs `support`) words it.
import type { MarketKind, MarketSupport } from '@xmcl/runtime-api'

/** Loaders of Java mods: they can never run in Zenith. */
export const JAVA_LOADERS = ['fabric', 'quilt', 'forge', 'neoforge', 'liteloader', 'rift']

/** The newest plain release (`1.21.4`, `26.2`) in a list of Minecraft versions. */
export function newestRelease(versions: string[]): string | undefined {
  const plain = versions.filter((v) => v.split('.').every((p) => /^\d+$/.test(p)))
  const key = (v: string) => v.split('.').map(Number)
  return plain.sort((a, b) => {
    const x = key(a)
    const y = key(b)
    for (let i = 0; i < Math.max(x.length, y.length); i++) {
      const d = (y[i] ?? 0) - (x[i] ?? 0)
      if (d) return d
    }
    return 0
  })[0]
}

export const DATAPACK_NOTE =
  "Zenith runs a data pack's functions, function tags, predicates and item modifiers; not yet its recipes, loot tables, advancements, other tags or worldgen."

/** A project made for Zenith (the registry): it runs when it is for the version Zenith plays. */
export function registrySupport(kind: MarketKind, gameVersions: string[], game: string): MarketSupport {
  const newest = newestRelease(gameVersions)
  if (!gameVersions.includes(game)) {
    return { level: 'no', reason: newest ? `Made for Zenith on ${newest}.` : 'Lists no Minecraft version.' }
  }
  switch (kind) {
    case 'native-mod':
      return { level: 'works', reason: 'Native mods are compiled into the game: they need a Zenith rebuild, which this launcher cannot do yet.' }
    case 'modpack':
      return { level: 'partial', reason: 'Modpacks (.hpack) cannot be installed from this launcher yet.' }
    case 'datapack':
      return { level: 'partial', reason: DATAPACK_NOTE }
    default:
      return { level: 'works' }
  }
}

/** A Modrinth project (Java ecosystem) judged for Zenith. */
export function modrinthSupport(kind: MarketKind, gameVersions: string[], loaders: string[], game: string): MarketSupport {
  const made = gameVersions.includes(game)
  const newest = newestRelease(gameVersions)
  switch (kind) {
    case 'resourcepack':
      if (made) return { level: 'works' }
      return {
        level: 'partial',
        reason: newest
          ? `Made for ${newest}: it loads with vanilla's compatibility warning, and what changed since may look wrong.`
          : 'Lists no Minecraft version.',
      }
    case 'datapack': {
      const prefix = made ? '' : newest ? `Made for ${newest}: the game may refuse it. ` : 'Lists no Minecraft version. '
      return { level: 'partial', reason: `${prefix}${DATAPACK_NOTE}` }
    }
    case 'shader':
      return { level: 'partial', reason: 'A Java-format shader pack: Zenith runs its own .hshader packs, so this may not work.' }
    case 'mod':
      return {
        level: 'no',
        reason: loaders.some((l) => JAVA_LOADERS.includes(l)) ? 'A Java mod: Zenith does not run Java code.' : 'Not made for Zenith.',
      }
    default:
      return { level: 'no', reason: 'Not made for Zenith.' }
  }
}

export function installable(support: MarketSupport) {
  return support.level !== 'no'
}
