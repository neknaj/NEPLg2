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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_drop_no_escape_gate.nepl";
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
        "# check/module/memo_trait_operation_drop_no_escape_gate",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "Drop no-escape gate must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("`SelfhostTypeId` だけを key にした proof reuse を禁止") &&
        source.includes("Drop body root") &&
        source.includes("effect、元 escape state"),
    "docs must explain that proof reuse is keyed by body identity and effect summary, not TypeId alone",
);
assert.ok(
    source.includes("`InternalAlloc + NotApplicable` だけが proof lookup の対象") &&
        source.includes("matching proof が `Proven` のときだけ `NoEscapeProven`") &&
        source.includes("`Missing` / `Unknown`") &&
        source.includes("no-escape 未証明を pure に mask しません"),
    "docs must define fail-closed InternalAlloc proof handling",
);
assert.ok(
    source.includes("`PureDrop`、`NoDropRequired`") &&
        source.includes("最終的な Drop evidence は既存の purity gate が作ります"),
    "docs must keep Drop evidence synthesis out of this boundary",
);
assertDocBeforeTopLevel(
    source,
    "SelfhostMemoTraitOperationDropNoEscapeProofKey",
    "pub struct SelfhostMemoTraitOperationDropNoEscapeProofKey:",
);
assertDocBeforeTopLevel(
    source,
    "SelfhostMemoTraitOperationDropNoEscapeProofRecord",
    "pub struct SelfhostMemoTraitOperationDropNoEscapeProofRecord:",
);
assertDocBeforeTopLevel(
    source,
    "SelfhostMemoTraitOperationDropNoEscapeGateErrorKind",
    "pub enum SelfhostMemoTraitOperationDropNoEscapeGateErrorKind:",
);
assertDocBeforeTopLevel(
    source,
    "Clone for SelfhostMemoTraitOperationDropNoEscapeProofKey",
    "impl Clone for SelfhostMemoTraitOperationDropNoEscapeProofKey:",
);
assertDocBeforeTopLevel(
    source,
    "Copy for SelfhostMemoTraitOperationDropNoEscapeProofKey",
    "impl Copy for SelfhostMemoTraitOperationDropNoEscapeProofKey:",
);
assert.ok(
    source.includes("owner を含まない識別子 tuple") &&
        source.includes("table owner の copy ではありません") &&
        source.includes("owner lifecycle を緩めるものではありません"),
    "Clone/Copy impl docs must explain typed-payload-only ownership safety",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_drop_no_escape_gate/,
    "Drop no-escape gate must remain facade-private until full Resource proof orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_drop_no_escape_gate/,
    "checker-layer Drop no-escape gate must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_drop_no_escape_gate_contract.js"),
    "source policy runner must execute the Drop no-escape gate contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"./memo_trait_operation_drop_impl_resolver\" as *",
        "#import \"./memo_trait_operation_purity_gate\" as *",
    ],
    "Drop no-escape gate must depend only on typed HIR id, effect/type ids, Drop resolver facts, and purity gate check types",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|private_cache|private_state)/,
    "Drop no-escape gate must not import backend, proof store/artifact, canonical-key, public-surface, evidence producer, impl table, scanner/materializer, PrivateCache, or PrivateState layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationDropNoEscapeProofKey:",
        "type_id %SelfhostTypeId",
        "body_root %SelfhostHirExprId",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
    ],
    "proof key must include type, body root, effect, and original escape state",
);
const keyBlock = topLevelBlock(source, "struct", "SelfhostMemoTraitOperationDropNoEscapeProofKey");
assert.doesNotMatch(
    keyBlock,
    /payload_hash|signature_hash|body_hash|public_surface|source_text|source_span|source_path|\bspan\b|\bpath\b|\bname\b|diagnostic|message|text/i,
    "proof key must not use source/display/hash authority instead of typed body identity",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_no_escape_proof_key_eq"),
    [
        "selfhost_type_id_eq a.type_id b.type_id",
        "eq selfhost_hir_expr_id_index a.body_root selfhost_hir_expr_id_index b.body_root",
        "selfhost_effect_kind_eq a.effect b.effect",
        "selfhost_memo_trait_operation_drop_no_escape_escape_state_eq a.escape b.escape",
    ],
    "proof key equality must compare type, body root, effect, and escape",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_no_escape_proof_lookup_loop"),
    [
        "selfhost_memo_trait_operation_drop_no_escape_proof_key_eq record.key key",
        "Option::Some _existing:",
        "Result::Err SelfhostMemoTraitOperationDropNoEscapeGateErrorKind::ProofDuplicate",
        "Option::None:",
        "some record.status",
    ],
    "proof lookup must reject duplicate matching proofs and must not use first-wins",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_no_escape_transform_internal_alloc_result"),
    [
        "SelfhostEffectEscapeState::NotApplicable:",
        "selfhost_memo_trait_operation_drop_no_escape_fact_key fact",
        "selfhost_memo_trait_operation_drop_no_escape_proof_lookup_result proofs key",
        "SelfhostEffectEscapeState::NoEscapeProven:",
        "UnexpectedPreProvenNoEscape",
        "SelfhostEffectEscapeState::MayEscape:",
        "Result::Ok fact",
    ],
    "only InternalAlloc NotApplicable may be updated by matching proof, pre-proven input must fail closed, MayEscape must remain impure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_no_escape_internal_alloc_from_status"),
    [
        "SelfhostMemoTraitOperationDropNoEscapeProofStatus::Proven:",
        "SelfhostEffectEscapeState::NoEscapeProven",
        "SelfhostMemoTraitOperationDropNoEscapeProofStatus::Refuted:",
        "SelfhostEffectEscapeState::MayEscape",
        "SelfhostMemoTraitOperationDropNoEscapeProofStatus::Missing:",
        "fact",
        "SelfhostMemoTraitOperationDropNoEscapeProofStatus::Unknown:",
        "fact",
    ],
    "proof status mapping must only prove matching Proven, refute to MayEscape, and leave Missing/Unknown unmasked",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_no_escape_transform_fact_result"),
    [
        "SelfhostEffectKind::Pure:",
        "Result::Ok fact",
        "SelfhostEffectKind::InternalAlloc:",
        "selfhost_memo_trait_operation_drop_no_escape_transform_internal_alloc_result proofs fact",
        "SelfhostEffectKind::UnsafeMemory:",
        "Result::Ok fact",
        "SelfhostEffectKind::ExternalIo:",
        "Result::Ok fact",
        "SelfhostEffectKind::Nondet:",
        "Result::Ok fact",
    ],
    "gate must only change InternalAlloc escape state and must not weaken observable effects",
);
assert.doesNotMatch(
    code,
    /\bSelfhostMemoTraitOperationDropEvidence::(?:PureDrop|NoDropRequired|ImpureDrop|Unknown|Missing)\b|\bSelfhostMemoTraitOperationEvidenceRecord\b|\bSelfhostMemoTraitAggregateProof\b|\bSelfhostMemoTraitProofStore\b/,
    "Drop no-escape gate must not construct Drop evidence, operation evidence, aggregate proof, or proof store values",
);
assert.doesNotMatch(
    code,
    /\b(?:call\.name|expr\.span|field::get(?:_ref)?\s+[^\n]*"(?:name|span|source|path|diagnostic|message|text)")/,
    "accepted proof authority must not use call names, expression spans, source text, paths, or diagnostic text",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationDropNoEscapeGateErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "Drop no-escape gate errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "Drop no-escape gate APIs must return typed Result errors instead of bool/string errors",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "Drop no-escape gate policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation Drop no-escape gate contract passed");
