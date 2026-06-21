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

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_purity_gate.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_purity_gate",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "operation purity gate must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("`InternalAlloc` は、`NoEscapeProven` がある場合だけ pure evidence へ畳みます") &&
        source.includes("no-escape 証明なしに `Pure` へ mask しません"),
    "docs must require no-escape proof before masking InternalAlloc as pure",
);
assert.ok(
    source.includes("actual method body checker、Drop impl resolver、generic impl binder、private cache effect masking、Resource IR escape proof、full public surface orchestration は後続 stage の責務"),
    "docs must keep actual checker, resolver, private cache masking, Resource IR proof, and orchestration outside this gate",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、HIR、Resource IR、backend artifact、proof store record を authority にしません"),
    "docs must exclude source/display/diagnostic/module path/HIR/Resource/backend/proof-store authority",
);
assert.ok(
    source.includes("`DropImplAbsent` は Drop 実装がないことを上流 resolver が確認した fact です") &&
        source.includes("`DropImplAbsent` は単なる lookup miss を success にする fallback ではありません"),
    "docs must define DropImplAbsent as resolver-confirmed evidence, not fallback success",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_purity_gate/,
    "operation purity gate must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_purity_gate/,
    "checker-layer purity gate must not be registered in the ty source list",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/check/module/memo_trait_operation_evidence_producer\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
    ],
    "purity gate must depend only on producer evidence enums, typed effect facts, and operation kind",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_impl_table)/,
    "purity gate must not import HIR, Resource IR, backend, proof store, artifact, canonical-key, public-surface, public-impl-header, or impl-table layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationEffectPurityKind:",
        "Pure",
        "Unknown",
        "Impure",
    ],
    "effect purity must be a typed enum rather than bool or text",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationMethodBodyCheckKind:",
        "Present",
        "Missing",
        "Unknown",
        "NotRequired",
        "pub enum SelfhostMemoTraitOperationDropCheckKind:",
        "DropImplAbsent",
        "DropImplPresent",
        "Missing",
        "Unknown",
        "NotRequired",
    ],
    "method and drop fact presence must be represented with typed enums",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationMethodBodyCheck:",
        "kind %SelfhostMemoTraitOperationMethodBodyCheckKind",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
        "pub struct SelfhostMemoTraitOperationDropCheck:",
        "kind %SelfhostMemoTraitOperationDropCheckKind",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
    ],
    "method and drop checks must carry typed effect and escape facts",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationPurityGateErrorKind:",
        "MethodBodyEvidenceRequired",
        "UnexpectedMethodBodyEvidence",
        "DropEvidenceRequired",
        "UnexpectedDropEvidence",
    ],
    "purity gate errors must distinguish method/drop required and unexpected evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_purity_gate_effect_kind"),
    [
        "SelfhostEffectKind::Pure:",
        "SelfhostMemoTraitOperationEffectPurityKind::Pure",
        "SelfhostEffectKind::InternalAlloc:",
        "SelfhostEffectEscapeState::NoEscapeProven:",
        "SelfhostMemoTraitOperationEffectPurityKind::Pure",
        "SelfhostEffectEscapeState::MayEscape:",
        "SelfhostMemoTraitOperationEffectPurityKind::Impure",
        "SelfhostEffectEscapeState::NotApplicable:",
        "SelfhostMemoTraitOperationEffectPurityKind::Unknown",
        "SelfhostEffectKind::PrivateState:",
        "SelfhostEffectEscapeState::NoEscapeProven:",
        "SelfhostMemoTraitOperationEffectPurityKind::Pure",
        "SelfhostEffectEscapeState::MayEscape:",
        "SelfhostMemoTraitOperationEffectPurityKind::Impure",
        "SelfhostEffectEscapeState::NotApplicable:",
        "SelfhostMemoTraitOperationEffectPurityKind::Unknown",
        "SelfhostEffectKind::PrivateCache:",
        "SelfhostEffectEscapeState::NoEscapeProven:",
        "SelfhostMemoTraitOperationEffectPurityKind::Pure",
        "SelfhostEffectEscapeState::MayEscape:",
        "SelfhostMemoTraitOperationEffectPurityKind::Impure",
        "SelfhostEffectEscapeState::NotApplicable:",
        "SelfhostMemoTraitOperationEffectPurityKind::Unknown",
        "SelfhostEffectKind::UnsafeMemory:",
        "SelfhostMemoTraitOperationEffectPurityKind::Impure",
        "SelfhostEffectKind::ExternalIo:",
        "SelfhostMemoTraitOperationEffectPurityKind::Impure",
        "SelfhostEffectKind::Nondet:",
        "SelfhostMemoTraitOperationEffectPurityKind::Impure",
    ],
    "effect mapping must only mask internal/private effects with NoEscapeProven and must reject observable effects as impure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_purity_gate_method_body_evidence_result"),
    [
        "SelfhostMemoTraitOperationEvidenceKind::Copy:",
        "selfhost_memo_trait_operation_purity_gate_method_not_required_result check",
        "SelfhostMemoTraitOperationEvidenceKind::Drop:",
        "selfhost_memo_trait_operation_purity_gate_method_not_required_result check",
        "SelfhostMemoTraitOperationEvidenceKind::Eq:",
        "selfhost_memo_trait_operation_purity_gate_method_required_result check",
        "SelfhostMemoTraitOperationEvidenceKind::Hash:",
        "selfhost_memo_trait_operation_purity_gate_method_required_result check",
    ],
    "method body evidence must be required only for Eq and Hash",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_purity_gate_drop_evidence_result"),
    [
        "SelfhostMemoTraitOperationEvidenceKind::Copy:",
        "selfhost_memo_trait_operation_purity_gate_drop_not_required_result check",
        "SelfhostMemoTraitOperationEvidenceKind::Drop:",
        "selfhost_memo_trait_operation_purity_gate_drop_required_result check",
        "SelfhostMemoTraitOperationEvidenceKind::Eq:",
        "selfhost_memo_trait_operation_purity_gate_drop_not_required_result check",
        "SelfhostMemoTraitOperationEvidenceKind::Hash:",
        "selfhost_memo_trait_operation_purity_gate_drop_not_required_result check",
    ],
    "drop evidence must be required only for Drop",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_purity_gate_method_required_result"),
    [
        "SelfhostMemoTraitOperationMethodBodyCheckKind::Present:",
        "selfhost_memo_trait_operation_purity_gate_effect_kind check.effect check.escape",
        "SelfhostMemoTraitOperationMethodBodyCheckKind::Missing:",
        "SelfhostMemoTraitOperationMethodBodyEvidence::Missing",
        "SelfhostMemoTraitOperationMethodBodyCheckKind::Unknown:",
        "SelfhostMemoTraitOperationMethodBodyEvidence::Unknown",
        "SelfhostMemoTraitOperationMethodBodyCheckKind::NotRequired:",
        "MethodBodyEvidenceRequired",
    ],
    "method required path must preserve missing and unknown status and reject not-required as structural error",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_purity_gate_drop_required_result"),
    [
        "SelfhostMemoTraitOperationDropCheckKind::DropImplAbsent:",
        "SelfhostMemoTraitOperationDropEvidence::NoDropRequired",
        "SelfhostMemoTraitOperationDropCheckKind::DropImplPresent:",
        "selfhost_memo_trait_operation_purity_gate_effect_kind check.effect check.escape",
        "SelfhostMemoTraitOperationDropCheckKind::Missing:",
        "SelfhostMemoTraitOperationDropEvidence::Missing",
        "SelfhostMemoTraitOperationDropCheckKind::Unknown:",
        "SelfhostMemoTraitOperationDropEvidence::Unknown",
        "SelfhostMemoTraitOperationDropCheckKind::NotRequired:",
        "DropEvidenceRequired",
    ],
    "drop required path must distinguish no-drop proof, pure drop, missing, unknown, and required evidence",
);
assert.ok(
    source.includes("wildcard arm は使いません。error variant が増えた場合はこの equality 境界を明示的に更新します"),
    "error equality docs must forbid wildcard arms",
);
assert.doesNotMatch(
    code,
    /source_text|source_span|\bspan\b|lexeme|display_name|diagnostic|module_path|file_path|path_suffix|payload_hash|signature_hash|body_hash/,
    "purity gate code must not use source text, spans, lexemes, display names, diagnostics, module paths, or hashes as evidence authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "purity gate policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation purity gate contract passed");
