const { nodeResolve } = require('@rollup/plugin-node-resolve')

module.exports = {
    plugins: [
        nodeResolve(),
    ],
}