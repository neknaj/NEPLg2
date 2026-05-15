#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');

const contracts = [
    {
        rel: ['stdlib', 'core', 'traits', 'stringify.nepl'],
        name: 'stringify_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'core', 'traits', 'serialize.nepl'],
        name: 'serialize_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'core', 'traits', 'hash.nepl'],
        name: 'hash_trait_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'core', 'traits', 'debug.nepl'],
        name: 'debug_string_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'core', 'traits', 'deserialize.nepl'],
        name: 'deserialize_doc',
        count: 1,
    },
];

for (const { rel, name, count } of contracts) {
    const parsed = parseFile(path.join(repoRoot, ...rel));
    assert.ok(parsed.doctests.length >= 1, `${rel.join('/')} must keep its public doc-comment doctest`);

    const doctest = parsed.doctests[0];
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
    assert.doesNotMatch(doctest.code, /\bchecks_exit_code\b/, `${name} must not hide report details behind checks_exit_code`);
    assert.doesNotMatch(doctest.code, /\bresult_exit_code\b/, `${name} must not hide report details behind result_exit_code`);
}

console.log('core traits doc report contract passed');
