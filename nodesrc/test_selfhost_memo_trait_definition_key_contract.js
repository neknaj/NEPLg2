#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_definition_key.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const seedProducerRelPath = "stdlib/neplg2/core/check/module/memo_trait_source_evidence_producer.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";
const tySourceRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl";

const source = read(relPath);
const facade = read(facadeRelPath);
const seedProducer = read(seedProducerRelPath);
const proofStore = read(proofStoreRelPath);
const tySource = read(tySourceRelPath);

assert.match(
    source,
    /# check\/module\/memo_trait_definition_key[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "stable definition key producer must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /source text、span、lexeme、path suffix、display name、diagnostic text は accepted key の authority にしません/,
    "stable definition key producer must reject source spelling and display metadata as authority",
);
assert.match(
    source,
    /trusted registry や `signature_available=true` source record を作りません/,
    "stable definition key producer must not bypass the stable source registry gate",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableDefinitionKey:[\s\S]*kind %SelfhostMemoTraitSourceKind[\s\S]*schema_version %i32[\s\S]*module_fingerprint %i32[\s\S]*definition_key_hash %i32/,
    "stable definition key must keep kind, schema version, module fingerprint, and key hash as typed fields",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitStableDefinitionKeyErrorKind:[\s\S]*ModuleFingerprintPlaceholder %SelfhostMemoTraitSourceKind[\s\S]*DeclarationOrdinalMissing %SelfhostMemoTraitSourceKind[\s\S]*DeclarationOrdinalPlaceholder %SelfhostMemoTraitSourceKind[\s\S]*DefinitionKeyFingerprintPlaceholder %SelfhostMemoTraitSourceKind/,
    "stable definition key errors must preserve source kind payloads",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_stable_definition_key_result %fn SelfhostMemoTraitSourceKind fn i32 fn Option i32 Result SelfhostMemoTraitStableDefinitionKey SelfhostMemoTraitStableDefinitionKeyErrorKind/,
    "stable definition key producer must expose a Result-returning typed API",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_definition_key_result[\s\S]*eq module_fingerprint 0[\s\S]*ModuleFingerprintPlaceholder[\s\S]*Option::Some ordinal:[\s\S]*eq ordinal 0[\s\S]*DeclarationOrdinalPlaceholder[\s\S]*Option::None:[\s\S]*DeclarationOrdinalMissing/,
    "stable definition key producer must fail closed on placeholder module and missing or placeholder ordinal",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_definition_key_fold_result[\s\S]*selfhost_memo_trait_stable_definition_key_kind_code[\s\S]*selfhost_memo_trait_stable_definition_key_schema_version[\s\S]*DefinitionKeyFingerprintPlaceholder/,
    "stable definition key fold must include schema version, source kind, and derived zero rejection",
);
assert.match(
    source,
    /SelfhostMemoTraitSourceKind::MemoKeyTrait:[\s\S]*42101[\s\S]*SelfhostMemoTraitSourceKind::MemoValueTrait:[\s\S]*42102/,
    "MemoKey and MemoValue must use distinct key-space codes",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_stable_definition_key_eq[\s\S]*selfhost_memo_trait_source_kind_eq[\s\S]*schema_version[\s\S]*module_fingerprint[\s\S]*definition_key_hash/,
    "stable definition key equality must compare kind, schema version, and payload fields",
);
assert.doesNotMatch(
    facade,
    /memo_trait_definition_key/,
    "stable definition key producer must not be re-exported by the module checker facade before it becomes a stable public surface",
);
assert.match(
    seedProducer,
    /#import "\.\/memo_trait_definition_key" as \*/,
    "seed evidence producer must consume the stable definition key producer by direct internal import",
);
assert.match(
    seedProducer,
    /selfhost_memo_trait_stable_source_seed_definition_key_error_result[\s\S]*DeclarationOrdinalMissing[\s\S]*MemoKeyDeclarationOrdinalMissing[\s\S]*MemoValueDeclarationOrdinalMissing[\s\S]*DefinitionKeyFingerprintPlaceholder[\s\S]*MemoKeyDerivedSymbolFingerprintPlaceholder[\s\S]*MemoValueDerivedSymbolFingerprintPlaceholder/,
    "seed evidence producer must map stable definition key errors into its existing typed seed error surface",
);
assert.match(
    seedProducer,
    /selfhost_memo_trait_stable_source_key_seed_to_evidence_result[\s\S]*selfhost_memo_trait_stable_definition_key_result seed\.kind module_hash seed\.declaration_ordinal[\s\S]*definition_key\.definition_key_hash/,
    "MemoKey seed conversion must use the stable definition key hash as the source symbol fingerprint",
);
assert.match(
    seedProducer,
    /selfhost_memo_trait_stable_source_value_seed_to_evidence_result[\s\S]*selfhost_memo_trait_stable_definition_key_result seed\.kind module_hash seed\.declaration_ordinal[\s\S]*definition_key\.definition_key_hash/,
    "MemoValue seed conversion must use the stable definition key hash as the source symbol fingerprint",
);
assert.doesNotMatch(
    seedProducer,
    /let symbol_hash %i32 selfhost_memo_trait_stable_source_seed_mix3 module_hash kind_code declaration_ordinal/,
    "seed evidence producer must not construct source symbol fingerprints directly from raw ordinal values",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_source_identity_new|selfhost_memo_trait_definition_source_record_new:[\s\S]*\n\s+true/,
    "stable definition key producer must not construct accepted source identities or source records",
);
assert.doesNotMatch(
    source,
    /memo_trait_proof_store|SelfhostMemoTraitProofStore/,
    "stable definition key producer must not depend on proof store",
);
assert.doesNotMatch(
    proofStore,
    /memo_trait_definition_key|selfhost_memo_trait_stable_definition_key_result/,
    "proof store must stay behind the trusted source registry and not depend on checker-layer definition keys",
);
assert.doesNotMatch(
    tySource,
    /memo_trait_definition_key|selfhost_memo_trait_stable_definition_key_result/,
    "core/ty trusted source registry must not import the checker-layer definition key producer",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "stable definition key policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait stable definition key contract passed");
