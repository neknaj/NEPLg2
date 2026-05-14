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
const LOADER = path.join(CORE_SRC, 'loader.rs');
const SOURCE_MAP = path.join(CORE_SRC, 'source_map.rs');
const SOURCE_CAPABILITY = path.join(CORE_SRC, 'source_capability.rs');
const SOURCE_CAPABILITY_MEMORY_TYPE_DEFINITION = path.join(
    CORE_SRC,
    'source_capability',
    'memory_type_definition.rs',
);
const SOURCE_CAPABILITY_RAW_MEMORY = path.join(CORE_SRC, 'source_capability', 'raw_memory.rs');
const SOURCE_CAPABILITY_OWNER_AGGREGATE = path.join(
    CORE_SRC,
    'source_capability',
    'owner_aggregate.rs',
);
const SOURCE_CAPABILITY_SCOPE = path.join(
    CORE_SRC,
    'source_capability',
    'scope.rs',
);
const PASSES_MOD = path.join(CORE_SRC, 'passes', 'mod.rs');
const DROP_INSERTION = path.join(CORE_SRC, 'passes', 'drop_insertion.rs');
const MOVE_CHECK_ROOT = path.join(CORE_SRC, 'passes', 'move_check.rs');
const MOVE_CHECK_DIR = path.join(CORE_SRC, 'passes', 'move_check');
const RESOURCE_IR_TESTS = path.join(ROOT, 'nepl-core', 'tests', 'resource_ir.rs');
const TEST_HARNESS = path.join(ROOT, 'nepl-core', 'tests', 'harness.rs');
const NEPLG2_TESTS = path.join(ROOT, 'nepl-core', 'tests', 'neplg2.rs');

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
const loader = assertFile(LOADER, 'loader.rs');
const sourceMap = assertFile(SOURCE_MAP, 'source_map.rs');
const sourceCapability = assertFile(SOURCE_CAPABILITY, 'source_capability.rs');
const sourceCapabilityMemoryTypeDefinition = assertFile(
    SOURCE_CAPABILITY_MEMORY_TYPE_DEFINITION,
    'source_capability/memory_type_definition.rs',
);
const sourceCapabilityRawMemory = assertFile(
    SOURCE_CAPABILITY_RAW_MEMORY,
    'source_capability/raw_memory.rs',
);
const sourceCapabilityOwnerAggregate = assertFile(
    SOURCE_CAPABILITY_OWNER_AGGREGATE,
    'source_capability/owner_aggregate.rs',
);
const sourceCapabilityScope = assertFile(
    SOURCE_CAPABILITY_SCOPE,
    'source_capability/scope.rs',
);
const passesMod = assertFile(PASSES_MOD, 'passes/mod.rs');
const dropInsertion = assertFile(DROP_INSERTION, 'passes/drop_insertion.rs');
const resourceIrTests = assertFile(RESOURCE_IR_TESTS, 'nepl-core/tests/resource_ir.rs');
const testHarness = assertFile(TEST_HARNESS, 'nepl-core/tests/harness.rs');
const neplg2Tests = assertFile(NEPLG2_TESTS, 'nepl-core/tests/neplg2.rs');
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
const typecheckFieldAccess = assertFile(
    path.join(TYPECHECK_DIR, 'field_access.rs'),
    'typecheck/field_access.rs',
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
    'compiler_memory_type_from_constructor_name(name)',
    'typecheck/driver.rs',
);
assertContains(
    typecheckDriver,
    'compiler_memory_type_definition_allowed(span.file_id, memory_type)',
    'typecheck/driver.rs',
);
assertContains(typecheckDriver, 'CompilerMemoryType::RawPointer', 'typecheck/driver.rs');
assertContains(typecheckDriver, 'CompilerMemoryType::OwnerToken', 'typecheck/driver.rs');
assertContains(
    typecheckDriver,
    'return StructConstructorPolicy::Public',
    'typecheck/driver.rs',
);
const typecheckCopyCapability = assertFile(
    path.join(TYPECHECK_DIR, 'copy_capability.rs'),
    'typecheck/copy_capability.rs',
);
assertContains(
    typecheckCopyCapability,
    'fn target_contains_owner_backed_aggregate',
    'typecheck/copy_capability.rs',
);
assertContains(
    typecheckCopyCapability,
    'loop {',
    'typecheck/copy_capability.rs owner-backed aggregate fixed point',
);
assertContains(
    typecheckCopyCapability,
    'StructConstructorPolicy::OwnerBackedAggregateBoundaryOnly',
    'typecheck/copy_capability.rs',
);
assertContains(
    typecheckConstructorApply,
    'match constructor_policy',
    'typecheck/constructor_apply.rs',
);
assertContains(
    typecheckConstructorApply,
    'target_contains_owner_backed_aggregate',
    'typecheck/constructor_apply.rs applied owner-backed aggregate constructor gate',
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
assertContains(
    typecheckFieldAccess,
    'restricted_struct_field_access_error',
    'typecheck/field_access.rs',
);
assertContains(
    typecheckFieldAccess,
    'compiler_memory_type_definition_allowed',
    'typecheck/field_access.rs',
);
assertMatches(
    typecheckFieldAccess,
    /StructConstructorPolicy::RawMemoryBoundaryOnly\(restricted\)\s*=>\s*Some\(restricted\)/,
    'typecheck/field_access.rs field access constructor policy gate',
);
assertContains(
    typecheckFieldAccess,
    'target_contains_owner_backed_aggregate',
    'typecheck/field_access.rs owner-backed aggregate field projection gate',
);
assertContains(
    typecheckFieldAccess,
    'OwnerAggregateFieldAccessRestricted',
    'typecheck/field_access.rs owner-backed aggregate field diagnostic',
);
assertContains(
    typecheckCopyCapability,
    'pub(super) fn target_contains_owner_backed_aggregate',
    'typecheck/copy_capability.rs exposes structural owner-backed aggregate predicate',
);
assertContains(
    typecheckCopyCapability,
    'target_apply_contains_owner_backed_aggregate',
    'typecheck/copy_capability.rs checks applied generic owner-backed aggregate fields',
);
assertMatches(
    typecheckFieldAccess,
    /RestrictedStructConstructor::OwnerToken[\s\S]*CompilerMemoryType::OwnerToken/,
    'typecheck/field_access.rs owner token definition capability branch',
);
assertMatches(
    typecheckFieldAccess,
    /RestrictedStructConstructor::RawPointer[\s\S]*CompilerMemoryType::RawPointer/,
    'typecheck/field_access.rs raw pointer definition capability branch',
);
assertMatches(
    typecheckFieldAccess,
    /RestrictedStructConstructor::OwnerToken\s*=>\s*\(\s*TypeDiagnosticCode::OwnerTokenFieldAccessRestricted/,
    'typecheck/field_access.rs owner token field diagnostic branch',
);
assertMatches(
    typecheckFieldAccess,
    /RestrictedStructConstructor::RawPointer\s*=>\s*\(\s*TypeDiagnosticCode::RawPointerFieldAccessRestricted/,
    'typecheck/field_access.rs raw pointer field diagnostic branch',
);
assertNotContains(
    typecheckFieldAccess,
    'RestrictedStructConstructor::_',
    'typecheck/field_access.rs',
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
assertLineLimit(SOURCE_CAPABILITY, 'source_capability.rs', 40);
assertLineLimit(
    SOURCE_CAPABILITY_MEMORY_TYPE_DEFINITION,
    'source_capability/memory_type_definition.rs',
    100,
);
assertLineLimit(SOURCE_CAPABILITY_RAW_MEMORY, 'source_capability/raw_memory.rs', 220);
assertLineLimit(
    SOURCE_CAPABILITY_OWNER_AGGREGATE,
    'source_capability/owner_aggregate.rs',
    160,
);
assertLineLimit(SOURCE_CAPABILITY_SCOPE, 'source_capability/scope.rs', 100);
assertContains(sourceMap, 'pub enum SourceCapability', 'source_map.rs');
assertContains(sourceMap, 'CompilerMemoryTypeDefinition(CompilerMemoryType)', 'source_map.rs');
assertContains(sourceMap, 'OwnerAggregateConstructorBoundary', 'source_map.rs');
assertContains(sourceMap, 'OwnerAggregateFieldBoundary', 'source_map.rs');
assertContains(sourceMap, 'pub enum CompilerMemoryType', 'source_map.rs');
assertContains(sourceMap, 'RawPointer', 'source_map.rs');
assertContains(sourceMap, 'OwnerToken', 'source_map.rs');
assertContains(
    sourceMap,
    'compiler_memory_type_definition_allowed',
    'source_map.rs',
);
assertContains(
    sourceCapabilityRawMemory,
    'enum RawMemoryBoundaryEvidence',
    'source_capability/raw_memory.rs',
);
assertContains(
    sourceCapabilityRawMemory,
    'pub(crate) fn module_has_raw_memory_boundary_evidence',
    'source_capability/raw_memory.rs',
);
assertContains(
    sourceCapability,
    'mod memory_type_definition;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'compiler_memory_type_from_constructor_name',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'module_compiler_memory_type_definitions',
    'source_capability.rs',
);
assertContains(
    sourceCapabilityMemoryTypeDefinition,
    'pub(crate) fn compiler_memory_type_from_constructor_name',
    'source_capability/memory_type_definition.rs',
);
assertContains(
    sourceCapabilityMemoryTypeDefinition,
    'match name',
    'source_capability/memory_type_definition.rs',
);
assertContains(
    sourceCapabilityMemoryTypeDefinition,
    'pub(crate) fn module_compiler_memory_type_definitions',
    'source_capability/memory_type_definition.rs',
);
assertContains(
    sourceCapabilityMemoryTypeDefinition,
    'is_mem_ptr_definition',
    'source_capability/memory_type_definition.rs',
);
assertContains(
    sourceCapabilityMemoryTypeDefinition,
    'is_region_token_definition',
    'source_capability/memory_type_definition.rs',
);
assertContains(
    sourceCapability,
    'mod owner_aggregate;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'mod scope;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'module_has_owner_aggregate_constructor_evidence',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'module_has_owner_aggregate_field_evidence',
    'source_capability.rs',
);
assertContains(
    sourceCapabilityOwnerAggregate,
    'enum OwnerAggregateCapabilityEvidence',
    'source_capability/owner_aggregate.rs',
);
assertContains(
    sourceCapabilityOwnerAggregate,
    'pub(crate) fn module_has_owner_aggregate_constructor_evidence',
    'source_capability/owner_aggregate.rs',
);
assertContains(
    sourceCapabilityOwnerAggregate,
    'pub(crate) fn module_has_owner_aggregate_field_evidence',
    'source_capability/owner_aggregate.rs',
);
assertContains(
    sourceCapabilityOwnerAggregate,
    'crate::qualified_name::member_tail(symbol) != symbol',
    'owner aggregate constructor evidence must ignore qualified enum variants',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_ignores_qualified_enum_variant_constructors()',
    'loader.rs owner aggregate enum variant regression',
);
assertContains(
    sourceCapabilityScope,
    'struct SourceCapabilityScope',
    'source_capability/scope.rs',
);
assertContains(
    sourceCapabilityScope,
    'bind_stmt_locals',
    'source_capability/scope.rs',
);
assertContains(
    sourceCapabilityScope,
    'bind_match_pattern',
    'source_capability/scope.rs',
);
assertContains(sourceCapabilityRawMemory, 'raw_memory_op_from_name', 'source_capability/raw_memory.rs');
assertContains(sourceCapabilityRawMemory, 'PrefixItem::Intrinsic', 'source_capability/raw_memory.rs');
assertContains(sourceCapabilityRawMemory, 'enum RawAddressBoundaryHelper', 'source_capability/raw_memory.rs');
assertContains(sourceCapabilityRawMemory, 'enum RawOwnerBoundaryHelper', 'source_capability/raw_memory.rs');
assertContains(sourceCapabilityRawMemory, 'RestrictedConstructor', 'source_capability/raw_memory.rs');
assertNotContains(loader, 'RAW_MEMORY_BOUNDARY_STDLIB_PATHS', 'loader.rs');
assertNotContains(loader, 'configured_raw_memory_boundary_path', 'loader.rs');
assertContains(loader, 'configured_stdlib_source_path', 'loader.rs');
assertContains(loader, 'module_has_raw_memory_boundary_evidence', 'loader.rs');
assertContains(loader, 'module_has_owner_aggregate_constructor_evidence', 'loader.rs');
assertContains(loader, 'module_has_owner_aggregate_field_evidence', 'loader.rs');
assertContains(loader, 'module_compiler_memory_type_definitions', 'loader.rs');
assertContains(
    testHarness,
    'pub fn compile_src_with_options_and_entry_capabilities',
    'nepl-core/tests/harness.rs',
);
assertContains(
    testHarness,
    'pub fn run_main_wasi_i32_raw_memory_boundary',
    'nepl-core/tests/harness.rs',
);
assertMatches(
    testHarness,
    /pub fn run_main_wasi_i32\(src: &str\) -> i32 \{[\s\S]*?compile_src_with_options\([\s\S]*?run_wasi_wasm_i32\(&wasm\)\s*\}/,
    'ordinary WASI test harness must not grant raw-memory-boundary capability',
);
assertMatches(
    testHarness,
    /pub fn run_main_wasi_i32_raw_memory_boundary\(src: &str\) -> i32 \{[\s\S]*?SourceCapabilities::raw_memory_boundary\(\)[\s\S]*?run_wasi_wasm_i32\(&wasm\)\s*\}/,
    'raw-memory fixture harness must grant the capability explicitly',
);
for (const testName of [
    'generic_intrinsic_store_load_struct_preserves_fields',
    'generic_hashkey_eq_after_load_uses_concrete_impl',
    'generic_hashkey_value_survives_hash_before_store',
    'generic_store_after_generic_trait_probe_preserves_struct',
    'generic_store_uses_nested_address_call_without_stealing_value_arg',
]) {
    const testBody = neplg2Tests.match(new RegExp(`fn ${testName}\\(\\) \\{[\\s\\S]*?\\n\\}`));
    assert(testBody, `nepl-core/tests/neplg2.rs must define ${testName}`);
    assertContains(
        testBody[0],
        'run_main_wasi_i32_raw_memory_boundary',
        `${testName} must use the explicit raw-memory-boundary harness`,
    );
}

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
    /ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc\s*\{\s*\.\.\s*\}\s*=>\s*\{\s*let Some\(span\) = resource_effect_boundary_diagnostic_span\(diagnostic\) else \{\s*return false;\s*\};\s*source_map\s*\.map\(\|map\| map\.raw_memory_boundary_allowed\(span\.file_id\)\)\s*\.unwrap_or\(false\)\s*\}/,
    'compiler.rs raw identity escape must require an explicit raw-memory-boundary source capability',
);
assertNotContains(
    compiler,
    'UnsafeMemoryInPureFunction {\n            ..\n        }\n        | crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc',
    'compiler.rs raw identity escape must not share unsafe-memory raw-boundary suppression',
);
assertContains(
    compiler,
    'fn resource_effect_gate_allows_raw_identity_escape_inside_raw_boundary()',
    'compiler.rs raw-boundary raw identity unit regression',
);
assertContains(
    resourceIrTests,
    'resource_ir_effect_check_propagates_internal_alloc_return_summary',
    'resource_ir.rs caller-side internal allocation identity propagation regression',
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
