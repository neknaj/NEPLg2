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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_resource_no_escape_materializer.nepl";
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
        "# check/module/memo_trait_operation_private_effect_resource_no_escape_materializer",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "private-effect Resource no-escape materializer must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("actual Resource traversal") &&
        source.includes("typed summary table") &&
        source.includes("producer が読む `SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTable`"),
    "docs must keep the materializer between actual Resource traversal summary and the existing private-effect observation table",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、public surface hash、payload hash") &&
        source.includes("no-escape を推測しません"),
    "docs must reject source/display/hash authority",
);
assert.ok(
    source.includes("`effect` は `PrivateState` または `PrivateCache`、`escape` は `NotApplicable` だけを受理") &&
        source.includes("`Missing` / `Unknown` を pure に mask しません"),
    "docs must fail closed outside private effect + NotApplicable and must not mask Missing/Unknown",
);
assert.ok(
    source.includes("必ず既存 producer を呼びます") &&
        source.includes("proof key をこの module で直接合成しません"),
    "docs must require proof table generation to go through the existing producer",
);
assert.ok(
    source.includes("行数や doc comment の長さによる制限は置きません"),
    "docs must explicitly allow detailed comments without numeric volume gates",
);
for (const [docSnippet, declarationSnippet] of [
    [
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalRecord",
        "pub struct SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalRecord:",
    ],
    [
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalTable",
        "pub struct SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalTable:",
    ],
    [
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeMaterializerErrorKind",
        "pub enum SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeMaterializerErrorKind:",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_table_result",
        "pub fn selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_table_result",
    ],
    [
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_proof_table_result",
        "pub fn selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_proof_table_result",
    ],
]) {
    assertDocBeforeTopLevel(source, docSnippet, declarationSnippet);
}
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_private_effect_resource_no_escape_materializer/,
    "private-effect Resource no-escape materializer must remain facade-private until full Resource proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_private_effect_resource_no_escape_materializer/,
    "checker-layer private-effect Resource no-escape materializer must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_contract.js"),
    "source policy runner must execute the private-effect Resource no-escape materializer contract",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_private_effect_no_escape_gate\" as *",
        "#import \"./memo_trait_operation_private_effect_resource_no_escape_producer\" as *",
    ],
    "materializer must depend only on typed HIR id, effect/type ids, operation kind, private-effect proof table type, and existing Resource observation producer boundary",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:backend|memo_call|resource\/|proof\/(?:api|solver|fact|query)\/resource|resource_tree|resource_graph|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|memo_trait_operation_drop_no_escape_gate|memo_trait_operation_drop_resource|private_cache|private_state)/,
    "materializer must not import backend, memo_call, Resource graph/proof internals, proof store/artifact, canonical-key, public-surface, evidence producer, impl table, scanner/materializer, purity gate, Drop proof layers, PrivateCache, or PrivateState layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalRecord:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
        "status %SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalStatus",
        "reason %SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalReason",
    ],
    "traversal summary record must carry typed operation/body identity, effect, escape, status, and reason",
);
assert.doesNotMatch(
    topLevelBlock(source, "struct", "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalRecord"),
    /payload_hash|signature_hash|body_hash|public_surface|source_text|source_span|source_path|\bspan\b|\bpath\b|\bname\b|diagnostic|message|text/i,
    "traversal summary record must not use source/display/hash authority instead of typed operation/body identity",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_record_key_eq"),
    [
        "selfhost_type_id_eq a.type_id b.type_id",
        "selfhost_memo_trait_operation_evidence_kind_eq a.operation b.operation",
        "eq a.body_module_fingerprint b.body_module_fingerprint",
        "eq selfhost_hir_expr_id_index a.body_root selfhost_hir_expr_id_index b.body_root",
        "selfhost_effect_kind_eq a.effect b.effect",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_escape_state_eq a.escape b.escape",
    ],
    "traversal record key equality must compare type, operation, module origin, body root, effect, and escape",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_validate_effect"),
    [
        "SelfhostEffectKind::Pure:",
        "TraversalEffectNotPrivate effect",
        "SelfhostEffectKind::InternalAlloc:",
        "TraversalEffectNotPrivate effect",
        "SelfhostEffectKind::PrivateState:",
        "Result::Ok unit",
        "SelfhostEffectKind::PrivateCache:",
        "Result::Ok unit",
        "SelfhostEffectKind::Nondet:",
        "TraversalEffectNotPrivate effect",
    ],
    "effect validation must accept only PrivateState and PrivateCache and must enumerate every effect variant",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_validate_record_result"),
    [
        "eq record.body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_validate_effect record.effect",
        "SelfhostEffectEscapeState::NotApplicable:",
        "Result::Ok unit",
        "TraversalEscapeNotApplicable record.escape",
    ],
    "validation must accept only non-placeholder private effect + NotApplicable traversal summaries",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_table_push"),
    [
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_validate_record_result record",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_table_contains_key &table record",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_table_free table",
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeMaterializerErrorKind::TraversalRecordDuplicate",
        "Result::Err e:",
        "let error %StdErrorKind field::get e \"error\"",
        "v::free v::vec_push_error_vec e",
    ],
    "input table push must validate, reject duplicate records, free table on structural rejection, and recover Vec owners on push failure",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_status_to_observation_status"),
    [
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalStatus::AllTraversedPlacesPrivate:",
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeStatus::NoEscapeProven",
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalStatus::EscapingPlaceObserved:",
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeStatus::MayEscape",
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalStatus::ResourceGraphMissing:",
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeStatus::Missing",
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalStatus::TraversalUnsupported:",
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeStatus::Unknown",
    ],
    "traversal statuses must map to observation statuses without masking Missing/Unknown",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_record_to_observation_record"),
    [
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_status_to_observation_status record.status",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_record_new record.type_id record.operation record.body_module_fingerprint record.body_root record.effect record.escape status",
    ],
    "materializer must preserve typed operation/body identity, effect, and escape when building observation records",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_resource_no_escape_materialize_loop"),
    [
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_validate_record_result record",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_duplicate_before_result source record idx",
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeMaterializerErrorKind::TraversalRecordDuplicate",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_output_push_result output record",
    ],
    "materializer loop must revalidate direct table contents, reject malformed duplicates, and only then push observation records",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_table_result"),
    /selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_table_free/,
    "materializer_table_result must borrow the source table and must not free it",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_proof_table_result"),
    [
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_table_result source",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_producer_table_result &observations",
        "SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeMaterializerErrorKind::ProofTableRejected producer_error",
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_table_free observations",
    ],
    "proof table helper must materialize observations, delegate proof construction to the existing producer, preserve nested typed errors, and free observations",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_operation_private_effect_resource_no_escape_materializer_proof_table_result"),
    /\b(selfhost_memo_trait_operation_private_effect_no_escape_proof_key_new|selfhost_memo_trait_operation_private_effect_no_escape_proof_record_new|selfhost_memo_trait_operation_private_effect_no_escape_proof_table_new|selfhost_memo_trait_operation_private_effect_no_escape_proof_table_push)\b/,
    "proof table helper must not synthesize proof keys, proof records, or proof tables directly",
);
assert.doesNotMatch(
    code,
    /\b(SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|SelfhostMemoTraitProofArtifact|RequestEvidenceProven|SelfhostPrivateCache|SelfhostPrivateState|PrivateCacheMask|PrivateStateMask|prechecked|Prechecked|neplobj|neplproof|Wasm|LLVM)\b/,
    "materializer must not synthesize operation evidence, aggregate proof, request evidence, proof-store/artifact values, prechecked artifacts, backend bytes, or PrivateCache/PrivateState masking",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "materializer APIs must return typed Result errors instead of bool/string errors",
);
console.log("selfhost private-effect Resource no-escape materializer contract passed");
