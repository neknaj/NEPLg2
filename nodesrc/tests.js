#!/usr/bin/env node
// nodesrc/tests.js
// 目的:
// - /tests/*.n.md, /tutorials/**/*.n.md, /stdlib/**/*.nepl などに埋め込まれた doctest を走査して実行し、結果を JSON にまとめる。
//
// 使い方例:
//   node nodesrc/tests.js -i tests -i tutorials -i stdlib -o dist/tests.json
//
// 注意:
// - dist の場所は自動探索（dist / web/dist）だが、NEPL_DIST 環境変数で強制できる。

const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { spawn } = require('node:child_process');
const { parseFile } = require('./parser');

function parseArgs(argv) {
    const inputs = [];
    let outPath = '';
    let distHint = '';
    let jobs = 0;

    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if (a === '-i' && i + 1 < argv.length) {
            inputs.push(argv[++i]);
            continue;
        }
        if (a === '-o' && i + 1 < argv.length) {
            outPath = argv[++i];
            continue;
        }
        if ((a === '--dist' || a === '--dist-hint') && i + 1 < argv.length) {
            distHint = argv[++i];
            continue;
        }
        if ((a === '-j' || a === '--jobs') && i + 1 < argv.length) {
            jobs = parseInt(argv[++i], 10);
            continue;
        }
        if (a === '-h' || a === '--help') {
            return { help: true, inputs, outPath, distHint, jobs };
        }
    }

    if (jobs <= 0) {
        // GH Actions でも暴れない程度
        jobs = Math.max(1, Math.min(8, Math.floor((os.cpus()?.length || 4) / 2)));
    }

    return { help: false, inputs, outPath, distHint, jobs };
}

function isDir(p) {
    try { return fs.statSync(p).isDirectory(); } catch { return false; }
}
function isFile(p) {
    try { return fs.statSync(p).isFile(); } catch { return false; }
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

function collectTestsFromPath(inputPath) {
    const abs = path.resolve(inputPath);
    const cases = [];

    const files = [];
    if (isFile(abs)) {
        files.push(abs);
    } else if (isDir(abs)) {
        for (const f of walkFiles(abs)) {
            if (f.endsWith('.n.md') || f.endsWith('.nepl')) files.push(f);
        }
    } else {
        return cases;
    }

    for (const f of files) {
        const p = parseFile(f);
        for (let i = 0; i < p.doctests.length; i++) {
            const dt = p.doctests[i];
            cases.push({
                id: `${path.relative(process.cwd(), f)}::doctest#${i + 1}`,
                file: path.relative(process.cwd(), f),
                index: i + 1,
                tags: dt.tags,
                source: dt.code,
            });
        }
    }

    return cases;
}

function runOne(caseObj, distHint) {
    return new Promise((resolve) => {
        const child = spawn(process.execPath, [path.join(__dirname, 'run_test.js')], {
            stdio: ['pipe', 'pipe', 'pipe'],
        });

        let stdout = '';
        let stderr = '';
        child.stdout.on('data', (c) => { stdout += c.toString('utf-8'); });
        child.stderr.on('data', (c) => { stderr += c.toString('utf-8'); });

        child.on('close', (code) => {
            let parsed = null;
            try {
                parsed = JSON.parse(stdout);
            } catch {
                parsed = { ok: false, status: 'error', error: 'invalid json from run_test.js', raw_stdout: stdout, raw_stderr: stderr };
            }

            resolve({
                ...parsed,
                id: caseObj.id,
                file: caseObj.file,
                index: caseObj.index,
                tags: caseObj.tags,
                exit_code: code,
                runner_stderr: stderr,
            });
        });

        const req = {
            id: caseObj.id,
            source: caseObj.source,
            tags: caseObj.tags,
            stdin: '',
            distHint,
        };
        child.stdin.write(JSON.stringify(req));
        child.stdin.end();
    });
}

async function runAll(cases, jobs, distHint) {
    const results = [];
    let idx = 0;

    async function worker() {
        while (true) {
            const i = idx;
            idx++;
            if (i >= cases.length) break;
            const r = await runOne(cases[i], distHint);
            results.push(r);
        }
    }

    const ws = [];
    for (let i = 0; i < jobs; i++) ws.push(worker());
    await Promise.all(ws);

    // 入力順に並べたいのでソート
    results.sort((a, b) => {
        if (a.file < b.file) return -1;
        if (a.file > b.file) return 1;
        return (a.index || 0) - (b.index || 0);
    });
    return results;
}

function summarize(results) {
    let passed = 0;
    let failed = 0;
    let errored = 0;
    for (const r of results) {
        if (r.status === 'pass') passed++;
        else if (r.status === 'fail') failed++;
        else errored++;
    }
    return {
        total: results.length,
        passed,
        failed,
        errored,
    };
}

function ensureDir(p) {
    fs.mkdirSync(p, { recursive: true });
}

async function main() {
    const { help, inputs, outPath, distHint, jobs } = parseArgs(process.argv.slice(2));
    if (help || inputs.length === 0 || !outPath) {
        console.log('Usage: node nodesrc/tests.js -i <dir_or_file> [-i ...] -o <out.json> [--dist <distDirHint>] [-j N]');
        process.exit(help ? 0 : 2);
    }

    const allCases = [];
    for (const p of inputs) {
        allCases.push(...collectTestsFromPath(p));
    }

    const results = await runAll(allCases, jobs, distHint);
    const summary = summarize(results);

    const out = {
        schema: 'neplg2-doctest/v1',
        generated_at: new Date().toISOString(),
        jobs,
        dist_hint: distHint || null,
        summary,
        results,
    };

    const outAbs = path.resolve(outPath);
    ensureDir(path.dirname(outAbs));
    fs.writeFileSync(outAbs, JSON.stringify(out, null, 2));

    // CI で見やすいように要約を stdout に出す
    console.log(`doctest: total=${summary.total} passed=${summary.passed} failed=${summary.failed} errored=${summary.errored}`);

    // 失敗があれば exit code を 1 にする（gh-pages では continue-on-error で許可する想定）
    if (summary.failed > 0 || summary.errored > 0) {
        process.exitCode = 1;
    }
}

if (require.main === module) {
    main().catch((e) => {
        console.error(String(e?.stack || e?.message || e));
        process.exit(1);
    });
}
