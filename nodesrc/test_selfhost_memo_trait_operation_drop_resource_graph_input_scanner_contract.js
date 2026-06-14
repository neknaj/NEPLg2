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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_drop_resource_graph_input_scanner.nepl";
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
        "# check/module/memo_trait_operation_drop_resource_graph_input_scanner",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "Resource graph input scanner must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("actual Resource IR graph walker") &&
        source.includes("typed walker event table") &&
        source.includes("graph walker 本体ではなく"),
    "docs must define this module as scanner input projection, not the full Resource graph walker",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、module path、public surface hash、payload hash、diagnostic text") &&
        source.includes("推測しません"),
    "docs must reject source/display/hash/diagnostic authority",
);
assert.ok(
    source.includes("行数や doc comment の長さによる制限は置きません"),
    "docs must explicitly allow detailed comments without numeric volume gates",
);
for (const [docSnippet, declarationSnippet] of [
    [
        "SelfhostDropResourceGraphWalkerUnsupportedReason",
        "pub enum SelfhostDropResourceGraphWalkerUnsupportedReason:",
    ],
    [
        "SelfhostDropResourceGraphWalkerBodyRecord",
        "pub struct SelfhostDropResourceGraphWalkerBodyRecord:",
    ],
    [
        "SelfhostDropResourceGraphWalkerPlaceEventRecord",
        "pub struct SelfhostDropResourceGraphWalkerPlaceEventRecord:",
    ],
    [
        "SelfhostDropResourceGraphWalkerEdgeEventRecord",
        "pub struct SelfhostDropResourceGraphWalkerEdgeEventRecord:",
    ],
    [
        "SelfhostDropResourceGraphWalkerUnsupportedEventRecord",
        "pub struct SelfhostDropResourceGraphWalkerUnsupportedEventRecord:",
    ],
    [
        "SelfhostDropResourceGraphWalkerInput",
        "pub struct SelfhostDropResourceGraphWalkerInput:",
    ],
    [
        "selfhost_drop_resource_graph_input_scanner_output_result",
        "pub fn selfhost_drop_resource_graph_input_scanner_output_result",
    ],
]) {
    assertDocBeforeTopLevel(source, docSnippet, declarationSnippet);
}
assertEveryTopLevelDeclarationHasDoc(source);
assert.doesNotMatch(
    facade,
    /memo_trait_operation_drop_resource_graph_input_scanner/,
    "Resource graph input scanner must remain facade-private until full Resource proof orchestration is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_drop_resource_graph_input_scanner/,
    "checker-layer Resource graph input scanner must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_drop_resource_graph_input_scanner_contract.js"),
    "source policy runner must execute the Resource graph input scanner contract",
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
        "#import \"./memo_trait_operation_drop_resource_no_escape_traversal_collector\" as *",
        "#import \"./memo_trait_operation_drop_resource_no_escape_materializer\" as *",
    ],
    "scanner must depend only on Vec, basic core helpers, typed HIR/effect/type ids, collector, and traversal table types",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:backend|resource\/|proof\/(?:api|solver|fact|query)\/resource|resource_tree|resource_graph|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_header|memo_trait_operation_evidence_producer|memo_trait_operation_impl_table|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_purity_gate|private_cache|private_state)/,
    "scanner must not import backend, Resource graph/proof internals, proof store/artifact, canonical-key, public-surface, evidence producer, impl table, scanner/materializer, purity gate, PrivateCache, or PrivateState layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostDropResourceGraphWalkerBodyRecord:",
        "type_id %SelfhostTypeId",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "effect %SelfhostEffectKind",
        "escape %SelfhostEffectEscapeState",
        "graph_id %SelfhostDropResourceGraphId",
        "completeness %SelfhostDropResourceGraphCompleteness",
    ],
    "walker body record must carry typed body identity, effect, escape, graph id, and upstream completeness",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostDropResourceGraphWalkerPlaceEventRecord:",
        "type_id %SelfhostTypeId",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "graph_id %SelfhostDropResourceGraphId",
        "operation_ordinal %i32",
        "place_id %SelfhostResourcePlaceId",
        "kind %SelfhostResourcePlaceKind",
    ],
    "walker place event must carry operation ordinal, place id, and typed place kind",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostDropResourceGraphWalkerEdgeEventRecord:",
        "type_id %SelfhostTypeId",
        "body_module_fingerprint %i32",
        "body_root %SelfhostHirExprId",
        "graph_id %SelfhostDropResourceGraphId",
        "operation_ordinal %i32",
        "from_place %SelfhostResourcePlaceId",
        "to_place %SelfhostResourcePlaceId",
        "kind %SelfhostResourceEdgeKind",
    ],
    "walker edge event must carry operation ordinal, endpoints, and typed edge kind",
);
for (const [kind, name] of [
    ["struct", "SelfhostDropResourceGraphWalkerBodyRecord"],
    ["struct", "SelfhostDropResourceGraphWalkerPlaceEventRecord"],
    ["struct", "SelfhostDropResourceGraphWalkerEdgeEventRecord"],
    ["struct", "SelfhostDropResourceGraphWalkerUnsupportedEventRecord"],
]) {
    assert.doesNotMatch(
        topLevelBlock(source, kind, name),
        /payload_hash|signature_hash|body_hash|public_surface|source_text|source_span|source_path|\bspan\b|\bpath\b|\bname\b|diagnostic|message|text/i,
        `${name} must not use source/display/hash authority instead of typed graph identity`,
    );
}
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_walker_validate_input_result"),
    [
        "selfhost_drop_resource_graph_walker_validate_all_bodies_loop input 0",
        "selfhost_drop_resource_graph_walker_validate_all_places_loop input 0",
        "selfhost_drop_resource_graph_walker_validate_all_edges_loop input 0",
        "selfhost_drop_resource_graph_walker_validate_all_unsupported_loop input 0",
    ],
    "scanner must preflight all body/place/edge/unsupported event tables before producing collector input",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_walker_validate_place_event_result"),
    [
        "selfhost_drop_resource_graph_walker_find_body_for_place input place",
        "selfhost_drop_resource_graph_walker_body_accepts_events body",
        "selfhost_drop_resource_graph_walker_validate_operation_count_result input body place.operation_ordinal",
        "EventForNonClosedGraph body.graph_id.index",
        "GraphBodyMissing place.graph_id.index",
    ],
    "place events must belong to a closed body and have a unique operation ordinal",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_walker_validate_edge_event_result"),
    [
        "selfhost_drop_resource_graph_walker_find_body_for_edge input edge",
        "selfhost_drop_resource_graph_walker_body_accepts_events body",
        "selfhost_drop_resource_graph_walker_validate_operation_count_result input body edge.operation_ordinal",
        "EventForNonClosedGraph body.graph_id.index",
        "GraphBodyMissing edge.graph_id.index",
    ],
    "edge events must belong to a closed body and have a unique operation ordinal",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_walker_validate_unsupported_event_result"),
    [
        "selfhost_drop_resource_graph_walker_find_body_for_unsupported input unsupported",
        "selfhost_drop_resource_graph_walker_body_accepts_events body",
        "selfhost_drop_resource_graph_walker_validate_operation_count_result input body unsupported.operation_ordinal",
        "EventForNonClosedGraph body.graph_id.index",
        "GraphBodyMissing unsupported.graph_id.index",
    ],
    "unsupported events must belong to a closed body and have a unique operation ordinal",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_walker_body_output_completeness"),
    [
        "SelfhostDropResourceGraphCompleteness::ClosedForDropBody:",
        "selfhost_drop_resource_graph_walker_has_unsupported_event input body",
        "SelfhostDropResourceGraphCompleteness::TraversalUnsupported",
        "SelfhostDropResourceGraphCompleteness::ClosedForDropBody",
        "SelfhostDropResourceGraphCompleteness::ResourceGraphMissing:",
        "SelfhostDropResourceGraphCompleteness::ResourceGraphMissing",
        "SelfhostDropResourceGraphCompleteness::TraversalUnsupported:",
        "SelfhostDropResourceGraphCompleteness::TraversalUnsupported",
    ],
    "unsupported events must override closed body completeness to TraversalUnsupported",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_input_scanner_output_result"),
    [
        "selfhost_drop_resource_graph_walker_validate_input_result source",
        "selfhost_drop_resource_graph_input_new",
        "selfhost_drop_resource_graph_input_scanner_body_loop output0 source 0",
        "selfhost_drop_resource_graph_input_scanner_place_loop output1 source 0",
        "selfhost_drop_resource_graph_input_scanner_edge_loop output2 source 0",
    ],
    "scanner output must validate first, then project body/place/edge events into collector input",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_input_scanner_body_loop"),
    [
        "selfhost_drop_resource_graph_walker_body_to_collector_body source body",
        "selfhost_drop_resource_graph_input_push_body output collector_body",
        "OutputCollectorInputRejected e",
    ],
    "body loop must use collector input constructor and preserve collector rejection as typed nested error",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_input_scanner_place_loop"),
    [
        "selfhost_drop_resource_graph_walker_body_emits_events source body",
        "selfhost_drop_resource_graph_walker_place_to_collector_place place",
        "selfhost_drop_resource_graph_input_push_place output collector_place",
    ],
    "place loop must skip events for unsupported/missing output bodies and use collector place push",
);
assertOrdered(
    functionBlock(source, "selfhost_drop_resource_graph_input_scanner_edge_loop"),
    [
        "selfhost_drop_resource_graph_walker_body_emits_events source body",
        "selfhost_drop_resource_graph_walker_edge_to_collector_edge edge",
        "selfhost_drop_resource_graph_input_push_edge output collector_edge",
    ],
    "edge loop must skip events for unsupported/missing output bodies and use collector edge push",
);
assert.doesNotMatch(
    code,
    /\b(SelfhostMemoTraitOperationDropNoEscapeProofTable|SelfhostMemoTraitOperationDropEvidence|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|SelfhostMemoTraitProofArtifact|PureDrop|NoDropRequired|prechecked|Prechecked|PrivateCacheMask|PrivateStateMask)\b/,
    "scanner must not synthesize proof tables, Drop evidence, aggregate proof, proof-store/artifact values, prechecked artifacts, or private effect mask proofs",
);
assert.doesNotMatch(
    code,
    /\bResult\s+[^ \n]+\s+(?:str|String|MlString)\b|\bResult::Err\s+"/,
    "scanner must use typed enum errors instead of string diagnostics as control flow",
);
assert.doesNotMatch(
    source,
    /(?:line|行数|comment|コメント)[^\n]{0,40}(?:<=|>=|<|>|max|maximum|cap|上限|制限)[^\n]{0,20}\d+|\d+[^\n]{0,20}(?:line|行|comment|コメント)/i,
    "source policy must not introduce numeric line/comment length caps; detailed docs are preferred",
);

console.log("[ok] selfhost Resource graph input scanner contract");
