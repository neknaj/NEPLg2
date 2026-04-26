// nodesrc/stdlib_vfs_cache.js
// Process-local cache for stdlib VFS objects used by compile-heavy tools.

const fs = require('node:fs');
const path = require('node:path');

const stdlibVfsCache = new Map();

function toPosixPath(p) {
    return String(p).replace(/\\/g, '/');
}

function isDir(p) {
    try {
        return fs.statSync(p).isDirectory();
    } catch {
        return false;
    }
}

function walkFiles(root) {
    const out = [];
    function rec(cur) {
        const ents = fs.readdirSync(cur, { withFileTypes: true });
        for (const e of ents) {
            const p = path.join(cur, e.name);
            if (e.isDirectory()) rec(p);
            else if (e.isFile()) out.push(p);
        }
    }
    rec(root);
    return out;
}

function loadStdlibVfsFromFs(stdlibRootDir, options = {}) {
    const root = path.resolve(stdlibRootDir || path.resolve(process.cwd(), 'stdlib'));
    const missing = options.missing || 'throw';
    const cached = stdlibVfsCache.get(root);
    if (cached) return cached;

    if (!isDir(root)) {
        if (missing === 'empty') {
            return {};
        }
        throw new Error(`stdlib root not found: ${root}`);
    }

    const out = {};
    for (const f of walkFiles(root)) {
        if (!f.endsWith('.nepl')) continue;
        const rel = toPosixPath(path.relative(root, f));
        out[`/stdlib/${rel}`] = fs.readFileSync(f, 'utf8');
    }
    stdlibVfsCache.set(root, out);
    return out;
}

function clearStdlibVfsCacheForTests() {
    stdlibVfsCache.clear();
}

function stdlibVfsCacheSizeForTests() {
    return stdlibVfsCache.size;
}

module.exports = {
    loadStdlibVfsFromFs,
    clearStdlibVfsCacheForTests,
    stdlibVfsCacheSizeForTests,
};
