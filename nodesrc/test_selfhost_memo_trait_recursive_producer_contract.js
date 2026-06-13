#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    TY_ROOT_REEXPORT_FILES,
    TY_SPLIT_FILES,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_recursive_producer.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const producerSource = readRepoFile(repoRoot, "stdlib/neplg2/core/ty/ty/memo_trait_producer.nepl");
const operationSource = readRepoFile(repoRoot, "stdlib/neplg2/core/ty/ty/memo_trait_operation_proof.nepl");
const recursiveAggregateSource = readRepoFile(repoRoot, "stdlib/neplg2/core/ty/ty/memo_trait_recursive_aggregate.nepl");
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "ty root re-export file list must include memo_trait_recursive_producer.nepl",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "ty split file list must include memo_trait_recursive_producer.nepl",
);
function assertRecursiveProducerOrder(list, label) {
    const layoutIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_layout.nepl");
    const producerIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_producer.nepl");
    const recursiveIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_recursive_aggregate.nepl");
    const operationIndex = list.indexOf("stdlib/neplg2/core/ty/ty/memo_trait_operation_proof.nepl");
    const recursiveProducerIndex = list.indexOf(relPath);
    assert.ok(
        layoutIndex >= 0
            && producerIndex >= 0
            && recursiveIndex >= 0
            && operationIndex >= 0
            && recursiveProducerIndex >= 0,
        `${label} must include layout, producer, recursive aggregate, operation proof, and recursive producer files`,
    );
    assert.ok(
        layoutIndex < producerIndex
            && producerIndex < recursiveIndex
            && recursiveIndex < operationIndex
            && operationIndex < recursiveProducerIndex,
        `${label} must keep layout before producer before recursive aggregate before operation proof before recursive producer`,
    );
}
assertRecursiveProducerOrder(TY_ROOT_REEXPORT_FILES, "ty root re-export order");
assertRecursiveProducerOrder(TY_SPLIT_FILES, "ty split order");
assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_recursive_producer" as \*$/m,
    "ty facade must re-export the recursive producer split module",
);
assert.match(
    facade,
    /pub #import "\.\/ty\/memo_trait_recursive_aggregate" as \*[\s\S]*pub #import "\.\/ty\/memo_trait_operation_proof" as \*[\s\S]*pub #import "\.\/ty\/memo_trait_recursive_producer" as \*/,
    "ty facade must keep recursive aggregate and operation proof before recursive producer",
);
assert.match(
    source,
    /# ty\/memo_trait_recursive_producer[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "recursive producer documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /traversal 成功 summary は accepted proof ではありません[\s\S]*root の field evidence は `memo_trait_layout\.nepl` の public validator から再取得/,
    "recursive producer docs must state that recursive traversal summary is not accepted proof and root field evidence is reacquired through the layout validator",
);
assert.match(
    source,
    /Copy \/ Drop \/ Eq \/ Hash proof が missing \/ impure \/ unknown の場合[\s\S]*producer gate の typed rejection へ進めます/,
    "recursive producer docs must preserve missing or impure operation proof statuses until the producer gate rejects them",
);
assert.match(
    source,
    /#import "\.\/memo_trait_layout" as \*[\s\S]*#import "\.\/memo_trait_operation_proof" as \*[\s\S]*#import "\.\/memo_trait_producer" as \*[\s\S]*#import "\.\/memo_trait_recursive_aggregate" as \*/,
    "recursive producer must import layout, operation proof, producer, and recursive aggregate boundaries explicitly",
);
for (const [label, lowerSource] of [
    ["producer", producerSource],
    ["operation proof", operationSource],
    ["recursive aggregate", recursiveAggregateSource],
]) {
    assert.doesNotMatch(
        lowerSource,
        /#import "\.\/memo_trait_recursive_producer"|\bmemo_trait_recursive_producer\b/,
        `${label} module must not import or call recursive producer, because the connector must not flow back into lower layers`,
    );
}
assert.doesNotMatch(
    codeOnly,
    /#import "\.\/memo_trait_(?:proof_store|proof_reader|proof_payload_reader|proof_serializer|proof_preseed|proof_stable_map|proof_artifact|proof_index|proof_decoded|artifact_word_codec|canonical_key|canonical_key_payload|canonical_key_payload_codec)"/,
    "recursive producer must not depend on proof store, artifacts, canonical key codecs, readers, serializers, indexes, or preseed modules",
);
assert.doesNotMatch(
    codeOnly,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "recursive producer must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitRecursiveProducerErrorKind:[\s\S]*RecursiveRejected %SelfhostMemoTraitRecursiveAggregateErrorKind[\s\S]*LayoutRejected %SelfhostMemoTraitLayoutEvidenceErrorKind[\s\S]*OperationRejected %SelfhostMemoTraitOperationProofErrorKind[\s\S]*ProducerRejected %SelfhostMemoTraitEvidenceProduceRejectKind/,
    "recursive producer errors must preserve recursive, layout, operation, and producer rejection payloads as typed variants",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitRecursiveProducerStage0Summary:[\s\S]*produced_record[\s\S]*recursive_cycle_rejected[\s\S]*operation_missing_rejected[\s\S]*producer_hazard_rejected/,
    "stage0 summary must expose accepted, recursive rejection, operation-missing producer rejection, and hazard producer rejection paths",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_recursive_producer_record_result_is_accept %fn Result SelfhostMemoTraitEvidenceRecord SelfhostMemoTraitRecursiveProducerErrorKind bool[\s\S]*Result::Ok _record:[\s\S]*true[\s\S]*Result::Err _kind:[\s\S]*false/,
    "recursive producer must provide a result accept helper for its own typed error domain instead of reusing the producer-only helper",
);
assert.match(
    source,
    /wildcard arm は使いません[\s\S]*selfhost_memo_trait_recursive_producer_error_kind_eq[\s\S]*RecursiveRejected a_recursive:[\s\S]*selfhost_memo_trait_recursive_aggregate_error_kind_eq a_recursive b_recursive[\s\S]*LayoutRejected a_layout:[\s\S]*selfhost_memo_trait_layout_error_kind_eq a_layout b_layout[\s\S]*OperationRejected a_operation:[\s\S]*selfhost_memo_trait_operation_proof_error_kind_eq a_operation b_operation[\s\S]*ProducerRejected a_producer:[\s\S]*selfhost_memo_trait_evidence_produce_reject_kind_eq a_producer b_producer/,
    "recursive producer error equality must avoid wildcard arms and compare nested typed payloads",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_producer_aggregate_proof_result[\s\S]*selfhost_memo_trait_recursive_aggregate_result types layout_table type_id max_depth[\s\S]*Result::Ok _summary:[\s\S]*selfhost_memo_trait_layout_evidence_for_type_result layout_table types type_id[\s\S]*selfhost_memo_trait_aggregate_proof_from_operation_table_result types operation_table type_id fields hazard key_result value_result/,
    "aggregate proof construction must run recursive traversal first, reacquire root layout evidence, and then merge operation proof statuses",
);
assert.doesNotMatch(
    codeOnly,
    /_summary\.(?:aggregate_count|field_count|max_depth_seen)|summary\.(?:aggregate_count|field_count|max_depth_seen)/,
    "recursive producer must not use traversal summary counters as producer field range evidence",
);
assert.match(
    source,
    /Result::Err operation_error:[\s\S]*OperationRejected operation_error[\s\S]*Result::Err layout_error:[\s\S]*LayoutRejected layout_error[\s\S]*Result::Err recursive_error:[\s\S]*RecursiveRejected recursive_error/,
    "aggregate proof construction must map recursive, layout, and operation failures to separate typed errors",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_producer_record_result[\s\S]*selfhost_memo_trait_recursive_producer_aggregate_proof_result[\s\S]*Result::Ok proof:[\s\S]*selfhost_memo_trait_aggregate_proof_to_record types proof[\s\S]*Result::Err producer_error:[\s\S]*ProducerRejected producer_error/,
    "record boundary must call the existing producer gate and preserve producer rejection payloads",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostMemoTraitEvidenceRecord\s+proof|SelfhostMemoTraitEvidenceRecord\s+record\s*SelfhostMemoTraitEvidenceRecord|selfhost_memo_trait_evidence_record_new/,
    "recursive producer must not build consumer evidence records directly and must use the producer gate",
);
assert.doesNotMatch(
    codeOnly,
    /selfhost_memo_trait_evidence_table_push/,
    "recursive producer must not insert accepted records into the consumer evidence table directly",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_producer_stage0_after_tables[\s\S]*produced_record[\s\S]*recursive_cycle_rejected[\s\S]*operation_missing_rejected[\s\S]*producer_hazard_rejected/,
    "stage0 must exercise accepted, recursive rejected, operation missing, and hazard rejected paths through the public boundary",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_producer_record_result &arena &layout_table &proven_table root_id 4 SelfhostMemoTraitAggregateHazardEvidence::NoHazard[\s\S]*selfhost_memo_trait_recursive_producer_record_result &arena &layout_table &proven_table cycle_id 4 SelfhostMemoTraitAggregateHazardEvidence::NoHazard[\s\S]*selfhost_memo_trait_recursive_producer_record_result &arena &layout_table &empty_table root_id 4 SelfhostMemoTraitAggregateHazardEvidence::NoHazard[\s\S]*selfhost_memo_trait_recursive_producer_record_result &arena &layout_table &proven_table root_id 4 SelfhostMemoTraitAggregateHazardEvidence::ExternalHandle/,
    "stage0 must route all representative paths through the public recursive producer record boundary",
);
assert.match(
    source,
    /selfhost_memo_trait_recursive_producer_record_result_is_accept summary\.produced_record[\s\S]*SelfhostMemoTraitRecursiveProducerErrorKind::RecursiveRejected SelfhostMemoTraitRecursiveAggregateErrorKind::CycleDetected[\s\S]*let cycle_actual %SelfhostMemoTraitRecursiveProducerErrorKind unwrap_err summary\.recursive_cycle_rejected[\s\S]*SelfhostMemoTraitRecursiveProducerErrorKind::ProducerRejected SelfhostMemoTraitEvidenceProduceRejectKind::CopyProofMissing[\s\S]*let operation_actual %SelfhostMemoTraitRecursiveProducerErrorKind unwrap_err summary\.operation_missing_rejected[\s\S]*SelfhostMemoTraitRecursiveProducerErrorKind::ProducerRejected SelfhostMemoTraitEvidenceProduceRejectKind::ExternalHandle[\s\S]*let hazard_actual %SelfhostMemoTraitRecursiveProducerErrorKind unwrap_err summary\.producer_hazard_rejected/,
    "doctest must check recursive cycle, missing operation proof, and producer hazard rejection payloads",
);
assert.doesNotMatch(
    source,
    /\(unwrap_err/,
    "recursive producer doctest must use current NEPLg2.1 syntax and must not reintroduce parenthesized unwrap expressions",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "recursive producer code must not use source text, spans, paths, display names, diagnostics, or lexemes as proof authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "recursive producer policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait recursive producer contract passed");
