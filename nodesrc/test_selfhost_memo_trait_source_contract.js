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

function sectionBetween(start, end) {
    const startIndex = source.indexOf(start);
    assert.notEqual(startIndex, -1, `missing section start: ${start}`);
    const endIndex = source.indexOf(end, startIndex + start.length);
    assert.notEqual(endIndex, -1, `missing section end: ${end}`);
    return source.slice(startIndex, endIndex);
}

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
    /Phase 1 では module \/ symbol \/ signature の fingerprint payload は事前に用意した i32 値です[\s\S]*typed definition source table と materializer の境界[\s\S]*full trait definition table ではありません/,
    "memo trait source registry must document that current table-backed prepared fingerprints are not a full trait definition table scanner",
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
    /pub struct SelfhostMemoTraitDefinitionFingerprint:[\s\S]*module_hash %i32[\s\S]*symbol_hash %i32[\s\S]*signature_hash %i32/,
    "memo trait source registry must group module/symbol/signature fingerprints before source identity materialization",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitDefinitionSourceRecord:[\s\S]*kind %SelfhostMemoTraitSourceKind[\s\S]*fingerprint %SelfhostMemoTraitDefinitionFingerprint[\s\S]*signature_available %bool/,
    "memo trait source registry must materialize source identity from a typed definition source record",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitDefinitionSourceTable:[\s\S]*memo_key %Option SelfhostMemoTraitDefinitionSourceRecord[\s\S]*memo_value %Option SelfhostMemoTraitDefinitionSourceRecord[\s\S]*duplicate_memo_key %bool[\s\S]*duplicate_memo_value %bool/,
    "memo trait source registry must collect MemoKey/MemoValue definition records through a typed table before registry materialization",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitSourceMaterializeErrorKind:[\s\S]*KindMismatch[\s\S]*SignatureMissing/,
    "memo trait source materializer must reject kind mismatch and missing signature with enum errors",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitTrustedSourceRegistryErrorKind:[\s\S]*MemoKeyDefinitionMissing[\s\S]*MemoValueDefinitionMissing[\s\S]*MemoKeyDefinitionDuplicate[\s\S]*MemoValueDefinitionDuplicate[\s\S]*MemoKeySourceRejected %SelfhostMemoTraitSourceMaterializeErrorKind[\s\S]*MemoValueSourceRejected %SelfhostMemoTraitSourceMaterializeErrorKind/,
    "trusted source registry current path must keep missing, duplicate, MemoKey, and MemoValue materialization errors distinct",
);
assert.match(
    source,
    /selfhost_memo_trait_definition_source_table_empty[\s\S]*SelfhostMemoTraitDefinitionSourceTable none none false false[\s\S]*selfhost_memo_trait_definition_source_table_add_record[\s\S]*SelfhostMemoTraitSourceKind::MemoKeyTrait:[\s\S]*selfhost_memo_trait_definition_source_table_with_key[\s\S]*SelfhostMemoTraitSourceKind::MemoValueTrait:[\s\S]*selfhost_memo_trait_definition_source_table_with_value/,
    "memo trait definition source table must start empty and classify records by typed source kind",
);
assert.match(
    source,
    /selfhost_memo_trait_definition_source_table_with_key[\s\S]*Option::Some _existing:[\s\S]*true[\s\S]*selfhost_memo_trait_definition_source_table_with_value[\s\S]*Option::Some _existing:[\s\S]*true/,
    "memo trait definition source table must retain duplicate flags instead of silently replacing existing records",
);
assert.match(
    source,
    /selfhost_memo_trait_source_identity_from_definition %fn SelfhostMemoTraitSourceKind fn SelfhostMemoTraitDefinitionSourceRecord Result SelfhostMemoTraitSourceIdentity SelfhostMemoTraitSourceMaterializeErrorKind/,
    "memo trait source identity materialization must be a Result-returning typed validator",
);
assert.match(
    source,
    /Result::Err SelfhostMemoTraitSourceMaterializeErrorKind::KindMismatch/,
    "memo trait source identity materialization must reject expected-kind mismatches",
);
assert.match(
    source,
    /Result::Err SelfhostMemoTraitSourceMaterializeErrorKind::SignatureMissing/,
    "memo trait source identity materialization must reject records without a trusted signature fingerprint",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_memo_key_definition_source_current[\s\S]*SelfhostMemoTraitSourceKind::MemoKeyTrait[\s\S]*selfhost_memo_trait_definition_fingerprint_new 10 11 12[\s\S]*selfhost_memo_trait_trusted_memo_value_definition_source_current[\s\S]*SelfhostMemoTraitSourceKind::MemoValueTrait[\s\S]*selfhost_memo_trait_definition_fingerprint_new 20 21 22/,
    "current MemoKey/MemoValue source fingerprints must be prepared definition records before materialization",
);
assert.match(
    source,
    /selfhost_memo_trait_definition_source_table_current[\s\S]*selfhost_memo_trait_definition_source_table_empty[\s\S]*selfhost_memo_trait_definition_source_table_add_record table0 selfhost_memo_trait_trusted_memo_key_definition_source_current[\s\S]*selfhost_memo_trait_definition_source_table_add_record table1 selfhost_memo_trait_trusted_memo_value_definition_source_current/,
    "current MemoKey/MemoValue source records must be assembled through the definition source table",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_memo_key_source_identity_current_result[\s\S]*selfhost_memo_trait_source_identity_from_definition SelfhostMemoTraitSourceKind::MemoKeyTrait selfhost_memo_trait_trusted_memo_key_definition_source_current[\s\S]*selfhost_memo_trait_trusted_memo_value_source_identity_current_result[\s\S]*selfhost_memo_trait_source_identity_from_definition SelfhostMemoTraitSourceKind::MemoValueTrait selfhost_memo_trait_trusted_memo_value_definition_source_current/,
    "private current source identity smoke helpers must be built through the materializer rather than raw constructor calls",
);
assert.match(
    source,
    /^fn selfhost_memo_trait_trusted_memo_key_source_identity_current_result %fn void Result SelfhostMemoTraitSourceIdentity SelfhostMemoTraitSourceMaterializeErrorKind/m,
    "MemoKey current source identity smoke helper must be private so callers cannot bypass the table-backed registry validator",
);
assert.match(
    source,
    /^fn selfhost_memo_trait_trusted_memo_value_source_identity_current_result %fn void Result SelfhostMemoTraitSourceIdentity SelfhostMemoTraitSourceMaterializeErrorKind/m,
    "MemoValue current source identity smoke helper must be private so callers cannot bypass the table-backed registry validator",
);
assert.doesNotMatch(
    source,
    /^pub fn selfhost_memo_trait_trusted_memo_key_source_identity_current_result/m,
    "MemoKey current source identity helper must not be a public policy authority",
);
assert.doesNotMatch(
    source,
    /^pub fn selfhost_memo_trait_trusted_memo_value_source_identity_current_result/m,
    "MemoValue current source identity helper must not be a public policy authority",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_current_result %fn void Result SelfhostMemoTraitTrustedSourceRegistry SelfhostMemoTraitTrustedSourceRegistryErrorKind/,
    "current trusted registry must expose a Result-returning API that preserves which source failed",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_from_definition_table[\s\S]*table\.duplicate_memo_key[\s\S]*MemoKeyDefinitionDuplicate[\s\S]*table\.duplicate_memo_value[\s\S]*MemoValueDefinitionDuplicate[\s\S]*Option::Some key_record:[\s\S]*Option::Some value_record:[\s\S]*selfhost_memo_trait_trusted_source_registry_from_definition_records key_record value_record[\s\S]*MemoValueDefinitionMissing[\s\S]*MemoKeyDefinitionMissing/,
    "trusted source registry must validate table missing and duplicate states before materializing source identities",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_current_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_definition_table selfhost_memo_trait_definition_source_table_current/,
    "current trusted source registry must be table-backed rather than directly combining prepared source helpers",
);
assert.match(
    source,
    /SelfhostMemoTraitTrustedSourceRegistryErrorKind::MemoKeySourceRejected key_error/,
    "current trusted registry Result API must report MemoKey materialization failures",
);
assert.match(
    source,
    /SelfhostMemoTraitTrustedSourceRegistryErrorKind::MemoValueSourceRejected value_error/,
    "current trusted registry Result API must report MemoValue materialization failures",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_identity_set_current_result %fn void Result SelfhostMemoTraitSourceIdentitySet SelfhostMemoTraitTrustedSourceRegistryErrorKind/,
    "current trusted source set must expose a Result-returning API for fail-closed callers such as the proof store",
);
assert.doesNotMatch(
    sectionBetween(
        "selfhost_memo_trait_trusted_memo_key_source_identity_current_result",
        "selfhost_memo_trait_trusted_memo_value_source_identity_current_result",
    ),
    /selfhost_memo_trait_source_identity_new/,
    "MemoKey current Result helper must not bypass the materializer with a raw source identity constructor",
);
assert.doesNotMatch(
    sectionBetween(
        "selfhost_memo_trait_trusted_memo_value_source_identity_current_result",
        "selfhost_memo_trait_trusted_source_registry_current_result",
    ),
    /selfhost_memo_trait_source_identity_new/,
    "MemoValue current Result helper must not bypass the materializer with a raw source identity constructor",
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
    /selfhost_memo_trait_trusted_source_registry_from_definition_records[\s\S]*selfhost_memo_trait_source_identity_from_definition SelfhostMemoTraitSourceKind::MemoKeyTrait key_record[\s\S]*selfhost_memo_trait_source_identity_from_definition SelfhostMemoTraitSourceKind::MemoValueTrait value_record/,
    "memo trait source registry must materialize MemoKey and MemoValue records with distinct expected source kinds",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_error_kind_eq[\s\S]*SelfhostMemoTraitTrustedSourceRegistryErrorKind::MemoKeySourceRejected a_error:[\s\S]*SelfhostMemoTraitTrustedSourceRegistryErrorKind::MemoKeySourceRejected b_error:[\s\S]*selfhost_memo_trait_source_materialize_error_kind_eq a_error b_error[\s\S]*SelfhostMemoTraitTrustedSourceRegistryErrorKind::MemoValueSourceRejected a_error:[\s\S]*SelfhostMemoTraitTrustedSourceRegistryErrorKind::MemoValueSourceRejected b_error:[\s\S]*selfhost_memo_trait_source_materialize_error_kind_eq a_error b_error/,
    "registry error equality must use explicit enum matching and compare materializer error payloads",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_sources %fn &SelfhostMemoTraitTrustedSourceRegistry SelfhostMemoTraitSourceIdentitySet[\s\S]*selfhost_memo_trait_source_identity_set_new \*field::get_ref registry "memo_key" \*field::get_ref registry "memo_value"/,
    "memo trait source registry must borrow the registry and project fields into the policy source identity set without consuming the registry",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_is_current %fn &SelfhostMemoTraitTrustedSourceRegistry bool[\s\S]*selfhost_memo_trait_trusted_source_identity_set_current_result:[\s\S]*Result::Ok current_sources:[\s\S]*selfhost_memo_trait_source_identity_set_eq:[\s\S]*selfhost_memo_trait_trusted_source_registry_sources registry[\s\S]*current_sources[\s\S]*Result::Err _error:[\s\S]*false/,
    "memo trait source registry must compare borrowed snapshots with typed source identity equality and fail closed on current source errors",
);
assert.doesNotMatch(
    source,
    /#intrinsic "unreachable"/,
    "memo trait source registry must not hide current materializer failures behind unreachable wrappers",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_stage0[\s\S]*kind_mismatch_rejected[\s\S]*signature_missing_rejected[\s\S]*table_missing_key_rejected[\s\S]*table_missing_value_rejected[\s\S]*table_duplicate_key_rejected[\s\S]*table_duplicate_value_rejected[\s\S]*table_key_source_rejected[\s\S]*table_value_source_rejected[\s\S]*MemoKeyDefinitionMissing[\s\S]*MemoValueDefinitionMissing[\s\S]*MemoKeyDefinitionDuplicate[\s\S]*MemoValueDefinitionDuplicate[\s\S]*MemoKeySourceRejected[\s\S]*MemoValueSourceRejected/,
    "memo trait source registry stage0 must exercise materializer and key/value table fail-closed rejection cases",
);
assert.match(
    proofStore,
    /selfhost_memo_trait_trusted_source_identity_set_current_result[\s\S]*Result::Ok sources:[\s\S]*selfhost_memo_trait_rule_identity_new[\s\S]*Result::Err _source_error:[\s\S]*selfhost_memo_trait_proof_store_stage0_abort_with_store arena store0/,
    "memo trait proof store stage0 must obtain source identity through the Result-returning trusted registry and fail closed on registry errors",
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
