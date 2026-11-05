const { resolveJava, resolveJavaWithDiagnostic, parseJavaVersion } = require('./java-resolve.cjs')
const path = require('path')
const root = path.resolve(__dirname, '..', process.argv[2] || 'jre')
;(async () => {
  for (const exe of ['java.exe', 'javaw.exe']) {
    const p = path.join(root, 'bin', exe)
    const d = await resolveJavaWithDiagnostic(p)
    console.log(exe, JSON.stringify({ java: d.java, exitCode: d.exitCode, stdout: d.stdout.slice(0, 40), stderr: d.stderr.slice(0, 60) }))
  }
})()
