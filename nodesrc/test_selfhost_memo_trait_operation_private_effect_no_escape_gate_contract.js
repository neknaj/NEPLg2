#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function stripDocComments(src) {
    return src
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

function functionBlock(src, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?${kind}\\s+${name}\\b`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${kind} ${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

function assertDocBeforeTopLevel(src, docSnippet, declarationSnippet) {
    const declarationIndex = src.indexOf(declarationSnippet);
    assert.notEqual(declarationIndex, -1, `missing declaration ${declarationSnippet}`);
    const docIndex = src.lastIndexOf(docSnippet, declarationIndex);
    assert.notEqual(docIndex, -1, `missing doc snippet before ${declarationSnippet}`);
    const between = src.slice(docIndex, declarationIndex);
    assert.doesNotMatch(
        between,
        /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/m,
        `${docSnippet} must document the immediately following top-level declaration`,
    );
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_no_escape_gate.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";

const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const runner = read(runnerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_private_effect_no_escape_gate",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "private effect no-escape gate must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("既存 `SelfhostMemoTraitOperationMethodBodyFact` は resolver 用の最終 fact") &&
        source.includes("body root identity を保持しません") &&
        source.includes("fact table を後から補正せず、HIR root から fact を作る直前で proof を適用します"),
    "docs must explain why the gate is placed before MethodBodyFact loses body identity",
);
assert.ok(
    source.includes("`SelfhostTypeId`、operation、body module fingerprint、body root、effect、元 escape state の完全一致") &&
        source.includes("`SelfhostTypeId` だけ、operation だけ、source span / display name / body hash だけの proof reuse は認めません"),
    "docs must require exact typed proof identity and reject weak source/display/hash authority",
);
assert.ok(
    source.includes("`body_module_fingerprint == 0` は placeholder origin なので proof record と identity-bearing input の両方で拒否します"),
    "docs must reject placeholder module origin on both proof and identity-bearing input",
);
assert.ok(
    source.includes("1 つの `&SelfhostHirModule` と 1 つの `body_module_fingerprint`") &&
        source.includes("複数 module 由来の method body root を 1 回の呼び出しに混ぜず") &&
        source.includes("module ごとの table に分割してからこの関数を呼びます"),
    "docs must keep scan-record table builds scoped to one HIR module fingerprint per call",
);
assert.ok(
    source.includes("`PrivateState + NotApplicable` または `PrivateCache + NotApplicable` のときだけ") &&
        source.includes("matching proof が `Proven` の場合だけ `NoEscapeProven`") &&
        source.includes("`Refuted` は `MayEscape`") &&
        source.includes("`Missing` / `Unknown` は元の `NotApplicable`"),
    "docs must define fail-closed private effect proof handling",
);
assert.ok(
    source.includes("operation evidence、aggregate proof、memo_call backend bytes") &&
        source.includes("Resource proof production は作りません"),
    "docs must keep evidence/backend/artifact/resource proof production out of this boundary",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_private_effect_no_escape_gate/,
    "private effect no-escape gate must remain facade-private until full proof orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_private_effect_no_escape_gate/,
    "checker-layer private effect no-escape gate must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_private_effect_no_escape_gate_contract.js"),
    "source policy runner must execute the private effect no-escape gate contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_method_body_effect_checker\" as *",
        "#import \"./memo_trait_operation_method_body_fact_input_scan\" as *",
        "#import \"./memo_trait_operation_method_body_fact_producer\" as *",
        "#import \"./memo_trait_operation_method_body_fact_table_inputs\" as *",
        "#import \"./memo_trait_operation_method_body_resolver\" as *",
        "#import \"./memo_trait_operation_purity_gate\" as *",
    ],
    "private effect no-escape gate must depend on HIR root identity, typed effects, method body fact builders/resolver, and purity check types only",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:backend|memo_call|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_operation_drop_no_escape_gate|memo_trait_operation_drop_resource)/,
    "private effect no-escape gate must not import backend, memo_call, proof store/artifact, canonical-key, public-surface, evidence producer, impl table, Drop no-escape, or Drop resource proof layers",
);
assertDocBeforeTopLevel(
    source,
    "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofKey",
    "pub struct SelfhostMemoTraitOperationPrivateEffectNoEscapeProofKey:",
);
assertDocBeforeTopLevel(
    source,
    "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofRecord",
    "pub struct SelfhostMemoTraitOperationPrivateEffectNoEscapeProofRecord:",
);
assertDocBeforeTopLevel(
    source,
    "SelfhostMemoTraitOperationPrivateEffectNoEscapeGateErrorKind",
    "pub enum SelfhostMemoTraitOperationPrivateEffectNoEscapeGateErrorKind:",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationPrivateEffectNoEscapeProofKey:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
    ],
    "proof key must include type, operation, module origin, body root, effect, and original escape state",
);
const keyBlock = topLevelBlock(source, "struct", "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofKey");
assert.doesNotMatch(
    keyBlock,
    /payload_hash|signature_hash|body_hash|public_surface|source_text|source_span|source_path|\bspan\b|\bpath\b|\bname\b|diagnostic|message|text/i,
    "proof key must not use source/display/hash authority instead of typed body identity",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_no_escape_proof_key_eq"),
    [
        "selfhost_type_id_eq a.type_id b.type_id",
        "selfhost_memo_trait_operation_evidence_kind_eq a.operation b.operation",
        "eq a.body_module_fingerprint b.body_module_fingerprint",
        "eq selfhost_hir_expr_id_index a.body_root selfhost_hir_expr_id_index b.body_root",
        "selfhost_effect_kind_eq a.effect b.effect",
        "selfhost_memo_trait_operation_private_effect_no_escape_escape_state_eq a.escape b.escape",
    ],
    "proof key equality must compare type, operation, module origin, body root, effect, and escape",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_no_escape_proof_table_push"),
    [
        "eq record.key.body_module_fingerprint 0",
        "selfhost_memo_trait_operation_private_effect_no_escape_proof_table_free table",
        "Result::Err SelfhostMemoTraitOperationPrivateEffectNoEscapeGateErrorKind::ProofBodyModuleFingerprintPlaceholder",
        "Result::Err e:",
        "let error %StdErrorKind field::get e \"error\"",
        "v::free v::vec_push_error_vec e",
        "Result::Err SelfhostMemoTraitOperationPrivateEffectNoEscapeGateErrorKind::ProofRecordPushFailed error",
    ],
    "proof table push must reject placeholder module origins and recover Vec owners on push failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_no_escape_proof_lookup_loop"),
    [
        "selfhost_memo_trait_operation_private_effect_no_escape_proof_key_eq record.key key",
        "Option::Some _existing:",
        "Result::Err SelfhostMemoTraitOperationPrivateEffectNoEscapeGateErrorKind::ProofDuplicate",
        "Option::None:",
        "some record.status",
    ],
    "proof lookup must reject duplicate matching proofs and must not use first-wins",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_no_escape_proof_lookup_not_applicable_result"),
    [
        "pub fn selfhost_memo_trait_operation_private_effect_no_escape_proof_lookup_not_applicable_result",
        "selfhost_memo_trait_operation_private_effect_no_escape_proof_key_new type_id operation body_module_fingerprint body_root effect SelfhostEffectEscapeState::NotApplicable",
        "selfhost_memo_trait_operation_private_effect_no_escape_proof_lookup_result proofs key",
    ],
    "public proof lookup must be a narrow NotApplicable slot wrapper over the private exact-key lookup",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_no_escape_transform_summary_result"),
    [
        "eq body_module_fingerprint 0",
        "Result::Err SelfhostMemoTraitOperationPrivateEffectNoEscapeGateErrorKind::ProofBodyModuleFingerprintPlaceholder",
        "SelfhostEffectKind::Pure:",
        "selfhost_memo_trait_operation_private_effect_no_escape_fact_from_summary_result type_id operation summary",
        "SelfhostEffectKind::InternalAlloc:",
        "selfhost_memo_trait_operation_private_effect_no_escape_fact_from_summary_result type_id operation summary",
        "SelfhostEffectKind::PrivateState:",
        "selfhost_memo_trait_operation_private_effect_no_escape_transform_private_summary_result proofs type_id operation body_module_fingerprint body_root summary",
        "SelfhostEffectKind::PrivateCache:",
        "selfhost_memo_trait_operation_private_effect_no_escape_transform_private_summary_result proofs type_id operation body_module_fingerprint body_root summary",
    ],
    "summary transform must reject placeholder identity and lookup proofs only for private effects",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_no_escape_transform_private_summary_result"),
    [
        "SelfhostEffectEscapeState::NotApplicable:",
        "selfhost_memo_trait_operation_private_effect_no_escape_proof_key_new type_id operation body_module_fingerprint body_root summary.effect summary.escape",
        "selfhost_memo_trait_operation_private_effect_no_escape_proof_lookup_result proofs key",
        "selfhost_memo_trait_operation_private_effect_no_escape_escape_from_status summary.escape status",
        "SelfhostEffectEscapeState::NoEscapeProven:",
        "Result::Err SelfhostMemoTraitOperationPrivateEffectNoEscapeGateErrorKind::UnexpectedPreProvenNoEscape",
        "SelfhostEffectEscapeState::MayEscape:",
        "selfhost_memo_trait_operation_private_effect_no_escape_fact_from_summary_result type_id operation summary",
    ],
    "private summary transform must require original NotApplicable, reject pre-proven bypass, and pass MayEscape through",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_no_escape_escape_from_status"),
    [
        "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofStatus::Proven:",
        "SelfhostEffectEscapeState::NoEscapeProven",
        "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofStatus::Refuted:",
        "SelfhostEffectEscapeState::MayEscape",
        "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofStatus::Missing:",
        "original",
        "SelfhostMemoTraitOperationPrivateEffectNoEscapeProofStatus::Unknown:",
        "original",
    ],
    "proof status mapping must only prove Proven and must leave Missing/Unknown unmasked",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_no_escape_push_hir_root_result"),
    [
        "selfhost_memo_trait_operation_method_body_effect_check_result module root fuel",
        "selfhost_memo_trait_operation_private_effect_no_escape_transform_summary_result proofs type_id operation body_module_fingerprint root summary",
        "selfhost_memo_trait_operation_method_body_table_push table fact",
        "Result::Err e:",
        "selfhost_memo_trait_operation_method_body_table_free table",
        "Result::Err e",
        "Result::Err effect_error:",
        "selfhost_memo_trait_operation_method_body_table_free table",
    ],
    "push_hir_root_result must apply proof before final MethodBodyFact table push and clean owners on failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_no_escape_table_from_scan_records_result"),
    [
        "selfhost_memo_trait_operation_method_body_fact_inputs_from_scan_records_result source",
        "selfhost_memo_trait_operation_method_body_table_new",
        "selfhost_memo_trait_operation_private_effect_no_escape_build_from_inputs_result table module body_module_fingerprint &inputs proofs",
        "selfhost_memo_trait_operation_method_body_fact_build_input_table_free inputs",
    ],
    "scan-record API must turn roots into facts before MethodBodyFact loses identity and must close build input owners",
);
assert.ok(
    source.includes("mismatched_root_check") &&
        source.includes("other_body_module_fingerprint") &&
        source.includes("mismatched_module_key") &&
        source.includes("mismatched_module_record") &&
        source.includes("mismatched_module_check") &&
        source.includes("duplicate_rejected") &&
        source.includes("preproven_rejected"),
    "stage0 must cover root/module key mismatch, duplicate proof, and pre-proven bypass rejection",
);
