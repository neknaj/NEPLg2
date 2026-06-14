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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_drop_resource_no_escape_traversal_collector.nepl";
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
        "# check/module/memo_trait_operation_drop_resource_no_escape_traversal_collector",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "Resource traversal collector must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("graph walker が作る typed graph input") &&
        source.includes("Drop body ごとの traversal summary") &&
        source.includes("graph walker 本体ではなく"),
    "docs must define this module as a typed collector boundary, not as the full Resource graph walker",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、module path、public surface hash、payload hash") &&
        source.includes("no-escape を推測しません"),
    "docs must reject source/display/hash authority",
);
assert.ok(
    source.includes("`ClosedForDropBody` の body だけが graph を no-escape 判定に使えます") &&
        source.includes("空 graph から `AllTraversedPlacesPrivate` を合成しません"),
    "docs must limit private traversal proof to closed non-empty graph input",
);
assert.ok(
    source.includes("行数や doc comment の長さによる制限は置きません"),
    "docs must explicitly allow detailed comments without numeric volume gates",
);
for (const [docSnippet, declarationSnippet] of [
    [
        "SelfhostDropResourceGraphCompleteness",
        "pub enum SelfhostDropResourceGraphCompleteness:",
    ],
    [
        "SelfhostResourcePlaceKind",
        "pub enum SelfhostResourcePlaceKind:",
    ],
    [
        "SelfhostResourceEdgeKind",
        "pub enum SelfhostResourceEdgeKind:",
    ],
    [
        "SelfhostDropResourceGraphInput",
        "pub struct SelfhostDropResourceGraphInput:",
    ],
    [
        "SelfhostDropResourceGraphTraversalCollectorErrorKind",
        "pub enum SelfhostDropResourceGraphTraversalCollectorErrorKind:",
    ],
    [
        "selfhost_drop_resource_graph_traversal_collector_table_result",
        "pub fn selfhost_drop_resource_graph_traversal_collector_table_result",
    ],
]) {
    assertDocBeforeTopLevel(source, docSnippet, declarationSnippet);
}
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_drop_resource_no_escape_traversal_collector/,
    "Resource traversal collector must remain facade-private until full Resource proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_drop_resource_no_escape_traversal_collector/,
    "checker-layer Resource traversal collector must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_drop_resource_no_escape_traversal_collector_contract.js"),
    "source policy runner must execute the Resource traversal collector contract",
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
        "#import \"./memo_trait_operation_drop_resource_no_escape_materializer\" as *",
    ],
    "collector must depend only on Vec, basic core helpers, typed HIR/effect/type ids, and the materializer boundary",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:backend|resource\/|proof\/(?:api|solver|fact|query)\/resource|resource_tree|resource_graph|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|private_cache|private_state)/,
    "collector must not import backend, Resource graph/proof internals, proof store/artifact, canonical-key, public-surface, evidence producer, impl table, scanner/materializer, purity gate, PrivateCache, or PrivateState layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostDropResourceGraphBodyRecord:",
        "type_id %SelfhostTypeId",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
        "graph_id %SelfhostDropResourceGraphId",
        "completeness %SelfhostDropResourceGraphCompleteness",
    ],
    "body record must carry typed body identity, effect, escape, graph id, and completeness",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostDropResourceGraphPlaceRecord:",
        "type_id %SelfhostTypeId",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "graph_id %SelfhostDropResourceGraphId",
        "place_id %SelfhostResourcePlaceId",
        "kind %SelfhostResourcePlaceKind",
    ],
    "place record must carry typed body identity, graph id, place id, and typed place kind",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostDropResourceGraphEdgeRecord:",
        "type_id %SelfhostTypeId",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "graph_id %SelfhostDropResourceGraphId",
        "from_place %SelfhostResourcePlaceId",
        "to_place %SelfhostResourcePlaceId",
        "kind %SelfhostResourceEdgeKind",
    ],
    "edge record must carry typed body identity, graph id, endpoints, and typed edge kind",
);
for (const [kind, name] of [
    ["struct", "SelfhostDropResourceGraphBodyRecord"],
    ["struct", "SelfhostDropResourceGraphPlaceRecord"],
    ["struct", "SelfhostDropResourceGraphEdgeRecord"],
]) {
    assert.doesNotMatch(
        topLevelBlock(source, kind, name),
        /payload_hash|signature_hash|body_hash|public_surface|source_text|source_span|source_path|\bspan\b|\bpath\b|\bname\b|diagnostic|message|text/i,
        `${name} must not use source/display/hash authority instead of typed graph identity`,
    );
}
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_validate_body_result"),
    [
        "eq body.body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "lt body.graph_id.index 0",
        "GraphIdInvalid body.graph_id.index",
        "selfhost_effect_kind_eq body.effect SelfhostEffectKind::InternalAlloc",
        "SelfhostEffectEscapeState::NotApplicable:",
        "Result::Ok unit",
        "EscapeNotApplicable body.escape",
        "EffectNotInternalAlloc body.effect",
    ],
    "body validation must accept only non-placeholder InternalAlloc + NotApplicable graph headers",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_validate_input_result"),
    [
        "selfhost_drop_resource_graph_validate_all_bodies_loop input 0",
        "selfhost_drop_resource_graph_validate_all_places_loop input 0",
        "selfhost_drop_resource_graph_validate_all_edges_loop input 0",
    ],
    "collector must preflight all bodies, places, and edges before producing summaries",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_validate_all_places_loop"),
    [
        "selfhost_drop_resource_graph_validate_place_result place",
        "selfhost_drop_resource_graph_seen_place_before_result input place idx",
        "SelfhostDropResourceGraphDuplicateDecision::DuplicateFound:",
        "GraphPlaceDuplicate",
        "selfhost_drop_resource_graph_place_has_body input place",
        "GraphBodyMissing place.graph_id.index",
    ],
    "place preflight must reject malformed, duplicate, and orphan places",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_validate_all_edges_loop"),
    [
        "selfhost_drop_resource_graph_validate_edge_result edge",
        "selfhost_drop_resource_graph_seen_edge_before_result input edge idx",
        "SelfhostDropResourceGraphDuplicateDecision::DuplicateFound:",
        "GraphEdgeDuplicate",
        "selfhost_drop_resource_graph_edge_has_body input edge",
        "selfhost_drop_resource_graph_validate_edge_endpoints_result input edge",
        "GraphBodyMissing edge.graph_id.index",
    ],
    "edge preflight must reject malformed, duplicate, orphan, and endpoint-missing edges",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_body_summary_result"),
    [
        "SelfhostDropResourceGraphCompleteness::ResourceGraphMissing:",
        "ResourceGraphMissing",
        "SelfhostDropResourceGraphCompleteness::TraversalUnsupported:",
        "TraversalUnsupported",
        "SelfhostDropResourceGraphCompleteness::ClosedForDropBody:",
        "selfhost_drop_resource_graph_body_has_place input body",
        "AllTraversedPlacesPrivate",
        "selfhost_drop_resource_graph_fold_places_loop",
        "selfhost_drop_resource_graph_fold_edges_loop",
        "ResourceGraphMissing",
    ],
    "only ClosedForDropBody with at least one place may attempt private traversal summary folding",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_place_summary"),
    [
        "SelfhostResourcePlaceKind::ReturnPlace:",
        "EscapingPlaceObserved",
        "SelfhostResourcePlaceKind::PublicStore:",
        "EscapingPlaceObserved",
        "SelfhostResourcePlaceKind::ExternalHandle:",
        "EscapingPlaceObserved",
        "SelfhostResourcePlaceKind::UnsupportedPlace:",
        "TraversalUnsupported",
    ],
    "place summary must fail closed for escape sinks and unsupported place kinds",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_edge_summary"),
    [
        "SelfhostResourceEdgeKind::StoreToPublic:",
        "EscapingPlaceObserved",
        "SelfhostResourceEdgeKind::Return:",
        "EscapingPlaceObserved",
        "SelfhostResourceEdgeKind::CallBoundaryUnsupported:",
        "TraversalUnsupported",
    ],
    "edge summary must fail closed for public/return edges and unsupported call boundaries",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_traversal_collector_table_result"),
    [
        "selfhost_drop_resource_graph_validate_input_result input",
        "selfhost_memo_trait_operation_drop_resource_no_escape_traversal_table_new",
        "selfhost_drop_resource_graph_collect_loop output0 input 0",
    ],
    "collector table result must validate graph input before allocating and collecting output summaries",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostDropResourceGraphTraversalCollectorStage0Summary"),
    [
        "duplicate_rejected %Result i32 SelfhostDropResourceGraphTraversalCollectorErrorKind",
        "placeholder_rejected %Result i32 SelfhostDropResourceGraphTraversalCollectorErrorKind",
        "endpoint_missing_rejected %Result i32 SelfhostDropResourceGraphTraversalCollectorErrorKind",
    ],
    "stage0 smoke summary must expose duplicate, placeholder, and endpoint-missing rejection results",
);
assert.ok(
    source.includes("SelfhostDropResourceGraphTraversalCollectorErrorKind::EdgeEndpointMissing 1"),
    "doctest must assert endpoint-missing rejection with the actual missing endpoint id",
);
assert.doesNotMatch(
    code,
    /\bResult\s+[^ \n]+\s+(?:str|String)\b|\bResult::Err\s+"|diagnostic text|message text/i,
    "collector must use typed enum errors instead of string diagnostics as control flow",
);
assert.doesNotMatch(
    source,
    /(?:line|行数|comment|コメント)[^\n]{0,40}(?:<=|>=|<|>|max|maximum|cap|上限|制限)[^\n]{0,20}\d+|\d+[^\n]{0,20}(?:line|行|comment|コメント)/i,
    "source policy must not introduce numeric line/comment length caps; detailed docs are preferred",
);
console.log("[ok] selfhost Resource no-escape traversal collector contract");
