// XMCL core.launch()/watcher against a fake java shim, no shell. Run: node e2e.cjs  (see SPIKE.md section 10)
const core = require('./core.cjs')
const cp = require('child_process'), fs = require('fs'), path = require('path')
const { access } = require('fs/promises'), { constants } = fs
const T = path.resolve(__dirname, '..')
const ROOT = path.join(T, 'w', 'xmcl'), GAME = path.join(ROOT, 'instances', 'Zenith')
const BS = String.fromCharCode(92), J = (...p) => p.join(BS)
const LONG_ROOT = J('C:', 'Users', 'Some Long User Name', 'AppData', 'Roaming', 'xmcl')
const LONG_JAVA = J('C:', 'Users', 'Some Long User Name', 'AppData', 'Roaming', 'zenith', 'java', 'bin', 'javaw.exe')
const token = (n) => { const b = 'eyJhbGciOiJSUzI1NiJ9'; let s = b + '.'; while (s.length < n - 20) s += 'abcdefghijklmnopqrstuvwxyz0123456789_-'.charAt(s.length % 38); return (s + '.' + 'x'.repeat(n)).slice(0, n) }
// faithful copy of LaunchService.#generateOptions (without yggdrasil): swap java.exe -> javaw.exe, shell only with prepend
async function swap(java) {
  const w = path.join(path.dirname(java), 'javaw.exe')
  return await access(w, constants.X_OK).then(() => true, () => false) ? w : java
}
async function opts({ java, root = ROOT, token: tk = token(1200), prepend, version }) {
  return {
    gameProfile: { name: 'FakeName', id: '069a79f444e94726a5befca90e38aaf5' }, accessToken: tk, properties: {},
    gamePath: GAME, resourcePath: root, javaPath: java, version: version ?? '26.3',
    extraExecOption: { shell: !!(prepend && prepend.length), detached: true, cwd: GAME, env: { ...process.env } },
    launcherBrand: 'XMCL (0.73.1)', launcherName: 'XMCL (0.73.1)', prependCommand: prepend,
    extraJVMArgs: undefined, useHashAssetsIndex: true,
    platform: { arch: 'x64', name: 'windows', version: '10.0.26300' }, demo: false, prechecks: [],
  }
}
function run(label, op) { // core.launch + watcher exactly as LaunchService wires them
  return core.launch(op).then(p => new Promise(res => {
    const out = { label, pid: p.pid, shell: !!op.extraExecOption.shell }
    let so = '', se = ''; p.stdout.on('data', d => so += d); p.stderr.on('data', d => se += d)
    const w = core.createMinecraftProcessWatcher(p); let ready = false
    w.on('minecraft-window-ready', () => ready = true)
    w.on('minecraft-exit', e => setTimeout(() => res({ ...out, ready, code: e.code, stdout: so.trim(), stderr: se.trim() }), 300))
    w.on('error', e => res({ ...out, error: String(e) }))
  }))
}
;(async () => {
  const dump = path.join(T, 'dump.txt')
  const rd = () => { const t = fs.readFileSync(dump, 'utf8').split(/\r?\n/); return { raw: +t[0].split('=')[1], argv: t.filter(l => l.startsWith('ARG=')).map(l => l.slice(4)), cwd: t.find(l => l.startsWith('CWD=')), pid: +(t.find(l => l.startsWith('PID='))||'PID=0').slice(4) } }
  const results = {}
  for (const [name, dir] of [['A1 java.exe consola + javaw.exe winsub (par real)', 'jre'], ['A2 solo java.exe consola', 'jre2'], ['A3 solo javaw.exe winsub (java.exe=javaw.exe)', 'jre3']]) {
    fs.rmSync(dump, { force: true })
    const java = await swap(path.join(T, dir, 'bin', 'java.exe'))
    const op = await opts({ java })
    op.extraExecOption.env.ARGDUMP_OUT = dump
    const r = await run(name, op)
    const d = fs.existsSync(dump) ? rd() : undefined
    console.log(JSON.stringify({ ...r, spawned: path.relative(T, java), rawLen: d?.raw, argc: d?.argv.length, cwdOk: d?.cwd === 'CWD=' + GAME, pidEqualsNodes: d?.pid === r.pid }))
    if (dir === 'jre') results.argv = d.argv
  }
  // argv against the repo fixture (different root: normalised)
  const fx = fs.readFileSync(process.argv[2] || path.join(__dirname, 'xmcl-args.txt'), 'utf8').split(/\r?\n/).filter(Boolean)
  const norm = (s) => s.replaceAll(ROOT.replaceAll(BS, '/'), 'ROOT').replaceAll('C:/fake/xmcl', 'ROOT').replace(/^.*javaw[.]exe$/, 'JAVA')
  const a = results.argv.slice(1).map(norm), b = fx.slice(1).map(norm) // el fixture trae javaw.exe como primer elemento; el shim lo recibe como argv[0]
  const diffs = a.map((x, i) => x !== b[i] ? `#${i + 1}: ${x.slice(0, 60)} != ${(b[i] || '').slice(0, 60)}` : null).filter(Boolean)
  console.log('argv vs fixture: argc', a.length, 'vs', b.length, '; differences (after normalising root/java):', diffs.length, diffs.slice(0, 3))
  // length: virtual long root (version already resolved, no disk), variable token
  const resolved = await core.Version.parse(ROOT, '26.3')
  const sh = path.join(T, 'jre', 'bin', 'javaw.exe')
  for (const n of [10, 700, 1200, 1500]) {
    const op = await opts({ java: LONG_JAVA, root: LONG_ROOT, token: token(n), version: resolved })
    op.gamePath = LONG_ROOT + '/instances/Zenith' // virtual --gameDir; the real cwd stays in GAME (extraExecOption.cwd)
    const args = await core.generateArguments(op)
    // length CreateProcess will see: libuv quotes arguments with spaces; measured for real with the real shim
    const op2 = { ...op, javaPath: sh }
    const args2 = await core.generateArguments(op2)
    const p = cp.spawn(args2[0], args2.slice(1), { detached: true, cwd: GAME, env: { ...process.env, ARGDUMP_OUT: dump } })
    await new Promise(r => p.on('close', r))
    const raw = rd().raw
    const virt = raw - sh.length + LONG_JAVA.length + 2   // path of the real shim (without spaces) -> long path (with spaces: libuv quotes it, +2)
    // with shell (prepend): what cmd.exe receives = the whole command in quotes
    const pre = [`"${path.join(T, 'jre', 'bin', 'javaw.exe')}"`, ...args.map(x => `"${x}"`)]
    const opS = { ...op, prependCommand: [J('C:', 'zenith', 'bridge.exe')], extraExecOption: { ...op.extraExecOption, shell: true } }
    const argsS = (await core.generateArguments(opS)).map(x => `"${x}"`)
    const cmdLen = argsS.join(' ').length
    console.log(`token ${n}: long root (${LONG_ROOT.length}) without shell: RAW measured with the real shim=${raw}, corrected to the long java path=${virt}, margin vs 32767=${32767 - virt}; with prepend+shell: command for cmd.exe=${cmdLen} (limit 8191)`)
  }

  // paths with spaces/Ñ and arguments with % and " (without a shell they are neither expanded nor lost)
  { const d = path.join(T, 'Jose Ñ', 'java dir'); fs.mkdirSync(path.join(d, 'bin'), { recursive: true })
    fs.copyFileSync(path.join(T, 'jre', 'bin', 'javaw.exe'), path.join(d, 'bin', 'javaw.exe')); fs.copyFileSync(path.join(T, 'jre', 'bin', 'java.exe'), path.join(d, 'bin', 'java.exe'))
    process.env.ZXTEST = 'EXPANDED'
    fs.rmSync(dump, { force: true })
    const op = await opts({ java: await swap(path.join(d, 'bin', 'java.exe')) }); op.extraMCArgs = ['--x', '100%ZXTEST%', '--y', 'a&b', '--z', 'q"uote', '--w', 'tab\there']
    op.extraExecOption.env.ARGDUMP_OUT = dump
    const r = await run('path with spaces and N', op), x = rd()
    console.log('path with spaces and Ñ:', r.ready, r.code, 'java argv[0] =', x.argv[0].replace(T, '<tmp>'), '| last args:', JSON.stringify(x.argv.slice(-8))) }

  // the earlier design (prependCommand + cmd.exe) with the recommended short root and a real token: C:\xmcl
  for (const n of [10, 1200]) {
    const root = J('C:', 'xmcl'), op = await opts({ java: J(root, 'jre', 'java-runtime-epsilon', 'bin', 'javaw.exe'), root, token: token(n), version: resolved, prepend: [path.join(T, 'jre', 'bin', 'javaw.exe')] })
    op.gamePath = root + '/instances/Zenith'
    const args = (await core.generateArguments(op)).map(x => `"${x}"`)
    fs.rmSync(dump, { force: true })
    const p = cp.spawn(args[0], args.slice(1), { shell: true, detached: true, cwd: GAME, env: { ...process.env, ARGDUMP_OUT: dump } })
    const code = await new Promise(r => p.on('close', r))
    console.log(`prepend+shell, root ${root} (${root.length}), token ${n}: command=${args.join(' ').length}, code ${code}, shim executed: ${fs.existsSync(dump)}`)
  }
  // the real cmd.exe rejects the long line (long root, token 1200, prepend with shell)
  { const op = await opts({ java: LONG_JAVA, root: LONG_ROOT, version: resolved, prepend: [path.join(T, 'jre', 'bin', 'javaw.exe')] })
    op.gamePath = LONG_ROOT + '/instances/Zenith'
    const args = await core.generateArguments(op).then(a => a.map(x => `"${x}"`))
    fs.rmSync(dump, { force: true })
    const p = cp.spawn(args[0], args.slice(1), { shell: true, detached: true, cwd: GAME, env: { ...process.env, ARGDUMP_OUT: dump } })
    let se = ''; p.stderr.on('data', d => se += d); const code = await new Promise(r => p.on('close', r))
    console.log('prepend+shell, long root, token 1200: code', code, 'shim executed:', fs.existsSync(dump), 'stderr:', JSON.stringify(se.trim())) }
})()
