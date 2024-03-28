const fs = require("node:fs")

async function copyVm2() {
    await fs.promises.copyFile('./node_modules/vm2/lib/bridge.js', './dist/bridge.js')
    await fs.promises.copyFile('./node_modules/vm2/lib/events.js', './dist/events.js')
    await fs.promises.copyFile('./node_modules/vm2/lib/setup-sandbox.js', './dist/setup-sandbox.js')
    await fs.promises.copyFile('./node_modules/vm2/lib/setup-node-sandbox.js', './dist/setup-node-sandbox.js')
}

copyVm2()