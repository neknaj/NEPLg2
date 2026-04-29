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
        expected_exit_code: Object.prototype.hasOwnProperty.call(dt, 'exit_code') ? dt.exit_code : null,
        expected_diag_codes: Array.isArray(dt.diag_codes) ? dt.diag_codes : [],
        expected_diag_spans: Array.isArray(dt.diag_spans) ? dt.diag_spans : [],
    };
}

function normalizeOutputByTags(s, tags) {
    let out = String(s ?? '');
    if (Array.isArray(tags) && tags.includes('normalize_newlines')) {
        out = out.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
    }
    if (Array.isArray(tags) && tags.includes('strip_ansi')) {
        out = out.replace(/\x1b\[[0-?]*[ -/]*[@-~]/g, '');
    }
    if (Array.isArray(tags) && tags.includes('trim_stdout')) {
        out = out.trim();
    }
    return out;
}

function stripAnsi(s) {
    return String(s || '').replace(/\x1b\[[0-?]*[ -/]*[@-~]/g, '');
}

function extractActualDiagCodes(compileErrorText) {
    const codes = [];
    const re = /(?:error|warning)\[([a-z][a-z0-9_.]*)\]/g;
    let m;
    while ((m = re.exec(String(compileErrorText || ''))) !== null) {
        codes.push(m[1]);
    }
    return codes;
}

function extractActualDiagSpans(compileErrorText) {
    const out = [];
    const lines = stripAnsi(compileErrorText).replace(/\r\n/g, '\n').split('\n');
    for (let i = 0; i < lines.length; i++) {
        const m = lines[i].match(/^\s*-->\s+(.+)\s*$/);
        if (!m) continue;
        const loc = String(m[1] || '').trim();
        const lm = loc.match(/^(.*):(\d+):(\d+)$/);
        if (!lm) continue;
        out.push({
            file: String(lm[1] || '').trim(),
            line: parseInt(lm[2], 10),
            col: parseInt(lm[3], 10),
        });
    }
    return out;
}

function hasActualExitCode(result) {
    return Object.prototype.hasOwnProperty.call(result, 'exit_code')
        && result.exit_code !== null
        && result.exit_code !== undefined;
}

function applyExpectations(result, testCase) {
    const r = { ...result };
    const tags = Array.isArray(testCase.tags) ? testCase.tags : [];
    const importsStdTest = /#import\s+"std\/test"\s+as\s+\*/.test(String(testCase.source || ''));

    if (tags.includes('compile_fail')) {
        const compileError = String(r.compile_error || '');
        if (!compileError) return r;
        if (testCase.expected_diag_codes.length > 0) {
            const actualCodes = extractActualDiagCodes(compileError);
            const missing = testCase.expected_diag_codes.filter((code) => !actualCodes.includes(code));
            if (missing.length > 0) {
                r.ok = false;
                r.status = 'fail';
                r.error = [
                    'compile_fail diagnostic code mismatch',
                    `expected codes: ${JSON.stringify(testCase.expected_diag_codes)}`,
                    `missing codes: ${JSON.stringify(missing)}`,
                    `actual codes: ${JSON.stringify(actualCodes)}`,
                ].join('\n');
                return r;
            }
        }
        if (testCase.expected_diag_spans.length > 0) {
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
        return r;
    }

    if (tags.includes('should_panic')) {
        return r;
    }

    if (Object.prototype.hasOwnProperty.call(testCase, 'expected_exit_code')
        && testCase.expected_exit_code !== null
        && testCase.expected_exit_code !== undefined) {
        const expected = testCase.expected_exit_code;
        if (!hasActualExitCode(r)) {
            r.ok = false;
            r.status = 'fail';
            r.error = [
                'exit code result missing',
                `expected: ${JSON.stringify(expected)}`,
            ].join('\n');
            return r;
        }
        const actual = r.exit_code;
        if (expected !== actual) {
            r.ok = false;
            r.status = 'fail';
            r.error = [
                'exit code mismatch',
                `expected: ${JSON.stringify(expected)}`,
                `actual:   ${JSON.stringify(actual)}`,
            ].join('\n');
            return r;
        }
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
        expected_exit_code: testCase.expected_exit_code,
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
