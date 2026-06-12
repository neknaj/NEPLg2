#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_canonical_key_payload.nepl";
const preseedRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl";
const artifactRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const preseed = readRepoFile(repoRoot, preseedRelPath);
const artifact = readRepoFile(repoRoot, artifactRelPath);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");
const preseedCodeOnly = preseed
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_canonical_key_payload" as \*$/m,
    "ty facade must re-export the canonical key payload hash boundary",
);
assert.match(
    source,
    /# ty\/memo_trait_canonical_key_payload[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "canonical key payload module documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /record key に入っている `canonical_payload_hash` を信用せず[\s\S]*decoded payload[\s\S]*source text、span、path、display name、diagnostic text、lexeme/,
    "module docs must state that payload hash is recomputed from decoded payload and not from source-text authority",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:check|lower|hir|resource|backend)\//,
    "canonical key payload hash must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitCanonicalKeyPayloadErrorKind:[\s\S]*MissingCanonicalNode[\s\S]*MissingCanonicalArgument[\s\S]*InvalidCanonicalArgumentRange[\s\S]*CanonicalPayloadHashPlaceholder[\s\S]*TraversalFuelExhausted[\s\S]*MissingNominalKey[\s\S]*DuplicateNominalKey[\s\S]*TypeParameterUnsupported[\s\S]*FunctionTypeUnsupported/,
    "payload hash projection must expose typed errors for malformed keys, traversal bounds, nominal table failures, and unsupported variants",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitCanonicalKeyPayloadHash:[\s\S]*schema_version %i32[\s\S]*payload_hash %i32/,
    "payload hash must carry schema version with the hash value",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_canonical_key_payload_schema_version %fn void i32 \\void:[\s\S]*\n    1/,
    "payload schema version must be an explicit payload boundary function",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_hash_eq[\s\S]*eq a\.schema_version b\.schema_version[\s\S]*eq a\.payload_hash b\.payload_hash/,
    "payload hash equality must compare schema version and hash value",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_nominal_hash[\s\S]*key\.schema_version[\s\S]*key\.module_fingerprint[\s\S]*key\.definition_fingerprint[\s\S]*key\.constructor_ordinal[\s\S]*key\.type_arity[\s\S]*key\.nominal_key_hash/,
    "stable nominal key material must include all stable nominal key fields rather than only the derived hash",
);
assert.match(
    source,
    /fn selfhost_memo_trait_canonical_key_payload_node_hash_result[\s\S]*SelfhostCanonicalTypeKeyNode::Primitive[\s\S]*SelfhostCanonicalTypeKeyNode::Named[\s\S]*SelfhostCanonicalTypeKeyNode::Parameter[\s\S]*TypeParameterUnsupported[\s\S]*SelfhostCanonicalTypeKeyNode::Applied[\s\S]*SelfhostCanonicalTypeKeyNode::Function[\s\S]*FunctionTypeUnsupported/,
    "payload hash projection must handle every canonical key node variant explicitly and fail closed for parameter and function nodes",
);
assert.match(
    source,
    /SelfhostCanonicalTypeKeyNode::Named nominal_id:[\s\S]*selfhost_memo_trait_stable_nominal_key_table_find_result nominal_table nominal_id/,
    "named canonical key payloads must resolve through the stable nominal key table",
);
assert.match(
    source,
    /SelfhostCanonicalTypeKeyNode::Applied applied:[\s\S]*selfhost_memo_trait_stable_nominal_key_table_find_result nominal_table applied\.nominal_id[\s\S]*selfhost_memo_trait_canonical_key_payload_args_hash_result/,
    "applied canonical key payloads must resolve the stable nominal key and recursively hash type arguments",
);
assert.match(
    source,
    /SelfhostCanonicalTypeKeyNode::Applied applied:[\s\S]*selfhost_memo_trait_canonical_key_payload_nonzero_result selfhost_memo_trait_canonical_key_payload_mix4 791104 selfhost_memo_trait_canonical_key_payload_schema_version[\s\S]*selfhost_memo_trait_canonical_key_payload_mix2 applied\.args\.arg_count args_hash/,
    "applied canonical key payload hash must mix the payload schema version with the nominal identity, argument count, and folded argument hash",
);
assert.match(
    source,
    /fn selfhost_memo_trait_canonical_key_payload_args_hash_result[\s\S]*selfhost_canonical_type_key_arg_range_is_valid[\s\S]*selfhost_canonical_type_key_arena_arg arena args idx[\s\S]*MissingCanonicalArgument/,
    "argument payload hashing must validate canonical arg ranges and fail closed on missing argument slots",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_hash_result[\s\S]*selfhost_canonical_type_key_arena_node_len arena[\s\S]*selfhost_canonical_type_key_arena_arg_len arena[\s\S]*selfhost_memo_trait_canonical_key_payload_node_hash_result nominal_table arena root fuel/,
    "public payload hash projection must derive traversal fuel from arena node and argument counts",
);
assert.match(
    source,
    /SelfhostMemoTraitCanonicalKeyPayloadStage0Summary:[\s\S]*named_payload[\s\S]*applied_payload[\s\S]*missing_nominal[\s\S]*duplicate_nominal[\s\S]*parameter_unsupported[\s\S]*function_unsupported[\s\S]*missing_node[\s\S]*missing_argument[\s\S]*invalid_argument_range[\s\S]*fuel_exhausted/,
    "stage0 summary must cover accepted payloads and the representative fail-closed paths",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_payload_stage0_fuel_projection[\s\S]*SelfhostCanonicalTypeKeyNode::Applied[\s\S]*v::push args0 root[\s\S]*selfhost_memo_trait_canonical_key_payload_stage0_fuel_projection_from_parts/,
    "stage0 must construct a cyclic canonical key arena for traversal fuel smoke",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme/,
    "accepted payload hash code must not use source text, spans, paths, display names, diagnostics, or lexemes as authority",
);
assert.doesNotMatch(
    codeOnly,
    /SelfhostTypeId|SelfhostMemoTraitProofStoreStableIdentity|SelfhostMemoTraitProofStoreRecord|SelfhostMemoTraitProofStoreStableIndexEntry/,
    "payload hash code must not serialize arena-local TypeId, proof store records, or store-local stable identities",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "canonical key payload policy must not introduce line-count or doc-comment-length restrictions",
);
assert.match(
    preseed,
    /^#import "\.\/memo_trait_canonical_key_payload" as \*$/m,
    "preseed bridge must import the canonical key payload producer",
);
assert.match(
    preseed,
    /selfhost_memo_trait_canonical_key_payload_hash_result &nominal_table &materialized_key_arena materialized_key_id[\s\S]*selfhost_memo_trait_canonical_key_payload_hash_value payload_hash[\s\S]*SelfhostMemoTraitNeplProofRecordKey selfhost_memo_trait_canonical_key_payload_schema_version fingerprint canonical_payload_hash policy/,
    "preseed stage0 must derive the record key payload hash from the materialized canonical key payload producer",
);
assert.match(
    preseed,
    /pub fn selfhost_memo_trait_neplproof_record_preseed_decision_materialized %fn &SelfhostMemoTraitProofStore fn &SelfhostMemoTraitStableNominalKeyTable fn &SelfhostCanonicalTypeKeyArena fn SelfhostCanonicalTypeKeyId fn SelfhostMemoTraitProofStorePolicy fn SelfhostMemoTraitCanonicalTypeFingerprint fn SelfhostMemoTraitNeplProofRecord Result/,
    "preseed public API must require the stable nominal table and materialized canonical key arena instead of a caller-supplied raw payload hash",
);
assert.match(
    preseed,
    /selfhost_memo_trait_neplproof_record_preseed_decision_materialized_checked[\s\S]*selfhost_memo_trait_canonical_key_payload_hash_result nominal_table materialized_key_arena materialized_key_id[\s\S]*selfhost_memo_trait_canonical_key_payload_hash_value payload_hash[\s\S]*not eq materialized_canonical_payload_hash record\.key\.canonical_payload_hash/,
    "preseed checked path must recompute the canonical payload hash before comparing it with the decoded record key",
);
assert.doesNotMatch(
    preseed,
    /pub fn selfhost_memo_trait_neplproof_record_preseed_decision_materialized[\s\S]*\\store\\materialized_key_arena\\materialized_key_id\\expected_policy\\materialized_fingerprint\\materialized_canonical_payload_hash\\record/,
    "preseed public API must not expose the old caller-supplied materialized_canonical_payload_hash parameter",
);
assert.doesNotMatch(
    preseedCodeOnly,
    /\b3003\b/,
    "preseed bridge code must not keep the old fixed placeholder canonical payload hash",
);
assert.match(
    artifact,
    /^#import "\.\/memo_trait_canonical_key_payload" as \*$/m,
    "artifact schema must depend on the canonical key payload schema boundary",
);
assert.match(
    artifact,
    /not eq canonical_payload_schema_version selfhost_memo_trait_canonical_key_payload_schema_version[\s\S]*not eq canonical_payload_schema_version selfhost_memo_trait_canonical_type_fingerprint_schema_version/,
    "artifact schema validation must check the payload schema authority and the current fingerprint schema compatibility",
);

console.log("selfhost memo trait canonical key payload contract passed");
