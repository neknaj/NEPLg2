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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_materializer_connector.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const materializerRelPath =
    "stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer.nepl";
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
        "# check/module/memo_trait_public_impl_generic_materializer_connector",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "generic materializer connector must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("後続 materializer が読む 1 つの typed evidence") &&
        source.includes("source text、span、lexeme、display name、diagnostic text") &&
        source.includes("record は `Detailed` generic binder evidence を持たなければなりません") &&
        source.includes("bound solving status はこの connector でも") &&
        source.includes("canonical target / trait material") &&
        source.includes("operation classifier、candidate builder、method body purity、Drop no-escape") &&
        source.includes("materializer が `connector_result` を実行するための input owner table") &&
        source.includes("input table は最終 evidence ではなく") &&
        source.includes("producer provenance の代替ではありません") &&
        source.includes("stage0 fixture が materializer record の trait source identity を作るためだけに使います") &&
        source.includes("connector input table を受け取る generic-aware API") &&
        source.includes("materializer record ごとに `connector_result` を実行") &&
        source.includes("connector table を受け取らない既存 materializer API") &&
        source.includes("recheck 関数で field から再検査します"),
    "docs must state that this is a typed bridge only, rejects source-derived authority, requires Detailed binder evidence, avoids proof/candidate layers, and documents the generic-aware materializer connection",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_generic_materializer_connector/,
    "generic materializer connector must remain facade-private until full materializer integration is complete",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_generic_materializer_connector/,
    "checker-layer generic materializer connector must not be registered in selfhost_ty_sources before accepted path integration",
);
assert.match(
    materializer,
    /GenericImplInstantiationUnsupported/,
    "materializer must keep the no-connector detailed generic path fail-closed",
);
assert.match(
    materializer,
    /^#import "\.\/memo_trait_public_impl_generic_materializer_connector" as \*$/m,
    "generic-aware materializer path must import the connector evidence boundary explicitly",
);
assertOrdered(
    functionBlock(materializer, "selfhost_memo_trait_operation_public_impl_materializer_generic_record_to_builder_input_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_generic_connector_for_record_result record connectors",
        "selfhost_memo_trait_public_impl_header_evidence_result impl_header",
        "selfhost_memo_trait_operation_classifier_evidence_result trait_application",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_new connector.concrete_target_type_id classifier.operation impl_header trait_application some connector.target_final_shape_hash record.method_body_root record.fuel",
    ],
    "generic-aware materializer path must consume rechecked connector evidence and use concrete target identity plus final target shape",
);
assertOrdered(
    functionBlock(materializer, "selfhost_memo_trait_operation_public_impl_materializer_generic_connector_for_record_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_generic_connector_lookup_loop connectors declaration_ordinal 0 none",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_result record input.bound_status input.instantiation input.projection_connector input.coherence",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_evidence_recheck_result evidence",
    ],
    "generic-aware materializer path must derive bridge evidence by running connector_result for each matching input instead of trusting a precomputed evidence table",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_operation_public_impl_materializer_record" as \*$/m,
    "bridge must consume typed materializer records through the shared acyclic record module",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_operation_public_impl_materializer" as \*$/m,
    "bridge must not import the materializer body because the materializer body imports the bridge for the generic-aware accepted path",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_instantiation" as \*$/m,
    "bridge must consume generic instantiation evidence",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_instantiation_projection_connector" as \*$/m,
    "bridge must consume projection connector evidence",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_concrete_coherence" as \*$/m,
    "bridge must consume concrete coherence evidence",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|private_cache|private_state|prechecked|neplmeta|neplobj)/,
    "bridge must not import Resource IR, backend, proof store, private effect layers, or prechecked artifact layers",
);
assert.doesNotMatch(
    code,
    /selfhost_memo_trait_operation_classifier_evidence_result|selfhost_memo_trait_operation_impl_candidate_builder|SelfhostMemoTraitOperationEvidenceProducerInput|SelfhostMemoTraitAggregateProofStatus/,
    "bridge must not invoke operation classifier, candidate builder, operation producer, or aggregate proof folding",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_materializer_connector_result"),
    /memo_trait_operation_classifier|source_identity|trusted_source|operation_source/,
    "result path must not use classifier, trusted source registry, or fixture-only source identity helpers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericMaterializerConnectorEvidence:",
        "schema_version %i32",
        "type_id %SelfhostTypeId",
        "concrete_target_type_id %SelfhostTypeId",
        "module_fingerprint %i32",
        "declaration_ordinal %i32",
        "type_parameter_count %i32",
        "type_parameter_bound_count %i32",
        "generic_binder_shape_hash %i32",
        "generic_parameter_table_shape_hash %i32",
        "generic_bound_table_shape_hash %i32",
        "bound_solution_shape_hash %i32",
        "pre_substitution_target_type_shape_hash %i32",
        "pre_substitution_trait_application_shape_hash %i32",
        "instantiation_shape_hash %i32",
        "connector_shape_hash %i32",
        "substitution_shape_hash %i32",
        "coherence_shape_hash %i32",
        "substituted_target_type_shape_hash %i32",
        "substituted_trait_application_shape_hash %i32",
        "target_canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint",
        "target_canonical_payload_hash %SelfhostMemoTraitCanonicalKeyPayloadHash",
        "target_final_shape_hash %i32",
        "trait_application_canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint",
        "trait_application_canonical_payload_hash %SelfhostMemoTraitCanonicalKeyPayloadHash",
        "trait_application_final_shape_hash %i32",
        "materializer_connector_shape_hash %i32",
    ],
    "bridge evidence must preserve record identity, concrete target identity, generic binder material, original shapes, downstream evidence roots, substituted shapes, substitution root, and bridge root hash",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericMaterializerConnectorInput:",
        "declaration_ordinal %i32",
        "bound_status %SelfhostMemoTraitPublicImplGenericBoundSolvingStatus",
        "instantiation %SelfhostMemoTraitPublicImplGenericInstantiationEvidence",
        "projection_connector %SelfhostMemoTraitPublicImplGenericInstantiationProjectionConnectorEvidence",
        "coherence %SelfhostMemoTraitPublicImplGenericConcreteCoherenceEvidence",
        "pub struct SelfhostMemoTraitPublicImplGenericMaterializerConnectorInputTable:",
        "records %Vec SelfhostMemoTraitPublicImplGenericMaterializerConnectorInput",
        "pub fn selfhost_memo_trait_public_impl_generic_materializer_connector_input_table_push",
    ],
    "bridge must provide connector input records and an input table without storing final bridge evidence",
);
assert.doesNotMatch(
    source,
    /SelfhostMemoTraitPublicImplGenericMaterializerConnectorEvidenceTable|selfhost_memo_trait_public_impl_generic_materializer_connector_evidence_table_push_result/,
    "bridge must not expose a final generic connector evidence table because materializer must run connector_result itself",
);
assert.doesNotMatch(
    materializer,
    /selfhost_memo_trait_operation_public_impl_materializer_generic_connector_table_push|pub struct SelfhostMemoTraitOperationPublicImplMaterializerGenericConnectorTable:|SelfhostMemoTraitPublicImplGenericMaterializerConnectorEvidenceTable/,
    "materializer must not own, populate, or accept a raw generic connector evidence table",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplGenericMaterializerConnectorErrorKind:",
        "ConnectorInputTableAllocFailed %StdErrorKind",
        "ConnectorInputTablePushFailed %StdErrorKind",
        "RecordDeclarationOrdinalMissing",
        "RecordDeclarationOrdinalPlaceholder",
        "RecordTargetShapeMissing",
        "RecordTargetShapePlaceholder",
        "RecordTraitApplicationShapeMissing",
        "RecordTraitApplicationShapePlaceholder",
        "RecordGenericBinderMonomorphic",
        "RecordGenericBinderCountMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "RecordGenericBoundCountMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "RecordGenericParameterTableHashMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "RecordGenericBoundTableHashMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "RecordGenericBinderShapeHashMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "InstantiationSchemaPlaceholder",
        "InstantiationSchemaMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "InstantiationShapeHashPlaceholder",
        "BoundSolvingRequired",
        "BoundSolvingUnexpected",
        "BoundSolvingUnsolved %i32",
        "BoundSolvingSchemaPlaceholder",
        "BoundSolvingSchemaMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "BoundSolvingCountMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "BoundSolvingPolicyHashPlaceholder",
        "BoundSolvingProofShapeHashPlaceholder",
        "BoundSolutionShapeHashMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "DerivedBoundSolutionShapeHashPlaceholder",
        "ConnectorSchemaPlaceholder",
        "ConnectorSchemaMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "ConnectorShapeHashPlaceholder",
        "ConnectorInstantiationShapeMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "ConnectorSubstitutionShapeHashMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "ConnectorSubstitutedTargetShapeMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "ConnectorSubstitutedTraitApplicationShapeMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "ConnectorTargetCanonicalFingerprintSchemaPlaceholder",
        "ConnectorTargetCanonicalPayloadHashPlaceholder",
        "ConnectorTraitApplicationCanonicalFingerprintSchemaPlaceholder",
        "ConnectorTraitApplicationCanonicalPayloadHashPlaceholder",
        "CoherenceSchemaPlaceholder",
        "CoherenceDeclarationMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "CoherenceConnectorShapeMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "CoherenceTargetCanonicalFingerprintHashMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "CoherenceTargetCanonicalPayloadHashMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "CoherenceTraitApplicationCanonicalFingerprintHashMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "CoherenceTraitApplicationCanonicalPayloadHashMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "ConcreteTargetTypeIdInvalid %i32",
        "MaterializerConnectorShapeHashMismatch %SelfhostMemoTraitPublicImplGenericMaterializerConnectorMismatch",
        "DerivedMaterializerConnectorShapeHashPlaceholder",
    ],
    "errors must separate record header, binder, instantiation, connector, coherence, concrete target identity, and derived bridge hash failures",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_materializer_connector_solved_bound_hash_result"),
    [
        "evidence.schema_version",
        "BoundSolvingSchemaPlaceholder",
        "selfhost_memo_trait_public_impl_generic_bound_solving_evidence_schema_version",
        "BoundSolvingSchemaMismatch",
        "evidence.solved_bound_count",
        "expected_bound_count",
        "BoundSolvingCountMismatch",
        "evidence.solver_policy_hash",
        "BoundSolvingPolicyHashPlaceholder",
        "evidence.proof_shape_hash",
        "BoundSolvingProofShapeHashPlaceholder",
    ],
    "all-solved bound status must validate schema against the public bound-solving evidence schema before using count or proof hashes",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_materializer_connector_binder_valid_result"),
    [
        "SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence::Monomorphic:",
        "RecordGenericBinderMonomorphic",
        "SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence::Detailed binder:",
        "record.type_parameter_count",
        "instantiation.type_parameter_count",
        "RecordGenericBinderCountMismatch",
        "record.type_parameter_bound_count",
        "instantiation.type_parameter_bound_count",
        "RecordGenericBoundCountMismatch",
        "binder.parameter_table_shape_hash",
        "instantiation.generic_parameter_table_shape_hash",
        "RecordGenericParameterTableHashMismatch",
        "binder.bound_table_shape_hash",
        "instantiation.generic_bound_table_shape_hash",
        "RecordGenericBoundTableHashMismatch",
        "binder.shape_hash",
        "instantiation.generic_binder_shape_hash",
        "RecordGenericBinderShapeHashMismatch",
    ],
    "binder validation must require Detailed evidence and compare record/binder fields with instantiation evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_materializer_connector_connector_valid_result"),
    [
        "connector.schema_version",
        "connector.connector_shape_hash",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_connector_target_canonical_valid_result connector",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_connector_trait_canonical_valid_result connector",
    ],
    "connector validation must recheck connector schema/root and target/trait canonical material before accepting public connector evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_materializer_connector_coherence_canonical_match_result"),
    [
        "connector.target_canonical_fingerprint.schema_version",
        "coherence.target_canonical_fingerprint.schema_version",
        "CoherenceTargetCanonicalFingerprintSchemaMismatch",
        "connector.target_canonical_fingerprint.root_hash",
        "coherence.target_canonical_fingerprint.root_hash",
        "CoherenceTargetCanonicalFingerprintHashMismatch",
        "connector.target_canonical_payload_hash.payload_hash",
        "coherence.target_canonical_payload_hash.payload_hash",
        "CoherenceTargetCanonicalPayloadHashMismatch",
        "connector.trait_application_canonical_fingerprint.root_hash",
        "coherence.trait_application_canonical_fingerprint.root_hash",
        "CoherenceTraitApplicationCanonicalFingerprintHashMismatch",
        "connector.trait_application_canonical_payload_hash.payload_hash",
        "coherence.trait_application_canonical_payload_hash.payload_hash",
        "CoherenceTraitApplicationCanonicalPayloadHashMismatch",
    ],
    "coherence match must compare canonical target and trait material field-by-field instead of trusting root hashes",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_materializer_connector_result"),
    [
        "selfhost_memo_trait_public_impl_generic_materializer_connector_instantiation_valid_result instantiation",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_connector_valid_result connector",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_coherence_valid_result coherence",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_bound_solution_hash_result checked_instantiation.type_parameter_bound_count bound_status",
        "BoundSolutionShapeHashMismatch",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_instantiation_match_result checked_instantiation checked_connector",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_record_declaration_ordinal_result record",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_record_target_shape_result record",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_record_trait_shape_result record",
        "RecordTargetShapeConnectorMismatch",
        "RecordTraitApplicationShapeConnectorMismatch",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_binder_valid_result record checked_instantiation",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_coherence_match_result declaration_ordinal checked_connector checked_coherence",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_shape_hash",
        "checked_instantiation.target_substitution_output_type_id",
        "DerivedMaterializerConnectorShapeHashPlaceholder",
        "checked_instantiation.substituted_target_type_shape_hash",
        "checked_connector.target_canonical_fingerprint",
        "checked_connector.target_final_shape_hash",
        "checked_connector.trait_application_canonical_fingerprint",
        "checked_connector.trait_application_final_shape_hash",
    ],
    "result function must validate instantiation, connector, coherence, bound solving status, record declaration/shapes, binder identity, coherence match, and derived bridge hash in order",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_materializer_connector_evidence_shape_hash"),
    [
        "evidence.type_id",
        "evidence.concrete_target_type_id",
        "evidence.module_fingerprint",
        "evidence.target_final_shape_hash",
        "evidence.trait_application_final_shape_hash",
        "evidence.substitution_shape_hash",
    ],
    "accepted evidence root recomputation must use typed evidence fields, including concrete target TypeId, substitution root, and final target/trait shapes",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_materializer_connector_shape_hash"),
    [
        "concrete_target_type_id",
        "selfhost_type_id_index concrete_target_type_id",
        "record.module_fingerprint",
    ],
    "producer root hash must bind the concrete target TypeId that the materializer later uses as candidate lookup key",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_materializer_connector_input_table_push"),
    [
        "let records %Vec SelfhostMemoTraitPublicImplGenericMaterializerConnectorInput field::get table \"records\"",
        "v::push records input",
        "ConnectorInputTablePushFailed",
    ],
    "connector input table push must store only connector inputs; final evidence is derived later inside the materializer",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_materializer_connector_evidence_recheck_result"),
    [
        "evidence.schema_version",
        "ConnectorSchemaPlaceholder",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_schema_version",
        "ConcreteTargetTypeIdInvalid",
        "ConnectorShapeHashPlaceholder",
        "ConnectorSubstitutionShapeHashMismatch",
        "CoherenceShapeHashPlaceholder",
        "ConnectorTargetFinalShapeHashPlaceholder",
        "ConnectorTraitApplicationFinalShapeHashPlaceholder",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_evidence_shape_hash evidence",
        "MaterializerConnectorShapeHashMismatch",
        "Result::Ok evidence",
    ],
    "connector evidence must be defensively rechecked after connector_result, while provenance comes from the materializer-side connector_result call",
);
assert.match(
    source,
    /summary\.bound_unsolved|bound_unsolved/,
    "stage0 smoke must expose an unsolved-bound rejection case",
);
assert.match(
    source,
    /summary\.bound_schema_mismatch|bound_schema_mismatch/,
    "stage0 smoke must expose a bound-solving evidence schema mismatch rejection case",
);
assert.doesNotMatch(
    contractSource,
    proseCapPattern,
    "source policy must not add line-count or doc-comment-length caps",
);

console.log("selfhost memo trait public impl generic materializer connector contract ok");
