#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');

const contracts = [
    {
        rel: ['stdlib', 'alloc', 'string', 'find.nepl'],
        index: 0,
        name: 'string_find_doc',
        count: 6,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'integer.nepl'],
        index: 0,
        name: 'string_integer_facade_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'float.nepl'],
        index: 0,
        name: 'string_float_facade_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'search', 'byte_find.nepl'],
        index: 0,
        name: 'str_find_doc',
        count: 4,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'search', 'compare.nepl'],
        name: 'str_starts_with_at_doc',
        count: 6,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'integer', 'parse.nepl'],
        index: 0,
        name: 'string_integer_parse_doc',
        count: 4,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'float', 'parse.nepl'],
        index: 0,
        name: 'string_float_parse_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'integer', 'common', 'bool.nepl'],
        index: 0,
        name: 'string_from_bool_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'integer', 'common', 'bool.nepl'],
        index: 1,
        name: 'string_to_bool_doc',
        count: 3,
    },
];

for (const { rel, index, name, count } of contracts) {
    const file = path.join(repoRoot, ...rel);
    const parsed = parseFile(file);
    const matchingDoctests = parsed.doctests.filter((doctest) =>
        new RegExp(`test_report_new "${name}"`).test(doctest.code),
    );
    assert.equal(
        matchingDoctests.length,
        1,
        `${rel.join('/')} must keep exactly one doc-comment doctest for ${name}`,
    );

    const source = fs.readFileSync(file, 'utf8');
    const doctest = matchingDoctests[0];
    assert.equal(doctest.ret, null, `${name} must not use ret as test-success metadata`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.match(
        doctest.stdout,
        new RegExp(`^test_report name="${name}" count=${count} failed=0\\n`),
        `${name} must pin canonical stdout report`,
    );
    assert.match(doctest.code, new RegExp(`test_report_new "${name}"`), `${name} must construct a named TestReport`);
    assert.match(doctest.code, /test_report_print_stdout report/, `${name} must print the report`);
    assert.match(doctest.code, /test_report_exit_code shown/, `${name} must derive exit code from the shown report`);
    assert.doesNotMatch(source, /\bchecks_exit_code\b/, `${rel.join('/')} must not hide report details behind checks_exit_code`);
    assert.doesNotMatch(source, /\bresult_exit_code\b/, `${rel.join('/')} must not hide report details behind result_exit_code`);
}

console.log('alloc string doc report contract passed');
