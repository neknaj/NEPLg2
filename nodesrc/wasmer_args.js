const { spawnSync } = require('node:child_process');

const helpCache = new Map();

function wasmerRunHelp(wasmerBin) {
    const key = String(wasmerBin || 'wasmer');
    if (helpCache.has(key)) return helpCache.get(key);
    const cp = spawnSync(key, ['run', '--help'], {
        encoding: 'utf8',
        maxBuffer: 1024 * 1024,
        timeout: 5000,
    });
    const help = `${cp.stdout || ''}\n${cp.stderr || ''}`;
    helpCache.set(key, help);
    return help;
}

function wasmerRunMountArgs(wasmerBin, hostDir, guestDir = hostDir) {
    const help = wasmerRunHelp(wasmerBin);
    if (/\s--volume(?:[=\s<]|$)/.test(help)) {
        return [`--volume=${hostDir}:${guestDir}`];
    }
    if (/\s--mapdir(?:[=\s<]|$)/.test(help)) {
        const sep = String(hostDir).includes(':') || String(guestDir).includes(':') ? '::' : ':';
        return [`--mapdir=${guestDir}${sep}${hostDir}`];
    }
    if (/\s--dir(?:[=\s<]|$)/.test(help)) {
        return [`--dir=${hostDir}`];
    }
    return [`--volume=${hostDir}:${guestDir}`];
}

module.exports = {
    wasmerRunMountArgs,
};
