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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_substitution_projection.nepl";
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
        "# check/module/memo_trait_public_impl_generic_substitution_projection",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "generic substitution projection module must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("`SelfhostTypeId` は arena-local index") &&
        source.includes("source text、span、display name、diagnostic text") &&
        source.includes("materializer の `GenericImplInstantiationUnsupported` はこの slice では維持します") &&
        source.includes("schema、root hash placeholder、field からの root hash 再計算") &&
        source.includes("同一 arena provenance は上流の owner boundary が保証する precondition") &&
        source.includes("`projection_shape_hash` は session-local linkage を含む checker-layer root hash"),
    "docs must explain arena-local TypeId, forbidden authority, fail-closed materializer, substitution evidence recheck, same-arena precondition, and session-local projection hash",
);
assert.match(
    source,
    /SelfhostMemoTraitPublicImplGenericSubstitutionProjectionEvidence:[\s\S]*schema_version %i32[\s\S]*substitution_shape_hash %i32[\s\S]*target %SelfhostMemoTraitPublicImplGenericSubstitutionProjectedType[\s\S]*trait_application %SelfhostMemoTraitPublicImplGenericSubstitutionProjectedType[\s\S]*projection_shape_hash %i32/,
    "accepted evidence must preserve schema, substitution shape hash, target projection, trait application projection, and projection root hash",
);
assert.match(
    source,
    /SelfhostMemoTraitPublicImplGenericSubstitutionProjectedType:[\s\S]*source_type_id %SelfhostTypeId[\s\S]*canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*canonical_payload_hash %SelfhostMemoTraitCanonicalKeyPayloadHash[\s\S]*final_shape_hash %i32/,
    "projected type must keep source TypeId only as local link and preserve canonical fingerprint plus payload hash",
);
assert.match(
    source,
    /SelfhostMemoTraitPublicImplGenericSubstitutionProjectionStage0Summary:[\s\S]*target_invalid %Result[\s\S]*target_missing_record %Result[\s\S]*target_type_parameter_unsupported %Result[\s\S]*target_trait_role_hash_diff %bool/,
    "stage0 summary must cover negative TypeId, positive missing record, unsupported type parameter projection failures, and target/trait role-tag hash distinction",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplGenericSubstitutionProjectionErrorKind:",
        "SubstitutionShapeSchemaPlaceholder",
        "SubstitutionShapeSchemaMismatch",
        "SubstitutionShapeHashPlaceholder",
        "SubstitutionShapeHashMismatch",
        "TargetOutputTypeIdInvalid",
        "TraitApplicationOutputTypeIdInvalid",
        "TargetTypeRejected %SelfhostMemoTraitPublicImplGenericSubstitutionProjectionTypeErrorKind",
        "TraitApplicationTypeRejected %SelfhostMemoTraitPublicImplGenericSubstitutionProjectionTypeErrorKind",
        "ProjectionShapeHashPlaceholder",
    ],
    "error enum must split substitution evidence, target projection, trait projection, and derived hash failures",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_projection_type_result"),
    [
        "selfhost_canonical_type_key_project_from_arena types type_id",
        "selfhost_memo_trait_canonical_type_fingerprint_result nominal_table &key_arena root",
        "selfhost_memo_trait_canonical_key_payload_hash_result nominal_table &key_arena root",
        "selfhost_memo_trait_public_impl_generic_substitution_projection_type_final_shape_hash_result fingerprint payload_hash",
        "selfhost_canonical_type_key_arena_free key_arena",
    ],
    "type projection must delegate to existing canonical key projection, fingerprint, payload hash, and free the temporary canonical arena",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_projection_result"),
    [
        "eq substitution_evidence.schema_version 0",
        "SubstitutionShapeSchemaPlaceholder",
        "not eq substitution_evidence.schema_version selfhost_memo_trait_public_impl_generic_substitution_shape_schema_version",
        "SubstitutionShapeSchemaMismatch",
        "eq substitution_evidence.substitution_shape_hash 0",
        "SubstitutionShapeHashPlaceholder",
        "not eq substitution_evidence.substitution_shape_hash selfhost_memo_trait_public_impl_generic_substitution_shape_evidence_hash substitution_evidence",
        "SubstitutionShapeHashMismatch",
        "lt selfhost_type_id_index substitution_evidence.target_substitution_output_type_id 0",
        "TargetOutputTypeIdInvalid",
        "lt selfhost_type_id_index substitution_evidence.trait_application_substitution_output_type_id 0",
        "TraitApplicationOutputTypeIdInvalid",
        "selfhost_memo_trait_public_impl_generic_substitution_projection_type_result types nominal_table substitution_evidence.target_substitution_output_type_id",
        "selfhost_memo_trait_public_impl_generic_substitution_projection_type_result types nominal_table substitution_evidence.trait_application_substitution_output_type_id",
        "SelfhostMemoTraitPublicImplGenericSubstitutionProjectionEvidence schema substitution_evidence.substitution_shape_hash target trait_application projection_hash",
    ],
    "producer must recheck substitution evidence before projecting target and trait application outputs",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_projection_evidence_hash"),
    [
        "let target_payload %i32",
        "evidence.target.source_type_id",
        "let trait_payload %i32",
        "evidence.trait_application.source_type_id",
        "let target_material %i32 selfhost_memo_trait_public_impl_generic_substitution_projection_mix2 613311 target_payload",
        "let trait_material %i32 selfhost_memo_trait_public_impl_generic_substitution_projection_mix2 613313 trait_payload",
        "let projection_material %i32 selfhost_memo_trait_public_impl_generic_substitution_projection_mix2 target_material trait_material",
        "selfhost_memo_trait_public_impl_generic_substitution_projection_mix4 613301 evidence.schema_version evidence.substitution_shape_hash projection_material",
    ],
    "projection evidence hash must keep target and trait application canonical material distinct with role tags before aggregation",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_projection_result"),
    /Result::Err SelfhostMemoTraitPublicImplGenericSubstitutionProjectionErrorKind::TraitApplicationTypeRejected e[\s\S]*Result::Err SelfhostMemoTraitPublicImplGenericSubstitutionProjectionErrorKind::TargetTypeRejected e/,
    "producer must wrap target and trait application projection failures in distinct typed errors",
);
assert.ok(
    source.includes("SelfhostCanonicalTypeKeyProjectErrorKind::MissingTypeRecord") &&
        source.includes("SelfhostMemoTraitCanonicalFingerprintErrorKind::TypeParameterUnsupported"),
    "contract fixtures must cover missing arena record and unresolved type parameter rejection payloads",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_projection_stage0_after_unit"),
    [
        "let missing_type_id %SelfhostTypeId selfhost_type_id_new 99",
        "let missing_record_evidence %SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence",
        "let parameter_evidence %SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence",
        "let target_missing_record %Result",
        "let target_type_parameter_unsupported %Result",
        "let target_trait_role_hash_diff %bool",
    ],
    "stage0 must exercise positive missing TypeId, parameter unsupported, and target/trait role-swap hash cases through the public producer",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_substitution_projection_type_error_eq"),
    [
        "ProjectionRejected error:",
        "ProjectionRejected other:",
        "selfhost_memo_trait_public_impl_generic_substitution_projection_project_error_eq error other",
        "FingerprintRejected error:",
        "FingerprintRejected other:",
        "selfhost_memo_trait_canonical_fingerprint_error_kind_eq error other",
        "PayloadRejected error:",
        "PayloadRejected other:",
        "selfhost_memo_trait_canonical_key_payload_error_kind_eq error other",
    ],
    "nested projection, fingerprint, and payload errors must compare typed payloads",
);
assert.doesNotMatch(
    code,
    /#import\s+"(?:neplg2\/core\/hir|neplg2\/core\/resource|neplg2\/core\/backend|\.\/memo_trait_operation_public_impl_materializer|\.\/memo_trait_operation_evidence_producer|\.\/memo_trait_operation_public_impl_materializer)"/,
    "projection producer must not import HIR, Resource IR, backend, operation evidence producer, or materializer layers",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_generic_substitution_projection/,
    "projection producer must remain private to focused tests until solver/coherence/materializer connections are ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_generic_substitution_projection/,
    "projection producer must not be registered as a selfhost source bundle before the accepted path is connected",
);
assert.doesNotMatch(
    materializer,
    /SelfhostMemoTraitPublicImplGenericSubstitutionProjectionEvidence|memo_trait_public_impl_generic_substitution_projection/,
    "materializer must remain fail-closed and must not consume projection evidence in this slice",
);
assert.match(
    materializer,
    /GenericImplInstantiationUnsupported/,
    "materializer must still reject detailed generic records until solver, coherence, and accepted-path evidence are connected",
);
assert.doesNotMatch(
    code,
    /source_text|display_name|diagnostic_text|span|lexeme|module_path|hir|resource_ir|backend|proof_store|public_surface_hash|proof_artifact|proof_reader|proof_serializer|payload_reader|preseed|decoded|PrivateCache|PrivateState|private_cache|private_state|prechecked|neplmeta|neplobj|method_body|operation_classifier|candidate_builder/,
    "projection producer must not use text/display/HIR/Resource/backend/proof/public-surface/private-effect authority",
);
assert.doesNotMatch(contractSource, proseCapPattern, "contract must not add line-count or doc-comment length caps");

console.log("selfhost memo trait generic substitution projection contract ok");
