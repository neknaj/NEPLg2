#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_source_fingerprint.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";

const source = read(relPath);
const facade = read(facadeRelPath);
const tySource = read(tySourceRelPath);
const proofStore = read(proofStoreRelPath);

assert.match(
    facade,
    /^pub #import "\.\/module\/memo_trait_source_fingerprint" as \*$/m,
    "module checker facade must expose the stable memo trait source fingerprint producer",
);
assert.match(
    source,
    /# check\/module\/memo_trait_source_fingerprint[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "stable source producer must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /fingerprint の計算規則そのものはこの module に置きません[\s\S]*source text、display name、path suffix、diagnostic text を accepted identity の authority にしません/,
    "stable source producer must state that it consumes typed evidence rather than trusting source text or display metadata",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:lower|hir|resource|backend)\//,
    "stable source producer must not depend on lowering, HIR, Resource IR, or backend layers",
);
assert.doesNotMatch(
    tySource,
    /memo_trait_source_fingerprint|#import "neplg2\/core\/check\/module\/memo_trait_source_fingerprint"/,
    "core/ty memo trait source registry must not depend on the checker-layer stable source producer",
);
assert.doesNotMatch(
    proofStore,
    /memo_trait_source_fingerprint|selfhost_memo_trait_trusted_source_registry_from_stable_evidence_result/,
    "proof store must not depend on stable source producer output directly",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableSourceFingerprintEvidence:[\s\S]*module_hash %Option i32[\s\S]*symbol_hash %Option i32[\s\S]*signature_hash %Option i32/,
    "stable fingerprint evidence must wrap optional module/symbol/signature fingerprints in a typed payload",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableSourceEvidenceRecord:[\s\S]*kind %SelfhostMemoTraitSourceKind[\s\S]*fingerprint %SelfhostMemoTraitStableSourceFingerprintEvidence/,
    "stable evidence record must carry typed source kind and typed fingerprint evidence separately",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_fingerprint_evidence_new[\s\S]*SelfhostMemoTraitStableSourceFingerprintEvidence module_hash symbol_hash signature_hash/,
    "stable fingerprint evidence must be constructed through a dedicated constructor instead of passing raw integer options as an unlabelled tuple",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableSourceEvidenceTable:[\s\S]*memo_key %Option SelfhostMemoTraitStableSourceEvidenceRecord[\s\S]*memo_value %Option SelfhostMemoTraitStableSourceEvidenceRecord[\s\S]*duplicate_memo_key %bool[\s\S]*duplicate_memo_value %bool/,
    "stable evidence table must track missing and duplicate evidence separately for MemoKey and MemoValue",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitStableSourceProduceErrorKind:[\s\S]*MemoKeyCandidateMissing[\s\S]*MemoValueCandidateMissing[\s\S]*MemoKeyCandidateDuplicate[\s\S]*MemoValueCandidateDuplicate[\s\S]*MemoKeyEvidenceMissing[\s\S]*MemoValueEvidenceMissing[\s\S]*MemoKeyEvidenceDuplicate[\s\S]*MemoValueEvidenceDuplicate[\s\S]*MemoKeyModuleFingerprintMissing[\s\S]*MemoKeyModuleFingerprintPlaceholder[\s\S]*MemoValueSignatureFingerprintMissing[\s\S]*MemoValueSignatureFingerprintPlaceholder/,
    "stable source producer must keep candidate, evidence, missing fingerprint, and placeholder fingerprint failures as typed enum variants",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_fingerprint_value_result[\s\S]*Option::Some value:[\s\S]*eq value 0[\s\S]*Result::Err placeholder_error[\s\S]*Option::None:[\s\S]*Result::Err missing_error/,
    "stable source producer must reject placeholder fingerprint 0 separately from missing fingerprints",
);
assert.match(
    source,
    /selfhost_memo_trait_definition_source_table_from_stable_evidence_result[\s\S]*candidates\.duplicate_memo_key[\s\S]*MemoKeyCandidateDuplicate[\s\S]*candidates\.duplicate_memo_value[\s\S]*MemoValueCandidateDuplicate[\s\S]*evidence\.duplicate_memo_key[\s\S]*MemoKeyEvidenceDuplicate[\s\S]*evidence\.duplicate_memo_value[\s\S]*MemoValueEvidenceDuplicate/,
    "stable source producer must reject duplicate candidate and evidence slots before accepting any record",
);
assert.match(
    source,
    /match candidates\.memo_key:[\s\S]*Option::Some _key_candidate:[\s\S]*match candidates\.memo_value:[\s\S]*Option::Some _value_candidate:[\s\S]*match evidence\.memo_key:[\s\S]*Option::Some key_evidence:[\s\S]*match evidence\.memo_value:[\s\S]*Option::Some value_evidence/,
    "stable source producer must require both scanner candidates and both stable evidence records",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_key_evidence_to_record[\s\S]*SelfhostMemoTraitSourceKind::MemoKeyTrait[\s\S]*selfhost_memo_trait_stable_source_fingerprint_value_result evidence\.fingerprint\.module_hash[\s\S]*MemoKeyModuleFingerprintMissing[\s\S]*MemoKeyModuleFingerprintPlaceholder[\s\S]*Result::Ok module_hash:[\s\S]*selfhost_memo_trait_stable_source_fingerprint_value_result evidence\.fingerprint\.symbol_hash[\s\S]*MemoKeySymbolFingerprintMissing[\s\S]*MemoKeySymbolFingerprintPlaceholder[\s\S]*Result::Ok symbol_hash:[\s\S]*selfhost_memo_trait_stable_source_fingerprint_value_result evidence\.fingerprint\.signature_hash[\s\S]*MemoKeySignatureFingerprintMissing[\s\S]*MemoKeySignatureFingerprintPlaceholder[\s\S]*Result::Ok signature_hash:[\s\S]*selfhost_memo_trait_definition_source_record_new:[\s\S]*true/,
    "MemoKey evidence must become signature_available=true only after all fingerprints are present and non-placeholder",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_value_evidence_to_record[\s\S]*SelfhostMemoTraitSourceKind::MemoValueTrait[\s\S]*selfhost_memo_trait_stable_source_fingerprint_value_result evidence\.fingerprint\.module_hash[\s\S]*MemoValueModuleFingerprintMissing[\s\S]*MemoValueModuleFingerprintPlaceholder[\s\S]*Result::Ok module_hash:[\s\S]*selfhost_memo_trait_stable_source_fingerprint_value_result evidence\.fingerprint\.symbol_hash[\s\S]*MemoValueSymbolFingerprintMissing[\s\S]*MemoValueSymbolFingerprintPlaceholder[\s\S]*Result::Ok symbol_hash:[\s\S]*selfhost_memo_trait_stable_source_fingerprint_value_result evidence\.fingerprint\.signature_hash[\s\S]*MemoValueSignatureFingerprintMissing[\s\S]*MemoValueSignatureFingerprintPlaceholder[\s\S]*Result::Ok signature_hash:[\s\S]*selfhost_memo_trait_definition_source_record_new:[\s\S]*true/,
    "MemoValue evidence must become signature_available=true only after all fingerprints are present and non-placeholder",
);
assert.match(
    source,
    /candidate table の fingerprint は使いません[\s\S]*candidate table は presence \/ duplicate evidence としてだけ扱います/,
    "stable source producer must not reuse scanner placeholder fingerprints as trusted source identity payloads",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_source_identity_new/,
    "stable source producer must not construct source identities directly; existing memo_trait_source validator must do final materialization",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_from_stable_evidence_result[\s\S]*selfhost_memo_trait_definition_source_table_from_stable_evidence_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_definition_table/,
    "stable source producer must feed the existing table-backed trusted registry validator instead of bypassing it",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_stage0[\s\S]*accepted_registry[\s\S]*missing_candidate_rejected[\s\S]*duplicate_candidate_rejected[\s\S]*missing_evidence_rejected[\s\S]*duplicate_evidence_rejected[\s\S]*missing_fingerprint_rejected[\s\S]*placeholder_fingerprint_rejected/,
    "stage0 smoke must cover accepted and fail-closed producer paths including placeholder fingerprint rejection",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_source_fingerprint_evidence_new \(some 0\)[\s\S]*placeholder_fingerprint_rejected/,
    "stage0 smoke must include a some 0 fingerprint fixture that is rejected",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "stable source producer policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait stable source fingerprint contract passed");
