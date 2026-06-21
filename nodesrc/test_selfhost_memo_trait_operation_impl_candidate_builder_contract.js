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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_impl_candidate_builder.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_impl_candidate_builder",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "impl candidate builder must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("actual public impl materializer が作る予定の typed record table") &&
        source.includes("method body fact boundary、body check resolver、operation impl table"),
    "docs must place the builder after typed public impl materialization and before operation impl table consumption",
);
assert.ok(
    source.includes("旧 API では Eq / Hash の method body fact を `memo_trait_operation_method_body_fact_orchestrator`") &&
        source.includes("proof-aware API では caller が渡す `SelfhostMemoTraitOperationPrivateEffectNoEscapeProofTable`") &&
        source.includes("MethodBodyFact が body root identity を失う前に `PrivateState` / `PrivateCache` の no-escape proof を適用します"),
    "docs must keep the old method fact orchestrator path and describe the proof-aware private-effect gate path",
);
assert.ok(
    source.includes("Drop operation が input に現れた場合は、未証明の `Unknown` evidence を作らず") &&
        source.includes("`DropOperationUnsupportedUntilResourceProof` で明示的に拒否します") &&
        source.includes("Drop proof がまだ接続されていない Phase 1 では Drop input を candidate 化しません"),
    "docs must reject Drop inputs explicitly until Resource-backed Drop proof is connected",
);
assert.ok(
    source.includes("同じ `SelfhostTypeId` と operation kind の candidate が複数来た場合は first-wins にせず") &&
        source.includes("`CandidateDuplicate` として拒否"),
    "docs must reject first-wins duplicate candidate handling",
);
assert.ok(
    source.includes("`body_module_fingerprint` は、1 回の呼び出しで渡す HIR root 群が同一 module origin に属することを caller が示す typed identity") &&
        source.includes("public materializer が接続される段階では module ごとに分割するか、単一 fingerprint であることを検証してから呼び出します"),
    "docs must keep proof-aware builder calls scoped to a caller-validated single module fingerprint",
);
assert.ok(
    source.includes("proof-aware API は caller から受け取った private-effect proof table だけを消費し") &&
        source.includes("operation evidence、aggregate proof、memo_call backend request evidence、artifact policy hash、Resource proof production は作りません"),
    "docs must allow caller-supplied private-effect proofs without producing operation evidence, backend request evidence, artifacts, or Resource proofs",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_impl_candidate_builder/,
    "impl candidate builder must remain facade-private until full operation orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_impl_candidate_builder/,
    "checker-layer impl candidate builder must not be registered in the ty source list",
);
assertOrdered(
    source,
    [
        "#import \"./memo_trait_operation_body_check_resolver\" as *",
        "#import \"./memo_trait_operation_classifier\" as *",
        "#import \"./memo_trait_operation_drop_impl_resolver\" as *",
        "#import \"./memo_trait_operation_impl_table\" as *",
        "#import \"./memo_trait_operation_method_body_fact_input_scan\" as *",
        "#import \"./memo_trait_operation_method_body_fact_orchestrator\" as *",
        "#import \"./memo_trait_operation_method_body_resolver\" as *",
        "#import \"./memo_trait_operation_private_effect_no_escape_gate\" as *",
        "#import \"./memo_trait_operation_purity_gate\" as *",
        "#import \"./memo_trait_public_impl_header\" as *",
    ],
    "builder imports must follow checker-layer dependency direction through existing resolver, table, purity, and header boundaries",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key)/,
    "builder must not import Resource IR, backend, proof store, artifact reader, serializer, preseed, decoded proof, payload reader, or canonical-key layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationImplCandidateBuilderInput:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "impl_header %SelfhostMemoTraitPublicImplHeaderInput",
        "trait_application %SelfhostMemoTraitOperationTraitApplicationInput",
        "resolved_type_shape_hash %Option i32",
        "method_body_root %Option SelfhostHirExprId",
        "fuel %i32",
    ],
    "builder input payload must carry typed impl header, classifier input, resolved type shape, optional method root, and traversal fuel",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationImplCandidateBuilderErrorKind:",
        "HirSetupFailed %StdErrorKind",
        "InputTableAllocFailed %StdErrorKind",
        "SourcePushFailed %StdErrorKind",
        "MethodScanTableAllocFailed %StdErrorKind",
        "SourceReadFailed %i32",
        "MethodScanRecordPushFailed %SelfhostMemoTraitOperationMethodBodyFactInputScanErrorKind",
        "MethodFactBuildRejected %SelfhostMemoTraitOperationMethodBodyFactOrchestratorErrorKind",
        "PrivateEffectNoEscapeGateRejected %SelfhostMemoTraitOperationPrivateEffectNoEscapeGateErrorKind",
        "DropTableAllocFailed %StdErrorKind",
        "DropOperationUnsupportedUntilResourceProof %SelfhostMemoTraitOperationImplCandidateBuilderIndexedOperation",
        "OutputTableAllocFailed %StdErrorKind",
        "BodyCheckRejected %SelfhostMemoTraitOperationBodyCheckResolverErrorKind",
        "CandidateRejected %SelfhostMemoTraitOperationPurityGateErrorKind",
        "CandidateDuplicate",
        "CandidateLookupRejected %SelfhostMemoTraitOperationImplTableErrorKind",
        "CandidatePushRejected %SelfhostMemoTraitOperationImplTableErrorKind",
    ],
    "builder errors must distinguish setup, scan, fact build, Drop unsupported, resolver, purity, duplicate, and push failures",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_method_facts_result"),
    [
        "selfhost_memo_trait_operation_method_body_fact_input_scan_record_table_new",
        "selfhost_memo_trait_operation_impl_candidate_builder_scan_loop scan0 source 0",
        "selfhost_memo_trait_operation_impl_candidate_builder_method_facts_from_scan module scan",
        "MethodScanTableAllocFailed alloc_error",
    ],
    "builder must create a method scan table and delegate fact construction to the existing orchestrator",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_method_facts_from_scan"),
    [
        "selfhost_memo_trait_operation_method_body_fact_table_from_scan_records_result module &scan",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_record_table_free scan",
        "MethodFactBuildRejected build_error",
    ],
    "builder must close the temporary scan owner on both success and fact-build rejection",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_method_facts_with_private_effect_proofs_result"),
    [
        "selfhost_memo_trait_operation_method_body_fact_input_scan_record_table_new",
        "selfhost_memo_trait_operation_impl_candidate_builder_scan_loop scan0 source 0",
        "selfhost_memo_trait_operation_impl_candidate_builder_method_facts_from_scan_with_private_effect_proofs module body_module_fingerprint scan proofs",
        "MethodScanTableAllocFailed alloc_error",
    ],
    "proof-aware builder path must create a scan table and delegate fact construction to the private-effect no-escape gate",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_method_facts_from_scan_with_private_effect_proofs"),
    [
        "selfhost_memo_trait_operation_private_effect_no_escape_table_from_scan_records_result module body_module_fingerprint &scan proofs",
        "selfhost_memo_trait_operation_method_body_fact_input_scan_record_table_free scan",
        "PrivateEffectNoEscapeGateRejected build_error",
    ],
    "proof-aware method fact path must close the temporary scan owner and preserve private-effect gate errors",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_drop_preflight_result"),
    [
        "SelfhostMemoTraitOperationEvidenceKind::Drop:",
        "SelfhostMemoTraitOperationImplCandidateBuilderIndexedOperation index record.operation",
        "DropOperationUnsupportedUntilResourceProof indexed",
        "SelfhostMemoTraitOperationEvidenceKind::Copy:",
        "selfhost_memo_trait_operation_impl_candidate_builder_drop_preflight_result source add index 1",
        "SourceReadFailed index",
    ],
    "Drop preflight must reject Drop records before method fact scan regardless of method root shape",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_duplicate_probe_result"),
    [
        "selfhost_memo_trait_operation_impl_candidate_for_type_operation_result output record.type_id record.operation",
        "Result::Ok _existing:",
        "CandidateDuplicate",
        "SelfhostMemoTraitOperationImplTableErrorKind::CandidateMissing:",
        "Result::Ok unit",
    ],
    "builder must reject duplicates before pushing instead of relying on downstream first-wins behavior",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_append_candidate_result"),
    [
        "selfhost_memo_trait_operation_body_check_resolve_result SelfhostMemoTraitOperationMethodBodySurfaceState::Complete method_table SelfhostMemoTraitOperationDropImplSurfaceState::Unknown drop_table record.type_id record.operation",
        "selfhost_memo_trait_operation_impl_candidate_from_checks_result record.type_id record.operation record.impl_header record.trait_application record.resolved_type_shape_hash checks.method_body checks.drop_impl",
        "selfhost_memo_trait_operation_impl_table_push output candidate",
        "CandidateRejected candidate_error",
        "BodyCheckRejected body_error",
    ],
    "builder must use complete method surface, purity gate candidate construction, and impl-table push without producing downstream evidence records",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_append_candidate_result"),
    [
        "Result::Err candidate_error:",
        "selfhost_memo_trait_operation_impl_table_free output",
        "CandidateRejected candidate_error",
        "Result::Err body_error:",
        "selfhost_memo_trait_operation_impl_table_free output",
        "BodyCheckRejected body_error",
        "Result::Err duplicate_error:",
        "selfhost_memo_trait_operation_impl_table_free output",
        "Result::Err duplicate_error",
    ],
    "append_candidate_result must close the output owner before returning duplicate, body-check, or purity-gate rejections",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_append_candidate_result"),
    /DropImplSurfaceState::Complete|SelfhostMemoTraitOperationDropEvidence::NoDropRequired/,
    "candidate append path must not turn missing Drop proof into NoDropRequired",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_output_loop"),
    [
        "SelfhostMemoTraitOperationEvidenceKind::Drop:",
        "selfhost_memo_trait_operation_impl_table_free output",
        "SelfhostMemoTraitOperationImplCandidateBuilderIndexedOperation index record.operation",
        "DropOperationUnsupportedUntilResourceProof indexed",
        "SelfhostMemoTraitOperationEvidenceKind::Copy:",
        "selfhost_memo_trait_operation_impl_candidate_builder_append_candidate_result output method_table drop_table record",
    ],
    "output loop must reject Drop inputs before append_candidate can create a Drop candidate",
);
assert.doesNotMatch(
    code,
    /SelfhostMemoTraitOperationEvidenceProducerInput|selfhost_memo_trait_operation_impl_candidate_producer_input_result|selfhost_memo_trait_operation_impl_record_for_type_operation_result|selfhost_memo_trait_operation_evidence_producer_input_new|selfhost_memo_trait_operation_evidence_producer_status_result|selfhost_memo_trait_operation_evidence_producer_record_result|SelfhostMemoTraitAggregateProofStatus|SelfhostMemoTraitOperationEvidenceRecord/,
    "builder must not produce producer input, evidence record, or aggregate proof status through direct or impl-table producer APIs",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_table_from_builder_inputs_result"),
    [
        "selfhost_memo_trait_operation_impl_candidate_builder_drop_preflight_result source 0",
        "selfhost_memo_trait_operation_impl_candidate_builder_method_facts_result module source",
        "selfhost_memo_trait_operation_drop_impl_table_new",
        "selfhost_memo_trait_operation_impl_candidate_builder_from_tables_result method_table drop_table source",
        "selfhost_memo_trait_operation_method_body_table_free method_table",
        "DropTableAllocFailed drop_alloc",
    ],
    "public builder entry must build method facts first, allocate a Drop table only as a borrowed resolver argument, and clean up method facts on Drop allocation failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_table_from_builder_inputs_with_private_effect_proofs_result"),
    [
        "selfhost_memo_trait_operation_impl_candidate_builder_drop_preflight_result source 0",
        "selfhost_memo_trait_operation_impl_candidate_builder_method_facts_with_private_effect_proofs_result module body_module_fingerprint source proofs",
        "selfhost_memo_trait_operation_drop_impl_table_new",
        "selfhost_memo_trait_operation_impl_candidate_builder_from_tables_result method_table drop_table source",
        "selfhost_memo_trait_operation_method_body_table_free method_table",
        "DropTableAllocFailed drop_alloc",
    ],
    "proof-aware public builder entry must reuse Drop preflight/output construction and only replace method fact construction",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_impl_candidate_builder_error_kind_eq"),
    [
        "HirSetupFailed a_setup:",
        "InputTableAllocFailed a_alloc:",
        "SourcePushFailed a_push:",
        "MethodScanTableAllocFailed a_scan_alloc:",
        "SourceReadFailed a_read:",
        "MethodScanRecordPushFailed a_scan_push:",
        "MethodFactBuildRejected a_fact:",
        "PrivateEffectNoEscapeGateRejected a_private_effect:",
        "DropTableAllocFailed a_drop_alloc:",
        "OutputTableAllocFailed a_output_alloc:",
        "BodyCheckRejected a_body:",
        "CandidateRejected a_candidate:",
        "CandidateDuplicate:",
        "CandidateLookupRejected a_lookup:",
        "CandidatePushRejected a_push_candidate:",
        "DropOperationUnsupportedUntilResourceProof a_drop_unsupported:",
    ],
    "builder error equality must cover every variant explicitly and compare nested payloads",
);
assert.ok(
    source.includes("wildcard arm は使いません。error variant が増えた場合はこの equality 境界を明示的に更新します"),
    "builder error equality docs must explicitly forbid wildcard arms",
);
assertOrdered(
    source,
    [
        "selfhost_memo_trait_operation_impl_candidate_builder_stage0",
        "SelfhostMemoTraitOperationMethodBodyEvidence::NotRequired",
        "SelfhostMemoTraitOperationMethodBodyEvidence::Pure",
        "SelfhostMemoTraitOperationMethodBodyEvidence::Impure",
        "selfhost_memo_trait_operation_impl_candidate_builder_scan_required_error_result_eq",
        "selfhost_memo_trait_operation_impl_candidate_builder_duplicate_error_result_eq",
        "selfhost_memo_trait_operation_impl_candidate_builder_drop_unsupported_error_result_eq",
        "summary.private_effect_proven",
        "SelfhostMemoTraitOperationMethodBodyEvidence::Unknown",
        "selfhost_memo_trait_operation_impl_candidate_builder_private_gate_error_result_eq",
    ],
    "stage0 must exercise accepted candidate method evidence, scan-missing rejection, duplicate rejection, Drop unsupported rejection, and proof-aware private-effect behavior",
);
assert.doesNotMatch(
    code,
    /source_text|lexeme|display_name|diagnostic|module_path|file_path|path_suffix|method_name|trait_name|payload_hash/,
    "builder code must not use source text, lexemes, display names, diagnostics, module paths, names, or flattened payload hash as authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "builder policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation impl candidate builder contract passed");
