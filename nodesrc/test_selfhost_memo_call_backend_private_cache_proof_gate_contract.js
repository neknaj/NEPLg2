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
        source.includes("Resource walker input scanner は、future actual Resource IR walker が返す typed event stream を module-private GraphInput へ正規化します") &&
        source.includes("Resource observation ban stage0") &&
        source.includes("unified stream normalizer") &&
        source.includes("Actual walker event producer bridge stage0") &&
        source.includes("Actual walker operation classifier stage0") &&
        source.includes("Actual walker traversal source stage0") &&
        source.includes("Actual walker operation producer bridge stage0") &&
        source.includes("Private cache region proof stage0") &&
        source.includes("Region no-escape candidate stage0") &&
        source.includes("Fresh region witness stage0") &&
        source.includes("Fresh witness request-evidence stage0") &&
        source.includes("Collector-owned traversal bundle stage0") &&
        source.includes("Operation-classified traversal bundle stage0") &&
        source.includes("public accepted path を追加せず") &&
        source.includes("stable artifact sidecar index"),
    "docs must state that caller proof tables are not direct authority, success is not executable backend output, table writes are private in phase 1, Resource observation uses the private writer, walker input scanner only normalizes typed events, observation-ban stage0, unified stream normalizer, HIR-root unified event producer bridge, operation classifier, traversal source, operation producer bridge, region proof, no-escape candidate checker, fresh witness bridge, and request-evidence bridge are present, no public accepted path is added, and index optimization is later contract-preserving work",
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
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheResourceWalker(?:UnsupportedReason|BodyRecord|PlaceEventRecord|EdgeEventRecord|UnsupportedEventRecord|Input)\b/,
    "Resource walker input event payloads and owner input must stay private until actual Resource IR walker owns the event stream",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason|SelfhostMemoCallBackendPrivateCacheResourceWalkerBodyRecord|SelfhostMemoCallBackendPrivateCacheResourceWalkerPlaceEventRecord|SelfhostMemoCallBackendPrivateCacheResourceWalkerEdgeEventRecord|SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedEventRecord|SelfhostMemoCallBackendPrivateCacheResourceWalkerInput)\b/m,
    "public functions must not expose private Resource walker event payload or owner types in their signatures",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheObservation(?:Kind|BanStatus|BanRecord|BanTable)\b/,
    "observation ban kind, status, record, and owner table must stay private until actual Resource IR walker owns observation production",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheObservationKind|SelfhostMemoCallBackendPrivateCacheObservationBanStatus|SelfhostMemoCallBackendPrivateCacheObservationBanRecord|SelfhostMemoCallBackendPrivateCacheObservationBanTable)\b/m,
    "public functions must not expose private observation ban payload or owner table types in their signatures",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualWalkerEvent(?:Payload|Table|SplitOutput)\b/,
    "actual walker unified event payload, owner table, and split output must stay private until actual Resource IR walker owns stream production",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload|SelfhostMemoCallBackendPrivateCacheActualWalkerEventTable|SelfhostMemoCallBackendPrivateCacheActualWalkerEventSplitOutput)\b/m,
    "public functions must not expose private actual walker unified event payload or owner types in their signatures",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSource(?:Kind|Record|Table)\b/,
    "actual walker traversal source kind, record, and owner table must stay private until actual Resource IR traversal owns source production",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind|SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceRecord|SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable)\b/m,
    "public functions must not expose private actual walker traversal source payload or owner table types in their signatures",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheRegion(?:Proof(?:InputKind|InputRecord|Status|Record|Table)|NoEscapeCandidate(?:Status|Record)|FreshWitness(?:Status|Record|Table))\b/,
    "private cache region proof input/status payload, no-escape candidate payload, fresh witness payload, and owner tables must stay private until actual Resource IR traversal and effect masking own the proof boundary",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheRegionProofInputKind|SelfhostMemoCallBackendPrivateCacheRegionProofInputRecord|SelfhostMemoCallBackendPrivateCacheRegionProofStatus|SelfhostMemoCallBackendPrivateCacheRegionProofRecord|SelfhostMemoCallBackendPrivateCacheRegionProofTable|SelfhostMemoCallBackendPrivateCacheRegionNoEscapeCandidateStatus|SelfhostMemoCallBackendPrivateCacheRegionNoEscapeCandidateRecord|SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessStatus|SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessRecord|SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessTable)\b/m,
    "public functions must not expose private cache region proof input/status payload, no-escape candidate payload, fresh witness payload, or owner table types in their signatures",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualWalkerOperation(?:Kind|Record|Table)\b/,
    "actual walker operation classifier kind, record, and owner table must stay private until actual Resource IR traversal owns operation production",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind|SelfhostMemoCallBackendPrivateCacheActualWalkerOperationRecord|SelfhostMemoCallBackendPrivateCacheActualWalkerOperationTable)\b/m,
    "public functions must not expose private actual walker operation classifier payload or owner table types in their signatures",
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
const walkerInputArea = source.slice(
    source.indexOf("struct SelfhostMemoCallBackendPrivateCacheResourceWalkerInput:"),
    source.indexOf("pub enum SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind:"),
);
assert.doesNotMatch(
    walkerInputArea,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheResourceWalkerInput/,
    "Resource walker input owner must not implement Clone or Copy",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheResourceWalkerInput\b/,
    "Resource walker input owner Clone/Copy ban must apply to the whole module, not only the declaration area",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheObservationBanTable\b/,
    "observation ban table owner must not implement Clone or Copy",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualWalkerEventTable\b/,
    "actual walker unified event table owner must not implement Clone or Copy",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualWalkerEventSplitOutput\b/,
    "actual walker split output owner pair must not implement Clone or Copy",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable\b/,
    "actual walker traversal source table owner must not implement Clone or Copy",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheRegionProofTable\b/,
    "private cache region proof table owner must not implement Clone or Copy",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessTable\b/,
    "fresh region witness table owner must not implement Clone or Copy",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualWalkerOperationTable\b/,
    "actual walker operation table owner must not implement Clone or Copy",
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
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason"),
    [
        "UnknownResourceOperation",
        "UnknownPlaceRoot",
        "UnknownProjection",
        "UnknownCallBoundary",
        "CacheObservationUnsupported",
        "FunctionIdentityObservationUnsupported",
        "RawIdentityObservationUnsupported",
        "PrivateStateBoundaryUnsupported",
    ],
    "Resource walker unsupported reason must use typed event reasons instead of display strings",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheObservationKind"),
    [
        "CacheHitObserved",
        "CacheMissObserved",
        "CacheSizeObserved",
        "CacheStatsObserved",
        "CacheClearObserved",
        "CacheDebugObserved",
        "CacheRegionIdentityObserved",
        "FunctionEqualityObserved",
        "FunctionHashObserved",
        "FunctionDebugObserved",
        "ClosureAllocationIdentityObserved",
        "RawIdentityObserved",
        "RawRepresentationObserved",
        "UnsupportedObservation",
    ],
    "observation kind must distinguish cache observation, function identity observation, raw identity observation, and unsupported observation explicitly",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheObservationBanStatus"),
    [
        "NoObservationDetected",
        "ObservationDetected %SelfhostMemoCallBackendPrivateCacheObservationKind",
    ],
    "observation ban status must keep one-record no-observation separate from a detected observation",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload"),
    [
        "Body %SelfhostMemoCallBackendPrivateCacheResourceWalkerBodyRecord",
        "Place %SelfhostMemoCallBackendPrivateCacheResourceWalkerPlaceEventRecord",
        "Edge %SelfhostMemoCallBackendPrivateCacheResourceWalkerEdgeEventRecord",
        "Unsupported %SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedEventRecord",
        "Observation %SelfhostMemoCallBackendPrivateCacheObservationBanRecord",
    ],
    "actual walker unified event payload must distinguish body, place, edge, unsupported, and observation events explicitly",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualWalkerEventTable"),
    [
        "events %Vec SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload",
    ],
    "actual walker unified event table must be a Vec-backed private owner",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualWalkerEventSplitOutput"),
    [
        "walker_input %SelfhostMemoCallBackendPrivateCacheResourceWalkerInput",
        "observations %SelfhostMemoCallBackendPrivateCacheObservationBanTable",
    ],
    "actual walker unified event split output must own both the graph-side walker input and the observation-side table",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualWalkerEventNormalizerErrorKind"),
    [
        "ActualWalkerEventTableAllocFailed %StdErrorKind",
        "ActualWalkerEventPushFailed %StdErrorKind",
        "ActualWalkerEventReadFailed %i32",
        "WalkerInputBuildRejected %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
        "ObservationTableBuildRejected %StdErrorKind",
        "ScannerOutputRejected %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
        "GraphGateRejected %SelfhostMemoCallBackendPrivateCacheResourceGraphProducerErrorKind",
        "ObservationGateRejected %SelfhostMemoCallBackendPrivateCacheObservationBanProducerErrorKind",
        "Stage0FixtureAllocFailed %StdErrorKind",
    ],
    "actual walker event normalizer error taxonomy must distinguish unified table, walker input, observation table, scanner, graph gate, observation gate, and fixture failures",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheObservationBanProducerErrorKind"),
    [
        "RequestCollectionFailed %SelfhostMemoCallBackendRequestCollectorErrorKind",
        "RequestEntryMissing %i32",
        "RequestRecheckRejected %SelfhostMemoCallBackendPrivateCacheProofGateErrorKind",
        "ProofKeyRejected %SelfhostMemoCallBackendPrivateCacheProofGateErrorKind",
        "ObservationRecordReadFailed %i32",
        "WalkerInputBuildRejected %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
        "ScannerOutputRejected %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
        "OutputGraphGateRejected %SelfhostMemoCallBackendPrivateCacheResourceGraphProducerErrorKind",
        "Stage0FixtureAllocFailed %StdErrorKind",
    ],
    "observation ban producer error taxonomy must distinguish request collection, request recheck, proof key, observation read, walker input, scanner output, graph gate, and fixture failures",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_observation_kind_to_unsupported_reason"),
    /_:/,
    "observation kind classifier must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_observation_kind_to_unsupported_reason"),
    [
        "CacheHitObserved:",
        "CacheObservationUnsupported",
        "CacheMissObserved:",
        "CacheObservationUnsupported",
        "CacheSizeObserved:",
        "CacheObservationUnsupported",
        "CacheStatsObserved:",
        "CacheObservationUnsupported",
        "CacheClearObserved:",
        "CacheObservationUnsupported",
        "CacheDebugObserved:",
        "CacheObservationUnsupported",
        "CacheRegionIdentityObserved:",
        "CacheObservationUnsupported",
        "FunctionEqualityObserved:",
        "FunctionIdentityObservationUnsupported",
        "FunctionHashObserved:",
        "FunctionIdentityObservationUnsupported",
        "FunctionDebugObserved:",
        "FunctionIdentityObservationUnsupported",
        "ClosureAllocationIdentityObserved:",
        "FunctionIdentityObservationUnsupported",
        "RawIdentityObserved:",
        "RawIdentityObservationUnsupported",
        "RawRepresentationObserved:",
        "RawIdentityObservationUnsupported",
        "UnsupportedObservation:",
        "UnknownResourceOperation",
    ],
    "observation kind classifier must map every visible observation class to the correct fail-closed unsupported reason",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_observation_status_to_reason"),
    [
        "NoObservationDetected:",
        "none",
        "ObservationDetected kind:",
        "some selfhost_memo_call_backend_private_cache_observation_kind_to_unsupported_reason kind",
    ],
    "observation ban status fold must not treat a single no-observation record as proof, and must map detected observations through the typed classifier",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_split_loop"),
    /_:/,
    "actual walker event split loop must not use wildcard fallback for unified event payloads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_split_loop"),
    [
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload::Body body:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_push_body input body",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload::Place place:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_push_place input place",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload::Edge edge:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_push_edge input edge",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload::Unsupported unsupported:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_push_unsupported input unsupported",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload::Observation record:",
        "NoObservationDetected:",
        "ObservationDetected _kind:",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_push observations record",
    ],
    "actual walker event split loop must route graph events through ResourceWalkerInput, detected observations through ObservationBanTable, and keep no-observation records neutral",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_split_loop"),
    /PrivateCacheNoEscapeProven|PrivateCacheStorage|CloneOutOwnedValue|resource_graph_input_push|proof_table_push/,
    "actual walker event split loop must not synthesize accepted proof, accepted private-cache graph payload, GraphInput, or proof table records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_split_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_input_new",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_new",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_split_loop &events input0 observations0 0 n",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_table_free events",
        "ObservationTableBuildRejected e",
        "WalkerInputBuildRejected e",
    ],
    "actual walker event split must allocate existing owner tables, run the splitter, free the unified table on success or failure, and preserve typed allocation errors",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_gate_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_event_split_result events",
        "field::get output \"walker_input\"",
        "field::get output \"observations\"",
        "gt selfhost_memo_call_backend_private_cache_observation_ban_table_len &observations 0",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_observation_gate_result module root fuel body_module_fingerprint input observations",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_graph_gate_result module root fuel body_module_fingerprint input",
    ],
    "actual walker event gate must split the unified stream, prioritize detected observations, free empty observation owners, and send graph-only streams through the graph path",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_graph_gate_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_graph_input_scanner_output_result input",
        "selfhost_memo_call_backend_private_cache_resource_graph_gate_from_hir_root_result module root fuel body_module_fingerprint &graph",
        "selfhost_memo_call_backend_private_cache_resource_graph_input_free graph",
        "GraphGateRejected e",
        "ScannerOutputRejected e",
    ],
    "actual walker event graph path must pass through the existing scanner and graph gate, close GraphInput, and wrap scanner/graph failures",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_observation_gate_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "selfhost_memo_call_backend_private_cache_observation_ban_gate_from_hir_root_result module root fuel body_module_fingerprint &observations",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "ObservationGateRejected e",
    ],
    "actual walker event observation path must close the graph-side walker input and route observations through the existing observation ban gate",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_walker_event_(?:split|gate|graph_gate|observation_gate|table|stage0_(?:body|unsupported|observation|mixed|run|push|table))/m,
    "actual walker event normalizer internals must stay module-private and must not expose private unified stream construction as public accepted-path APIs",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_normalizer_error_code"),
    /_:/,
    "actual walker event normalizer error code helper must not use wildcard fallback",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_normalizer_unknown_result_eq"),
    /_:/,
    "actual walker event normalizer unknown result helper must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_normalizer_stage0"),
    [
        "unsupported_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_stage0_unsupported_result",
        "observation_rejected",
        "CacheHitObserved",
        "mixed_observation_rejected",
        "FunctionEqualityObserved",
    ],
    "actual walker event normalizer stage0 must cover unsupported graph-only, observation-only, and mixed graph/observation streams",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind"),
    [
        "RequestCollectionFailed %SelfhostMemoCallBackendRequestCollectorErrorKind",
        "RequestEntryMissing %i32",
        "RequestRecheckRejected %SelfhostMemoCallBackendPrivateCacheProofGateErrorKind",
        "ProofKeyRejected %SelfhostMemoCallBackendPrivateCacheProofGateErrorKind",
        "ActualWalkerTraversalInputRejected %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
        "ActualTraversalBodyInputMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputUnavailable %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputUnsupported %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputMalformed %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
        "ActualWalkerTraversalBodyReadFailed %i32",
        "ActualWalkerTraversalPlaceReadFailed %i32",
        "ActualWalkerTraversalEdgeReadFailed %i32",
        "ActualWalkerTraversalUnsupportedReadFailed %i32",
        "ActualWalkerTraversalObservationReadFailed %i32",
        "ActualWalkerTraversalSourceTableAllocFailed %StdErrorKind",
        "ActualWalkerTraversalSourcePushFailed %StdErrorKind",
        "ActualWalkerTraversalSourceReadFailed %i32",
        "ActualWalkerOperationTableAllocFailed %StdErrorKind",
        "ActualWalkerOperationPushFailed %StdErrorKind",
        "ActualWalkerOperationReadFailed %i32",
        "ActualWalkerEventBuildRejected %SelfhostMemoCallBackendPrivateCacheActualWalkerEventNormalizerErrorKind",
        "NormalizerRejected %SelfhostMemoCallBackendPrivateCacheActualWalkerEventNormalizerErrorKind",
        "Stage0FixtureAllocFailed %StdErrorKind",
    ],
    "actual walker event producer bridge error taxonomy must distinguish request collection, request entry, request recheck, proof key, body-input availability, traversal input validation/read, traversal source table, traversal source push/read, operation table, operation push/read, event build, normalizer, and fixture failures",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_error_code"),
    /_:/,
    "actual walker event producer bridge error code helper must not use wildcard fallback",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_unknown_result_eq"),
    /_:/,
    "actual walker event producer bridge unknown result helper must not use wildcard fallback",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_refuted_result_eq"),
    /_:/,
    "actual walker event producer bridge refuted result helper must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_events_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_request_table_from_hir_root_result module root fuel",
        "Result::Ok table:",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_table_new",
        "selfhost_memo_call_backend_request_table_len &table",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_append_requests_loop module &table events0 root body_module_fingerprint 0 request_count",
        "selfhost_memo_call_backend_request_table_free table",
        "RequestCollectionFailed e",
    ],
    "actual walker event producer bridge must build request authority internally from HIR root, create a private unified event table, append request-derived events, and close the request table",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_append_request_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_gate_recheck_entry_result module entry",
        "selfhost_memo_call_backend_private_cache_proof_key_from_entry_result entry root_expr_id body_module_fingerprint",
        "SelfhostMemoCallBackendPrivateCacheResourceGraphCompleteness::ClosedForPrivateCacheBoundary",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload::Body body",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_table_push events body_payload",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::UnknownResourceOperation",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload::Unsupported unsupported",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_table_push events1 unsupported_payload",
    ],
    "actual walker event producer bridge must recheck each request entry, derive the proof key from the request, and emit only body plus typed unsupported unified events while actual traversal is unavailable",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_append_request_result"),
    /PrivateCacheNoEscapeProven|PrivateCacheStorage|CloneOutOwnedValue|resource_graph_input_push|proof_table_push/,
    "actual walker event producer bridge must not synthesize accepted proof, accepted private-cache graph payload, GraphInput, or proof table records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_gate_events_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_event_gate_from_hir_root_result module root fuel body_module_fingerprint events",
        "Result::Ok summary:",
        "NormalizerRejected e",
    ],
    "actual walker event producer bridge must pass its producer-owned unified event table through the existing normalizer gate",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_from_hir_root_result"),
    /resource_graph_input_scanner_output_result|resource_graph_gate_from_hir_root_result|observation_ban_gate_from_hir_root_result/,
    "actual walker event producer bridge must not bypass the unified event normalizer by directly calling scanner, graph gate, or observation ban gate",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_from_hir_root_with_stage0_observation_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_events_from_hir_root_result module root fuel body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_append_stage0_observation_result events0 kind",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_gate_events_result module root fuel body_module_fingerprint events1",
    ],
    "actual walker event producer bridge observation fixture must append detected observations to the private unified stream and still use the normalizer gate",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_(?:events_from_hir_root_result|append_request_result|append_requests_loop|append_stage0_observation_result|gate_events_result|from_hir_root_result|from_hir_root_with_stage0_observation_result|stage0_run|stage0_observation)/m,
    "actual walker event producer bridge internals must stay module-private and must not expose private unified event construction or injected observation fixtures",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_stage0"),
    [
        "unsupported_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_stage0_run_i32_result 77",
        "observation_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_stage0_observation_run_i32_result 77 SelfhostMemoCallBackendPrivateCacheObservationKind::CacheHitObserved",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_stage0_run_i32_result 0",
    ],
    "actual walker event producer bridge stage0 must cover unsupported traversal, observation precedence, and placeholder fingerprint rejection without exposing private unified event tables",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind"),
    [
        "PrivateCacheStoragePlace",
        "PrivateCacheEntryPlace",
        "ReturnedOwnedClonePlace",
        "ReturnCacheReferencePlace",
        "PublicStorePlace",
        "ExternalHandlePlace",
        "UnsupportedPlace",
        "OwnsEdge",
        "BorrowViewEdge",
        "CloneOutOwnedValueEdge",
        "ReturnCacheReferenceEdge",
        "PublicStoreEdge",
        "CallBoundaryUnsupportedEdge",
        "CacheHitObservation",
        "FunctionIdentityObservation",
        "RawIdentityObservation",
        "UnsupportedTraversalSource",
        "UnsupportedObservationSource",
        "ResourceIrTraversalUnavailable",
    ],
    "actual walker traversal source kind must keep accepted, escaping, observation, and unavailable traversal vocabulary typed before operation projection",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceRecord"),
    [
        "key %SelfhostMemoCallBackendPrivateCacheProofKey",
        "graph_id %SelfhostMemoCallBackendPrivateCacheResourceGraphId",
        "operation_ordinal %i32",
        "from_place %SelfhostMemoCallBackendPrivateCacheResourcePlaceId",
        "to_place %SelfhostMemoCallBackendPrivateCacheResourcePlaceId",
        "kind %SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind",
    ],
    "actual walker traversal source record must retain request key, graph id, ordinal, endpoint ids, and typed source kind",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_push"),
    [
        "eq record.key.body_module_fingerprint 0",
        "lt record.graph_id.index 0",
        "lt record.operation_ordinal 0",
        "lt record.from_place.index 0",
        "lt record.to_place.index 0",
        "ActualWalkerTraversalSourcePushFailed StdErrorKind::InvalidOperation",
    ],
    "actual walker traversal source table push must reject placeholder fingerprints and invalid graph/operation/place ids before projection",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_to_operation_record"),
    /_:/,
    "actual walker traversal source to operation projection must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_to_operation_record"),
    [
        "PrivateCacheStoragePlace:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::PrivateCacheStoragePlace",
        "PrivateCacheEntryPlace:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::PrivateCacheEntryPlace",
        "ReturnedOwnedClonePlace:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::ReturnedOwnedClonePlace",
        "ReturnCacheReferencePlace:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::ReturnCacheReferencePlace",
        "PublicStorePlace:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::PublicStorePlace",
        "ExternalHandlePlace:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::ExternalHandlePlace",
        "UnsupportedPlace:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::UnsupportedPlace",
        "OwnsEdge:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::OwnsEdge",
        "BorrowViewEdge:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::BorrowViewEdge",
        "CloneOutOwnedValueEdge:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CloneOutOwnedValueEdge",
        "ReturnCacheReferenceEdge:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::ReturnCacheReferenceEdge",
        "PublicStoreEdge:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::PublicStoreEdge",
        "CallBoundaryUnsupportedEdge:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CallBoundaryUnsupportedEdge",
        "CacheHitObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheHitObservation",
        "FunctionIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::FunctionIdentityObservation",
        "RawIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::RawIdentityObservation",
        "UnsupportedTraversalSource:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::UnsupportedTraversalSource",
        "UnsupportedObservationSource:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::UnsupportedObservationSource",
        "ResourceIrTraversalUnavailable:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::UnknownResourceOperation",
    ],
    "actual walker traversal source projection must explicitly map every source class into the operation classifier vocabulary",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_to_operation_record"),
    /PrivateCacheNoEscapeProven|proof_table_push|resource_graph_input_push|GraphInput|Wasm|LLVM/,
    "actual walker traversal source projection may only create operation records and must not synthesize accepted proof, direct GraphInput, proof table records, or backend bytes",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind"),
    [
        "PrivateCacheStoragePlace",
        "PrivateCacheEntryPlace",
        "ReturnedOwnedClonePlace",
        "ReturnCacheReferencePlace",
        "PublicStorePlace",
        "ExternalHandlePlace",
        "UnsupportedPlace",
        "OwnsEdge",
        "BorrowViewEdge",
        "CloneOutOwnedValueEdge",
        "ReturnCacheReferenceEdge",
        "PublicStoreEdge",
        "CallBoundaryUnsupportedEdge",
        "UnsupportedTraversalSource",
        "UnsupportedObservationSource",
        "UnknownResourceOperation",
        "CacheHitObservation",
        "FunctionIdentityObservation",
        "RawIdentityObservation",
    ],
    "actual walker operation classifier must keep accepted graph, escaping graph, unknown operation, and observation operation vocabulary typed and explicit",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationRecord"),
    [
        "key %SelfhostMemoCallBackendPrivateCacheProofKey",
        "graph_id %SelfhostMemoCallBackendPrivateCacheResourceGraphId",
        "operation_ordinal %i32",
        "from_place %SelfhostMemoCallBackendPrivateCacheResourcePlaceId",
        "to_place %SelfhostMemoCallBackendPrivateCacheResourcePlaceId",
        "kind %SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind",
    ],
    "actual walker operation record must retain request key, graph id, ordinal, endpoint ids, and typed operation kind",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_push"),
    [
        "eq record.key.body_module_fingerprint 0",
        "lt record.graph_id.index 0",
        "lt record.operation_ordinal 0",
        "lt record.from_place.index 0",
        "lt record.to_place.index 0",
        "ActualWalkerOperationPushFailed StdErrorKind::InvalidOperation",
    ],
    "actual walker operation table push must reject placeholder fingerprints and invalid graph/operation/place ids before classification",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_append_record_result"),
    /_:/,
    "actual walker operation classifier must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_append_record_result"),
    [
        "PrivateCacheStoragePlace:",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::PrivateCacheStorage",
        "PrivateCacheEntryPlace:",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::PrivateCacheEntry",
        "ReturnedOwnedClonePlace:",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::ReturnedOwnedClone",
        "ReturnCacheReferencePlace:",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::ReturnCacheReference",
        "PublicStorePlace:",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::PublicStore",
        "ExternalHandlePlace:",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::ExternalHandle",
        "UnsupportedPlace:",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::UnsupportedPlace",
        "OwnsEdge:",
        "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::Owns",
        "BorrowViewEdge:",
        "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::BorrowView",
        "CloneOutOwnedValueEdge:",
        "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::CloneOutOwnedValue",
        "ReturnCacheReferenceEdge:",
        "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::ReturnCacheReference",
        "PublicStoreEdge:",
        "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::StoreToPublic",
        "CallBoundaryUnsupportedEdge:",
        "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::CallBoundaryUnsupported",
        "UnsupportedTraversalSource:",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::UnknownResourceOperation",
        "UnsupportedObservationSource:",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::UnknownProjection",
        "UnknownResourceOperation:",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::UnknownResourceOperation",
        "CacheHitObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::CacheHitObserved",
        "FunctionIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::FunctionEqualityObserved",
        "RawIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::RawIdentityObserved",
    ],
    "actual walker operation classifier must route accepted graph, escaping graph, unknown operation, and observations to distinct typed event payloads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_append_request_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_gate_recheck_entry_result module entry",
        "selfhost_memo_call_backend_private_cache_proof_key_from_entry_result entry root_expr_id body_module_fingerprint",
        "SelfhostMemoCallBackendPrivateCacheResourceGraphCompleteness::ClosedForPrivateCacheBoundary",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload::Body body",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_append_records_loop operations events1 key graph_id 0",
    ],
    "actual walker operation classifier must recheck request authority, create only body headers itself, and classify operation records for graph/observation events",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_from_hir_root_result"),
    /resource_graph_input_scanner_output_result|resource_graph_gate_from_hir_root_result|observation_ban_gate_from_hir_root_result/,
    "actual walker operation classifier must not bypass the unified event normalizer by directly calling scanner, graph gate, or observation ban gate",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_events_from_hir_root_result module root fuel body_module_fingerprint operations",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_gate_events_result module root fuel body_module_fingerprint events",
    ],
    "actual walker operation classifier must pass classified unified event stream through the existing normalizer gate",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_walker_operation_(?:record|table|classifier_stage0_(?:run|accepted|escape|single)|classifier_(?:append|events|from)|stage0_(?:record|push|closed|escape|single))/m,
    "actual walker operation classifier internals must stay module-private and must not expose private operation table or classifier construction APIs",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_stage0"),
    [
        "accepted_result",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_stage0_accepted_result 77",
        "may_escape_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_stage0_escape_result 77",
        "unsupported_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::UnknownResourceOperation",
        "observation_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheHitObservation",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_stage0_accepted_result 0",
    ],
    "actual walker operation classifier stage0 must cover accepted, escaping, unknown, observation, and placeholder paths",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_append_record_result"),
    /PrivateCacheNoEscapeProven|proof_table_push|resource_graph_input_push|Wasm|LLVM/,
    "actual walker operation classifier must not synthesize proof table records, direct GraphInput, or backend bytes",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationProducerBridgeStage0Summary"),
    [
        "unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
    ],
    "actual walker operation producer bridge stage0 summary must expose only typed result payloads and not private operation tables",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceProjectionStage0Summary"),
    [
        "accepted_result %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "may_escape_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
    ],
    "actual walker traversal source projection stage0 summary must expose only typed result payloads and not private source or operation tables",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0"),
    [
        "accepted_result",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_accepted_result 77",
        "may_escape_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_escape_result 77",
        "unsupported_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::ResourceIrTraversalUnavailable",
        "observation_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CacheHitObservation",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_accepted_result 0",
    ],
    "actual walker traversal source projection stage0 must cover accepted, escaping, unknown, observation, and placeholder paths through source projection",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_run_summary_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_operations_from_sources_result &sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_from_hir_root_result &module root 8 body_module_fingerprint &operations",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_free operations",
    ],
    "actual walker traversal source projection fixture must project source records to operation records, close owners, and use the existing operation classifier",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_run_summary_result"),
    /resource_graph_input_push|proof_table_push|PrivateCacheNoEscapeProven|Wasm|LLVM/,
    "actual walker traversal source projection fixture must not synthesize direct GraphInput, proof table records, accepted proof, or backend bytes",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_(?:record|push|closed|escape|single|run|accepted)/m,
    "actual walker traversal source projection helpers must stay module-private and only the typed smoke summary may be public",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceCollectorStage0Summary"),
    [
        "accepted_result %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "neutral_edge_accepted_result %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "may_escape_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
    ],
    "actual walker traversal source collector stage0 summary must expose only typed result payloads and not private walker input, observation, source, or operation tables",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_place_kind"),
    /_:/,
    "actual walker traversal source place-kind projection must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_place_kind"),
    [
        "PrivateCacheStorage:",
        "PrivateCacheStoragePlace",
        "PrivateCacheEntry:",
        "PrivateCacheEntryPlace",
        "ReturnedOwnedClone:",
        "ReturnedOwnedClonePlace",
        "ReturnCacheReference:",
        "ReturnCacheReferencePlace",
        "PublicStore:",
        "PublicStorePlace",
        "ExternalHandle:",
        "ExternalHandlePlace",
        "UnsupportedPlace:",
        "UnsupportedPlace",
    ],
    "actual walker traversal source place-kind projection must preserve accepted, escaping, external, and unsupported place vocabulary before proof folding",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_place_kind"),
    /ExternalHandle:[\s\S]*?ResourceIrTraversalUnavailable|UnsupportedPlace:[\s\S]*?ResourceIrTraversalUnavailable/,
    "actual walker traversal source place-kind projection must not turn known external or unsupported places into traversal-unavailable source",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_edge_kind"),
    /_:/,
    "actual walker traversal source edge-kind projection must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_edge_kind"),
    [
        "Owns:",
        "OwnsEdge",
        "BorrowView:",
        "BorrowViewEdge",
        "CloneOutOwnedValue:",
        "CloneOutOwnedValueEdge",
        "ReturnCacheReference:",
        "ReturnCacheReferenceEdge",
        "StoreToPublic:",
        "PublicStoreEdge",
        "CallBoundaryUnsupported:",
        "CallBoundaryUnsupportedEdge",
    ],
    "actual walker traversal source edge-kind projection must preserve neutral, clone-out, escaping, public-store, and unsupported edge vocabulary before proof folding",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_edge_kind"),
    /Owns:\s*\n\s*SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CloneOutOwnedValueEdge|BorrowView:\s*\n\s*SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CloneOutOwnedValueEdge|CallBoundaryUnsupported:\s*\n\s*SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::ResourceIrTraversalUnavailable/,
    "actual walker traversal source edge-kind projection must not disguise neutral edges as clone-out or known unsupported edges as traversal-unavailable source",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_observation_kind"),
    /_:/,
    "actual walker traversal source observation-kind projection must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_observation_kind"),
    [
        "CacheHitObserved:",
        "CacheHitObservation",
        "CacheMissObserved:",
        "CacheHitObservation",
        "FunctionEqualityObserved:",
        "FunctionIdentityObservation",
        "ClosureAllocationIdentityObserved:",
        "FunctionIdentityObservation",
        "RawIdentityObserved:",
        "RawIdentityObservation",
        "UnsupportedObservation:",
        "UnsupportedObservationSource",
    ],
    "actual walker traversal source observation-kind projection must keep cache, function identity, raw identity, and unsupported observation classes typed",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_observation_kind"),
    /UnsupportedObservation:\s*\n\s*SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::ResourceIrTraversalUnavailable/,
    "actual walker traversal source observation-kind projection must not turn known unsupported observations into traversal-unavailable source",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_observation_status"),
    [
        "NoObservationDetected:",
        "none",
        "ObservationDetected kind:",
        "some selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_observation_kind kind",
    ],
    "actual walker traversal source observation status projection must not turn no-observation into accepted proof evidence",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_validate_input_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_validate_bodies_loop input 0 selfhost_memo_call_backend_private_cache_resource_walker_body_len input",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_validate_places_loop input 0 selfhost_memo_call_backend_private_cache_resource_walker_place_len input",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_validate_edges_loop input 0 selfhost_memo_call_backend_private_cache_resource_walker_edge_len input",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_validate_unsupported_loop input 0 selfhost_memo_call_backend_private_cache_resource_walker_unsupported_len input",
    ],
    "actual walker traversal source collector must validate walker body, place, edge, and unsupported event tables before source collection",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collect_from_walker_input_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_new",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_validate_input_result input",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_append_places_loop input sources0 0 selfhost_memo_call_backend_private_cache_resource_walker_place_len input",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_append_edges_loop input sources1 0 selfhost_memo_call_backend_private_cache_resource_walker_edge_len input",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_append_unsupported_loop input sources2 0 selfhost_memo_call_backend_private_cache_resource_walker_unsupported_len input",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_append_observations_loop observations sources3 0 selfhost_memo_call_backend_private_cache_observation_ban_table_len observations",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collector_fail_with_sources sources0 e",
    ],
    "actual walker traversal source collector must build only a source table from validated borrowed walker input and observation tables",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_append_unsupported_result"),
    [
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::UnsupportedTraversalSource",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_push sources record",
    ],
    "actual walker traversal source collector must preserve known unsupported traversal events as unsupported source, not traversal-unavailable",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collect_from_walker_input_result"),
    /resource_graph_input_push|proof_table_push|PrivateCacheNoEscapeProven|Wasm|LLVM|actual_walker_operation_table_new/,
    "actual walker traversal source collector must not synthesize GraphInput, proof table records, accepted proof, backend bytes, or operation tables",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collector_stage0"),
    [
        "accepted_result",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::PrivateCacheStorage",
        "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::CloneOutOwnedValue",
        "neutral_edge_accepted_result",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::PrivateCacheStorage",
        "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::Owns",
        "may_escape_rejected",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::ExternalHandle",
        "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::BorrowView",
        "unsupported_rejected",
        "selfhost_memo_call_backend_private_cache_resource_walker_stage0_unsupported_input_result",
        "observation_rejected",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::CacheHitObserved",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_resource_walker_stage0_placeholder_input_result",
    ],
    "actual walker traversal source collector stage0 must cover accepted clone-out, neutral edge, escaping, unsupported, observation, and placeholder paths",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_(?:collector_fail|place_kind|edge_kind|observation_kind|observation_status|validate|append|collect_from|collector_stage0_)/m,
    "actual walker traversal source collector helpers must stay module-private and only the typed smoke summary may be public",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheRegionProofInputKind"),
    [
        "PrivateCacheRegionRoot",
        "PrivateCacheRegionEntry",
        "ReturnedOwnedValue",
        "InternalOwnsEdge",
        "InternalBorrowViewEdge",
        "OwnedCloneOutEdge",
        "EscapingReference",
        "PublicStoreEscape",
        "ExternalHandleEscape",
        "CacheStateObservation",
        "FunctionIdentityObservation",
        "RawIdentityObservation",
        "UnsupportedTraversal",
        "UnsupportedObservation",
        "TraversalUnavailable",
    ],
    "private cache region proof input kind must distinguish accepted candidates, escapes, observations, unsupported sources, and traversal-unavailable sources",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheRegionProofStatus"),
    [
        "PrivateCacheRegionRootCandidate",
        "PrivateCacheRegionSupportCandidate",
        "PrivateCacheRegionMayEscape",
        "PrivateCacheRegionObservationRejected",
        "PrivateCacheRegionUnsupported",
        "PrivateCacheRegionUnavailable",
    ],
    "private cache region proof status must keep root candidate, support candidate, escape, observation, unsupported, and unavailable results distinct",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheRegionProofInputRecord"),
    [
        "key %SelfhostMemoCallBackendPrivateCacheProofKey",
        "graph_id %SelfhostMemoCallBackendPrivateCacheResourceGraphId",
        "operation_ordinal %i32",
        "from_place %SelfhostMemoCallBackendPrivateCacheResourcePlaceId",
        "to_place %SelfhostMemoCallBackendPrivateCacheResourcePlaceId",
        "kind %SelfhostMemoCallBackendPrivateCacheRegionProofInputKind",
    ],
    "private cache region proof input record must retain request key, graph id, ordinal, endpoint ids, and typed input kind",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheRegionProofRecord"),
    [
        "key %SelfhostMemoCallBackendPrivateCacheProofKey",
        "graph_id %SelfhostMemoCallBackendPrivateCacheResourceGraphId",
        "operation_ordinal %i32",
        "status %SelfhostMemoCallBackendPrivateCacheRegionProofStatus",
    ],
    "private cache region proof record must retain request key, graph id, ordinal, and typed proof status",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheRegionProofTable"),
    ["records %Vec SelfhostMemoCallBackendPrivateCacheRegionProofRecord"],
    "private cache region proof table must be a Vec-backed owner table",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheRegionNoEscapeCandidateStatus"),
    ["PrivateCacheRegionNoEscapeCandidateAccepted"],
    "region no-escape candidate status must stay distinct from Resource no-escape proof status",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheRegionNoEscapeCandidateRecord"),
    [
        "key %SelfhostMemoCallBackendPrivateCacheProofKey",
        "graph_id %SelfhostMemoCallBackendPrivateCacheResourceGraphId",
        "root_operation_ordinal %i32",
        "support_operation_ordinal %i32",
        "status %SelfhostMemoCallBackendPrivateCacheRegionNoEscapeCandidateStatus",
    ],
    "region no-escape candidate record must retain the single request key, graph id, root/support ordinals, and candidate-only status",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessStatus"),
    [
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
        "PrivateCacheRegionFreshWitnessMissing",
        "PrivateCacheRegionFreshWitnessRejected",
        "PrivateCacheRegionFreshWitnessUnavailable",
    ],
    "fresh region witness status must keep accepted, missing, rejected, and unavailable states distinct",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessRecord"),
    [
        "key %SelfhostMemoCallBackendPrivateCacheProofKey",
        "graph_id %SelfhostMemoCallBackendPrivateCacheResourceGraphId",
        "root_operation_ordinal %i32",
        "support_operation_ordinal %i32",
        "status %SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessStatus",
    ],
    "fresh region witness record must retain only key, graph id, root/support ordinals, and typed witness status as authority",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessTable"),
    ["records %Vec SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessRecord"],
    "fresh region witness table must be a private Vec-backed owner table",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind"),
    [
        "RegionProofTableAllocFailed %StdErrorKind",
        "RegionProofRecordPushFailed %StdErrorKind",
        "RegionProofRecordReadFailed %i32",
        "TraversalSourceReadFailed %i32",
        "RegionProofEmpty",
        "RegionProofBodyModuleFingerprintPlaceholder %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionProofGraphIdInvalid %i32",
        "RegionProofOperationOrdinalInvalid %i32",
        "RegionProofKeyMismatch %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionProofGraphMismatch %i32",
        "RegionProofRootDuplicate %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionProofSupportDuplicate %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionProofOperationOrdinalDuplicate %i32",
        "RegionFreshWitnessTableAllocFailed %StdErrorKind",
        "RegionFreshWitnessRecordPushFailed %StdErrorKind",
        "RegionFreshWitnessRecordReadFailed %i32",
        "RegionFreshWitnessEmpty",
        "RegionFreshWitnessBodyModuleFingerprintPlaceholder %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionFreshWitnessGraphIdInvalid %i32",
        "RegionFreshWitnessOperationOrdinalInvalid %i32",
        "RegionFreshWitnessKeyMismatch %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionFreshWitnessGraphMismatch %i32",
        "RegionFreshWitnessRootOrdinalMismatch %i32",
        "RegionFreshWitnessSupportOrdinalMismatch %i32",
        "RegionFreshWitnessRootSupportOrdinalDuplicate %i32",
        "RegionFreshWitnessDuplicate %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionFreshWitnessMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionFreshWitnessRejected %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionFreshWitnessUnavailable %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionFreshWitnessResourceProofRejected %SelfhostMemoCallBackendPrivateCacheResourceProofProducerErrorKind",
        "RegionProofMayEscape %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionProofObservationRejected %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionProofUnsupported %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionProofUnavailable %SelfhostMemoCallBackendPrivateCacheProofKey",
        "RegionProofMissingFreshRegion",
        "Stage0SourceRejected %SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
    ],
    "private cache region proof producer errors must keep allocation, read, empty, malformed origin, mismatch, duplicate, fresh witness, escape, observation, unsupported, unavailable, missing, and source-fixture failures distinct",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_input_kind_from_source_kind"),
    /_:/,
    "private cache region proof input kind projection must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_input_kind_from_source_kind"),
    [
        "PrivateCacheStoragePlace:",
        "PrivateCacheRegionRoot",
        "PrivateCacheEntryPlace:",
        "PrivateCacheRegionEntry",
        "ReturnedOwnedClonePlace:",
        "ReturnedOwnedValue",
        "ReturnCacheReferencePlace:",
        "EscapingReference",
        "PublicStorePlace:",
        "PublicStoreEscape",
        "ExternalHandlePlace:",
        "ExternalHandleEscape",
        "UnsupportedPlace:",
        "UnsupportedTraversal",
        "OwnsEdge:",
        "InternalOwnsEdge",
        "BorrowViewEdge:",
        "InternalBorrowViewEdge",
        "CloneOutOwnedValueEdge:",
        "OwnedCloneOutEdge",
        "CacheHitObservation:",
        "CacheStateObservation",
        "FunctionIdentityObservation:",
        "FunctionIdentityObservation",
        "RawIdentityObservation:",
        "RawIdentityObservation",
        "UnsupportedTraversalSource:",
        "UnsupportedTraversal",
        "UnsupportedObservationSource:",
        "UnsupportedObservation",
        "ResourceIrTraversalUnavailable:",
        "TraversalUnavailable",
    ],
    "private cache region proof input projection must preserve accepted candidates, neutral edges, escaping edges, observations, unsupported sources, and unavailable traversal",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_status_from_input_kind"),
    /_:/,
    "private cache region proof status projection must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_status_from_input_kind"),
    [
        "PrivateCacheRegionRoot:",
        "PrivateCacheRegionRootCandidate",
        "PrivateCacheRegionEntry:",
        "PrivateCacheRegionSupportCandidate",
        "ReturnedOwnedValue:",
        "PrivateCacheRegionSupportCandidate",
        "InternalOwnsEdge:",
        "PrivateCacheRegionSupportCandidate",
        "OwnedCloneOutEdge:",
        "PrivateCacheRegionSupportCandidate",
        "EscapingReference:",
        "PrivateCacheRegionMayEscape",
        "CacheStateObservation:",
        "PrivateCacheRegionObservationRejected",
        "UnsupportedTraversal:",
        "PrivateCacheRegionUnsupported",
        "TraversalUnavailable:",
        "PrivateCacheRegionUnavailable",
    ],
    "private cache region proof status projection must map root and support source classes separately and all bad sources to distinct fail-closed statuses",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_table_push"),
    [
        "eq record.key.body_module_fingerprint 0",
        "lt record.graph_id.index 0",
        "lt record.operation_ordinal 0",
        "RegionProofRecordPushFailed StdErrorKind::InvalidOperation",
    ],
    "private cache region proof table push must reject placeholder fingerprints and invalid graph/operation ids",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_collect_sources_loop"),
    /resource_graph_input_push|proof_table_push|PrivateCacheNoEscapeProven|Wasm|LLVM|PrivateCacheInPureFunction|mask_private/,
    "private cache region proof source collection must not synthesize GraphInput, request proof table records, accepted Resource proof, backend bytes, or effect masking",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_fold_loop"),
    /_:/,
    "private cache region proof fold must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_fold_loop"),
    [
        "and seen_region_root seen_support",
        "RegionProofMissingFreshRegion",
        "PrivateCacheRegionRootCandidate:",
        "selfhost_memo_call_backend_private_cache_region_proof_fold_loop table add idx 1 n true seen_support",
        "PrivateCacheRegionSupportCandidate:",
        "selfhost_memo_call_backend_private_cache_region_proof_fold_loop table add idx 1 n seen_region_root true",
        "PrivateCacheRegionMayEscape:",
        "RegionProofMayEscape record.key",
        "PrivateCacheRegionObservationRejected:",
        "RegionProofObservationRejected record.key",
        "PrivateCacheRegionUnsupported:",
        "RegionProofUnsupported record.key",
        "PrivateCacheRegionUnavailable:",
        "RegionProofUnavailable record.key",
    ],
    "private cache region proof fold must require both a region root candidate and a support candidate while keeping escape, observation, unsupported, and unavailable rejection distinct",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheRegionProofStage0Summary"),
    [
        "accepted_result %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "neutral_edge_accepted_result %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "entry_without_root_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "returned_value_without_root_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "owns_edge_without_root_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "clone_out_edge_without_root_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "may_escape_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "unavailable_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
    ],
    "private cache region proof stage0 summary must expose only typed result payloads and not private source or region proof owner tables",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_stage0"),
    [
        "accepted_result",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_closed_clone_table_result",
        "neutral_edge_accepted_result",
        "selfhost_memo_call_backend_private_cache_region_proof_stage0_neutral_edge_table_result",
        "entry_without_root_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::PrivateCacheEntryPlace",
        "returned_value_without_root_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::ReturnedOwnedClonePlace",
        "owns_edge_without_root_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::OwnsEdge",
        "clone_out_edge_without_root_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CloneOutOwnedValueEdge",
        "may_escape_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_escape_table_result",
        "observation_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CacheHitObservation",
        "unsupported_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::UnsupportedTraversalSource",
        "unavailable_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::ResourceIrTraversalUnavailable",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_region_proof_stage0_placeholder_result",
    ],
    "private cache region proof stage0 must cover accepted, neutral, rootless support, escaping, observation, unsupported, unavailable, and placeholder paths",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_stage0")),
    /PrivateCacheNoEscapeProven|proof_table_push|resource_graph_input_push|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend/,
    "private cache region proof stage0 must not synthesize accepted Resource proof, request proof records, GraphInput, backend bytes, or effect masking",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_scan_record_result"),
    /_:/,
    "region no-escape candidate scan must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_scan_record_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_key_eq record.key expected_key",
        "selfhost_memo_call_backend_private_cache_resource_graph_id_eq record.graph_id expected_graph_id",
        "PrivateCacheRegionRootCandidate:",
        "RegionProofRootDuplicate record.key",
        "RegionProofOperationOrdinalDuplicate record.operation_ordinal",
        "PrivateCacheRegionSupportCandidate:",
        "RegionProofSupportDuplicate record.key",
        "RegionProofOperationOrdinalDuplicate record.operation_ordinal",
        "PrivateCacheRegionMayEscape:",
        "RegionProofMayEscape record.key",
        "PrivateCacheRegionObservationRejected:",
        "RegionProofObservationRejected record.key",
        "PrivateCacheRegionUnsupported:",
        "RegionProofUnsupported record.key",
        "PrivateCacheRegionUnavailable:",
        "RegionProofUnavailable record.key",
        "RegionProofGraphMismatch record.graph_id.index",
        "RegionProofKeyMismatch record.key",
    ],
    "region no-escape candidate scan must require one key, one graph, one root, one support, unique ordinal, and distinct bad-status errors",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_loop"),
    /_:/,
    "region no-escape candidate loop must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_loop"),
    [
        "and seen_root seen_support",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_record_new expected_key expected_graph_id root_operation_ordinal support_operation_ordinal",
        "RegionProofMissingFreshRegion",
        "RegionProofRecordReadFailed idx",
    ],
    "region no-escape candidate loop must require both root and support before returning candidate-only accepted status",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_from_table_result"),
    [
        "eq n 0",
        "RegionProofEmpty",
        "selfhost_memo_call_backend_private_cache_region_proof_table_get table 0",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_record_validate_result first",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_loop table 0 n first.key first.graph_id false -1 false -1",
    ],
    "region no-escape candidate table checker must reject empty tables and seed the single key/graph expectation from the first validated record",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_stage0")),
    /PrivateCacheNoEscapeProven|RequestEvidenceProven|proof_table_push|resource_proof_table_push|resource_graph_input_push|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend/,
    "region no-escape candidate stage0 must not synthesize Resource proof, request-evidence proof, GraphInput, backend bytes, or effect masking",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheRegionNoEscapeCandidateStage0Summary"),
    [
        "accepted_result %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "empty_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "key_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "graph_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "root_duplicate_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "support_duplicate_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "ordinal_duplicate_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "missing_support_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "may_escape_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "unavailable_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
    ],
    "region no-escape candidate stage0 summary must expose only typed Result payloads for accepted and fail-closed representative paths",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_stage0"),
    [
        "accepted_result",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_closed_clone_table_result",
        "empty_rejected",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_stage0_empty_result",
        "key_mismatch_rejected",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_stage0_key_mismatch_result",
        "graph_mismatch_rejected",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_stage0_graph_mismatch_result",
        "root_duplicate_rejected",
        "PrivateCacheRegionRootCandidate",
        "support_duplicate_rejected",
        "PrivateCacheRegionSupportCandidate",
        "ordinal_duplicate_rejected",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_stage0_ordinal_duplicate_result",
        "missing_support_rejected",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_stage0_missing_support_result",
        "may_escape_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_escape_table_result",
        "observation_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CacheHitObservation",
        "unsupported_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::UnsupportedTraversalSource",
        "unavailable_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::ResourceIrTraversalUnavailable",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_region_proof_stage0_placeholder_result",
    ],
    "region no-escape candidate stage0 must cover accepted, empty, key/graph mismatch, duplicate, missing support, escaping, observation, unsupported, unavailable, and placeholder paths",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_record_validate_result"),
    [
        "eq record.key.body_module_fingerprint 0",
        "RegionFreshWitnessBodyModuleFingerprintPlaceholder record.key",
        "lt record.graph_id.index 0",
        "RegionFreshWitnessGraphIdInvalid record.graph_id.index",
        "lt record.root_operation_ordinal 0",
        "RegionFreshWitnessOperationOrdinalInvalid record.root_operation_ordinal",
        "lt record.support_operation_ordinal 0",
        "RegionFreshWitnessOperationOrdinalInvalid record.support_operation_ordinal",
        "eq record.root_operation_ordinal record.support_operation_ordinal",
        "RegionFreshWitnessRootSupportOrdinalDuplicate record.root_operation_ordinal",
    ],
    "fresh witness validation must reject placeholder key, invalid graph, invalid ordinal, and root/support ordinal collision before proof table generation",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_scan_record_result"),
    /_:/,
    "fresh witness scan must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_scan_record_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_key_eq record.key candidate.key",
        "selfhost_memo_call_backend_private_cache_resource_graph_id_eq record.graph_id candidate.graph_id",
        "eq record.root_operation_ordinal candidate.root_operation_ordinal",
        "eq record.support_operation_ordinal candidate.support_operation_ordinal",
        "PrivateCacheRegionFreshWitnessCandidateAccepted:",
        "RegionFreshWitnessDuplicate record.key",
        "PrivateCacheRegionFreshWitnessMissing:",
        "RegionFreshWitnessMissing record.key",
        "PrivateCacheRegionFreshWitnessRejected:",
        "RegionFreshWitnessRejected record.key",
        "PrivateCacheRegionFreshWitnessUnavailable:",
        "RegionFreshWitnessUnavailable record.key",
        "RegionFreshWitnessSupportOrdinalMismatch record.support_operation_ordinal",
        "RegionFreshWitnessRootOrdinalMismatch record.root_operation_ordinal",
        "RegionFreshWitnessGraphMismatch record.graph_id.index",
        "RegionFreshWitnessKeyMismatch record.key",
    ],
    "fresh witness scan must require exact key, graph, root/support ordinals, accepted status, and distinct fail-closed status errors",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_loop"),
    /_:/,
    "fresh witness loop must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_loop"),
    [
        "if:",
        "seen",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_resource_table_from_candidate candidate",
        "RegionFreshWitnessMissing candidate.key",
        "RegionFreshWitnessRecordReadFailed idx",
    ],
    "fresh witness loop must require exactly one accepted witness before producing the module-private Resource proof table",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_resource_table_from_candidate"),
    [
        "selfhost_memo_call_backend_private_cache_resource_proof_table_new",
        "selfhost_memo_call_backend_private_cache_resource_proof_record_new candidate.key SelfhostMemoCallBackendPrivateCacheResourceProofStatus::PrivateCacheNoEscapeProven",
        "selfhost_memo_call_backend_private_cache_resource_proof_table_push table0 resource_record",
        "RegionFreshWitnessResourceProofRejected e",
    ],
    "fresh witness bridge may create only a module-private Resource proof table record and must keep lower Resource proof construction failures typed",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_resource_table_from_candidate")),
    /resource_proof_gate_from_hir_root_result|resource_proof_table_to_request_evidence_result|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|resource_graph_input_push|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend|neplobj|neplproof/,
    "fresh witness Resource table bridge must not call request-evidence gate, synthesize request proof table records, GraphInput, backend bytes, effect masking, or artifact keys",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_stage0")),
    /resource_proof_gate_from_hir_root_result|resource_proof_table_to_request_evidence_result|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|resource_graph_input_push|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend|neplobj|neplproof/,
    "fresh witness stage0 must not call request-evidence gate, synthesize request proof table records, GraphInput, backend bytes, effect masking, or artifact keys",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_request_evidence_gate_result"),
    [
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_resource_table_result candidate &witnesses",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_free witnesses",
        "selfhost_memo_call_backend_private_cache_resource_proof_gate_from_hir_root_result module root fuel body_module_fingerprint &resource_proofs",
        "selfhost_memo_call_backend_private_cache_resource_proof_table_free resource_proofs",
        "RegionFreshWitnessResourceProofRejected e",
    ],
    "fresh witness request-evidence bridge must consume witness owner, build Resource proof table, call only the existing Resource proof gate, and close owner tables",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_request_evidence_stage0_run_summary_with_table_result"),
    [
        "match witness_result:",
        "Result::Ok witnesses:",
        "match selfhost_memo_call_backend_private_cache_region_fresh_witness_stage0_candidate_result:",
        "Result::Ok candidate:",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_request_evidence_gate_result &module root 8 body_module_fingerprint candidate witnesses",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_free witnesses",
        "Stage0FixtureAllocFailed e",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_free witnesses",
        "Result::Err e",
    ],
    "fresh witness request-evidence stage0 helper must close witness owner when module fixture or candidate construction fails",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_request_evidence_gate_result")),
    /resource_proof_table_to_request_evidence_result|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|resource_graph_input_push|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend|neplobj|neplproof/,
    "fresh witness request-evidence bridge must not bypass the existing Resource proof gate or create backend/effect/artifact outputs",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessRequestEvidenceStage0Summary"),
    [
        "accepted_request_count %i32",
        "accepted_proof_count %i32",
        "body_fingerprint_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "missing_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "rejected_status_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
    ],
    "fresh witness request-evidence stage0 summary must expose only counts and typed Result payloads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_request_evidence_stage0"),
    [
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_request_evidence_stage0_run_summary_result 77 77 0 0 1",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
        "request_evidence_stage0_run_i32_result 78 77 0 0 1",
        "missing_witness_rejected",
        "PrivateCacheRegionFreshWitnessMissing",
        "rejected_status_rejected",
        "PrivateCacheRegionFreshWitnessRejected",
        "accepted.request_count",
        "accepted.proven_request_count",
    ],
    "fresh witness request-evidence stage0 must cover accepted HIR-root gate path, body fingerprint mismatch, missing witness, and rejected witness",
);
assert.doesNotMatch(
    source,
    /^pub\s+struct\s+SelfhostMemoCallBackendPrivateCacheActualTraversalBundle\b/m,
    "actual traversal bundle owner must remain module-private and must not be public",
);
assert.doesNotMatch(
    source,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualTraversalBundle\b/,
    "actual traversal bundle owner must not implement Clone or Copy",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalBundle"),
    [
        "sources %SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable",
        "witnesses %SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessTable",
    ],
    "actual traversal bundle must carry only the source table owner and fresh witness table owner",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_free"),
    [
        'field::get bundle "sources"',
        'field::get bundle "witnesses"',
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_free witnesses",
    ],
    "actual traversal bundle cleanup must close source owner before witness owner",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0_accepted_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_closed_clone_table_result",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0_with_sources_result sources witness_body_module_fingerprint 0 0 1 status",
        "Stage0SourceRejected e",
    ],
    "actual traversal accepted fixture must pair root/support source ordinals 0/1 with matching witness ordinals",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0_unsupported_source_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_projection_stage0_single_table_result SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::UnsupportedTraversalSource",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0_with_sources_result sources witness_body_module_fingerprint 0 0 1 SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessStatus::PrivateCacheRegionFreshWitnessCandidateAccepted",
        "Stage0SourceRejected e",
    ],
    "actual traversal unsupported fixture must keep unsupported source distinct even when a matching witness exists",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_request_evidence_gate_result"),
    [
        'let sources %SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable field::get bundle "sources"',
        'let witnesses %SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessTable field::get bundle "witnesses"',
        "selfhost_memo_call_backend_private_cache_region_proof_table_from_sources_result &sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_from_table_result &table",
        "selfhost_memo_call_backend_private_cache_region_proof_table_free table",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_request_evidence_gate_result module root fuel body_module_fingerprint candidate witnesses",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_free witnesses",
    ],
    "actual traversal bundle gate must close source owner, extract candidate through existing checker, and pass witness owner only to the existing request-evidence bridge",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_request_evidence_gate_result")),
    /selfhost_memo_call_backend_private_cache_resource_proof_table_push|resource_proof_table_to_request_evidence_result|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|resource_graph_input_push|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend|neplobj|neplproof/,
    "actual traversal bundle gate must not synthesize lower proof records, GraphInput, backend bytes, effect masking, or artifact keys",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalBundleStage0Summary"),
    [
        "accepted_request_count %i32",
        "accepted_proof_count %i32",
        "body_fingerprint_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "missing_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "rejected_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "unsupported_source_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
    ],
    "actual traversal bundle summary must expose only counts and typed Result payloads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0_run_summary_result 77 77",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
        "body_fingerprint_mismatch_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0_run_i32_result 78 77",
        "missing_witness_rejected",
        "PrivateCacheRegionFreshWitnessMissing",
        "rejected_witness_rejected",
        "PrivateCacheRegionFreshWitnessRejected",
        "unsupported_source_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0_unsupported_source_result 77",
        "accepted.request_count",
        "accepted.proven_request_count",
    ],
    "actual traversal bundle stage0 must cover accepted, body fingerprint mismatch, missing witness, rejected witness, and unsupported source paths",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_with_owners_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collect_from_walker_input_result &input &observations",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0_with_sources_result sources witness_body_module_fingerprint graph_index root_operation_ordinal support_operation_ordinal status",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "Stage0SourceRejected e",
    ],
    "collector-owned traversal bundle helper must collect source table, close collector inputs, and delegate witness/source cleanup to the existing bundle helper",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_from_input_result"),
    [
        "Result::Ok input:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collector_stage0_empty_observations_result",
        "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_with_owners_result input observations witness_body_module_fingerprint graph_index root_operation_ordinal support_operation_ordinal status",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "Stage0SourceRejected e",
        "Result::Err e:",
        "ActualWalkerTraversalInputRejected e",
    ],
    "collector-owned traversal bundle input wrapper must close input on observation-table failure and map input fixture failures through Stage0SourceRejected",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_accepted_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_stage0_closed_place_edge_input_result",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::PrivateCacheStorage",
        "SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::CloneOutOwnedValue",
        "witness_body_module_fingerprint 0 0 1 status",
    ],
    "collector-owned accepted fixture must build source through collector and pair source ordinals 0/1 with matching witness ordinals",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_unsupported_source_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_stage0_unsupported_input_result",
        "witness_body_module_fingerprint 0 0 1",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
    ],
    "collector-owned unsupported fixture must preserve unsupported collector output even with a matching witness",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_observation_source_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_input_new",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collector_stage0_single_observation_result SelfhostMemoCallBackendPrivateCacheObservationKind::CacheHitObserved",
        "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_with_owners_result input observations witness_body_module_fingerprint 0 0 1",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "Stage0SourceRejected e",
        "ActualWalkerTraversalInputRejected e",
    ],
    "collector-owned observation fixture must route cache-hit observation through collector output and fail closed",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_with_owners_result")),
    /selfhost_memo_call_backend_private_cache_resource_proof_table_push|resource_proof_table_to_request_evidence_result|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|resource_graph_input_push|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend|neplobj|neplproof/,
    "collector-owned traversal bundle helper must not synthesize lower proof records, GraphInput, backend bytes, effect masking, or artifact keys",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheCollectorOwnedTraversalBundleStage0Summary"),
    [
        "accepted_request_count %i32",
        "accepted_proof_count %i32",
        "body_fingerprint_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "missing_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "unsupported_source_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "observation_source_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
    ],
    "collector-owned traversal bundle summary must expose only counts and typed Result payloads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_stage0"),
    [
        "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_stage0_run_summary_result 77 77",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
        "body_fingerprint_mismatch_rejected",
        "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_stage0_run_i32_result 78 77",
        "missing_witness_rejected",
        "PrivateCacheRegionFreshWitnessMissing",
        "unsupported_source_rejected",
        "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_unsupported_source_result 77",
        "observation_source_rejected",
        "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_observation_source_result 77",
        "accepted.request_count",
        "accepted.proven_request_count",
    ],
    "collector-owned traversal bundle stage0 must cover accepted, body fingerprint mismatch, missing witness, unsupported collector source, and observation collector source paths",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_producer_owned_unavailable_traversal_bundle_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_traversal_sources_from_hir_root_result module root fuel body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0_with_sources_result sources witness_body_module_fingerprint 0 root_operation_ordinal support_operation_ordinal status",
        "Stage0SourceRejected e",
    ],
    "producer-owned unavailable traversal bundle helper must use HIR-root producer source table and delegate source/witness cleanup to the existing bundle helper",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_producer_owned_unavailable_traversal_bundle_from_hir_root_result")),
    /resource_walker_stage0_closed_place_edge_input_result|actual_walker_traversal_source_projection_stage0_closed_clone_table_result|collector_owned_traversal_bundle_accepted_bundle_result|selfhost_memo_call_backend_private_cache_resource_proof_table_push|resource_proof_table_to_request_evidence_result|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|resource_graph_input_push|GraphInput|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend|neplobj|neplproof/,
    "producer-owned unavailable traversal bundle helper must not use accepted fixtures, collector fixtures, lower proof synthesis, GraphInput, backend bytes, effect masking, or artifact keys",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheProducerOwnedUnavailableTraversalBundleStage0Summary"),
    [
        "well_formed_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "missing_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "rejected_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
    ],
    "producer-owned unavailable traversal bundle summary must expose only typed rejection payloads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_producer_owned_unavailable_traversal_bundle_stage0"),
    [
        "well_formed_witness_rejected",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
        "missing_witness_rejected",
        "PrivateCacheRegionFreshWitnessMissing",
        "rejected_witness_rejected",
        "PrivateCacheRegionFreshWitnessRejected",
        "ProducerOwnedUnavailableTraversalBundleStage0Summary well_formed_witness_rejected missing_witness_rejected rejected_witness_rejected",
    ],
    "producer-owned unavailable traversal bundle stage0 must keep all representative witness statuses on rejection payloads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_from_split_events_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_event_split_result events",
        "Result::Ok output:",
        'field::get output "walker_input"',
        'field::get output "observations"',
        "selfhost_memo_call_backend_private_cache_collector_owned_traversal_bundle_with_owners_result input observations witness_body_module_fingerprint graph_index root_operation_ordinal support_operation_ordinal status",
        "Result::Err e:",
        "Stage0SourceRejected SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind::NormalizerRejected e",
    ],
    "operation-classified split helper must split unified events, transfer walker/observation owners to collector-owned bundle, and map split failures as NormalizerRejected",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_with_operations_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_events_from_hir_root_result module root fuel operation_body_module_fingerprint &operations",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_free operations",
        "Result::Ok events:",
        "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_from_split_events_result events witness_body_module_fingerprint graph_index root_operation_ordinal support_operation_ordinal status",
        "Result::Err e:",
        "Stage0SourceRejected e",
    ],
    "operation-classified operation owner helper must classify through HIR-root request authority, close the operation owner, and keep classifier errors distinct from split normalizer errors",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_from_operation_table_result"),
    [
        "Result::Ok operations:",
        "selfhost_memo_call_backend_private_cache_proof_gate_stage0_build_memoized_module function_ty span def_id",
        "Result::Ok module:",
        "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_with_operations_result &module root 8 operation_body_module_fingerprint operations witness_body_module_fingerprint graph_index root_operation_ordinal support_operation_ordinal status",
        "selfhost_hir_module_free module",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_free operations",
        "Stage0FixtureAllocFailed e",
        "Result::Err e:",
        "Stage0SourceRejected e",
    ],
    "operation-classified operation-table wrapper must free operations on module fixture failure and map operation table build failures as Stage0SourceRejected",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_accepted_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_stage0_closed_clone_table_result",
        "witness_body_module_fingerprint 0 0 1 status",
    ],
    "operation-classified accepted bundle must pair closed private-cache storage and clone-out operation ordinals 0/1 with matching witness ordinals",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_escape_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_stage0_escape_table_result",
        "witness_body_module_fingerprint 0 0 1",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
    ],
    "operation-classified escape bundle must preserve escaping source rejection even with a matching witness",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_observation_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_stage0_single_table_result SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheHitObservation",
        "witness_body_module_fingerprint 0 0 1",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
    ],
    "operation-classified observation bundle must route cache-hit observation through split/collector rejection",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_unsupported_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_stage0_single_table_result SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::UnknownResourceOperation",
        "witness_body_module_fingerprint 0 0 1",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
    ],
    "operation-classified unsupported bundle must keep unknown operations as unsupported traversal source failures",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_with_operations_result")),
    /actual_walker_event_gate_from_hir_root_result|resource_graph_gate_from_hir_root_result|observation_ban_gate_from_hir_root_result|actual_walker_traversal_source_projection_stage0_closed_clone_table_result|collector_owned_traversal_bundle_accepted_bundle_result|selfhost_memo_call_backend_private_cache_resource_proof_table_push|resource_proof_table_to_request_evidence_result|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|resource_graph_input_push|GraphInput|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend|neplobj|neplproof/,
    "operation-classified bundle helper must not bypass through lower gates, accepted source fixtures, lower proof synthesis, GraphInput, backend bytes, effect masking, or artifact keys",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheOperationClassifiedTraversalBundleStage0Summary"),
    [
        "accepted_request_count %i32",
        "accepted_proof_count %i32",
        "body_fingerprint_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "missing_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "may_escape_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
    ],
    "operation-classified traversal bundle summary must expose only counts and typed Result payloads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_stage0"),
    [
        "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_stage0_run_summary_result 77 77 77",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
        "body_fingerprint_mismatch_rejected",
        "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_stage0_run_i32_result 78 77 77",
        "missing_witness_rejected",
        "PrivateCacheRegionFreshWitnessMissing",
        "may_escape_rejected",
        "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_escape_result 77",
        "observation_rejected",
        "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_observation_result 77",
        "unsupported_rejected",
        "selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_unsupported_result 77",
        "accepted.request_count",
        "accepted.proven_request_count",
    ],
    "operation-classified traversal bundle stage0 must cover accepted, body fingerprint mismatch, missing witness, escape, observation, and unsupported paths",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_operation_classified_traversal_bundle_(?!stage0\b)/m,
    "operation-classified traversal bundle internals must stay module-private; only the typed stage0 summary function may be public",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_result")),
    /PrivateCacheStoragePlace|CloneOutOwnedValueEdge|PrivateCacheRegionFreshWitnessCandidateAccepted/,
    "HIR-root operation producer path must not emit accepted source or witness while actual Resource IR traversal is still unconnected",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessStage0Summary"),
    [
        "accepted_result %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "empty_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "missing_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "key_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "graph_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "root_ordinal_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "support_ordinal_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "ordinal_duplicate_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "duplicate_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "invalid_graph_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "invalid_ordinal_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "rejected_status_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "unavailable_status_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
    ],
    "fresh witness stage0 summary must expose only typed Result payloads for accepted and fail-closed representative paths",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_stage0"),
    [
        "accepted_result",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
        "empty_witness_rejected",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_stage0_empty_table_result",
        "missing_witness_rejected",
        "PrivateCacheRegionFreshWitnessMissing",
        "key_mismatch_rejected",
        "78 0 0 1",
        "graph_mismatch_rejected",
        "77 1 0 1",
        "root_ordinal_mismatch_rejected",
        "77 0 2 1",
        "support_ordinal_mismatch_rejected",
        "77 0 0 2",
        "ordinal_duplicate_rejected",
        "77 0 0 0",
        "duplicate_witness_rejected",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_stage0_duplicate_table_result",
        "placeholder_rejected",
        "0 0 0 1",
        "invalid_graph_rejected",
        "77 -1 0 1",
        "invalid_ordinal_rejected",
        "77 0 -1 1",
        "rejected_status_rejected",
        "PrivateCacheRegionFreshWitnessRejected",
        "unavailable_status_rejected",
        "PrivateCacheRegionFreshWitnessUnavailable",
    ],
    "fresh witness stage0 must cover accepted, empty, missing, key/graph mismatch, ordinal mismatch, duplicate, invalid, rejected, and unavailable witness paths",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_region_(?:proof_(?:input|status|record|table|fail|append|collect|fold|stage0_)|no_escape_candidate_(?:record|scan|loop|from|i32|stage0_)|fresh_witness_(?:record|table|candidate|resource|scan|loop|stage0_))/m,
    "private cache region proof, no-escape candidate, and fresh witness helpers must stay module-private and only typed smoke summaries may be public",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_region_fresh_witness_request_evidence_(?:gate|stage0_run)\b/m,
    "fresh witness request-evidence bridge internals must stay module-private; only the typed stage0 summary function may be public",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_traversal_sources_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_request_table_from_hir_root_result module root fuel",
        "Result::Ok table:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_new",
        "selfhost_memo_call_backend_request_table_len &table",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_requests_loop module &table sources0 root body_module_fingerprint 0 request_count",
        "selfhost_memo_call_backend_request_table_free table",
        "RequestCollectionFailed e",
    ],
    "actual walker operation producer bridge must build request authority internally from HIR root, create a private traversal source table, append request-derived sources, and close the request table",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_fail_with_operations"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_free operations",
        "Result::Err error",
    ],
    "actual walker operation producer bridge must close the private operation table on non-push failures",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_fail_with_traversal_sources"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "Result::Err error",
    ],
    "actual walker operation producer bridge must close the private traversal source table on source build failures",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind"),
    [
        "ActualTraversalBodyInputProducerNotConnected %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputUnavailable %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputUnsupported %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputMalformed %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
    ],
    "actual traversal body input availability error taxonomy must distinguish producer-not-connected fallback, missing, real unavailable, unsupported, and malformed body inputs before source table production",
);
assert.doesNotMatch(
    code,
    /^pub\s+enum\s+SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind\b/m,
    "actual traversal body input availability error must stay module-private until the real Resource IR body reader owns the public boundary",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_bridge_error_from_availability_error"),
    [
        "ActualTraversalBodyInputProducerNotConnected key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputUnavailable key",
        "ActualTraversalBodyInputMissing key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputMissing key",
        "ActualTraversalBodyInputUnavailable key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputUnavailable key",
        "ActualTraversalBodyInputUnsupported key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputUnsupported key",
        "ActualTraversalBodyInputMalformed scanner_error",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputMalformed scanner_error",
    ],
    "actual traversal body adapter must map private availability errors to public bridge errors without collapsing missing, unavailable, unsupported, or malformed cases",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_availability_from_request_result"),
    [
        "SelfhostHirModule",
        "SelfhostMemoCallBackendRequestTableEntry",
        "SelfhostHirExprId",
        "body_module_fingerprint",
        "ActualTraversalBodyInputProducerNotConnected key",
    ],
    "actual traversal body input availability boundary must exist on the production request path and use producer-not-connected fallback until the real Resource IR body reader is connected",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_unavailable_source_record"),
    [
        "selfhost_memo_call_backend_private_cache_resource_place_id_new 0",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_record_new key graph_id 0 place place",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::ResourceIrTraversalUnavailable",
    ],
    "actual traversal body adapter unavailable helper must encode only the producer-not-connected fallback as a typed unavailable traversal source",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_source_from_request_result"),
    [
        "SelfhostHirModule",
        "SelfhostMemoCallBackendRequestTableEntry",
        "SelfhostHirExprId",
        "body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_availability_from_request_result module entry root_expr_id body_module_fingerprint key graph_id",
        "Result::Ok output:",
        "field::get output \"walker_input\"",
        "field::get output \"observations\"",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputUnsupported key",
        "ActualTraversalBodyInputProducerNotConnected fallback_key",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_unavailable_source_record fallback_key graph_id",
    ],
    "actual traversal body adapter single-source compatibility helper must pass through the availability boundary and keep accepted real input out of the singular record path",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_unavailable_sources_from_request_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_new",
        "let record",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_unavailable_source_record key graph_id",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_push sources0 record",
    ],
    "actual traversal body adapter unavailable fallback must return a request-local source table owner",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_result"),
    [
        "SelfhostHirModule",
        "SelfhostMemoCallBackendRequestTableEntry",
        "SelfhostHirExprId",
        "body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_availability_from_request_result module entry root_expr_id body_module_fingerprint key graph_id",
        "Result::Ok output:",
        "field::get output \"walker_input\"",
        "field::get output \"observations\"",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_input_owners_result input observations",
        "ActualTraversalBodyInputProducerNotConnected fallback_key",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_unavailable_sources_from_request_result fallback_key graph_id",
        "ActualTraversalBodyInputMissing missing_key",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_bridge_error_from_availability_error SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind::ActualTraversalBodyInputMissing missing_key",
        "ActualTraversalBodyInputUnavailable unavailable_key",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_bridge_error_from_availability_error SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind::ActualTraversalBodyInputUnavailable unavailable_key",
        "ActualTraversalBodyInputUnsupported unsupported_key",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_bridge_error_from_availability_error SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind::ActualTraversalBodyInputUnsupported unsupported_key",
        "ActualTraversalBodyInputMalformed scanner_error",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_bridge_error_from_availability_error SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind::ActualTraversalBodyInputMalformed scanner_error",
    ],
    "actual traversal body adapter request boundary must pass through typed availability, consume available owners, keep unavailable fallback explicit, and reject missing/unsupported/malformed body inputs as typed bridge errors",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_input_owners_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collect_from_walker_input_result &input &observations",
        "Result::Ok sources:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "Result::Ok sources",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "Result::Err e",
    ],
    "actual traversal body adapter must consume typed body input owners through the existing collector and close input/observation owners on success and failure",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_availability_result"),
    [
        "Result::Ok output:",
        "field::get output \"walker_input\"",
        "field::get output \"observations\"",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_input_owners_result input observations",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_bridge_error_from_availability_error e",
    ],
    "actual traversal body adapter availability helper must consume split-output owners only on Ok and map availability errors without synthesizing sources",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_source_count_with_owners_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_input_owners_result input observations",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_len &sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
    ],
    "actual traversal body adapter stage0 count helper must close the private source table after reading the count",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_source_from_request_result")),
    /PrivateCacheNoEscapeProven|PrivateCacheStoragePlace|CloneOutOwnedValueEdge|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual traversal body adapter stage0 must not synthesize accepted sources, fresh witnesses, lower proof tables, backend bytes, effect masks, or artifact records",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_result")),
    /PrivateCacheNoEscapeProven|PrivateCacheStoragePlace|CloneOutOwnedValueEdge|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual traversal body adapter request-local source table helper must not synthesize accepted sources, fresh witnesses, lower proof tables, backend bytes, effect masks, or artifact records",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_input_owners_result")),
    /PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual traversal body input adapter must only produce traversal source tables and must not synthesize proof, fresh witness, backend, effect, or artifact records",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_availability_result")),
    /PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual traversal body availability helper must only route available owners into the body adapter and must not synthesize proof, fresh witness, backend, effect, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_sources_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_sources_loop sources &request_sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free request_sources",
        "result",
    ],
    "actual walker operation producer bridge must close request-local source table owners after merge success or failure",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_sources_result")),
    /PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual walker operation producer source table merge must not synthesize proof, fresh witness, backend, effect, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyAdapterInputStage0Summary"),
    [
        "unavailable_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "accepted_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "observation_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "unsupported_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "merged_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "availability_available_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "availability_missing_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "availability_unavailable_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "availability_unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "availability_malformed_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
    ],
    "actual traversal body input adapter stage0 summary must expose only source counts, availability rejections, and typed Result payloads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_stage0"),
    [
        "unavailable_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_unavailable_source_count_result",
        "accepted_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_source_count_from_input_result selfhost_memo_call_backend_private_cache_resource_walker_stage0_closed_place_edge_input_result",
        "observation_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_observation_source_count_from_input_result selfhost_memo_call_backend_private_cache_resource_walker_stage0_closed_place_edge_input_result",
        "unsupported_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_source_count_from_input_result selfhost_memo_call_backend_private_cache_resource_walker_stage0_unsupported_input_result",
        "merged_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_merged_source_count_from_input_result selfhost_memo_call_backend_private_cache_resource_walker_stage0_closed_place_edge_input_result",
        "availability_available_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_availability_available_source_count_from_input_result selfhost_memo_call_backend_private_cache_resource_walker_stage0_closed_place_edge_input_result",
        "availability_missing_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_availability_missing_source_count_result",
        "availability_unavailable_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_availability_unavailable_source_count_result",
        "availability_unsupported_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_availability_unsupported_source_count_result",
        "availability_malformed_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_availability_malformed_source_count_from_input_result selfhost_memo_call_backend_private_cache_resource_walker_stage0_placeholder_input_result",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_source_count_from_input_result selfhost_memo_call_backend_private_cache_resource_walker_stage0_placeholder_input_result",
    ],
    "actual traversal body input adapter stage0 must cover unavailable fallback, accepted-shaped, observation-shaped, unsupported, merged, typed availability, and malformed placeholder body inputs",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_(?!input_stage0\b)/m,
    "actual traversal body adapter helpers must stay module-private; only the typed stage0 summary function may be public",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_gate_recheck_entry_result module entry",
        "selfhost_memo_call_backend_private_cache_proof_key_from_entry_result entry root_expr_id body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_resource_graph_id_new graph_index",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_result module entry root_expr_id body_module_fingerprint key graph_id",
        "Result::Ok request_sources:",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_sources_result sources request_sources",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_fail_with_traversal_sources sources e",
        "ProofKeyRejected e",
        "RequestRecheckRejected e",
    ],
    "actual walker operation producer bridge must recheck each request entry, derive the proof key, and merge request-local source table owners from the actual traversal body adapter",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_requests_loop"),
    [
        "selfhost_memo_call_backend_request_table_get_entry table idx",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_result module entry root_expr_id body_module_fingerprint sources idx",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_requests_loop module table next_sources root_expr_id body_module_fingerprint add idx 1 n",
        "RequestEntryMissing idx",
    ],
    "actual walker operation producer bridge request loop must read request entries, thread the traversal source owner, and fail closed on missing entries",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_result")),
    /PrivateCacheNoEscapeProven|PrivateCacheStoragePlace|CloneOutOwnedValueEdge|ResourceIrTraversalUnavailable|resource_place_id_new|resource_graph_input_push|proof_table_push|Wasm|LLVM/,
    "actual walker operation producer bridge must not synthesize accepted proof, accepted private-cache operation records, direct unavailable source records, direct GraphInput, proof table records, or backend bytes",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_operations_from_sources_loop"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_get sources idx",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_to_operation_record source",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_push operations record",
        "ActualWalkerTraversalSourceReadFailed idx",
    ],
    "actual walker operation producer bridge must project traversal source records into operation records through a typed source-to-operation boundary",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_operations_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_traversal_sources_from_hir_root_result module root fuel body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_operations_from_sources_result &sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
    ],
    "actual walker operation producer bridge must build traversal sources first, project them to operation records, and close the source table",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_operations_from_hir_root_result module root fuel body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_from_hir_root_result module root fuel body_module_fingerprint &operations",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_free operations",
    ],
    "actual walker operation producer bridge must pass its producer-owned operation table through the classifier and then close the table",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_from_hir_root_result"),
    /actual_walker_event_gate_from_hir_root_result|resource_graph_input_scanner_output_result|resource_graph_gate_from_hir_root_result|observation_ban_gate_from_hir_root_result/,
    "actual walker operation producer bridge must not bypass the operation classifier or unified normalizer by directly calling lower gates",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_(?:fail|append|traversal|operations|from|stage0_run)/m,
    "actual walker operation producer bridge internals must stay module-private and must not expose private traversal source or operation table construction APIs",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_stage0"),
    [
        "unsupported_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_stage0_run_i32_result 77",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_stage0_run_i32_result 0",
    ],
    "actual walker operation producer bridge stage0 must cover unsupported traversal and placeholder fingerprint rejection without exposing private operation tables",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_append_observation_result"),
    [
        "selfhost_memo_call_backend_private_cache_observation_ban_record_matches_key record key",
        "Option::Some reason:",
        "selfhost_memo_call_backend_private_cache_resource_walker_unsupported_event_record_new record.key record.graph_id record.operation_ordinal reason",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_push_unsupported input unsupported",
        "Option::None:",
        "Result::Ok input",
    ],
    "observation producer bridge must append detected observations as unsupported events and leave no-observation records neutral",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_append_observation_result"),
    /PrivateCacheNoEscapeProven|PrivateCacheStorage|CloneOutOwnedValue/,
    "observation producer bridge must not synthesize accepted private-cache proof, place, or clone-out edge",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_observation_ban_gate_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_observation_ban_input_from_hir_root_result module root fuel body_module_fingerprint observations",
        "selfhost_memo_call_backend_private_cache_resource_graph_input_scanner_output_result input",
        "Result::Ok graph:",
        "selfhost_memo_call_backend_private_cache_resource_graph_gate_from_hir_root_result module root fuel body_module_fingerprint &graph",
        "selfhost_memo_call_backend_private_cache_resource_graph_input_free graph",
        "OutputGraphGateRejected e",
        "ScannerOutputRejected e",
    ],
    "observation ban gate must rebuild request authority from HIR root, pass private walker input through the scanner, close GraphInput, and wrap graph/scanner failures",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_observation_ban_(?:input_from_hir_root_result|gate_from_hir_root_result|append_request_result|append_requests_loop|append_records_loop|table_new|table_push|table_free)\b/m,
    "observation ban internals must stay module-private and must not expose HIR-root gate or private observation table construction as public accepted-path APIs",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_observation_ban_stage0"),
    [
        "cache_observation_rejected",
        "CacheHitObserved",
        "function_identity_observation_rejected",
        "FunctionEqualityObserved",
        "raw_identity_observation_rejected",
        "RawIdentityObserved",
    ],
    "observation ban stage0 must cover cache observation, function identity observation, and raw identity observation",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_observation_ban_unknown_result_eq"),
    /_:/,
    "observation ban unknown result helper must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind"),
    [
        "WalkerBodyTableAllocFailed %StdErrorKind",
        "WalkerPlaceEventTableAllocFailed %StdErrorKind",
        "WalkerEdgeEventTableAllocFailed %StdErrorKind",
        "WalkerUnsupportedEventTableAllocFailed %StdErrorKind",
        "WalkerBodyPushFailed %StdErrorKind",
        "WalkerPlaceEventPushFailed %StdErrorKind",
        "WalkerEdgeEventPushFailed %StdErrorKind",
        "WalkerUnsupportedEventPushFailed %StdErrorKind",
        "WalkerBodyReadFailed %i32",
        "WalkerPlaceEventReadFailed %i32",
        "WalkerEdgeEventReadFailed %i32",
        "WalkerUnsupportedEventReadFailed %i32",
        "WalkerBodyDuplicate %SelfhostMemoCallBackendPrivateCacheProofKey",
        "WalkerOperationDuplicate %i32",
        "WalkerBodyMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "WalkerEventForNonClosedGraph %i32",
        "BodyModuleFingerprintPlaceholder",
        "GraphIdInvalid %i32",
        "PlaceIdInvalid %i32",
        "OperationOrdinalInvalid %i32",
        "OutputGraphInputRejected %SelfhostMemoCallBackendPrivateCacheResourceGraphProducerErrorKind",
        "OutputGraphGateRejected %SelfhostMemoCallBackendPrivateCacheResourceGraphProducerErrorKind",
        "Stage0FixtureAllocFailed %StdErrorKind",
    ],
    "Resource walker scanner error taxonomy must distinguish allocation, push, read, duplicate operation, missing body, non-closed graph event, invalid ids, lower graph input, lower graph gate, and fixture failures",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_input_scanner_error_code"),
    /_:/,
    "Resource walker scanner error code helper must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_validate_input_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_validate_all_bodies_loop input 0",
        "selfhost_memo_call_backend_private_cache_resource_walker_validate_all_places_loop input 0",
        "selfhost_memo_call_backend_private_cache_resource_walker_validate_all_edges_loop input 0",
        "selfhost_memo_call_backend_private_cache_resource_walker_validate_all_unsupported_loop input 0",
    ],
    "Resource walker scanner must validate body, place, edge, and unsupported event tables before producing graph input",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_validate_all_edges_loop"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_seen_place_operation_loop input edge.key edge.graph_id edge.operation_ordinal",
        "WalkerOperationDuplicate edge.operation_ordinal",
        "selfhost_memo_call_backend_private_cache_resource_walker_seen_edge_operation_loop input edge.key edge.graph_id edge.operation_ordinal 0 idx",
        "WalkerOperationDuplicate edge.operation_ordinal",
        "selfhost_memo_call_backend_private_cache_resource_walker_event_closed_body_result input edge.key edge.graph_id",
    ],
    "Resource walker edge validation must reject operation ordinals already used by place events, reject duplicate edge ordinals, and require a closed body event",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_validate_all_unsupported_loop"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_seen_place_operation_loop input event.key event.graph_id event.operation_ordinal",
        "WalkerOperationDuplicate event.operation_ordinal",
        "selfhost_memo_call_backend_private_cache_resource_walker_seen_edge_operation_loop input event.key event.graph_id event.operation_ordinal",
        "WalkerOperationDuplicate event.operation_ordinal",
        "selfhost_memo_call_backend_private_cache_resource_walker_seen_unsupported_operation_loop input event.key event.graph_id event.operation_ordinal 0 idx",
        "WalkerOperationDuplicate event.operation_ordinal",
        "selfhost_memo_call_backend_private_cache_resource_walker_event_closed_body_result input event.key event.graph_id",
    ],
    "Resource walker unsupported validation must reject cross-kind ordinal reuse, duplicate unsupported ordinals, and missing or non-closed body events",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_body_to_graph_body_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_body_has_unsupported_result input body.key body.graph_id",
        "has_unsupported",
        "SelfhostMemoCallBackendPrivateCacheResourceGraphCompleteness::TraversalUnsupported",
        "body.completeness",
    ],
    "Resource walker scanner must turn any unsupported event for a body into TraversalUnsupported instead of silently accepting the original body completeness",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_graph_input_scanner_output_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_validate_input_result &input",
        "selfhost_memo_call_backend_private_cache_resource_graph_input_new",
        "selfhost_memo_call_backend_private_cache_resource_walker_append_bodies_loop output0 &input 0",
        "selfhost_memo_call_backend_private_cache_resource_walker_append_places_loop output1 &input 0",
        "selfhost_memo_call_backend_private_cache_resource_walker_append_edges_loop output2 &input 0",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
    ],
    "Resource walker scanner output must validate first, allocate GraphInput, append bodies/places/edges in order, and always close the walker input owner",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_append_places_loop"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_body_has_unsupported_result input place.key place.graph_id",
        "has_unsupported",
        "selfhost_memo_call_backend_private_cache_resource_walker_append_places_loop output input add idx 1",
        "selfhost_memo_call_backend_private_cache_resource_graph_input_push_place output",
    ],
    "Resource walker scanner must skip place events for bodies already marked unsupported",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_append_edges_loop"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_body_has_unsupported_result input edge.key edge.graph_id",
        "has_unsupported",
        "selfhost_memo_call_backend_private_cache_resource_walker_append_edges_loop output input add idx 1",
        "selfhost_memo_call_backend_private_cache_resource_graph_input_push_edge output",
    ],
    "Resource walker scanner must skip edge events for bodies already marked unsupported",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_input_scanner_stage0"),
    [
        "accepted_result",
        "PrivateCacheStorage 0 SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::CloneOutOwnedValue",
        "may_escape_rejected",
        "ReturnCacheReference 0 SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::BorrowView",
        "missing_rejected",
        "ResourceGraphMissing",
        "unsupported_rejected",
        "selfhost_memo_call_backend_private_cache_resource_walker_stage0_unsupported_input_result",
        "duplicate_ordinal_rejected",
        "selfhost_memo_call_backend_private_cache_resource_walker_stage0_duplicate_ordinal_input_result",
        "missing_body_event_rejected",
        "selfhost_memo_call_backend_private_cache_resource_walker_stage0_missing_body_event_input_result",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_resource_walker_stage0_placeholder_input_result",
    ],
    "Resource walker scanner stage0 must cover accepted, may-escape, missing, unsupported, duplicate ordinal, missing body event, and placeholder fingerprint paths",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheResourceWalkerProducerBridgeErrorKind"),
    [
        "RequestCollectionFailed %SelfhostMemoCallBackendRequestCollectorErrorKind",
        "RequestEntryMissing %i32",
        "RequestRecheckRejected %SelfhostMemoCallBackendPrivateCacheProofGateErrorKind",
        "ProofKeyRejected %SelfhostMemoCallBackendPrivateCacheProofGateErrorKind",
        "WalkerInputBuildRejected %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
        "ScannerOutputRejected %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
        "OutputGraphGateRejected %SelfhostMemoCallBackendPrivateCacheResourceGraphProducerErrorKind",
        "Stage0FixtureAllocFailed %StdErrorKind",
    ],
    "Resource walker producer bridge error taxonomy must distinguish request collection, request entry, request recheck, proof key, private walker input, scanner output, graph gate, and fixture failures",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_error_code"),
    /_:/,
    "Resource walker producer bridge error code helper must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_input_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_request_table_from_hir_root_result module root fuel",
        "Result::Ok table:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_new",
        "selfhost_memo_call_backend_request_table_len &table",
        "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_append_requests_loop module &table input0 root body_module_fingerprint 0 request_count",
        "selfhost_memo_call_backend_request_table_free table",
        "RequestCollectionFailed e",
    ],
    "Resource walker producer bridge must build its request table internally from HIR root, create a private walker input owner, append request-derived events, and close the request table",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_append_request_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_gate_recheck_entry_result module entry",
        "selfhost_memo_call_backend_private_cache_proof_key_from_entry_result entry root_expr_id body_module_fingerprint",
        "SelfhostMemoCallBackendPrivateCacheResourceGraphCompleteness::ClosedForPrivateCacheBoundary",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_push_body input body",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::UnknownResourceOperation",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_push_unsupported input1 unsupported",
    ],
    "Resource walker producer bridge must recheck each request entry, derive the proof key from the request, emit a closed body header, and represent the unimplemented actual traversal as a typed unsupported event",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_append_request_result"),
    /PrivateCacheNoEscapeProven|PrivateCacheStorage|CloneOutOwnedValue/,
    "Resource walker producer bridge must not synthesize an accepted no-escape proof or accepted private-cache place/edge while actual Resource IR traversal is still unsupported",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_input_from_hir_root_result module root fuel body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_resource_graph_input_scanner_output_result input",
        "Result::Ok graph:",
        "selfhost_memo_call_backend_private_cache_resource_graph_gate_from_hir_root_result module root fuel body_module_fingerprint &graph",
        "selfhost_memo_call_backend_private_cache_resource_graph_input_free graph",
        "OutputGraphGateRejected e",
        "ScannerOutputRejected e",
    ],
    "Resource walker producer bridge must pass producer-owned private walker input through the existing scanner before invoking the graph gate, and must close GraphInput on graph gate success or failure",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_(?:input_from_hir_root_result|from_hir_root_result|append_request_result|append_requests_loop)\b/m,
    "Resource walker producer bridge internals must stay module-private and must not expose HIR-root bridge or private event input construction as public accepted-path APIs",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_stage0"),
    [
        "unsupported_rejected",
        "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_stage0_run_i32_result 77",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_resource_walker_producer_bridge_stage0_run_i32_result 0",
    ],
    "Resource walker producer bridge stage0 must cover unsupported actual traversal and placeholder fingerprint rejection without exposing private walker input",
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
