#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function stripDocComments(src) {
    return src
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

function functionBlock(src, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

function assertNoDuplicateErrorKindCodes(src, functionName) {
    const block = functionBlock(src, functionName);
    const entries = [...block.matchAll(/ErrorKind::([A-Za-z0-9_]+)(?: _)?[^\n]*:\n\s+(\d+)/g)].map(
        (match) => ({ variant: match[1], code: Number(match[2]) }),
    );
    assert.ok(entries.length > 0, `${functionName} must expose comparable error kind codes`);
    const seen = new Map();
    for (const entry of entries) {
        const previous = seen.get(entry.code);
        assert.equal(
            previous,
            undefined,
            `${functionName} reuses code ${entry.code} for ${previous} and ${entry.variant}`,
        );
        seen.set(entry.code, entry.variant);
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_substitution_shape.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const materializerRelPath = "stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer.nepl";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const materializer = read(materializerRelPath);
const contractSource = fs.readFileSync(__filename, "utf8").replace(/\r\n/g, "\n");
const proseCapPattern = new RegExp(
    [
        ["max", "Lines"].join(""),
        ["max", "_lines"].join(""),
        ["line", "Limit"].join(""),
        ["line", "_limit"].join(""),
        ["doc", "Comment", "Limit"].join(""),
        ["doc", "_comment", "_limit"].join(""),
        ["doc", "Comment", "Max"].join(""),
        ["doc", "_comment", "_max"].join(""),
        ["documentation", "Line", "Limit"].join(""),
        ["documentation", "_line", "_limit"].join(""),
        ["lines", "\\.length\\s*[<>]=?\\s*\\d+"].join(""),
    ].join("|"),
);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_public_impl_generic_substitution_shape",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "generic substitution shape module must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
        source.includes("任意の nonzero hash として渡せる状態を長く残す") &&
        source.includes("actual substitution engine が作る step stream evidence") &&
        source.includes("終端 step の source/output TypeId") &&
        source.includes("substituted output TypeId から final canonical shape hash を作る処理は、別 producer の責務として残します") &&
        source.includes("既存 materializer の `GenericImplInstantiationUnsupported` はこの slice では維持します"),
    "docs must explain the nonzero-hash hazard, the step-stream evidence boundary, terminal-step binding, the canonical projection residual, and the fail-closed materializer boundary",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store record は accepted substitution shape material に入りません"),
    "docs must exclude source, display, public-surface-hash, HIR, Resource IR, backend, and proof-store authority",
);
assert.doesNotMatch(
    contractSource,
    proseCapPattern,
    "source policy for this producer must not add prose-volume caps",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_generic_substitution_shape/,
    "generic substitution shape producer must remain facade-private until engine and materializer integration are complete",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_generic_substitution_shape/,
    "checker-layer generic substitution shape producer must not be registered in the ty source list",
);
assert.match(
    source,
    /^#import "neplg2\/core\/ty\/ty\/memo_trait_type_argument_identity" as \*$/m,
    "generic substitution shape producer must consume stable type argument identity evidence",
);
assert.match(
    source,
    /^#import "neplg2\/core\/ty\/ty\/substitution" as \*$/m,
    "generic substitution shape producer must consume actual type substitution traversal evidence",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_binder" as \*$/m,
    "generic substitution shape producer must consume detailed generic binder evidence",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_substitution_trace" as \*$/m,
    "generic substitution shape producer must consume typed substitution trace evidence",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_canonical_key|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_operation_impl_candidate_builder|memo_trait_operation_classifier|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop|memo_trait_public_impl_header|private_cache|private_state|prechecked|neplmeta|neplobj)/,
    "generic substitution shape producer must not import canonical-key projection, HIR, Resource IR, backend, proof store, operation classifier/candidate/proof layers, public impl header, private effect layers, or prechecked artifact layers",
);
assert.doesNotMatch(
    code,
    /\b(?:display_name|source_path|source_text|span|lexeme|diagnostic_text|module_path|public_surface_hash|SelfhostSource|SourceSpan|SourceText|Lexeme)\b|hash32\s+(?:source|span|lexeme|display|module|diagnostic)|mix[0-9]*\s+(?:source|span|lexeme|display|module|diagnostic)/,
    "accepted substitution shape evidence must not derive authority from source, display, span, diagnostic, module-path, or public-surface material",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericSubstitutionShapeInput:",
        "generic_binder_evidence %SelfhostMemoTraitPublicImplGenericBinderEvidence",
        "type_argument_count %i32",
        "type_argument_identity_hash %SelfhostMemoTraitStableTypeArgumentIdentityHash",
        "pre_substitution_target_type_shape_hash %Option i32",
        "pre_substitution_trait_application_shape_hash %Option i32",
        "substitution_trace_evidence %SelfhostMemoTraitPublicImplGenericSubstitutionTraceEvidence",
        "target_substitution_evidence %SelfhostTypeSubstitutionEvidence",
        "substituted_target_type_shape_hash %Option i32",
        "trait_application_substitution_evidence %SelfhostTypeSubstitutionEvidence",
        "substituted_trait_application_shape_hash %Option i32",
    ],
    "input must keep binder evidence, type argument identity, pre-substitution shapes, typed trace evidence, target/trait substitution evidence, and substituted shapes as distinct typed fields",
);
assert.doesNotMatch(
    source,
    /substitution_trace_shape_hash %Option i32|SubstitutionTraceShapeHashMissing/,
    "substitution shape producer must not keep the old raw optional trace hash path",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence:",
        "schema_version %i32",
        "type_parameter_count %i32",
        "type_argument_count %i32",
        "type_parameter_bound_count %i32",
        "generic_binder_shape_hash %i32",
        "generic_parameter_table_shape_hash %i32",
        "generic_bound_table_shape_hash %i32",
        "type_argument_identity_hash %SelfhostMemoTraitStableTypeArgumentIdentityHash",
        "pre_substitution_target_type_shape_hash %i32",
        "pre_substitution_trait_application_shape_hash %i32",
        "substitution_trace_shape_hash %i32",
        "target_substitution_root_type_id %SelfhostTypeId",
        "target_substitution_output_type_id %SelfhostTypeId",
        "target_substitution_step_count %i32",
        "target_substitution_step_stream_hash %i32",
        "substituted_target_type_shape_hash %i32",
        "trait_application_substitution_root_type_id %SelfhostTypeId",
        "trait_application_substitution_output_type_id %SelfhostTypeId",
        "trait_application_substitution_step_count %i32",
        "trait_application_substitution_step_stream_hash %i32",
        "substituted_trait_application_shape_hash %i32",
        "substitution_shape_hash %i32",
    ],
    "accepted evidence must preserve every typed component instead of collapsing to one untyped hash",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind:",
        "BinderEvidenceSchemaPlaceholder",
        "BinderEvidenceHashPlaceholder",
        "GenericParameterCountMissing",
        "TypeParameterBoundCountNegative",
        "TypeArgumentCountNegative",
        "TypeArgumentCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "TypeArgumentIdentitySchemaPlaceholder",
        "TypeArgumentIdentityHashPlaceholder",
        "PreSubstitutionTargetTypeShapeHashMissing",
        "PreSubstitutionTargetTypeShapeHashPlaceholder",
        "PreSubstitutionTraitApplicationShapeHashMissing",
        "PreSubstitutionTraitApplicationShapeHashPlaceholder",
        "SubstitutionTraceSchemaPlaceholder",
        "SubstitutionTraceShapeHashPlaceholder",
        "SubstitutionTraceTypeParameterCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "SubstitutionTraceTypeArgumentCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "SubstitutionTraceTypeParameterBoundCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "SubstitutionTraceBinderHashMismatch",
        "SubstitutionTraceParameterTableShapeHashMismatch",
        "SubstitutionTraceBoundTableShapeHashMismatch",
        "SubstitutionTraceTypeArgumentIdentitySchemaPlaceholder",
        "SubstitutionTraceTypeArgumentIdentityHashPlaceholder",
        "SubstitutionTraceTypeArgumentIdentitySchemaMismatch",
        "SubstitutionTraceTypeArgumentIdentityHashMismatch",
        "SubstitutionTraceRecordCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "TargetSubstitutionSchemaPlaceholder",
        "TargetSubstitutionSchemaMismatch",
        "TargetSubstitutionStepStreamHashPlaceholder",
        "TargetSubstitutionStepCountMissing",
        "TargetSubstitutionStepCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "TargetSubstitutionBindingCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "TargetSubstitutionRootTypeIdInvalid",
        "TargetSubstitutionOutputTypeIdInvalid",
        "TargetSubstitutionStepHashMismatch",
        "TargetSubstitutionRootTypeIdMismatch",
        "TargetSubstitutionOutputTypeIdMismatch",
        "TargetSubstitutionStepTableRejected %SelfhostTypeSubstitutionErrorKind",
        "SubstitutedTargetTypeShapeHashMissing",
        "SubstitutedTargetTypeShapeHashPlaceholder",
        "TraitApplicationSubstitutionSchemaPlaceholder",
        "TraitApplicationSubstitutionSchemaMismatch",
        "TraitApplicationSubstitutionStepStreamHashPlaceholder",
        "TraitApplicationSubstitutionStepCountMissing",
        "TraitApplicationSubstitutionStepCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "TraitApplicationSubstitutionBindingCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "TraitApplicationSubstitutionRootTypeIdInvalid",
        "TraitApplicationSubstitutionOutputTypeIdInvalid",
        "TraitApplicationSubstitutionStepHashMismatch",
        "TraitApplicationSubstitutionRootTypeIdMismatch",
        "TraitApplicationSubstitutionOutputTypeIdMismatch",
        "TraitApplicationSubstitutionStepTableRejected %SelfhostTypeSubstitutionErrorKind",
        "SubstitutedTraitApplicationShapeHashMissing",
        "SubstitutedTraitApplicationShapeHashPlaceholder",
        "DerivedSubstitutionShapeHashPlaceholder",
    ],
    "errors must preserve binder, type-argument, input-shape, trace-shape, target/trait substitution, substituted-shape, and derived-hash failures as typed variants",
);
assertOrdered(
    source,
    [
        "enum SelfhostMemoTraitPublicImplGenericSubstitutionShapeStage0ErrorKind:",
        "Stage0StepTableAllocFailed %StdErrorKind",
        "Stage0StepPushFailed %StdErrorKind",
        "Stage0StepHashRejected %SelfhostTypeSubstitutionErrorKind",
    ],
    "stage0 setup failures must use a fixture-only error enum instead of being production producer rejection reasons",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_binder_hash_result"),
    [
        "eq evidence.schema_version 0",
        "BinderEvidenceSchemaPlaceholder",
        "eq evidence.shape_hash 0",
        "BinderEvidenceHashPlaceholder",
        "lt evidence.type_parameter_count 1",
        "GenericParameterCountMissing",
        "lt evidence.type_parameter_bound_count 0",
        "TypeParameterBoundCountNegative",
        "Result::Ok evidence.shape_hash",
    ],
    "binder gate must validate schema, shape hash, generic parameter count, and bound count",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_type_argument_identity_result"),
    [
        "eq hash.schema_version 0",
        "TypeArgumentIdentitySchemaPlaceholder",
        "eq hash.identity_hash 0",
        "TypeArgumentIdentityHashPlaceholder",
        "Result::Ok hash",
    ],
    "type argument identity gate must reject schema and identity placeholders separately",
);
for (const [name, placeholder, missing] of [
    [
        "selfhost_memo_trait_public_impl_generic_substitution_shape_pre_target_result",
        "PreSubstitutionTargetTypeShapeHashPlaceholder",
        "PreSubstitutionTargetTypeShapeHashMissing",
    ],
    [
        "selfhost_memo_trait_public_impl_generic_substitution_shape_pre_trait_result",
        "PreSubstitutionTraitApplicationShapeHashPlaceholder",
        "PreSubstitutionTraitApplicationShapeHashMissing",
    ],
    [
        "selfhost_memo_trait_public_impl_generic_substitution_shape_substituted_target_result",
        "SubstitutedTargetTypeShapeHashPlaceholder",
        "SubstitutedTargetTypeShapeHashMissing",
    ],
    [
        "selfhost_memo_trait_public_impl_generic_substitution_shape_substituted_trait_result",
        "SubstitutedTraitApplicationShapeHashPlaceholder",
        "SubstitutedTraitApplicationShapeHashMissing",
    ],
]) {
    assertOrdered(
        functionBlock(source, name),
        ["Option::Some value", "eq value 0", placeholder, "Result::Ok value", "Option::None", missing],
        `${name} must reject missing and placeholder shape hashes separately`,
    );
}
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_trace_result"),
    [
        "eq evidence.schema_version 0",
        "SubstitutionTraceSchemaPlaceholder",
        "eq evidence.trace_shape_hash 0",
        "SubstitutionTraceShapeHashPlaceholder",
        "not eq evidence.type_parameter_count binder.type_parameter_count",
        "SubstitutionTraceTypeParameterCountMismatch",
        "not eq evidence.type_argument_count type_argument_count",
        "SubstitutionTraceTypeArgumentCountMismatch",
        "not eq evidence.type_parameter_bound_count binder.type_parameter_bound_count",
        "SubstitutionTraceTypeParameterBoundCountMismatch",
        "not eq evidence.generic_binder_shape_hash binder_hash",
        "SubstitutionTraceBinderHashMismatch",
        "not eq evidence.generic_parameter_table_shape_hash binder.parameter_table_shape_hash",
        "SubstitutionTraceParameterTableShapeHashMismatch",
        "not eq evidence.generic_bound_table_shape_hash binder.bound_table_shape_hash",
        "SubstitutionTraceBoundTableShapeHashMismatch",
        "eq evidence.type_argument_identity_hash.schema_version 0",
        "SubstitutionTraceTypeArgumentIdentitySchemaPlaceholder",
        "eq evidence.type_argument_identity_hash.identity_hash 0",
        "SubstitutionTraceTypeArgumentIdentityHashPlaceholder",
        "not eq evidence.type_argument_identity_hash.schema_version type_argument_hash.schema_version",
        "SubstitutionTraceTypeArgumentIdentitySchemaMismatch",
        "not eq evidence.type_argument_identity_hash.identity_hash type_argument_hash.identity_hash",
        "SubstitutionTraceTypeArgumentIdentityHashMismatch",
        "not eq evidence.trace_record_count type_argument_count",
        "SubstitutionTraceRecordCountMismatch",
        "Result::Ok evidence",
    ],
    "trace gate must re-check schema, root hash, counts, binder hash, type argument identity, and trace record count",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_target_terminal_step_result"),
    [
        "let terminal_index %i32 sub evidence.step_count 1",
        'field::get_ref step_table "records"',
        "v::get records terminal_index",
        "not eq selfhost_type_id_index record.source_type_id selfhost_type_id_index evidence.root_type_id",
        "TargetSubstitutionRootTypeIdMismatch",
        "not eq selfhost_type_id_index record.output_type_id selfhost_type_id_index evidence.output_type_id",
        "TargetSubstitutionOutputTypeIdMismatch",
        "TargetSubstitutionStepTableRejected SelfhostTypeSubstitutionErrorKind::EvidenceHashMismatch",
    ],
    "target terminal step gate must bind the final step source/output TypeId to the public evidence root/output TypeId",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_trait_terminal_step_result"),
    [
        "let terminal_index %i32 sub evidence.step_count 1",
        "TraitApplicationSubstitutionRootTypeIdMismatch",
        "TraitApplicationSubstitutionOutputTypeIdMismatch",
        "TraitApplicationSubstitutionStepTableRejected SelfhostTypeSubstitutionErrorKind::EvidenceHashMismatch",
    ],
    "trait terminal step gate must bind the final step source/output TypeId to the trait evidence root/output TypeId",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_target_substitution_result"),
    [
        "eq evidence.schema_version 0",
        "TargetSubstitutionSchemaPlaceholder",
        "not eq evidence.schema_version selfhost_type_substitution_schema_version",
        "TargetSubstitutionSchemaMismatch",
        "eq evidence.step_stream_hash 0",
        "TargetSubstitutionStepStreamHashPlaceholder",
        "lt evidence.step_count 1",
        "TargetSubstitutionStepCountMissing",
        "not eq evidence.step_count selfhost_type_substitution_step_table_len step_table",
        "TargetSubstitutionStepCountMismatch",
        "not eq evidence.binding_count type_argument_count",
        "TargetSubstitutionBindingCountMismatch",
        "lt selfhost_type_id_index evidence.root_type_id 0",
        "TargetSubstitutionRootTypeIdInvalid",
        "lt selfhost_type_id_index evidence.output_type_id 0",
        "TargetSubstitutionOutputTypeIdInvalid",
        "selfhost_type_substitution_step_table_hash_result step_table",
        "not eq step_hash evidence.step_stream_hash",
        "TargetSubstitutionStepHashMismatch",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_target_terminal_step_result evidence step_table",
        "Result::Ok evidence",
        "Result::Err SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TargetSubstitutionStepTableRejected e",
    ],
    "target substitution gate must re-check schema, counts, root/output TypeId, step table hash, and terminal step binding",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_trait_substitution_result"),
    [
        "TraitApplicationSubstitutionSchemaPlaceholder",
        "TraitApplicationSubstitutionSchemaMismatch",
        "TraitApplicationSubstitutionStepStreamHashPlaceholder",
        "TraitApplicationSubstitutionStepCountMissing",
        "TraitApplicationSubstitutionStepCountMismatch",
        "TraitApplicationSubstitutionBindingCountMismatch",
        "TraitApplicationSubstitutionRootTypeIdInvalid",
        "TraitApplicationSubstitutionOutputTypeIdInvalid",
        "selfhost_type_substitution_step_table_hash_result step_table",
        "TraitApplicationSubstitutionStepHashMismatch",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_trait_terminal_step_result evidence step_table",
        "Result::Ok evidence",
        "Result::Err SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TraitApplicationSubstitutionStepTableRejected e",
    ],
    "trait substitution gate must keep diagnostics separate from the target substitution gate and bind terminal step endpoints",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_evidence_hash"),
    [
        "type_argument_material",
        "target_evidence %SelfhostTypeSubstitutionEvidence",
        "trait_evidence %SelfhostTypeSubstitutionEvidence",
        "target_substitution_material",
        "trait_substitution_material",
        "input_shape_material",
        "output_shape_material",
        "binder_table_material",
        "binder_count_material",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_mix4 503301 evidence.schema_version",
    ],
    "substitution shape module must expose a deterministic evidence hash helper so public evidence consumers can recompute the root hash",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_evidence_result"),
    [
        "lt input.type_argument_count 0",
        "TypeArgumentCountNegative",
        "not eq input.generic_binder_evidence.type_parameter_count input.type_argument_count",
        "TypeArgumentCountMismatch",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_binder_hash_result input.generic_binder_evidence",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_type_argument_identity_result input.type_argument_identity_hash",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_pre_target_result input.pre_substitution_target_type_shape_hash",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_pre_trait_result input.pre_substitution_trait_application_shape_hash",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_trace_result input.substitution_trace_evidence input.generic_binder_evidence input.type_argument_count type_argument_hash binder_hash",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_target_substitution_result input.target_substitution_evidence target_step_table input.type_argument_count",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_substituted_target_result input.substituted_target_type_shape_hash",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_trait_substitution_result input.trait_application_substitution_evidence trait_step_table input.type_argument_count",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_substituted_trait_result input.substituted_trait_application_shape_hash",
        "type_argument_hash.schema_version",
        "type_argument_hash.identity_hash",
        "target_substitution_material",
        "trait_substitution_material",
        "trace_evidence.trace_shape_hash",
        "input.generic_binder_evidence.parameter_table_shape_hash",
        "input.generic_binder_evidence.bound_table_shape_hash",
        "input.type_argument_count",
        "DerivedSubstitutionShapeHashPlaceholder",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence schema input.generic_binder_evidence.type_parameter_count",
    ],
    "evidence producer must validate every typed component before deriving accepted substitution evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_count_mismatch_eq"),
    ["eq a.expected b.expected", "eq a.actual b.actual"],
    "count mismatch equality must compare expected and actual payload fields",
);
assertNoDuplicateErrorKindCodes(
    source,
    "selfhost_memo_trait_public_impl_generic_substitution_shape_error_kind_code",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_shape_error_kind_eq"),
    [
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TypeArgumentCountMismatch mismatch",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TypeArgumentCountMismatch other",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_count_mismatch_eq mismatch other",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::SubstitutionTraceTypeParameterCountMismatch mismatch",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::SubstitutionTraceTypeParameterCountMismatch other",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::SubstitutionTraceTypeArgumentCountMismatch mismatch",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::SubstitutionTraceTypeArgumentCountMismatch other",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::SubstitutionTraceTypeParameterBoundCountMismatch mismatch",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::SubstitutionTraceTypeParameterBoundCountMismatch other",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::SubstitutionTraceRecordCountMismatch mismatch",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::SubstitutionTraceRecordCountMismatch other",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TargetSubstitutionStepCountMismatch mismatch",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TargetSubstitutionStepCountMismatch other",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TargetSubstitutionBindingCountMismatch mismatch",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TargetSubstitutionBindingCountMismatch other",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TargetSubstitutionStepTableRejected error",
        "selfhost_type_substitution_error_kind_eq error other",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TraitApplicationSubstitutionStepCountMismatch mismatch",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TraitApplicationSubstitutionStepCountMismatch other",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TraitApplicationSubstitutionBindingCountMismatch mismatch",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TraitApplicationSubstitutionBindingCountMismatch other",
        "SelfhostMemoTraitPublicImplGenericSubstitutionShapeErrorKind::TraitApplicationSubstitutionStepTableRejected error",
        "selfhost_type_substitution_error_kind_eq error other",
    ],
    "error equality must compare TypeArgumentCountMismatch, trace count mismatch, substitution count mismatch, and step-table rejection payloads instead of only matching the variant",
);
assert.doesNotMatch(
    materializer,
    /SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence|memo_trait_public_impl_generic_substitution_shape/,
    "materializer must remain fail-closed until generic substitution evidence is connected with solver and coherence evidence",
);

console.log("selfhost memo trait generic substitution shape contract ok");
