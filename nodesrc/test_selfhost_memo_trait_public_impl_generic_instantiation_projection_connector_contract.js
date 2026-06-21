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

function topLevelBlock(src, name) {
    const pattern = new RegExp(
        `^(?:pub\\s+)?(?:struct|enum)\\s+${name}:[\\s\\S]*?(?=\\n(?:pub\\s+)?(?:struct|enum|impl|fn)\\s|\\n//: [^\\n]*\\n(?:pub\\s+)?(?:struct|enum|impl|fn)\\s|\\s*$)`,
        "m",
    );
    const match = src.match(pattern);
    assert.ok(match, `${name} not found`);
    return match[0];
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath =
    "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_instantiation_projection_connector.nepl";
const instantiationRelPath =
    "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_instantiation.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const materializerRelPath =
    "stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer.nepl";
const source = read(relPath);
const code = stripDocComments(source);
const instantiation = read(instantiationRelPath);
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
        "# check/module/memo_trait_public_impl_generic_instantiation_projection_connector",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "connector module must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("operation materializer、trait bound solver、generic coherence") &&
        source.includes("field からの root hash 再計算") &&
        source.includes("public impl header 側の original target / trait application shape") &&
        source.includes("TypeId はここでも session-local link") &&
        source.includes("materializer の `GenericImplInstantiationUnsupported` を維持します") &&
        source.includes("hash値を直書きせず"),
    "docs must keep materializer/solver/coherence separate, require field-based hash recomputation, explain header original shape, distinguish local TypeId links, keep materializer fail-closed, and avoid brittle fixed hash doctests",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_instantiation" as \*$/m,
    "connector must consume generic instantiation evidence",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_substitution_projection" as \*$/m,
    "connector must consume generic substitution projection evidence",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_substitution_shape" as \*$/m,
    "connector must consume substitution shape evidence for root hash recomputation",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_operation_impl_candidate_builder|memo_trait_operation_classifier|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop|memo_trait_public_impl_header|private_cache|private_state|prechecked|neplmeta|neplobj)/,
    "connector must not import HIR, Resource IR, backend, proof store, operation candidate/proof layers, public impl header, private effect layers, or prechecked artifact layers",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_generic_instantiation_projection_connector/,
    "connector must remain facade-private until solver/coherence/materializer integration is complete",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_generic_instantiation_projection_connector/,
    "connector must not be registered in selfhost_ty_sources before accepted path integration",
);
assert.match(
    materializer,
    /GenericImplInstantiationUnsupported/,
    "materializer must still reject detailed generic records until solver, coherence, and connector evidence are wired into the accepted path",
);
assert.doesNotMatch(
    materializer,
    /InstantiationProjectionConnector|memo_trait_public_impl_generic_instantiation_projection_connector/,
    "materializer must not consume the new connector in this slice",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericInstantiationProjectionConnectorEvidence:",
        "schema_version %i32",
        "instantiation_shape_hash %i32",
        "substitution_shape_hash %i32",
        "projection_shape_hash %i32",
        "pre_substitution_target_type_shape_hash %i32",
        "pre_substitution_trait_application_shape_hash %i32",
        "substituted_target_type_shape_hash %i32",
        "substituted_trait_application_shape_hash %i32",
        "target_canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint",
        "target_canonical_payload_hash %SelfhostMemoTraitCanonicalKeyPayloadHash",
        "target_final_shape_hash %i32",
        "trait_application_canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint",
        "trait_application_canonical_payload_hash %SelfhostMemoTraitCanonicalKeyPayloadHash",
        "trait_application_final_shape_hash %i32",
        "connector_shape_hash %i32",
    ],
    "accepted connector evidence must preserve instantiation, substitution, projection, pre-shape, substituted shape, canonical target material, canonical trait material, and connector root hash",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericInstantiationProjectionConnectorInput:",
        "expected_pre_substitution_target_type_shape_hash %i32",
        "expected_pre_substitution_trait_application_shape_hash %i32",
        "substitution_shape_evidence %SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence",
        "instantiation_evidence %SelfhostMemoTraitPublicImplGenericInstantiationEvidence",
        "projection_evidence %SelfhostMemoTraitPublicImplGenericSubstitutionProjectionEvidence",
    ],
    "input must keep expected header shapes, substitution evidence, instantiation evidence, and projection evidence as distinct typed fields",
);
assertOrdered(
    instantiation,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericInstantiationEvidence:",
        "substitution_shape_hash %i32",
        "substitution_trace_shape_hash %i32",
        "target_substitution_root_type_id %SelfhostTypeId",
    ],
    "instantiation evidence must preserve substitution trace shape hash so later connectors can recompute the instantiation root from accepted evidence fields",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_validate_substitution_result"),
    [
        "eq evidence.schema_version 0",
        "SubstitutionShapeSchemaPlaceholder",
        "not eq evidence.schema_version selfhost_memo_trait_public_impl_generic_substitution_shape_schema_version",
        "SubstitutionShapeSchemaMismatch",
        "eq evidence.substitution_shape_hash 0",
        "SubstitutionShapeHashPlaceholder",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_evidence_hash evidence",
        "not eq recomputed evidence.substitution_shape_hash",
        "SubstitutionShapeHashMismatch",
    ],
    "connector must recheck substitution schema and root hash from fields",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_instantiation_hash"),
    [
        "evidence.substitution_trace_shape_hash",
        "evidence.type_parameter_bound_count",
    ],
    "connector instantiation hash must use stored substitution trace shape hash and not invent a replacement material",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_validate_instantiation_result"),
    [
        "not eq evidence.schema_version selfhost_memo_trait_public_impl_generic_instantiation_schema_version",
        "InstantiationSchemaMismatch",
        "eq evidence.instantiation_shape_hash 0",
        "InstantiationShapeHashPlaceholder",
        "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_instantiation_hash evidence",
        "not eq recomputed evidence.instantiation_shape_hash",
        "InstantiationShapeHashMismatch",
        "not eq evidence.substitution_shape_hash substitution.substitution_shape_hash",
        "InstantiationSubstitutionShapeHashMismatch",
        "not eq selfhost_type_id_index evidence.target_substitution_output_type_id selfhost_type_id_index substitution.target_substitution_output_type_id",
        "InstantiationTargetOutputTypeIdMismatch",
        "not eq selfhost_type_id_index evidence.trait_application_substitution_output_type_id selfhost_type_id_index substitution.trait_application_substitution_output_type_id",
        "InstantiationTraitApplicationOutputTypeIdMismatch",
        "not eq evidence.substituted_target_type_shape_hash substitution.substituted_target_type_shape_hash",
        "InstantiationSubstitutedTargetShapeMismatch",
        "not eq evidence.substituted_trait_application_shape_hash substitution.substituted_trait_application_shape_hash",
        "InstantiationSubstitutedTraitApplicationShapeMismatch",
    ],
    "connector must recheck instantiation hash and link it back to substitution evidence fields",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_validate_projection_result"),
    [
        "eq evidence.projection_shape_hash 0",
        "ProjectionShapeHashPlaceholder",
        "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_projection_hash evidence",
        "not eq recomputed evidence.projection_shape_hash",
        "ProjectionShapeHashMismatch",
        "not eq evidence.substitution_shape_hash substitution.substitution_shape_hash",
        "ProjectionSubstitutionShapeHashMismatch",
        "not eq selfhost_type_id_index evidence.target.source_type_id selfhost_type_id_index instantiation.target_substitution_output_type_id",
        "ProjectionTargetSourceTypeIdMismatch",
        "not eq selfhost_type_id_index evidence.trait_application.source_type_id selfhost_type_id_index instantiation.trait_application_substitution_output_type_id",
        "ProjectionTraitApplicationSourceTypeIdMismatch",
        "not eq evidence.target.final_shape_hash instantiation.substituted_target_type_shape_hash",
        "ProjectionTargetFinalShapeMismatch",
        "not eq evidence.trait_application.final_shape_hash instantiation.substituted_trait_application_shape_hash",
        "ProjectionTraitApplicationFinalShapeMismatch",
    ],
    "connector must recheck projection hash and link projection source/final shapes to instantiation evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_validate_canonical_material_result"),
    [
        "TargetCanonicalFingerprintSchemaPlaceholder",
        "not eq evidence.target.canonical_fingerprint.schema_version selfhost_memo_trait_canonical_type_fingerprint_schema_version",
        "TargetCanonicalFingerprintSchemaMismatch",
        "TargetCanonicalFingerprintHashPlaceholder",
        "TargetCanonicalPayloadSchemaPlaceholder",
        "not eq evidence.target.canonical_payload_hash.schema_version selfhost_memo_trait_canonical_key_payload_schema_version",
        "TargetCanonicalPayloadSchemaMismatch",
        "TargetCanonicalPayloadHashPlaceholder",
        "TraitApplicationCanonicalFingerprintSchemaPlaceholder",
        "not eq evidence.trait_application.canonical_fingerprint.schema_version selfhost_memo_trait_canonical_type_fingerprint_schema_version",
        "TraitApplicationCanonicalFingerprintSchemaMismatch",
        "TraitApplicationCanonicalFingerprintHashPlaceholder",
        "TraitApplicationCanonicalPayloadSchemaPlaceholder",
        "not eq evidence.trait_application.canonical_payload_hash.schema_version selfhost_memo_trait_canonical_key_payload_schema_version",
        "TraitApplicationCanonicalPayloadSchemaMismatch",
        "TraitApplicationCanonicalPayloadHashPlaceholder",
    ],
    "connector must keep target and trait canonical fingerprint/payload schemas and hashes explicit",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_result"),
    [
        "eq input.expected_pre_substitution_target_type_shape_hash 0",
        "ExpectedPreTargetShapeHashPlaceholder",
        "eq input.expected_pre_substitution_trait_application_shape_hash 0",
        "ExpectedPreTraitApplicationShapeHashPlaceholder",
        "not eq input.expected_pre_substitution_target_type_shape_hash substitution.pre_substitution_target_type_shape_hash",
        "ExpectedPreTargetShapeMismatch",
        "not eq input.expected_pre_substitution_trait_application_shape_hash substitution.pre_substitution_trait_application_shape_hash",
        "ExpectedPreTraitApplicationShapeMismatch",
        "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_validate_instantiation_result substitution input.instantiation_evidence",
        "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_validate_projection_result substitution instantiation input.projection_evidence",
        "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_validate_canonical_material_result projection",
        "DerivedConnectorShapeHashPlaceholder",
        "SelfhostMemoTraitPublicImplGenericInstantiationProjectionConnectorEvidence selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_schema_version",
    ],
    "public result API must validate expected pre-shapes, substitution, instantiation, projection, canonical material, and nonzero connector hash before success",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_error_kind_eq"),
    [
        "SubstitutionShapeSchemaMismatch mismatch:",
        "SubstitutionShapeSchemaMismatch other:",
        "SubstitutionShapeHashMismatch mismatch:",
        "SubstitutionShapeHashMismatch other:",
        "InstantiationSchemaMismatch mismatch:",
        "InstantiationSchemaMismatch other:",
        "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_mismatch_eq mismatch other",
        "InstantiationShapeHashMismatch mismatch:",
        "InstantiationShapeHashMismatch other:",
        "InstantiationSubstitutionShapeHashMismatch mismatch:",
        "InstantiationSubstitutionShapeHashMismatch other:",
        "InstantiationTargetOutputTypeIdMismatch mismatch:",
        "InstantiationTargetOutputTypeIdMismatch other:",
        "InstantiationTraitApplicationOutputTypeIdMismatch mismatch:",
        "InstantiationTraitApplicationOutputTypeIdMismatch other:",
        "InstantiationSubstitutedTargetShapeMismatch mismatch:",
        "InstantiationSubstitutedTargetShapeMismatch other:",
        "InstantiationSubstitutedTraitApplicationShapeMismatch mismatch:",
        "InstantiationSubstitutedTraitApplicationShapeMismatch other:",
        "ProjectionSchemaMismatch mismatch:",
        "ProjectionSchemaMismatch other:",
        "ProjectionShapeHashMismatch mismatch:",
        "ProjectionShapeHashMismatch other:",
        "ProjectionSubstitutionShapeHashMismatch mismatch:",
        "ProjectionSubstitutionShapeHashMismatch other:",
        "ProjectionTargetSourceTypeIdMismatch mismatch:",
        "ProjectionTargetSourceTypeIdMismatch other:",
        "ProjectionTraitApplicationSourceTypeIdMismatch mismatch:",
        "ProjectionTraitApplicationSourceTypeIdMismatch other:",
        "ProjectionTargetFinalShapeMismatch mismatch:",
        "ProjectionTargetFinalShapeMismatch other:",
        "ProjectionTraitApplicationFinalShapeMismatch mismatch:",
        "ProjectionTraitApplicationFinalShapeMismatch other:",
        "ExpectedPreTargetShapeMismatch mismatch:",
        "ExpectedPreTargetShapeMismatch other:",
        "ExpectedPreTraitApplicationShapeMismatch mismatch:",
        "ExpectedPreTraitApplicationShapeMismatch other:",
        "TargetCanonicalFingerprintSchemaMismatch mismatch:",
        "TargetCanonicalFingerprintSchemaMismatch other:",
        "TargetCanonicalPayloadSchemaMismatch mismatch:",
        "TargetCanonicalPayloadSchemaMismatch other:",
        "TraitApplicationCanonicalFingerprintSchemaMismatch mismatch:",
        "TraitApplicationCanonicalFingerprintSchemaMismatch other:",
        "TraitApplicationCanonicalPayloadSchemaMismatch mismatch:",
        "TraitApplicationCanonicalPayloadSchemaMismatch other:",
    ],
    "error equality must compare mismatch payloads for every payload-carrying variant",
);
for (const helperName of [
    "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_instantiation_hash",
    "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_projection_hash",
    "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_validate_instantiation_result",
    "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_validate_canonical_material_result",
    "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_error_kind_code",
    "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_stage0_instantiation_evidence",
]) {
    assert.match(
        source,
        new RegExp(`//: ${helperName}:`),
        `${helperName} must have a direct doc comment explaining its role`,
    );
}
assert.match(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_stage0"),
    /accepted[\s\S]*substitution_hash_mismatch[\s\S]*instantiation_hash_mismatch[\s\S]*projection_hash_mismatch[\s\S]*pre_target_mismatch[\s\S]*target_source_mismatch[\s\S]*target_final_mismatch/,
    "stage0 must exercise accepted and representative substitution, instantiation, projection, pre-shape, source TypeId, and final-shape mismatch cases",
);
assert.doesNotMatch(
    code,
    /source_text|display_name|diagnostic_text|span|lexeme|module_path|hir|resource_ir|backend|proof_store|public_surface_hash|proof_artifact|proof_reader|proof_serializer|payload_reader|preseed|decoded|PrivateCache|PrivateState|private_cache|private_state|prechecked|neplmeta|neplobj|method_body|operation_classifier|candidate_builder|GenericImplInstantiationUnsupported\s*:/,
    "connector must not use text/display/HIR/Resource/backend/proof/public-surface/private-effect/materializer authority",
);
assert.doesNotMatch(contractSource, proseCapPattern, "contract must not add line-count or doc-comment length caps");

console.log("selfhost memo trait public impl generic instantiation projection connector contract ok");
