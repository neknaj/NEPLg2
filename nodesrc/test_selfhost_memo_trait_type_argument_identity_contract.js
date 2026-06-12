#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_type_argument_identity.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_type_argument_identity" as \*$/m,
    "ty facade must re-export the stable type argument identity boundary",
);
assert.match(
    source,
    /# ty\/memo_trait_type_argument_identity[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[戻\/もど\]り\[値\/ち\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "stable type argument identity docs must record purpose, contract, return values, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /proof acceptance ではありません[\s\S]*MemoKey \/ MemoValue proof の受理、policy equality、producer gate、proof store lookup は後続の責務/,
    "stable type argument identity docs must state that identity creation is not proof acceptance",
);
assert.match(
    source,
    /aggregate hash は compact lookup key であり、最終的な同一性 authority ではありません[\s\S]*ordered entry vector と schema 付き hash を一緒に確認/,
    "stable type argument identity docs must state that aggregate hash alone is not final identity authority",
);
assert.match(
    source,
    /source text、span、path、display name、diagnostic text、lexeme、session-local `SelfhostTypeId`、store-local `SelfhostCanonicalTypeKeyId` は accepted authority にしません/,
    "module docs must exclude source-display data and local compiler ids from accepted authority",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_canonical_key" as \*$/m,
    "type argument identity must reuse canonical key fingerprint projection",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_canonical_key_payload" as \*$/m,
    "type argument identity must reuse canonical payload hash projection",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:check|lower|hir|resource|backend)\//,
    "type argument identity must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.doesNotMatch(
    source,
    /^#import "\.\/memo_trait_(?:proof|canonical_key_payload_codec|artifact_word_codec)/m,
    "type argument identity must not depend on proof, artifact, reader, serializer, preseed, stable-map, or payload-codec modules",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableTypeArgumentIdentityHash:[\s\S]*schema_version %i32[\s\S]*identity_hash %i32/,
    "aggregate identity hash must carry schema version with the hash value",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableTypeArgumentIdentityEntry:[\s\S]*ordinal %i32[\s\S]*canonical_fingerprint %SelfhostMemoTraitCanonicalTypeFingerprint[\s\S]*canonical_payload_hash %SelfhostMemoTraitCanonicalKeyPayloadHash/,
    "each type argument entry must carry ordinal, canonical fingerprint, and canonical payload hash",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableTypeArgumentIdentity:[\s\S]*entries %Vec SelfhostMemoTraitStableTypeArgumentIdentityEntry[\s\S]*identity_hash %SelfhostMemoTraitStableTypeArgumentIdentityHash/,
    "identity owner must carry full entry material and the aggregate hash",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitStableTypeArgumentIdentityErrorKind:[\s\S]*TypeArgumentMissing[\s\S]*ProjectionRejected %SelfhostCanonicalTypeKeyProjectErrorKind[\s\S]*FingerprintRejected %SelfhostMemoTraitCanonicalFingerprintErrorKind[\s\S]*PayloadRejected %SelfhostMemoTraitCanonicalKeyPayloadErrorKind[\s\S]*EntryPushFailed %StdErrorKind[\s\S]*IdentityHashPlaceholder/,
    "identity creation failures must use typed enum variants and preserve nested boundary errors",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_type_argument_identity_hash_eq[\s\S]*eq a\.schema_version b\.schema_version[\s\S]*eq a\.identity_hash b\.identity_hash/,
    "aggregate hash equality must compare schema version and hash value",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_stable_type_argument_identity_entry_result[\s\S]*selfhost_canonical_type_key_project_from_arena types type_id[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_result nominal_table &key_arena key_id[\s\S]*selfhost_memo_trait_canonical_key_payload_hash_result nominal_table &key_arena key_id/,
    "each type argument must be projected through canonical key, canonical fingerprint, and canonical payload hash",
);
assert.match(
    source,
    /Result::Err payload_error:[\s\S]*selfhost_canonical_type_key_arena_free key_arena[\s\S]*PayloadRejected payload_error[\s\S]*Result::Err fingerprint_error:[\s\S]*selfhost_canonical_type_key_arena_free key_arena[\s\S]*FingerprintRejected fingerprint_error/,
    "temporary canonical key arenas must be closed on fingerprint and payload failures",
);
assert.match(
    source,
    /Result::Ok payload_hash:[\s\S]*selfhost_canonical_type_key_arena_free key_arena[\s\S]*Result::Ok SelfhostMemoTraitStableTypeArgumentIdentityEntry ordinal fingerprint payload_hash/,
    "temporary canonical key arenas must also be closed on successful entry creation",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_stable_type_argument_identity_free[\s\S]*v::free field::get identity "entries"/,
    "owned identity free API must close the entry vector owner",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_stable_type_argument_identity_result[\s\S]*let n %i32 v::len type_args[\s\S]*selfhost_memo_trait_stable_type_argument_identity_initial_hash n[\s\S]*selfhost_memo_trait_stable_type_argument_identity_loop/,
    "public identity producer must preserve argument count and delegate ordered entry construction to the loop",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_type_argument_identity_initial_hash[\s\S]*selfhost_memo_trait_stable_type_argument_identity_schema_version argument_count/,
    "aggregate hash material must include schema version and argument count",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_type_argument_identity_loop[\s\S]*v::get type_args idx[\s\S]*selfhost_memo_trait_stable_type_argument_identity_entry_result types nominal_table type_id idx[\s\S]*selfhost_memo_trait_stable_type_argument_identity_fold_entry_result current_hash entry[\s\S]*v::push entries entry/,
    "identity loop must derive ordinal from vector index, fold the entry into the aggregate hash, and keep the entry vector",
);
assert.match(
    source,
    /Option::None:[\s\S]*v::free entries[\s\S]*TypeArgumentMissing/,
    "identity loop must fail closed if the type argument vector does not provide an indexed entry",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_stable_type_argument_identity_hash_from_args_result[\s\S]*selfhost_memo_trait_stable_type_argument_identity_result types nominal_table type_args[\s\S]*v::free entries[\s\S]*Result::Ok hash/,
    "hash-only API must reuse the full identity producer and close the entry owner",
);
assert.match(
    source,
    /SelfhostMemoTraitStableTypeArgumentIdentityStage0Summary:[\s\S]*empty_accepted[\s\S]*single_accepted[\s\S]*accepted[\s\S]*order_sensitive[\s\S]*missing_nominal[\s\S]*duplicate_nominal[\s\S]*parameter_unsupported[\s\S]*function_unsupported/,
    "stage0 summary must cover empty, single, ordered two-argument identity, order sensitivity, missing/duplicate nominal key, parameter rejection, and function rejection",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_type_argument_identity_stage0_finish_with_empty_single[\s\S]*v::new[\s\S]*empty_accepted[\s\S]*stage0_one_arg i32_id[\s\S]*single_accepted[\s\S]*stage0_summary_new empty_accepted single_accepted accepted order_sensitive/,
    "stage0 must exercise empty and single type argument vectors through the public identity producer",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_type_argument_identity_stage0_order_sensitive[\s\S]*not selfhost_memo_trait_stable_type_argument_identity_hash_eq accepted_hash reversed_hash/,
    "stage0 must verify that generic type argument order changes the stable identity",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_type_argument_identity_stage0_build_duplicate_table[\s\S]*selfhost_memo_trait_stable_nominal_key_table_push[\s\S]*selfhost_memo_trait_stable_nominal_key_table_push/,
    "stage0 must construct a duplicate nominal-key table through the stable nominal key table API",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_type_argument_identity_stage0_collect_duplicate[\s\S]*duplicate_nominal[\s\S]*selfhost_memo_trait_stable_nominal_key_table_free duplicate_table[\s\S]*stage0_collect_parameter/,
    "stage0 must route duplicate nominal-key rejection through the public identity producer and close the duplicate table owner",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_type_argument_identity_stage0_collect_parameter[\s\S]*parameter_unsupported[\s\S]*function_unsupported[\s\S]*stage0_finish_with_empty_single/,
    "stage0 must route unsupported parameter and function types through the public identity producer",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme|file_path|module_path/,
    "type argument identity code must not use source text, spans, paths, display names, diagnostics, or lexemes as authority",
);
assert.doesNotMatch(
    topLevelBlock(source, "SelfhostMemoTraitStableTypeArgumentIdentityEntry"),
    /SelfhostTypeId|SelfhostCanonicalTypeKeyId|SelfhostNamedTypeId/,
    "identity entries must not store session-local TypeId, canonical key ids, or nominal ids",
);
assert.doesNotMatch(
    topLevelBlock(source, "SelfhostMemoTraitStableTypeArgumentIdentityHash"),
    /SelfhostTypeId|SelfhostCanonicalTypeKeyId|SelfhostNamedTypeId/,
    "identity hash must not store session-local TypeId, canonical key ids, or nominal ids",
);
assert.doesNotMatch(
    topLevelBlock(source, "SelfhostMemoTraitStableTypeArgumentIdentity"),
    /SelfhostTypeId|SelfhostCanonicalTypeKeyId|SelfhostNamedTypeId/,
    "identity owner must not store session-local TypeId, canonical key ids, or nominal ids",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限/,
    "type argument identity policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait stable type argument identity contract passed");

function topLevelBlock(text, name) {
    const pattern = new RegExp(`^pub struct ${name}:[\\s\\S]*?(?=\\n(?:pub\\s+)?(?:struct|enum|impl|fn)\\s|\\n//: [^\\n]*\\n(?:pub\\s+)?(?:struct|enum|impl|fn)\\s|\\s*$)`, "m");
    const match = text.match(pattern);
    assert.ok(match, `${name} must exist`);
    return match[0];
}
