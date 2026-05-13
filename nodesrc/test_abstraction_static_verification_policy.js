#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const ROOT = path.resolve(__dirname, "..");
const TYPECHECK_DIR = path.join(ROOT, "nepl-core", "src", "typecheck");
const FUNCTION_CHECK = path.join(TYPECHECK_DIR, "function_check.rs");
const SELECTED_CALL_APPLY = path.join(TYPECHECK_DIR, "selected_call_apply.rs");
const HIR = path.join(ROOT, "nepl-core", "src", "hir.rs");
const DROP_INSERTION = path.join(ROOT, "nepl-core", "src", "passes", "drop_insertion.rs");
const MONOMORPHIZE = path.join(ROOT, "nepl-core", "src", "monomorphize.rs");
const MONOMORPHIZE_TRAIT_LOOKUP = path.join(
    ROOT,
    "nepl-core",
    "src",
    "monomorphize",
    "trait_lookup.rs",
);
const MONOMORPHIZE_TRAIT_IDENTITY = path.join(
    ROOT,
    "nepl-core",
    "src",
    "monomorphize",
    "trait_identity.rs",
);
const RESOURCE_MODEL = path.join(ROOT, "nepl-core", "src", "resource", "model.rs");
const RESOURCE_TRAIT_IDENTITY = path.join(ROOT, "nepl-core", "src", "resource", "trait_identity.rs");
const TRAITS = path.join(TYPECHECK_DIR, "traits.rs");
const PLAN = path.join(ROOT, "doc", "neplg2", "abstraction_static_verification_plan.md");
const RUNNER = path.join(ROOT, "nodesrc", "run_source_policy_regressions.js");

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function read(filePath) {
    return fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
}

function walkRustFiles(dir) {
    const files = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const child = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...walkRustFiles(child));
        } else if (entry.isFile() && entry.name.endsWith(".rs")) {
            files.push(child);
        }
    }
    return files;
}

function countOccurrences(text, needle) {
    return text.split(needle).length - 1;
}

function countAll(files, needle) {
    return files.reduce((sum, filePath) => sum + countOccurrences(read(filePath), needle), 0);
}

const plan = read(PLAN);
for (const marker of [
    "ISS-20260512T143721313Z-GENERIC-AND-TRAIT-ABSTRACTION-MODEL--1F2FF429",
    "TraitApplication",
    "ImplKind",
    "PendingTraitCheck",
    "MonoTraitLookupKey",
    "source policy",
]) {
    assert(plan.includes(marker), `abstraction static verification plan must mention ${marker}`);
}

const runner = read(RUNNER);
assert(
    runner.includes("nodesrc/test_abstraction_static_verification_policy.js"),
    "source policy runner must include abstraction static verification policy",
);

const traits = read(TRAITS);
const implInfoStruct = traits.match(/pub\(super\) struct ImplInfo\s*\{[\s\S]*?\n\}/);
const traitSemanticsStruct = traits.match(/pub\(super\) struct TraitSemantics\s*\{[\s\S]*?\n\}/);
const files = walkRustFiles(TYPECHECK_DIR).concat([MONOMORPHIZE]);
const counts = {
    parseTraitRefName: countAll(files, "parse_trait_ref_name"),
    formatTraitRefName: countAll(files, "format_trait_ref_name"),
    traitBoundRef: countAll(files, "TraitBoundRef"),
    implInfo: countAll(files, "ImplInfo"),
    traitLookupCache: countAll(files, "trait_lookup_cache"),
    implInfoOptionalFields: implInfoStruct ? countOccurrences(implInfoStruct[0], "Option<") : 0,
};

assert(counts.parseTraitRefName === 0, "trait ref string parser must not be reintroduced");
assert(counts.traitBoundRef === 0, "TraitBoundRef old model must not be reintroduced");
assert(
    counts.implInfoOptionalFields === 0,
    "ImplInfo optional field model must not be reintroduced",
);
const formatTraitRefAuthorityFiles = files.filter(
    (filePath) => filePath !== TRAITS && read(filePath).includes("format_trait_ref_name"),
);
assert(
    formatTraitRefAuthorityFiles.length === 0,
    `format_trait_ref_name must stay in traits.rs display boundary: ${formatTraitRefAuthorityFiles.join(", ")}`,
);

assert(traits.includes("pub(super) enum TraitCapability"), "TraitCapability must remain an enum");
assert(traits.includes("TraitCapability::Copy"), "TraitCapability::Copy match coverage must remain visible");
assert(traits.includes("TraitCapability::Clone"), "TraitCapability::Clone match coverage must remain visible");
assert(traits.includes("TraitCapability::Drop"), "TraitCapability::Drop match coverage must remain visible");
assert(traitSemanticsStruct, "TraitSemantics struct body must be visible to source policy");
for (const field of ["copy_traits", "clone_traits", "drop_traits"]) {
    assert(
        traitSemanticsStruct[0].includes(`${field}: Vec<TypeId>`),
        `TraitSemantics ${field} must store typed trait identity only`,
    );
}
assert(
    !traitSemanticsStruct[0].includes("String"),
    "TraitSemantics must not retain rendered trait names as capability authority",
);
assert(
    !traits.includes("Vec<(String, TypeId)>"),
    "TraitSemantics capability sets must not be encoded as raw name plus TypeId pairs",
);
assert(
    traits.includes("fn insert_trait(&mut self, capability: TraitCapability, trait_self_ty: TypeId)"),
    "TraitSemantics capability insertion must branch on TraitCapability enum",
);
assert(traits.includes("pub(super) struct TraitApplication"), "TraitApplication must be a typed model");
assert(traits.includes("pub(super) struct TraitId"), "TraitApplication must use a typed TraitId");
const traitApplicationStruct = traits.match(/pub\(super\) struct TraitApplication\s*\{[\s\S]*?\n\}/);
assert(traitApplicationStruct, "TraitApplication struct body must be visible to source policy");
assert(
    traitApplicationStruct[0].includes("trait_id: TraitId"),
    "TraitApplication must store trait identity as TraitId",
);
assert(
    !traitApplicationStruct[0].includes("base_name: String"),
    "TraitApplication must not store trait identity as raw String",
);
assert(traits.includes("pub(super) struct TypeParamId"), "type parameter declaration identity must use TypeParamId");
assert(traits.includes("pub(super) struct BoundEnv"), "type parameter bounds must use BoundEnv");
assert(
    traits.includes("bounds: BTreeMap<TypeParamId, Vec<TraitBound>>"),
    "BoundEnv must key bounds by TypeParamId",
);
assert(
    !traits.includes("bounds: BTreeMap<TypeId, Vec<TraitBound>>"),
    "BoundEnv must not key bounds by raw TypeId",
);
assert(
    !traits.includes("fn insert(&mut self, type_param: TypeId"),
    "BoundEnv insertion must require TypeParamId",
);
assert(
    traits.includes("fn has_trait_application_bound("),
    "BoundEnv must own trait application bound lookup",
);
assert(traits.includes("pub(super) struct PendingTraitCheck"), "pending trait checks must be named state");
const pendingTraitCheckStruct = traits.match(/pub\(super\) struct PendingTraitCheck\s*\{[\s\S]*?\n\}/);
assert(pendingTraitCheckStruct, "PendingTraitCheck struct body must be visible to source policy");
assert(pendingTraitCheckStruct[0].includes("bound: TraitBound"), "PendingTraitCheck must name its bound");
assert(pendingTraitCheckStruct[0].includes("target_ty: TypeId"), "PendingTraitCheck must name its target type");
assert(pendingTraitCheckStruct[0].includes("span: Span"), "PendingTraitCheck must name its diagnostic span");
assert(traits.includes("pub(super) enum ImplKind"), "ImplKind must model impl identity as an enum");
assert(traits.includes("ImplKind::Inherent"), "ImplKind::Inherent match branch must remain visible");
assert(traits.includes("ImplKind::Trait"), "ImplKind::Trait match branch must remain visible");
assert(
    traits.includes("application: TraitApplication"),
    "trait impl identity must store a typed TraitApplication",
);
assert(
    traits.includes("self_ty: TypeId"),
    "trait impl identity must store trait self TypeId explicitly",
);
assert(implInfoStruct, "ImplInfo struct body must be visible to source policy");
assert(implInfoStruct[0].includes("kind: ImplKind"), "ImplInfo must own ImplKind");
assert(!implInfoStruct[0].includes("trait_name"), "ImplInfo must not store rendered trait names");
assert(!implInfoStruct[0].includes("trait_base_name"), "ImplInfo must not store split trait base names");
assert(!implInfoStruct[0].includes("trait_args"), "ImplInfo must not store split trait args");
assert(!implInfoStruct[0].includes("Option<"), "ImplInfo must not encode kind through optional fields");
assert(traits.includes("pub(super) struct TraitBound"), "typed trait bound model must be named TraitBound");
assert(
    !traits.includes("pub(super) struct TraitBoundRef"),
    "old TraitBoundRef model must not be reintroduced",
);
const traitBoundStruct = traits.match(/pub\(super\) struct TraitBound\s*\{[\s\S]*?\n\}/);
assert(traitBoundStruct, "TraitBound struct body must be visible to source policy");
assert(
    traitBoundStruct[0].includes("application: TraitApplication"),
    "TraitBound must own a typed TraitApplication",
);
assert(
    !/\n\s*pub\(super\) name:\s*String/.test(traitBoundStruct[0]),
    "TraitBound must not store rendered diagnostic names",
);
assert(
    !traitBoundStruct[0].includes("trait_base_name"),
    "TraitBound must not split trait application base name out of TraitApplication",
);
assert(
    !traitBoundStruct[0].includes("trait_args"),
    "TraitBound must not split trait application args out of TraitApplication",
);

const functionCheck = read(FUNCTION_CHECK);
const context = read(path.join(TYPECHECK_DIR, "context.rs"));
const traitBoundApply = read(path.join(TYPECHECK_DIR, "trait_bound_apply.rs"));
const blockCheck = read(path.join(TYPECHECK_DIR, "block_check.rs"));
const driver = read(path.join(TYPECHECK_DIR, "driver.rs"));
const env = read(path.join(TYPECHECK_DIR, "env.rs"));
const selectedCallApply = read(SELECTED_CALL_APPLY);
const hir = read(HIR);
const dropInsertion = read(DROP_INSERTION);
const monomorphize = read(MONOMORPHIZE);
const monomorphizeTraitLookup = read(MONOMORPHIZE_TRAIT_LOOKUP);
const monomorphizeTraitIdentity = read(MONOMORPHIZE_TRAIT_IDENTITY);
const resourceModel = read(RESOURCE_MODEL);
const resourceTraitIdentity = read(RESOURCE_TRAIT_IDENTITY);
for (const [name, text] of [
    ["traits.rs", traits],
    ["context.rs", context],
    ["env.rs", env],
    ["function_check.rs", functionCheck],
    ["trait_bound_apply.rs", traitBoundApply],
    ["selected_call_apply.rs", selectedCallApply],
]) {
    assert(
        !text.includes("BTreeMap<TypeId, Vec<TraitBound>>"),
        `${name} must not expose raw type parameter bound maps`,
    );
}
assert(
    !traits.includes("bounds_map.insert(id, bounds.clone())"),
    "collect_type_params must insert bounds with TypeParamId",
);
for (const [name, text] of [
    ["block_check.rs", blockCheck],
    ["driver.rs", driver],
]) {
    assert(
        !text.includes("insert(*p_id, bounds)"),
        `${name} must not insert raw TypeId into BoundEnv`,
    );
    assert(
        text.includes("TypeParamId::new(*p_id)"),
        `${name} must wrap type parameter declarations as TypeParamId`,
    );
}
assert(
    !driver.includes("format_trait_ref_name"),
    "driver.rs must not carry rendered trait application names as impl lowering authority",
);
assert(
    !driver.includes("applied_trait_name"),
    "driver.rs must not keep applied trait names as split display payloads",
);
assert(
    driver.includes("trait_application.display_name(&ctx)"),
    "driver.rs may derive trait display names only at the symbol-mangling boundary",
);
assert(
    !context.includes("Vec<(TraitBound, TypeId, Span)>"),
    "BlockChecker pending trait checks must not use positional tuple state",
);
assert(
    context.includes("Vec<PendingTraitCheck>"),
    "BlockChecker pending trait checks must use PendingTraitCheck",
);
assert(
    !traitBoundApply.includes(".push((substituted_bound, inferred_arg, span))"),
    "trait_bound_apply.rs must not enqueue positional pending trait check tuples",
);
assert(
    traitBoundApply.includes("PendingTraitCheck {"),
    "trait_bound_apply.rs must enqueue named PendingTraitCheck values",
);
assert(
    functionCheck.includes("type_param_has_trait_application_bound"),
    "deferred function-level trait checks must use typed trait application lookup",
);
assert(
    !functionCheck.includes("type_param_has_trait_bound("),
    "deferred function-level trait checks must not call rendered-name trait bound lookup",
);
assert(
    !functionCheck.includes("&bound.name"),
    "deferred function-level trait checks must not use rendered bound names as authority",
);
const traitCheck = read(path.join(TYPECHECK_DIR, "trait_check.rs"));
const traitCallApply = read(path.join(TYPECHECK_DIR, "trait_call_apply.rs"));
assert(
    !traitCheck.includes("type_param_has_bound_ref"),
    "old type_param_has_bound_ref helper must not be reintroduced",
);
assert(
    !traitCheck.includes("same_label"),
    "BlockChecker trait bound lookup must not duplicate label fallback outside the typed helper",
);
assert(
    traitCheck.includes("type_param_has_trait_application_bound("),
    "BlockChecker trait bound lookup must delegate to the typed helper",
);
for (const [name, text] of [
    ["function_check.rs", functionCheck],
    ["trait_check.rs", traitCheck],
    ["trait_call_apply.rs", traitCallApply],
]) {
    assert(!text.includes("imp.trait_base_name"), `${name} must not inspect split impl trait base names`);
    assert(!text.includes("imp.trait_args"), `${name} must not inspect split impl trait args`);
}
assert(
    traitCallApply.includes("pub(super) enum TraitMethodResolution"),
    "trait method resolution must use a typed enum",
);
for (const variant of [
    "TraitMethodResolution::NotTraitMethod",
    "TraitMethodResolution::Resolved",
    "TraitMethodResolution::MissingSelfType",
    "TraitMethodResolution::UnsatisfiedBound",
    "TraitMethodResolution::PureCallsImpure",
]) {
    assert(traitCallApply.includes(variant), `trait_call_apply.rs must handle ${variant}`);
    assert(selectedCallApply.includes(variant), `selected_call_apply.rs must handle ${variant}`);
}
assert(
    traitCallApply.includes("UnsatisfiedBound { application: TraitApplication }"),
    "trait method unsatisfied-bound diagnostics must carry typed TraitApplication",
);
const traitMethodCallStruct = traitCallApply.match(/pub\(super\) struct TraitMethodCall\s*\{[\s\S]*?\n\}/);
assert(traitMethodCallStruct, "TraitMethodCall struct body must be visible to source policy");
assert(
    traitMethodCallStruct[0].includes("application: TraitApplication"),
    "TraitMethodCall must carry typed TraitApplication",
);
for (const oldField of ["trait_name", "trait_args", "applied_trait_name"]) {
    assert(!traitMethodCallStruct[0].includes(oldField), `TraitMethodCall must not carry ${oldField}`);
    assert(!traitCallApply.includes(`${oldField}:`), `trait_call_apply.rs must not carry ${oldField} as field`);
}
assert(
    !traitCallApply.includes("infer_trait_application_name"),
    "trait method resolution must not derive rendered trait application names before diagnostics",
);
assert(
    !traitCallApply.includes("Option<FuncRef>"),
    "trait method resolution must not collapse resolution state into Option<FuncRef>",
);
assert(
    !traitCallApply.includes("infer_selected_trait_method_callee"),
    "selected trait method resolution must not use the old optional callee helper",
);
assert(
    selectedCallApply.includes("match trait_resolution"),
    "selected callable trait resolution must match TraitMethodResolution explicitly",
);
assert(hir.includes("pub struct HirTraitApplication"), "HIR must define HirTraitApplication");
assert(hir.includes("pub struct HirTraitId"), "HIR trait application must use HirTraitId");
assert(hir.includes("pub struct HirTraitMethodId"), "HIR trait method identity must use HirTraitMethodId");
const hirTraitApplicationStruct = hir.match(/pub struct HirTraitApplication\s*\{[\s\S]*?\n\}/);
assert(hirTraitApplicationStruct, "HirTraitApplication struct body must be visible to source policy");
assert(
    hirTraitApplicationStruct[0].includes("trait_id: HirTraitId"),
    "HirTraitApplication must store trait identity as HirTraitId",
);
assert(
    !hirTraitApplicationStruct[0].includes("base_name: String"),
    "HirTraitApplication must not store trait identity as raw String",
);
const funcRefEnum = hir.match(/pub enum FuncRef\s*\{[\s\S]*?\n\}/);
assert(funcRefEnum, "FuncRef enum body must be visible to source policy");
assert(
    funcRefEnum[0].includes("application: HirTraitApplication"),
    "FuncRef::Trait must store a HirTraitApplication",
);
assert(
    funcRefEnum[0].includes("method: HirTraitMethodId"),
    "FuncRef::Trait must store trait method identity as HirTraitMethodId",
);
assert(
    !funcRefEnum[0].includes("method: String"),
    "FuncRef::Trait must not store trait method identity as raw String",
);
assert(
    !funcRefEnum[0].includes("trait_name: String"),
    "FuncRef::Trait must not store split trait_name",
);
assert(
    !funcRefEnum[0].includes("trait_args: Vec<TypeId>"),
    "FuncRef::Trait must not store split trait_args",
);
const hirImplStruct = hir.match(/pub struct HirImpl\s*\{[\s\S]*?\n\}/);
assert(hirImplStruct, "HirImpl struct body must be visible to source policy");
assert(
    hirImplStruct[0].includes("trait_application: HirTraitApplication"),
    "HirImpl must store HirTraitApplication",
);
for (const oldField of ["trait_name:", "trait_base_name:", "trait_args:"]) {
    assert(!hirImplStruct[0].includes(oldField), `HirImpl must not store ${oldField}`);
}
const dropPlanStruct = dropInsertion.match(/struct DropPlan\s*\{[\s\S]*?\n\}/);
assert(dropPlanStruct, "DropPlan struct body must be visible to source policy");
assert(
    dropPlanStruct[0].includes("trait_application: HirTraitApplication"),
    "DropPlan must carry typed HirTraitApplication",
);
assert(
    dropPlanStruct[0].includes("method_id: HirTraitMethodId"),
    "DropPlan must carry typed HirTraitMethodId",
);
for (const oldField of ["trait_name: String", "method_name: String"]) {
    assert(!dropPlanStruct[0].includes(oldField), `DropPlan must not carry raw ${oldField}`);
}
assert(
    !dropInsertion.includes("plan.trait_name") && !dropInsertion.includes("plan.method_name"),
    "drop insertion must not construct generated Drop trait calls from raw plan names",
);
assert(
    resourceTraitIdentity.includes("pub struct ResourceTraitApplication"),
    "Resource IR must define ResourceTraitApplication",
);
assert(
    resourceTraitIdentity.includes("pub struct ResourceTraitId"),
    "Resource IR trait application must use ResourceTraitId",
);
assert(
    resourceTraitIdentity.includes("pub struct ResourceTraitMethodId"),
    "Resource IR trait method identity must use ResourceTraitMethodId",
);
const resourceTraitApplicationStruct = resourceTraitIdentity.match(
    /pub struct ResourceTraitApplication\s*\{[\s\S]*?\n\}/,
);
assert(
    resourceTraitApplicationStruct,
    "ResourceTraitApplication struct body must be visible to source policy",
);
assert(
    resourceTraitApplicationStruct[0].includes("trait_id: ResourceTraitId"),
    "ResourceTraitApplication must store trait identity as ResourceTraitId",
);
assert(
    !resourceTraitApplicationStruct[0].includes("base_name: String"),
    "ResourceTraitApplication must not store trait identity as raw String",
);
const resourceCallTarget = resourceModel.match(/pub enum ResourceCallTarget\s*\{[\s\S]*?\n\}/);
assert(resourceCallTarget, "ResourceCallTarget enum body must be visible to source policy");
assert(
    resourceCallTarget[0].includes("application: ResourceTraitApplication"),
    "ResourceCallTarget::Trait must store ResourceTraitApplication",
);
assert(
    resourceCallTarget[0].includes("method: ResourceTraitMethodId"),
    "ResourceCallTarget::Trait must store trait method identity as ResourceTraitMethodId",
);
assert(
    !resourceCallTarget[0].includes("method: String"),
    "ResourceCallTarget::Trait must not store trait method identity as raw String",
);
assert(
    !resourceCallTarget[0].includes("trait_name: String"),
    "ResourceCallTarget::Trait must not store split trait_name",
);
assert(
    !resourceCallTarget[0].includes("trait_args: Vec<TypeId>"),
    "ResourceCallTarget::Trait must not store split trait_args",
);
for (const marker of [
    "struct MonoTraitId",
    "struct MonoTraitMethodId",
]) {
    assert(monomorphizeTraitIdentity.includes(marker), `monomorphize/trait_identity.rs must define ${marker}`);
}
assert(
    !monomorphizeTraitLookup.includes("struct MonoTraitId"),
    "monomorphize/trait_lookup.rs must not own trait identity type definitions",
);
assert(
    !monomorphizeTraitLookup.includes("struct MonoTraitMethodId"),
    "monomorphize/trait_lookup.rs must not own trait method identity type definitions",
);
for (const marker of [
    "struct MonoTraitApplication",
    "struct MonoTraitMethodKey",
    "struct MonoTraitLookupKey",
]) {
    assert(monomorphizeTraitLookup.includes(marker), `monomorphize/trait_lookup.rs must define ${marker}`);
}
const monoTraitApplicationStruct = monomorphizeTraitLookup.match(
    /struct MonoTraitApplication\s*\{[\s\S]*?\n\}/,
);
assert(monoTraitApplicationStruct, "MonoTraitApplication struct body must be visible to source policy");
assert(
    monoTraitApplicationStruct[0].includes("trait_id: MonoTraitId"),
    "MonoTraitApplication must use typed MonoTraitId",
);
assert(
    !monoTraitApplicationStruct[0].includes("base_name: String"),
    "MonoTraitApplication must not store trait identity as raw String",
);
const monoTraitMethodKeyStruct = monomorphizeTraitLookup.match(
    /struct MonoTraitMethodKey\s*\{[\s\S]*?\n\}/,
);
assert(monoTraitMethodKeyStruct, "MonoTraitMethodKey struct body must be visible to source policy");
assert(
    monoTraitMethodKeyStruct[0].includes("trait_id: MonoTraitId"),
    "MonoTraitMethodKey must use typed MonoTraitId",
);
assert(
    !monoTraitMethodKeyStruct[0].includes("trait_base_name: String"),
    "MonoTraitMethodKey must not store trait identity as raw String",
);
assert(
    monoTraitMethodKeyStruct[0].includes("method: MonoTraitMethodId"),
    "MonoTraitMethodKey must use typed MonoTraitMethodId",
);
assert(
    !monoTraitMethodKeyStruct[0].includes("method: String"),
    "MonoTraitMethodKey must not store method identity as raw String",
);
const monoTraitLookupKeyStruct = monomorphizeTraitLookup.match(
    /struct MonoTraitLookupKey\s*\{[\s\S]*?\n\}/,
);
assert(monoTraitLookupKeyStruct, "MonoTraitLookupKey struct body must be visible to source policy");
assert(
    monoTraitLookupKeyStruct[0].includes("method: MonoTraitMethodId"),
    "MonoTraitLookupKey must use typed MonoTraitMethodId",
);
assert(
    !monoTraitLookupKeyStruct[0].includes("method: String"),
    "MonoTraitLookupKey must not store method identity as raw String",
);
assert(
    monomorphize.includes("mod trait_lookup;"),
    "monomorphize.rs must keep trait lookup model in a dedicated module",
);
assert(
    monomorphize.includes("mod trait_identity;"),
    "monomorphize.rs must keep trait identity model in a dedicated module",
);
assert(
    monomorphize.includes("impl_map: BTreeMap<MonoTraitLookupKey, usize>"),
    "monomorphize exact impl lookup must use MonoTraitLookupKey",
);
assert(
    monomorphize.includes("impl_method_index: BTreeMap<MonoTraitMethodKey, Vec<usize>>"),
    "monomorphize impl candidate index must use MonoTraitMethodKey",
);
assert(
    monomorphize.includes("trait_lookup_cache: BTreeMap<MonoTraitLookupKey, Option<TraitImplResolution>>"),
    "monomorphize trait lookup cache must use MonoTraitLookupKey",
);
assert(
    !monomorphize.includes("resolve_trait_impl_name"),
    "monomorphize must not resolve trait impls through split trait name parameters",
);
assert(
    /fn resolve_trait_impl\(\s*&mut self,\s*application: &HirTraitApplication,\s*method: &HirTraitMethodId,/m.test(monomorphize),
    "monomorphize trait impl resolver must accept typed HIR trait application and method identity",
);
assert(
    !monomorphize.includes("impl_entry_index"),
    "monomorphize must not keep a duplicate tuple-style impl_entry_index",
);
for (const tupleKey of [
    "BTreeMap<(String, String, TypeId), usize>",
    "BTreeMap<(String, String), Vec<usize>>",
    "BTreeMap<(String, String, Vec<TypeId>, TypeId), Option<TraitImplResolution>>",
]) {
    assert(
        !monomorphize.includes(tupleKey) && !monomorphizeTraitLookup.includes(tupleKey),
        `monomorphize trait lookup must not use positional tuple key ${tupleKey}`,
    );
}

const typedLookup = traits.match(
    /pub\(super\) fn type_param_has_trait_application_bound[\s\S]*?\npub\(super\) fn merge_inferred_instantiation/,
);
assert(typedLookup, "typed trait application bound lookup must exist before inference helper boundary");
assert(
    !typedLookup[0].includes("parse_trait_ref_name("),
    "typed trait application bound lookup must not parse rendered trait names",
);
assert(
    !typedLookup[0].includes("b.name"),
    "typed trait application bound lookup must not compare rendered bound names",
);
assert(
    !typedLookup[0].includes("same_label"),
    "typed trait application bound lookup must not accept same-label TypeId fallback",
);
assert(
    !typedLookup[0].includes("v.label.as_deref"),
    "typed trait application bound lookup must not inspect TypeVar labels as identity",
);
assert(
    !traits.includes("parse_trait_ref_name"),
    "rendered trait application parser must not be reintroduced",
);

console.log("abstraction static verification policy final contract ok");
console.log(JSON.stringify(counts, null, 2));
