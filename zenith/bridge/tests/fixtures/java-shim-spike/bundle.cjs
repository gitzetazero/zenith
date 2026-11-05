// node bundle.cjs <XMCL clone (read only)> <work folder>; esbuild via ESBUILD_DIR or 'esbuild'
const path = require('path'), fs = require('fs')
const esbuild = require(process.env.ESBUILD_DIR || 'esbuild')
const [clone, work] = process.argv.slice(2).map(p => path.resolve(p)), h = path.join(work, 'h')
fs.mkdirSync(h, { recursive: true })
const u = (p) => path.join(clone, p).replaceAll(String.fromCharCode(92), '/')
const b = (entry, out, alias) => esbuild.buildSync({ entryPoints: [entry], bundle: true, platform: 'node', format: 'cjs', outfile: path.join(h, out), logLevel: 'warning', alias })
// @xmcl/core: the clone's package is bundled (with @xmcl/unzip replaced by an empty module: it is only used to extract natives)
fs.writeFileSync(path.join(h, 'unzip-stub.js'), 'module.exports = {}')
b(u('packages/core/index.ts'), 'core.cjs', { '@xmcl/unzip': path.join(h, 'unzip-stub.js') })
const core = path.join(h, 'core.cjs')
for (const [src, out] of [['packages/installer/java.ts', 'java-resolve.cjs'], ['xmcl-runtime-api/src/util/java.ts', 'ui-java.cjs']]) {
  fs.writeFileSync(path.join(h, 'entry.ts'), `export * from ${JSON.stringify(u(src))}`)
  b(path.join(h, 'entry.ts'), out, { '@xmcl/core': core })
}
