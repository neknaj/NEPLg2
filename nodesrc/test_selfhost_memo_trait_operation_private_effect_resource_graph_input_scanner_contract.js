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

function assertEveryTopLevelDeclarationHasDoc(src) {
    const lines = src.split("\n");
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+\S+/;
    for (let i = 0; i < lines.length; i += 1) {
        if (!topLevel.test(lines[i])) continue;
        let cursor = i - 1;
        while (cursor >= 0 && lines[cursor].trim() === "") cursor -= 1;
        assert.ok(
            cursor >= 0 && lines[cursor].trimStart().startsWith("//:"),
            `top-level declaration must have an immediately preceding doc comment at line ${i + 1}: ${lines[i]}`,
        );
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_resource_graph_input_scanner.nepl";
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
        "# check/module/memo_trait_operation_private_effect_resource_graph_input_scanner",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "private-effect Resource graph input scanner must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("SelfhostPrivateEffectResourceGraphWalkerBodyRecord") &&
        source.includes("typed payload") &&
        source.includes("source text、span、lexeme、display name、module path、public surface hash、payload hash、diagnostic text"),
    "scanner docs must accept only typed walker payload and reject source/display/hash/diagnostic authority",
);
assert.ok(
    source.includes("fresh private region 証拠は walker body record の `fresh_region` だけを authority") &&
        source.includes("fresh 状態を補完しません"),
    "scanner docs must not infer fresh region evidence from non-typed authority",
);
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_private_effect_resource_graph_input_scanner/,
    "private-effect Resource graph input scanner must remain facade-private until full orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_private_effect_resource_graph_input_scanner/,
    "checker-layer private-effect Resource graph input scanner must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_private_effect_resource_graph_input_scanner_contract.js"),
    "source policy runner must execute the private-effect Resource graph input scanner contract",
);
assertOrdered(
    source,
    [
        "#import \"alloc/collections/vec\" as v",
        "#import \"core/field\" as field",
        "#import \"core/math\" as *",
        "#import \"core/option\" as *",
        "#import \"core/result\" as *",
        "#import \"core/traits/copy\" as *",
        "#import \"neplg2/core/hir/hir\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty/id\" as *",
        "#import \"neplg2/core/ty/ty/memo_trait_operation_evidence\" as *",
        "#import \"./memo_trait_operation_private_effect_resource_no_escape_traversal_collector\" as *",
        "#import \"./memo_trait_operation_private_effect_resource_no_escape_materializer\" as *",
    ],
    "scanner import allowlist must stay at typed core helpers, operation kind, collector, and traversal table types",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:backend|memo_call|resource\/|proof\/(?:api|solver|fact|query)\/resource|resource_tree|resource_graph|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|memo_trait_operation_drop_resource|private_cache|private_state)/,
    "scanner must not import backend, memo_call, Resource graph/proof internals, proof artifacts, public surface, impl scanners, purity gate, Drop Resource, PrivateCache, or PrivateState layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostPrivateEffectResourceGraphWalkerBodyRecord:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
        "graph_id %SelfhostPrivateEffectResourceGraphId",
        "completeness %SelfhostPrivateEffectResourceGraphCompleteness",
        "fresh_region %SelfhostPrivateEffectResourceFreshRegionEvidence",
    ],
    "walker body record must carry typed operation/body identity, graph id, completeness, and fresh-region evidence",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostPrivateEffectResourceGraphWalkerPlaceEventRecord:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "graph_id %SelfhostPrivateEffectResourceGraphId",
        "operation_ordinal %i32",
        "place_id %SelfhostPrivateEffectResourcePlaceId",
        "kind %SelfhostPrivateEffectResourcePlaceKind",
    ],
    "walker place event must carry operation, operation ordinal, place id, and typed place kind",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostPrivateEffectResourceGraphWalkerEdgeEventRecord:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "graph_id %SelfhostPrivateEffectResourceGraphId",
        "operation_ordinal %i32",
        "from_place %SelfhostPrivateEffectResourcePlaceId",
        "to_place %SelfhostPrivateEffectResourcePlaceId",
        "kind %SelfhostPrivateEffectResourceEdgeKind",
    ],
    "walker edge event must carry operation, operation ordinal, endpoints, and typed edge kind",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostPrivateEffectResourceGraphWalkerUnsupportedEventRecord:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "graph_id %SelfhostPrivateEffectResourceGraphId",
        "operation_ordinal %i32",
        "reason %SelfhostPrivateEffectResourceGraphWalkerUnsupportedReason",
    ],
    "unsupported event must carry operation, operation ordinal, and typed unsupported reason",
);
for (const [kind, name] of [
    ["struct", "SelfhostPrivateEffectResourceGraphWalkerBodyRecord"],
    ["struct", "SelfhostPrivateEffectResourceGraphWalkerPlaceEventRecord"],
    ["struct", "SelfhostPrivateEffectResourceGraphWalkerEdgeEventRecord"],
    ["struct", "SelfhostPrivateEffectResourceGraphWalkerUnsupportedEventRecord"],
]) {
    assert.doesNotMatch(
        topLevelBlock(source, kind, name),
        /payload_hash|signature_hash|body_hash|public_surface|source_text|source_span|source_path|\bspan\b|\bpath\b|\bname\b|diagnostic|message|text/i,
        `${name} must not use source/display/hash/diagnostic authority instead of typed walker identity`,
    );
}
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_input_scanner_effect_is_private"),
    [
        "SelfhostEffectKind::Pure:",
        "false",
        "SelfhostEffectKind::InternalAlloc:",
        "false",
        "SelfhostEffectKind::PrivateState:",
        "true",
        "SelfhostEffectKind::PrivateCache:",
        "true",
        "SelfhostEffectKind::UnsafeMemory:",
        "false",
        "SelfhostEffectKind::ExternalIo:",
        "false",
        "SelfhostEffectKind::Nondet:",
        "false",
    ],
    "scanner must accept only PrivateState and PrivateCache effects",
);
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_walker_validate_body_result"),
    [
        "eq body.body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "lt body.graph_id.index 0",
        "GraphIdInvalid body.graph_id.index",
        "selfhost_private_effect_resource_graph_input_scanner_effect_is_private body.effect",
        "SelfhostEffectEscapeState::NotApplicable:",
        "Result::Ok unit",
        "EscapeNotApplicable body.escape",
        "EffectNotPrivate body.effect",
    ],
    "walker body validation must accept only non-placeholder private-effect + NotApplicable graph headers",
);
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_walker_body_key_eq"),
    [
        "selfhost_type_id_eq a.type_id b.type_id",
        "selfhost_memo_trait_operation_evidence_kind_eq a.operation b.operation",
        "eq a.body_module_fingerprint b.body_module_fingerprint",
        "eq selfhost_hir_expr_id_index a.body_root selfhost_hir_expr_id_index b.body_root",
        "selfhost_effect_kind_eq a.effect b.effect",
        "selfhost_private_effect_resource_graph_input_scanner_escape_state_eq a.escape b.escape",
        "selfhost_private_effect_resource_graph_input_scanner_graph_id_eq a.graph_id b.graph_id",
    ],
    "scanner body key must include operation and graph id",
);
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_walker_place_body_key_eq"),
    [
        "selfhost_type_id_eq body.type_id place.type_id",
        "selfhost_memo_trait_operation_evidence_kind_eq body.operation place.operation",
        "eq body.body_module_fingerprint place.body_module_fingerprint",
        "eq selfhost_hir_expr_id_index body.body_root selfhost_hir_expr_id_index place.body_root",
        "selfhost_private_effect_resource_graph_input_scanner_graph_id_eq body.graph_id place.graph_id",
    ],
    "place membership must include operation and graph id",
);
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_walker_body_to_collector_body"),
    [
        "selfhost_private_effect_resource_graph_walker_body_output_completeness input body",
        "selfhost_private_effect_resource_graph_body_record_new body.type_id body.operation body.body_module_fingerprint body.body_root body.effect body.escape body.graph_id completeness body.fresh_region",
    ],
    "scanner must project operation and fresh-region evidence into the collector body record",
);
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_walker_place_to_collector_place"),
    [
        "selfhost_private_effect_resource_graph_place_record_new place.type_id place.operation place.body_module_fingerprint place.body_root place.graph_id place.place_id place.kind",
    ],
    "scanner must project operation into collector place records",
);
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_walker_edge_to_collector_edge"),
    [
        "selfhost_private_effect_resource_graph_edge_record_new edge.type_id edge.operation edge.body_module_fingerprint edge.body_root edge.graph_id edge.from_place edge.to_place edge.kind",
    ],
    "scanner must project operation into collector edge records",
);
const traversalMatch = functionBlock(source, "selfhost_private_effect_resource_graph_input_scanner_traversal_record_matches_body");
assertOrdered(
    traversalMatch,
    [
        "selfhost_type_id_eq record.type_id body.type_id",
        "selfhost_memo_trait_operation_evidence_kind_eq record.operation body.operation",
        "eq record.body_module_fingerprint body.body_module_fingerprint",
        "eq selfhost_hir_expr_id_index record.body_root selfhost_hir_expr_id_index body.body_root",
        "selfhost_effect_kind_eq record.effect body.effect",
        "selfhost_private_effect_resource_graph_input_scanner_escape_state_eq record.escape body.escape",
    ],
    "scanner status lookup must use materializer identity including operation/body/effect/escape",
);
assert.doesNotMatch(traversalMatch, /graph_id/, "scanner traversal status lookup must not require graph id after materialization");
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_walker_body_output_completeness"),
    [
        "SelfhostPrivateEffectResourceGraphCompleteness::ClosedForPrivateEffectBody:",
        "selfhost_private_effect_resource_graph_walker_has_unsupported_event input body",
        "SelfhostPrivateEffectResourceGraphCompleteness::TraversalUnsupported",
        "SelfhostPrivateEffectResourceGraphCompleteness::ClosedForPrivateEffectBody",
    ],
    "unsupported walker events must force TraversalUnsupported collector body completeness",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostPrivateEffectResourceGraphInputScannerStage0Summary"),
    [
        "fresh_missing_status %Result SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalStatus",
        "duplicate_ordinal_rejected %Result i32 SelfhostPrivateEffectResourceGraphInputScannerErrorKind",
        "missing_body_event_rejected %Result i32 SelfhostPrivateEffectResourceGraphInputScannerErrorKind",
        "placeholder_rejected %Result i32 SelfhostPrivateEffectResourceGraphInputScannerErrorKind",
    ],
    "stage0 smoke summary must expose freshness and scanner structural rejection checks",
);
assert.ok(
    source.includes("summary.fresh_missing_status SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalStatus::ResourceGraphMissing"),
    "doctest must assert fresh missing is preserved through scanner and collected as ResourceGraphMissing",
);
assert.doesNotMatch(
    code,
    /\bResult\s+[^ \n]+\s+(?:str|String)\b|\bResult::Err\s+"|diagnostic text|message text/i,
    "scanner must use typed enum errors instead of string diagnostics as control flow",
);
console.log("[ok] selfhost private-effect Resource graph input scanner contract");
