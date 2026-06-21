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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_resource_no_escape_traversal_collector.nepl";
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
        "# check/module/memo_trait_operation_private_effect_resource_no_escape_traversal_collector",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "private-effect Resource traversal collector must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("accepted authority は `SelfhostPrivateEffectResourceGraphBodyRecord`") &&
        source.includes("source text、span、lexeme、display name、module path、public surface hash、payload hash") &&
        source.includes("no-escape を推測しません"),
    "collector docs must reject source/display/hash authority",
);
assert.ok(
    source.includes("materializer output key は graph id を含まない") &&
        source.includes("graph id だけ異なる同一 materializer key は fail-closed に拒否"),
    "collector docs must separate graph-local input keys from materializer output keys",
);
assert.ok(
    source.includes("`ClosedForPrivateEffectBody` かつ `FreshPrivateRegionWitnessed`") &&
        source.includes("`FreshRegionMissing` は `ResourceGraphMissing`") &&
        source.includes("`FreshRegionUnsupported` は `TraversalUnsupported`"),
    "collector docs must require fresh private region evidence and fail closed for missing/unsupported freshness",
);
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_private_effect_resource_no_escape_traversal_collector/,
    "private-effect Resource traversal collector must remain facade-private until full orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_private_effect_resource_no_escape_traversal_collector/,
    "checker-layer private-effect Resource traversal collector must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_collector_contract.js"),
    "source policy runner must execute the private-effect Resource traversal collector contract",
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
        "#import \"./memo_trait_operation_private_effect_resource_no_escape_materializer\" as *",
    ],
    "collector import allowlist must stay at typed core helpers, operation kind, and materializer boundary",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:backend|memo_call|resource\/|proof\/(?:api|solver|fact|query)\/resource|resource_tree|resource_graph|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|memo_trait_operation_drop_resource|private_cache|private_state)/,
    "collector must not import backend, memo_call, Resource graph/proof internals, proof artifacts, public surface, impl scanners, purity gate, Drop Resource, PrivateCache, or PrivateState layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostPrivateEffectResourceGraphBodyRecord:",
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
    "body record must carry typed operation/body identity, graph id, completeness, and fresh-region evidence",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostPrivateEffectResourceGraphPlaceRecord:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "graph_id %SelfhostPrivateEffectResourceGraphId",
        "place_id %SelfhostPrivateEffectResourcePlaceId",
        "kind %SelfhostPrivateEffectResourcePlaceKind",
    ],
    "place record must carry typed operation/body identity, graph id, place id, and typed place kind",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostPrivateEffectResourceGraphEdgeRecord:",
        "type_id %SelfhostTypeId",
        "operation %SelfhostMemoTraitOperationEvidenceKind",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "graph_id %SelfhostPrivateEffectResourceGraphId",
        "from_place %SelfhostPrivateEffectResourcePlaceId",
        "to_place %SelfhostPrivateEffectResourcePlaceId",
        "kind %SelfhostPrivateEffectResourceEdgeKind",
    ],
    "edge record must carry typed operation/body identity, graph id, endpoints, and typed edge kind",
);
for (const [kind, name] of [
    ["struct", "SelfhostPrivateEffectResourceGraphBodyRecord"],
    ["struct", "SelfhostPrivateEffectResourceGraphPlaceRecord"],
    ["struct", "SelfhostPrivateEffectResourceGraphEdgeRecord"],
]) {
    assert.doesNotMatch(
        topLevelBlock(source, kind, name),
        /payload_hash|signature_hash|body_hash|public_surface|source_text|source_span|source_path|\bspan\b|\bpath\b|\bname\b|diagnostic|message|text/i,
        `${name} must not use source/display/hash authority instead of typed graph identity`,
    );
}
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_effect_is_private"),
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
    "collector must accept only PrivateState and PrivateCache effects",
);
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_validate_body_result"),
    [
        "eq body.body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "lt body.graph_id.index 0",
        "GraphIdInvalid body.graph_id.index",
        "selfhost_private_effect_resource_graph_effect_is_private body.effect",
        "SelfhostEffectEscapeState::NotApplicable:",
        "Result::Ok unit",
        "EscapeNotApplicable body.escape",
        "EffectNotPrivate body.effect",
    ],
    "body validation must accept only non-placeholder private-effect + NotApplicable graph headers",
);
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_body_key_eq"),
    [
        "selfhost_type_id_eq a.type_id b.type_id",
        "selfhost_memo_trait_operation_evidence_kind_eq a.operation b.operation",
        "eq a.body_module_fingerprint b.body_module_fingerprint",
        "eq selfhost_hir_expr_id_index a.body_root selfhost_hir_expr_id_index b.body_root",
        "selfhost_effect_kind_eq a.effect b.effect",
        "selfhost_private_effect_resource_graph_escape_state_eq a.escape b.escape",
        "selfhost_private_effect_resource_graph_id_eq a.graph_id b.graph_id",
    ],
    "collector graph key must include operation and graph id",
);
const materializerKeyBlock = functionBlock(source, "selfhost_private_effect_resource_graph_body_materializer_key_eq");
assertOrdered(
    materializerKeyBlock,
    [
        "selfhost_type_id_eq a.type_id b.type_id",
        "selfhost_memo_trait_operation_evidence_kind_eq a.operation b.operation",
        "eq a.body_module_fingerprint b.body_module_fingerprint",
        "eq selfhost_hir_expr_id_index a.body_root selfhost_hir_expr_id_index b.body_root",
        "selfhost_effect_kind_eq a.effect b.effect",
        "selfhost_private_effect_resource_graph_escape_state_eq a.escape b.escape",
    ],
    "materializer key must include operation/body/effect/escape identity",
);
assert.doesNotMatch(materializerKeyBlock, /graph_id/, "materializer key must not include graph id");
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_validate_all_bodies_loop"),
    [
        "selfhost_private_effect_resource_graph_seen_body_before_result input body idx",
        "GraphBodyDuplicate",
        "selfhost_private_effect_resource_graph_seen_materializer_key_before_result input body idx",
        "GraphMaterializerKeyDuplicate",
    ],
    "body preflight must reject exact graph duplicates and graph-id-only materializer key collisions",
);
assertOrdered(
    functionBlock(source, "selfhost_private_effect_resource_graph_body_summary_result"),
    [
        "SelfhostPrivateEffectResourceGraphCompleteness::ResourceGraphMissing:",
        "ResourceGraphMissing",
        "SelfhostPrivateEffectResourceGraphCompleteness::TraversalUnsupported:",
        "TraversalUnsupported",
        "SelfhostPrivateEffectResourceGraphCompleteness::ClosedForPrivateEffectBody:",
        "SelfhostPrivateEffectResourceFreshRegionEvidence::FreshPrivateRegionWitnessed:",
        "selfhost_private_effect_resource_graph_body_has_place input body",
        "AllTraversedPlacesPrivate",
        "selfhost_private_effect_resource_graph_fold_places_loop",
        "selfhost_private_effect_resource_graph_fold_edges_loop",
        "SelfhostPrivateEffectResourceFreshRegionEvidence::FreshRegionMissing:",
        "ResourceGraphMissing",
        "SelfhostPrivateEffectResourceFreshRegionEvidence::FreshRegionUnsupported:",
        "TraversalUnsupported",
    ],
    "only closed graph with fresh private region evidence may prove all traversed places private",
);
const outputPush = functionBlock(source, "selfhost_private_effect_resource_graph_output_push_result");
assertOrdered(
    outputPush,
    [
        "selfhost_memo_trait_operation_private_effect_resource_no_escape_traversal_record_new",
        "body.type_id",
        "body.operation",
        "body.body_module_fingerprint",
        "body.body_root",
        "body.effect",
        "body.escape",
        "summary.status",
        "summary.reason",
    ],
    "output traversal record must carry operation/body/effect/escape/status/reason to the materializer",
);
assert.doesNotMatch(outputPush, /body\.graph_id/, "output traversal record must not carry graph id into the materializer key");
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostPrivateEffectResourceGraphTraversalCollectorStage0Summary"),
    [
        "fresh_missing_status %Result SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalStatus",
        "duplicate_rejected %Result i32 SelfhostPrivateEffectResourceGraphTraversalCollectorErrorKind",
        "materializer_key_duplicate_rejected %Result i32 SelfhostPrivateEffectResourceGraphTraversalCollectorErrorKind",
        "endpoint_missing_rejected %Result i32 SelfhostPrivateEffectResourceGraphTraversalCollectorErrorKind",
    ],
    "stage0 smoke summary must expose freshness, duplicate, materializer-key collision, and endpoint-missing checks",
);
assert.ok(
    source.includes("summary.fresh_missing_status SelfhostMemoTraitOperationPrivateEffectResourceNoEscapeTraversalStatus::ResourceGraphMissing") &&
        source.includes("summary.materializer_key_duplicate_rejected SelfhostPrivateEffectResourceGraphTraversalCollectorErrorKind::GraphMaterializerKeyDuplicate"),
    "doctest must assert fresh missing and graph-id-only materializer key collision fail closed",
);
assert.doesNotMatch(
    code,
    /\bResult\s+[^ \n]+\s+(?:str|String)\b|\bResult::Err\s+"|diagnostic text|message text/i,
    "collector must use typed enum errors instead of string diagnostics as control flow",
);
console.log("[ok] selfhost private-effect Resource no-escape traversal collector contract");
