// nodesrc/stdlib_vfs_cache.js
// Process-local cache for stdlib VFS objects used by compile-heavy tools.

const fs = require('node:fs');
const path = require('node:path');

const stdlibVfsCache = new Map();
const stdlibNewestMtimeCache = new Map();
const stdlibContentHashCache = new Map();

const FNV_OFFSET_BASIS = 0xcbf29ce484222325n;
const FNV_PRIME = 0x100000001b3n;
const FNV_MASK = 0xffffffffffffffffn;

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

function fnv1aUpdate(hash, bytes) {
    let next = hash;
    for (const byte of bytes) {
        next ^= BigInt(byte);
        next = (next * FNV_PRIME) & FNV_MASK;
    }
    return next;
}

function hashStdlibFiles(root, files) {
    let hash = FNV_OFFSET_BASIS;
    const sorted = [...files].sort((a, b) => {
        const aRel = toPosixPath(path.relative(root, a));
        const bRel = toPosixPath(path.relative(root, b));
        if (aRel < bRel) return -1;
        if (aRel > bRel) return 1;
        return 0;
    });
    for (const file of sorted) {
        const rel = toPosixPath(path.relative(root, file));
        hash = fnv1aUpdate(hash, Buffer.from(rel, 'utf8'));
        hash = fnv1aUpdate(hash, Buffer.from([0]));
        hash = fnv1aUpdate(hash, fs.readFileSync(file));
        hash = fnv1aUpdate(hash, Buffer.from([0xff]));
    }
    return `fnv1a64:${hash.toString(16).padStart(16, '0')}`;
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

function stdlibContentHashFromFs(stdlibRootDir, options = {}) {
    const root = path.resolve(stdlibRootDir || path.resolve(process.cwd(), 'stdlib'));
    const missing = options.missing || 'throw';
    const cached = stdlibContentHashCache.get(root);
    if (cached !== undefined) return cached;

    if (!isDir(root)) {
        if (missing === 'empty') {
            const emptyHash = hashStdlibFiles(root, []);
            stdlibContentHashCache.set(root, emptyHash);
            return emptyHash;
        }
        throw new Error(`stdlib root not found: ${root}`);
    }

    const files = walkFiles(root).filter((f) => {
        if (!f.endsWith('.nepl')) return false;
        const rel = toPosixPath(path.relative(root, f));
        return !rel.startsWith('tests/') && !rel.startsWith('tests_backup/');
    });
    const hash = hashStdlibFiles(root, files);
    stdlibContentHashCache.set(root, hash);
    return hash;
}

function stdlibOverrideIsNewerThanArtifact(stdlibRootDir, artifactPath, options = {}) {
    const artifactHash = options.artifactHash || null;
    if (artifactHash) {
        return stdlibContentHashFromFs(stdlibRootDir, options) !== artifactHash;
    }
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
    stdlibContentHashCache.clear();
}

function stdlibVfsCacheSizeForTests() {
    return stdlibVfsCache.size;
}

module.exports = {
    loadStdlibVfsFromFs,
    newestStdlibMtimeMs,
    stdlibContentHashFromFs,
    stdlibOverrideIsNewerThanArtifact,
    clearStdlibVfsCacheForTests,
    stdlibVfsCacheSizeForTests,
};
