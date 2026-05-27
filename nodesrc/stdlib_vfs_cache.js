// nodesrc/stdlib_vfs_cache.js
// Process-local cache for stdlib VFS objects used by compile-heavy tools.

const fs = require('node:fs');
const path = require('node:path');

const stdlibVfsCache = new Map();
const stdlibNewestMtimeCache = new Map();

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

function newestStdlibMtimeMs(stdlibRootDir, options = {}) {
    const root = path.resolve(stdlibRootDir || path.resolve(process.cwd(), 'stdlib'));
    const missing = options.missing || 'throw';
    const cached = stdlibNewestMtimeCache.get(root);
    if (cached !== undefined) return cached;

    if (!isDir(root)) {
        if (missing === 'empty') {
            stdlibNewestMtimeCache.set(root, 0);
            return 0;
        }
        throw new Error(`stdlib root not found: ${root}`);
    }

    let newest = 0;
    for (const f of walkFiles(root)) {
        if (!f.endsWith('.nepl')) continue;
        const stat = fs.statSync(f);
        newest = Math.max(newest, stat.mtimeMs);
    }
    stdlibNewestMtimeCache.set(root, newest);
    return newest;
}

function stdlibOverrideIsNewerThanArtifact(stdlibRootDir, artifactPath, options = {}) {
    if (!artifactPath) return true;
    let artifactStat = null;
    try {
        artifactStat = fs.statSync(artifactPath);
    } catch {
        return true;
    }
    const newestStdlib = newestStdlibMtimeMs(stdlibRootDir, options);
    return newestStdlib > artifactStat.mtimeMs + 1;
}

function clearStdlibVfsCacheForTests() {
    stdlibVfsCache.clear();
    stdlibNewestMtimeCache.clear();
}

function stdlibVfsCacheSizeForTests() {
    return stdlibVfsCache.size;
}

module.exports = {
    loadStdlibVfsFromFs,
    newestStdlibMtimeMs,
    stdlibOverrideIsNewerThanArtifact,
    clearStdlibVfsCacheForTests,
    stdlibVfsCacheSizeForTests,
};
