#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { parseNmdText, parseNeplText } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const activeRoots = ['tests', 'tutorials', 'stdlib', 'examples'];

function walkActiveDocs(dir, out = []) {
    if (!fs.existsSync(dir)) {
        return out;
    }
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            walkActiveDocs(full, out);
        } else if (entry.name.endsWith('.n.md') || entry.name.endsWith('.nepl')) {
            out.push(full);
        }
    }
    return out;
}

function findLegacyDiagnosticMetadata(text) {
    const legacyIdMetadata = /\bdiag_ids?\s*:/;
    const numericDiagCode = /\bdiag_codes?\s*:\s*(?:\[\s*)?(?:"?D?\d{3,4}"?|\d{3,4})\b/i;
    return text
        .split(/\r?\n/)
        .map((line, index) => ({ line, lineNumber: index + 1 }))
        .filter(({ line }) => legacyIdMetadata.test(line) || numericDiagCode.test(line));
}

const nmdSource = `# doctest diag metadata

neplg2:test[compile_fail]
diag_code: type.return.mismatch
diag_codes: ["parser.token.expected", "resolve.identifier.undefined"]
diag_codes: type.stack.extra_values, type.annotation.mismatch
diag_span: 4:5
\`\`\`neplg2
#entry main
#target wasm
fn main <()->i32> ():
    true
\`\`\`
`;

{
    const parsed = parseNmdText(nmdSource);
    assert.equal(parsed.doctests.length, 1);
    const dt = parsed.doctests[0];
    assert.deepEqual(dt.diag_codes, [
        'type.return.mismatch',
        'parser.token.expected',
        'resolve.identifier.undefined',
        'type.stack.extra_values',
        'type.annotation.mismatch',
    ]);
    assert.equal(Object.prototype.hasOwnProperty.call(dt, 'diag_ids'), false);
    assert.deepEqual(dt.diag_spans, [{ file: null, line: 4, col: 5 }]);
}

const neplDocSource = `//: neplg2:test[compile_fail]
//: diag_code: lexer.string.unterminated
//: diag_codes: lexer.string.invalid_escape, parser.token.unexpected
//: \`\`\`neplg2
//:| #entry main
//:| #target wasm
//: fn main <()->i32> ():
//:     "unterminated
//: \`\`\`
`;

{
    const parsed = parseNeplText(neplDocSource);
    assert.equal(parsed.doctests.length, 1);
    const dt = parsed.doctests[0];
    assert.deepEqual(dt.diag_codes, [
        'lexer.string.unterminated',
        'lexer.string.invalid_escape',
        'parser.token.unexpected',
    ]);
    assert.equal(Object.prototype.hasOwnProperty.call(dt, 'diag_ids'), false);
}

{
    const good = `neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
diag_codes: ["type.return.mismatch", "parser.token.expected"]
\`\`\`neplg2
\`\`\`
`;
    const bad = `neplg2:test[compile_fail]
diag_id: 3092
diag_code: 3092
diag_codes: ["D3004"]
\`\`\`neplg2
\`\`\`
`;
    assert.deepEqual(findLegacyDiagnosticMetadata(good), []);
    assert.deepEqual(findLegacyDiagnosticMetadata(bad).map((hit) => hit.lineNumber), [2, 3, 4]);
}

const legacyViolations = [];
for (const root of activeRoots) {
    for (const file of walkActiveDocs(path.join(repoRoot, root))) {
        const text = fs.readFileSync(file, 'utf8');
        for (const hit of findLegacyDiagnosticMetadata(text)) {
            legacyViolations.push(
                `${path.relative(repoRoot, file)}:${hit.lineNumber}: ${hit.line.trim()}`,
            );
        }
    }
}

assert.deepEqual(
    legacyViolations,
    [],
    `active doctests must use stable string diag_code values, not legacy numeric diagnostic IDs:\n${legacyViolations.join('\n')}`,
);

const distDir = path.join(repoRoot, 'web', 'dist');
if (fs.existsSync(distDir)) {
    const tmpDir = path.join(repoRoot, 'tmp');
    fs.mkdirSync(tmpDir, { recursive: true });
    const goodFixture = path.join(tmpDir, 'doctest-diag-code-good.n.md');
    const badFixture = path.join(tmpDir, 'doctest-diag-code-bad.n.md');
    const baseCase = `neplg2:test[compile_fail]
diag_code: TYPE_CODE_PLACEHOLDER
diag_span: 4:5
\`\`\`neplg2
#entry main
#target wasm
fn main <()->i32> ():
    true
\`\`\`
`;
    fs.writeFileSync(goodFixture, baseCase.replace('TYPE_CODE_PLACEHOLDER', 'type.return.mismatch'));
    fs.writeFileSync(badFixture, baseCase.replace('TYPE_CODE_PLACEHOLDER', 'parser.token.unexpected'));

    const runDoctest = (fixture) => spawnSync(process.execPath, [
        path.join(__dirname, 'run_doctest.js'),
        '-i',
        fixture,
        '-n',
        '1',
        '--dist',
        distDir,
    ], {
        cwd: repoRoot,
        encoding: 'utf8',
        env: {
            ...process.env,
            NO_COLOR: 'true',
        },
    });
    const runAggregate = (fixture, outputPath) => spawnSync(process.execPath, [
        path.join(__dirname, 'tests.js'),
        '-i',
        fixture,
        '--no-tree',
        '-o',
        outputPath,
        '-j',
        '1',
        '--dist',
        distDir,
    ], {
        cwd: repoRoot,
        encoding: 'utf8',
        env: {
            ...process.env,
            NO_COLOR: 'true',
        },
    });

    const good = runDoctest(goodFixture);
    assert.equal(
        good.status,
        0,
        `expected matching diag_code to pass\nstdout:\n${good.stdout}\nstderr:\n${good.stderr}`,
    );

    const bad = runDoctest(badFixture);
    assert.equal(
        bad.status,
        1,
        `expected mismatching diag_code to fail\nstdout:\n${bad.stdout}\nstderr:\n${bad.stderr}`,
    );
    assert.match(bad.stdout, /compile_fail diagnostic code mismatch/);
    assert.match(bad.stdout, /parser\.token\.unexpected/);
    assert.match(bad.stdout, /type\.return\.mismatch/);

    const aggregateGoodOut = path.join(tmpDir, 'doctest-diag-code-good-tests.json');
    const aggregateGood = runAggregate(goodFixture, aggregateGoodOut);
    assert.equal(
        aggregateGood.status,
        0,
        `expected aggregate runner with matching diag_code to pass\nstdout:\n${aggregateGood.stdout}\nstderr:\n${aggregateGood.stderr}`,
    );
    const aggregateGoodJson = JSON.parse(fs.readFileSync(aggregateGoodOut, 'utf8'));
    assert.equal(aggregateGoodJson.summary.passed, 1);
    assert.equal(aggregateGoodJson.summary.failed, 0);

    const aggregateBadOut = path.join(tmpDir, 'doctest-diag-code-bad-tests.json');
    const aggregateBad = runAggregate(badFixture, aggregateBadOut);
    assert.equal(
        aggregateBad.status,
        1,
        `expected aggregate runner with mismatching diag_code to fail\nstdout:\n${aggregateBad.stdout}\nstderr:\n${aggregateBad.stderr}`,
    );
    const aggregateBadJson = JSON.parse(fs.readFileSync(aggregateBadOut, 'utf8'));
    assert.equal(aggregateBadJson.summary.passed, 0);
    assert.equal(aggregateBadJson.summary.failed, 1);
    assert.match(aggregateBadJson.results[0].error, /compile_fail diagnostic code mismatch/);
    assert.match(aggregateBadJson.results[0].error, /parser\.token\.unexpected/);
    assert.match(aggregateBadJson.results[0].compile_error, /type\.return\.mismatch/);
}

console.log('doctest diag_code metadata regression passed');
