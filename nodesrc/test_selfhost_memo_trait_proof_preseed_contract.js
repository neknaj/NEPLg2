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
const artifactCodeOnly = artifact
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

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
    /^#import "\.\/memo_trait_proof_store" as \*$/m,
    "preseed bridge must delegate store relation decisions to the proof store boundary",
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
    /pub enum SelfhostMemoTraitNeplProofPreseedErrorKind:[\s\S]*ArtifactRecordInvalid %SelfhostMemoTraitNeplProofArtifactErrorKind[\s\S]*MaterializedCanonicalKeyMissing[\s\S]*CanonicalPayloadMaterializationInvalid %SelfhostMemoTraitCanonicalKeyPayloadErrorKind[\s\S]*MaterializedFingerprintMismatch[\s\S]*MaterializedPolicyMismatch[\s\S]*CanonicalPayloadHashMismatch/,
    "preseed bridge errors must be typed enum variants for artifact, key materialization, fingerprint, policy, and payload-hash failures",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_neplproof_preseed_error_kind_eq[\s\S]*ArtifactRecordInvalid a_error[\s\S]*selfhost_memo_trait_neplproof_artifact_error_kind_eq a_error b_error[\s\S]*CanonicalPayloadMaterializationInvalid a_error[\s\S]*selfhost_memo_trait_canonical_key_payload_error_kind_eq a_error b_error[\s\S]*CanonicalPayloadHashMismatch/,
    "preseed error equality must compare nested artifact and payload materialization error payloads and every outer variant explicitly",
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
    /selfhost_memo_trait_proof_store_push_stable_key &arena &nominal_table store policy type_id proof[\s\S]*Result::Ok seeded_store:[\s\S]*selfhost_memo_trait_neplproof_preseed_stage0_after_seeded_store/,
    "preseed stage0 must create a real stable proof store record before testing decoded bridge skip and conflict decisions",
);
assert.match(
    source,
    /let existing_matching[\s\S]*selfhost_memo_trait_neplproof_record_preseed_decision_materialized &seeded_store &nominal_table[\s\S]*valid_record[\s\S]*let conflict_record[\s\S]*SelfhostMemoTraitStoredProofKind::KeyOnlyUnsupported[\s\S]*let rejected_conflict[\s\S]*selfhost_memo_trait_neplproof_record_preseed_decision_materialized &seeded_store &nominal_table[\s\S]*conflict_record/,
    "preseed stage0 must exercise ExistingMatching and RejectedConflict through the decoded bridge public API",
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
    source,
    /fingerprint-only|fingerprint only|return Ok immediately after fingerprint|MaterializedFingerprintMismatch[\s\S]{0,160}Result::Ok/,
    "preseed bridge must not allow fingerprint-only acceptance",
);

console.log("selfhost memo trait proof preseed contract passed");
