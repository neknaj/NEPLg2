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
    /store record は `SelfhostTypeId` を保持しません[\s\S]*現在の `SelfhostTypeArena` から対象 TypeId を canonical key へ再投影/,
    "proof store must document that persistent records do not store session-local TypeId and lookup reprojects the current TypeId",
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
assert.match(
    source,
    /pub struct SelfhostMemoTraitStoredAggregateProof:[\s\S]*fields %SelfhostMemoTraitAggregateFieldEvidence[\s\S]*copy_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*drop_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*eq_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*hash_proof %SelfhostMemoTraitAggregateProofStatus[\s\S]*hazard %SelfhostMemoTraitAggregateHazardEvidence[\s\S]*key_result %Result unit SelfhostMemoTraitRejectKind[\s\S]*value_result %Result unit SelfhostMemoTraitRejectKind/,
    "stored aggregate proof must carry proof payloads without SelfhostTypeId",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitProofStoreRecord:[\s\S]*key_id %SelfhostCanonicalTypeKeyId[\s\S]*policy %SelfhostMemoTraitProofStorePolicy[\s\S]*proof_kind %SelfhostMemoTraitStoredProofKind[\s\S]*proof %SelfhostMemoTraitStoredAggregateProof/,
    "proof store record must be keyed by canonical type key and policy, not by TypeId",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitProofStoreLookupErrorKind:[\s\S]*TypeProjectionFailed[\s\S]*MissingProof[\s\S]*PolicyMismatch[\s\S]*ProofKindMismatch[\s\S]*ProducerRejected %SelfhostMemoTraitEvidenceProduceRejectKind/,
    "proof store lookup must expose typed fail-closed errors for projection, key, policy, proof-kind, and producer rejection",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_lookup_record[\s\S]*selfhost_canonical_type_key_project_from_arena[\s\S]*selfhost_memo_trait_proof_store_find_projected[\s\S]*selfhost_canonical_type_key_arena_free lookup_arena/,
    "lookup must project the current TypeId into a temporary canonical key arena and free it after lookup",
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
    /selfhost_memo_trait_proof_store_lookup_result_is_accept summary\.found[\s\S]*PolicyMismatch[\s\S]*MissingProof[\s\S]*ProducerRejected SelfhostMemoTraitEvidenceProduceRejectKind::PrimitiveNotAggregate[\s\S]*ProofKindMismatch/,
    "stage0 must prove accepted lookup, stale policy rejection, missing key rejection, primitive fake proof rejection, and unsupported proof kind rejection",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "memo trait proof store policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait proof store contract passed");
