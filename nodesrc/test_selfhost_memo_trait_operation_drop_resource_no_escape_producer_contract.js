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

function assertEveryTopLevelDeclarationHasDoc(src) {
    const lines = src.split("\n");
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+\S+/;
    for (let i = 0; i < lines.length; i += 1) {
        if (!topLevel.test(lines[i])) {
            continue;
        }
        let cursor = i - 1;
        while (cursor >= 0 && lines[cursor].trim() === "") {
            cursor -= 1;
        }
        assert.ok(
            cursor >= 0 && lines[cursor].trimStart().startsWith("//:"),
            `top-level declaration must have an immediately preceding doc comment at line ${i + 1}: ${lines[i]}`,
        );
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_drop_resource_no_escape_producer.nepl";
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
        "# check/module/memo_trait_operation_drop_resource_no_escape_producer",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "Resource no-escape producer must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("HIR effect summary だけから proof を作らず") &&
        source.includes("Resource IR 側が明示的に渡した") &&
        source.includes("typed observation table"),
    "docs must keep proof production tied to Resource IR typed observations, not HIR effect summaries alone",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、public surface hash、payload hash") &&
        source.includes("no-escape を推測しません"),
    "docs must reject source/display/hash authority",
);
assert.ok(
    source.includes("`effect` は `InternalAlloc`、`escape` は `NotApplicable` だけを受理") &&
        source.includes("`Missing` / `Unknown` を pure に mask しません"),
    "docs must fail closed outside InternalAlloc + NotApplicable and must not mask Missing/Unknown",
);
assert.ok(
    source.includes("行数や doc comment の長さによる制限は置きません"),
    "docs must explicitly allow detailed comments without numeric volume gates",
);
for (const [docSnippet, declarationSnippet] of [
    [
        "SelfhostMemoTraitOperationDropResourceNoEscapeRecord",
        "pub struct SelfhostMemoTraitOperationDropResourceNoEscapeRecord:",
    ],
    [
        "SelfhostMemoTraitOperationDropResourceNoEscapeTable",
        "pub struct SelfhostMemoTraitOperationDropResourceNoEscapeTable:",
    ],
    [
        "SelfhostMemoTraitOperationDropResourceNoEscapeProducerErrorKind",
        "pub enum SelfhostMemoTraitOperationDropResourceNoEscapeProducerErrorKind:",
    ],
    [
        "selfhost_memo_trait_operation_drop_resource_no_escape_producer_table_result",
        "pub fn selfhost_memo_trait_operation_drop_resource_no_escape_producer_table_result",
    ],
]) {
    assertDocBeforeTopLevel(source, docSnippet, declarationSnippet);
}
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_drop_resource_no_escape_producer/,
    "Resource no-escape producer must remain facade-private until full Resource proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_drop_resource_no_escape_producer/,
    "checker-layer Resource no-escape producer must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_drop_resource_no_escape_producer_contract.js"),
    "source policy runner must execute the Resource no-escape producer contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"./memo_trait_operation_drop_no_escape_gate\" as *",
    ],
    "producer must depend only on typed HIR id, effect/type ids, and the Drop no-escape proof table boundary",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:backend|resource\/|proof\/(?:api|solver|fact|query)\/resource|resource_tree|resource_graph|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|private_cache|private_state)/,
    "producer must not import backend, Resource graph/proof internals, proof store/artifact, canonical-key, public-surface, evidence producer, impl table, scanner/materializer, purity gate, PrivateCache, or PrivateState layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationDropResourceNoEscapeRecord:",
        "type_id %SelfhostTypeId",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
        "status %SelfhostMemoTraitOperationDropResourceNoEscapeStatus",
    ],
    "Resource observation record must carry typed body identity, effect, escape, and status",
);
assert.doesNotMatch(
    topLevelBlock(source, "struct", "SelfhostMemoTraitOperationDropResourceNoEscapeRecord"),
    /payload_hash|signature_hash|body_hash|public_surface|source_text|source_span|source_path|\bspan\b|\bpath\b|\bname\b|diagnostic|message|text/i,
    "Resource observation record must not use source/display/hash authority instead of typed body identity",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_resource_no_escape_record_key_eq"),
    [
        "selfhost_type_id_eq a.type_id b.type_id",
        "eq a.body_module_fingerprint b.body_module_fingerprint",
        "eq selfhost_hir_expr_id_index a.body_root selfhost_hir_expr_id_index b.body_root",
        "selfhost_effect_kind_eq a.effect b.effect",
        "selfhost_memo_trait_operation_drop_resource_no_escape_escape_state_eq a.escape b.escape",
    ],
    "record key equality must compare type, module origin, body root, effect, and escape",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_resource_no_escape_validate_record_result"),
    [
        "eq record.body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "selfhost_effect_kind_eq record.effect SelfhostEffectKind::InternalAlloc",
        "SelfhostEffectEscapeState::NotApplicable:",
        "Result::Ok unit",
        "RecordEscapeNotApplicable record.escape",
        "RecordEffectNotInternalAlloc record.effect",
    ],
    "validation must accept only non-placeholder InternalAlloc + NotApplicable observations",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_resource_no_escape_table_push"),
    [
        "selfhost_memo_trait_operation_drop_resource_no_escape_validate_record_result record",
        "selfhost_memo_trait_operation_drop_resource_no_escape_table_contains_key &table record",
        "selfhost_memo_trait_operation_drop_resource_no_escape_table_free table",
        "SelfhostMemoTraitOperationDropResourceNoEscapeProducerErrorKind::ResourceRecordDuplicate",
        "Result::Err e:",
        "let error %StdErrorKind field::get e \"error\"",
        "v::free v::vec_push_error_vec e",
    ],
    "input table push must validate, reject duplicate records, free table on structural rejection, and recover Vec owners on push failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_resource_no_escape_status_to_proof_status"),
    [
        "SelfhostMemoTraitOperationDropResourceNoEscapeStatus::NoEscapeProven:",
        "SelfhostMemoTraitOperationDropNoEscapeProofStatus::Proven",
        "SelfhostMemoTraitOperationDropResourceNoEscapeStatus::MayEscape:",
        "SelfhostMemoTraitOperationDropNoEscapeProofStatus::Refuted",
        "SelfhostMemoTraitOperationDropResourceNoEscapeStatus::Missing:",
        "SelfhostMemoTraitOperationDropNoEscapeProofStatus::Missing",
        "SelfhostMemoTraitOperationDropResourceNoEscapeStatus::Unknown:",
        "SelfhostMemoTraitOperationDropNoEscapeProofStatus::Unknown",
    ],
    "Resource statuses must map to proof statuses without masking Missing/Unknown",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_resource_no_escape_produce_loop"),
    [
        "selfhost_memo_trait_operation_drop_resource_no_escape_validate_record_result record",
        "selfhost_memo_trait_operation_drop_resource_no_escape_duplicate_before_result source record idx",
        "SelfhostMemoTraitOperationDropResourceNoEscapeProducerErrorKind::ResourceRecordDuplicate",
        "selfhost_memo_trait_operation_drop_resource_no_escape_output_push_result output record",
    ],
    "producer loop must revalidate direct table contents, reject malformed duplicates, and only then push proof records",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_resource_no_escape_output_push_result"),
    [
        "selfhost_memo_trait_operation_drop_resource_no_escape_record_to_proof_record record",
        "selfhost_memo_trait_operation_drop_no_escape_proof_table_push output proof_record",
        "SelfhostMemoTraitOperationDropResourceNoEscapeProducerErrorKind::OutputProofTableRejected gate_error",
    ],
    "output push must use the existing no-escape proof table boundary and preserve nested typed gate errors",
);
assert.doesNotMatch(
    code,
    /\b(SelfhostMemoTraitOperationDropEvidence|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|SelfhostMemoTraitProofArtifact|SelfhostPrivateCache|SelfhostPrivateState|PrivateCacheMask|PrivateStateMask|NoDropRequired|PureDrop|prechecked|Prechecked)\b/,
    "producer must not synthesize Drop evidence, aggregate proof, proof-store/artifact values, prechecked artifacts, or PrivateCache/PrivateState masking",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "producer APIs must return typed Result errors instead of bool/string errors",
);
