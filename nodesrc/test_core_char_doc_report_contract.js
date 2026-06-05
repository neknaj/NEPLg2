#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');
const charPath = path.join(repoRoot, 'stdlib', 'core', 'char.nepl');
const charSource = fs.readFileSync(charPath, 'utf8');
const parsed = parseFile(charPath);

assert.ok(parsed.doctests.length >= 1, 'core/char must keep its public doc-comment doctest');

const doctest = parsed.doctests[0];
const name = 'core_char_basic';

assert.equal(doctest.ret, null, `${name} must not use ret as test-success metadata`);
assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
assert.match(
    doctest.stdout,
    /^test_report name="core_char_basic" count=13 failed=0\n/,
    `${name} must pin canonical stdout report`,
);
for (const label of [
    'ascii byte 0',
    'ascii byte 1 none',
    'hiragana byte 2',
    'negative byte none',
]) {
    assert.match(doctest.stdout, new RegExp(`label="${label}"`), `${name} must report ${label}`);
}
assert.match(doctest.code, /test_report_new "core_char_basic"/, `${name} must construct a named TestReport`);
assert.match(doctest.code, /\bchar_utf8_byte_at\b/, `${name} must exercise typed UTF-8 byte access`);
assert.match(doctest.code, /\bis_none\s+char_utf8_byte_at\b/, `${name} must document absent UTF-8 bytes`);
assert.match(doctest.code, /test_report_print_stdout report/, `${name} must print the report`);
assert.match(doctest.code, /test_report_exit_code shown/, `${name} must derive exit code from the shown report`);
assert.doesNotMatch(doctest.code, /checks_exit_code\s+checks/, `${name} must not hide report details behind checks_exit_code`);

for (const [reportName, count, labels] of [
    ['char_utf8_step_new_doc', 2, ['step value', 'step next']],
    ['char_utf8_cont_byte_doc', 2, ['continuation byte', 'continuation range']],
]) {
    const reportDoctest = parsed.doctests.find((case_) => case_.code.includes(`test_report_new "${reportName}"`));
    assert.ok(reportDoctest, `${reportName} doctest must stay present`);
    assert.equal(reportDoctest.ret, null, `${reportName} must not use ret as test-success metadata`);
    assert.equal(reportDoctest.exit_code, 0, `${reportName} must pin exit_code: 0`);
    assert.deepEqual(reportDoctest.tags, ['stdio', 'normalize_newlines'], `${reportName} must normalize stdout as stdio`);
    assert.match(
        reportDoctest.stdout,
        new RegExp(`^test_report name="${reportName}" count=${count} failed=0\\n`),
        `${reportName} must pin canonical stdout report`,
    );
    for (const label of labels) {
        assert.match(reportDoctest.stdout, new RegExp(`label="${label}"`), `${reportName} must report ${label}`);
    }
    assert.match(reportDoctest.code, /test_report_print_stdout report/, `${reportName} must print the report`);
    assert.match(reportDoctest.code, /test_report_exit_code shown/, `${reportName} must derive exit code from the shown report`);
}

for (const snippet of [
    "この constructor は `next` の範囲検査を行いません",
    "`CharUtf8Step` は `Copy`",
    "`0x80 | (bits & 0x3F)`",
    "戻り値は常に `0x80..0xBF`",
    "文字長や index の検査を行いません",
]) {
    assert.match(
        charSource,
        new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')),
        `core/char must preserve UTF-8 helper contract snippet: ${snippet}`,
    );
}

console.log('core char doc report contract passed');
