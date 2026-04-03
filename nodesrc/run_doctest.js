#!/usr/bin/env node
// nodesrc/run_doctest.js
// 目的:
// - 1 ファイルに含まれる doctest 1 件を直接実行し、原因切り分けを高速に行う。
// - stdlib reboot 中に `nodesrc/tests.js` 全体集計を回さずに focused 確認できる入口を提供する。

const path = require('node:path');
const { parseFile } = require('./parser');
const { runSingle } = require('./run_test');

function parseArgs(argv) {
    let inputPath = '';
    let index = 1;
    let distHint = '';

    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if ((a === '-i' || a === '--input') && i + 1 < argv.length) {
            inputPath = argv[++i];
            continue;
        }
        if ((a === '-n' || a === '--index') && i + 1 < argv.length) {
            index = parseInt(argv[++i], 10);
            continue;
        }
        if (a === '--dist' && i + 1 < argv.length) {
            distHint = argv[++i];
            continue;
        }
        if (a === '-h' || a === '--help') {
            return { help: true, inputPath, index, distHint };
        }
    }

    return { help: false, inputPath, index, distHint };
}

function usage() {
    console.log('Usage: node nodesrc/run_doctest.js -i <file.nepl|file.n.md> [-n <index>] [--dist <dir>]');
}

function buildCase(inputPath, index) {
    const abs = path.resolve(inputPath);
    const parsed = parseFile(abs);
    if (!Array.isArray(parsed.doctests) || parsed.doctests.length === 0) {
        throw new Error(`no doctest found: ${abs}`);
    }
    if (!Number.isFinite(index) || index < 1 || index > parsed.doctests.length) {
        throw new Error(`doctest index out of range: ${index} (1..${parsed.doctests.length})`);
    }

    const dt = parsed.doctests[index - 1];
    return {
        id: `${path.relative(process.cwd(), abs)}::doctest#${index}`,
        file: path.relative(process.cwd(), abs),
        source: dt.code,
        tags: Array.isArray(dt.tags) ? dt.tags : [],
        stdin: dt.stdin || '',
        argv: Array.isArray(dt.argv) ? dt.argv.map((v) => String(v)) : [],
        expected_stdout: dt.stdout ?? null,
        expected_stderr: dt.stderr ?? null,
        expected_ret: Object.prototype.hasOwnProperty.call(dt, 'ret') ? dt.ret : null,
        expected_diag_ids: Array.isArray(dt.diag_ids) ? dt.diag_ids : [],
        expected_diag_spans: Array.isArray(dt.diag_spans) ? dt.diag_spans : [],
    };
}

function normalizeOutputByTags(s, tags) {
    let out = String(s ?? '');
    if (Array.isArray(tags) && tags.includes('normalize_newlines')) {
        out = out.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
    }
    if (Array.isArray(tags) && tags.includes('strip_ansi')) {
        out = out.replace(/\x1b\[[0-9;]*m/g, '');
    }
    if (Array.isArray(tags) && tags.includes('trim_stdout')) {
        out = out.trim();
    }
    return out;
}

function extractActualDiagIds(compileErrorText) {
    const ids = [];
    const re = /error\[D(\d+)\]/g;
    let m;
    while ((m = re.exec(String(compileErrorText || ''))) !== null) {
        ids.push(parseInt(m[1], 10));
    }
    return ids;
}

function extractActualDiagSpans(compileErrorText) {
    const out = [];
    const lines = String(compileErrorText || '').split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const m = lines[i].match(/^\s*-->\s+([^:]+):(\d+):(\d+)\s*$/);
        if (!m) continue;
        out.push({
            file: String(m[1]).trim(),
            line: parseInt(m[2], 10),
            col: parseInt(m[3], 10),
        });
    }
    return out;
}

function applyExpectations(result, testCase) {
    const r = { ...result };
    const tags = Array.isArray(testCase.tags) ? testCase.tags : [];
    const importsStdTest = /#import\s+"std\/test"\s+as\s+\*/.test(String(testCase.source || ''));

    if (tags.includes('compile_fail')) {
        if (!r.ok) {
            const compileError = String(r.compile_error || r.error || '');
            if (testCase.expected_diag_ids.length > 0) {
                const actualIds = extractActualDiagIds(compileError);
                const missing = testCase.expected_diag_ids.filter((id) => !actualIds.includes(id));
                if (missing.length > 0) {
                    r.ok = false;
                    r.status = 'fail';
                    r.error = [
                        'compile_fail diagnostic id mismatch',
                        `expected ids: ${JSON.stringify(testCase.expected_diag_ids)}`,
                        `missing ids: ${JSON.stringify(missing)}`,
                        `actual ids: ${JSON.stringify(actualIds)}`,
                    ].join('\n');
                }
            }
            if (r.ok && testCase.expected_diag_spans.length > 0) {
                const actualSpans = extractActualDiagSpans(compileError);
                const missing = testCase.expected_diag_spans.filter((want) => {
                    return !actualSpans.some((got) => {
                        const wantFile = want.file ? String(want.file).replace(/\\/g, '/') : null;
                        const gotFile = got.file ? String(got.file).replace(/\\/g, '/') : null;
                        return (!wantFile || wantFile === gotFile) && want.line === got.line && want.col === got.col;
                    });
                });
                if (missing.length > 0) {
                    r.ok = false;
                    r.status = 'fail';
                    r.error = [
                        'compile_fail diagnostic span mismatch',
                        `expected spans: ${JSON.stringify(testCase.expected_diag_spans)}`,
                        `missing spans: ${JSON.stringify(missing)}`,
                        `actual spans: ${JSON.stringify(actualSpans)}`,
                    ].join('\n');
                }
            }
        }
        return r;
    }

    if (tags.includes('should_panic')) {
        return r;
    }

    if (Object.prototype.hasOwnProperty.call(testCase, 'expected_ret')
        && testCase.expected_ret !== null
        && testCase.expected_ret !== undefined) {
        const expected = testCase.expected_ret;
        const actual = Object.prototype.hasOwnProperty.call(r, 'return_value') ? r.return_value : null;
        if (expected !== actual) {
            r.ok = false;
            r.status = 'fail';
            r.error = [
                'return value mismatch',
                `expected: ${JSON.stringify(expected)}`,
                `actual:   ${JSON.stringify(actual)}`,
            ].join('\n');
            return r;
        }
    }

    if (testCase.expected_stdout !== null) {
        const expected = normalizeOutputByTags(testCase.expected_stdout, tags);
        const actual = normalizeOutputByTags(r.stdout || '', tags);
        if (expected !== actual) {
            r.ok = false;
            r.status = 'fail';
            r.error = [
                'stdout mismatch',
                `expected: ${JSON.stringify(expected)}`,
                `actual:   ${JSON.stringify(actual)}`,
            ].join('\n');
            return r;
        }
    }

    if (testCase.expected_stderr !== null) {
        const expected = normalizeOutputByTags(testCase.expected_stderr, tags);
        const actual = normalizeOutputByTags(r.stderr || '', tags);
        if (expected !== actual) {
            r.ok = false;
            r.status = 'fail';
            r.error = [
                'stderr mismatch',
                `expected: ${JSON.stringify(expected)}`,
                `actual:   ${JSON.stringify(actual)}`,
            ].join('\n');
            return r;
        }
    }

    if (importsStdTest && testCase.expected_stdout === null) {
        const actual = normalizeOutputByTags(r.stdout || '', tags);
        if (/^FAIL:/m.test(actual)) {
            r.ok = false;
            r.status = 'fail';
            r.error = 'std/test reported FAIL output';
            return r;
        }
    }

    return r;
}

async function main() {
    const { help, inputPath, index, distHint } = parseArgs(process.argv.slice(2));
    if (help || !inputPath) {
        usage();
        process.exit(help ? 0 : 2);
    }

    const testCase = buildCase(inputPath, index);
    const raw = await runSingle({
        id: testCase.id,
        file: testCase.file,
        source: testCase.source,
        tags: testCase.tags,
        stdin: testCase.stdin,
        argv: testCase.argv,
        expected_ret: testCase.expected_ret,
        distHint,
    });
    const result = applyExpectations(raw, testCase);
    process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
    if (!result.ok) process.exitCode = 1;
}

if (require.main === module) {
    main().catch((e) => {
        process.stderr.write(`${String(e?.stack || e?.message || e)}\n`);
        process.exit(1);
    });
}
