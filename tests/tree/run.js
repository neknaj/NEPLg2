#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');
const { loadApi } = require('./_shared');

function isTestFile(name) {
    return name.endsWith('.js') && name !== 'run.js' && name !== '_shared.js';
}

async function main() {
    const dir = __dirname;
    const files = fs
        .readdirSync(dir)
        .filter(isTestFile)
        .sort();

    const api = await loadApi(process.env.NEPL_DIST || '');
    const results = [];

    for (const file of files) {
        const modPath = path.join(dir, file);
        const mod = require(modPath);
        const id = String(mod?.id || file);
        const run = typeof mod === 'function' ? mod : mod?.run;
        if (typeof run !== 'function') {
            results.push({
                id,
                status: 'error',
                error: 'test module must export run(api) function',
            });
            continue;
        }

        try {
            const detail = await run(api);
            results.push({ id, status: 'pass', detail: detail || null });
        } catch (e) {
            results.push({
                id,
                status: 'fail',
                error: String(e?.stack || e?.message || e),
            });
        }
    }

    const passed = results.filter((r) => r.status === 'pass').length;
    const failed = results.filter((r) => r.status === 'fail').length;
    const errored = results.filter((r) => r.status === 'error').length;
    const out = {
        summary: {
            total: results.length,
            passed,
            failed,
            errored,
        },
        results,
    };

    console.log(JSON.stringify(out, null, 2));
    if (failed > 0 || errored > 0) {
        process.exitCode = 1;
    }
}

if (require.main === module) {
    main().catch((e) => {
        console.error(String(e?.stack || e?.message || e));
        process.exit(1);
    });
}
