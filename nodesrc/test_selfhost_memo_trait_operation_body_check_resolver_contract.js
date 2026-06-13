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

function walkFiles(root, predicate, out = []) {
    for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
        const fullPath = path.join(root, entry.name);
        if (entry.isDirectory()) {
            walkFiles(fullPath, predicate, out);
        } else if (predicate(fullPath)) {
            out.push(fullPath);
        }
    }
    return out;
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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_body_check_resolver.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_body_check_resolver",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "body check resolver must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("`Copy` は method body も Drop impl proof も必要としません") &&
        source.includes("`Eq` / `Hash` は method body purity check を必要とし") &&
        source.includes("`Drop` は Drop impl proof を必要とし"),
    "docs must fix the Copy/Drop/Eq/Hash body proof requirement matrix",
);
assert.ok(
    source.includes("method_duplicate_rejected") &&
        source.includes("drop_duplicate_rejected") &&
        source.includes("MethodBodyResolverRejected SelfhostMemoTraitOperationMethodBodyResolverErrorKind::RecordDuplicate") &&
        source.includes("DropImplResolverRejected SelfhostMemoTraitOperationDropImplResolverErrorKind::RecordDuplicate"),
    "doctest must cover both method and Drop duplicate wrapper errors",
);
assert.ok(
    source.includes("この module は `SelfhostMemoTraitOperationBodyChecks` だけを返します") &&
        source.includes("operation evidence record、producer input、aggregate proof status は作りません"),
    "body check resolver must stay before evidence producer and aggregate proof construction",
);
assert.ok(
    source.includes("この module が直接作る check は operation 上不要な `NotRequired` だけです") &&
        source.includes("`Missing` / `Unknown` は status check であり、resolver error ではありません"),
    "docs must separate status checks from structural resolver errors",
);
assert.ok(
    source.includes("method_unknown_checks") &&
        source.includes("drop_unknown_checks") &&
        source.includes("SelfhostMemoTraitOperationMethodBodyCheckKind::Unknown") &&
        source.includes("SelfhostMemoTraitOperationDropCheckKind::Unknown"),
    "doctest must exercise method and Drop Unknown status pass-through",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、HIR、Resource IR、backend artifact、proof store record、public surface hash を authority にしません"),
    "docs must exclude source/display/diagnostic/module path/HIR/Resource/backend/proof-store/public-surface authority",
);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_body_check_resolver/,
    "body check resolver must remain facade-private until full orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_body_check_resolver/,
    "checker-layer body check resolver must not be registered in the ty source list",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_purity_gate\" as *",
        "#import \"./memo_trait_operation_method_body_resolver\" as *",
        "#import \"./memo_trait_operation_drop_impl_resolver\" as *",
    ],
    "body check resolver must depend on typed effect, TypeId, operation kind, purity gate check types, and the two resolver boundaries",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table)/,
    "body check resolver must not import HIR, Resource IR, backend, proof store, artifact, canonical-key, public-surface, public-impl-header, evidence-producer, or operation impl table layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationBodyChecks:",
        "method_body %SelfhostMemoTraitOperationMethodBodyCheck",
        "drop_impl %SelfhostMemoTraitOperationDropCheck",
        "pub enum SelfhostMemoTraitOperationBodyCheckResolverErrorKind:",
        "MethodBodyResolverRejected %SelfhostMemoTraitOperationMethodBodyResolverErrorKind",
        "DropImplResolverRejected %SelfhostMemoTraitOperationDropImplResolverErrorKind",
    ],
    "body check resolver must expose typed check pair and payload-carrying wrapper errors",
);
assert.match(
    source,
    /(?:^|\n)fn selfhost_memo_trait_operation_body_checks_new\s+/,
    "generic body check pair constructor must remain private so callers cannot bypass the operation matrix",
);
assert.doesNotMatch(
    source,
    /(?:^|\n)pub fn selfhost_memo_trait_operation_body_checks_new\s+/,
    "generic body check pair constructor must not be public",
);
for (const filePath of walkFiles(path.join(repoRoot, "stdlib", "neplg2"), (candidate) => candidate.endsWith(".nepl"))) {
    const rel = path.relative(repoRoot, filePath).replace(/\\/g, "/");
    if (rel === relPath) {
        continue;
    }
    const fileSource = stripDocComments(fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n"));
    assert.doesNotMatch(
        fileSource,
        /\bSelfhostMemoTraitOperationBodyChecks\s+[^:\r\n]/,
        `external code must not construct SelfhostMemoTraitOperationBodyChecks directly: ${rel}`,
    );
}
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationBodyCheckResolverErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "body check resolver errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "body check resolver APIs must return typed Result errors instead of bool/string errors",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_method_not_required"),
    [
        "SelfhostMemoTraitOperationMethodBodyCheckKind::NotRequired",
        "SelfhostEffectKind::Pure",
        "SelfhostEffectEscapeState::NotApplicable",
    ],
    "direct method check construction must be limited to NotRequired",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_drop_not_required"),
    [
        "SelfhostMemoTraitOperationDropCheckKind::NotRequired",
        "SelfhostEffectKind::Pure",
        "SelfhostEffectEscapeState::NotApplicable",
    ],
    "direct Drop check construction must be limited to NotRequired",
);
const directConstructors = [
    "selfhost_memo_trait_operation_body_check_method_not_required",
    "selfhost_memo_trait_operation_body_check_drop_not_required",
];
for (const name of directConstructors) {
    const block = functionBlock(source, name);
    assert.doesNotMatch(
        block,
        /SelfhostMemoTraitOperationMethodBodyCheckKind::(?:Present|Missing|Unknown)|SelfhostMemoTraitOperationDropCheckKind::(?:DropImplAbsent|DropImplPresent|Missing|Unknown)/,
        `${name} must not directly construct present/missing/unknown/drop-present/drop-absent checks`,
    );
}
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_resolve_result"),
    [
        "SelfhostMemoTraitOperationEvidenceKind::Copy:",
        "selfhost_memo_trait_operation_body_check_resolve_copy",
        "SelfhostMemoTraitOperationEvidenceKind::Drop:",
        "selfhost_memo_trait_operation_body_check_resolve_drop_operation drop_surface drop_table type_id",
        "SelfhostMemoTraitOperationEvidenceKind::Eq:",
        "selfhost_memo_trait_operation_body_check_resolve_method_operation method_surface method_table type_id operation",
        "SelfhostMemoTraitOperationEvidenceKind::Hash:",
        "selfhost_memo_trait_operation_body_check_resolve_method_operation method_surface method_table type_id operation",
    ],
    "operation resolver must use an explicit four-arm operation matrix",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_resolve_copy"),
    /method_body_resolve_result|drop_impl_resolve_result|method_table|drop_table/,
    "Copy resolution must not use either method or Drop table as authority",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_resolve_method_operation"),
    /selfhost_memo_trait_operation_method_body_resolve_result method_surface method_table type_id operation/,
    "Eq/Hash resolution must call the method body resolver",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_resolve_method_operation"),
    /drop_impl_resolve_result|drop_table/,
    "Eq/Hash resolution must not call the Drop resolver",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_resolve_drop_operation"),
    /selfhost_memo_trait_operation_drop_impl_resolve_result drop_surface drop_table type_id/,
    "Drop resolution must call the Drop impl resolver",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_resolve_drop_operation"),
    /method_body_resolve_result|method_table/,
    "Drop resolution must not call method body table lookup",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_resolve_method_operation"),
    [
        "Result::Ok method_body:",
        "Result::Ok selfhost_memo_trait_operation_body_checks_new method_body selfhost_memo_trait_operation_body_check_drop_not_required",
        "Result::Err method_error:",
        "Result::Err selfhost_memo_trait_operation_body_check_method_error method_error",
    ],
    "method resolver status values must remain success checks and structural errors must be wrapped",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_resolve_drop_operation"),
    [
        "Result::Ok drop_impl:",
        "Result::Ok selfhost_memo_trait_operation_body_checks_new selfhost_memo_trait_operation_body_check_method_not_required drop_impl",
        "Result::Err drop_error:",
        "Result::Err selfhost_memo_trait_operation_body_check_drop_error drop_error",
    ],
    "Drop resolver status values must remain success checks and structural errors must be wrapped",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_body_check_resolver_error_kind_eq"),
    [
        "MethodBodyResolverRejected a_method:",
        "MethodBodyResolverRejected b_method:",
        "selfhost_memo_trait_operation_method_body_resolver_error_kind_eq a_method b_method",
        "DropImplResolverRejected a_drop:",
        "DropImplResolverRejected b_drop:",
        "selfhost_memo_trait_operation_drop_impl_resolver_error_kind_eq a_drop b_drop",
    ],
    "error equality must be exhaustive and compare nested payloads",
);
assert.ok(
    source.includes("wildcard arm は使いません。error variant が増えた場合はこの equality 境界を明示的に更新します"),
    "error equality docs must explicitly forbid wildcard arms",
);
assert.doesNotMatch(
    code,
    /source_text|source_span|\bspan\b|lexeme|display_name|diagnostic|module_path|file_path|path_suffix|payload_hash|signature_hash|body_hash|public_surface_hash/,
    "body check resolver code must not use source text, spans, lexemes, display names, diagnostics, module paths, or hashes as evidence authority",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "body check resolver policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait operation body check resolver contract passed");
