#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');

const contracts = [
    {
        rel: ['stdlib', 'alloc', 'collections', 'btreemap', 'search.nepl'],
        name: 'btreemap_key_eq_doc',
        helper: 'btreemap_key_eq',
    },
    {
        rel: ['stdlib', 'alloc', 'collections', 'btreeset', 'search.nepl'],
        name: 'btreeset_key_eq_doc',
        helper: 'btreeset_key_eq',
    },
];

for (const { rel, name, helper } of contracts) {
    const file = path.join(repoRoot, ...rel);
    const source = fs.readFileSync(file, 'utf8');
    const parsed = parseFile(file);
    assert.equal(parsed.doctests.length, 1, `${rel.join('/')} must keep exactly one key equality doctest`);

    const doctest = parsed.doctests[0];
    assert.equal(doctest.ret, null, `${name} must not use ret-only success reporting`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.match(
        doctest.stdout,
        new RegExp(`^test_report name="${name}" count=2 failed=0\\n`),
        `${name} must pin canonical stdout report`,
    );
    assert.match(doctest.code, new RegExp(`test_report_new "${name}"`), `${name} must construct a named TestReport`);
    assert.match(doctest.code, new RegExp(`${helper}<i32> 7 7`), `${name} must exercise equal keys`);
    assert.match(doctest.code, new RegExp(`${helper}<i32> 7 9`), `${name} must exercise unequal keys`);
    assert.match(doctest.code, /test_report_print_stdout report/, `${name} must print the report`);
    assert.match(doctest.code, /test_report_exit_code shown/, `${name} must derive exit code from the shown report`);
    assert.doesNotMatch(source, /\bchecks_exit_code\b/, `${rel.join('/')} must not hide report details behind checks_exit_code`);
    assert.doesNotMatch(source, /\bchecks_print_report\b/, `${rel.join('/')} must use canonical TestReport output`);
    assert.doesNotMatch(source, /\bchecks_new\b/, `${rel.join('/')} must not reintroduce legacy Checks construction`);
}

console.log('stdlib btree search doc report contract passed');
