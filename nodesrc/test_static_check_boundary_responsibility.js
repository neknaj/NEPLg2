#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const CORE_SRC = path.join(ROOT, 'nepl-core', 'src');
const NEPL_LANGUAGE_LIB = path.join(ROOT, 'nepl-language', 'src', 'lib.rs');
const NEPL_WEB_LIB = path.join(ROOT, 'nepl-web', 'src', 'lib.rs');
const DIAGNOSTIC_CODES = path.join(CORE_SRC, 'diagnostic_codes.rs');
const CORE_LIB = path.join(CORE_SRC, 'lib.rs');
const BACKEND_SCALAR_TYPE = path.join(CORE_SRC, 'backend_scalar_type.rs');
const LAYOUT = path.join(CORE_SRC, 'layout.rs');
const WASM_SHARED = path.join(CORE_SRC, 'wasm_shared.rs');
const CODEGEN_WASM = path.join(CORE_SRC, 'codegen_wasm.rs');
const CODEGEN_LLVM = path.join(CORE_SRC, 'codegen_llvm.rs');
const CODEGEN_LLVM_SCALAR_INTRINSIC = path.join(CORE_SRC, 'codegen_llvm', 'scalar_intrinsic.rs');
const CODEGEN_LLVM_TYPE_MAP = path.join(CORE_SRC, 'codegen_llvm', 'type_map.rs');
const CODEGEN_PRECHECK = path.join(CORE_SRC, 'passes', 'codegen_precheck.rs');
const TYPES_RS = path.join(CORE_SRC, 'types.rs');
const TYPECHECK_ROOT = path.join(CORE_SRC, 'typecheck.rs');
const TYPECHECK_DIR = path.join(CORE_SRC, 'typecheck');
const RESOURCE_ROOT = path.join(CORE_SRC, 'resource', 'mod.rs');
const RESOURCE_LOWER = path.join(CORE_SRC, 'resource', 'lower.rs');
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
const INTRINSIC_KINDS = path.join(CORE_SRC, 'intrinsic_kinds.rs');
const LOADER = path.join(CORE_SRC, 'loader.rs');
const SOURCE_MAP = path.join(CORE_SRC, 'source_map.rs');
const RESOURCE_PRIMITIVES = path.join(CORE_SRC, 'resource_primitives.rs');
const RESOURCE_PRIMITIVES_COMPILER_MEMORY = path.join(
    CORE_SRC,
    'resource_primitives',
    'compiler_memory.rs',
);
const RESOURCE_PRIMITIVES_MEMORY_HELPER = path.join(
    CORE_SRC,
    'resource_primitives',
    'memory_helper.rs',
);
const SOURCE_CAPABILITY = path.join(CORE_SRC, 'source_capability.rs');
const SOURCE_CAPABILITY_MEMORY_TYPE_DEFINITION = path.join(
    CORE_SRC,
    'source_capability',
    'memory_type_definition.rs',
);
const SOURCE_CAPABILITY_BINDING = path.join(
    CORE_SRC,
    'source_capability',
    'binding.rs',
);
const SOURCE_CAPABILITY_COMPILER_MEMORY_FIELD = path.join(
    CORE_SRC,
    'source_capability',
    'compiler_memory_field.rs',
);
const SOURCE_CAPABILITY_FACT = path.join(
    CORE_SRC,
    'source_capability',
    'fact.rs',
);
const SOURCE_CAPABILITY_CONSTRUCTOR_POSITION = path.join(
    CORE_SRC,
    'source_capability',
    'constructor_position.rs',
);
const SOURCE_CAPABILITY_FIELD_SELECTOR = path.join(
    CORE_SRC,
    'source_capability',
    'field_selector.rs',
);
const SOURCE_CAPABILITY_IMPORT_PATH = path.join(
    CORE_SRC,
    'source_capability',
    'import_path.rs',
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
const SOURCE_CAPABILITY_PROOF_BUILDER = path.join(
    CORE_SRC,
    'source_capability',
    'proof_builder.rs',
);
const SOURCE_CAPABILITY_RAW_EVIDENCE_GATE = path.join(
    CORE_SRC,
    'source_capability',
    'raw_evidence_gate.rs',
);
const SOURCE_CAPABILITY_RAW_OPERATION_PROOF = path.join(
    CORE_SRC,
    'source_capability',
    'raw_operation_proof.rs',
);
const SOURCE_CAPABILITY_RULE = path.join(CORE_SRC, 'source_capability', 'rule.rs');
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
const SOURCE_CAPABILITY_TOP_LEVEL_RAW_CALLS = path.join(
    CORE_SRC,
    'source_capability',
    'top_level_raw_calls.rs',
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

function assertNotMatches(text, pattern, label) {
    assert(!pattern.test(text), `${label} must not match ${pattern}`);
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
const resourcePrimitivesCompilerMemory = assertFile(
    RESOURCE_PRIMITIVES_COMPILER_MEMORY,
    'resource_primitives/compiler_memory.rs',
);
const resourcePrimitivesMemoryHelper = assertFile(
    RESOURCE_PRIMITIVES_MEMORY_HELPER,
    'resource_primitives/memory_helper.rs',
);
const resourceLower = assertFile(RESOURCE_LOWER, 'resource/lower.rs');
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
const intrinsicKinds = assertFile(INTRINSIC_KINDS, 'intrinsic_kinds.rs');
const loader = assertFile(LOADER, 'loader.rs');
const sourceMap = assertFile(SOURCE_MAP, 'source_map.rs');
const sourceCapability = assertFile(SOURCE_CAPABILITY, 'source_capability.rs');
const sourceCapabilityMemoryTypeDefinition = assertFile(
    SOURCE_CAPABILITY_MEMORY_TYPE_DEFINITION,
    'source_capability/memory_type_definition.rs',
);
const sourceCapabilityBinding = assertFile(
    SOURCE_CAPABILITY_BINDING,
    'source_capability/binding.rs',
);
const sourceCapabilityCompilerMemoryField = assertFile(
    SOURCE_CAPABILITY_COMPILER_MEMORY_FIELD,
    'source_capability/compiler_memory_field.rs',
);
const sourceCapabilityFact = assertFile(
    SOURCE_CAPABILITY_FACT,
    'source_capability/fact.rs',
);
const sourceCapabilityConstructorPosition = assertFile(
    SOURCE_CAPABILITY_CONSTRUCTOR_POSITION,
    'source_capability/constructor_position.rs',
);
const sourceCapabilityFieldSelector = assertFile(
    SOURCE_CAPABILITY_FIELD_SELECTOR,
    'source_capability/field_selector.rs',
);
const sourceCapabilityImportPath = assertFile(
    SOURCE_CAPABILITY_IMPORT_PATH,
    'source_capability/import_path.rs',
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
const sourceCapabilityProofBuilder = assertFile(
    SOURCE_CAPABILITY_PROOF_BUILDER,
    'source_capability/proof_builder.rs',
);
const sourceCapabilityRawEvidenceGate = assertFile(
    SOURCE_CAPABILITY_RAW_EVIDENCE_GATE,
    'source_capability/raw_evidence_gate.rs',
);
const sourceCapabilityRawOperationProof = assertFile(
    SOURCE_CAPABILITY_RAW_OPERATION_PROOF,
    'source_capability/raw_operation_proof.rs',
);
const sourceCapabilityRule = assertFile(SOURCE_CAPABILITY_RULE, 'source_capability/rule.rs');
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
const sourceCapabilityTopLevelRawCalls = assertFile(
    SOURCE_CAPABILITY_TOP_LEVEL_RAW_CALLS,
    'source_capability/top_level_raw_calls.rs',
);
const passesMod = assertFile(PASSES_MOD, 'passes/mod.rs');
const dropInsertion = assertFile(DROP_INSERTION, 'passes/drop_insertion.rs');
const resourceIrTests = assertFile(RESOURCE_IR_TESTS, 'nepl-core/tests/resource_ir.rs');
const testHarness = assertFile(TEST_HARNESS, 'nepl-core/tests/harness.rs');
const neplg2Tests = assertFile(NEPLG2_TESTS, 'nepl-core/tests/neplg2.rs');
const neplLanguageLib = assertFile(NEPL_LANGUAGE_LIB, 'nepl-language/src/lib.rs');
const neplWebLib = assertFile(NEPL_WEB_LIB, 'nepl-web/src/lib.rs');
const diagnosticCodes = assertFile(DIAGNOSTIC_CODES, 'diagnostic_codes.rs');
const coreLib = assertFile(CORE_LIB, 'lib.rs');
const backendScalarType = assertFile(BACKEND_SCALAR_TYPE, 'backend_scalar_type.rs');
const layout = assertFile(LAYOUT, 'layout.rs');
const wasmShared = assertFile(WASM_SHARED, 'wasm_shared.rs');
const codegenWasm = assertFile(CODEGEN_WASM, 'codegen_wasm.rs');
const codegenLlvm = assertFile(CODEGEN_LLVM, 'codegen_llvm.rs');
const codegenLlvmScalarIntrinsic = assertFile(
    CODEGEN_LLVM_SCALAR_INTRINSIC,
    'codegen_llvm/scalar_intrinsic.rs',
);
const codegenLlvmTypeMap = assertFile(CODEGEN_LLVM_TYPE_MAP, 'codegen_llvm/type_map.rs');
const codegenPrecheck = assertFile(CODEGEN_PRECHECK, 'passes/codegen_precheck.rs');
const typesRs = assertFile(TYPES_RS, 'types.rs');
const typecheckMatchCheck = assertFile(
    path.join(TYPECHECK_DIR, 'match_check.rs'),
    'typecheck/match_check.rs',
);
const typecheckModel = assertFile(
    path.join(TYPECHECK_DIR, 'model.rs'),
    'typecheck/model.rs',
);
const typecheckBindingRules = assertFile(
    path.join(TYPECHECK_DIR, 'binding_rules.rs'),
    'typecheck/binding_rules.rs',
);
const typecheckDriver = assertFile(
    path.join(TYPECHECK_DIR, 'driver.rs'),
    'typecheck/driver.rs',
);
const typecheckCompilerMemoryType = assertFile(
    path.join(TYPECHECK_DIR, 'compiler_memory_type.rs'),
    'typecheck/compiler_memory_type.rs',
);
const typecheckEffectCheck = assertFile(
    path.join(TYPECHECK_DIR, 'effect_check.rs'),
    'typecheck/effect_check.rs',
);
const typecheckConstructorApply = assertFile(
    path.join(TYPECHECK_DIR, 'constructor_apply.rs'),
    'typecheck/constructor_apply.rs',
);
const typecheckControlApply = assertFile(
    path.join(TYPECHECK_DIR, 'control_apply.rs'),
    'typecheck/control_apply.rs',
);
const typecheckControlSpecial = assertFile(
    path.join(TYPECHECK_DIR, 'control_special.rs'),
    'typecheck/control_special.rs',
);
const typecheckStructShape = assertFile(
    path.join(TYPECHECK_DIR, 'struct_shape.rs'),
    'typecheck/struct_shape.rs',
);
const typecheckCallReduction = assertFile(
    path.join(TYPECHECK_DIR, 'call_reduction.rs'),
    'typecheck/call_reduction.rs',
);
const typecheckFieldAccess = assertFile(
    path.join(TYPECHECK_DIR, 'field_access.rs'),
    'typecheck/field_access.rs',
);
const typecheckFunctionApply = assertFile(
    path.join(TYPECHECK_DIR, 'function_apply.rs'),
    'typecheck/function_apply.rs',
);
const typecheckFunctionCheck = assertFile(
    path.join(TYPECHECK_DIR, 'function_check.rs'),
    'typecheck/function_check.rs',
);
const typecheckPrefixCheck = assertFile(
    path.join(TYPECHECK_DIR, 'prefix_check.rs'),
    'typecheck/prefix_check.rs',
);
const typecheckSyntaxHelpers = assertFile(
    path.join(TYPECHECK_DIR, 'syntax_helpers.rs'),
    'typecheck/syntax_helpers.rs',
);
const resourceCoverageHirProjection = assertFile(
    path.join(CORE_SRC, 'resource', 'coverage_hir_projection.rs'),
    'resource/coverage_hir_projection.rs',
);
const resourceLowerAggregate = assertFile(
    path.join(CORE_SRC, 'resource', 'lower_aggregate.rs'),
    'resource/lower_aggregate.rs',
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
    'control_special',
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
    'struct_shape',
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

assertLineLimit(path.join(TYPECHECK_DIR, 'control_special.rs'), 'typecheck/control_special.rs', 80);
assertContains(
    typecheckControlSpecial,
    'pub(super) enum ControlSpecialFunction',
    'typecheck/control_special.rs must own control special spelling',
);
assertContains(
    typecheckControlSpecial,
    'pub(super) fn from_name',
    'typecheck/control_special.rs must classify control special names once',
);
assertContains(
    typecheckControlSpecial,
    'pub(super) const fn name',
    'typecheck/control_special.rs must expose canonical control special names',
);
assertContains(
    typecheckControlApply,
    'ControlSpecialFunction::from_name(name)',
    'typecheck/control_apply.rs must dispatch control specials through the enum classifier',
);
assertContains(
    typecheckControlApply,
    'Some(ControlSpecialFunction::If)',
    'typecheck/control_apply.rs must exhaustively match the if special form',
);
assertContains(
    typecheckControlApply,
    'Some(ControlSpecialFunction::While)',
    'typecheck/control_apply.rs must exhaustively match the while special form',
);
assertNotContains(
    typecheckControlApply,
    'name == "if"',
    'typecheck/control_apply.rs must not use direct if string guards',
);
assertNotContains(
    typecheckControlApply,
    'name == "while"',
    'typecheck/control_apply.rs must not use direct while string guards',
);
assertContains(
    typecheckPrefixCheck,
    'ControlSpecialFunction::If.name()',
    'typecheck/prefix_check.rs must construct if special vars through ControlSpecialFunction',
);
assertContains(
    typecheckPrefixCheck,
    'ControlSpecialFunction::While.name()',
    'typecheck/prefix_check.rs must construct while special vars through ControlSpecialFunction',
);

assertLineLimit(path.join(TYPECHECK_DIR, 'struct_shape.rs'), 'typecheck/struct_shape.rs', 140);
assertContains(
    typecheckStructShape,
    'pub(super) enum StructConstructorShape',
    'typecheck/struct_shape.rs must own struct constructor shape classification',
);
assertContains(
    typecheckStructShape,
    'fn from_name',
    'typecheck/struct_shape.rs must classify unit-like struct tag spelling once',
);
assertContains(
    typecheckModel,
    'constructor_shape: StructConstructorShape',
    'StructInfo must store the classified constructor shape',
);
assertContains(
    typecheckDriver,
    'StructConstructorShape::classify(&ctx, &fs, &f_names)',
    'typecheck/driver.rs must classify struct constructor shape once',
);
assertContains(
    typecheckDriver,
    'constructor_shape.constructor_params(&fs)',
    'typecheck/driver.rs must derive constructor params from StructConstructorShape',
);
assertContains(
    typecheckConstructorApply,
    'StructConstructorShape::UnitLikeTag',
    'typecheck/constructor_apply.rs must lower unit-like structs from StructConstructorShape',
);
assertNotContains(
    typecheckDriver,
    'f_names[0] == "tag"',
    'typecheck/driver.rs must not classify unit-like structs with direct tag indexing',
);
assertNotContains(
    typecheckConstructorApply,
    'field_names[0] == "tag"',
    'typecheck/constructor_apply.rs must not reclassify unit-like structs with direct tag indexing',
);

assertContains(typecheckRoot, 'pub use driver::{typecheck, TypeCheckResult};', 'typecheck.rs');
assertContains(
    intrinsicKinds,
    'fn from_intrinsic_name',
    'intrinsic_kinds.rs must keep field accessor intrinsic spelling on FieldAccessorKind',
);
assertContains(
    intrinsicKinds,
    'const fn argument_count',
    'intrinsic_kinds.rs must keep field accessor intrinsic arity on FieldAccessorKind',
);
assertContains(
    intrinsicKinds,
    'from_core_field_member_name',
    'FieldAccessorKind must own core/field source member spelling',
);
assertContains(
    intrinsicKinds,
    'core_field_member_name',
    'FieldAccessorKind must expose core/field source member spelling for round-trip tests',
);
assertContains(
    intrinsicKinds,
    'pub(crate) enum ScalarIntrinsicKind',
    'intrinsic_kinds.rs must keep scalar intrinsic signatures in a shared typed enum domain',
);
assertContains(
    intrinsicKinds,
    'pub(crate) enum ScalarIntrinsicType',
    'intrinsic_kinds.rs must keep scalar intrinsic types in a shared typed enum domain',
);
assertContains(
    intrinsicKinds,
    'pub(crate) enum ScalarIntrinsicBackendOp',
    'intrinsic_kinds.rs must keep scalar intrinsic backend semantics in a typed enum domain',
);
assertContains(
    intrinsicKinds,
    'pub(crate) const fn backend_op',
    'ScalarIntrinsicKind must own backend lowering semantics',
);
assertContains(
    intrinsicKinds,
    'pub(crate) enum CoreIntrinsicKind',
    'intrinsic_kinds.rs must keep shared core intrinsic spelling in a typed enum domain',
);
assertContains(
    intrinsicKinds,
    'pub(crate) enum CoreIntrinsicResultKind',
    'intrinsic_kinds.rs must keep shared core intrinsic result types in a typed enum domain',
);
assertContains(
    intrinsicKinds,
    'pub(crate) fn layout_i32_value',
    'intrinsic_kinds.rs must keep layout intrinsic value semantics on CoreIntrinsicKind',
);
assertNotContains(
    typecheckModel,
    'enum CoreIntrinsicKind',
    'typecheck/model.rs must not re-localize shared core intrinsic classification',
);
assertNotContains(
    typecheckModel,
    'enum CoreIntrinsicResultKind',
    'typecheck/model.rs must not re-localize shared core intrinsic result classification',
);
assertNotContains(
    typecheckModel,
    'enum ScalarIntrinsicKind',
    'typecheck/model.rs must not re-localize shared scalar intrinsic classification',
);
assertNotContains(
    typecheckModel,
    'enum ScalarIntrinsicType',
    'typecheck/model.rs must not re-localize shared scalar intrinsic type classification',
);
assertContains(
    typecheckBindingRules,
    'FieldAccessorKind::from_intrinsic_name',
    'typecheck/binding_rules.rs must use typed field accessor intrinsic classification',
);
assertNotContains(
    typecheckBindingRules,
    'intrin.name == "get_field"',
    'typecheck/binding_rules.rs must not duplicate get_field spelling outside FieldAccessorKind',
);
assertNotContains(
    typecheckBindingRules,
    'intrin.name == "get_field_ref"',
    'typecheck/binding_rules.rs must not duplicate get_field_ref spelling outside FieldAccessorKind',
);
assertNotContains(
    typecheckBindingRules,
    'intrin.name == "set_field"',
    'typecheck/binding_rules.rs must not duplicate set_field spelling outside FieldAccessorKind',
);
assertContains(
    typecheckPrefixCheck,
    'FieldAccessorKind::from_intrinsic_name',
    'typecheck/prefix_check.rs must use typed field accessor intrinsic classification',
);
assertContains(
    resourceCoverageHirProjection,
    'FieldAccessorKind::from_intrinsic_name',
    'resource/coverage_hir_projection.rs must use shared field accessor intrinsic classification',
);
assertContains(
    resourceLowerAggregate,
    'FieldAccessorKind::from_intrinsic_name',
    'resource/lower_aggregate.rs must use shared field accessor intrinsic classification',
);
assertContains(
    resourceCoverageHirProjection,
    'FieldAccessorKind::from_core_field_member_name',
    'resource/coverage_hir_projection.rs must use shared core/field source member classification',
);
assertContains(
    resourceLowerAggregate,
    'FieldAccessorKind::from_core_field_member_name',
    'resource/lower_aggregate.rs must use shared core/field source member classification',
);
assertContains(
    typecheckPrefixCheck,
    'field_accessor.argument_count()',
    'typecheck/prefix_check.rs must validate field accessor arity through FieldAccessorKind',
);
assertContains(
    typecheckPrefixCheck,
    'ScalarIntrinsicKind::from_intrinsic_name',
    'typecheck/prefix_check.rs must use typed scalar intrinsic classification',
);
assertContains(
    typecheckPrefixCheck,
    'CoreIntrinsicKind::from_intrinsic_name',
    'typecheck/prefix_check.rs must use typed core intrinsic classification',
);
assertContains(
    typecheckPrefixCheck,
    'core_intrinsic_type_id',
    'typecheck/prefix_check.rs must derive core intrinsic result types from CoreIntrinsicKind',
);
assertContains(
    typecheckPrefixCheck,
    'scalar_intrinsic.output_type()',
    'typecheck/prefix_check.rs must derive scalar intrinsic result types from ScalarIntrinsicKind',
);
assertContains(
    typecheckPrefixCheck,
    'validate_scalar_intrinsic_args',
    'typecheck/prefix_check.rs must validate scalar intrinsic arguments through ScalarIntrinsicKind',
);
assertContains(
    wasmShared,
    'ScalarIntrinsicKind::from_intrinsic_name',
    'wasm_shared.rs must derive wasm scalar intrinsic support from ScalarIntrinsicKind',
);
assertContains(
    codegenPrecheck,
    'ScalarIntrinsicKind::from_intrinsic_name',
    'codegen_precheck.rs must derive llvm scalar intrinsic support from ScalarIntrinsicKind',
);
assertContains(
    codegenWasm,
    'ScalarIntrinsicKind::from_intrinsic_name',
    'codegen_wasm.rs must lower scalar intrinsics through ScalarIntrinsicKind',
);
assertContains(
    codegenWasm,
    'kind.backend_op()',
    'codegen_wasm.rs must consume ScalarIntrinsicKind backend_op instead of scalar spelling branches',
);
assertContains(
    codegenLlvm,
    'ScalarIntrinsicKind::from_intrinsic_name',
    'codegen_llvm.rs must route scalar intrinsics through ScalarIntrinsicKind',
);
assertContains(
    codegenLlvm,
    'scalar_intrinsic::lower_scalar_intrinsic',
    'codegen_llvm.rs must delegate scalar intrinsic lowering to the typed scalar intrinsic module',
);
assertContains(
    codegenLlvmScalarIntrinsic,
    'kind.backend_op()',
    'codegen_llvm/scalar_intrinsic.rs must consume ScalarIntrinsicKind backend_op',
);
for (const scalarIntrinsicName of [
    'i32_to_f32',
    'reinterpret_i32_f32',
    'i32_to_u8',
    'i32_to_u32',
    'i32_to_char',
    'char_to_i32',
    'f32_to_i32',
    'reinterpret_f32_i32',
    'u8_to_i32',
    'u32_to_i32',
    'i64_to_u64',
    'u64_to_i64',
    'str_addr',
    'str_from_addr_unchecked',
]) {
    assertNotContains(
        typecheckPrefixCheck,
        `intrin.name == "${scalarIntrinsicName}"`,
        `typecheck/prefix_check.rs must not duplicate ${scalarIntrinsicName} branch spelling outside ScalarIntrinsicKind`,
    );
    for (const [label, source] of [
        ['wasm_shared.rs', wasmShared],
        ['passes/codegen_precheck.rs', codegenPrecheck],
        ['codegen_wasm.rs', codegenWasm],
        ['codegen_llvm.rs', codegenLlvm],
        ['codegen_llvm/scalar_intrinsic.rs', codegenLlvmScalarIntrinsic],
    ]) {
        assertNotContains(
            source,
            `"${scalarIntrinsicName}"`,
            `${label} must not duplicate ${scalarIntrinsicName} spelling outside ScalarIntrinsicKind`,
        );
    }
}
assertContains(coreLib, 'mod backend_scalar_type;', 'lib.rs');
assertLineLimit(BACKEND_SCALAR_TYPE, 'backend_scalar_type.rs', 160);
assertContains(
    backendScalarType,
    'pub(crate) enum BackendScalarType',
    'backend_scalar_type.rs must represent backend named scalars as a typed enum domain',
);
assertContains(
    backendScalarType,
    'pub(crate) fn from_name',
    'backend_scalar_type.rs must classify backend scalar spelling once',
);
assertContains(
    backendScalarType,
    'pub(crate) fn from_type_kind',
    'backend_scalar_type.rs must classify TypeKind through the same scalar domain',
);
assertContains(
    backendScalarType,
    'pub(crate) fn from_type_expr',
    'backend_scalar_type.rs must classify TypeExpr through the same scalar domain',
);
assertContains(
    backendScalarType,
    'storage_size_bytes',
    'backend_scalar_type.rs must own backend scalar storage size semantics',
);
assertContains(
    backendScalarType,
    'storage_align_bytes',
    'backend_scalar_type.rs must own backend scalar storage alignment semantics',
);
assertContains(
    layout,
    'BackendScalarType::from_name(name.as_str())',
    'layout.rs must consume BackendScalarType for named scalar layout',
);
assertNotContains(
    layout,
    'name == "i64" || name == "u64" || name == "f64"',
    'layout.rs must not duplicate backend scalar spelling',
);
assertContains(
    wasmShared,
    'BackendScalarType::from_type_kind(kind)',
    'wasm_shared.rs must lower named scalar signatures through BackendScalarType',
);
assertContains(
    codegenWasm,
    'BackendScalarType::from_type_kind(kind)',
    'codegen_wasm.rs must lower named scalar locals through BackendScalarType',
);
assertContains(
    codegenLlvmTypeMap,
    'BackendScalarType::from_name(name.as_str())',
    'codegen_llvm/type_map.rs must lower named scalar TypeKind through BackendScalarType',
);
assertContains(
    codegenLlvm,
    'BackendScalarType::from_type_expr(ty)',
    'codegen_llvm.rs must lower named scalar TypeExpr through BackendScalarType',
);
assertContains(
    typecheckPrefixCheck,
    'BackendScalarType::I64.type_id(self.ctx)',
    'typecheck/prefix_check.rs must derive i64 scalar intrinsic ids through BackendScalarType',
);
assertContains(
    typecheckPrefixCheck,
    'BackendScalarType::U32.type_id(self.ctx)',
    'typecheck/prefix_check.rs must derive u32 scalar intrinsic ids through BackendScalarType',
);
assertContains(
    typecheckPrefixCheck,
    'BackendScalarType::U64.type_id(self.ctx)',
    'typecheck/prefix_check.rs must derive u64 scalar intrinsic ids through BackendScalarType',
);
assertNotContains(
    typecheckPrefixCheck,
    'lookup_named("i64")',
    'typecheck/prefix_check.rs must not manually look up named scalar ids',
);
assertNotContains(
    typecheckPrefixCheck,
    'register_named("u64"',
    'typecheck/prefix_check.rs must not manually register named scalar ids',
);
assertContains(
    typesRs,
    'BackendScalarType::from_name(name.as_str()).is_some()',
    'types.rs must derive named scalar Copy eligibility from BackendScalarType',
);
assertNotContains(
    typesRs,
    'matches!(name.as_str(), "i64" | "f64")',
    'types.rs must not duplicate backend scalar Copy eligibility strings',
);
for (const coreIntrinsicName of [
    'size_of',
    'align_of',
    'load',
    'store',
    'callsite_span',
    'unreachable',
]) {
    assertNotContains(
        typecheckPrefixCheck,
        `intrin.name == "${coreIntrinsicName}"`,
        `typecheck/prefix_check.rs must not duplicate ${coreIntrinsicName} branch spelling outside CoreIntrinsicKind`,
    );
}
assertNotContains(
    typecheckPrefixCheck,
    'intrin.name == "get_field"',
    'typecheck/prefix_check.rs must not duplicate get_field branch spelling outside FieldAccessorKind',
);
assertNotContains(
    typecheckPrefixCheck,
    'intrin.name == "get_field_ref"',
    'typecheck/prefix_check.rs must not duplicate get_field_ref branch spelling outside FieldAccessorKind',
);
assertNotContains(
    typecheckPrefixCheck,
    'intrin.name == "set_field"',
    'typecheck/prefix_check.rs must not duplicate set_field branch spelling outside FieldAccessorKind',
);
assertNotContains(
    resourceCoverageHirProjection,
    'helper_base_name(name) != "get_field"',
    'resource/coverage_hir_projection.rs must not duplicate get_field spelling outside FieldAccessorKind',
);
assertNotContains(
    resourceCoverageHirProjection,
    'helper_base_name(name) != "get_field_ref"',
    'resource/coverage_hir_projection.rs must not duplicate get_field_ref spelling outside FieldAccessorKind',
);
assertNotContains(
    resourceLowerAggregate,
    'helper_base_name(name) != "get_field"',
    'resource/lower_aggregate.rs must not duplicate get_field spelling outside FieldAccessorKind',
);
assertNotContains(
    resourceLowerAggregate,
    'helper_base_name(name) != "get_field_ref"',
    'resource/lower_aggregate.rs must not duplicate get_field_ref spelling outside FieldAccessorKind',
);
assertNotContains(
    resourceCoverageHirProjection,
    'name != "get"',
    'resource/coverage_hir_projection.rs must not duplicate core/field get spelling outside FieldAccessorKind',
);
assertNotContains(
    resourceLowerAggregate,
    'func_ref_base_name(callee)? != "get"',
    'resource/lower_aggregate.rs must not duplicate core/field get spelling outside FieldAccessorKind',
);
assertNotContains(
    resourceLowerAggregate,
    'func_ref_base_name(callee)? != "get_ref"',
    'resource/lower_aggregate.rs must not duplicate core/field get_ref spelling outside FieldAccessorKind',
);
const staleTypecheckVerboseNameFilters = [
    {
        source: typecheckCallReduction,
        forbidden: 'debug_name.as_deref()',
        label: 'typecheck/call_reduction.rs',
    },
    {
        source: typecheckPrefixCheck,
        forbidden: '"A" | "use_a" | "DefaultHash32" | "new" | "must_hm"',
        label: 'typecheck/prefix_check.rs',
    },
    {
        source: typecheckConstructorApply,
        forbidden: 'enm == "Result" && var == "Ok"',
        label: 'typecheck/constructor_apply.rs',
    },
    {
        source: typecheckDriver,
        forbidden: 'f.name.name == "new"',
        label: 'typecheck/driver.rs',
    },
    {
        source: typecheckFunctionApply,
        forbidden: 'name.contains("Result")',
        label: 'typecheck/function_apply.rs',
    },
    {
        source: typecheckFunctionCheck,
        forbidden: 'function.name.contains("partition")',
        label: 'typecheck/function_check.rs',
    },
];
for (const { source, forbidden, label } of staleTypecheckVerboseNameFilters) {
    assertNotContains(
        source,
        forbidden,
        `${label} must not filter typecheck verbose diagnostics by stale concrete symbol names`,
    );
}
for (const { source, label } of [
    { source: typecheckBindingRules, label: 'typecheck/binding_rules.rs' },
    { source: neplLanguageLib, label: 'nepl-language/src/lib.rs' },
    { source: neplWebLib, label: 'nepl-web/src/lib.rs' },
]) {
    assertNotContains(
        source,
        'is_important_shadow_symbol',
        `${label} must not classify shadow warnings through a hardcoded stdlib-name allowlist`,
    );
    assertNotContains(
        source,
        'important stdlib symbol',
        `${label} must not warn from important stdlib-name allowlists`,
    );
    assertNotMatches(
        source,
        /matches!\(\s*name,[\s\S]{0,1000}"print"/,
        `${label} must not keep print in a shadow-warning allowlist`,
    );
}
assertContains(
    typecheckBindingRules,
    'env.lookup_outer_defined(name)',
    'typecheck shadow warnings must be based on actual outer binding evidence',
);
assertContains(
    diagnosticCodes,
    'ShadowOuterDefinition',
    'diagnostic registry must expose source-derived outer shadow warning code',
);
assertNotContains(
    diagnosticCodes,
    'ShadowImportantSymbol',
    'diagnostic registry must not keep the old important-shadow diagnostic id',
);
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
    'compiler_memory_type_field_specs(memory_type)',
    'typecheck/compiler_memory_type.rs must consume shared compiler memory field shape spec',
);
assertContains(
    typecheckCompilerMemoryType,
    'source_map.compiler_memory_type_definition_allowed_at(def.name.span, memory_type)',
    'typecheck/compiler_memory_type.rs must require exact SourceMap source proof',
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
    /RestrictedStructConstructor::OwnerToken\s*=>\s*\{\s*self\.owner_aggregate_field_boundary_allowed\(span\)\s*\|\|\s*source_map\s*\.compiler_memory_type_definition_allowed_at\(\s*span,\s*CompilerMemoryType::OwnerToken,\s*\)/,
    'typecheck/field_access.rs owner token field projection must accept proven field-boundary source',
);
assertMatches(
    typecheckFieldAccess,
    /RestrictedStructConstructor::RawPointer[\s\S]*CompilerMemoryType::RawPointer/,
    'typecheck/field_access.rs raw pointer definition capability branch',
);
assertContains(
    typecheckFieldAccess,
    'compiler_memory_field_boundary_allowed_at(field, span)',
    'typecheck/field_access.rs raw pointer projection must require exact field-specific source proof',
);
assertContains(
    typecheckFieldAccess,
    'compiler_memory_field_for_restricted_access',
    'typecheck/field_access.rs raw pointer projection must derive compiler-memory field identity from typed selector proof',
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
assertContains(effects, 'pub enum RawBodyDirectCallee', 'effects.rs');
assertContains(effects, 'pub enum RawBodyBackend', 'effects.rs');
assertContains(effects, 'RawBodyDirectCallee::BackendIntrinsic', 'effects.rs');
assertContains(effects, 'pub enum WasmRawBodyMemoryOp', 'effects.rs');
assertContains(effects, 'pub enum LlvmRawBodyMemoryOp', 'effects.rs');
assertContains(effects, 'pub enum RawMemoryHelper', 'effects.rs');
assertContains(effects, 'impl RawMemoryHelper', 'effects.rs');
assertContains(effects, 'pub fn from_name(name: &str) -> Option<Self>', 'effects.rs');
assertContains(effects, 'RawMemoryHelper::from_name(name).map(RawMemoryHelper::operation)', 'effects.rs');
assertNotContains(effects, 'RAW_MEMORY_HELPER_EFFECT_MARKERS', 'effects.rs');
assertContains(effects, 'pub enum ExternalIoOp', 'effects.rs');
assertContains(effects, 'impl ExternalIoOp', 'effects.rs');
assertContains(effects, 'pub const ALL: &', 'effects.rs');
assertContains(effects, 'Self::FdRead', 'effects.rs');
assertContains(effects, 'pub enum NondetOp', 'effects.rs');
assertContains(effects, 'impl NondetOp', 'effects.rs');
assertContains(effects, 'Self::RandomGet', 'effects.rs');
assertNotContains(effects, 'IMPURE_IO_EFFECT_MARKERS', 'effects.rs');
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
assertContains(
    effects,
    'pub fn raw_body_direct_callee_effects(body: &HirBody) -> Vec<RawBodyDirectCallee>',
    'effects.rs',
);
assertNotContains(
    effects,
    'pub fn raw_body_direct_callees(body: &HirBody) -> Vec<String>',
    'effects.rs',
);
assertNotContains(effects, 'fn wasm_memory_operation(line: &str) -> Option<String>', 'effects.rs');
assertNotContains(effects, 'fn llvm_memory_operation(line: &str) -> Option<String>', 'effects.rs');
assertContains(
    typecheckEffectCheck,
    'raw_body_direct_callee_effects',
    'typecheck/effect_check.rs must consume typed raw body direct-callee evidence',
);
assertContains(
    typecheckEffectCheck,
    'RawBodyDirectCallee::RawMemory',
    'typecheck/effect_check.rs must match typed raw body raw-memory callees',
);
assertContains(
    typecheckEffectCheck,
    'RawBodyDirectCallee::BackendIntrinsic',
    'typecheck/effect_check.rs must match typed backend intrinsic callees',
);
assertNotContains(
    typecheckEffectCheck,
    'raw_body_direct_callees',
    'typecheck/effect_check.rs must not consume untyped raw body callee strings',
);
assertNotContains(
    typecheckEffectCheck,
    'raw_memory_op_from_name(&callee)',
    'typecheck/effect_check.rs must not reclassify raw body callees at the consumer',
);
assertNotContains(
    typecheckEffectCheck,
    'starts_with("llvm.")',
    'typecheck/effect_check.rs must not hard-code backend intrinsic purity with string prefixes',
);
assertContains(
    effects,
    'pub fn raw_memory_intrinsic_op_from_name',
    'effects.rs must classify raw-memory intrinsics as typed RawMemoryOp values',
);
assertNotContains(
    effects,
    'RAW_MEMORY_INTRINSIC_EFFECT_MARKERS',
    'effects.rs must not keep raw-memory intrinsic effect classification as a string marker list',
);
assertContains(
    typecheckEffectCheck,
    'raw_memory_intrinsic_op_from_name',
    'typecheck/effect_check.rs must consume typed raw-memory intrinsic operation evidence',
);
assertNotContains(
    typecheckEffectCheck,
    'intrinsic_is_raw_memory_effect',
    'typecheck/effect_check.rs must not combine marker-list intrinsic checks with operation reclassification',
);
assertLineLimit(SOURCE_CAPABILITY, 'source_capability.rs', 40);
assertContains(
    sourceCapability,
    'mod import_path;',
    'source_capability.rs must keep source capability import module classification in a separate module',
);
assertContains(
    sourceCapability,
    'mod raw_operation_proof;',
    'source_capability.rs must keep raw operation proof types in a separate module',
);
assertLineLimit(SOURCE_CAPABILITY_IMPORT_PATH, 'source_capability/import_path.rs', 100);
assertContains(
    sourceCapabilityImportPath,
    'pub(in crate::source_capability) enum SourceCapabilityImportModule',
    'source_capability/import_path.rs must represent proof-relevant import modules as an enum',
);
assertContains(
    sourceCapabilityImportPath,
    'pub(in crate::source_capability) fn from_path',
    'source_capability/import_path.rs must own import path classification',
);
assertContains(
    sourceCapabilityImportPath,
    'strip_supported_source_extension',
    'source_capability/import_path.rs must normalize supported source file extensions',
);
assertLineLimit(
    SOURCE_CAPABILITY_MEMORY_TYPE_DEFINITION,
    'source_capability/memory_type_definition.rs',
    100,
);
assertLineLimit(SOURCE_CAPABILITY_BINDING, 'source_capability/binding.rs', 60);
assertLineLimit(
    SOURCE_CAPABILITY_CONSTRUCTOR_POSITION,
    'source_capability/constructor_position.rs',
    80,
);
assertLineLimit(SOURCE_CAPABILITY_FIELD_SELECTOR, 'source_capability/field_selector.rs', 60);
assertLineLimit(SOURCE_CAPABILITY_PREFIX_CALL, 'source_capability/prefix_call.rs', 80);
assertLineLimit(
    SOURCE_CAPABILITY_RAW_EVIDENCE_GATE,
    'source_capability/raw_evidence_gate.rs',
    60,
);
assertLineLimit(
    SOURCE_CAPABILITY_RAW_OPERATION_PROOF,
    'source_capability/raw_operation_proof.rs',
    80,
);
assertLineLimit(SOURCE_CAPABILITY_RULE, 'source_capability/rule.rs', 240);
assertLineLimit(SOURCE_CAPABILITY_WALK, 'source_capability/walk.rs', 260);
assertLineLimit(SOURCE_CAPABILITY_PROOF, 'source_capability/proof.rs', 320);
assertLineLimit(SOURCE_CAPABILITY_PROOF_BUILDER, 'source_capability/proof_builder.rs', 120);
assertLineLimit(
    SOURCE_CAPABILITY_TOP_LEVEL_RAW_CALLS,
    'source_capability/top_level_raw_calls.rs',
    120,
);
assertLineLimit(RESOURCE_PRIMITIVES, 'resource_primitives.rs', 40);
assertLineLimit(
    RESOURCE_PRIMITIVES_COMPILER_MEMORY,
    'resource_primitives/compiler_memory.rs',
    130,
);
assertLineLimit(
    RESOURCE_PRIMITIVES_MEMORY_HELPER,
    'resource_primitives/memory_helper.rs',
    90,
);
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
assertNotContains(
    sourceMap,
    'pub enum SourceCapability {',
    'source_map.rs broad file-level SourceCapability enum',
);
assertNotContains(
    sourceMap,
    'capabilities: BTreeSet<SourceCapability>',
    'source_map.rs broad file-level SourceCapability storage',
);
assertNotContains(
    sourceMap,
    'pub fn with(capability: SourceCapability)',
    'source_map.rs broad SourceCapabilities constructor',
);
assertNotContains(
    sourceMap,
    'raw_memory_boundary()',
    'source_map.rs broad raw-memory-boundary constructor',
);
assertNotContains(
    sourceMap,
    'self.allows(SourceCapability::',
    'source_map.rs file-level SourceCapability query',
);
assertNotMatches(
    sourceMap,
    /pub fn capabilities\s*\(/,
    'source_map.rs must not expose the per-file SourceCapabilities accessor publicly',
);
assertContains(sourceMap, 'RawMemoryStructuralBoundary', 'source_map.rs');
assertContains(sourceMap, 'RawAddressViewBoundary', 'source_map.rs');
assertContains(sourceMap, 'RawMemoryOperationBoundary {', 'source_map.rs');
assertContains(sourceMap, 'RawBodyMemoryOperationBoundary {', 'source_map.rs');
assertContains(sourceMap, 'CompilerMemoryTypeDefinition {', 'source_map.rs');
assertContains(sourceMap, 'OwnerAggregateConstructorBoundary {', 'source_map.rs');
assertContains(sourceMap, 'OwnerAggregateFieldBoundary', 'source_map.rs');
assertContains(sourceMap, 'CompilerMemoryFieldBoundary', 'source_map.rs');
assertContains(sourceMap, 'pub enum CompilerMemoryField', 'source_map.rs');
assertContains(sourceMap, 'field: CompilerMemoryField', 'source_map.rs');
assertContains(sourceMap, 'pub enum SourceCapabilityUseSite', 'source_map.rs');
assertContains(sourceMap, 'pub struct SourceCapabilitySpan', 'source_map.rs');
assertContains(sourceMap, 'raw_memory_operation_boundary_allowed_at', 'source_map.rs');
assertContains(sourceMap, 'raw_body_memory_operation_boundary_allowed_at', 'source_map.rs');
assertContains(sourceMap, 'raw_address_view_boundary_allowed_at', 'source_map.rs');
assertContains(sourceMap, 'owner_aggregate_constructor_boundary_allowed_at', 'source_map.rs');
assertContains(sourceMap, 'compiler_memory_field_boundary_allowed_at', 'source_map.rs');
assertContains(sourceMap, 'compiler_memory_type_definition_allowed_at', 'source_map.rs');
assertNotContains(
    sourceMap,
    'allowed_within',
    'source_map.rs must not expose broad span-contained source proof queries',
);
for (const broadQuery of [
    'raw_memory_structural_boundary_allowed',
    'raw_address_view_boundary_allowed',
    'raw_memory_operation_boundary_allowed',
    'raw_body_memory_operation_boundary_allowed',
    'owner_aggregate_constructor_boundary_allowed',
    'owner_aggregate_field_boundary_allowed',
    'compiler_memory_field_boundary_allowed',
    'compiler_memory_type_definition_allowed',
]) {
    assertNotMatches(
        sourceMap,
        new RegExp(`pub fn ${broadQuery}\\s*\\(`),
        `source_map.rs must not expose file-level aggregate query ${broadQuery}`,
    );
}
for (const broadQuery of [
    'allows_raw_memory_structural_boundary',
    'allows_raw_address_view_boundary',
    'allows_raw_memory_operation_boundary',
    'allows_raw_body_memory_operation_boundary',
    'allows_owner_aggregate_constructor_boundary',
    'allows_owner_aggregate_field_boundary',
    'allows_compiler_memory_field_boundary',
    'allows_compiler_memory_type_definition',
]) {
    assertNotMatches(
        sourceMap,
        new RegExp(`pub fn ${broadQuery}\\s*\\(`),
        `SourceCapabilities must not expose file-level aggregate query ${broadQuery}`,
    );
}
assertContains(
    sourceMap,
    'fn source_capabilities_keep_source_proof_at_exact_use_site()',
    'source_map.rs exact source proof regression',
);
assertContains(
    sourceMap,
    'assert!(!source_map.raw_memory_operation_boundary_allowed_at(',
    'source_map.rs exact source proof regression must reject broad file capability at exact query',
);
assertNotMatches(
    sourceMap,
    /pub fn allows_[a-z_]+_at[\s\S]*?self\.allows\(SourceCapability::/,
    'source_map.rs exact use-site queries must not fall back to file-level SourceCapability',
);
assertContains(sourceMap, 'pub enum CompilerMemoryType', 'source_map.rs');
assertContains(sourceMap, 'RawPointer', 'source_map.rs');
assertContains(sourceMap, 'OwnerToken', 'source_map.rs');
assertContains(
    sourceMap,
    'compiler_memory_type_definition_allowed_at',
    'source_map.rs',
);
assertContains(
    sourceMap,
    'raw_address_view_boundary_allowed_at',
    'source_map.rs',
);
assertContains(
    sourceCapabilityRawMemoryEvidence,
    'enum RawMemoryStructuralEvidence',
    'source_capability/raw_memory/evidence.rs',
);
assertContains(
    sourceCapabilityRawMemoryEvidence,
    'enum RawAddressViewEvidence',
    'source_capability/raw_memory/evidence.rs',
);
assertContains(
    sourceCapability,
    'mod binding;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'mod proof;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'mod compiler_memory_field;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'mod constructor_position;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'mod field_selector;',
    'source_capability.rs',
);
assertContains(
    sourceCapabilityFieldSelector,
    'field_selector_after_call_head',
    'compiler memory field source proof must keep selector extraction in a bounded helper',
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
    'PrefixItem::Symbol(Symbol::Ident(ident, _type_args, _)) => Some(ident.name.as_str())',
    'source capability explicit constructor evidence must not require generic type arguments',
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
    'expr.items.get(index + 1).is_some()',
    'source capability proof traversal must observe nested payload-leading calls',
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
    sourceCapabilityProofBuilder,
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
    sourceCapabilityRule,
    'enum SourceCapabilityProofEvent',
    'source capability proof events must be a typed enum',
);
assertContains(
    sourceCapabilityRule,
    'pub(in crate::source_capability) fn dispatch_source_capability_proof_event',
    'source capability proof domains must be applied through one typed dispatcher',
);
assertContains(
    sourceCapabilityRule,
    'match event',
    'source capability proof dispatcher must use exhaustive event matching',
);
for (const eventVariant of [
    'SourceCapabilityProofEvent::Symbol',
    'SourceCapabilityProofEvent::ExplicitConstructor',
    'SourceCapabilityProofEvent::StructDefinition',
    'SourceCapabilityProofEvent::Intrinsic',
    'SourceCapabilityProofEvent::RawBody',
]) {
    assertContains(
        sourceCapabilityRule,
        eventVariant,
        'source capability proof dispatcher must cover every observer event variant',
    );
}
assertContains(
    sourceCapabilityProof,
    'dispatch_source_capability_proof_event',
    'source capability observer callbacks must route through the typed proof dispatcher',
);
assertNotContains(
    sourceCapabilityProof,
    'fn collect_raw_symbol_evidence',
    'source capability collector must not keep per-domain symbol dispatch methods',
);
assertContains(
    sourceCapability,
    'mod memory_type_definition;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'mod proof_builder;',
    'source_capability.rs',
);
assertContains(
    sourceCapability,
    'mod rule;',
    'source_capability.rs must route source proof events through a typed rule dispatcher',
);
assertContains(
    sourceCapability,
    'mod top_level_raw_calls;',
    'source_capability.rs',
);
assertNotContains(
    sourceCapability,
    'compiler_memory_type_from_constructor_name',
    'source_capability.rs must not re-export compiler memory primitive classifiers',
);
assertContains(
    sourceCapabilityRule,
    'compiler_memory_type_from_struct_def',
    'source_capability/rule.rs',
);
assertContains(
    resourcePrimitives,
    'mod compiler_memory;',
    'resource_primitives.rs must split compiler memory primitive contracts into a dedicated module',
);
assertContains(
    resourcePrimitives,
    'mod memory_helper;',
    'resource_primitives.rs must split memory helper primitive contracts into a dedicated module',
);
assertContains(
    resourcePrimitivesCompilerMemory,
    'pub(crate) fn compiler_memory_type_from_constructor_name',
    'resource_primitives/compiler_memory.rs',
);
assertContains(
    resourcePrimitivesCompilerMemory,
    'pub(crate) enum CompilerMemoryFieldSpec',
    'resource_primitives/compiler_memory.rs must keep compiler memory field shape in a typed enum domain',
);
assertContains(
    resourcePrimitivesCompilerMemory,
    'pub(crate) fn compiler_memory_type_field_specs',
    'resource_primitives/compiler_memory.rs must own compiler memory type field shape contracts',
);
assertContains(
    resourcePrimitivesCompilerMemory,
    'match name',
    'resource_primitives/compiler_memory.rs',
);
assertContains(
    resourcePrimitivesCompilerMemory,
    'pub(crate) fn compiler_memory_type_of_type',
    'resource_primitives/compiler_memory.rs',
);
assertContains(
    resourcePrimitivesCompilerMemory,
    'pub(crate) fn type_is_raw_pointer',
    'resource_primitives/compiler_memory.rs',
);
assertContains(
    resourcePrimitivesCompilerMemory,
    'pub(crate) fn type_is_owner_token',
    'resource_primitives/compiler_memory.rs',
);
assertContains(
    resourcePrimitivesMemoryHelper,
    'pub(crate) enum MemoryHelperPrimitive',
    'resource_primitives/memory_helper.rs',
);
assertContains(
    resourcePrimitivesMemoryHelper,
    'is_raw_address_view_boundary_evidence',
    'memory helper primitive roles must distinguish raw address view evidence from owner-token construction',
);
assertContains(
    resourcePrimitivesMemoryHelper,
    'has_resource_call_lowering',
    'memory helper primitive roles must expose dedicated Resource IR call-lowering authority',
);
assertContains(
    sourceCapabilityMemoryTypeDefinition,
    'compiler_memory_type_from_constructor_name(def.name.name.as_str())',
    'source_capability/memory_type_definition.rs must use central compiler memory type classification',
);
assertContains(
    sourceCapabilityMemoryTypeDefinition,
    'compiler_memory_type_field_specs(memory_type)',
    'source_capability/memory_type_definition.rs must consume shared compiler memory field shape spec',
);
assertContains(
    sourceCapabilityMemoryTypeDefinition,
    'pub(in crate::source_capability) fn compiler_memory_type_from_struct_def',
    'source_capability/memory_type_definition.rs',
);
assertNotContains(
    sourceCapabilityMemoryTypeDefinition,
    'is_mem_ptr_definition',
    'source_capability/memory_type_definition.rs must not keep per-memory-type shape checkers',
);
assertNotContains(
    sourceCapabilityMemoryTypeDefinition,
    'is_region_token_definition',
    'source_capability/memory_type_definition.rs must not keep per-memory-type shape checkers',
);
assertNotContains(
    sourceCapabilityMemoryTypeDefinition,
    '.name == "raw"',
    'source_capability/memory_type_definition.rs must not duplicate raw field spelling outside CompilerMemoryFieldSpec',
);
assertNotContains(
    sourceCapabilityMemoryTypeDefinition,
    '.name == "size"',
    'source_capability/memory_type_definition.rs must not duplicate size field spelling outside CompilerMemoryFieldSpec',
);
assertNotContains(
    typecheckCompilerMemoryType,
    'raw == "raw"',
    'typecheck/compiler_memory_type.rs must not duplicate raw field spelling outside CompilerMemoryFieldSpec',
);
assertNotContains(
    typecheckCompilerMemoryType,
    'size == "size"',
    'typecheck/compiler_memory_type.rs must not duplicate size field spelling outside CompilerMemoryFieldSpec',
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
    'mod fact;',
    'source_capability.rs must keep evidence-to-fact conversion in a separate module',
);
assertLineLimit(SOURCE_CAPABILITY_FACT, 'source_capability/fact.rs', 80);
assertContains(
    sourceCapabilityRule,
    'owner_aggregate_symbol_evidence',
    'source_capability/rule.rs',
);
assertContains(
    sourceCapabilityProofBuilder,
    'pub(in crate::source_capability) enum SourceCapabilityProofFact',
    'source_capability/proof_builder.rs must represent source proof output as typed facts',
);
assertContains(
    sourceCapabilityProofBuilder,
    'pub(in crate::source_capability) fn insert_fact',
    'source_capability/proof_builder.rs must route proof facts through one insert path',
);
assertContains(
    sourceCapabilityProofBuilder,
    'match fact',
    'source_capability/proof_builder.rs must map proof facts through an exhaustive match',
);
for (const oldProofInsert of [
    'fn insert_raw_memory_structural_boundary',
    'fn insert_raw_address_view_boundary',
    'fn insert_raw_memory_operation_boundary',
    'fn insert_raw_body_memory_operation_boundary',
    'fn insert_owner_aggregate_evidence',
    'fn insert_compiler_memory_field_evidence',
    'fn insert_compiler_memory_type_definition',
]) {
    assertNotContains(
        sourceCapabilityProofBuilder,
        oldProofInsert,
        'source_capability/proof_builder.rs must not expose per-domain proof insert APIs',
    );
}
assertContains(
    sourceCapabilityFact,
    'SourceCapabilityProofFact::OwnerAggregateConstructorBoundary',
    'source_capability/fact.rs must store owner aggregate evidence as exact typed proof facts',
);
assertContains(
    sourceCapabilityFact,
    'owner_aggregate_proof_fact',
    'source_capability/fact.rs must map owner aggregate evidence to proof facts',
);
assertContains(
    sourceCapabilityFact,
    'compiler_memory_field_proof_fact',
    'source_capability/fact.rs must map compiler memory field evidence to proof facts',
);
assertContains(
    sourceCapabilityOwnerAggregateEvidence,
    'enum OwnerAggregateCapabilityEvidence',
    'source_capability/owner_aggregate/evidence.rs',
);
assertContains(
    sourceCapabilityOwnerAggregateEvidence,
    'FieldAccessorKind::from_intrinsic_name',
    'owner aggregate intrinsic field evidence must use shared FieldAccessorKind classification',
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
    sourceCapabilityProofBuilder,
    'SourceCapabilityUseSite::OwnerAggregateConstructorBoundary',
    'owner aggregate constructor evidence must be tracked by constructor name and exact span',
);
assertContains(
    sourceCapabilityProofBuilder,
    'SourceCapabilityUseSite::CompilerMemoryFieldBoundary',
    'compiler memory field projection must use exact field-access source proof',
);
assertContains(
    sourceCapabilityProofBuilder,
    'SourceCapabilityProofFact::CompilerMemoryFieldBoundary',
    'compiler memory field projection must not be inserted through owner aggregate evidence',
);
const ownerAggregateFieldEvidenceArm = sourceCapabilityFact.match(
    /Some\(OwnerAggregateCapabilityEvidence::FieldAccessor\) => \{[\s\S]*?SourceCapabilityProofFact::OwnerAggregateFieldBoundary[\s\S]*?\n        \}/,
);
assert(ownerAggregateFieldEvidenceArm, 'source_capability/fact.rs owner field fact arm');
assertNotContains(
    ownerAggregateFieldEvidenceArm[0],
    'CompilerMemoryFieldBoundary',
    'owner aggregate field evidence must not grant compiler memory field proof',
);
assertContains(
    sourceCapabilityCompilerMemoryField,
    'enum CompilerMemoryFieldEvidence',
    'compiler memory field evidence must be a typed source proof domain',
);
assertContains(
    sourceCapabilityCompilerMemoryField,
    'FieldAccessorKind::Get | FieldAccessorKind::GetRef',
    'compiler memory field evidence must use exhaustive field accessor classification',
);
assertContains(
    sourceCapabilityCompilerMemoryField,
    'CompilerMemoryField::from_name',
    'compiler memory field evidence must classify representation fields through typed field domain',
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
    'SourceCapabilityImportModule::from_path(path)',
    'owner aggregate field import evidence must consume typed source capability import module classification',
);
assertNotContains(
    sourceCapabilityOwnerAggregateFieldImports,
    'strip_suffix(".nepl").unwrap_or(path) == "core/field"',
    'owner aggregate field import evidence must not classify core/field imports with local path string checks',
);
assertContains(
    sourceCapabilityOwnerAggregateFieldImports,
    'field_aliases',
    'owner aggregate field evidence must track core/field import aliases',
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
assertContains(
    sourceCapabilityOwnerAggregateFieldImports,
    'FieldAccessorKind::from_core_field_member_name',
    'owner aggregate field import evidence must use shared core/field source member classification',
);
assertNotContains(
    sourceCapabilityOwnerAggregateFieldImports,
    'enum CoreFieldAccessorMember',
    'owner aggregate field import evidence must not keep a local field accessor enum',
);
assertNotContains(
    sourceCapabilityOwnerAggregateFieldImports,
    '"get" =>',
    'owner aggregate field import evidence must not duplicate core/field get spelling outside FieldAccessorKind',
);
assertNotContains(
    sourceCapabilityOwnerAggregateFieldImports,
    '"get_ref" =>',
    'owner aggregate field import evidence must not duplicate core/field get_ref spelling outside FieldAccessorKind',
);
assertNotContains(
    sourceCapabilityOwnerAggregateFieldImports,
    '"put" =>',
    'owner aggregate field import evidence must not duplicate core/field put spelling outside FieldAccessorKind',
);
assertNotContains(
    sourceCapabilityOwnerAggregateEvidence,
    '"get" | "get_ref" | "put" | "get_field" | "get_field_ref"',
    'owner aggregate field evidence must not accept broad helper names without import proof',
);
assertNotContains(
    sourceCapabilityOwnerAggregateEvidence,
    '"get_field" | "get_field_ref"',
    'owner aggregate intrinsic field evidence must not duplicate field accessor spelling outside FieldAccessorKind',
);
assertNotContains(
    sourceCapabilityOwnerAggregateEvidence,
    '"get_field"',
    'owner aggregate intrinsic field evidence must not duplicate get_field spelling outside FieldAccessorKind',
);
assertNotContains(
    sourceCapabilityOwnerAggregateEvidence,
    '"get_field_ref"',
    'owner aggregate intrinsic field evidence must not duplicate get_field_ref spelling outside FieldAccessorKind',
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
    'fn owner_aggregate_boundary_rejects_set_field_intrinsic_evidence()',
    'loader.rs owner aggregate write intrinsic evidence rejection regression',
);
assertContains(
    loader,
    'fn compiler_memory_field_boundary_requires_representation_field_selector()',
    'loader.rs compiler memory field selector regression',
);
assertContains(
    resourceIrTests,
    'typecheck_rejects_compiler_owned_aggregate_mem_ptr_payload_field_access',
    'resource_ir.rs compiler-owned aggregate MemPtr payload field access regression',
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
    sourceCapabilityBinding,
    'enum SourceCapabilityBindingKind',
    'source capability scope must distinguish top-level callable and local shadow kinds',
);
assertContains(
    sourceCapabilityBinding,
    'TopLevelCallable',
    'source capability scope must represent top-level callable shadows separately',
);
assertContains(
    sourceCapabilityBinding,
    'ImplMethod',
    'source capability scope must represent impl method names without granting raw helper proof',
);
assertContains(
    sourceCapabilityBinding,
    'LocalValue',
    'source capability scope must represent local/parameter shadows separately',
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
    'shadow_kind_symbol_or_qualifier',
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
    sourceCapabilityRule,
    'raw_memory_op_from_name',
    'source_capability/rule.rs',
);
assertContains(
    sourceCapabilityRule,
    'raw_body_direct_callee_effects',
    'source_capability/rule.rs must consume typed raw body direct-callee evidence',
);
assertContains(
    sourceCapabilityRule,
    'RawBodyDirectCallee::RawMemory',
    'source_capability/rule.rs must match typed raw body raw-memory callees',
);
assertNotContains(
    sourceCapabilityRule,
    'raw_body_direct_callees',
    'source_capability/rule.rs must not consume untyped raw body callee strings',
);
assertNotContains(
    sourceCapabilityRule,
    'raw_memory_op_from_name(&callee)',
    'source_capability/rule.rs must not reclassify raw body callees at the consumer',
);
assertContains(
    sourceCapabilityProof,
    'raw_operation_function_frames',
    'raw helper definitions must grant their operation only when the body has raw operation evidence',
);
assertContains(
    sourceCapabilityProof,
    'raw_operation_function_frames.last_mut()',
    'raw helper definition operation evidence must be scoped to the currently walked function frame',
);
assertContains(
    sourceCapabilityProof,
    'record_top_level_raw_call_evidence',
    'raw helper calls through top-level functions must be recorded for proof-based propagation',
);
assertContains(
    sourceCapabilityTopLevelRawCalls,
    'apply_top_level_raw_call_evidence',
    'top-level raw helper calls must be resolved by a shared proof propagation pass',
);
assertContains(
    sourceCapabilityTopLevelRawCalls,
    'has_direct_raw_evidence',
    'top-level raw helper propagation must start only from functions with source evidence',
);
assertContains(
    sourceCapabilityRawOperationProof,
    'RawOperationBoundaryContract',
    'top-level raw helper propagation must use an explicit typed boundary contract',
);
assertContains(
    sourceCapabilityRawOperationProof,
    'RawOperationFunctionEvidence',
    'top-level raw helper propagation must carry typed source-derived function evidence',
);
assertContains(
    sourceCapabilityTopLevelRawCalls,
    'proven_functions',
    'top-level raw helper propagation must use proven helper summaries instead of file-level authority',
);
assertNotContains(
    sourceCapabilityTopLevelRawCalls,
    'raw_memory_op_from_name',
    'top-level raw helper propagation must not re-query helper spellings inside the proof worklist',
);
assertContains(
    sourceCapabilityProof,
    'raw_memory_boundary_contract_from_function_name(name)',
    'source_capability/proof.rs must classify raw helper boundary contracts before worklist propagation',
);
assertContains(
    sourceCapabilityRule,
    'scope.shadow_kind_symbol_or_qualifier(symbol)',
    'raw memory source evidence must reject shadowed qualified helper-looking symbols',
);
assertContains(
    sourceCapabilityRule,
    'raw_symbol_shadow_allows_evidence',
    'raw helper wrappers must prove same-name raw primitive evidence without allowing local shadowing',
);
assertContains(
    sourceCapabilityRawEvidenceGate,
    'kind == SourceCapabilityBindingKind::TopLevelCallable',
    'raw helper shadow evidence gate must only allow top-level callable shadows',
);
assertContains(
    sourceCapabilityRawEvidenceGate,
    'current_function.is_some_and(|name| name == symbol)',
    'raw helper shadow evidence gate must require the current same-name helper body',
);
assertContains(
    sourceCapabilityRawEvidenceGate,
    'raw_memory_op_from_name(symbol).is_some()',
    'raw helper shadow evidence gate must require a known raw operation symbol',
);
assertNotContains(
    sourceCapabilityProof,
    'for frame in &mut self.raw_operation_function_frames',
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
assertContains(
    sourceCapabilityRawMemoryEvidence,
    'MemoryHelperPrimitive::is_raw_address_view_boundary_evidence',
    'raw address helper evidence must use the central helper role classifier',
);
assertContains(
    sourceCapabilityRawMemoryEvidence,
    'compiler_memory_type_from_constructor_name(name)',
    'raw structural evidence must use compiler memory constructor registry only for restricted constructors',
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
    resourceLower,
    'let lowered_core_mem_wrapper',
    'resource/lower.rs must remember when dedicated memory helper lowering handled a call',
);
assertContains(
    resourceLower,
    'if !lowered_core_mem_wrapper',
    'resource/lower.rs must not also run generic raw-address named proof for dedicated memory helper calls',
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
    'fn raw_memory_boundary_does_not_promote_address_view_helper_to_operation_definition()',
    'loader.rs raw address view must not become raw operation evidence',
);
assertContains(
    loader,
    'fn raw_memory_boundary_rejects_owner_constructor_helper_as_address_view_evidence()',
    'loader.rs owner-token helper calls must not prove raw address view boundary',
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
    'fn raw_memory_boundary_accepts_same_name_raw_helper_wrapper_evidence()',
    'loader.rs same-name raw helper wrapper regression',
);
assertContains(
    loader,
    'fn raw_memory_boundary_accepts_raw_helper_call_in_constructor_payload()',
    'loader.rs raw helper constructor-payload regression',
);
assertContains(
    loader,
    'allows_raw_memory_operation_boundary_at(RawMemoryOp::Load, call_span)',
    'loader.rs raw helper constructor-payload regression must assert exact use-site proof',
);
assertContains(
    loader,
    'fn raw_memory_boundary_rejects_local_shadow_inside_same_name_raw_helper()',
    'loader.rs same-name raw helper local shadow regression',
);
assertContains(
    loader,
    'fn raw_memory_boundary_requires_raw_structural_call_head()',
    'loader.rs raw structural call-head regression',
);
assertNotContains(
    testHarness,
    'pub fn compile_src_with_options_and_entry_capabilities',
    'nepl-core/tests/harness.rs broad entry capability injection helper',
);
assertContains(
    testHarness,
    'pub fn run_main_wasi_i32_raw_memory_boundary',
    'nepl-core/tests/harness.rs',
);
assertContains(
    testHarness,
    'compile_src_with_options_at_path',
    'nepl-core/tests/harness.rs raw fixture must use a compiler-owned source path',
);
assertNotContains(
    testHarness,
    'SourceCapabilities::raw_memory_boundary()',
    'nepl-core/tests/harness.rs broad raw-memory test capability',
);
assertMatches(
    testHarness,
    /pub fn run_main_wasi_i32\(src: &str\) -> i32 \{[\s\S]*?compile_src_with_options\([\s\S]*?run_wasi_wasm_i32\(&wasm\)\s*\}/,
    'ordinary WASI test harness must not grant raw-memory-boundary capability',
);
assertMatches(
    testHarness,
    /pub fn run_main_wasi_i32_raw_memory_boundary\(src: &str\) -> i32 \{[\s\S]*?stdlib_root\(\)\.join\("__raw_boundary_test\.nepl"\)[\s\S]*?compile_src_with_options_at_path\([\s\S]*?run_wasi_wasm_i32\(&wasm\)\s*\}/,
    'raw-memory fixture harness must obtain source proof through a compiler-owned virtual source path',
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
    /ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction\s*\{\s*operation,\s*\.\.\s*\}\s*=>\s*\{\s*let Some\(span\) = resource_effect_boundary_diagnostic_span\(diagnostic\) else \{\s*return false;\s*\};\s*source_map\s*\.map\(\|map\| map\.raw_memory_operation_boundary_allowed_at\(span, \*operation\)\)\s*\.unwrap_or\(false\)\s*\}/,
    'compiler.rs unsafe memory suppression must require the exact raw-memory operation capability',
);
assertMatches(
    compiler,
    /ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary\s*\{\s*operation,\s*\.\.\s*\}\s*=>\s*\{\s*let Some\(span\) = resource_effect_boundary_diagnostic_span\(diagnostic\) else \{\s*return false;\s*\};\s*source_map\s*\.map\(\|map\| map\.raw_memory_operation_boundary_allowed_at\(span, \*operation\)\)\s*\.unwrap_or\(false\)\s*\}/,
    'compiler.rs raw memory outside-boundary suppression must require the exact raw-memory operation capability',
);
assertMatches(
    compiler,
    /ResourceEffectBoundaryDiagnostic::RawAddressViewOutsideBoundary\s*\{\s*\.\.\s*\}\s*=>\s*\{\s*let Some\(span\) = resource_effect_boundary_diagnostic_span\(diagnostic\) else \{\s*return false;\s*\};\s*source_map\s*\.map\(\|map\| map\.raw_address_view_boundary_allowed_at\(span\)\)\s*\.unwrap_or\(false\)\s*\}/,
    'compiler.rs raw address view suppression must require the raw-address-view capability',
);
assertMatches(
    compiler,
    /ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc\s*\{\s*operation,\s*origin_span,\s*\.\.\s*\}\s*=>\s*source_map\s*\.map\(\|map\| raw_identity_escape_allowed\(\*operation, \*origin_span, map\)\)\s*\.unwrap_or\(false\)/,
    'compiler.rs raw identity escape must require exact origin-span source capability',
);
assertContains(
    compiler,
    'fn raw_identity_escape_allowed(',
    'compiler.rs raw identity escape helper',
);
assertContains(
    compiler,
    'RawMemoryOp::Alloc =>',
    'compiler.rs raw identity escape operation-specific suppression',
);
assertContains(
    compiler,
    'RawMemoryOp::Realloc =>',
    'compiler.rs raw identity escape operation-specific suppression',
);
assertContains(
    compiler,
    'raw_memory_operation_boundary_allowed_at(origin_span, RawMemoryOp::Realloc)',
    'compiler.rs raw identity escape must require exact realloc origin proof',
);
assertNotContains(
    compiler,
    'raw_memory_operation_boundary_allowed_within',
    'compiler.rs raw identity escape must not use broad return-span proof search',
);
assertContains(
    compiler,
    'fn resource_effect_gate_requires_raw_identity_origin_span_capability()',
    'compiler.rs raw-boundary origin-span unit regression',
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
