#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

function parseArgs(argv) {
    let outPath = '';
    const inputs = [];
    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if ((a === '-o' || a === '--output') && i + 1 < argv.length) {
            outPath = argv[++i];
            continue;
        }
        if (a === '-h' || a === '--help') {
            return { help: true, outPath, inputs };
        }
        inputs.push(a);
    }
    return { help: false, outPath, inputs };
}

function isFile(p) {
    try { return fs.statSync(p).isFile(); } catch { return false; }
}

function ensureDir(p) {
    fs.mkdirSync(p, { recursive: true });
}

function readReport(p) {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
}

function isTimeoutResult(result) {
    return Boolean(result && typeof result.timeout === 'object' && result.timeout !== null);
}

function summarize(results) {
    let passed = 0;
    let failed = 0;
    let errored = 0;
    let timedOut = 0;
    let timeoutFailed = 0;
    let timeoutErrored = 0;
    let nonTimeoutFailed = 0;
    let nonTimeoutErrored = 0;
    for (const r of results) {
        const isTimeout = isTimeoutResult(r);
        if (isTimeout) timedOut++;
        if (r.status === 'pass') {
            passed++;
        } else if (r.status === 'fail') {
            failed++;
            if (isTimeout) timeoutFailed++;
            else nonTimeoutFailed++;
        } else {
            errored++;
            if (isTimeout) timeoutErrored++;
            else nonTimeoutErrored++;
        }
    }
    return {
        total: results.length,
        passed,
        failed,
        errored,
        timed_out: timedOut,
        timeout_failed: timeoutFailed,
        timeout_errored: timeoutErrored,
        non_timeout_failed: nonTimeoutFailed,
        non_timeout_errored: nonTimeoutErrored,
    };
}

function uniqueSortedStrings(values) {
    return Array.from(new Set(values.filter((v) => typeof v === 'string' && v.length > 0))).sort();
}

function compareResultOrder(a, b) {
    const af = String(a?.file || '');
    const bf = String(b?.file || '');
    if (af < bf) return -1;
    if (af > bf) return 1;
    const ai = Number(a?.index || 0);
    const bi = Number(b?.index || 0);
    if (ai !== bi) return ai - bi;
    const aid = String(a?.id || '');
    const bid = String(b?.id || '');
    return aid < bid ? -1 : aid > bid ? 1 : 0;
}

function mergeReports(inputPaths) {
    const existingInputs = inputPaths.filter(isFile);
    const reports = existingInputs.map(readReport);
    const results = reports.flatMap((report) => Array.isArray(report?.results) ? report.results : []);
    results.sort(compareResultOrder);
    const resolvedDistDirs = uniqueSortedStrings(reports.flatMap((report) => report?.resolved_dist_dirs || []));
    return {
        schema: 'neplg2-doctest/v1',
        generated_at: new Date().toISOString(),
        merged: true,
        partial: reports.some((report) => report?.partial === true),
        inputs: existingInputs.map((p) => path.relative(process.cwd(), path.resolve(p))),
        shards: reports.map((report, index) => ({
            input: path.relative(process.cwd(), path.resolve(existingInputs[index])),
            summary: report?.summary || summarize(Array.isArray(report?.results) ? report.results : []),
            scan: report?.scan || null,
            partial: report?.partial === true,
        })),
        resolved_dist_dirs: resolvedDistDirs,
        summary: summarize(results),
        results,
    };
}

function main() {
    const { help, outPath, inputs } = parseArgs(process.argv.slice(2));
    if (help || !outPath || inputs.length === 0) {
        console.log('Usage: node nodesrc/merge_doctest_json.js -o <out.json> <input.json>...');
        process.exit(help ? 0 : 2);
    }
    const outAbs = path.resolve(outPath);
    ensureDir(path.dirname(outAbs));
    fs.writeFileSync(outAbs, JSON.stringify(mergeReports(inputs), null, 2));
}

main();
