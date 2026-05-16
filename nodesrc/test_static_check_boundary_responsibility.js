#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const CORE_SRC = path.join(ROOT, 'nepl-core', 'src');
const TYPECHECK_ROOT = path.join(CORE_SRC, 'typecheck.rs');
const TYPECHECK_DIR = path.join(CORE_SRC, 'typecheck');
const RESOURCE_ROOT = path.join(CORE_SRC, 'resource', 'mod.rs');
const RESOURCE_LOWER_RAW_ADDRESS = path.join(CORE_SRC, 'resource', 'lower_raw_address.rs');
const RESOURCE_LOWER_RAW_ADDRESS_PLACE = path.join(
    CORE_SRC,
    'resource',
    'lower_raw_address_place.rs',
);
const RESOURCE_LOWER_RAW_ADDRESS_RETURN = path.join(
    CORE_SRC,
    'resource',
    'lower_raw_address_return.rs',
);
const RESOURCE_OWNER_FLOW = path.join(CORE_SRC, 'resource', 'owner_flow.rs');
const RESOURCE_OWNER_RAW_ADDRESS = path.join(CORE_SRC, 'resource', 'owner_raw_address.rs');
const RESOURCE_PLACE_UTILS = path.join(CORE_SRC, 'resource', 'place_utils.rs');
const COMPILER = path.join(CORE_SRC, 'compiler.rs');
const EFFECTS = path.join(CORE_SRC, 'effects.rs');
const LOADER = path.join(CORE_SRC, 'loader.rs');
const SOURCE_MAP = path.join(CORE_SRC, 'source_map.rs');
const RESOURCE_PRIMITIVES = path.join(CORE_SRC, 'resource_primitives.rs');
const SOURCE_CAPABILITY = path.join(CORE_SRC, 'source_capability.rs');
const SOURCE_CAPABILITY_MEMORY_TYPE_DEFINITION = path.join(
    CORE_SRC,
    'source_capability',
    'memory_type_definition.rs',
);
const SOURCE_CAPABILITY_CONSTRUCTOR_POSITION = path.join(
    CORE_SRC,
    'source_capability',
    'constructor_position.rs',
);
const SOURCE_CAPABILITY_PREFIX_CALL = path.join(
    CORE_SRC,
    'source_capability',
    'prefix_call.rs',
);
const SOURCE_CAPABILITY_WALK = path.join(
    CORE_SRC,
    'source_capability',
    'walk.rs',
);
const SOURCE_CAPABILITY_PROOF = path.join(
    CORE_SRC,
    'source_capability',
    'proof.rs',
);
const SOURCE_CAPABILITY_RAW_MEMORY = path.join(CORE_SRC, 'source_capability', 'raw_memory.rs');
const SOURCE_CAPABILITY_RAW_MEMORY_EVIDENCE = path.join(
    CORE_SRC,
    'source_capability',
    'raw_memory',
    'evidence.rs',
);
const SOURCE_CAPABILITY_OWNER_AGGREGATE = path.join(
    CORE_SRC,
    'source_capability',
    'owner_aggregate.rs',
);
const SOURCE_CAPABILITY_OWNER_AGGREGATE_CONTEXT = path.join(
    CORE_SRC,
    'source_capability',
    'owner_aggregate',
    'context.rs',
);
const SOURCE_CAPABILITY_OWNER_AGGREGATE_EVIDENCE = path.join(
    CORE_SRC,
    'source_capability',
    'owner_aggregate',
    'evidence.rs',
);
const SOURCE_CAPABILITY_OWNER_AGGREGATE_FIELD_IMPORTS = path.join(
    CORE_SRC,
    'source_capability',
    'owner_aggregate',
    'field_imports.rs',
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
const resourcePrimitives = assertFile(RESOURCE_PRIMITIVES, 'resource_primitives.rs');
const resourceLowerRawAddress = assertFile(
    RESOURCE_LOWER_RAW_ADDRESS,
    'resource/lower_raw_address.rs',
);
const resourceLowerRawAddressPlace = assertFile(
    RESOURCE_LOWER_RAW_ADDRESS_PLACE,
    'resource/lower_raw_address_place.rs',
);
const resourceLowerRawAddressReturn = assertFile(
    RESOURCE_LOWER_RAW_ADDRESS_RETURN,
    'resource/lower_raw_address_return.rs',
);
const resourceOwnerFlow = assertFile(RESOURCE_OWNER_FLOW, 'resource/owner_flow.rs');
const resourceOwnerRawAddress = assertFile(
    RESOURCE_OWNER_RAW_ADDRESS,
    'resource/owner_raw_address.rs',
);
const resourcePlaceUtils = assertFile(RESOURCE_PLACE_UTILS, 'resource/place_utils.rs');
const compiler = assertFile(COMPILER, 'compiler.rs');
const effects = assertFile(EFFECTS, 'effects.rs');
const loader = assertFile(LOADER, 'loader.rs');
const sourceMap = assertFile(SOURCE_MAP, 'source_map.rs');
const sourceCapability = assertFile(SOURCE_CAPABILITY, 'source_capability.rs');
const sourceCapabilityMemoryTypeDefinition = assertFile(
    SOURCE_CAPABILITY_MEMORY_TYPE_DEFINITION,
    'source_capability/memory_type_definition.rs',
);
const sourceCapabilityConstructorPosition = assertFile(
    SOURCE_CAPABILITY_CONSTRUCTOR_POSITION,
    'source_capability/constructor_position.rs',
);
const sourceCapabilityPrefixCall = assertFile(
    SOURCE_CAPABILITY_PREFIX_CALL,
    'source_capability/prefix_call.rs',
);
const sourceCapabilityWalk = assertFile(
    SOURCE_CAPABILITY_WALK,
    'source_capability/walk.rs',
);
const sourceCapabilityProof = assertFile(
    SOURCE_CAPABILITY_PROOF,
    'source_capability/proof.rs',
);
const sourceCapabilityRawMemory = assertFile(
    SOURCE_CAPABILITY_RAW_MEMORY,
    'source_capability/raw_memory.rs',
);
const sourceCapabilityRawMemoryEvidence = assertFile(
    SOURCE_CAPABILITY_RAW_MEMORY_EVIDENCE,
    'source_capability/raw_memory/evidence.rs',
);
const sourceCapabilityOwnerAggregate = assertFile(
    SOURCE_CAPABILITY_OWNER_AGGREGATE,
    'source_capability/owner_aggregate.rs',
);
const sourceCapabilityOwnerAggregateContext = assertFile(
    SOURCE_CAPABILITY_OWNER_AGGREGATE_CONTEXT,
    'source_capability/owner_aggregate/context.rs',
);
const sourceCapabilityOwnerAggregateEvidence = assertFile(
    SOURCE_CAPABILITY_OWNER_AGGREGATE_EVIDENCE,
    'source_capability/owner_aggregate/evidence.rs',
);
const sourceCapabilityOwnerAggregateFieldImports = assertFile(
    SOURCE_CAPABILITY_OWNER_AGGREGATE_FIELD_IMPORTS,
    'source_capability/owner_aggregate/field_imports.rs',
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
const typecheckCompilerMemoryType = assertFile(
    path.join(TYPECHECK_DIR, 'compiler_memory_type.rs'),
    'typecheck/compiler_memory_type.rs',
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
    'compiler_memory_type',
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
    typecheckCompilerMemoryType,
    'compiler_memory_type_from_constructor_name(def.name.name.as_str())',
    'typecheck/compiler_memory_type.rs must classify the current struct definition',
);
assertContains(
    typecheckCompilerMemoryType,
    'compiler_memory_type_definition_shape_holds',
    'typecheck/compiler_memory_type.rs must re-check typed struct shape',
);
assertContains(
    typecheckCompilerMemoryType,
    'source_map.compiler_memory_type_definition_allowed(def.name.span.file_id, memory_type)',
    'typecheck/compiler_memory_type.rs must still require SourceMap source proof',
);
assertContains(
    typecheckCompilerMemoryType,
    'fn type_id_is_i32',
    'typecheck/compiler_memory_type.rs must validate typed field shape',
);
assertContains(
    typecheckDriver,
    'let compiler_memory_type = compiler_memory_type_definition_allowed(',
    'typecheck/driver.rs must use compiler memory registration helper',
);
assertContains(
    typecheckDriver,
    's, &fs, &f_names, &tps, &ctx, source_map',
    'typecheck/driver.rs must pass the current struct definition and typed fields to compiler memory registration',
);
assertNotContains(
    typecheckDriver,
    'compiler_memory_type_definition_allowed(&s.name.name, s.name.span, source_map)',
    'typecheck/driver.rs must not register compiler memory type from name plus file capability only',
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
    typecheckCopyCapability,
    'type_is_owner_token(ctx, ty)',
    'typecheck/copy_capability.rs owner token root must use proven TypeCtx identity',
);
assertNotContains(
    typecheckCopyCapability,
    'RestrictedStructConstructor::OwnerToken',
    'typecheck/copy_capability.rs must not classify owner token roots through constructor policy metadata',
);
assertNotContains(
    typecheckCopyCapability,
    'StructConstructorPolicy::RawMemoryBoundaryOnly',
    'typecheck/copy_capability.rs must not classify compiler owner token roots through struct policy',
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
    /StructConstructorPolicy::RawMemoryBoundaryOnly\(restricted\)\s*=>\s*\{\s*if\s+!self\.raw_memory_structural_boundary_allowed\(span\)/,
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
assertContains(
    typecheckFieldAccess,
    'compiler_memory_type_of_type(self.ctx, base_ty)',
    'typecheck/field_access.rs field access must classify compiler memory types through proven TypeCtx identity',
);
assertNotContains(
    typecheckFieldAccess,
    'StructConstructorPolicy::RawMemoryBoundaryOnly(restricted) => Some(restricted)',
    'typecheck/field_access.rs must not classify compiler memory fields through struct constructor policy',
);
assertMatches(
    typecheckFieldAccess,
    /let resolved_ty = self\.ctx\.resolve\(base_ty\);\s*if let Some\(restricted\) = self\.restricted_struct_constructor_for_field_access\(resolved_ty\)[\s\S]*?return None;[\s\S]*?let access = match self\.ctx\.get\(resolved_ty\)/,
    'typecheck/field_access.rs must gate restricted compiler memory field access before field-name validation',
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
assertLineLimit(
    SOURCE_CAPABILITY_CONSTRUCTOR_POSITION,
    'source_capability/constructor_position.rs',
    80,
);
assertLineLimit(SOURCE_CAPABILITY_PREFIX_CALL, 'source_capability/prefix_call.rs', 80);
assertLineLimit(SOURCE_CAPABILITY_WALK, 'source_capability/walk.rs', 170);
assertLineLimit(SOURCE_CAPABILITY_PROOF, 'source_capability/proof.rs', 240);
assertLineLimit(RESOURCE_PRIMITIVES, 'resource_primitives.rs', 170);
assertLineLimit(SOURCE_CAPABILITY_RAW_MEMORY, 'source_capability/raw_memory.rs', 40);
assertLineLimit(
    SOURCE_CAPABILITY_RAW_MEMORY_EVIDENCE,
    'source_capability/raw_memory/evidence.rs',
    120,
);
assertLineLimit(
    SOURCE_CAPABILITY_OWNER_AGGREGATE,
    'source_capability/owner_aggregate.rs',
    60,
);
assertLineLimit(
    SOURCE_CAPABILITY_OWNER_AGGREGATE_CONTEXT,
    'source_capability/owner_aggregate/context.rs',
    110,
);
assertLineLimit(
    SOURCE_CAPABILITY_OWNER_AGGREGATE_EVIDENCE,
    'source_capability/owner_aggregate/evidence.rs',
    130,
);
assertLineLimit(
    SOURCE_CAPABILITY_OWNER_AGGREGATE_FIELD_IMPORTS,
    'source_capability/owner_aggregate/field_imports.rs',
    110,
);
assertLineLimit(SOURCE_CAPABILITY_SCOPE, 'source_capability/scope.rs', 100);
assertContains(sourceMap, 'pub enum SourceCapability', 'source_map.rs');
assertContains(sourceMap, 'RawMemoryStructuralBoundary', 'source_map.rs');
assertContains(sourceMap, 'RawMemoryOperationBoundary(RawMemoryOp)', 'source_map.rs');
assertContains(sourceMap, 'RawBodyMemoryOperationBoundary(RawBodyMemoryOp)', 'source_map.rs');
assertContains(sourceMap, 'CompilerMemoryTypeDefinition(CompilerMemoryType)', 'source_map.rs');
assertContains(sourceMap, 'OwnerAggregateConstructorBoundary(String)', 'source_map.rs');
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
    sourceCapabilityRawMemoryEvidence,
    'enum RawMemoryBoundaryEvidence',
    'source_capability/raw_memory/evidence.rs',
);
assertContains(
    sourceCapability,
    'mod proof;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'mod constructor_position;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'mod prefix_call;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'mod walk;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'pub(crate) use proof::module_source_capabilities',
    'source_capability.rs',
);
assertContains(
    sourceCapabilityPrefixCall,
    'struct PrefixCallHead',
    'source capability prefix call-head tracking must be shared',
);
assertContains(
    sourceCapabilityPrefixCall,
    'pub(super) fn call_head_symbol',
    'source capability call-head symbol extraction must be shared',
);
assertContains(
    sourceCapabilityPrefixCall,
    'fn prefix_item_allows_following_call_head',
    'source capability prefix initializer positions must be centrally modeled',
);
assertContains(
    sourceCapabilityPrefixCall,
    'match item',
    'source capability prefix call-head item classification must use enum matching',
);
assertContains(
    sourceCapabilityPrefixCall,
    'match symbol',
    'source capability prefix call-head symbol classification must use enum matching',
);
assertNotContains(
    sourceCapabilityPrefixCall,
    'matches!',
    'source capability prefix call-head classification must not hide enum coverage in matches',
);
assertContains(
    sourceCapabilityConstructorPosition,
    'pub(super) fn explicit_constructor_symbol',
    'source capability explicit constructor positions must be centrally modeled',
);
assertContains(
    sourceCapabilityConstructorPosition,
    'type_args.is_empty()',
    'source capability explicit constructor evidence must require explicit type arguments',
);
assertContains(
    sourceCapabilityConstructorPosition,
    'match item',
    'source capability explicit constructor item classification must use enum matching',
);
assertContains(
    sourceCapabilityWalk,
    'pub(super) trait SourceCapabilityObserver',
    'source capability proof traversal must be observer-driven',
);
assertContains(
    sourceCapabilityWalk,
    'pub(super) fn walk_module_capability_evidence',
    'source capability proof traversal must have one module entry point',
);
assertContains(
    sourceCapabilityWalk,
    'PrefixCallHead::new',
    'source capability proof traversal must centralize prefix call-head tracking',
);
assertContains(
    sourceCapabilityWalk,
    'fn observe_call_head_item',
    'source capability proof traversal must restrict call-head evidence centrally',
);
assertContains(
    sourceCapabilityWalk,
    'fn observe_explicit_constructor_item',
    'source capability proof traversal must model nested explicit constructor evidence centrally',
);
assertContains(
    sourceCapabilityWalk,
    'match item',
    'source capability proof traversal must classify call-head items with enum matching',
);
assertContains(
    sourceCapabilityWalk,
    'block_scope.bind_stmt_locals(stmt)',
    'source capability proof traversal must centralize statement scope updates',
);
assertContains(
    sourceCapabilityWalk,
    'arm_scope.bind_match_pattern(&arm.pattern)',
    'source capability proof traversal must centralize match-arm scope updates',
);
assertContains(
    sourceCapabilityWalk,
    'observer.observe_raw_body',
    'source capability proof traversal must centralize raw body observation',
);
assertContains(
    sourceCapabilityWalk,
    'observer.observe_struct_definition',
    'source capability proof traversal must centralize struct definition observation',
);
assertContains(
    sourceCapabilityProof,
    'struct SourceCapabilityProof',
    'source capability proof must have a single typed proof value',
);
assertContains(
    sourceCapabilityProof,
    'struct SourceCapabilityProofCollector',
    'source capability proof must have one collector for all capability domains',
);
assertContains(
    sourceCapabilityProof,
    'impl SourceCapabilityObserver for SourceCapabilityProofCollector',
    'source capability proof must consume the shared proof walker once',
);
assertContains(
    sourceCapabilityProof,
    'fn observe_call_head_symbol',
    'source capability proof must restrict source symbol evidence to shared call-head callbacks',
);
assertContains(
    sourceCapabilityProof,
    'fn observe_explicit_constructor_symbol',
    'source capability proof must classify nested explicit constructor evidence through the unified proof collector',
);
assertContains(
    sourceCapabilityProof,
    'fn observe_intrinsic',
    'source capability proof must classify intrinsic callbacks through the unified proof collector',
);
assertContains(
    sourceCapabilityProof,
    'fn observe_raw_body',
    'source capability proof must classify raw bodies through the unified proof collector',
);
assertContains(
    sourceCapabilityProof,
    'fn observe_struct_definition',
    'source capability proof must classify compiler memory type definitions through the unified proof collector',
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
    sourceCapabilityProof,
    'compiler_memory_type_from_struct_def',
    'source_capability/proof.rs',
);
assertContains(
    resourcePrimitives,
    'pub(crate) fn compiler_memory_type_from_constructor_name',
    'resource_primitives.rs',
);
assertContains(
    resourcePrimitives,
    'match name',
    'resource_primitives.rs',
);
assertContains(
    resourcePrimitives,
    'pub(crate) fn compiler_memory_type_of_type',
    'resource_primitives.rs',
);
assertContains(
    resourcePrimitives,
    'pub(crate) fn type_is_raw_pointer',
    'resource_primitives.rs',
);
assertContains(
    resourcePrimitives,
    'pub(crate) fn type_is_owner_token',
    'resource_primitives.rs',
);
assertContains(
    resourcePrimitives,
    'pub(crate) enum MemoryHelperPrimitive',
    'resource_primitives.rs',
);
assertContains(
    sourceCapabilityMemoryTypeDefinition,
    'compiler_memory_type_from_constructor_name(def.name.name.as_str())',
    'source_capability/memory_type_definition.rs must use central compiler memory type classification',
);
assertContains(
    sourceCapabilityMemoryTypeDefinition,
    'pub(in crate::source_capability) fn compiler_memory_type_from_struct_def',
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
    sourceCapabilityProof,
    'owner_aggregate_symbol_evidence',
    'source_capability/proof.rs',
);
assertContains(
    sourceCapabilityProof,
    'OwnerAggregateProofEvidence',
    'source_capability/proof.rs',
);
assertContains(
    sourceCapabilityOwnerAggregateEvidence,
    'enum OwnerAggregateCapabilityEvidence',
    'source_capability/owner_aggregate/evidence.rs',
);
assertContains(sourceCapabilityOwnerAggregate, 'mod evidence;', 'source_capability/owner_aggregate.rs');
assertContains(sourceCapabilityOwnerAggregate, 'mod context;', 'source_capability/owner_aggregate.rs');
assertContains(
    sourceCapabilityOwnerAggregate,
    'OwnerAggregateEvidenceContext',
    'owner aggregate source evidence must expose context to the unified proof collector',
);
assertContains(
    sourceCapabilityOwnerAggregate,
    'OwnerAggregateCapabilityEvidence',
    'owner aggregate source evidence must expose typed evidence to the unified proof collector',
);
assertContains(
    sourceCapabilityProof,
    'constructors: BTreeSet<String>',
    'owner aggregate constructor evidence must be tracked by constructor name',
);
assertContains(
    sourceCapabilityProof,
    'fn observe_intrinsic',
    'owner aggregate source capability must inspect intrinsic field access evidence through the shared proof walker',
);
assertContains(
    sourceCapabilityProof,
    'fn observe_call_head_symbol',
    'owner aggregate constructor evidence must be restricted to shared call-head callbacks',
);
assertContains(
    sourceCapabilityOwnerAggregateContext,
    'OwnerAggregateEvidenceContext',
    'owner aggregate evidence must carry declaration context for syntax filtering',
);
assertContains(
    sourceCapabilityOwnerAggregate,
    'mod field_imports;',
    'owner aggregate core/field import proof must be separated from evidence classification',
);
assertContains(
    sourceCapabilityOwnerAggregateContext,
    'CoreFieldAccessorImports',
    'owner aggregate context must depend on the dedicated core/field import proof',
);
assertContains(
    sourceCapabilityOwnerAggregateFieldImports,
    'field_aliases',
    'owner aggregate field evidence must track core/field import aliases',
);
assertContains(
    sourceCapabilityOwnerAggregateFieldImports,
    'enum CoreFieldAccessorMember',
    'owner aggregate field accessor names must be represented as an enum domain',
);
assertContains(
    sourceCapabilityOwnerAggregateFieldImports,
    'ImportClause::Open',
    'owner aggregate field evidence must prove open core/field imports',
);
assertContains(
    sourceCapabilityOwnerAggregateFieldImports,
    'ImportClause::Merge',
    'owner aggregate field evidence must follow resolver merge import visibility',
);
assertContains(
    sourceCapabilityOwnerAggregateFieldImports,
    'split_leading_qualifier',
    'owner aggregate field evidence must check qualified import aliases',
);
assertNotContains(
    sourceCapabilityOwnerAggregateEvidence,
    '"get" | "get_ref" | "put" | "get_field" | "get_field_ref"',
    'owner aggregate field evidence must not accept broad helper names without import proof',
);
assertContains(
    sourceCapabilityOwnerAggregateEvidence,
    'crate::qualified_name::member_tail(symbol) != symbol',
    'owner aggregate constructor evidence must ignore qualified enum variants',
);
assertContains(
    sourceCapabilityOwnerAggregateEvidence,
    'context.is_enum_variant(base)',
    'owner aggregate constructor evidence must ignore same-module enum variants',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_ignores_qualified_enum_variant_constructors()',
    'loader.rs owner aggregate enum variant regression',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_ignores_same_module_enum_variant_constructors()',
    'loader.rs owner aggregate same-module enum variant regression',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_requires_constructor_call_head()',
    'loader.rs owner aggregate call-head regression',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_accepts_constructor_initializer_call_head()',
    'loader.rs owner aggregate initializer constructor call-head regression',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_accepts_field_initializer_call_head()',
    'loader.rs owner aggregate initializer field call-head regression',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_rejects_unrelated_get_call_head()',
    'loader.rs owner aggregate unrelated get regression',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_accepts_field_alias_import_call_head()',
    'loader.rs owner aggregate core field alias import regression',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_accepts_field_merge_import_call_head()',
    'loader.rs owner aggregate core field merge import regression',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_does_not_share_unrelated_constructor_evidence()',
    'loader.rs owner aggregate constructor-name regression',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_accepts_intrinsic_field_evidence()',
    'loader.rs owner aggregate intrinsic field evidence regression',
);
assertContains(
    loader,
    'fn owner_aggregate_boundary_accepts_same_module_struct_constructor_evidence()',
    'loader.rs owner aggregate same-module constructor regression',
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
assertContains(
    sourceCapabilityScope,
    'shadows_symbol_or_qualifier',
    'source capability scope must centralize qualified shadowing',
);
assertContains(
    sourceCapabilityScope,
    'split_leading_qualifier',
    'source capability scope must check qualified symbol aliases before helper classification',
);
assertNotContains(
    sourceCapabilityScope,
    'Stmt::StructDef(def) => self.bind(&def.name.name)',
    'source capability scope must not treat type definitions as value-level shadows',
);
assertContains(
    sourceCapabilityProof,
    'raw_memory_op_from_name',
    'source_capability/proof.rs',
);
assertContains(
    sourceCapabilityProof,
    'function_has_raw_memory_evidence',
    'raw helper definitions must grant their operation only when the body has raw evidence',
);
assertContains(
    sourceCapabilityProof,
    'function_has_raw_memory_evidence.last_mut()',
    'raw helper definition evidence must be scoped to the currently walked function frame',
);
assertContains(
    sourceCapabilityProof,
    'scope.shadows_symbol_or_qualifier(symbol)',
    'raw memory source evidence must reject shadowed qualified helper-looking symbols',
);
assertNotContains(
    sourceCapabilityProof,
    'for frame in &mut self.function_has_raw_memory_evidence',
    'nested raw helper evidence must not mark every active function frame',
);
assertNotContains(
    sourceCapabilityRawMemory,
    'PrefixCallHead::new',
    'raw memory source evidence must not reimplement prefix call-head traversal',
);
assertNotContains(
    sourceCapabilityOwnerAggregate,
    'PrefixCallHead::new',
    'owner aggregate source evidence must not reimplement prefix call-head traversal',
);
assertNotContains(
    sourceCapabilityProof,
    'PrefixItem::Match',
    'unified source capability proof must not duplicate AST traversal',
);
assertNotContains(
    sourceCapabilityRawMemory,
    'SourceCapabilityObserver',
    'raw memory source evidence must not own a domain-specific proof walker',
);
assertNotContains(
    sourceCapabilityOwnerAggregate,
    'SourceCapabilityObserver',
    'owner aggregate source evidence must not own a domain-specific proof walker',
);
assertContains(
    sourceCapabilityRawMemoryEvidence,
    'MemoryHelperPrimitive::from_symbol(name)',
    'raw address helper evidence must use the central memory helper registry',
);
for (const helperName of [
    '"mem_ptr_wrap"',
    '"mem_ptr_addr"',
    '"mem_ptr_add"',
    '"region_new"',
    '"region_ptr"',
    '"region_ptr_at"',
    '"region_token_raw_ref"',
    '"str_addr"',
    '"str_from_addr_unchecked"',
]) {
    assertNotContains(
        sourceCapabilityRawMemoryEvidence,
        helperName,
        'source_capability/raw_memory/evidence.rs must not duplicate memory helper names',
    );
}
assertContains(
    resourceLowerRawAddress,
    'MemoryHelperPrimitive::from_base_name',
    'resource/lower_raw_address.rs must use central helper classification for function references',
);
assertContains(
    resourceLowerRawAddress,
    'MemoryHelperPrimitive::from_symbol',
    'resource/lower_raw_address.rs must use central helper classification for named calls',
);
assertContains(
    resourceLowerRawAddressReturn,
    'MemoryHelperPrimitive::from_symbol',
    'resource/lower_raw_address_return.rs must use central helper classification for return expressions',
);
assertContains(
    resourceLowerRawAddressReturn,
    'MemoryHelperPrimitive::from_base_name',
    'resource/lower_raw_address_return.rs must use central helper classification for function references',
);
assertNotContains(
    resourceLowerRawAddressReturn,
    'compiler_memory_type_from_constructor_name',
    'resource/lower_raw_address_return.rs must not classify compiler memory constructs by constructor name',
);
assertContains(
    resourceOwnerRawAddress,
    'MemoryHelperPrimitive::returns_non_owning_address_view',
    'resource/owner_raw_address.rs must use central non-owning view classification',
);
assertContains(
    resourceOwnerFlow,
    'type_is_owner_token(self.types, output.ty)',
    'resource/owner_flow.rs must use proven TypeCtx identity for owner-token construct classification',
);
assertNotContains(
    resourceOwnerFlow,
    'compiler_memory_type_from_constructor_name',
    'resource/owner_flow.rs must not classify compiler memory constructs by constructor name',
);
assertNotContains(
    resourceLowerRawAddressPlace,
    'fn is_named_struct_type',
    'resource/lower_raw_address_place.rs must not reintroduce ad hoc memory type classification',
);
assertNotContains(
    resourcePlaceUtils,
    'name == "MemPtr"',
    'resource/place_utils.rs must not reintroduce ad hoc MemPtr classification',
);
assertNotContains(
    resourcePlaceUtils,
    'name == "RegionToken"',
    'resource/place_utils.rs must not reintroduce ad hoc RegionToken classification',
);
assertNotContains(
    sourceCapabilityRawMemory,
    'enum RawOwnerBoundaryHelper',
    'checked owner wrappers must not be raw boundary evidence',
);
assertNotContains(
    sourceCapabilityRawMemory,
    '"alloc_region"',
    'safe alloc_region wrapper must not be raw boundary evidence',
);
assertContains(sourceCapabilityRawMemoryEvidence, 'RestrictedConstructor', 'source_capability/raw_memory/evidence.rs');
assertNotContains(loader, 'RAW_MEMORY_BOUNDARY_STDLIB_PATHS', 'loader.rs');
assertNotContains(loader, 'configured_raw_memory_boundary_path', 'loader.rs');
assertContains(loader, 'configured_stdlib_source_path', 'loader.rs');
assertContains(loader, 'module_source_capabilities(module)', 'loader.rs');
for (const oldCapabilityCollector of [
    'module_has_raw_memory_boundary_evidence',
    'module_raw_memory_operation_evidence',
    'module_raw_body_memory_operation_evidence',
    'module_owner_aggregate_constructor_evidence',
    'module_has_owner_aggregate_field_evidence',
    'module_compiler_memory_type_definitions(module)',
]) {
    assertNotContains(
        loader,
        oldCapabilityCollector,
        'loader.rs must consume one unified source capability proof',
    );
}
assertContains(
    loader,
    'fn raw_memory_boundary_accepts_raw_helper_definition_evidence()',
    'loader.rs raw helper definition operation evidence regression',
);
assertContains(
    loader,
    'fn raw_memory_boundary_keeps_raw_helper_body_evidence_function_scoped()',
    'loader.rs raw helper definition nested-scope regression',
);
assertContains(
    loader,
    'fn raw_memory_boundary_ignores_shadowed_qualified_parameter_names()',
    'loader.rs raw qualified shadow regression',
);
assertContains(
    loader,
    'fn raw_memory_boundary_requires_raw_operation_call_head()',
    'loader.rs raw operation call-head regression',
);
assertContains(
    loader,
    'fn raw_memory_boundary_requires_raw_structural_call_head()',
    'loader.rs raw structural call-head regression',
);
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
assertNotContains(
    cellGateBody[0],
    'raw_memory_operation_boundary_allowed',
    'compiler.rs Resource cell gate',
);
assertNotContains(
    cellGateBody[0],
    'raw_memory_structural_boundary_allowed',
    'compiler.rs Resource cell gate',
);
const ownerGateBody = compiler.match(/fn run_resource_owner_obligation_gate\([\s\S]*?\n}\n\nfn resource_owner_diagnostic_to_error/);
assert(ownerGateBody, 'compiler.rs must expose Resource owner gate body');
assertNotContains(
    ownerGateBody[0],
    'raw_memory_boundary_allowed',
    'compiler.rs Resource owner gate',
);
assertNotContains(
    ownerGateBody[0],
    'raw_memory_operation_boundary_allowed',
    'compiler.rs Resource owner gate',
);
assertNotContains(
    ownerGateBody[0],
    'raw_memory_structural_boundary_allowed',
    'compiler.rs Resource owner gate',
);
assertMatches(
    compiler,
    /ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction\s*\{\s*operation,\s*\.\.\s*\}\s*=>\s*\{\s*let Some\(span\) = resource_effect_boundary_diagnostic_span\(diagnostic\) else \{\s*return false;\s*\};\s*source_map\s*\.map\(\|map\| map\.raw_memory_operation_boundary_allowed\(span\.file_id, \*operation\)\)\s*\.unwrap_or\(false\)\s*\}/,
    'compiler.rs unsafe memory suppression must require the exact raw-memory operation capability',
);
assertMatches(
    compiler,
    /ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary\s*\{\s*operation,\s*\.\.\s*\}\s*=>\s*\{\s*let Some\(span\) = resource_effect_boundary_diagnostic_span\(diagnostic\) else \{\s*return false;\s*\};\s*source_map\s*\.map\(\|map\| map\.raw_memory_operation_boundary_allowed\(span\.file_id, \*operation\)\)\s*\.unwrap_or\(false\)\s*\}/,
    'compiler.rs raw memory outside-boundary suppression must require the exact raw-memory operation capability',
);
assertMatches(
    compiler,
    /ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc\s*\{\s*operation,\s*\.\.\s*\}\s*=>\s*\{\s*let Some\(span\) = resource_effect_boundary_diagnostic_span\(diagnostic\) else \{\s*return false;\s*\};\s*source_map\s*\.map\(\|map\| raw_identity_escape_allowed\(\*operation, span, map\)\)\s*\.unwrap_or\(false\)\s*\}/,
    'compiler.rs raw identity escape must require structural or alloc/realloc operation source capability',
);
assertContains(
    compiler,
    'fn raw_identity_escape_allowed(operation: RawMemoryOp, span: Span, source_map: &SourceMap) -> bool',
    'compiler.rs raw identity escape helper',
);
assertContains(
    compiler,
    'matches!(operation, RawMemoryOp::Alloc | RawMemoryOp::Realloc)',
    'compiler.rs raw identity escape operation-specific suppression',
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
    compiler,
    'fn resource_effect_gate_requires_matching_raw_operation_capability()',
    'compiler.rs raw-boundary operation-specific unit regression',
);
assertContains(
    compiler,
    'fn resource_effect_gate_allows_raw_alloc_identity_with_alloc_capability()',
    'compiler.rs raw identity alloc operation unit regression',
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
    ['compiler_memory_type.rs', 90],
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
