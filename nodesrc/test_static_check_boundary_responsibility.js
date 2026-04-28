#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const CORE_SRC = path.join(ROOT, 'nepl-core', 'src');
const TYPECHECK_ROOT = path.join(CORE_SRC, 'typecheck.rs');
const TYPECHECK_DIR = path.join(CORE_SRC, 'typecheck');
const MOVE_CHECK_ROOT = path.join(CORE_SRC, 'passes', 'move_check.rs');
const MOVE_CHECK_DIR = path.join(CORE_SRC, 'passes', 'move_check');

function read(filePath) {
    return fs.readFileSync(filePath, 'utf8').replace(/\r\n/g, '\n');
}

function lineCount(text) {
    return text.split('\n').length;
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function assertFile(filePath, label) {
    assert(fs.existsSync(filePath), `missing ${label}`);
    return read(filePath);
}

function assertContains(text, needle, label) {
    assert(text.includes(needle), `${label} must contain ${needle}`);
}

function assertLineLimit(filePath, label, limit) {
    const lines = lineCount(assertFile(filePath, label));
    assert(lines <= limit, `${label} has ${lines} lines; responsibility split limit is ${limit}`);
}

const typecheckRoot = assertFile(TYPECHECK_ROOT, 'typecheck.rs');
const moveCheckRoot = assertFile(MOVE_CHECK_ROOT, 'passes/move_check.rs');

assertLineLimit(TYPECHECK_ROOT, 'typecheck.rs', 90);
assertLineLimit(MOVE_CHECK_ROOT, 'passes/move_check.rs', 240);

for (const moduleName of [
    'ascription',
    'assignment_apply',
    'binding_rules',
    'block_check',
    'call_binding_lookup',
    'call_reduction',
    'call_resolution',
    'constructor_apply',
    'context',
    'control_apply',
    'driver',
    'driver_entry',
    'effect_check',
    'env',
    'field_access',
    'field_apply',
    'function_apply',
    'function_check',
    'hir_finalize',
    'indirect_apply',
    'match_check',
    'model',
    'name_lookup',
    'overload_selection',
    'prefix_check',
    'selected_call_apply',
    'signature',
    'syntax_helpers',
    'trait_bound_apply',
    'trait_call_apply',
    'trait_check',
    'traits',
    'type_expr',
]) {
    assertFile(path.join(TYPECHECK_DIR, `${moduleName}.rs`), `typecheck/${moduleName}.rs`);
    assertContains(typecheckRoot, `mod ${moduleName};`, 'typecheck.rs');
}

assertContains(typecheckRoot, 'pub use driver::{typecheck, TypeCheckResult};', 'typecheck.rs');

for (const [moduleName, limit] of [
    ['driver.rs', 1700],
    ['prefix_check.rs', 2200],
    ['call_resolution.rs', 760],
    ['block_check.rs', 700],
    ['overload_selection.rs', 460],
    ['selected_call_apply.rs', 420],
]) {
    assertLineLimit(path.join(TYPECHECK_DIR, moduleName), `typecheck/${moduleName}`, limit);
}

for (const moduleName of [
    'alias',
    'branch_merge',
    'context_state',
    'provenance',
    'raw_memory',
    'raw_place',
    'raw_state',
    'state',
    'summary',
    'summary_build',
    'visitor',
]) {
    assertFile(path.join(MOVE_CHECK_DIR, `${moduleName}.rs`), `passes/move_check/${moduleName}.rs`);
    assertContains(moveCheckRoot, `mod ${moduleName};`, 'passes/move_check.rs');
}

assertContains(moveCheckRoot, 'pub fn run', 'passes/move_check.rs');
assertContains(moveCheckRoot, 'struct MoveCheckContext', 'passes/move_check.rs');

for (const [moduleName, limit] of [
    ['alias.rs', 1850],
    ['visitor.rs', 1450],
    ['context_state.rs', 1250],
    ['summary_build.rs', 820],
    ['branch_merge.rs', 760],
    ['provenance.rs', 620],
    ['raw_state.rs', 360],
]) {
    assertLineLimit(path.join(MOVE_CHECK_DIR, moduleName), `passes/move_check/${moduleName}`, limit);
}

console.log('static check responsibility boundaries ok');
