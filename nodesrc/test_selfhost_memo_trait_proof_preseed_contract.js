#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";
const artifactRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const proofStore = readRepoFile(repoRoot, proofStoreRelPath);
const artifact = readRepoFile(repoRoot, artifactRelPath);
const sourceCodeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");
const artifactCodeOnly = artifact
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");
const candidateProjectorMatch = sourceCodeOnly.match(
    /pub fn selfhost_memo_trait_neplproof_decoded_batch_record_from_candidate_result[\s\S]*?(?=\nfn selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_build_error)/,
);
assert.ok(
    candidateProjectorMatch,
    "single-candidate projector body must remain inspectable by source policy",
);
const candidateProjector = candidateProjectorMatch[0];
const candidateRangePreseedMatch = sourceCodeOnly.match(
    /fn selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_build_error[\s\S]*?(?=\nfn selfhost_memo_trait_neplproof_preseed_batch_error_new)/,
);
assert.ok(
    candidateRangePreseedMatch,
    "candidate range preseed body must remain inspectable by source policy",
);
const candidateRangePreseed = candidateRangePreseedMatch[0];
const stage0SummaryMatch = sourceCodeOnly.match(
    /pub struct SelfhostMemoTraitNeplProofPreseedStage0Summary:[\s\S]*?(?=\nimpl Clone for SelfhostMemoTraitNeplProofPreseedErrorKind:)/,
);
assert.ok(
    stage0SummaryMatch,
    "decoded preseed stage0 summary struct must remain inspectable by source policy",
);
const stage0Summary = stage0SummaryMatch[0];

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_proof_preseed" as \*$/m,
    "ty facade must re-export the decoded .neplproof preseed bridge",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_artifact" as \*$/m,
    "preseed bridge must validate decoded records through the artifact schema boundary",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_decoded" as \*$/m,
    "preseed bridge must consume validated decoded artifact owners before building batch input records",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_proof_store" as \*$/m,
    "preseed bridge must delegate store relation decisions to the proof store boundary",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_canonical_key_payload_codec" as \*$/m,
    "preseed bridge must decode serialized canonical payload bytes through the dedicated codec module",
);
assert.match(
    source,
    /# ty\/memo_trait_proof_preseed[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "preseed bridge documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /materialized_fingerprint[\s\S]*decoded canonical key payload[\s\S]*payload hash[\s\S]*この module が再計算[\s\S]*caller から raw `i32` として受け取りません/,
    "preseed bridge docs must require decoded canonical key payload evidence and reject caller-supplied raw payload hashes",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofPreseedErrorKind:[\s\S]*ArtifactRecordInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*CanonicalPayloadDecodeInvalid %SelfhostMemoTraitCanonicalKeyPayloadDecodeErrorKind[\s\S]*MaterializedCanonicalKeyMissing[\s\S]*CanonicalPayloadMaterializationInvalid %SelfhostMemoTraitCanonicalKeyPayloadErrorKind[\s\S]*MaterializedFingerprintInvalid %SelfhostMemoTraitCanonicalFingerprintErrorKind[\s\S]*MaterializedFingerprintMismatch[\s\S]*MaterializedPolicyMismatch[\s\S]*CanonicalPayloadHashMismatch/,
    "preseed bridge errors must be typed enum variants for artifact, bytes decode, key materialization, fingerprint, policy, and payload-hash failures",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofPreseedAppendErrorKind:[\s\S]*DecisionInvalid %SelfhostMemoTraitNeplProofPreseedErrorKind[\s\S]*RejectedConflict[\s\S]*StoreAppendInvalid %SelfhostMemoTraitProofStorePushErrorKind/,
    "preseed append errors must distinguish decision rejection, stable conflict, and proof-store append failure",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofDecodedBatchRecord:[\s\S]*materialized_key_id %SelfhostCanonicalTypeKeyId[\s\S]*expected_policy %SelfhostMemoTraitProofStorePolicy[\s\S]*record %SelfhostMemoTraitNeplProofRecord/,
    "batch preseed records must bundle the materialized canonical key id, expected policy, and typed artifact record",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofDecodedBatchRecordError:[\s\S]*record_ordinal %i32[\s\S]*kind %SelfhostMemoTraitNeplProofDecodedArtifactErrorKind/,
    "decoded batch builder record errors must keep the artifact record ordinal and typed decoded-artifact error",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofDecodedBatchStdError:[\s\S]*record_ordinal %i32[\s\S]*kind %StdErrorKind/,
    "decoded batch builder standard errors must keep the failing record ordinal",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofDecodedBatchCandidateError:[\s\S]*candidate_offset %i32[\s\S]*kind %SelfhostMemoTraitNeplProofDecodedArtifactErrorKind/,
    "decoded batch builder candidate errors must keep the candidate-range offset and typed decoded-artifact error",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofDecodedBatchBuildErrorKind:[\s\S]*ArtifactInvalid %SelfhostMemoTraitNeplProofDecodedArtifactErrorKind[\s\S]*MaterializedKeyCountMismatch[\s\S]*MaterializedKeyMissing %i32[\s\S]*RecordInvalid %SelfhostMemoTraitNeplProofDecodedBatchRecordError[\s\S]*CandidateInvalid %SelfhostMemoTraitNeplProofDecodedBatchCandidateError[\s\S]*BatchRecordAllocInvalid %StdErrorKind[\s\S]*BatchRecordPushInvalid %SelfhostMemoTraitNeplProofDecodedBatchStdError/,
    "decoded batch builder errors must separate artifact invalidity, key-id coverage, record access, candidate access, and vector allocation/push failures",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofPreseedBatchErrorKind:[\s\S]*RecordMissing[\s\S]*RecordAppendInvalid %SelfhostMemoTraitNeplProofPreseedAppendErrorKind/,
    "batch preseed errors must distinguish missing vector entries from nested single-record append failures",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofPreseedBatchError:[\s\S]*record_ordinal %i32[\s\S]*kind %SelfhostMemoTraitNeplProofPreseedBatchErrorKind/,
    "batch preseed errors must carry the failing record ordinal and typed batch error kind",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitNeplProofDecodedCandidateRangePreseedErrorKind:[\s\S]*BuildInvalid %SelfhostMemoTraitNeplProofDecodedBatchBuildErrorKind[\s\S]*AppendInvalid %SelfhostMemoTraitNeplProofPreseedAppendErrorKind/,
    "candidate range preseed errors must keep typed nested build and append failures",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitNeplProofDecodedCandidateRangePreseedError:[\s\S]*candidate_offset %i32[\s\S]*kind %SelfhostMemoTraitNeplProofDecodedCandidateRangePreseedErrorKind/,
    "candidate range preseed errors must carry the range-local candidate offset and typed failure kind",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_preseed_error_kind_eq[\s\S]*ArtifactRecordInvalid a_error[\s\S]*selfhost_memo_trait_neplproof_artifact_error_kind_eq a_error b_error[\s\S]*CanonicalPayloadDecodeInvalid a_error[\s\S]*selfhost_memo_trait_canonical_key_payload_decode_error_kind_eq a_error b_error[\s\S]*CanonicalPayloadMaterializationInvalid a_error[\s\S]*selfhost_memo_trait_canonical_key_payload_error_kind_eq a_error b_error[\s\S]*MaterializedFingerprintInvalid a_error[\s\S]*selfhost_memo_trait_canonical_fingerprint_error_kind_eq a_error b_error[\s\S]*CanonicalPayloadHashMismatch/,
    "preseed error equality must compare nested artifact, decode, payload materialization, and fingerprint error payloads",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_preseed_append_error_kind_eq[\s\S]*DecisionInvalid a_error[\s\S]*selfhost_memo_trait_neplproof_preseed_error_kind_eq a_error b_error[\s\S]*RejectedConflict[\s\S]*StoreAppendInvalid a_error[\s\S]*selfhost_memo_trait_proof_store_push_error_kind_eq a_error b_error/,
    "preseed append error equality must compare nested decision and store push payloads",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_preseed_batch_error_kind_eq[\s\S]*RecordMissing[\s\S]*RecordAppendInvalid a_error[\s\S]*selfhost_memo_trait_neplproof_preseed_append_error_kind_eq a_error b_error/,
    "preseed batch error equality must compare nested single-record append error payloads",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_preseed_batch_error_eq[\s\S]*eq a\.record_ordinal b\.record_ordinal[\s\S]*selfhost_memo_trait_neplproof_preseed_batch_error_kind_eq a\.kind b\.kind/,
    "preseed batch error equality must require both the failing ordinal and typed error kind to match",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_batch_build_error_kind_eq[\s\S]*ArtifactInvalid a_error[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_error_kind_eq a_error b_error[\s\S]*MaterializedKeyMissing a_ordinal[\s\S]*eq a_ordinal b_ordinal[\s\S]*RecordInvalid a_error[\s\S]*selfhost_memo_trait_neplproof_decoded_batch_record_error_eq a_error b_error[\s\S]*CandidateInvalid a_error[\s\S]*selfhost_memo_trait_neplproof_decoded_batch_candidate_error_eq a_error b_error[\s\S]*BatchRecordPushInvalid a_error[\s\S]*selfhost_memo_trait_neplproof_decoded_batch_std_error_eq a_error b_error/,
    "decoded batch builder error equality must compare nested artifact, ordinal, candidate, and standard-error payloads",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_error_kind_eq[\s\S]*BuildInvalid a_error[\s\S]*selfhost_memo_trait_neplproof_decoded_batch_build_error_kind_eq a_error b_error[\s\S]*AppendInvalid a_error[\s\S]*selfhost_memo_trait_neplproof_preseed_append_error_kind_eq a_error b_error/,
    "candidate range preseed error equality must compare nested build and append payloads",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_record_preseed_decision_materialized[\s\S]*selfhost_memo_trait_neplproof_record_key_result record\.key\.canonical_fingerprint record\.key\.canonical_payload_schema_version record\.key\.canonical_payload_hash record\.key\.policy[\s\S]*selfhost_memo_trait_neplproof_record_result checked_key record\.proof_kind record\.proof record\.record_payload_hash/,
    "preseed bridge must revalidate decoded record key and record body before using a .neplproof record",
);
assert.match(
    source,
    /selfhost_canonical_type_key_arena_get_node materialized_key_arena materialized_key_id[\s\S]*MaterializedCanonicalKeyMissing/,
    "preseed bridge must reject missing materialized canonical keys before scanning the proof store",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_record_preseed_decision_materialized %fn &SelfhostMemoTraitProofStore fn &SelfhostMemoTraitStableNominalKeyTable fn &SelfhostCanonicalTypeKeyArena fn SelfhostCanonicalTypeKeyId fn SelfhostMemoTraitProofStorePolicy fn SelfhostMemoTraitCanonicalTypeFingerprint fn SelfhostMemoTraitNeplProofRecord Result/,
    "preseed public API must take the stable nominal table and materialized key arena, not a caller-supplied payload hash",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_record_preseed_decision_decoded_payload_bytes %impure fn &SelfhostMemoTraitProofStore impure fn &SelfhostMemoTraitStableNominalKeyTable impure fn &Vec u8 impure fn SelfhostMemoTraitProofStorePolicy impure fn SelfhostMemoTraitNeplProofRecord Result/,
    "preseed bridge must expose a bytes-decoding decision API for decoded .neplproof records",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_record_preseed_decision_decoded_payload_bytes[\s\S]*selfhost_memo_trait_canonical_key_payload_decode_result nominal_table canonical_payload_bytes[\s\S]*selfhost_memo_trait_neplproof_record_preseed_decision_decoded_checked store nominal_table &decoded expected_policy record[\s\S]*selfhost_memo_trait_canonical_key_payload_decoded_free decoded[\s\S]*CanonicalPayloadDecodeInvalid decode_error/,
    "bytes decision API must decode through the codec, free decoded owners, and report typed decode errors",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_record_preseed_decision_decoded_checked[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_result nominal_table materialized_key_arena materialized_key_id[\s\S]*selfhost_memo_trait_neplproof_record_preseed_decision_materialized store nominal_table materialized_key_arena materialized_key_id expected_policy materialized_fingerprint record[\s\S]*MaterializedFingerprintInvalid fingerprint_error/,
    "decoded decision API must recompute stable fingerprint from decoded key material before delegating to materialized preseed",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_record_append_decoded_payload_bytes %impure fn SelfhostMemoTraitProofStore impure fn &SelfhostMemoTraitStableNominalKeyTable impure fn &Vec u8 impure fn SelfhostMemoTraitProofStorePolicy impure fn SelfhostMemoTraitNeplProofRecord Result SelfhostMemoTraitProofStore SelfhostMemoTraitNeplProofPreseedAppendErrorKind/,
    "preseed bridge must expose a single-record append API that consumes and returns the proof store owner",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_record_append_materialized_decision[\s\S]*SelfhostMemoTraitProofStorePreseedDecision::AcceptMissing:[\s\S]*selfhost_memo_trait_proof_store_push_materialized_key store materialized_key_arena materialized_key_id expected_policy materialized_fingerprint record\.proof_kind record\.proof[\s\S]*SelfhostMemoTraitProofStorePreseedDecision::ExistingMatching:[\s\S]*Result::Ok store[\s\S]*SelfhostMemoTraitProofStorePreseedDecision::RejectedConflict:[\s\S]*selfhost_memo_trait_proof_store_free store[\s\S]*RejectedConflict/,
    "live append path must append only on AcceptMissing, skip ExistingMatching, and fail-closed on RejectedConflict",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_neplproof_record_append_decoded_decision/,
    "preseed contract must not be anchored to an obsolete decoded-decision helper that is not on the public append path",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_record_append_decoded_payload_bytes[\s\S]*selfhost_memo_trait_canonical_key_payload_decode_result nominal_table canonical_payload_bytes[\s\S]*selfhost_memo_trait_neplproof_record_append_decoded_checked store nominal_table &decoded expected_policy record[\s\S]*selfhost_memo_trait_canonical_key_payload_decoded_free decoded[\s\S]*selfhost_memo_trait_proof_store_free store[\s\S]*CanonicalPayloadDecodeInvalid decode_error/,
    "append API must free decoded owners and close the input store on bytes decode failure",
);
assert.match(
    source,
    /fn selfhost_memo_trait_neplproof_record_append_materialized_checked[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_result nominal_table materialized_key_arena materialized_key_id[\s\S]*selfhost_memo_trait_neplproof_record_preseed_decision_materialized &store nominal_table materialized_key_arena materialized_key_id expected_policy materialized_fingerprint record[\s\S]*selfhost_memo_trait_neplproof_record_append_materialized_decision store materialized_key_arena materialized_key_id expected_policy materialized_fingerprint record decision[\s\S]*selfhost_memo_trait_proof_store_free store[\s\S]*DecisionInvalid decision_error[\s\S]*selfhost_memo_trait_proof_store_free store[\s\S]*MaterializedFingerprintInvalid fingerprint_error/,
    "materialized append helper must recompute the canonical fingerprint, delegate to the live decision helper, and close the store on decision or fingerprint failure",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_record_batch_append %impure fn SelfhostMemoTraitProofStore impure fn &SelfhostMemoTraitStableNominalKeyTable impure fn &SelfhostCanonicalTypeKeyArena impure fn &Vec SelfhostMemoTraitNeplProofDecodedBatchRecord Result SelfhostMemoTraitProofStore SelfhostMemoTraitNeplProofPreseedBatchError/,
    "preseed bridge must expose a public batch append API over materialized decoded .neplproof records",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_batch_records_from_artifact_result %impure fn &SelfhostMemoTraitNeplProofDecodedArtifact impure fn &SelfhostCanonicalTypeKeyArena impure fn &Vec SelfhostCanonicalTypeKeyId impure fn SelfhostMemoTraitProofStorePolicy Result Vec SelfhostMemoTraitNeplProofDecodedBatchRecord SelfhostMemoTraitNeplProofDecodedBatchBuildErrorKind/,
    "preseed bridge must expose a decoded artifact plus materialized key arena to batch-record projector before the public batch append API",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_batch_record_from_candidate_result %impure fn &SelfhostMemoTraitNeplProofDecodedArtifact impure fn &SelfhostCanonicalTypeKeyArena impure fn &Vec SelfhostCanonicalTypeKeyId impure fn SelfhostMemoTraitProofStorePolicy impure fn SelfhostMemoTraitCanonicalTypeFingerprint impure fn SelfhostMemoTraitNeplProofIndexCandidateRange impure fn i32 Result SelfhostMemoTraitNeplProofDecodedBatchRecord SelfhostMemoTraitNeplProofDecodedBatchBuildErrorKind/,
    "preseed bridge must expose a single-candidate batch-record projector for sorted-index lookup paths",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_decoded_candidate_range_preseed %impure fn SelfhostMemoTraitProofStore impure fn &SelfhostMemoTraitStableNominalKeyTable impure fn &SelfhostCanonicalTypeKeyArena impure fn &SelfhostMemoTraitNeplProofDecodedArtifact impure fn &Vec SelfhostCanonicalTypeKeyId impure fn SelfhostMemoTraitProofStorePolicy impure fn SelfhostMemoTraitCanonicalTypeFingerprint impure fn SelfhostMemoTraitNeplProofIndexCandidateRange Result SelfhostMemoTraitProofStore SelfhostMemoTraitNeplProofDecodedCandidateRangePreseedError/,
    "preseed bridge must expose a candidate range API that consumes and returns the proof store owner while borrowing decoded artifact and materialized key inputs",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_decoded_batch_records_from_artifact_result[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_validate_result artifact[\s\S]*let header %SelfhostMemoTraitNeplProofHeader \*field::get_ref artifact "header"[\s\S]*ne v::len materialized_key_ids header\.record_count[\s\S]*MaterializedKeyCountMismatch[\s\S]*let out_result %Result Vec SelfhostMemoTraitNeplProofDecodedBatchRecord StdErrorKind v::new[\s\S]*selfhost_memo_trait_neplproof_decoded_batch_records_from_artifact_loop artifact materialized_key_arena materialized_key_ids expected_policy out 0 header\.record_count[\s\S]*ArtifactInvalid artifact_error/,
    "decoded artifact projector must validate artifact invariants, check materialized-key count, type its output vector, pass the materialized arena into the loop, and fail closed on invalid artifacts",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_decoded_batch_records_from_artifact_loop[\s\S]*v::get materialized_key_ids record_ordinal[\s\S]*selfhost_canonical_type_key_arena_get_node materialized_key_arena materialized_key_id[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_record_at_result artifact record_ordinal[\s\S]*selfhost_memo_trait_neplproof_decoded_batch_record_new materialized_key_id expected_policy record[\s\S]*selfhost_memo_trait_neplproof_decoded_batch_records_push out batch_record record_ordinal[\s\S]*v::free out[\s\S]*RecordInvalid build_error[\s\S]*v::free out[\s\S]*MaterializedKeyMissing record_ordinal/,
    "decoded artifact projector loop must pair records and materialized key ids by ordinal, verify each id exists in the materialized arena, and close partial output on record/key failures",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_decoded_batch_record_from_candidate_result[\s\S]*selfhost_memo_trait_neplproof_decoded_artifact_candidate_record_at_result artifact target candidate_range candidate_offset[\s\S]*ne v::len materialized_key_ids header\.record_count[\s\S]*MaterializedKeyCountMismatch[\s\S]*let candidate_index_entry %SelfhostMemoTraitNeplProofIndexEntry \*field::get_ref &candidate_record "index_entry"[\s\S]*let candidate_payload_record %SelfhostMemoTraitNeplProofRecord \*field::get_ref &candidate_record "record"[\s\S]*let record_ordinal %i32 candidate_index_entry\.record_ordinal[\s\S]*v::get materialized_key_ids record_ordinal[\s\S]*selfhost_canonical_type_key_arena_get_node materialized_key_arena materialized_key_id[\s\S]*selfhost_memo_trait_neplproof_decoded_batch_record_new materialized_key_id expected_policy candidate_payload_record[\s\S]*CandidateInvalid candidate_error/,
    "single-candidate projector must use the decoded candidate accessor, use index_entry.record_ordinal for materialized-key lookup, verify arena existence, and type candidate failures",
);
assert.doesNotMatch(
    source,
    /v::get materialized_key_ids candidate_offset|v::get materialized_key_ids candidate_range\.start_index|v::get materialized_key_ids add candidate_range\.start_index candidate_offset|let record_ordinal %i32 candidate_offset/,
    "single-candidate projector must not treat candidate offset as the decoded record ordinal or key-id ordinal",
);
assert.doesNotMatch(
    candidateProjector,
    /selfhost_memo_trait_neplproof_record_append|selfhost_memo_trait_neplproof_decoded_record_batch_append|selfhost_memo_trait_proof_store_preseed_decision/,
    "single-candidate projector must not perform proof-store preseed decisions or append work",
);
assert.match(
    candidateRangePreseed,
    /selfhost_memo_trait_neplproof_decoded_batch_record_from_candidate_result artifact materialized_key_arena materialized_key_ids expected_policy target candidate_range candidate_offset[\s\S]*selfhost_memo_trait_neplproof_record_append_materialized_checked store nominal_table materialized_key_arena batch_record\.materialized_key_id batch_record\.expected_policy batch_record\.record/,
    "candidate range preseed must build each candidate through the single-candidate projector and delegate acceptance to the existing materialized append boundary",
);
assert.match(
    candidateRangePreseed,
    /selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_build_error[\s\S]*BuildInvalid build_error[\s\S]*selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_append_error[\s\S]*AppendInvalid append_error/,
    "candidate range preseed error constructors must preserve typed build and append error payloads",
);
assert.match(
    candidateRangePreseed,
    /Result::Err append_error:[\s\S]*selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_append_error candidate_offset append_error[\s\S]*Result::Err build_error:[\s\S]*selfhost_memo_trait_proof_store_free store[\s\S]*selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_build_error candidate_offset build_error/,
    "candidate range preseed must close the store on build failure and avoid double-closing on append failure",
);
assert.match(
    candidateRangePreseed,
    /ge candidate_offset candidate_range\.candidate_count[\s\S]*selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_one store nominal_table materialized_key_arena artifact materialized_key_ids expected_policy target candidate_range candidate_offset[\s\S]*add candidate_offset 1/,
    "candidate range preseed must use range candidate_count as the loop bound and advance only the range-local candidate offset",
);
assert.match(
    candidateRangePreseed,
    /le candidate_range\.candidate_count 0[\s\S]*selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_one store nominal_table materialized_key_arena artifact materialized_key_ids expected_policy target candidate_range 0/,
    "candidate range preseed must reject non-positive ranges through the same typed candidate build path",
);
assert.doesNotMatch(
    candidateRangePreseed,
    /v::get materialized_key_ids candidate_offset|v::get materialized_key_ids candidate_range\.start_index|v::get materialized_key_ids add candidate_range\.start_index candidate_offset|record\.key\.policy|selfhost_memo_trait_proof_store_push|selfhost_memo_trait_proof_store_preseed_decision/,
    "candidate range preseed must not treat candidate offsets as key ordinals, derive policy from records, or bypass the append boundary",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_decoded_batch_record_from_candidate_result:[\s\S]*proof acceptance ではありません[\s\S]*selfhost_memo_trait_neplproof_record_append_materialized_checked[\s\S]*canonical payload hash[\s\S]*fingerprint[\s\S]*policy[\s\S]*store relation[\s\S]*candidate accessor は artifact validation、candidate range validation、target fingerprint check、index entry \/ record 対応検査を再実行します/,
    "single-candidate projector documentation must state validation cost and delegate proof acceptance to the later append boundary",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_decoded_candidate_range_preseed:[\s\S]*reader \/ serializer[\s\S]*persistent stable map[\s\S]*proof acceptance ではありません[\s\S]*canonical payload hash[\s\S]*fingerprint[\s\S]*policy[\s\S]*store relation[\s\S]*selfhost_memo_trait_neplproof_record_append_materialized_checked[\s\S]*\[計算量\/けいさんりょう\]/,
    "candidate range preseed documentation must state delegation, non-reader scope, missing persistent-index work, and complexity",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_decoded_record_batch_append_loop[\s\S]*let n %i32 v::len records[\s\S]*match v::get records idx:[\s\S]*selfhost_memo_trait_neplproof_decoded_record_batch_append_one store nominal_table materialized_key_arena batch_record idx[\s\S]*selfhost_memo_trait_neplproof_decoded_record_batch_append_loop next_store nominal_table materialized_key_arena records add idx 1[\s\S]*selfhost_memo_trait_proof_store_free store[\s\S]*RecordMissing/,
    "batch append loop must process records in order, report the failing ordinal, and close the store if the vector read fails",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_decoded_record_batch_append_one[\s\S]*selfhost_memo_trait_neplproof_record_append_materialized_checked store nominal_table materialized_key_arena batch_record\.materialized_key_id batch_record\.expected_policy batch_record\.record[\s\S]*selfhost_memo_trait_neplproof_decoded_record_batch_append_error record_ordinal append_error/,
    "batch append must wrap single-record append failures with the batch record ordinal",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_decoded_record_batch_append:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*O\(m \* n \* k \+ materialization\)/,
    "batch append documentation must record the current complexity and materialization cost",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_hash_result nominal_table materialized_key_arena materialized_key_id[\s\S]*Result::Err payload_error:[\s\S]*CanonicalPayloadMaterializationInvalid payload_error/,
    "preseed bridge must recompute payload hash internally and report payload materialization failures as typed errors",
);
assert.match(
    source,
    /materialized_canonical_payload_hash record\.key\.canonical_payload_hash[\s\S]*CanonicalPayloadHashMismatch[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_eq materialized_fingerprint record\.key\.canonical_fingerprint[\s\S]*MaterializedFingerprintMismatch[\s\S]*selfhost_memo_trait_proof_store_policy_eq expected_policy record\.key\.policy[\s\S]*MaterializedPolicyMismatch/,
    "preseed bridge must check payload hash, canonical fingerprint, and policy before asking the store for a decision",
);
assert.match(
    source,
    /Result::Ok selfhost_memo_trait_proof_store_preseed_decision_materialized_key store materialized_key_arena materialized_key_id expected_policy materialized_fingerprint record\.proof_kind record\.proof/,
    "preseed bridge must delegate accepted materialized records to the proof store materialized decision API",
);
assert.match(
    source,
    /SelfhostMemoTraitNeplProofPreseedStage0Summary:[\s\S]*accept_missing[\s\S]*existing_matching[\s\S]*rejected_conflict[\s\S]*missing_key[\s\S]*hash_mismatch[\s\S]*fingerprint_mismatch[\s\S]*policy_mismatch[\s\S]*invalid_record/,
    "preseed stage0 summary must exercise empty-store append, seeded-store skip, seeded-store conflict, and all fail-closed materialization errors",
);
assert.match(
    source,
    /SelfhostMemoTraitNeplProofPreseedStage0Summary:[\s\S]*batch_empty %Result unit SelfhostMemoTraitNeplProofPreseedBatchError[\s\S]*batch_existing_matching %Result unit SelfhostMemoTraitNeplProofPreseedBatchError[\s\S]*batch_rejected_conflict %Result unit SelfhostMemoTraitNeplProofPreseedBatchError[\s\S]*batch_invalid_record %Result unit SelfhostMemoTraitNeplProofPreseedBatchError/,
    "preseed stage0 summary must include batch smoke cases for empty, matching, conflict, and invalid-record inputs",
);
assert.match(
    source,
    /SelfhostMemoTraitNeplProofPreseedStage0Summary:[\s\S]*batch_from_artifact %Result unit SelfhostMemoTraitNeplProofDecodedBatchBuildErrorKind[\s\S]*batch_from_artifact_count_mismatch %Result unit SelfhostMemoTraitNeplProofDecodedBatchBuildErrorKind[\s\S]*batch_candidate_from_artifact %Result unit SelfhostMemoTraitNeplProofDecodedBatchBuildErrorKind[\s\S]*batch_candidate_offset_mismatch %Result unit SelfhostMemoTraitNeplProofDecodedBatchBuildErrorKind/,
    "preseed stage0 summary must include decoded artifact projector and candidate projector smoke cases",
);
assert.doesNotMatch(stage0Summary, /candidate_range/i, "candidate range fields must not expand the already-large decoded preseed stage0 summary");
assert.doesNotMatch(stage0Summary, /CandidateRange/i, "candidate range error types must not appear in the decoded preseed stage0 summary");
assert.doesNotMatch(stage0Summary, /DecodedCandidateRangePreseedError/i, "candidate range preseed errors must stay out of the decoded preseed stage0 summary payload");
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_payload_bytes[\s\S]*selfhost_memo_trait_neplproof_record_preseed_decision_decoded_payload_bytes &store &nominal_table &canonical_payload_bytes policy valid_record[\s\S]*SelfhostMemoTraitProofStorePreseedDecision::AcceptMissing:[\s\S]*selfhost_memo_trait_neplproof_record_append_decoded_payload_bytes store &nominal_table &canonical_payload_bytes policy valid_record[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_after_appended_store/,
    "preseed stage0 must build canonical payload bytes, classify an empty-store decoded record, and append it through the public decoded append API",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_after_appended_store[\s\S]*selfhost_memo_trait_neplproof_record_append_decoded_payload_bytes appended_store &nominal_table &canonical_payload_bytes policy valid_record[\s\S]*let existing_matching[\s\S]*SelfhostMemoTraitProofStorePreseedDecision::ExistingMatching[\s\S]*let conflict_record[\s\S]*SelfhostMemoTraitStoredProofKind::KeyOnlyUnsupported[\s\S]*selfhost_memo_trait_neplproof_record_append_decoded_payload_bytes existing_store &nominal_table &canonical_payload_bytes policy conflict_record[\s\S]*SelfhostMemoTraitNeplProofPreseedAppendErrorKind::RejectedConflict[\s\S]*SelfhostMemoTraitProofStorePreseedDecision::RejectedConflict/,
    "preseed stage0 must exercise ExistingMatching and RejectedConflict through the public decoded append API, not through direct materialized decisions",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_after_appended_store[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_batch_empty &nominal_table &materialized_key_arena[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_batch_same_record_skip &nominal_table &materialized_key_arena materialized_key_id policy valid_record[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_batch_conflict &nominal_table &materialized_key_arena materialized_key_id policy valid_record conflict_record[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_batch_invalid_record &nominal_table &materialized_key_arena materialized_key_id policy valid_record invalid_record/,
    "preseed stage0 must exercise the public batch API before returning its summary",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_after_appended_store[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_decoded_batch_from_artifact &materialized_key_arena materialized_key_id policy valid_record[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_decoded_batch_count_mismatch &materialized_key_arena policy valid_record[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_decoded_candidate_from_artifact &materialized_key_arena materialized_key_id policy valid_record[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_decoded_candidate_offset_mismatch &materialized_key_arena materialized_key_id policy valid_record[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_summary_new accept_missing existing_matching rejected_conflict missing_key hash_mismatch fingerprint_mismatch policy_mismatch invalid_result batch_empty batch_existing_matching batch_rejected_conflict batch_invalid_record batch_from_artifact batch_from_artifact_count_mismatch batch_candidate_from_artifact batch_candidate_offset_mismatch/,
    "preseed stage0 must keep decoded artifact projection and single-candidate projection in the existing summary without expanding that constructor",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_candidate_range_preseed_success[\s\S]*SelfhostMemoTraitNeplProofIndexCandidateRange 0 1[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_candidate_range_preseed_from_record/,
    "candidate range preseed must keep a small helper-level success smoke path without wiring it into the large decoded preseed summary",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_candidate_range_preseed_smoke[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_candidate_range_preseed_success[\s\S]*selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_result_is_ok success_result[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_candidate_range_preseed_invalid_range[\s\S]*selfhost_memo_trait_neplproof_decoded_candidate_range_preseed_result_is_ok invalid_range_result[\s\S]*StdErrorKind::InvalidOperation/,
    "candidate range preseed smoke must execute success and invalid-range paths while keeping summary payload small",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_after_appended_store[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_candidate_range_preseed_smoke &nominal_table &materialized_key_arena materialized_key_id policy valid_record[\s\S]*selfhost_memo_trait_neplproof_record_append_decoded_payload_bytes existing_store &nominal_table &canonical_payload_bytes policy conflict_record/,
    "decoded preseed stage0 must execute candidate range preseed smoke before returning its existing summary",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_candidate_range_preseed_invalid_range[\s\S]*SelfhostMemoTraitNeplProofIndexCandidateRange -1 1/,
    "preseed stage0 must keep an invalid range smoke case that returns a typed candidate build error",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_decoded_batch_count_mismatch[\s\S]*let key_ids_result %Result Vec SelfhostCanonicalTypeKeyId StdErrorKind v::new[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_decoded_batch_build_from_record valid_record materialized_key_arena key_ids policy/,
    "preseed stage0 must explicitly cover a decoded-artifact/key-id count mismatch",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_finish[\s\S]*selfhost_memo_trait_proof_store_free store[\s\S]*selfhost_canonical_type_key_arena_free materialized_key_arena[\s\S]*selfhost_type_arena_free arena/,
    "preseed stage0 must close proof store, materialized canonical key arena, and type arena on success",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_finish_with_nominal_table[\s\S]*selfhost_memo_trait_stable_nominal_key_table_free nominal_table[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_finish/,
    "preseed stage0 must close the stable nominal key table used to seed the store",
);
assert.match(
    source,
    /selfhost_memo_trait_neplproof_preseed_stage0_abort_with_key_store[\s\S]*selfhost_memo_trait_proof_store_free store[\s\S]*selfhost_canonical_type_key_arena_free materialized_key_arena[\s\S]*selfhost_type_arena_free arena/,
    "preseed stage0 must close proof store, materialized canonical key arena, and type arena on setup failure",
);
assert.match(
    proofStore,
    /pub fn selfhost_memo_trait_proof_store_preseed_decision_materialized_key[\s\S]*&SelfhostMemoTraitProofStore[\s\S]*&SelfhostCanonicalTypeKeyArena[\s\S]*SelfhostCanonicalTypeKeyId[\s\S]*SelfhostMemoTraitProofStorePolicy[\s\S]*SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*SelfhostMemoTraitStoredProofKind[\s\S]*SelfhostMemoTraitStoredAggregateProof[\s\S]*SelfhostMemoTraitProofStorePreseedDecision/,
    "proof store must expose a public materialized-key preseed decision that does not expose the private stable identity struct",
);
assert.match(
    proofStore,
    /pub fn selfhost_memo_trait_proof_store_push_materialized_key[\s\S]*selfhost_canonical_type_key_copy_from_arena candidate_arena key_arena candidate_key[\s\S]*selfhost_memo_trait_proof_store_stable_duplicate_exists &next_key_arena &records stable_identity[\s\S]*selfhost_memo_trait_proof_store_record_new copied_key \(some candidate_fingerprint\) policy proof_kind proof/,
    "proof store must append decoded artifact proof by copying materialized canonical keys into the store-owned arena",
);
assert.match(
    proofStore,
    /fn selfhost_memo_trait_proof_store_record_materialized_identity_matches[\s\S]*match record\.stable_fingerprint:[\s\S]*selfhost_memo_trait_proof_store_canonical_key_equal_cross store_key_arena record\.key_id candidate_arena candidate_key[\s\S]*selfhost_memo_trait_proof_store_policy_eq record\.policy candidate_policy[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_eq record_fingerprint candidate_fingerprint[\s\S]*Option::None:[\s\S]*false/,
    "proof store materialized identity matching must require stable record fingerprint, cross-arena canonical equality, policy equality, and fingerprint equality",
);
assert.match(
    proofStore,
    /fn selfhost_memo_trait_proof_store_preseed_decision_for_materialized_record[\s\S]*selfhost_memo_trait_proof_store_record_materialized_identity_matches[\s\S]*selfhost_memo_trait_proof_store_record_payload_matches record proof_kind proof[\s\S]*ExistingMatching[\s\S]*RejectedConflict[\s\S]*AcceptMissing/,
    "proof store materialized decision must classify same-identity same-payload, same-identity conflict, and different identity separately",
);
assert.match(
    proofStore,
    /selfhost_memo_trait_proof_store_stage0_preseed_from_first_stable_loop[\s\S]*selfhost_memo_trait_proof_store_preseed_decision_materialized_key store key_arena record\.key_id record\.policy fingerprint proof_kind proof/,
    "proof store stage0 must exercise the public materialized preseed API for existing and conflicting stable records",
);
assert.doesNotMatch(
    artifactCodeOnly,
    /SelfhostCanonicalTypeKeyId|SelfhostTypeId|SelfhostNamedTypeId|SelfhostMemoTraitProofStoreStableIdentity|SelfhostMemoTraitProofStoreRecord|SelfhostMemoTraitProofStoreStableIndexEntry/,
    "artifact schema code must continue to exclude store-local ids, type ids, store records, and store-local stable identities",
);
assert.doesNotMatch(
    sourceCodeOnly,
    /neplproof.*(?:reader|serializer)|(?:reader|serializer).*neplproof|read_bytes|write_bytes/,
    "preseed bridge must not grow a binary reader or serializer in the decoded batch projector slice",
);
assert.doesNotMatch(
    source,
    /fingerprint-only|fingerprint only|return Ok immediately after fingerprint|MaterializedFingerprintMismatch[\s\S]{0,160}Result::Ok/,
    "preseed bridge must not allow fingerprint-only acceptance",
);

console.log("selfhost memo trait proof preseed contract passed");
