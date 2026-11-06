// Replica of XMCL's validation path: JavaService.resolveJava (validateJavaPath + resolveJava + getJavaArch)
// and the UI side (getAutoSelectedJava + getAutoOrManuallJava + getInstanceJavaIssue).
const { resolveJava } = require('./java-resolve.cjs')
const ui = require('./ui-java.cjs')
const cp = require('child_process'), path = require('path'), fs = require('fs')
const T = path.resolve(__dirname, '..')
const arch = (p) => new Promise(r => cp.execFile(p, ['JavaProxy', '-jvm_info'], { cwd: T, windowsHide: true }, (e, so) => { try { r(JSON.parse(so).java_arch) } catch { r(undefined) } }))
// getInstanceJavaIssue de instanceJavaDiagnose.ts (copy of the 10 useful lines)
const issue = (stat) => stat.javaPath ? (!stat.java?.valid ? 'invalid' : stat.compatible !== undefined && stat.compatible !== ui.JavaCompatibleState.Matched ? 'incompatible' : undefined) : (!stat.java ? 'missing' : undefined)
const ver = { javaVersion: { component: 'java-runtime-epsilon', majorVersion: 25 } }
async function status(javaPath, all) {
  const detected = ui.getAutoSelectedJava(all, '26.3', undefined, ver)
  const r = await ui.getAutoOrManuallJava(detected, async (p) => { const j = await resolveJava(p); return j && { ...j, valid: true } }, javaPath)
  const stat = { javaPath, ...r.auto, java: r.java || r.auto.java, compatible: r.quality }
  return { quality: r.quality, issue: issue(stat) || null, req: detected.preference.requirement }
}
;(async () => {
  const cases = { 'shim 25 (java.exe)': 'jre/bin/java.exe', 'shim 25 (javaw.exe)': 'jre/bin/javaw.exe', 'shim that says 21': 'jre21/bin/java.exe', 'foreign binary (whoami.exe)': 'C:/Windows/System32/whoami.exe', 'Temurin 25 real': 'C:/Program Files/Eclipse Adoptium/jdk-25.0.5.8-hotspot/bin/java.exe', 'missing path': 'nope/bin/java.exe' }
  for (const [k, rel] of Object.entries(cases)) {
    const p = path.isAbsolute(rel) || rel.startsWith('C:') ? rel : path.join(T, rel)
    let access = 'ok'; try { fs.accessSync(p, fs.constants.X_OK) } catch (e) { access = e.code }
    const j = await resolveJava(p)
    const all = j ? [{ ...j, valid: true }] : []
    const s = await status(p, all), s0 = await status(p, [])
    console.log(k.padEnd(28), 'X_OK:', access, '| resolveJava:', j ? `${j.version} (major ${j.majorVersion})` : 'undefined', '| arch:', j ? await arch(p) : '-', '| UI quality:', s.quality, 'problem:', s.issue, '(all empty:', s0.issue + ')', 'requirement', s.req)
  }
})()
// risk: other instances without their own java may pick the shim as "Java 25" (selection by minimum version)
{
  const shim = { path: 'S/bin/java.exe', version: '25.0.5', majorVersion: 25, valid: true }, real = { path: 'R/bin/java.exe', version: '21.0.5', majorVersion: 21, valid: true }
  const v21 = { javaVersion: { component: 'java-runtime-delta', majorVersion: 21 } }
  for (const [n, all] of [['[real21]', [real]], ['[shim25]', [shim]], ['[shim25, real21]', [shim, real]], ['[real21, shim25]', [real, shim]]]) {
    const r = ui.getAutoSelectedJava(all, '1.21.4', undefined, v21)
    console.log('1.21.4 instance without its own java, list', n.padEnd(18), '-> picks', r.java?.path ?? '(nothing: it will ask to install)')
  }
}
