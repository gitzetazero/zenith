import 'dotenv/config'
import { BuildOptions } from 'esbuild'
import { yamlPlugin } from 'esbuild-plugin-yaml'
import path from 'path'
import pluginVueDevtools from './plugins/esbuild.devtool.plugin'
import plugin7Zip from './plugins/esbuild.native.plugin'
import pluginNode from './plugins/esbuild.node.plugin'
import pluginPreload from './plugins/esbuild.preload.plugin'
import pluginRenderer from './plugins/esbuild.renderer.plugin'
import createSourcemapPlugin from './plugins/esbuild.sourcemap.plugin'
import pluginStatic from './plugins/esbuild.static.plugin'
import pluginWorker from './plugins/esbuild.worker.plugin'
import pluginElevate from './plugins/esbuild.elevate.plugin'
import pluginTreeshake from 'plugins/esbuild.treeshake.plugin'

const config = {
  bundle: true,
  metafile: true,
  assetNames: '[name]',
  entryNames: '[dir]/[name]',
  format: 'cjs',
  sourcemap: process.env.NODE_ENV === 'production' ? 'external' : 'linked',
  minifyWhitespace: process.env.NODE_ENV === 'production',
  minifySyntax: process.env.NODE_ENV === 'production',
  treeShaking: true,
  keepNames: true,
  define: {
    'process.env.BUILD_NUMBER': JSON.stringify(process.env.BUILD_NUMBER) ?? '0',
    'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV) ?? '"development"',
    // zenith: optional build-time values (empty by default; see xmcl-runtime/zenith/config.ts)
    'process.env.CURSEFORGE_API_KEY_BUILD': JSON.stringify(process.env.CURSEFORGE_API_KEY ?? ''),
    'process.env.ZENITH_MSA_CLIENT_ID_BUILD': JSON.stringify(process.env.ZENITH_MSA_CLIENT_ID ?? ''),
    // zenith: the commit this launcher was built from; the updater compares it with the release notes
    'process.env.ZENITH_COMMIT_BUILD': JSON.stringify(process.env.ZENITH_COMMIT ?? ''),
    'process.env.ZENITH_DISCORD_CLIENT_ID_BUILD': JSON.stringify(process.env.ZENITH_DISCORD_CLIENT_ID ?? ''),
  } as Record<string, string>,
  platform: 'node',
  loader: {
    '.png': 'file',
    '.jpeg': 'file',
    '.jpg': 'file',
    '.svg': 'file',
    '.webp': 'file',
    '.gif': 'file',
    '.cs': 'file',
    '.vbs': 'text',
    '.ico': 'file',
    '.jvmclass': 'binary', // zenith: compiled Java helpers are stored as .jvmclass (the public-tree check forbids .class),
    '.html': 'file',
    '.wasm': 'file',
  },
  plugins: [
    pluginTreeshake(),
    pluginRenderer(),
    pluginStatic(),
    createSourcemapPlugin(),
    pluginPreload(path.resolve(__dirname, './preload')),
    pluginVueDevtools(path.resolve(__dirname, '../extensions')),
    pluginWorker(),
    pluginElevate(),
    // pluginJsdetect(),
    plugin7Zip(),
    pluginNode(),
    yamlPlugin({}) as any,
  ],
  external: ['electron'],
} satisfies BuildOptions

export default config
