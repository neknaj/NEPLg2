#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";
const policyRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_policy.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const policySource = readRepoFile(repoRoot, policyRelPath);

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_proof_store" as \*$/m,
    "ty facade must re-export the memo trait proof store split module",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_policy" as \*$/m,
    "memo trait proof store must import the typed memo trait policy module",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_source" as \*$/m,
    "memo trait proof store must import the trusted memo trait source registry",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_canonical_key" as \*$/m,
    "memo trait proof store must import stable canonical key projection through the core/ty split module",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "memo trait proof store must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /# ty\/memo_trait_proof_store[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "memo trait proof store module documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /store record は `SelfhostTypeId` を保持しません[\s\S]*stable API[\s\S]*stable_fingerprint = some[\s\S]*cross-arena canonical equality[\s\S]*producer gate/,
    "proof store must document that records do not store session-local TypeId and stable lookup still reprojects through canonical equality and producer gate",
);
assert.match(
    policySource,
    /pub struct SelfhostMemoTraitProofStorePolicy:[\s\S]*sources %SelfhostMemoTraitSourceIdentitySet[\s\S]*rules %SelfhostMemoTraitRuleIdentity/,
    "proof store policy must keep solver policy identity as typed source and rule payloads",
);
assert.doesNotMatch(
    source,
    /trait_source_hash %i32|rule_hash %i32|selfhost_memo_trait_proof_store_policy_new %fn i32 fn i32 fn i32 fn i32/,
    "proof store must not expose raw trait_source_hash/rule_hash fields or a raw-i32 policy constructor",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_source_identity_new SelfhostMemoTraitSourceKind::MemoKeyTrait|selfhost_memo_trait_source_identity_new SelfhostMemoTraitSourceKind::MemoValueTrait/,
    "proof store must not construct trusted MemoKey/MemoValue source identities locally",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStoredAggregateProof:[\s\S]*fields %SelfhostMemoTraitAggregateFieldEvidence[\s\S]*copy_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*drop_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*eq_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*hash_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*hazard %SelfhostMemoTraitAggregateHazardEvidence[\s\S]*key_result %Result unit SelfhostMemoTraitRejectKind[\s\S]*value_result %Result unit SelfhostMemoTraitRejectKind/,
    "stored aggregate proof must carry proof payloads without SelfhostTypeId",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitProofStoreRecord:[\s\S]*key_id %SelfhostCanonicalTypeKeyId[\s\S]*stable_fingerprint %Option SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*policy %SelfhostMemoTraitProofStorePolicy[\s\S]*proof_kind %SelfhostMemoTraitStoredProofKind[\s\S]*proof %SelfhostMemoTraitStoredAggregateProof/,
    "proof store record must be keyed by canonical type key, optional stable fingerprint, and policy, not by TypeId",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitProofStoreStableIndexEntry:[\s\S]*stable_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*record_index %i32/,
    "stable proof store index entries must contain only the stable fingerprint and record index candidate pointer",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitProofStore:[\s\S]*key_arena %SelfhostCanonicalTypeKeyArena[\s\S]*records %Vec SelfhostMemoTraitProofStoreRecord[\s\S]*stable_index %Vec SelfhostMemoTraitProofStoreStableIndexEntry/,
    "proof store must own a stable sidecar index in addition to the canonical key arena and record vector",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitProofStoreLookupErrorKind:[\s\S]*TypeProjectionFailed[\s\S]*StableFingerprintProjectionRejected %SelfhostMemoTraitCanonicalFingerprintErrorKind[\s\S]*MissingProof[\s\S]*RecordStableFingerprintMissing[\s\S]*StableFingerprintMismatch[\s\S]*StableIndexMissing[\s\S]*PolicyMismatch[\s\S]*ProofKindMismatch[\s\S]*ProducerRejected %SelfhostMemoTraitEvidenceProduceRejectKind/,
    "proof store lookup must expose typed fail-closed errors for projection, stable fingerprint, index invariant, key, policy, proof-kind, and producer rejection",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitProofStorePushErrorKind:[\s\S]*TypeProjectionFailed[\s\S]*StableFingerprintProjectionRejected %SelfhostMemoTraitCanonicalFingerprintErrorKind[\s\S]*StableDuplicate[\s\S]*OutOfMemory[\s\S]*InternalInvariant/,
    "stable proof store push must preserve canonical fingerprint projection rejection and stable duplicate rejection as typed errors",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_record_new[\s\S]*Option SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*SelfhostMemoTraitProofStoreRecord key_id stable_fingerprint policy proof_kind proof/,
    "proof store record constructor must require the stable fingerprint option explicitly",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_push_with_kind[\s\S]*selfhost_memo_trait_proof_store_record_new key_id none policy proof_kind proof/,
    "existing session-only push must create legacy records with stable_fingerprint = none",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_push_with_kind_stable_key[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_result nominal_table &next_key_arena key_id[\s\S]*let record_index %i32 v::len &records[\s\S]*selfhost_memo_trait_proof_store_record_new key_id \(some fingerprint\) policy proof_kind proof[\s\S]*selfhost_memo_trait_proof_store_stable_index_entry_new fingerprint record_index[\s\S]*v::push stable_index index_entry[\s\S]*StableFingerprintProjectionRejected fingerprint_error/,
    "stable push must compute canonical type fingerprint, store it on the record, and append a sidecar index entry for the record index",
);
assert.match(
    source,
    /fn selfhost_memo_trait_proof_store_stable_duplicate_exists_loop[\s\S]*selfhost_memo_trait_proof_store_canonical_key_equal_cross key_arena record\.key_id key_arena candidate_key[\s\S]*selfhost_memo_trait_proof_store_policy_eq record\.policy expected_policy[\s\S]*match record\.stable_fingerprint:[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_eq record_fingerprint candidate_fingerprint/,
    "stable duplicate detection must require canonical equality, policy equality, and stable fingerprint equality instead of trusting the fingerprint alone",
);
assert.match(
    source,
    /Result::Ok fingerprint:[\s\S]*selfhost_memo_trait_proof_store_stable_duplicate_exists &next_key_arena &records policy key_id fingerprint[\s\S]*SelfhostMemoTraitProofStorePushErrorKind::StableDuplicate[\s\S]*let record_index %i32 v::len &records/,
    "stable push must reject duplicate stable proof identity before appending the record and sidecar index",
);
assert.match(
    source,
    /then:\s*v::free records\s*v::free stable_index\s*selfhost_canonical_type_key_arena_free next_key_arena\s*Result::Err SelfhostMemoTraitProofStorePushErrorKind::StableDuplicate/,
    "stable duplicate rejection must close records, stable index, and the projected key arena before returning the typed error",
);
assert.match(
    source,
    /Result::Err fingerprint_error:[\s\S]*v::free records[\s\S]*v::free stable_index[\s\S]*selfhost_canonical_type_key_arena_free next_key_arena[\s\S]*StableFingerprintProjectionRejected fingerprint_error/,
    "stable push must free records, stable index, and the projected key arena if fingerprint projection fails",
);
assert.match(
    source,
    /SelfhostMemoTraitProofStorePushErrorKind::StableDuplicate:[\s\S]*SelfhostMemoTraitProofStorePushErrorKind::StableDuplicate:[\s\S]*true/,
    "push error equality must compare StableDuplicate explicitly instead of relying on wildcard behavior",
);
assert.match(
    source,
    /Result::Err index_error:[\s\S]*v::free v::vec_push_error_vec index_error[\s\S]*v::free next_records[\s\S]*selfhost_canonical_type_key_arena_free next_key_arena[\s\S]*selfhost_memo_trait_proof_store_push_error_from_std error/,
    "stable push must clean up both records and key arena if the stable index append fails after the record append",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_lookup_record[\s\S]*selfhost_canonical_type_key_project_from_arena[\s\S]*selfhost_memo_trait_proof_store_find_projected[\s\S]*selfhost_canonical_type_key_arena_free lookup_arena/,
    "lookup must project the current TypeId into a temporary canonical key arena and free it after lookup",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_lookup_record_stable_key[\s\S]*selfhost_canonical_type_key_project_from_arena[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_result nominal_table &lookup_arena lookup_key[\s\S]*selfhost_memo_trait_proof_store_find_projected_stable[\s\S]*selfhost_canonical_type_key_arena_free lookup_arena/,
    "stable lookup must project the current TypeId, compute its stable fingerprint, run stable store lookup, and free the temporary arena",
);
assert.match(
    source,
    /fn selfhost_memo_trait_proof_store_lookup_record_kind[\s\S]*SelfhostMemoTraitStoredProofKind::KeyAndValue:[\s\S]*selfhost_memo_trait_stored_aggregate_proof_to_session record\.proof type_id[\s\S]*selfhost_memo_trait_aggregate_proof_to_record types proof[\s\S]*SelfhostMemoTraitStoredProofKind::KeyOnlyUnsupported:[\s\S]*ProofKindMismatch[\s\S]*SelfhostMemoTraitStoredProofKind::ValueOnlyUnsupported:[\s\S]*ProofKindMismatch/,
    "lookup must recreate a session proof and reuse the producer gate while failing closed for unsupported proof kinds",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_policy_eq record\.policy expected_policy[\s\S]*Result::Err SelfhostMemoTraitProofStoreLookupErrorKind::PolicyMismatch/,
    "lookup must reject stale solver policy instead of using a structurally matching proof",
);
assert.match(
    source,
    /fn selfhost_memo_trait_proof_store_find_projected_loop[\s\S]*\\idx\\saw_policy_mismatch:[\s\S]*ge idx n[\s\S]*saw_policy_mismatch[\s\S]*SelfhostMemoTraitProofStoreLookupErrorKind::PolicyMismatch[\s\S]*SelfhostMemoTraitProofStoreLookupErrorKind::MissingProof/,
    "lookup must remember stale policy matches and report PolicyMismatch only after the full record scan has no expected-policy hit",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_policy_eq record\.policy expected_policy[\s\S]*selfhost_memo_trait_proof_store_lookup_record_kind types record type_id[\s\S]*selfhost_memo_trait_proof_store_find_projected_loop[\s\S]*add idx 1 true/,
    "lookup must continue scanning after a stale-policy canonical key match because the key is canonical type key plus policy identity",
);
assert.match(
    source,
    /SelfhostMemoTraitProofStoreLookupErrorKind::ProducerRejected a_kind:[\s\S]*SelfhostMemoTraitProofStoreLookupErrorKind::ProducerRejected b_kind:[\s\S]*selfhost_memo_trait_evidence_produce_reject_kind_eq a_kind b_kind/,
    "lookup error equality must compare the producer reject payload and not only the outer ProducerRejected variant",
);
assert.match(
    source,
    /SelfhostMemoTraitProofStoreLookupErrorKind::StableFingerprintProjectionRejected a_kind:[\s\S]*SelfhostMemoTraitProofStoreLookupErrorKind::StableFingerprintProjectionRejected b_kind:[\s\S]*selfhost_memo_trait_canonical_fingerprint_error_kind_eq a_kind b_kind/,
    "lookup error equality must compare the stable fingerprint projection reject payload",
);
assert.match(
    source,
    /別 arena の canonical key[\s\S]*fn selfhost_memo_trait_proof_store_canonical_key_equal_cross/,
    "proof store must define cross-arena canonical key equality for lookup without mutating the store key arena",
);
assert.match(
    source,
    /fn selfhost_memo_trait_proof_store_canonical_nodes_equal_cross[\s\S]*SelfhostCanonicalTypeKeyNode::Applied[\s\S]*SelfhostCanonicalTypeKeyNode::Function/,
    "cross-arena canonical key equality must compare applied and function nodes structurally",
);
assert.match(
    source,
    /fn selfhost_memo_trait_proof_store_find_projected_stable_loop[\s\S]*selfhost_memo_trait_proof_store_canonical_key_equal_cross[\s\S]*selfhost_memo_trait_proof_store_policy_eq record\.policy expected_policy[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_eq record_fingerprint lookup_fingerprint[\s\S]*selfhost_memo_trait_proof_store_lookup_record_kind types record type_id/,
    "stable lookup must require canonical equality, policy equality, stable fingerprint equality, and producer gate validation",
);
assert.match(
    source,
    /fn selfhost_memo_trait_proof_store_find_projected_stable_index_loop[\s\S]*field::get_ref store "stable_index"[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_eq entry\.stable_fingerprint lookup_fingerprint[\s\S]*v::get records entry\.record_index[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_eq record_fingerprint lookup_fingerprint[\s\S]*selfhost_memo_trait_proof_store_canonical_key_equal_cross[\s\S]*selfhost_memo_trait_proof_store_policy_eq record\.policy expected_policy[\s\S]*selfhost_memo_trait_proof_store_lookup_record_kind types record type_id/,
    "stable index lookup must use fingerprint only to narrow candidates and must still validate record fingerprint, canonical equality, policy equality, and producer gate",
);
assert.match(
    source,
    /fn selfhost_memo_trait_proof_store_stable_full_scan_after_index[\s\S]*Result::Ok _record:[\s\S]*StableIndexMissing[\s\S]*Result::Err kind:[\s\S]*Result::Err kind/,
    "stable lookup must fail closed with StableIndexMissing if full scan can accept a proof that the sidecar index did not expose",
);
assert.match(
    source,
    /fn selfhost_memo_trait_proof_store_find_projected_stable[\s\S]*selfhost_memo_trait_proof_store_find_projected_stable_index_loop[\s\S]*Result::Ok evidence:[\s\S]*Result::Ok evidence[\s\S]*Result::Err index_error:[\s\S]*selfhost_memo_trait_proof_store_stable_fallback_for_index_error/,
    "stable lookup must try the stable sidecar index before the diagnostic-preserving full scan fallback",
);
assert.match(
    source,
    /Option::None:[\s\S]*selfhost_memo_trait_proof_store_find_projected_stable_loop[\s\S]*true saw_stable_mismatch/,
    "stable lookup must fail closed on legacy records that do not carry a stable fingerprint",
);
assert.match(
    source,
    /SelfhostMemoTraitProofStoreLookupErrorKind::StableIndexMissing:[\s\S]*SelfhostMemoTraitProofStoreLookupErrorKind::StableIndexMissing:[\s\S]*true/,
    "lookup error equality must compare StableIndexMissing explicitly instead of relying on a wildcard branch",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_lookup_result_is_accept summary\.found[\s\S]*PolicyMismatch[\s\S]*MissingProof[\s\S]*ProducerRejected SelfhostMemoTraitEvidenceProduceRejectKind::PrimitiveNotAggregate[\s\S]*ProofKindMismatch[\s\S]*selfhost_memo_trait_proof_store_lookup_result_is_accept summary\.stable_found[\s\S]*RecordStableFingerprintMissing[\s\S]*StableFingerprintMismatch[\s\S]*selfhost_memo_trait_proof_store_push_error_kind_eq \(unwrap_err summary\.stable_duplicate\) SelfhostMemoTraitProofStorePushErrorKind::StableDuplicate/,
    "stage0 must prove accepted lookup, stale policy rejection, missing key rejection, primitive fake proof rejection, unsupported proof kind rejection, stable lookup acceptance, legacy stable-fingerprint rejection, stable mismatch rejection, and stable duplicate rejection",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_canonical_type_fingerprint_eq\s+(?:entry|record_fingerprint)[\s\S]{0,240}Result::Ok evidence/,
    "stable lookup must not return Ok immediately after a fingerprint comparison without canonical equality, policy equality, and producer validation",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_stage0_build_mismatched_nominal_table[\s\S]*selfhost_memo_trait_proof_store_stage0_build_nominal_table_with_definition 42[\s\S]*selfhost_memo_trait_proof_store_lookup_record_stable_key &arena &mismatched_nominal_table &store3 policy named_id/,
    "stage0 must exercise stable fingerprint mismatch with a typed mismatched nominal table",
);
assert.match(
    source,
    /fn selfhost_memo_trait_proof_store_stage0_duplicate_rejection[\s\S]*selfhost_memo_trait_proof_store_new[\s\S]*selfhost_memo_trait_proof_store_push_stable_key types nominal_table store0 policy type_id proof[\s\S]*selfhost_memo_trait_proof_store_push_stable_key types nominal_table store1 policy type_id proof[\s\S]*Result::Err kind/,
    "stage0 must exercise duplicate stable proof insertion through the public stable push API",
);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|memo_trait_definition_key|core\/check\/module/,
    "proof store stable fingerprint path must not use source text, spans, display names, diagnostics, lexemes, or checker-layer definition key producers as authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "memo trait proof store policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait proof store contract passed");
