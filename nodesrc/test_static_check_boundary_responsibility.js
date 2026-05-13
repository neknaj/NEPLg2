#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const CORE_SRC = path.join(ROOT, 'nepl-core', 'src');
const TYPECHECK_ROOT = path.join(CORE_SRC, 'typecheck.rs');
const TYPECHECK_DIR = path.join(CORE_SRC, 'typecheck');
const RESOURCE_ROOT = path.join(CORE_SRC, 'resource', 'mod.rs');
const COMPILER = path.join(CORE_SRC, 'compiler.rs');
const EFFECTS = path.join(CORE_SRC, 'effects.rs');
const PASSES_MOD = path.join(CORE_SRC, 'passes', 'mod.rs');
const DROP_INSERTION = path.join(CORE_SRC, 'passes', 'drop_insertion.rs');
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

function assertMatches(text, pattern, label) {
    assert(pattern.test(text), `${label} must match ${pattern}`);
}

function assertLineLimit(filePath, label, limit) {
    const lines = lineCount(assertFile(filePath, label));
    assert(lines <= limit, `${label} has ${lines} lines; responsibility split limit is ${limit}`);
}

function toPosixPath(filePath) {
    return path.relative(ROOT, filePath).split(path.sep).join('/');
}

function walkRustFiles(dir) {
    const files = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const child = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...walkRustFiles(child));
        } else if (entry.isFile() && entry.name.endsWith('.rs')) {
            files.push(child);
        }
    }
    return files;
}

const typecheckRoot = assertFile(TYPECHECK_ROOT, 'typecheck.rs');
const resourceRoot = assertFile(RESOURCE_ROOT, 'resource/mod.rs');
const compiler = assertFile(COMPILER, 'compiler.rs');
const effects = assertFile(EFFECTS, 'effects.rs');
const passesMod = assertFile(PASSES_MOD, 'passes/mod.rs');
const dropInsertion = assertFile(DROP_INSERTION, 'passes/drop_insertion.rs');
const typecheckMatchCheck = assertFile(
    path.join(TYPECHECK_DIR, 'match_check.rs'),
    'typecheck/match_check.rs',
);
const typecheckModel = assertFile(
    path.join(TYPECHECK_DIR, 'model.rs'),
    'typecheck/model.rs',
);
const typecheckDriver = assertFile(
    path.join(TYPECHECK_DIR, 'driver.rs'),
    'typecheck/driver.rs',
);
const typecheckConstructorApply = assertFile(
    path.join(TYPECHECK_DIR, 'constructor_apply.rs'),
    'typecheck/constructor_apply.rs',
);
const typecheckSyntaxHelpers = assertFile(
    path.join(TYPECHECK_DIR, 'syntax_helpers.rs'),
    'typecheck/syntax_helpers.rs',
);

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
assertContains(typecheckMatchCheck, 'variant_member_tail', 'typecheck/match_check.rs');
assertNotContains(typecheckMatchCheck, 'find("::")', 'typecheck/match_check.rs');
assertContains(typecheckSyntaxHelpers, 'fn split_qualified_name', 'typecheck/syntax_helpers.rs');
assertContains(typecheckSyntaxHelpers, 'fn variant_member_tail', 'typecheck/syntax_helpers.rs');
assertNotContains(typecheckSyntaxHelpers, 'parse_variant_name', 'typecheck/syntax_helpers.rs');
assertContains(typecheckModel, 'pub(super) enum StructConstructorPolicy', 'typecheck/model.rs');
assertContains(typecheckModel, 'RawMemoryBoundaryOnly(RestrictedStructConstructor)', 'typecheck/model.rs');
assertContains(typecheckModel, 'pub(super) enum RestrictedStructConstructor', 'typecheck/model.rs');
assertContains(typecheckModel, 'OwnerToken', 'typecheck/model.rs');
assertContains(typecheckModel, 'RawPointer', 'typecheck/model.rs');
assertContains(
    typecheckModel,
    'pub(super) constructor_policy: StructConstructorPolicy',
    'typecheck/model.rs',
);
assertContains(typecheckDriver, 'fn struct_constructor_policy', 'typecheck/driver.rs');
assertContains(
    typecheckDriver,
    'raw_memory_boundary_allowed(span.file_id)',
    'typecheck/driver.rs',
);
assertMatches(
    typecheckDriver,
    /"MemPtr"\s+if\s+raw_memory_boundary\s*=>\s*\{\s*StructConstructorPolicy::RawMemoryBoundaryOnly\(RestrictedStructConstructor::RawPointer\)\s*\}/,
    'typecheck/driver.rs MemPtr constructor policy',
);
assertMatches(
    typecheckDriver,
    /"RegionToken"\s+if\s+raw_memory_boundary\s*=>\s*\{\s*StructConstructorPolicy::RawMemoryBoundaryOnly\(RestrictedStructConstructor::OwnerToken\)\s*\}/,
    'typecheck/driver.rs RegionToken constructor policy',
);
assertContains(
    typecheckDriver,
    '_ => StructConstructorPolicy::Public',
    'typecheck/driver.rs',
);
assertContains(
    typecheckConstructorApply,
    'match info.constructor_policy',
    'typecheck/constructor_apply.rs',
);
assertMatches(
    typecheckConstructorApply,
    /StructConstructorPolicy::RawMemoryBoundaryOnly\(restricted\)\s*=>\s*\{\s*if\s+!self\.raw_memory_boundary_allowed\(span\)/,
    'typecheck/constructor_apply.rs constructor capability gate',
);
assertMatches(
    typecheckConstructorApply,
    /RestrictedStructConstructor::OwnerToken\s*=>\s*\(\s*TypeDiagnosticCode::OwnerTokenConstructorRestricted/,
    'typecheck/constructor_apply.rs owner token diagnostic branch',
);
assertMatches(
    typecheckConstructorApply,
    /RestrictedStructConstructor::RawPointer\s*=>\s*\(\s*TypeDiagnosticCode::RawPointerConstructorRestricted/,
    'typecheck/constructor_apply.rs raw pointer diagnostic branch',
);
assertNotContains(
    typecheckConstructorApply,
    'RestrictedStructConstructor::_',
    'typecheck/constructor_apply.rs',
);
assertContains(effects, 'pub enum RawBodyMemoryOp', 'effects.rs');
assertContains(effects, 'pub enum WasmRawBodyMemoryOp', 'effects.rs');
assertContains(effects, 'pub enum LlvmRawBodyMemoryOp', 'effects.rs');
assertContains(
    effects,
    'pub fn raw_body_memory_operations(body: &HirBody) -> Vec<RawBodyMemoryOp>',
    'effects.rs',
);
assertNotContains(
    effects,
    'pub fn raw_body_memory_operations(body: &HirBody) -> Vec<String>',
    'effects.rs',
);
assertNotContains(effects, 'fn wasm_memory_operation(line: &str) -> Option<String>', 'effects.rs');
assertNotContains(effects, 'fn llvm_memory_operation(line: &str) -> Option<String>', 'effects.rs');

assertMatches(
    compiler,
    /fn run_resource_cell_gate\(\s*report: &crate::resource::ResourceCheckReport,\s*diagnostics: &mut Vec<Diagnostic>,\s*\)/,
    'compiler.rs Resource cell gate must not take SourceMap',
);
assertMatches(
    compiler,
    /fn run_resource_owner_obligation_gate\(\s*report: &crate::resource::ResourceOwnerCheckReport,\s*diagnostics: &mut Vec<Diagnostic>,\s*\)/,
    'compiler.rs Resource owner gate must not take SourceMap',
);
const cellGateBody = compiler.match(/fn run_resource_cell_gate\([\s\S]*?\n}\n\nfn resource_cell_diagnostic_to_error/);
assert(cellGateBody, 'compiler.rs must expose Resource cell gate body');
assertNotContains(
    cellGateBody[0],
    'raw_memory_boundary_allowed',
    'compiler.rs Resource cell gate',
);
const ownerGateBody = compiler.match(/fn run_resource_owner_obligation_gate\([\s\S]*?\n}\n\nfn resource_owner_diagnostic_to_error/);
assert(ownerGateBody, 'compiler.rs must expose Resource owner gate body');
assertNotContains(
    ownerGateBody[0],
    'raw_memory_boundary_allowed',
    'compiler.rs Resource owner gate',
);
assertMatches(
    compiler,
    /ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc\s*\{\s*\.\.\s*\}\s*=>\s*false,/,
    'compiler.rs raw identity escape must not be raw-boundary-suppressed',
);
assertNotContains(
    compiler,
    'UnsafeMemoryInPureFunction {\n            ..\n        }\n        | crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc',
    'compiler.rs raw identity escape must not share unsafe-memory raw-boundary suppression',
);

for (const filePath of walkRustFiles(CORE_SRC)) {
    const rel = toPosixPath(filePath);
    if (rel === 'nepl-core/src/qualified_name.rs') {
        continue;
    }
    const text = read(filePath);
    assertNotContains(text, 'rfind("::")', rel);
    assertNotContains(text, 'rsplit("::")', rel);
    assertNotContains(text, 'splitn(2, "::")', rel);
}

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
assertContains(passesMod, 'pub use drop_insertion::insert_resource_drops;', 'passes/mod.rs');

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
assertContains(compiler, 'passes::insert_resource_drops', 'compiler.rs');
assertNotContains(compiler, 'passes::insert_drops', 'compiler.rs');
assertNotContains(compiler, 'move_check', 'compiler.rs');
assertContains(dropInsertion, 'ResourceDropElaborationPlan', 'passes/drop_insertion.rs');
assertContains(dropInsertion, 'ResourceAutoDropKind::ScopeLocal', 'passes/drop_insertion.rs');
assertContains(
    dropInsertion,
    'ResourceAutoDropKind::AssignmentOverwrite',
    'passes/drop_insertion.rs',
);
assertNotContains(dropInsertion, 'enum VarState', 'passes/drop_insertion.rs');
assertNotContains(dropInsertion, 'var_stacks', 'passes/drop_insertion.rs');

console.log('static check responsibility boundaries ok');
