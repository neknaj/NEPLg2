#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";
const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
const runner = fs.readFileSync(path.join(repoRoot, runnerRelPath), "utf8").replace(/\r\n/g, "\n");

function stripDocComments(src) {
    return src
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const declaration = kind === "fn"
        ? new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s`)
        : new RegExp(`^(?:pub\\s+)?${kind}\\s+${name}:`);
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

const code = stripDocComments(source);

assertOrdered(
    source,
    [
        "# codegen/memo_call_backend_private_cache_proof_gate",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "private-cache proof gate module must document purpose, contract, current limits, complexity, and doctest",
);
assert.ok(
    source.includes("caller が渡す proof table をそのまま authority にせず") &&
        source.includes("non-executable summary") &&
        source.includes("proof record / proof table / proof table writer / gate 本体は module-private") &&
        source.includes("Resource observation producer も同じ module-private proof table writer を使います") &&
        source.includes("public accepted path を追加せず") &&
        source.includes("stable artifact sidecar index"),
    "docs must state that caller proof tables are not direct authority, success is not executable backend output, table writes are private in phase 1, Resource observation uses the private writer without adding a public accepted path, and index optimization is a later contract-preserving change",
);
assert.doesNotMatch(
    source,
    /public entrypoint/,
    "docs must not describe the private proof gate as a public entrypoint",
);
assert.ok(
    runner.includes('"nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js"'),
    "private-cache proof gate source policy test must be registered in run_source_policy_regressions.js",
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
        "#import \"neplg2/core/infra/span\" as *",
        "#import \"neplg2/core/resolve/name_resolver\" as *",
        "#import \"neplg2/core/ty/effect\" as *",
        "#import \"neplg2/core/ty/ty\" as *",
        "#import \"./memo_call_backend_request\" as *",
        "#import \"./memo_call_backend_request_table\" as *",
    ],
    "private-cache proof gate imports must stay at Vec, HIR, identity, effect, type, request manifest, and request table layers",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|proof\/|memo_trait|PrivateCache|PrivateState|prechecked|wasm|llvm|lower|check\/expr|compiler_known|artifact|serializer|reader|neplobj|neplproof)/,
    "private-cache proof gate must not import Resource IR, proof store, memo trait proof layers, private cache/state implementation, prechecked artifacts, backend bytes, checker, compiler-known registry, or artifact IO",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+.*SelfhostMemoCallBackendRequestTable/m,
    "private-cache proof gate must not expose a public API that accepts a caller-supplied request table as authority",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoCallBackendPrivateCacheProofKey:",
        "memoized_expr_id %SelfhostHirExprId",
        "source_function_def_id %SelfhostDefId",
        "function_ty %SelfhostTypeId",
        "root_expr_id %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "request_kind %SelfhostMemoCallBackendRequestKind",
        "source_effect %SelfhostEffectKind",
        "type_arg_count %i32",
        "proof_kind %SelfhostMemoCallBackendPrivateCacheProofKind",
        "proof_schema_version %i32",
        "struct SelfhostMemoCallBackendPrivateCacheProofTable:",
        "records %Vec SelfhostMemoCallBackendPrivateCacheProofRecord",
    ],
    "private-cache proof key must bind memoized occurrence, source def, function type, root expr, body module fingerprint, request kind, effect, type arg count, proof kind, and schema, and proof table must be Vec-backed",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheProof(?:Status|Record|Table)\b/,
    "request evidence status, record, and table must stay private until a producer-owned Resource proof boundary exists",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheProofStatus|SelfhostMemoCallBackendPrivateCacheProofRecord|SelfhostMemoCallBackendPrivateCacheProofTable)\b/m,
    "public functions must not expose private request-evidence proof status, record, or table types in their signatures",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheResourceProof(?:Status|Record|Table)\b/,
    "Resource proof status, record, and table must stay private until the real Resource graph producer owns their construction",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheResourceProofStatus|SelfhostMemoCallBackendPrivateCacheResourceProofRecord|SelfhostMemoCallBackendPrivateCacheResourceProofTable)\b/m,
    "public functions must not expose private Resource proof status, record, or table types in their signatures",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheResource(?:GraphId|PlaceId|GraphCompleteness|PlaceKind|EdgeKind|GraphBodyRecord|GraphPlaceRecord|GraphEdgeRecord|GraphInput|GraphFoldSummary)\b/,
    "Resource graph input ids, payload records, owner input, and fold summary must stay private until the real Resource graph walker owns their construction",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheResourceGraphId|SelfhostMemoCallBackendPrivateCacheResourcePlaceId|SelfhostMemoCallBackendPrivateCacheResourceGraphCompleteness|SelfhostMemoCallBackendPrivateCacheResourcePlaceKind|SelfhostMemoCallBackendPrivateCacheResourceEdgeKind|SelfhostMemoCallBackendPrivateCacheResourceGraphBodyRecord|SelfhostMemoCallBackendPrivateCacheResourceGraphPlaceRecord|SelfhostMemoCallBackendPrivateCacheResourceGraphEdgeRecord|SelfhostMemoCallBackendPrivateCacheResourceGraphInput|SelfhostMemoCallBackendPrivateCacheResourceGraphFoldSummary)\b/m,
    "public functions must not expose private Resource graph input payload or owner types in their signatures",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_proof_(?:key_new|record_new|table_new|table_free|table_len)\b/m,
    "proof key/table constructors and owner operations must not be public accepted-path building blocks",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_proof_gate_from_hir_root_result\b/m,
    "request-evidence gate must stay module-private until a producer-owned Resource proof boundary owns the proof table",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_resource_proof_gate_from_hir_root_result\b/m,
    "Resource proof producer gate must stay module-private until the Resource graph walker owns the proof table input",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_resource_graph_gate_from_hir_root_result\b/m,
    "Resource graph producer gate must stay module-private until the actual Resource graph walker owns the graph input",
);
const tableArea = source.slice(
    source.indexOf("struct SelfhostMemoCallBackendPrivateCacheProofTable:"),
    source.indexOf("pub struct SelfhostMemoCallBackendPrivateCacheProofGateSummary:"),
);
assert.doesNotMatch(
    tableArea,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheProofTable/,
    "private-cache proof table owner must not implement Clone or Copy",
);
const resourceTableArea = source.slice(
    source.indexOf("struct SelfhostMemoCallBackendPrivateCacheResourceProofTable:"),
    source.indexOf("pub enum SelfhostMemoCallBackendPrivateCacheProofGateErrorKind:"),
);
assert.doesNotMatch(
    resourceTableArea,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheResourceProofTable/,
    "Resource proof table owner must not implement Clone or Copy",
);
const graphInputArea = source.slice(
    source.indexOf("struct SelfhostMemoCallBackendPrivateCacheResourceGraphInput:"),
    source.indexOf("pub enum SelfhostMemoCallBackendPrivateCacheResourceGraphProducerErrorKind:"),
);
assert.doesNotMatch(
    graphInputArea,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheResourceGraphInput/,
    "Resource graph input owner must not implement Clone or Copy",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoCallBackendPrivateCacheProofGateErrorKind:",
        "RequestCollectionFailed %SelfhostMemoCallBackendRequestCollectorErrorKind",
        "RequestEntryMissing %i32",
        "RequestExpressionMissing %SelfhostHirExprId",
        "RequestRecheckRejected %SelfhostMemoCallBackendRequestRejection",
        "RequestIdentityMismatch %SelfhostHirExprId",
        "BodyModuleFingerprintPlaceholder",
        "ProofTableAllocFailed %StdErrorKind",
        "ProofRecordPushFailed %StdErrorKind",
        "ProofRecordReadFailed %i32",
        "ProofMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ProofRefuted %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ProofDuplicate %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ProofOrphan %SelfhostMemoCallBackendPrivateCacheProofKey",
        "Stage0FixtureAllocFailed %StdErrorKind",
    ],
    "private-cache proof gate error taxonomy must keep collection, HIR recheck, placeholder fingerprint, proof table, missing/refuted/duplicate/orphan, and fixture failures distinct",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_error_code"),
    /_:/,
    "private-cache proof gate error code helper must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_request_table_from_hir_root_result module root fuel",
        "Result::Ok table:",
        "selfhost_memo_call_backend_private_cache_proof_gate_from_collected_table_result module table root body_module_fingerprint proofs",
        "Result::Err e:",
        "RequestCollectionFailed e",
    ],
    "public gate must build the request table internally from HIR root and wrap collector errors",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_from_collected_table_result"),
    [
        "selfhost_memo_call_backend_request_table_len &table",
        "selfhost_memo_call_backend_private_cache_proof_gate_recheck_loop_result module &table 0 request_count",
        "selfhost_memo_call_backend_private_cache_proof_gate_apply_loop_result &table proofs root_expr_id body_module_fingerprint 0 request_count",
        "selfhost_memo_call_backend_request_table_free table",
        "Result::Ok SelfhostMemoCallBackendPrivateCacheProofGateSummary request_count proven_count",
        "Result::Err e:",
        "selfhost_memo_call_backend_request_table_free table",
    ],
    "collected-table helper must recheck entries, apply proof records, and close the internal request table on success and errors",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_recheck_entry_result"),
    [
        "selfhost_hir_module_get_expr module entry.memoized_expr_id",
        "selfhost_memo_call_backend_request_from_hir_expr_result &expr",
        "selfhost_memo_call_backend_private_cache_proof_gate_request_record_matches &entry rebuilt",
        "RequestIdentityMismatch entry.memoized_expr_id",
        "RequestRecheckRejected SelfhostMemoCallBackendRequestRejection entry.memoized_expr_id request_error.kind",
        "RequestExpressionMissing entry.memoized_expr_id",
    ],
    "private-cache proof gate must re-read HIR payload for each request entry and re-run the typed request builder",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_request_record_matches"),
    [
        "stored.request_kind",
        "rebuilt.request_kind",
        "selfhost_def_id_eq stored.source_function_def_id rebuilt.source_function_def_id",
        "selfhost_type_id_eq stored.function_ty rebuilt.function_ty",
        "selfhost_effect_kind_eq stored.source_effect SelfhostEffectKind::Pure",
        "selfhost_effect_kind_eq rebuilt.source_effect SelfhostEffectKind::Pure",
        "eq stored.type_arg_count 0",
        "eq rebuilt.type_arg_count 0",
    ],
    "private-cache proof gate recheck must compare typed request fields instead of diagnostic metadata",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_key_eq"),
    [
        "a.memoized_expr_id",
        "b.memoized_expr_id",
        "selfhost_def_id_eq a.source_function_def_id b.source_function_def_id",
        "selfhost_type_id_eq a.function_ty b.function_ty",
        "a.root_expr_id",
        "b.root_expr_id",
        "eq a.body_module_fingerprint b.body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_request_kind_eq a.request_kind b.request_kind",
        "selfhost_effect_kind_eq a.source_effect b.source_effect",
        "eq a.type_arg_count b.type_arg_count",
        "selfhost_memo_call_backend_private_cache_proof_kind_eq a.proof_kind b.proof_kind",
        "eq a.proof_schema_version b.proof_schema_version",
    ],
    "proof key equality must use occurrence, source def, function type, root expr, body module fingerprint, request kind, source effect, type arg count, proof kind, and schema version",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_key_from_entry_result"),
    [
        "eq body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "selfhost_memo_call_backend_private_cache_proof_key_new entry.memoized_expr_id entry.request.source_function_def_id entry.request.function_ty root_expr_id body_module_fingerprint entry.request.request_kind entry.request.source_effect entry.request.type_arg_count",
    ],
    "proof key construction must reject placeholder body module fingerprints and derive key material from request entry, request kind/effect/type arguments, and root origin",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_lookup_loop"),
    [
        "selfhost_memo_call_backend_private_cache_proof_table_len proofs",
        "selfhost_memo_call_backend_private_cache_proof_lookup_finish key found",
        "field::get_ref proofs \"records\"",
        "v::get records idx",
        "selfhost_memo_call_backend_private_cache_proof_key_eq record.key key",
        "ProofDuplicate key",
        "some record.status",
        "ProofRecordReadFailed idx",
    ],
    "proof lookup must scan records by exact key, reject duplicate matching records, and fail on missing table entry",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_status_result"),
    [
        "SelfhostMemoCallBackendPrivateCacheProofStatus::RequestEvidenceProven:",
        "Result::Ok unit",
        "SelfhostMemoCallBackendPrivateCacheProofStatus::RequestEvidenceRefuted:",
        "ProofRefuted key",
    ],
    "proof status fold must accept only request-evidence Proven and reject request-evidence Refuted",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheResourceProofStatus"),
    [
        "PrivateCacheNoEscapeProven",
        "PrivateCacheMayEscape",
        "PrivateCacheMissing",
        "PrivateCacheUnknown",
    ],
    "Resource proof status must distinguish no-escape proof, escape, missing proof, and unknown proof explicitly",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_status_to_request_status_result"),
    [
        "SelfhostMemoCallBackendPrivateCacheResourceProofStatus::PrivateCacheNoEscapeProven:",
        "Result::Ok SelfhostMemoCallBackendPrivateCacheProofStatus::RequestEvidenceProven",
        "SelfhostMemoCallBackendPrivateCacheResourceProofStatus::PrivateCacheMayEscape:",
        "Result::Ok SelfhostMemoCallBackendPrivateCacheProofStatus::RequestEvidenceRefuted",
        "SelfhostMemoCallBackendPrivateCacheResourceProofStatus::PrivateCacheMissing:",
        "ResourceProofMissing key",
        "SelfhostMemoCallBackendPrivateCacheResourceProofStatus::PrivateCacheUnknown:",
        "ResourceProofUnknown key",
    ],
    "Resource status fold must translate no-escape into request evidence, translate escape into refuted request evidence, and keep missing/unknown as producer errors",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_status_to_request_status_result"),
    /PrivateCache(?:Missing|Unknown)[\s\S]*?Result::Ok/,
    "Resource Missing and Unknown status must not become successful request evidence",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_proof_gate_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_proof_table_to_request_evidence_result resource_proofs",
        "Result::Ok request_proofs:",
        "selfhost_memo_call_backend_private_cache_proof_gate_from_hir_root_result module root fuel body_module_fingerprint &request_proofs",
        "selfhost_memo_call_backend_private_cache_proof_table_free request_proofs",
        "Result::Err gate_error:",
        "RequestEvidenceGateRejected gate_error",
    ],
    "Resource proof producer gate must convert private Resource observations through the module-private request-evidence table and wrap the existing private gate rejection",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheResourceGraphCompleteness"),
    [
        "ClosedForPrivateCacheBoundary",
        "ResourceGraphMissing",
        "TraversalUnsupported",
    ],
    "Resource graph completeness must distinguish closed, missing, and unsupported graph inputs explicitly",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind"),
    [
        "PrivateCacheStorage",
        "PrivateCacheEntry",
        "ReturnedOwnedClone",
        "ReturnCacheReference",
        "PublicStore",
        "ExternalHandle",
        "UnsupportedPlace",
    ],
    "Resource graph place kind must distinguish private cache storage, owned clone output, escaping references/stores, external handles, and unsupported places",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind"),
    [
        "Owns",
        "BorrowView",
        "CloneOutOwnedValue",
        "ReturnCacheReference",
        "StoreToPublic",
        "CallBoundaryUnsupported",
    ],
    "Resource graph edge kind must distinguish private ownership/view/clone-out edges from escaping and unsupported edges",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheResourceGraphProducerErrorKind"),
    [
        "GraphBodyTableAllocFailed %StdErrorKind",
        "GraphPlaceTableAllocFailed %StdErrorKind",
        "GraphEdgeTableAllocFailed %StdErrorKind",
        "GraphBodyPushFailed %StdErrorKind",
        "GraphPlacePushFailed %StdErrorKind",
        "GraphEdgePushFailed %StdErrorKind",
        "GraphBodyReadFailed %i32",
        "GraphPlaceReadFailed %i32",
        "GraphEdgeReadFailed %i32",
        "GraphBodyDuplicate %SelfhostMemoCallBackendPrivateCacheProofKey",
        "GraphPlaceDuplicate %i32",
        "GraphEdgeDuplicate %i32",
        "BodyModuleFingerprintPlaceholder",
        "GraphIdInvalid %i32",
        "PlaceIdInvalid %i32",
        "OperationOrdinalInvalid %i32",
        "GraphBodyMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "GraphEventForNonClosedGraph %i32",
        "EdgeEndpointMissing %i32",
        "OutputResourceProofRejected %SelfhostMemoCallBackendPrivateCacheResourceProofProducerErrorKind",
        "Stage0FixtureAllocFailed %StdErrorKind",
    ],
    "Resource graph producer error taxonomy must keep allocation, read, duplicate, invalid id, orphan, non-closed graph event, endpoint, lower producer, and fixture failures distinct",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_error_code"),
    /_:/,
    "Resource graph producer error code helper must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_validate_input_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_graph_validate_all_bodies_loop input 0",
        "selfhost_memo_call_backend_private_cache_resource_graph_validate_all_places_loop input 0",
        "selfhost_memo_call_backend_private_cache_resource_graph_validate_all_edges_loop input 0",
    ],
    "Resource graph input validation must preflight bodies, places, and edges before producing proof records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_validate_place_result"),
    [
        "GraphIdInvalid place.graph_id.index",
        "OperationOrdinalInvalid place.operation_ordinal",
        "PlaceIdInvalid place.place_id.index",
    ],
    "Resource graph place validation must directly reject invalid graph id, operation ordinal, and place id",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_validate_edge_result"),
    [
        "GraphIdInvalid edge.graph_id.index",
        "OperationOrdinalInvalid edge.operation_ordinal",
        "PlaceIdInvalid edge.from_place.index",
        "PlaceIdInvalid edge.to_place.index",
    ],
    "Resource graph edge validation must directly reject invalid graph id, operation ordinal, and endpoint place ids",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_validate_all_places_loop"),
    [
        "selfhost_memo_call_backend_private_cache_resource_graph_seen_place_before_result input place idx",
        "GraphPlaceDuplicate place.place_id.index",
        "selfhost_memo_call_backend_private_cache_resource_graph_place_has_closed_body_result input place",
    ],
    "Resource graph place validation must reject duplicate places and require a matching closed body",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_validate_all_edges_loop"),
    [
        "selfhost_memo_call_backend_private_cache_resource_graph_seen_edge_before_result input edge idx",
        "GraphEdgeDuplicate edge.operation_ordinal",
        "selfhost_memo_call_backend_private_cache_resource_graph_edge_has_closed_body_result input edge",
        "selfhost_memo_call_backend_private_cache_resource_graph_validate_edge_endpoints_result input edge",
    ],
    "Resource graph edge validation must reject duplicate edges, require a matching closed body, and verify both endpoints",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_place_has_closed_body_loop"),
    [
        "selfhost_memo_call_backend_private_cache_resource_graph_completeness_is_closed body.completeness",
        "Result::Ok unit",
        "GraphEventForNonClosedGraph place.graph_id.index",
    ],
    "Resource graph place validation must reject events attached to a non-closed graph body",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_edge_has_closed_body_loop"),
    [
        "selfhost_memo_call_backend_private_cache_resource_graph_completeness_is_closed body.completeness",
        "Result::Ok unit",
        "GraphEventForNonClosedGraph edge.graph_id.index",
    ],
    "Resource graph edge validation must reject events attached to a non-closed graph body",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_closed_status_result"),
    [
        "PrivateCacheNoEscapeProven false",
        "selfhost_memo_call_backend_private_cache_resource_graph_fold_places_loop input body 0 initial",
        "place_summary.saw_place",
        "selfhost_memo_call_backend_private_cache_resource_graph_fold_edges_loop input body 0 place_summary",
        "PrivateCacheUnknown",
    ],
    "closed Resource graph status fold must reject empty closed graphs by returning Unknown instead of no-escape proof",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_body_to_status_result"),
    [
        "ClosedForPrivateCacheBoundary:",
        "selfhost_memo_call_backend_private_cache_resource_graph_closed_status_result input body",
        "ResourceGraphMissing:",
        "PrivateCacheMissing",
        "TraversalUnsupported:",
        "PrivateCacheUnknown",
    ],
    "Resource graph body status fold must keep missing and unsupported graph inputs out of no-escape proof",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_place_status"),
    [
        "PrivateCacheStorage:",
        "PrivateCacheNoEscapeProven",
        "PrivateCacheEntry:",
        "PrivateCacheNoEscapeProven",
        "ReturnedOwnedClone:",
        "PrivateCacheNoEscapeProven",
        "ReturnCacheReference:",
        "PrivateCacheMayEscape",
        "PublicStore:",
        "PrivateCacheMayEscape",
        "ExternalHandle:",
        "PrivateCacheMayEscape",
        "UnsupportedPlace:",
        "PrivateCacheUnknown",
    ],
    "Resource graph place status fold must map escaping places to MayEscape and unsupported places to Unknown",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_edge_status"),
    [
        "Owns:",
        "PrivateCacheNoEscapeProven",
        "BorrowView:",
        "PrivateCacheNoEscapeProven",
        "CloneOutOwnedValue:",
        "PrivateCacheNoEscapeProven",
        "ReturnCacheReference:",
        "PrivateCacheMayEscape",
        "StoreToPublic:",
        "PrivateCacheMayEscape",
        "CallBoundaryUnsupported:",
        "PrivateCacheUnknown",
    ],
    "Resource graph edge status fold must map escaping edges to MayEscape and unsupported call boundaries to Unknown",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_gate_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_graph_input_to_resource_proof_table_result graph",
        "Result::Ok resource_proofs:",
        "selfhost_memo_call_backend_private_cache_resource_proof_gate_from_hir_root_result module root fuel body_module_fingerprint &resource_proofs",
        "selfhost_memo_call_backend_private_cache_resource_proof_table_free resource_proofs",
        "OutputResourceProofRejected e",
    ],
    "Resource graph producer gate must lower graph input through private Resource proof table before calling the existing Resource proof gate",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_producer_stage0"),
    [
        "PrivateCacheStorage 0 SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::CloneOutOwnedValue",
        "may_escape_rejected",
        "ReturnCacheReference",
        "missing_rejected",
        "ResourceGraphMissing",
        "unknown_rejected",
        "TraversalUnsupported",
        "duplicate_rejected",
        "endpoint_missing_rejected",
        "PrivateCacheStorage 9 SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::Owns",
    ],
    "Resource graph producer stage0 must cover accepted, may-escape, missing, unknown, duplicate, and endpoint-missing paths",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_table_push"),
    /^pub\s+fn/m,
    "private-cache proof table push must stay private until a producer-owned Resource proof boundary exists",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_proof_gate_validate_orphan_loop"),
    [
        "v::get records idx",
        "selfhost_memo_call_backend_private_cache_proof_gate_request_table_contains_key_loop table root_expr_id body_module_fingerprint record.key 0 request_count",
        "Result::Ok found:",
        "found",
        "ProofOrphan record.key",
        "ProofRecordReadFailed idx",
    ],
    "proof gate must reject proof records that do not correspond to a request entry from the current root",
);
assert.doesNotMatch(
    code,
    /ProofDeferred|Deferred|ProofReady|StableKeyReady|Result::Ok\s+.*ProofMissing|Result::Ok\s+.*ProofRefuted|PrivateCacheProofUnavailable/,
    "private-cache proof gate must not model missing/refuted proof as a successful ready/deferred plan or reuse the preflight proof-unavailable error",
);
assert.doesNotMatch(
    code,
    /diagnostic_symbol|diagnostic_span|string_search::str_eq|candidate\.name|memo_call"/,
    "private-cache proof gate must not use display symbol, diagnostic span, candidate name, string search, or memo_call string literal as authority",
);
assert.doesNotMatch(
    code,
    /cache_lookup|cache_insert|CacheAlloc|CacheDrop|Wasm|LLVM|wasm_|llvm_|sealed|backend_bytes|neplobj|neplproof/,
    "private-cache proof gate must not create executable cache operations, backend bytes, sealed representation, or persistent artifact IO",
);
assert.doesNotMatch(
    source,
    /line[_-]?count|doc(?:umentation)?[_-]?comment(?:s)?[^\n]*(?:limit|cap|max)|max[_-]?(?:lines|doc)/i,
    "private-cache proof gate source policy must not introduce line-count or doc-comment length limits",
);

console.log("selfhost memo_call backend private cache proof gate contract ok");
