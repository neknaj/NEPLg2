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
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const proofStore = readRepoFile(repoRoot, proofStoreRelPath);

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_source" as \*$/m,
    "ty facade must re-export the memo trait source registry split module",
);
assert.ok(
    TY_ROOT_REEXPORT_FILES.includes(relPath),
    "selfhost ty source registry must be listed as a root re-export source file",
);
assert.ok(
    TY_SPLIT_FILES.includes(relPath),
    "selfhost ty source registry must be listed as a split source file",
);
assert.match(
    proofStore,
    /^#import "\.\/memo_trait_source" as \*$/m,
    "memo trait proof store must import the trusted source registry instead of constructing source fingerprints locally",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|check|resource|backend)\//,
    "memo trait source registry must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /# ty\/memo_trait_source[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "memo trait source registry module documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /Phase 1 では module \/ symbol \/ signature の fingerprint payload は事前に用意した i32 値です[\s\S]*trait definition table から source text や public surface hash を走査する実装ではありません/,
    "memo trait source registry must document that current prepared fingerprints are not a full trait definition table materializer",
);
assert.match(
    source,
    /表示名、source path suffix、diagnostic message は accepted path の authority にしません/,
    "memo trait source registry must keep display metadata out of the accepted identity authority",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitTrustedSourceRegistry:[\s\S]*memo_key %SelfhostMemoTraitSourceIdentity[\s\S]*memo_value %SelfhostMemoTraitSourceIdentity/,
    "memo trait source registry must carry typed MemoKey and MemoValue source identities",
);
assert.match(
    source,
    /^fn selfhost_memo_trait_trusted_source_registry_new %fn SelfhostMemoTraitSourceIdentity fn SelfhostMemoTraitSourceIdentity SelfhostMemoTraitTrustedSourceRegistry/m,
    "memo trait source registry constructor must be private so callers cannot build swapped trusted registries",
);
assert.doesNotMatch(
    source,
    /^pub fn selfhost_memo_trait_trusted_source_registry_new/m,
    "memo trait source registry constructor must not be public; future artifact snapshots need a typed Result validator instead",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_memo_key_source_identity_current[\s\S]*SelfhostMemoTraitSourceKind::MemoKeyTrait[\s\S]*selfhost_memo_trait_trusted_memo_value_source_identity_current[\s\S]*SelfhostMemoTraitSourceKind::MemoValueTrait/,
    "memo trait source registry must construct MemoKey and MemoValue identities with distinct source kinds",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_sources %fn &SelfhostMemoTraitTrustedSourceRegistry SelfhostMemoTraitSourceIdentitySet[\s\S]*selfhost_memo_trait_source_identity_set_new \*field::get_ref registry "memo_key" \*field::get_ref registry "memo_value"/,
    "memo trait source registry must borrow the registry and project fields into the policy source identity set without consuming the registry",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_is_current %fn &SelfhostMemoTraitTrustedSourceRegistry bool[\s\S]*selfhost_memo_trait_source_identity_set_eq:[\s\S]*selfhost_memo_trait_trusted_source_registry_sources registry[\s\S]*selfhost_memo_trait_trusted_source_identity_set_current/,
    "memo trait source registry must compare borrowed snapshots with typed source identity equality",
);
assert.match(
    proofStore,
    /selfhost_memo_trait_trusted_source_identity_set_current[\s\S]*selfhost_memo_trait_rule_identity_new/,
    "memo trait proof store stage0 must obtain source identity from the trusted registry before building the policy",
);
assert.doesNotMatch(
    proofStore,
    /SelfhostMemoTraitSourceKind::MemoKeyTrait|SelfhostMemoTraitSourceKind::MemoValueTrait|selfhost_memo_trait_source_identity_new/,
    "memo trait proof store must not mention source kind constructors or manually construct MemoKey/MemoValue source identities from raw fingerprints",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "memo trait source registry policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait source registry contract passed");
