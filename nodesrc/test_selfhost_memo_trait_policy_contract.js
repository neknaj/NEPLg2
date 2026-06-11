#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_policy.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const proofStore = readRepoFile(repoRoot, proofStoreRelPath);

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_policy" as \*$/m,
    "ty facade must re-export the memo trait policy split module",
);
assert.match(
    proofStore,
    /^#import "\.\/memo_trait_policy" as \*$/m,
    "memo trait proof store must import the typed memo trait policy module",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "memo trait policy must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /# ty\/memo_trait_policy[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "memo trait policy module documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /診断表示名、source text、path suffix は authority にしません[\s\S]*表示は後続 diagnostic layer の責務/,
    "memo trait policy must keep source identity separate from diagnostic display text",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitSourceKind:[\s\S]*MemoKeyTrait[\s\S]*MemoValueTrait/,
    "memo trait policy must distinguish MemoKey and MemoValue trait sources with an enum",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitSourceIdentity:[\s\S]*kind %SelfhostMemoTraitSourceKind[\s\S]*module_hash %i32[\s\S]*symbol_hash %i32[\s\S]*signature_hash %i32/,
    "memo trait policy must wrap source fingerprint payloads in a typed source identity",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitSourceIdentitySet:[\s\S]*memo_key %SelfhostMemoTraitSourceIdentity[\s\S]*memo_value %SelfhostMemoTraitSourceIdentity/,
    "memo trait policy must carry MemoKey and MemoValue source identities separately",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitRuleIdentity:[\s\S]*schema_version %i32[\s\S]*solver_version %i32[\s\S]*primitive_rule_hash %i32[\s\S]*aggregate_rule_hash %i32[\s\S]*hazard_rule_hash %i32/,
    "memo trait policy must split solver rule identity into typed rule components",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitProofStorePolicy:[\s\S]*sources %SelfhostMemoTraitSourceIdentitySet[\s\S]*rules %SelfhostMemoTraitRuleIdentity/,
    "proof store policy must be composed from source identity set and rule identity",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_source_kind_eq[\s\S]*SelfhostMemoTraitSourceKind::MemoKeyTrait:[\s\S]*SelfhostMemoTraitSourceKind::MemoValueTrait:[\s\S]*pub fn selfhost_memo_trait_source_identity_eq/,
    "memo trait policy equality must explicitly compare source kind variants without relying on a numeric tag",
);
assert.match(
    source,
    /selfhost_memo_trait_source_identity_eq[\s\S]*selfhost_memo_trait_source_kind_eq a\.kind b\.kind[\s\S]*eq a\.module_hash b\.module_hash[\s\S]*eq a\.symbol_hash b\.symbol_hash[\s\S]*eq a\.signature_hash b\.signature_hash/,
    "memo trait source identity equality must compare kind and every fingerprint payload",
);
assert.match(
    source,
    /selfhost_memo_trait_rule_identity_eq[\s\S]*eq a\.schema_version b\.schema_version[\s\S]*eq a\.solver_version b\.solver_version[\s\S]*eq a\.primitive_rule_hash b\.primitive_rule_hash[\s\S]*eq a\.aggregate_rule_hash b\.aggregate_rule_hash[\s\S]*eq a\.hazard_rule_hash b\.hazard_rule_hash/,
    "memo trait rule identity equality must compare schema, solver, primitive, aggregate, and hazard rule components",
);
assert.match(
    source,
    /selfhost_memo_trait_proof_store_policy_eq[\s\S]*selfhost_memo_trait_source_identity_set_eq a\.sources b\.sources[\s\S]*selfhost_memo_trait_rule_identity_eq a\.rules b\.rules/,
    "proof store policy equality must compare source and rule identity as typed values",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_proof_store_policy_new %fn SelfhostMemoTraitSourceIdentitySet fn SelfhostMemoTraitRuleIdentity SelfhostMemoTraitProofStorePolicy/,
    "proof store policy constructor must accept typed source identity set and typed rule identity",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_proof_store_policy_new %fn i32 fn i32 fn i32 fn i32|pub struct SelfhostMemoTraitProofStorePolicy:[\s\S]*trait_source_hash %i32|pub struct SelfhostMemoTraitProofStorePolicy:[\s\S]*rule_hash %i32/,
    "memo trait policy module must not reintroduce the old raw-i32 proof store policy constructor or raw hash fields",
);
assert.doesNotMatch(
    proofStore,
    /trait_source_hash %i32|rule_hash %i32|selfhost_memo_trait_proof_store_policy_new %fn i32 fn i32 fn i32 fn i32/,
    "memo trait proof store must not expose raw trait_source_hash/rule_hash policy fields or raw-i32 policy constructor",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "memo trait policy source policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait policy contract passed");
