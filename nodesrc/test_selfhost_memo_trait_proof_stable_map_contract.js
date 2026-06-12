#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_stable_map.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_proof_stable_map" as \*$/m,
    "ty facade must re-export the .neplproof stable map contract module",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_artifact" as \*$/m,
    "stable map contract must reuse artifact record and index-entry validators",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_store" as \*$/m,
    "stable map contract may carry policy payloads but must not call proof store operations",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "stable artifact map contract must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /# ty\/memo_trait_proof_stable_map[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[戻\/もど\]り\[値\/ち\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "stable map module documentation must record purpose, contract, return values, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /stable map entry は policy と record payload hash を保持しますが、proof acceptance authority ではありません[\s\S]*policy equality、canonical payload decode、payload hash 再計算、proof kind、producer gate、proof store lookup は後続の責務/,
    "stable map documentation must limit map hits to candidate narrowing",
);
assert.match(
    source,
    /SelfhostCanonicalTypeKeyId.*SelfhostTypeId.*SelfhostNamedTypeId.*source text.*span.*path suffix.*display name.*diagnostic text.*lexeme[\s\S]*stable map key、sort key、tie-break authority に入りません/,
    "stable map documentation must exclude session-local ids and source-display data from authority",
);
assert.match(
    source,
    /producer は record 数 n に対して O\(n\^2\)[\s\S]*後続の persistent binary writer \/ stable map codec/,
    "stable map producer documentation must state the current O(n^2) implementation and the future faster artifact boundary",
);
assert.match(
    source,
    /candidate range lookup は lower-bound binary search と collision group count により O\(log m \+ c\)/,
    "candidate lookup documentation must state the lower-bound search complexity",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofStableMapEntry:[\s\S]*canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*canonical_payload_hash %i32[\s\S]*policy %SelfhostMemoTraitProofStorePolicy[\s\S]*record_ordinal %i32[\s\S]*record_payload_hash %i32/,
    "stable map entry must carry stable artifact identity, policy payload, record ordinal, and record payload hash",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofStableMapCandidateRange:[\s\S]*start_index %i32[\s\S]*candidate_count %i32/,
    "stable lookup must return a typed candidate range instead of proof payload",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofStableMapErrorKind:[\s\S]*KeyInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*IndexEntryInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*RecordInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*EntryMissing[\s\S]*FingerprintOrderInvalid[\s\S]*PayloadHashOrderInvalid[\s\S]*RecordOrdinalOrderInvalid[\s\S]*CandidateMissing/,
    "stable map failures must use typed enum variants and preserve nested artifact errors",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofStableMapProducerErrorKind:[\s\S]*AllocFailed %StdErrorKind[\s\S]*PushFailed %StdErrorKind[\s\S]*RecordEntryMissing[\s\S]*EntryMissing[\s\S]*RecordInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*EntryBuildRejected %SelfhostMemoTraitNeplProofStableMapErrorKind[\s\S]*ProducedOrderRejected %SelfhostMemoTraitNeplProofStableMapErrorKind/,
    "stable map producer failures must use typed enum variants and preserve nested validation errors",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_stable_map_entry_result[\s\S]*selfhost_memo_trait_neplproof_record_key_result[\s\S]*selfhost_memo_trait_neplproof_index_entry_result/,
    "stable map entry validator must delegate key and ordinal/hash checks to artifact validators",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_stable_map_entry_from_record_result[\s\S]*selfhost_memo_trait_neplproof_record_key_result[\s\S]*selfhost_memo_trait_neplproof_record_result[\s\S]*selfhost_memo_trait_neplproof_stable_map_entry_result/,
    "stable map producer must revalidate record keys, record payloads, and produced map entries",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_stable_map_lookup_result[\s\S]*&Vec SelfhostMemoTraitNeplProofStableMapEntry[\s\S]*SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*fn i32[\s\S]*Result SelfhostMemoTraitNeplProofStableMapCandidateRange SelfhostMemoTraitNeplProofStableMapErrorKind/,
    "stable lookup API must take entries, target fingerprint, and target canonical payload hash",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_stable_map_lookup_result[\s\S]*selfhost_memo_trait_neplproof_stable_map_order_result[\s\S]*selfhost_memo_trait_neplproof_stable_map_lookup_loop/,
    "stable lookup must validate order before lower-bound lookup",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_stable_map_lower_bound_loop[\s\S]*ge low high[\s\S]*let mid %i32 add low div_s sub high low 2[\s\S]*selfhost_memo_trait_neplproof_stable_map_target_lt entry target_fingerprint target_payload_hash[\s\S]*add mid 1 high[\s\S]*low mid/,
    "stable map lookup must use a half-open lower-bound binary search",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_stable_map_lookup_loop[\s\S]*selfhost_memo_trait_neplproof_stable_map_lower_bound_loop entries target_fingerprint target_payload_hash 0 n[\s\S]*selfhost_memo_trait_neplproof_stable_map_entry_key_eq entry target_fingerprint target_payload_hash[\s\S]*selfhost_memo_trait_neplproof_stable_map_count_loop entries target_fingerprint target_payload_hash start start 0[\s\S]*CandidateMissing/,
    "stable map lookup must return only the contiguous stable-key candidate range",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_stable_map_pair_result[\s\S]*stable_map_fingerprint_lt current\.canonical_fingerprint previous\.canonical_fingerprint[\s\S]*FingerprintOrderInvalid[\s\S]*lt current\.canonical_payload_hash previous\.canonical_payload_hash[\s\S]*PayloadHashOrderInvalid[\s\S]*ge previous\.record_ordinal current\.record_ordinal[\s\S]*RecordOrdinalOrderInvalid/,
    "stable map order check must reject fingerprint, payload hash, and record ordinal order corruption",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_stable_map_build_result[\s\S]*&Vec SelfhostMemoTraitNeplProofRecord[\s\S]*Result Vec SelfhostMemoTraitNeplProofStableMapEntry SelfhostMemoTraitNeplProofStableMapProducerErrorKind/,
    "stable map producer API must borrow decoded records and return an owned entry vector wrapped in a typed Result",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_stable_map_finish_build[\s\S]*selfhost_memo_trait_neplproof_stable_map_order_result/,
    "stable map producer output must pass sorted-order validation before being returned",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_stable_map_bubble_back_loop[\s\S]*v::replace[\s\S]*v::replace/,
    "stable map producer bubble-back helper must swap Copy entries in place",
);
assert.match(
    source,
    /stage0[\s\S]*accepted_range[\s\S]*accepted_collision_range[\s\S]*missing_candidate[\s\S]*fingerprint_unsorted[\s\S]*payload_hash_unsorted[\s\S]*ordinal_unsorted[\s\S]*built_first_range[\s\S]*built_collision_range[\s\S]*invalid_record/,
    "stage0 must exercise accepted lookup, collision lookup, missing candidate, order errors, producer output, and invalid-record rejection",
);
assert.match(
    source,
    /let expected_collision_range SelfhostMemoTraitNeplProofStableMapCandidateRange 0 2[\s\S]*built_collision_range[\s\S]*RecordInvalid SelfhostMemoTraitNeplProofArtifactErrorKind::RecordPayloadHashPlaceholder/,
    "doctest must verify collision output and invalid-record rejection",
);
assert.doesNotMatch(
    codeOnly,
    /selfhost_memo_trait_proof_store_(?:lookup|push|preseed|stable|materialized|new|free)|selfhost_memo_trait_neplproof_decoded|selfhost_memo_trait_neplproof_reader|selfhost_memo_trait_neplproof_serializer/,
    "stable map implementation must not call proof-store, preseed, decoded, reader, or serializer APIs directly",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostCanonicalTypeKeyId|SelfhostTypeId|SelfhostNamedTypeId|SelfhostMemoTraitProofStoreRecord|SelfhostMemoTraitProofStoreStableIndexEntry/,
    "stable map contract must not store session-local ids or proof-store records/index entries",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "stable map contract code must not use source text, spans, paths, display names, diagnostics, or lexemes as authority",
);
assert.doesNotMatch(
    source,
    /fingerprint-only|fingerprint only|index hit is authority|stable map hit is authority|proof acceptance by stable map|return Ok immediately after map hit|payload hash only/,
    "stable map contract must not document or implement map-hit-only proof acceptance",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "stable map contract must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait proof stable map contract passed");
