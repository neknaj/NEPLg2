#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const CORE_SRC = path.join(ROOT, 'nepl-core', 'src');
const TYPECHECK_ROOT = path.join(CORE_SRC, 'typecheck.rs');
const TYPECHECK_DIR = path.join(CORE_SRC, 'typecheck');
const RESOURCE_ROOT = path.join(CORE_SRC, 'resource', 'mod.rs');
const COMPILER = path.join(CORE_SRC, 'compiler.rs');
const PASSES_MOD = path.join(CORE_SRC, 'passes', 'mod.rs');
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

function assertMissing(filePath, label) {
    assert(!fs.existsSync(filePath), `${label} must not be reintroduced`);
}

function assertContains(text, needle, label) {
    assert(text.includes(needle), `${label} must contain ${needle}`);
}

function assertNotContains(text, needle, label) {
    assert(!text.includes(needle), `${label} must not contain ${needle}`);
}

function assertLineLimit(filePath, label, limit) {
    const lines = lineCount(assertFile(filePath, label));
    assert(lines <= limit, `${label} has ${lines} lines; responsibility split limit is ${limit}`);
}

const typecheckRoot = assertFile(TYPECHECK_ROOT, 'typecheck.rs');
const resourceRoot = assertFile(RESOURCE_ROOT, 'resource/mod.rs');
const compiler = assertFile(COMPILER, 'compiler.rs');
const passesMod = assertFile(PASSES_MOD, 'passes/mod.rs');

assertLineLimit(TYPECHECK_ROOT, 'typecheck.rs', 90);

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

assertMissing(MOVE_CHECK_ROOT, 'legacy passes/move_check.rs');
assertMissing(MOVE_CHECK_DIR, 'legacy passes/move_check directory');
assertNotContains(passesMod, 'move_check', 'passes/mod.rs');
assertContains(passesMod, 'pub mod drop_insertion;', 'passes/mod.rs');
assertContains(passesMod, 'pub use drop_insertion::insert_drops;', 'passes/mod.rs');

for (const moduleName of [
    'borrow_check',
    'drop_elaboration',
    'drop_elaboration_hir_bridge',
    'drop_model',
    'drop_plan',
    'drop_point_resolve',
    'drop_requirement',
    'effect',
    'initialized',
    'lower',
    'owner_check',
    'shadow',
]) {
    assertContains(resourceRoot, `mod ${moduleName};`, 'resource/mod.rs');
}

for (const exportName of [
    'check_resource_initialized_moves',
    'compute_resource_drop_elaboration_plan',
    'validate_resource_drop_elaboration_hir_bridge',
    'check_resource_borrow_lifetimes',
    'check_resource_effect_boundaries',
    'check_resource_owner_obligations',
    'check_hir_resource_safety_shadow',
]) {
    assertContains(resourceRoot, exportName, 'resource/mod.rs');
}

assertContains(compiler, 'fn run_resource_static_check(', 'compiler.rs');
assertContains(compiler, 'check_resource_initialized_moves', 'compiler.rs');
assertContains(compiler, 'compute_resource_drop_elaboration_plan', 'compiler.rs');
assertContains(compiler, 'check_resource_borrow_lifetimes', 'compiler.rs');
assertContains(compiler, 'check_resource_effect_boundaries', 'compiler.rs');
assertContains(compiler, 'check_resource_owner_obligations', 'compiler.rs');
assertContains(compiler, 'run_resource_drop_elaboration_hir_bridge_gate', 'compiler.rs');
assertContains(compiler, 'passes::insert_drops', 'compiler.rs');
assertNotContains(compiler, 'move_check', 'compiler.rs');

console.log('static check responsibility boundaries ok');
