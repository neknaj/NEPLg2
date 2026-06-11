#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_source_evidence_producer.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";

const source = read(relPath);
const facade = read(facadeRelPath);
const tySource = read(tySourceRelPath);
const proofStore = read(proofStoreRelPath);

assert.match(
    facade,
    /^pub #import "\.\/module\/memo_trait_source_evidence_producer" as \*$/m,
    "module checker facade must expose the typed memo trait source evidence producer",
);
assert.match(
    source,
    /# check\/module\/memo_trait_source_evidence_producer[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "seed producer must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /source spelling、span、display name、path suffix、diagnostic text は accepted source identity の authority にしません/,
    "seed producer must reject source spelling/span/display metadata as accepted authority",
);
assert.match(
    source,
    /full public surface materializer ではありません[\s\S]*re-export、trait body、method signature normalization、stable nominal key、serialized `.neplmeta` \/ `.neplproof` 入力/,
    "seed producer must state the full public surface materializer residual explicitly",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableSourceModuleSeed:[\s\S]*module_identity_hash %Option i32[\s\S]*public_surface_hash %Option i32/,
    "module seed must keep module identity and public surface fingerprints as named Option fields",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableSourceTraitSeed:[\s\S]*kind %SelfhostMemoTraitSourceKind[\s\S]*visibility %SelfhostModuleDeclarationVisibility[\s\S]*declaration_ordinal %Option i32[\s\S]*normalized_signature_hash %Option i32/,
    "trait seed must keep kind, visibility, declaration ordinal, and normalized signature as typed fields",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableSourceSeedTable:[\s\S]*memo_key %Option SelfhostMemoTraitStableSourceTraitSeed[\s\S]*memo_value %Option SelfhostMemoTraitStableSourceTraitSeed[\s\S]*duplicate_memo_key %bool[\s\S]*duplicate_memo_value %bool/,
    "seed table must preserve missing and duplicate state for MemoKey and MemoValue",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitStableSourceSeedErrorKind:[\s\S]*ModuleIdentityFingerprintMissing[\s\S]*ModulePublicSurfaceFingerprintMissing[\s\S]*MemoKeySeedMissing[\s\S]*MemoValueSeedMissing[\s\S]*MemoKeySeedDuplicate[\s\S]*MemoValueSeedDuplicate[\s\S]*MemoKeySeedKindMismatch[\s\S]*MemoValueSeedKindMismatch[\s\S]*MemoKeyVisibilityPrivate[\s\S]*MemoValueVisibilityPrivate[\s\S]*MemoKeyNormalizedSignatureFingerprintMissing[\s\S]*MemoValueNormalizedSignatureFingerprintPlaceholder[\s\S]*ModuleDerivedFingerprintPlaceholder[\s\S]*MemoKeyDerivedSymbolFingerprintPlaceholder[\s\S]*MemoValueDerivedSignatureFingerprintPlaceholder/,
    "seed producer must keep missing, placeholder, duplicate, kind mismatch, visibility, and signature failures as enum variants",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_seed_fingerprint_value_result[\s\S]*Option::Some value:[\s\S]*eq value 0[\s\S]*Result::Err placeholder_error[\s\S]*Option::None:[\s\S]*Result::Err missing_error/,
    "seed producer must reject missing and placeholder seed fingerprints separately",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_seed_derived_fingerprint_result[\s\S]*eq fingerprint 0[\s\S]*Result::Err placeholder_error[\s\S]*Result::Ok fingerprint/,
    "seed producer must reject derived zero fingerprints before returning a public evidence table",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_seed_module_hash_result[\s\S]*ModuleDerivedFingerprintPlaceholder/,
    "module seed folding must reject a derived zero module fingerprint in the seed producer",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_key_seed_to_evidence_result[\s\S]*MemoKeyDerivedSymbolFingerprintPlaceholder[\s\S]*MemoKeyDerivedSignatureFingerprintPlaceholder/,
    "MemoKey seed folding must reject derived zero symbol and signature fingerprints",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_value_seed_to_evidence_result[\s\S]*MemoValueDerivedSymbolFingerprintPlaceholder[\s\S]*MemoValueDerivedSignatureFingerprintPlaceholder/,
    "MemoValue seed folding must reject derived zero symbol and signature fingerprints",
);
for (const fnName of [
    "selfhost_memo_trait_stable_source_module_seed_new",
    "selfhost_memo_trait_stable_source_trait_seed_new",
    "selfhost_memo_trait_stable_source_seed_table_empty",
]) {
    const block = source.match(new RegExp(`//: ${fnName}:[\\s\\S]*?pub fn ${fnName}`));
    assert.ok(block, `${fnName} must have a doc comment`);
    for (const heading of ["[目的/もくてき]:", "[契約/けいやく]:", "[現状/げんじょう]:", "[計算量/けいさんりょう]:"]) {
        assert.ok(block[0].includes(heading), `${fnName} doc comment must include ${heading}`);
    }
}
assert.match(
    source,
    /selfhost_memo_trait_stable_source_evidence_table_from_seed_table_result[\s\S]*seed_table\.duplicate_memo_key[\s\S]*MemoKeySeedDuplicate[\s\S]*seed_table\.duplicate_memo_value[\s\S]*MemoValueSeedDuplicate[\s\S]*selfhost_memo_trait_stable_source_seed_module_hash_result[\s\S]*seed_table\.memo_key[\s\S]*seed_table\.memo_value/,
    "seed producer must reject duplicate seed slots and require module, MemoKey, and MemoValue seeds",
);
const keySeedConversion = source.match(/selfhost_memo_trait_stable_source_key_seed_to_evidence_result[\s\S]*?\/\/: selfhost_memo_trait_stable_source_value_seed_to_evidence_result/);
assert.ok(keySeedConversion, "MemoKey seed conversion function must exist");
for (const required of [
    "SelfhostMemoTraitSourceKind::MemoKeyTrait",
    "MemoKeySeedKindMismatch",
    "MemoKeyVisibilityPrivate",
    "MemoKeyDeclarationOrdinalMissing",
    "MemoKeyNormalizedSignatureFingerprintMissing",
    "selfhost_memo_trait_stable_source_fingerprint_evidence_new",
]) {
    assert.match(keySeedConversion[0], new RegExp(required.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")), `MemoKey seed conversion must contain ${required}`);
}
const valueSeedConversion = source.match(/selfhost_memo_trait_stable_source_value_seed_to_evidence_result[\s\S]*?\/\/: selfhost_memo_trait_stable_source_seed_table_records_to_evidence_result/);
assert.ok(valueSeedConversion, "MemoValue seed conversion function must exist");
for (const required of [
    "SelfhostMemoTraitSourceKind::MemoValueTrait",
    "MemoValueSeedKindMismatch",
    "MemoValueVisibilityPrivate",
    "MemoValueDeclarationOrdinalMissing",
    "MemoValueNormalizedSignatureFingerprintMissing",
    "selfhost_memo_trait_stable_source_fingerprint_evidence_new",
]) {
    assert.match(valueSeedConversion[0], new RegExp(required.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")), `MemoValue seed conversion must contain ${required}`);
}
assert.match(
    source,
    /pub fn selfhost_memo_trait_stable_source_evidence_table_from_seed_table_result %fn SelfhostMemoTraitStableSourceModuleSeed fn SelfhostMemoTraitStableSourceSeedTable Result SelfhostMemoTraitStableSourceEvidenceTable SelfhostMemoTraitStableSourceSeedErrorKind/,
    "seed producer must expose a Result-returning evidence table API",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_from_seed_evidence_result[\s\S]*selfhost_memo_trait_stable_source_evidence_table_from_seed_table_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_stable_evidence_result/,
    "seed producer must connect to the existing stable source fingerprint gate rather than bypassing it",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_source_identity_new/,
    "seed producer must not construct accepted source identities directly",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_definition_source_record_new:[\s\S]*\n\s+true/,
    "seed producer must not directly create signature_available=true definition source records",
);
assert.doesNotMatch(
    tySource,
    /memo_trait_source_evidence_producer|#import "neplg2\/core\/check\/module\/memo_trait_source_evidence_producer"/,
    "core/ty memo trait source registry must not depend on the checker-layer seed producer",
);
assert.doesNotMatch(
    proofStore,
    /memo_trait_source_evidence_producer|selfhost_memo_trait_trusted_source_registry_from_seed_evidence_result/,
    "proof store must not depend on seed producer output directly",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "seed producer policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait source evidence producer contract passed");
