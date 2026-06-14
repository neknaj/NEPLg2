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
        source.includes("actual type substitution engine はまだ実装しません") &&
        source.includes("既存 materializer の `GenericImplInstantiationUnsupported` はこの slice では維持します"),
    "docs must explain the nonzero-hash hazard, the current engine limit, and the fail-closed materializer boundary",
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
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_operation_impl_candidate_builder|memo_trait_operation_classifier|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop|memo_trait_public_impl_header|private_cache|private_state|prechecked|neplmeta|neplobj)/,
    "generic substitution shape producer must not import HIR, Resource IR, backend, proof store, operation classifier/candidate/proof layers, public impl header, private effect layers, or prechecked artifact layers",
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
        "substituted_target_type_shape_hash %Option i32",
        "substituted_trait_application_shape_hash %Option i32",
    ],
    "input must keep binder evidence, type argument identity, pre-substitution shapes, typed trace evidence, and substituted shapes as distinct typed fields",
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
        "substituted_target_type_shape_hash %i32",
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
        "SubstitutedTargetTypeShapeHashMissing",
        "SubstitutedTargetTypeShapeHashPlaceholder",
        "SubstitutedTraitApplicationShapeHashMissing",
        "SubstitutedTraitApplicationShapeHashPlaceholder",
        "DerivedSubstitutionShapeHashPlaceholder",
    ],
    "errors must preserve binder, type-argument, input-shape, trace-shape, substituted-shape, and derived-hash failures as typed variants",
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
        "selfhost_memo_trait_public_impl_generic_substitution_shape_substituted_target_result input.substituted_target_type_shape_hash",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_substituted_trait_result input.substituted_trait_application_shape_hash",
        "type_argument_hash.schema_version",
        "type_argument_hash.identity_hash",
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
    ],
    "error equality must compare TypeArgumentCountMismatch and trace count mismatch payloads instead of only matching the variant",
);
assert.doesNotMatch(
    materializer,
    /SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence|memo_trait_public_impl_generic_substitution_shape/,
    "materializer must remain fail-closed until generic substitution evidence is connected with solver and coherence evidence",
);

console.log("selfhost memo trait generic substitution shape contract ok");
