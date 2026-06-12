#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_index.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");
function codeSliceBetween(startNeedle, endNeedle) {
    const start = source.indexOf(startNeedle);
    const end = source.indexOf(endNeedle);
    assert.notEqual(start, -1, `missing source slice start: ${startNeedle}`);
    assert.notEqual(end, -1, `missing source slice end: ${endNeedle}`);
    assert.ok(start < end, `source slice must be ordered: ${startNeedle}`);
    return source
        .slice(start, end)
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

const producerImplementation = codeSliceBetween(
    "fn selfhost_memo_trait_neplproof_sorted_index_entry_lt",
    "//: selfhost_memo_trait_neplproof_sorted_index_stage0_policy",
);

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_proof_index" as \*$/m,
    "ty facade must re-export the .neplproof sorted index contract module",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_artifact" as \*$/m,
    "sorted index contract must reuse artifact record/index schema and decoded table validation",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "sorted artifact index contract must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /# ty\/memo_trait_proof_index[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[戻\/もど\]り\[値\/ち\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "sorted index contract module documentation must record purpose, contract, return values, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /fingerprint hit は proof acceptance ではありません|fingerprint hit、record payload hash match だけで proof を受理する経路はありません/,
    "sorted index documentation must explicitly limit fingerprint hits to candidate narrowing",
);
assert.match(
    source,
    /serializer \/ preseed writer[\s\S]*producer boundary[\s\S]*既存の table validation と sorted order validation/,
    "sorted index documentation must describe the decoded-record producer boundary and its post-build validation",
);
assert.match(
    source,
    /producer は record 数 n に対して O\(n\^2\)[\s\S]*後続の serialized stable map \/ binary writer/,
    "sorted index producer documentation must state the current O(n^2) implementation and the future faster artifact boundary",
);
assert.match(
    source,
    /source text、span、path suffix、display name、diagnostic text、lexeme[\s\S]*lookup key、sort key、tie-break authority に入りません/,
    "sorted index documentation must exclude source text, spans, paths, names, diagnostics, and lexemes from lookup authority",
);
assert.match(
    source,
    /proof store の stable sidecar index とは責務を混ぜません[\s\S]*artifact record ordinal 候補/,
    "sorted artifact index contract must keep artifact ordinal narrowing separate from proof-store stable index authority",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofIndexCandidateRange:[\s\S]*start_index %i32[\s\S]*candidate_count %i32/,
    "sorted lookup must return a typed candidate range instead of proof payload",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofSortedIndexErrorKind:[\s\S]*HeaderInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*TableValidationRejected %SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*IndexEntryMissing[\s\S]*FingerprintOrderInvalid[\s\S]*RecordOrdinalOrderInvalid[\s\S]*CandidateMissing/,
    "sorted index failures must use typed enum variants and preserve nested validation errors",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofIndexProducerErrorKind:[\s\S]*HeaderInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*AllocFailed %StdErrorKind[\s\S]*PushFailed %StdErrorKind[\s\S]*RecordEntryMissing[\s\S]*IndexEntryMissing[\s\S]*RecordInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*IndexEntryBuildRejected %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*ProducedTableRejected %SelfhostMemoTraitNeplProofIndexValidationErrorKind[\s\S]*ProducedOrderRejected %SelfhostMemoTraitNeplProofSortedIndexErrorKind/,
    "sorted index producer failures must use typed enum variants and preserve nested validation errors",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_sorted_index_build_result[\s\S]*&Vec SelfhostMemoTraitNeplProofRecord[\s\S]*Result Vec SelfhostMemoTraitNeplProofIndexEntry SelfhostMemoTraitNeplProofIndexProducerErrorKind/,
    "sorted index producer API must borrow decoded records and return an owned index vector wrapped in a typed Result",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_sorted_index_entry_from_record_result[\s\S]*selfhost_memo_trait_neplproof_record_key_result[\s\S]*selfhost_memo_trait_neplproof_record_result[\s\S]*selfhost_memo_trait_neplproof_index_entry_result/,
    "sorted index producer must revalidate record keys, record payloads, and produced index entries",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_sorted_index_finish_build[\s\S]*selfhost_memo_trait_neplproof_header_result[\s\S]*selfhost_memo_trait_neplproof_index_table_result[\s\S]*selfhost_memo_trait_neplproof_sorted_index_order_result/,
    "sorted index producer output must pass existing header, decoded table, and sorted-order validation before being returned",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_sorted_index_bubble_back_loop[\s\S]*v::replace[\s\S]*v::replace/,
    "sorted index producer bubble-back helper must swap Copy index entries in place",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_sorted_index_bubble_back_loop[\s\S]*IndexEntryMissing[\s\S]*v::replace[\s\S]*v::replace/,
    "sorted index producer bubble-back defensive reads must classify missing index slots as IndexEntryMissing",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_sorted_index_push_sorted[\s\S]*v::push[\s\S]*selfhost_memo_trait_neplproof_sorted_index_bubble_back_loop/,
    "sorted index producer push helper must run the bounded bubble-back insertion step for Phase 1 ordering",
);
assert.doesNotMatch(
    producerImplementation,
    /selfhost_memo_trait_proof_store_(?:lookup|push|preseed|stable|materialized|new|free)|selfhost_memo_trait_neplproof_decoded_record_batch_append/,
    "sorted index producer implementation must not call proof-store, preseed, or decoded-batch append APIs directly",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_sorted_index_lookup_result[\s\S]*SelfhostMemoTraitNeplProofHeader[\s\S]*&Vec SelfhostMemoTraitNeplProofRecord[\s\S]*&Vec SelfhostMemoTraitNeplProofIndexEntry[\s\S]*SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*Result SelfhostMemoTraitNeplProofIndexCandidateRange SelfhostMemoTraitNeplProofSortedIndexErrorKind/,
    "sorted lookup API must take header, records, indexes, and target fingerprint, and return Result candidate range",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_header_result[\s\S]*selfhost_memo_trait_neplproof_index_table_result[\s\S]*selfhost_memo_trait_neplproof_sorted_index_order_result[\s\S]*selfhost_memo_trait_neplproof_candidate_range_lookup_loop/,
    "sorted lookup must revalidate header/table, then check order, then perform candidate lookup",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_sorted_index_pair_result[\s\S]*selfhost_memo_trait_neplproof_fingerprint_lt current\.canonical_fingerprint previous\.canonical_fingerprint[\s\S]*FingerprintOrderInvalid[\s\S]*ge previous\.record_ordinal current\.record_ordinal[\s\S]*RecordOrdinalOrderInvalid/,
    "sorted index order check must reject decreasing fingerprint order and non-increasing record ordinals inside a fingerprint group",
);
assert.match(
    source,
    /candidate range lookup は lower-bound binary search と collision group count により O\(log m \+ c\)/,
    "candidate lookup documentation must state the lower-bound search complexity",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_candidate_range_lower_bound_loop[\s\S]*ge low high[\s\S]*let mid %i32 add low div_s sub high low 2[\s\S]*selfhost_memo_trait_neplproof_fingerprint_lt entry\.canonical_fingerprint target[\s\S]*add mid 1 high[\s\S]*low mid/,
    "candidate lookup must use a half-open lower-bound binary search before counting the collision group",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_candidate_range_lookup_loop[\s\S]*selfhost_memo_trait_neplproof_candidate_range_lower_bound_loop indexes target 0 n[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_eq entry\.canonical_fingerprint target[\s\S]*selfhost_memo_trait_neplproof_candidate_range_count_loop indexes target start start 0[\s\S]*CandidateMissing/,
    "candidate lookup must return only the contiguous fingerprint range found by lower-bound search",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_sorted_index_error_kind_eq[\s\S]*HeaderInvalid a_kind[\s\S]*selfhost_memo_trait_neplproof_artifact_error_kind_eq a_kind b_kind[\s\S]*TableValidationRejected a_kind[\s\S]*selfhost_memo_trait_neplproof_index_validation_error_kind_eq a_kind b_kind/,
    "sorted index error equality must compare nested typed payloads",
);
assert.match(
    source,
    /stage0[\s\S]*accepted_range[\s\S]*accepted_collision_range[\s\S]*missing_candidate[\s\S]*fingerprint_unsorted[\s\S]*ordinal_unsorted[\s\S]*table_rejected/,
    "stage0 must exercise accepted lookup, accepted fingerprint collision group lookup, missing candidate, fingerprint order error, ordinal order error, and table validation rejection",
);
assert.match(
    source,
    /producer_stage0[\s\S]*built_first_range[\s\S]*built_second_range[\s\S]*built_collision_range[\s\S]*invalid_record/,
    "stage0 producer smoke must exercise unordered records, collision group lookup, and invalid-record rejection",
);
assert.match(
    source,
    /second_record first_record[\s\S]*selfhost_memo_trait_neplproof_sorted_index_stage0_run_producer_lookup[\s\S]*first_fingerprint[\s\S]*second_fingerprint/,
    "stage0 producer smoke must build sorted indexes from intentionally unordered records",
);
assert.match(
    source,
    /expected_collision_range SelfhostMemoTraitNeplProofIndexCandidateRange 0 2[\s\S]*accepted_collision_range[\s\S]*collision_second_index/,
    "stage0 runtime coverage must accept a same-fingerprint collision group as a two-entry candidate range",
);
assert.match(
    source,
    /built_collision_range[\s\S]*expected_collision_range[\s\S]*RecordInvalid SelfhostMemoTraitNeplProofArtifactErrorKind::RecordPayloadHashPlaceholder/,
    "doctest must verify producer collision output and invalid-record rejection",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostCanonicalTypeKeyId|SelfhostTypeId|SelfhostNamedTypeId|SelfhostMemoTraitProofStoreRecord|SelfhostMemoTraitProofStoreStableIndexEntry/,
    "sorted artifact index contract must not store session-local ids or proof-store records/index entries",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "sorted artifact index contract code must not use source text, spans, paths, display names, diagnostics, or lexemes as authority",
);
assert.doesNotMatch(
    source,
    /fingerprint-only|fingerprint only|return Ok immediately after fingerprint|index hit is authority|proof acceptance by index|stable index only|record payload hash only/,
    "sorted artifact index contract must not document or implement fingerprint-only or index-only proof acceptance",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "sorted artifact index contract must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait proof index contract passed");
