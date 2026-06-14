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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_substitution_trace.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const shapeRelPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_substitution_shape.nepl";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const shape = read(shapeRelPath);
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
        "# check/module/memo_trait_public_impl_generic_substitution_trace",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "generic substitution trace module must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("raw hash として受け取る状態を残すと") &&
        source.includes("entry vector と aggregate hash を再照合します") &&
        source.includes("actual type substitution engine ではありません") &&
        source.includes("`memo_trait_operation_public_impl_materializer` の `GenericImplInstantiationUnsupported` は維持します"),
    "docs must explain the raw trace hash hazard, identity entry/hash revalidation, current engine limit, and fail-closed materializer boundary",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store record は accepted trace material に入りません"),
    "docs must exclude source, display, public-surface-hash, HIR, Resource IR, backend, and proof-store authority",
);
assert.doesNotMatch(contractSource, proseCapPattern, "trace source policy must not add prose-volume caps");
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_generic_substitution_trace/,
    "generic substitution trace producer must remain facade-private until actual substitution/materializer integration is complete",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_generic_substitution_trace/,
    "checker-layer generic substitution trace producer must not be registered in the ty source list",
);
assert.match(
    source,
    /^#import "neplg2\/core\/ty\/ty\/memo_trait_type_argument_identity" as \*$/m,
    "trace producer must consume stable type argument identity entry types",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_binder" as \*$/m,
    "trace producer must consume detailed generic binder records",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_operation_impl_candidate_builder|memo_trait_operation_classifier|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop|memo_trait_public_impl_header|private_cache|private_state|prechecked|neplmeta|neplobj)/,
    "trace producer must not import HIR, Resource IR, backend, proof store, operation classifier/candidate/proof layers, public impl header, private effect layers, or prechecked artifact layers",
);
assert.doesNotMatch(
    code,
    /\b(?:display_name|source_path|source_text|span|lexeme|diagnostic_text|module_path|public_surface_hash|SelfhostSource|SourceSpan|SourceText|Lexeme)\b|hash32\s+(?:source|span|lexeme|display|module|diagnostic)|mix[0-9]*\s+(?:source|span|lexeme|display|module|diagnostic)/,
    "accepted trace evidence must not derive authority from source, display, span, diagnostic, module-path, or public-surface material",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericSubstitutionTraceRecord:",
        "parameter_ordinal %i32",
        "binding %SelfhostTypeParameterBinding",
        "stable_symbol_hash %i32",
        "argument_ordinal %i32",
        "argument_identity_entry %SelfhostMemoTraitStableTypeArgumentIdentityEntry",
    ],
    "trace records must tie binder ordinal/binding/symbol hash to a stable type argument identity entry",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericSubstitutionTraceEvidence:",
        "schema_version %i32",
        "type_parameter_count %i32",
        "type_argument_count %i32",
        "type_parameter_bound_count %i32",
        "generic_binder_shape_hash %i32",
        "type_argument_identity_hash %SelfhostMemoTraitStableTypeArgumentIdentityHash",
        "trace_record_count %i32",
        "trace_shape_hash %i32",
    ],
    "trace evidence must preserve schema, counts, binder hash, type argument identity hash, trace record count, and root trace hash",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplGenericSubstitutionTraceErrorKind:",
        "TraceTableAllocFailed %StdErrorKind",
        "TracePushFailed %StdErrorKind",
        "ParameterTableSetupRejected %SelfhostMemoTraitPublicImplGenericBinderErrorKind",
        "BinderEvidenceSchemaPlaceholder",
        "BinderEvidenceHashPlaceholder",
        "GenericParameterCountMissing",
        "TypeParameterBoundCountNegative",
        "TypeArgumentCountNegative",
        "TypeArgumentCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "TypeArgumentIdentitySchemaPlaceholder",
        "TypeArgumentIdentityHashPlaceholder",
        "TypeArgumentIdentityAggregateRejected %SelfhostMemoTraitStableTypeArgumentIdentityErrorKind",
        "TypeArgumentIdentityAggregateMismatch",
        "ParameterRecordCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "TraceRecordCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "TypeArgumentIdentityEntryReadFailed %i32",
        "TraceParameterOrdinalMismatch %SelfhostMemoTraitPublicImplGenericBinderOrdinalMismatch",
        "TraceParameterBindingIndexMismatch %SelfhostMemoTraitPublicImplGenericBinderOrdinalMismatch",
        "TraceArgumentEntryOrdinalMismatch %SelfhostMemoTraitPublicImplGenericBinderOrdinalMismatch",
        "TraceArgumentIdentityEntryMismatch %i32",
        "ArgumentFingerprintSchemaPlaceholder %i32",
        "ArgumentFingerprintHashPlaceholder %i32",
        "ArgumentPayloadSchemaPlaceholder %i32",
        "ArgumentPayloadHashPlaceholder %i32",
        "DerivedTraceShapeHashPlaceholder",
    ],
    "trace errors must keep allocation, binder, count, ordinal, binding, argument entry, fingerprint, payload, and derived-hash failures as typed variants",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_trace_binder_hash_result"),
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
    "trace producer must validate binder schema, binder hash, generic count, and bound count",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_trace_identity_result"),
    [
        "eq hash.schema_version 0",
        "TypeArgumentIdentitySchemaPlaceholder",
        "eq hash.identity_hash 0",
        "TypeArgumentIdentityHashPlaceholder",
        "selfhost_memo_trait_stable_type_argument_identity_hash_from_entries_result entries argument_count",
        "TypeArgumentIdentityAggregateMismatch",
        "Result::Ok hash",
        "TypeArgumentIdentityAggregateRejected",
    ],
    "trace producer must validate schema, nonzero identity hash, and entry-derived aggregate hash together",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_trace_identity_entry_eq"),
    [
        "eq record_entry.ordinal identity_entry.ordinal",
        "eq record_entry.canonical_fingerprint.schema_version identity_entry.canonical_fingerprint.schema_version",
        "eq record_entry.canonical_fingerprint.root_hash identity_entry.canonical_fingerprint.root_hash",
        "eq record_entry.canonical_payload_hash.schema_version identity_entry.canonical_payload_hash.schema_version",
        "eq record_entry.canonical_payload_hash.payload_hash identity_entry.canonical_payload_hash.payload_hash",
    ],
    "trace producer must compare copied trace entry material with the stable type argument identity owner entry",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_trace_entry_material_result"),
    [
        "eq entry.canonical_fingerprint.schema_version 0",
        "ArgumentFingerprintSchemaPlaceholder",
        "eq entry.canonical_fingerprint.root_hash 0",
        "ArgumentFingerprintHashPlaceholder",
        "eq entry.canonical_payload_hash.schema_version 0",
        "ArgumentPayloadSchemaPlaceholder",
        "eq entry.canonical_payload_hash.payload_hash 0",
        "ArgumentPayloadHashPlaceholder",
        "Result::Ok",
    ],
    "trace producer must validate fingerprint and payload schema/hash before using type argument entries",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_trace_validate_record_result"),
    [
        "not eq record.parameter_ordinal expected_parameter_ordinal",
        "TraceParameterOrdinalMismatch",
        "not selfhost_type_parameter_binding_is_valid record.binding",
        "TraceParameterBindingInvalid",
        "not eq record.binding.binder_depth parameter.binding.binder_depth",
        "TraceParameterBindingDepthUnsupported",
        "not eq record.binding.parameter_index parameter.binding.parameter_index",
        "TraceParameterBindingIndexMismatch",
        "eq record.stable_symbol_hash 0",
        "TraceParameterSymbolHashPlaceholder",
        "not eq record.stable_symbol_hash parameter.stable_symbol_hash",
        "TraceParameterSymbolHashMismatch",
        "not eq record.argument_ordinal index",
        "TraceArgumentOrdinalMismatch",
        "not eq record.argument_identity_entry.ordinal record.argument_ordinal",
        "TraceArgumentEntryOrdinalMismatch",
        "not selfhost_memo_trait_public_impl_generic_substitution_trace_identity_entry_eq record.argument_identity_entry identity_entry",
        "TraceArgumentIdentityEntryMismatch",
        "selfhost_memo_trait_public_impl_generic_substitution_trace_entry_material_result record.argument_identity_entry index",
    ],
    "trace producer must validate parameter identity and argument entry order against binder and table order",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_trace_evidence_result"),
    [
        "lt type_argument_count 0",
        "TypeArgumentCountNegative",
        "not eq binder_evidence.type_parameter_count type_argument_count",
        "TypeArgumentCountMismatch",
        "selfhost_memo_trait_public_impl_generic_substitution_trace_binder_hash_result binder_evidence",
        "selfhost_memo_trait_public_impl_generic_substitution_trace_identity_result type_argument_identity type_argument_count",
        "not eq parameter_count binder_evidence.type_parameter_count",
        "ParameterRecordCountMismatch",
        "not eq trace_count type_argument_count",
        "TraceRecordCountMismatch",
        "field::get_ref type_argument_identity \"entries\"",
        "selfhost_memo_trait_public_impl_generic_substitution_trace_validate_loop parameters trace identity_entries 0 type_argument_count",
        "DerivedTraceShapeHashPlaceholder",
        "SelfhostMemoTraitPublicImplGenericSubstitutionTraceEvidence schema binder_evidence.type_parameter_count",
    ],
    "trace evidence result must validate counts, binder, identity, records, and nonzero root hash before success",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_trace_error_kind_eq"),
    [
        "TypeArgumentIdentityAggregateRejected error:",
        "TypeArgumentIdentityAggregateRejected other:",
        "TypeArgumentCountMismatch mismatch:",
        "TypeArgumentCountMismatch other:",
        "ParameterRecordCountMismatch mismatch:",
        "ParameterRecordCountMismatch other:",
        "TraceRecordCountMismatch mismatch:",
        "TraceRecordCountMismatch other:",
        "TraceParameterOrdinalMismatch mismatch:",
        "TraceParameterOrdinalMismatch other:",
        "TraceParameterBindingIndexMismatch mismatch:",
        "TraceParameterBindingIndexMismatch other:",
        "TraceArgumentEntryOrdinalMismatch mismatch:",
        "TraceArgumentEntryOrdinalMismatch other:",
        "TypeArgumentIdentityEntryReadFailed index:",
        "TypeArgumentIdentityEntryReadFailed other:",
        "TraceArgumentIdentityEntryMismatch index:",
        "TraceArgumentIdentityEntryMismatch other:",
        "ArgumentFingerprintHashPlaceholder index:",
        "ArgumentFingerprintHashPlaceholder other:",
    ],
    "trace error equality must compare payloads for mismatch and placeholder-index variants",
);
assertOrdered(
    source,
    [
        "accepted %Result",
        "argument_mismatch %Result",
        "identity_placeholder %Result",
        "trace_count_mismatch %Result",
        "binding_mismatch %Result",
        "fingerprint_placeholder %Result",
        "selfhost_memo_trait_public_impl_generic_substitution_trace_stage0_summary_new accepted argument_mismatch identity_placeholder trace_count_mismatch binding_mismatch fingerprint_placeholder",
    ],
    "stage0 must exercise accepted, argument mismatch, identity placeholder, trace count mismatch, binding mismatch, and fingerprint placeholder cases",
);
assert.match(
    shape,
    /^#import "\.\/memo_trait_public_impl_generic_substitution_trace" as \*$/m,
    "substitution shape producer must import typed trace evidence after the trace module exists",
);
assert.match(
    shape,
    /substitution_trace_evidence %SelfhostMemoTraitPublicImplGenericSubstitutionTraceEvidence/,
    "substitution shape input must consume typed trace evidence",
);
assert.doesNotMatch(
    shape,
    /substitution_trace_shape_hash %Option i32|SubstitutionTraceShapeHashMissing/,
    "substitution shape input must not keep the old raw optional trace hash path",
);

console.log("selfhost memo trait public impl generic substitution trace contract ok");
