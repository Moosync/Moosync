
// On windows for some reason running stat on symlinks returns EPERM access denied
// So we need to patch the module resolution to read the target dir from the symlink
// (I know it makes no sense but it works). Everthing below resolves symlinks from
// aspect_rules_js's pnpm style node_modules structure.
if (process.platform === 'win32') {
    const fs = require('fs');
    const path = require('path');
    const Module = require('module');


    const originalResolveFilename = Module._resolveFilename;

    function findPackage(request, parent) {
        if (parent && Array.isArray(parent.paths)) {
            for (const lookupPath of parent.paths) {
                const candidate = path.join(lookupPath, request);
                if (fs.existsSync(candidate)) {
                    return candidate;
                }
                try {
                    const link = fs.readlinkSync(candidate);
                    return link;
                } catch (_) {

                }
            }
        }
    }

    // 
    Module._resolveFilename = function (request, parent, isMain, options) {

        try {
            return originalResolveFilename.call(this, request, parent, isMain, options)
        } catch (e) {
            const normalizedRequest = request.split('/').join('\\');
            let result = findPackage(request, parent);
            if (result.endsWith(normalizedRequest)) {
                result = result.slice(0, -normalizedRequest.length);
            }
            const original = parent.paths;
            parent.paths = [result, ...original];
            return Module._resolveFilename.call(this, request, parent, isMain, options);
        }
    };
}

const { nodeResolve } = require('@rollup/plugin-node-resolve')


function findTauriApiPath() {
    if (process.platform === 'win32') {
        const tauriApiPath = require.resolve('@tauri-apps/api');
        return [path.dirname(path.dirname(path.dirname(tauriApiPath)))]
    }
    return []
}

module.exports = {
    plugins: [
        nodeResolve({
            moduleDirectories: findTauriApiPath()
        }),
    ],
}