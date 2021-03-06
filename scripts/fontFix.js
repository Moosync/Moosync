const path = require('path')
const fs = require('fs')
const replace = require('replace')

exports.default = async function (context) {
  const workDir = path.join(context.appDir, 'css')
  const files = fs.readdirSync(workDir)
  replace({
    regex: 'moosync:///fonts',
    replacement: 'moosync://./fonts',
    paths: files.map((val) => path.join(workDir, val)),
    recursive: false,
    silent: false
  })

  return true
}
