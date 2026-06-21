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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer.nepl";
const recordRelPath =
    "stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer_record.nepl";
const connectorRelPath =
    "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_materializer_connector.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const recordSource = read(recordRelPath);
const connectorSource = read(connectorRelPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_public_impl_materializer",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "public impl materializer must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("operation kind は caller supplied field ではなく、trusted operation classifier が返す shape-bound evidence から採用します") &&
        source.includes("method body fact、Drop proof、operation evidence record、aggregate proof status を作りません"),
    "docs must place the materializer between typed public impl records and the candidate builder without producing proof artifacts",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、method name string、trait name string") &&
        source.includes("operation や HIR root を推測しません"),
    "docs must reject source-derived authority for operation or method root materialization",
);
assert.ok(
        source.includes("Drop record が現れた場合、classifier で shape を確認したうえで builder input に写し") &&
        source.includes("`DropOperationUnsupportedUntilResourceProof`") &&
        source.includes("detailed generic binder evidence を持つ record は header evidence までは検査します") &&
        source.includes("`GenericImplInstantiationUnsupported`") &&
        source.includes("connector input table を受け取る generic-aware API") &&
        source.includes("matching input を見つけた後に `memo_trait_public_impl_generic_materializer_connector` の `connector_result` を実行") &&
        source.includes("concrete target TypeId と final target / trait shape") &&
        source.includes("private-effect proof-aware API") &&
        source.includes("全 record の `module_fingerprint` が call-level `body_module_fingerprint` と一致することを検査します"),
    "docs must route Drop through the existing fail-closed builder boundary, describe generic connector paths, and validate proof-aware body module fingerprints",
);
assertOrdered(
    recordSource,
    [
        "# check/module/memo_trait_operation_public_impl_materializer_record",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
    ],
    "shared public impl materializer record module must document purpose, contract, current status, and complexity",
);
assert.ok(
    recordSource.includes("両 module が互いを import すると dependency が cycle になるため") &&
        recordSource.includes("record table owner、connector evidence、classifier、candidate builder、Resource proof は扱いません"),
    "shared record docs must explain that the split exists to keep materializer and connector dependencies acyclic",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_public_impl_materializer/,
    "public impl materializer must remain facade-private until full operation orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_public_impl_materializer/,
    "checker-layer public impl materializer must not be registered in the ty source list",
);
assertOrdered(
    source,
    [
        "#import \"./memo_trait_operation_classifier\" as *",
        "#import \"./memo_trait_operation_impl_candidate_builder\" as *",
        "#import \"./memo_trait_operation_impl_table\" as *",
        "#import \"./memo_trait_operation_private_effect_no_escape_gate\" as *",
        "#import \"./memo_trait_operation_public_impl_materializer_record\" as *",
        "#import \"./memo_trait_public_impl_generic_binder\" as *",
        "#import \"./memo_trait_public_impl_generic_materializer_connector\" as *",
        "#import \"./memo_trait_public_impl_header\" as *",
    ],
    "materializer imports must go through classifier, candidate builder, impl table lookup helpers, private-effect proof table type, generic binder evidence, generic materializer connector evidence, and public impl header boundaries",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop_impl_resolver)/,
    "materializer must not import Resource IR, backend, proof store, canonical-key, producer, purity, body-check, method-body, or Drop resolver layers directly",
);
assert.doesNotMatch(
    source,
    /^pub struct SelfhostMemoTraitOperationPublicImplMaterializerRecord:/m,
    "materializer body must not own the shared record type after the connector accepted path split",
);
assert.match(
    recordSource,
    /^pub fn selfhost_memo_trait_operation_public_impl_materializer_record_new /m,
    "shared record module must own the typed constructor so connector fixtures do not import the materializer body",
);
assert.doesNotMatch(
    source,
    /^pub fn selfhost_memo_trait_operation_public_impl_materializer_record_new /m,
    "materializer body must not keep a duplicate record constructor after the shared record split",
);
assertOrdered(
    recordSource,
    [
        "pub struct SelfhostMemoTraitOperationPublicImplMaterializerRecord:",
        "type_id %SelfhostTypeId",
        "module_fingerprint %i32",
        "declaration_ordinal %Option i32",
        "visibility %SelfhostModuleDeclarationVisibility",
        "impl_kind %SelfhostMemoTraitPublicImplHeaderKind",
        "target_type_shape_hash %Option i32",
        "trait_source %SelfhostMemoTraitOperationSourceIdentity",
        "trait_type_argument_count %i32",
        "trait_application_shape_hash %Option i32",
        "type_parameter_count %i32",
        "type_parameter_bound_count %i32",
        "generic_binder_evidence %SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence",
        "method_body_root %Option SelfhostHirExprId",
        "fuel %i32",
    ],
    "shared materializer record must carry typed public impl header fields, generic binder evidence mode, classifier fields, method root, and fuel",
);
assert.doesNotMatch(
    source,
    /pub struct SelfhostMemoTraitOperationPublicImplMaterializerGenericConnectorTable:|selfhost_memo_trait_operation_public_impl_materializer_generic_connector_table_push|SelfhostMemoTraitPublicImplGenericMaterializerConnectorEvidenceTable/,
    "materializer must not own, populate, or accept a raw generic connector evidence table",
);
assertOrdered(
    connectorSource,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericMaterializerConnectorInput:",
        "declaration_ordinal %i32",
        "bound_status %SelfhostMemoTraitPublicImplGenericBoundSolvingStatus",
        "instantiation %SelfhostMemoTraitPublicImplGenericInstantiationEvidence",
        "projection_connector %SelfhostMemoTraitPublicImplGenericInstantiationProjectionConnectorEvidence",
        "coherence %SelfhostMemoTraitPublicImplGenericConcreteCoherenceEvidence",
        "pub struct SelfhostMemoTraitPublicImplGenericMaterializerConnectorInputTable:",
        "records %Vec SelfhostMemoTraitPublicImplGenericMaterializerConnectorInput",
    ],
    "connector module must own the generic connector input table without storing final evidence",
);
assert.doesNotMatch(
    connectorSource,
    /SelfhostMemoTraitPublicImplGenericMaterializerConnectorEvidenceTable|selfhost_memo_trait_public_impl_generic_materializer_connector_evidence_table_push_result/,
    "connector module must not expose a final generic connector evidence table because materializer must run connector_result itself",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationPublicImplMaterializerErrorKind:",
        "RecordTableAllocFailed %StdErrorKind",
        "RecordPushFailed %StdErrorKind",
        "BuilderInputTableAllocFailed %StdErrorKind",
        "SourceReadFailed %i32",
        "GenericConnectorReadFailed %i32",
        "GenericConnectorMissing %i32",
        "GenericConnectorDuplicate %i32",
        "HeaderRejected %SelfhostMemoTraitPublicImplHeaderErrorKind",
        "GenericImplInstantiationUnsupported",
        "GenericConnectorRejected %SelfhostMemoTraitPublicImplGenericMaterializerConnectorErrorKind",
        "ClassifierRejected %SelfhostMemoTraitOperationClassifierErrorKind",
        "BodyModuleFingerprintPlaceholder",
        "BodyModuleFingerprintMismatch %SelfhostMemoTraitOperationPublicImplMaterializerBodyModuleFingerprintMismatch",
        "BuilderInputPushRejected %SelfhostMemoTraitOperationImplCandidateBuilderErrorKind",
        "CandidateBuilderRejected %SelfhostMemoTraitOperationImplCandidateBuilderErrorKind",
    ],
    "materializer errors must distinguish setup, connector table, read, header, generic-instantiation, connector rejection, classifier, proof-aware fingerprint validation, builder-input push, and candidate-builder failures",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationPublicImplMaterializerBodyModuleFingerprintMismatch:",
        "record_module_fingerprint %i32",
        "body_module_fingerprint %i32",
    ],
    "materializer must keep proof-aware fingerprint mismatch as a typed payload",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_record_to_builder_input_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_header_input record",
        "selfhost_memo_trait_operation_public_impl_materializer_trait_application_input record",
        "selfhost_memo_trait_public_impl_header_evidence_result impl_header",
        "SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence::Monomorphic:",
        "selfhost_memo_trait_operation_classifier_evidence_result trait_application",
        "Result::Ok classifier:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_new record.type_id classifier.operation impl_header trait_application record.target_type_shape_hash record.method_body_root record.fuel",
        "SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence::Detailed _binder_evidence:",
        "GenericImplInstantiationUnsupported",
        "HeaderRejected header_error",
    ],
    "record materialization must validate the public impl header, stop detailed generic records before candidate construction, and derive operation from classifier evidence",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_record_to_builder_input_result"),
    /record\.trait_source\.operation|SelfhostMemoTraitOperationEvidenceKind::Copy|SelfhostMemoTraitOperationEvidenceKind::Drop|SelfhostMemoTraitOperationEvidenceKind::Eq|SelfhostMemoTraitOperationEvidenceKind::Hash/,
    "record_to_builder_input_result must not directly trust operation kind from source identity or hard-code operation variants",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_generic_connector_for_record_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_record_declaration_ordinal_result record",
        "selfhost_memo_trait_operation_public_impl_materializer_generic_connector_lookup_loop connectors declaration_ordinal 0 none",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_result record input.bound_status input.instantiation input.projection_connector input.coherence",
        "selfhost_memo_trait_public_impl_generic_materializer_connector_evidence_recheck_result evidence",
        "not selfhost_type_id_eq record.type_id checked.type_id",
        "GenericConnectorRejected connector_error",
        "not eq record.module_fingerprint checked.module_fingerprint",
        "selfhost_memo_trait_operation_public_impl_materializer_shape_option_result record.target_type_shape_hash",
        "RecordTargetShapeConnectorMismatch mismatch",
        "selfhost_memo_trait_operation_public_impl_materializer_shape_option_result record.trait_application_shape_hash",
        "RecordTraitApplicationShapeConnectorMismatch mismatch",
        "Result::Ok checked",
    ],
    "generic connector lookup must require declaration ordinal, run connector_result in the materializer, recheck generated evidence, and compare record identity plus pre-substitution target and trait shapes before accepting a connector",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_generic_record_to_builder_input_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_generic_connector_for_record_result record connectors",
        "selfhost_memo_trait_operation_public_impl_materializer_generic_header_input record connector",
        "selfhost_memo_trait_operation_public_impl_materializer_generic_trait_application_input record connector",
        "selfhost_memo_trait_public_impl_header_evidence_result impl_header",
        "selfhost_memo_trait_operation_classifier_evidence_result trait_application",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_new connector.concrete_target_type_id classifier.operation impl_header trait_application some connector.target_final_shape_hash record.method_body_root record.fuel",
    ],
    "generic record conversion must use connector-rechecked concrete target TypeId and final target shape when building candidate input",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_record_to_builder_input_with_generics_result"),
    [
        "SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence::Monomorphic:",
        "selfhost_memo_trait_operation_public_impl_materializer_record_to_builder_input_result record",
        "SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence::Detailed _binder_evidence:",
        "selfhost_memo_trait_operation_public_impl_materializer_generic_record_to_builder_input_result record connectors",
    ],
    "generic-aware record conversion must keep the old monomorphic path and require connector-backed conversion only for Detailed generic records",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_builder_input_loop"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_record_to_builder_input_result record",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_push builder builder_input",
        "BuilderInputPushRejected push_error",
        "Result::Err e:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder",
        "Result::Err e",
        "Option::None:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder",
        "SourceReadFailed index",
    ],
    "builder_input_loop must clean up the temporary builder input table on classifier and read failures while relying on push boundary cleanup for push failures",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_builder_inputs_from_records_result source",
        "selfhost_memo_trait_operation_impl_candidate_table_from_builder_inputs_result module &builder_inputs",
        "Result::Ok candidates:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder_inputs",
        "Result::Ok candidates",
        "Result::Err builder_error:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder_inputs",
        "CandidateBuilderRejected builder_error",
    ],
    "candidate table entry must close the temporary builder input owner after both builder success and builder rejection",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_validate_body_module_fingerprint_result"),
    [
        "eq body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "selfhost_memo_trait_operation_public_impl_materializer_validate_body_module_fingerprint_loop source body_module_fingerprint 0",
    ],
    "proof-aware materializer entry must reject placeholder body module fingerprints before candidate construction",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_validate_body_module_fingerprint_loop"),
    [
        "field::get_ref source \"records\"",
        "v::get records index",
        "eq record.module_fingerprint body_module_fingerprint",
        "selfhost_memo_trait_operation_public_impl_materializer_validate_body_module_fingerprint_loop source body_module_fingerprint add index 1",
        "BodyModuleFingerprintMismatch mismatch",
        "SourceReadFailed index",
    ],
    "proof-aware materializer entry must validate every source record against the call-level body module fingerprint",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_with_private_effect_proofs_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_validate_body_module_fingerprint_result source body_module_fingerprint",
        "selfhost_memo_trait_operation_public_impl_materializer_builder_inputs_from_records_result source",
        "selfhost_memo_trait_operation_impl_candidate_table_from_builder_inputs_with_private_effect_proofs_result module body_module_fingerprint &builder_inputs proofs",
        "Result::Ok candidates:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder_inputs",
        "Result::Ok candidates",
        "Result::Err builder_error:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder_inputs",
        "CandidateBuilderRejected builder_error",
    ],
    "proof-aware candidate table entry must validate fingerprint, reuse old builder-input construction, call only the proof-aware candidate builder API, and close the temporary builder input owner",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_with_generics_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_builder_inputs_from_records_with_generics_result source connectors",
        "selfhost_memo_trait_operation_impl_candidate_table_from_builder_inputs_result module &builder_inputs",
        "Result::Ok candidates:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder_inputs",
        "Result::Ok candidates",
        "Result::Err builder_error:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder_inputs",
        "CandidateBuilderRejected builder_error",
    ],
    "generic-aware candidate table entry must use the connector-backed builder input path and close the temporary builder input owner on success and builder rejection",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_with_generics_and_private_effect_proofs_result"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_validate_body_module_fingerprint_result source body_module_fingerprint",
        "selfhost_memo_trait_operation_public_impl_materializer_builder_inputs_from_records_with_generics_result source connectors",
        "selfhost_memo_trait_operation_impl_candidate_table_from_builder_inputs_with_private_effect_proofs_result module body_module_fingerprint &builder_inputs proofs",
        "Result::Ok candidates:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder_inputs",
        "Result::Ok candidates",
        "Result::Err builder_error:",
        "selfhost_memo_trait_operation_impl_candidate_builder_input_table_free builder_inputs",
        "CandidateBuilderRejected builder_error",
    ],
    "generic+proof candidate table entry must use the generic builder-input path, validate fingerprint, call only the proof-aware candidate builder API, and close the temporary builder input owner",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_with_private_effect_proofs_result"),
    /selfhost_memo_trait_operation_impl_candidate_table_from_builder_inputs_result\b/,
    "proof-aware materializer entry must not call the old candidate builder API",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_with_generics_and_private_effect_proofs_result"),
    /selfhost_memo_trait_operation_impl_candidate_table_from_builder_inputs_result\b/,
    "generic+proof materializer entry must not call the old candidate builder API",
);
for (const name of [
    "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_with_private_effect_proofs_result",
    "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_with_generics_and_private_effect_proofs_result",
]) {
    const block = functionBlock(source, name);
    assert.doesNotMatch(
        block,
        /SelfhostMemoTraitOperationPrivateEffectNoEscapeProofKey|selfhost_memo_trait_operation_private_effect_no_escape_proof_key_new/,
        `${name} must not synthesize private-effect proof keys; proof keys are produced by the private-effect gate`,
    );
    assert.doesNotMatch(
        block,
        /selfhost_memo_trait_operation_private_effect_no_escape_proof_table_free/,
        `${name} must borrow the proof table and must not free it`,
    );
}
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_with_generics_and_private_effect_proofs_result"),
    /selfhost_memo_trait_public_impl_generic_materializer_connector_input_table_free|generic_materializer_connector_input_table_free/,
    "generic+proof materializer entry must borrow the generic connector table and must not free it",
);
assert.doesNotMatch(
    code,
    /selfhost_memo_trait_operation_impl_table_push|SelfhostMemoTraitOperationEvidenceProducerInput|selfhost_memo_trait_operation_impl_candidate_producer_input_result|selfhost_memo_trait_operation_impl_record_for_type_operation_result|selfhost_memo_trait_operation_evidence_producer_input_new|selfhost_memo_trait_operation_evidence_producer_status_result|selfhost_memo_trait_operation_evidence_producer_record_result|SelfhostMemoTraitAggregateProofStatus|SelfhostMemoTraitOperationEvidenceRecord/,
    "materializer must not push candidates directly, produce evidence records, or aggregate proof status",
);
assertOrdered(
    source,
    [
        "selfhost_memo_trait_operation_public_impl_materializer_stage0",
        "selfhost_memo_trait_operation_public_impl_materializer_accepted_len_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_accepted_operation_present_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_accepted_method_evidence_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_classifier_rejected_result_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_candidate_builder_rejected_result_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_private_gate_error_result_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_body_module_fingerprint_mismatch_result_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_drop_unsupported_result_eq",
        "selfhost_memo_trait_operation_public_impl_materializer_generic_unsupported_result_eq",
    ],
    "materializer must expose a stage0 smoke API and typed assertion helpers for accepted, method evidence, classifier rejection, builder rejection, private gate rejection, fingerprint mismatch, Drop unsupported, and generic unsupported paths",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_drop_unsupported_result_eq"),
    [
        "CandidateBuilderRejected builder_error:",
        "let builder_result %Result i32 SelfhostMemoTraitOperationImplCandidateBuilderErrorKind Result::Err builder_error",
        "selfhost_memo_trait_operation_impl_candidate_builder_drop_unsupported_error_result_eq builder_result expected_index",
        "ClassifierRejected _classifier:",
        "false",
    ],
    "Drop unsupported helper must prove materializer-originated Drop records reach the existing builder Drop rejection",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_public_impl_materializer_stage0_with_shapes"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_accepted",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_untrusted",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_duplicate",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_drop_unsupported",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_generic_unsupported",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_private_effect_proven",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_private_effect_missing",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_private_effect_duplicate",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_fingerprint_mismatch",
        "selfhost_memo_trait_operation_public_impl_materializer_stage0_summary_new accepted untrusted_rejected duplicate_rejected drop_unsupported generic_unsupported private_effect_proven private_effect_missing private_effect_duplicate_rejected fingerprint_mismatch",
    ],
    "stage0 must cover accepted records, classifier rejection, duplicate rejection, Drop unsupported routing, generic operation instantiation rejection, proof-aware proven/missing/duplicate, and fingerprint mismatch",
);
assert.doesNotMatch(
    source,
    /maxLine|line count limit|line-count limit|doc comment length cap|documentation length cap|行数制限|コメント量制限|ドキュメントコメント.{0,12}上限/,
    "materializer contract must not introduce line-count or documentation-comment length limits",
);

console.log("selfhost memo trait operation public impl materializer contract ok");
