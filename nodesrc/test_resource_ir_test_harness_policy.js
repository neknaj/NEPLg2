#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const testPath = path.join(repoRoot, 'nepl-core', 'tests', 'resource_ir.rs');
const code = fs.readFileSync(testPath, 'utf8').replace(/\r\n/g, '\n');

function testBody(name) {
    const marker = `fn ${name}()`;
    const start = code.indexOf(marker);
    assert.notEqual(start, -1, `missing test ${name}`);
    const next = code.indexOf('\n#[test]', start + marker.length);
    return next === -1 ? code.slice(start) : code.slice(start, next);
}

for (const name of [
    'resource_ir_check_reports_non_copy_use_after_move',
    'resource_ir_check_reports_read_after_drop',
]) {
    const body = testBody(name);
    assert.match(
        body,
        /let\s+resource\s*=\s*lower_hir_module\(&module,\s*&types\);/,
        `${name} must lower with the TypeCtx that owns the custom non-Copy TypeId`,
    );
    assert.doesNotMatch(
        body,
        /lower_hir_module_skeleton\(&module\)/,
        `${name} must not use skeleton lowering with a fresh TypeCtx`,
    );
}

console.log('resource_ir test harness policy passed');
