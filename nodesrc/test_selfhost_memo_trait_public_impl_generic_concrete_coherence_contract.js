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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_concrete_coherence.nepl";
const connectorRelPath =
    "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_instantiation_projection_connector.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const materializerRelPath =
    "stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer.nepl";
const source = read(relPath);
const code = stripDocComments(source);
const connector = read(connectorRelPath);
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
        "# check/module/memo_trait_public_impl_generic_concrete_coherence",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "generic concrete coherence module must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("full overlap solver ではありません") &&
        source.includes("concrete key collision だけを fail-closed") &&
        source.includes("canonical fingerprint / payload hash / final shape") &&
        source.includes("`memo_trait_operation_public_impl_materializer` の `GenericImplInstantiationUnsupported` はこの slice では維持します") &&
        source.includes("sorted index、bucket 化、merge cursor は後続最適化"),
    "docs must explain the exact concrete coherence boundary, current overlap limitation, materializer fail-closed state, and later performance replacement path",
);
assert.match(
    connector,
    /pub fn selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_schema_version/,
    "connector schema version must be public so downstream coherence can exact-check connector evidence schema",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_instantiation_projection_connector" as \*$/m,
    "coherence must consume projection connector evidence",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_operation_impl_candidate_builder|memo_trait_operation_classifier|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop|memo_trait_public_impl_header|private_cache|private_state|prechecked|neplmeta|neplobj)/,
    "coherence must not import HIR, Resource IR, backend, proof store, operation candidate/proof layers, public impl header, private effect layers, or prechecked artifact layers",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_generic_concrete_coherence/,
    "generic coherence must remain facade-private until solver/coherence/materializer integration is complete",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_generic_concrete_coherence/,
    "checker-layer generic coherence must not be registered in selfhost_ty_sources before accepted path integration",
);
assert.match(
    materializer,
    /GenericImplInstantiationUnsupported/,
    "materializer must still reject detailed generic records until bound solver, connector, and coherence evidence are wired into the accepted path",
);
assert.doesNotMatch(
    materializer,
    /GenericCoherence|memo_trait_public_impl_generic_concrete_coherence/,
    "materializer must not consume the new coherence boundary in this slice",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericConcreteCoherenceRecord:",
        "declaration_ordinal %i32",
        "instantiation_shape_hash %i32",
        "connector_shape_hash %i32",
        "target_canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint",
        "target_canonical_payload_hash %SelfhostMemoTraitCanonicalKeyPayloadHash",
        "target_final_shape_hash %i32",
        "trait_application_canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint",
        "trait_application_canonical_payload_hash %SelfhostMemoTraitCanonicalKeyPayloadHash",
        "trait_application_final_shape_hash %i32",
        "coherence_shape_hash %i32",
    ],
    "record must preserve declaration, connector, instantiation, target key, trait key, and coherence root hash",
);
assert.doesNotMatch(
    source,
    /impl Clone for SelfhostMemoTraitPublicImplGenericConcreteCoherenceRecordTable/,
    "coherence table owns a Vec buffer and must not provide a shallow Clone implementation",
);
assert.doesNotMatch(
    source,
    /impl Copy for SelfhostMemoTraitPublicImplGenericConcreteCoherenceRecordTable/,
    "coherence table owns a Vec buffer and must not provide a Copy implementation",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplGenericConcreteCoherenceErrorKind:",
        "ConnectorSchemaPlaceholder",
        "ConnectorSchemaMismatch %SelfhostMemoTraitPublicImplGenericConcreteCoherenceCollision",
        "InstantiationShapeHashPlaceholder",
        "ConnectorShapeHashPlaceholder",
        "SubstitutionShapeHashPlaceholder",
        "ProjectionShapeHashPlaceholder",
        "TargetFinalShapeHashPlaceholder",
        "TraitApplicationFinalShapeHashPlaceholder",
        "TargetCanonicalFingerprintSchemaPlaceholder",
        "TargetCanonicalFingerprintSchemaMismatch",
        "TargetCanonicalFingerprintHashPlaceholder",
        "TargetCanonicalPayloadSchemaPlaceholder",
        "TargetCanonicalPayloadSchemaMismatch",
        "TargetCanonicalPayloadHashPlaceholder",
        "TraitApplicationCanonicalFingerprintSchemaPlaceholder",
        "TraitApplicationCanonicalFingerprintSchemaMismatch",
        "TraitApplicationCanonicalFingerprintHashPlaceholder",
        "TraitApplicationCanonicalPayloadSchemaPlaceholder",
        "TraitApplicationCanonicalPayloadSchemaMismatch",
        "TraitApplicationCanonicalPayloadHashPlaceholder",
        "DuplicateExactMatch %SelfhostMemoTraitPublicImplGenericConcreteCoherenceCollision",
        "OverlapUnsupported %SelfhostMemoTraitPublicImplGenericConcreteCoherenceCollision",
        "DerivedCoherenceShapeHashPlaceholder",
    ],
    "error kind must keep connector schema, placeholder, canonical material, duplicate, and overlap failures typed",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_concrete_coherence_connector_valid_result"),
    [
        "eq connector.schema_version 0",
        "ConnectorSchemaPlaceholder",
        "not eq connector.schema_version selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_schema_version",
        "ConnectorSchemaMismatch",
        "eq connector.instantiation_shape_hash 0",
        "InstantiationShapeHashPlaceholder",
        "eq connector.connector_shape_hash 0",
        "ConnectorShapeHashPlaceholder",
        "eq connector.substitution_shape_hash 0",
        "SubstitutionShapeHashPlaceholder",
        "eq connector.projection_shape_hash 0",
        "ProjectionShapeHashPlaceholder",
        "eq connector.target_final_shape_hash 0",
        "TargetFinalShapeHashPlaceholder",
        "eq connector.trait_application_final_shape_hash 0",
        "TraitApplicationFinalShapeHashPlaceholder",
    ],
    "connector validation must exact-check schema and reject placeholder root/final-shape fields",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_concrete_coherence_canonical_material_valid_result"),
    [
        "TargetCanonicalFingerprintSchemaPlaceholder",
        "not eq connector.target_canonical_fingerprint.schema_version selfhost_memo_trait_canonical_type_fingerprint_schema_version",
        "TargetCanonicalFingerprintSchemaMismatch",
        "TargetCanonicalFingerprintHashPlaceholder",
        "TargetCanonicalPayloadSchemaPlaceholder",
        "not eq connector.target_canonical_payload_hash.schema_version selfhost_memo_trait_canonical_key_payload_schema_version",
        "TargetCanonicalPayloadSchemaMismatch",
        "TargetCanonicalPayloadHashPlaceholder",
        "TraitApplicationCanonicalFingerprintSchemaPlaceholder",
        "not eq connector.trait_application_canonical_fingerprint.schema_version selfhost_memo_trait_canonical_type_fingerprint_schema_version",
        "TraitApplicationCanonicalFingerprintSchemaMismatch",
        "TraitApplicationCanonicalFingerprintHashPlaceholder",
        "TraitApplicationCanonicalPayloadSchemaPlaceholder",
        "not eq connector.trait_application_canonical_payload_hash.schema_version selfhost_memo_trait_canonical_key_payload_schema_version",
        "TraitApplicationCanonicalPayloadSchemaMismatch",
        "TraitApplicationCanonicalPayloadHashPlaceholder",
    ],
    "coherence must exact-check target and trait canonical fingerprint/payload schemas and nonzero hashes",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_concrete_coherence_same_key"),
    [
        "record.target_canonical_fingerprint",
        "connector.target_canonical_fingerprint",
        "record.target_canonical_payload_hash",
        "connector.target_canonical_payload_hash",
        "record.target_final_shape_hash",
        "connector.target_final_shape_hash",
        "record.trait_application_canonical_fingerprint",
        "connector.trait_application_canonical_fingerprint",
        "record.trait_application_canonical_payload_hash",
        "connector.trait_application_canonical_payload_hash",
        "record.trait_application_final_shape_hash",
        "connector.trait_application_final_shape_hash",
    ],
    "same-key check must use target and trait fingerprint, payload, and final shape",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_concrete_coherence_scan_loop"),
    [
        "selfhost_memo_trait_public_impl_generic_concrete_coherence_exact_record",
        "DuplicateExactMatch",
        "selfhost_memo_trait_public_impl_generic_concrete_coherence_same_key",
        "OverlapUnsupported",
    ],
    "scan loop must reject exact duplicates before broader concrete key overlap",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_concrete_coherence_result"),
    [
        "le declaration_ordinal 0",
        "DeclarationOrdinalPlaceholder",
        "selfhost_memo_trait_public_impl_generic_concrete_coherence_connector_valid_result connector",
        "selfhost_memo_trait_public_impl_generic_concrete_coherence_record_table_len existing",
        "selfhost_memo_trait_public_impl_generic_concrete_coherence_scan_loop existing 0 existing_count declaration_ordinal checked_connector",
        "selfhost_memo_trait_public_impl_generic_concrete_coherence_connector_key_hash declaration_ordinal checked_connector",
        "DerivedCoherenceShapeHashPlaceholder",
        "SelfhostMemoTraitPublicImplGenericConcreteCoherenceEvidence selfhost_memo_trait_public_impl_generic_concrete_coherence_schema_version",
    ],
    "public result API must validate ordinal, connector evidence, existing table, duplicate/overlap scan, and nonzero coherence hash before success",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_concrete_coherence_error_kind_eq"),
    [
        "ConnectorSchemaMismatch payload:",
        "ConnectorSchemaMismatch other:",
        "DuplicateExactMatch payload:",
        "DuplicateExactMatch other:",
        "OverlapUnsupported payload:",
        "OverlapUnsupported other:",
        "selfhost_memo_trait_public_impl_generic_concrete_coherence_collision_eq payload other",
    ],
    "error equality must compare collision payloads for every payload-carrying variant",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_concrete_coherence_stage0"),
    /accepted[\s\S]*schema_placeholder[\s\S]*target_final_placeholder[\s\S]*duplicate[\s\S]*overlap/,
    "stage0 must cover accepted, schema placeholder, final-shape placeholder, exact duplicate, and concrete overlap cases",
);
assert.doesNotMatch(
    code,
    /source_text|display_name|diagnostic_text|span|lexeme|module_path|hir|resource_ir|backend|proof_store|public_surface_hash|proof_artifact|proof_reader|proof_serializer|payload_reader|preseed|decoded|PrivateCache|PrivateState|private_cache|private_state|prechecked|neplmeta|neplobj|method_body|operation_classifier|candidate_builder|GenericImplInstantiationUnsupported\s*:/,
    "coherence must not use text/display/HIR/Resource/backend/proof/public-surface/private-effect/materializer authority",
);
assert.doesNotMatch(contractSource, proseCapPattern, "coherence source policy must not add line-count or doc-comment length caps");

console.log("selfhost memo trait public impl generic concrete coherence contract ok");
