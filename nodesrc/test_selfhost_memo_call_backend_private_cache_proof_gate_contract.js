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

function countOccurrences(text, snippet) {
    let count = 0;
    let offset = 0;
    while (true) {
        const found = text.indexOf(snippet, offset);
        if (found === -1) {
            return count;
        }
        count += 1;
        offset = found + snippet.length;
    }
}

function enumVariantNames(src, enumName) {
    return stripDocComments(topLevelBlock(src, "enum", enumName))
        .split("\n")
        .slice(1)
        .map((line) => line.trim().match(/^([A-Za-z_][A-Za-z0-9_]*)\b/))
        .filter(Boolean)
        .map((match) => match[1]);
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
        source.includes("Context-bound reader traversal bundle stage0") &&
        source.includes("Context-bound coverage witness bundle stage0") &&
        source.includes("Actual no-escape coverage authority stage0") &&
        source.includes("Operation-classified no-escape coverage authority stage0") &&
        source.includes("Backend readiness stage0") &&
        source.includes("Private-effect readiness handoff API") &&
        source.includes("Actual traversal private-effect readiness projection stage0") &&
        source.includes("Actual traversal private-effect coverage handoff API") &&
        source.includes("Resource lowering fresh witness authority bundle stage0") &&
        source.includes("public accepted path を追加せず") &&
        source.includes("stable artifact sidecar index"),
    "docs must state that caller proof tables are not direct authority, success is not executable backend output, table writes are private in phase 1, Resource observation uses the private writer, walker input scanner only normalizes typed events, observation-ban stage0, unified stream normalizer, HIR-root unified event producer bridge, operation classifier, traversal source, operation producer bridge, region proof, no-escape candidate checker, fresh witness bridge, request-evidence bridge, actual no-escape coverage authority, operation-classified no-escape coverage authority, backend readiness stage, and actual traversal private-effect readiness projection are present, no public accepted path is added, and index optimization is later contract-preserving work",
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
    /memo_trait_operation_private_effect_(?:no_escape_gate|resource_no_escape_producer)/,
    "backend private-cache proof gate must not call or import checker-layer private effect proof producers",
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
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheBackendReadinessUpstreamPrivateEffect(?:Status|Evidence)\b/,
    "backend readiness upstream private-effect status and evidence must stay private to this backend module",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheBackendReadinessUpstreamPrivateEffectStatus|SelfhostMemoCallBackendPrivateCacheBackendReadinessUpstreamPrivateEffectEvidence)\b/m,
    "public functions must not expose backend readiness upstream private-effect status or evidence types in their signatures",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus"),
    ["Proven", "Refuted", "Missing", "Unknown"],
    "private-effect readiness handoff status must expose only the neutral public transport states",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffEvidence"),
    [
        "root_expr_id %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "status %SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus",
    ],
    "private-effect readiness handoff evidence must carry only same-body identity and neutral status",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffEvidence")),
    /Proof|MaskEvidence|SlotCoverage|GraphInput|Wasm|LLVM|neplobj|neplproof|artifact|sealed/i,
    "private-effect readiness handoff evidence must not expose checker evidence, proof, graph, backend, or artifact payloads",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffStatus"),
    [
        "EffectObservedNoEscape",
        "EffectObservedMayEscape",
        "EffectAbsentAfterCompleteTraversal",
        "ResourceGraphMissing",
        "TraversalUnsupported",
    ],
    "actual traversal private-effect coverage handoff status must keep observed, explicit absent, missing, and unsupported states distinct",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffEvidence"),
    [
        "body_root %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "effect %SelfhostEffectKind",
        "status %SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffStatus",
    ],
    "actual traversal private-effect coverage handoff evidence must carry only body identity, effect, and coverage status",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffEvidence")),
    /ActualWalkerTraversalSource|Proof|MaskEvidence|SlotCoverage|GraphInput|Wasm|LLVM|neplobj|neplproof|artifact|sealed/i,
    "actual traversal private-effect coverage handoff evidence must not expose traversal source tables, checker proof, graph, backend, or artifact payloads",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverage(?:CompleteAuthority|HandoffPair)\b/,
    "actual traversal coverage complete authority and handoff pair must stay backend-private",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageCompleteAuthority|SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffPair)\b/m,
    "public functions must not expose coverage complete authority or backend-private handoff pair types",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageCompleteAuthority"),
    [
        "request_root_expr_id %SelfhostHirExprId",
        "body_root_expr_id %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "graph_id %SelfhostMemoCallBackendPrivateCacheResourceGraphId",
    ],
    "coverage complete authority must bind request root, body root, body fingerprint, and graph id before source table can produce explicit absence",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerStage0Summary"),
    [
        "complete_absence_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
        "reader_context_complete_absence_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
        "reader_context_private_cache_effect_unsupported_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
        "private_cache_effect_unsupported_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
        "private_state_effect_unsupported_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
        "may_escape_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
        "mixed_absence_escape_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
        "empty_source_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
        "fingerprint_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
        "identity_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
        "graph_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind",
    ],
    "coverage handoff producer summary must distinguish fixture absence, production reader-context absence, production private-effect unsupported source, mixed escape, empty source rejection, and all authority mismatch classes",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind"),
    /SourceRejected\s+%/,
    "coverage handoff producer SourceRejected must not expose the internal bridge error taxonomy as public payload",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_handoff_status_rank"),
    [
        "EffectObservedNoEscape:\n            1",
        "EffectAbsentAfterCompleteTraversal:\n            2",
        "EffectObservedMayEscape:\n            3",
        "ResourceGraphMissing:\n            4",
        "TraversalUnsupported:\n            5",
    ],
    "coverage handoff status merge priority must keep escaping / missing / unsupported stronger than complete absence",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_cache_status_from_source_kind"),
    /EffectObservedNoEscape/,
    "coverage producer must not infer PrivateCache no-escape directly from traversal source kind",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_state_status_from_source_kind"),
    /EffectObservedNoEscape/,
    "coverage producer must not infer PrivateState no-escape directly from traversal source kind",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_cache_status_from_source_kind"),
    [
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::PrivateCacheEffectOperation:",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffStatus::TraversalUnsupported",
    ],
    "PrivateCache effect operation source must remain TraversalUnsupported until fresh witness / no-escape authority is connected",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_state_status_from_source_kind"),
    [
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::PrivateStateEffectOperation:",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffStatus::TraversalUnsupported",
    ],
    "PrivateState effect operation source must remain TraversalUnsupported until no-escape authority is connected",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_handoff_pair_from_sources_result"),
    [
        "eq authority.body_module_fingerprint 0",
        "BodyModuleFingerprintPlaceholder",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_len sources",
        "eq source_count 0",
        "SourceTableEmpty",
        "EffectAbsentAfterCompleteTraversal",
    ],
    "coverage handoff producer must reject placeholder and empty source tables before explicit complete absence is emitted",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_source_record_validate_result"),
    [
        "record.key.body_module_fingerprint",
        "authority.body_module_fingerprint",
        "record.key.root_expr_id",
        "authority.request_root_expr_id",
        "record.graph_id.index",
        "authority.graph_id.index",
    ],
    "coverage source validation must bind source records to the complete authority body fingerprint, request root, and graph id",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_authority_from_reader_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_lookup_result module context resolutions",
        "Result::Ok body_root:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_authority_new context.root_expr_id body_root context.body_module_fingerprint context.graph_id",
        "Result::Err _e:",
        "SourceRejected",
    ],
    "production coverage authority must use resolver-validated body root rather than treating request root as body root",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_handoff_pair_code_from_reader_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_authority_from_reader_context_result module context resolutions",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_context_result module context resolutions",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_handoff_pair_code_from_source_result authority source_result",
    ],
    "production reader-context coverage must build authority from resolver lookup, then map the pre-witness source owner through the coverage producer",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_handoff_pair_code_from_reader_context_result")),
    /actual_traversal_bundle_source_derived_witness_result|actual_traversal_body_reader_bundle_from_request_context_result|context_bound_reader_traversal_bundle_from_context_result|actual_traversal_bundle_request_evidence_gate_result|region_fresh_witness|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "production reader-context coverage must not consume post-witness bundles or synthesize lower proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_mixed_absence_escape_table_result"),
    [
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::PrivateCacheStoragePlace",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::ReturnCacheReferencePlace",
    ],
    "coverage producer must keep a mixed absence-plus-escape runtime fixture",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_handoff_producer_stage0_summary_eq"),
    [
        "summary.reader_context_complete_absence_pair_code 33",
        "summary.reader_context_private_cache_effect_unsupported_pair_code 53",
        "summary.may_escape_pair_code 23",
        "summary.mixed_absence_escape_pair_code 23",
        "summary.fingerprint_mismatch_rejected SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind::SourceBodyIdentityMismatch",
        "summary.identity_mismatch_rejected SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind::SourceBodyIdentityMismatch",
        "summary.graph_mismatch_rejected SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffProducerErrorKind::SourceGraphIdentityMismatch",
    ],
    "coverage producer stage0 must prove production reader-context absence, pre-witness private effect mapping, mixed escape priority, and all authority mismatch runtime cases",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind"),
    [
        "CoverageRejected",
        "WitnessMissing",
        "WitnessRejected",
        "WitnessUnavailable",
        "WitnessUnsupportedSource",
        "WitnessEscapingSource",
        "WitnessAuthorityMismatch",
        "WitnessInternalRejected",
    ],
    "no-escape coverage error must distinguish coverage authority rejection from compact witness/source rejection classes",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind"),
    /SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind/,
    "no-escape coverage public error must not carry the internal region proof error enum as payload",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageStage0Summary"),
    [
        "accepted_no_escape_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "missing_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "rejected_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "unsupported_source_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "escaping_source_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "fingerprint_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "graph_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
    ],
    "no-escape coverage summary must expose only pair code and typed rejection payloads",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualTraversalFreshWitnessAuthorityBundle\b/,
    "actual traversal-owned fresh witness authority bundle must stay backend-private",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheActualTraversalFreshWitnessAuthorityBundle|SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable|SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessTable|SelfhostMemoCallBackendPrivateCacheRegionNoEscapeCandidateRecord|SelfhostMemoCallBackendPrivateCacheResourceProofTable|SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageCompleteAuthority|SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffPair)\b/m,
    "public functions must not expose no-escape coverage authority bundle, source table, witness table, candidate, proof table, coverage authority, or handoff pair types",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualTraversalFreshWitnessAuthorityBundle\b/,
    "actual traversal-owned fresh witness authority bundle must not implement Clone or Copy",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalFreshWitnessAuthorityBundle"),
    [
        "sources %SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable",
        "witnesses %SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessTable",
    ],
    "actual traversal-owned fresh witness authority bundle must own exactly source and witness tables",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_source_derived_witness_result")),
    /FreshWitnessAuthorityBundle|fresh_witness_authority_bundle|no_escape_coverage/i,
    "source-derived witness helper must not create or feed the no-escape coverage authority wrapper",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_coverage_witness_bundle_from_sources_result")),
    /FreshWitnessAuthorityBundle|fresh_witness_authority_bundle|no_escape_coverage/i,
    "context-bound source-derived coverage/witness helper must not feed the no-escape coverage authority wrapper",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_candidate_validate_result"),
    [
        "candidate.key.body_module_fingerprint",
        "authority.body_module_fingerprint",
        "candidate.key.root_expr_id",
        "authority.request_root_expr_id",
        "candidate.graph_id.index",
        "authority.graph_id.index",
    ],
    "no-escape coverage candidate validation must directly bind candidate root, fingerprint, and graph to the complete authority at the emission boundary",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_pair_from_base"),
    [
        "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffStatus::EffectObservedNoEscape",
        "base_pair.state_handoff.status",
    ],
    "no-escape coverage pair construction must update only PrivateCache slot and preserve base PrivateState status",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_handoff_pair_from_authority_bundle_result"),
    [
        'let sources %SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable field::get bundle "sources"',
        'let witnesses %SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessTable field::get bundle "witnesses"',
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_handoff_pair_from_sources_result authority &sources",
        "Result::Ok base_pair:",
        "selfhost_memo_call_backend_private_cache_region_proof_table_from_sources_result &sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_from_table_result &table",
        "selfhost_memo_call_backend_private_cache_region_proof_table_free table",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_candidate_validate_result authority candidate",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_resource_table_result candidate &witnesses",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_free witnesses",
        "selfhost_memo_call_backend_private_cache_resource_proof_table_free resource_proofs",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_pair_from_base authority base_pair",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_free witnesses",
        "CoverageRejected e",
    ],
    "no-escape coverage authority helper must compute base coverage first, reuse the same source owner for region proof, validate candidate authority, consume fresh witness, free Resource proof table, and clean up owners on coverage rejection",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_handoff_pair_from_authority_bundle_result")),
    /actual_traversal_bundle_source_derived_witness_result|context_bound_reader_traversal_bundle_from_context_result|actual_traversal_bundle_request_evidence_gate_result|region_fresh_witness_request_evidence_gate_result|resource_proof_gate_from_hir_root_result|resource_proof_table_to_request_evidence_result|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|resource_graph_input_push|GraphInput|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "no-escape coverage authority helper must not consume source-derived bundles or synthesize request evidence, GraphInput, effect mask, backend bytes, or artifact keys",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_handoff_pair_from_authority_bundle_result")),
    /region_fresh_witness_stage0_table_result|region_fresh_witness_table_from_candidate_result/,
    "no-escape coverage authority helper must not generate or regenerate witness tables",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_error_from_region_error"),
    [
        "RegionFreshWitnessMissing",
        "WitnessMissing",
        "RegionFreshWitnessRejected",
        "WitnessRejected",
        "RegionProofUnsupported",
        "WitnessUnsupportedSource",
        "RegionProofMayEscape",
        "WitnessEscapingSource",
        "RegionFreshWitnessKeyMismatch",
        "WitnessAuthorityMismatch",
        "WitnessInternalRejected",
    ],
    "no-escape coverage helper must normalize internal region proof errors into compact public error classes",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_stage0_summary_eq"),
    [
        "summary.accepted_no_escape_pair_code 13",
        "summary.missing_witness_rejected missing_expected",
        "summary.rejected_witness_rejected rejected_expected",
        "summary.unsupported_source_rejected unsupported_expected",
        "summary.escaping_source_rejected escape_expected",
        "summary.fingerprint_mismatch_rejected fingerprint_expected",
        "summary.graph_mismatch_rejected graph_expected",
    ],
    "no-escape coverage stage0 must prove accepted cache no-escape code, witness rejection, unsupported/escape rejection, and authority mismatch cases",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_stage0"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_stage0_authority 77 0",
        "accepted_no_escape_pair_code",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
        "missing_witness_rejected",
        "PrivateCacheRegionFreshWitnessMissing",
        "rejected_witness_rejected",
        "PrivateCacheRegionFreshWitnessRejected",
        "unsupported_source_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_stage0_unsupported_authority_bundle_result 77",
        "escaping_source_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_stage0_escape_authority_bundle_result 77",
        "fingerprint_authority",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_stage0_authority 78 0",
        "graph_authority",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_stage0_authority 77 1",
    ],
    "no-escape coverage stage0 must cover accepted, missing/rejected witness, unsupported/escaping source, and fingerprint/graph authority mismatch",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheOperationClassifiedNoEscapeCoverageStage0Summary"),
    [
        "accepted_no_escape_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "missing_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "rejected_witness_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "witness_authority_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "may_escape_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "fingerprint_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "graph_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
    ],
    "operation-classified no-escape coverage summary must expose only compact no-escape Result payloads",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheBodyReaderNoEscapeCoverageStage0Summary"),
    [
        "accepted_no_escape_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "hir_body_private_cache_effect_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "hir_body_fn_value_observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "hir_body_memoized_function_value_observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
        "hir_body_pure_call_unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind",
    ],
    "body-reader no-escape coverage summary must expose only compact no-escape Result payloads for accepted same-source authority and HIR source rejections",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin"),
    ["HirReaderSourceDerived", "ResourceLoweringTraversalProduced"],
    "actual traversal source output origin must distinguish source-derived HIR reader output from future Resource lowering traversal output",
);
assert.equal(
    countOccurrences(code, "SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin::ResourceLoweringTraversalProduced"),
    4,
    "ResourceLoweringTraversalProduced must appear only in the producer-output conversion, the body-reader rejection guard, the production-gate branch, and the production fresh-witness input move branch",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutput"),
    [
        "origin %SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin",
        "context %SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext",
        "coverage_authority %SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageCompleteAuthority",
        "body_root %SelfhostHirExprId",
        "sources %SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable",
    ],
    "actual traversal source output must carry origin, rechecked context, same-body coverage authority, resolver body root, and the owner-bearing source table",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutput\b/,
    "actual traversal source output owner must stay module-private",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutput\b/,
    "actual traversal source output must not implement Clone or Copy because it owns a source table",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalResourceLoweringProducerOutput"),
    [
        "context %SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext",
        "coverage_authority %SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageCompleteAuthority",
        "body_root %SelfhostHirExprId",
        "sources %SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable",
    ],
    "resource-lowering producer output must carry rechecked context, same-body coverage authority, resolver body root, and the producer-owned source table",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualTraversalResourceLoweringProducerOutput\b/,
    "resource-lowering producer output owner must stay module-private",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualTraversalResourceLoweringProducerOutput\b/,
    "resource-lowering producer output must not implement Clone or Copy because it owns a source table",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalResourceLoweringProducerAuthorityOutput"),
    [
        "context %SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext",
        "coverage_authority %SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageCompleteAuthority",
        "body_root %SelfhostHirExprId",
        "witness_bundle %SelfhostMemoCallBackendPrivateCacheActualTraversalFreshWitnessAuthorityBundle",
    ],
    "resource-lowering producer authority output must carry rechecked context, same-body coverage authority, resolver body root, and the producer-issued fresh-witness authority bundle",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualTraversalResourceLoweringProducerAuthorityOutput\b/,
    "resource-lowering producer authority output owner must stay module-private",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualTraversalResourceLoweringProducerAuthorityOutput\b/,
    "resource-lowering producer authority output must not implement Clone or Copy because it owns the witness bundle",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalProductionFreshWitnessAuthorityInput"),
    [
        "context %SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext",
        "body_root %SelfhostHirExprId",
        "sources %SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable",
    ],
    "production fresh-witness authority input must carry the rechecked context, resolver body root, and the moved source owner only",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualTraversalProductionFreshWitnessAuthorityInput\b/,
    "production fresh-witness authority input owner must stay module-private",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualTraversalProductionFreshWitnessAuthorityInput\b/,
    "production fresh-witness authority input must not implement Clone or Copy because it owns a source table",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind"),
    ["OutputRejected", "SourceDerivedHirBodyReaderRejected"],
    "production output gate error must separate no-escape output rejection from source-derived origin rejection",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputStage0Summary"),
    [
        "source_derived_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
        "source_derived_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
        "source_derived_production_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
        "source_derived_fresh_witness_input_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
        "hir_body_private_cache_effect_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
        "resource_lowering_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
        "resource_lowering_fresh_witness_input_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
        "resource_lowering_authority_bundle_witness_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
        "resource_lowering_no_escape_pair_code %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
        "resource_lowering_private_cache_effect_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
        "resource_lowering_private_cache_effect_no_escape_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualTraversalProductionOutputErrorKind",
    ],
    "production output stage0 summary must expose only source-derived count/pair smoke, resource-lowering source/input/authority counts, no-escape pair code, and typed production-gate rejection payloads",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalProductionFreshWitnessAuthorityBundleStage0Summary"),
    [
        "resource_lowering_bundle_witness_count %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "resource_lowering_private_cache_effect_bundle_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
    ],
    "production fresh-witness authority bundle stage0 summary must expose only bundle witness count and typed bundle rejection",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|fn)\s+SelfhostMemoCallBackendPrivateCacheActualTraversalProductionFreshWitnessAuthorityBundleStage0Summary\b|pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_bundle_stage0\b|pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_bundle_stage0_summary_eq\b/,
    "production fresh-witness authority bundle smoke must stay module-private until no-escape authority and request-evidence boundaries are connected",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_closed_wrapper_source_shape_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_len sources",
        "not eq n 2",
        "RegionProofUnsupported candidate.key",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_get sources 0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_root_source_shape_result candidate root_record",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_get sources 1",
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_support_source_shape_result candidate support_record",
    ],
    "fresh witness authority source-shape guard must require exactly the closed wrapper root/support source pair before witness creation",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_root_source_shape_result"),
    [
        "record.operation_ordinal candidate.root_operation_ordinal",
        "not eq record.operation_ordinal 0",
        "record.from_place.index 0",
        "record.to_place.index 0",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::PrivateCacheStoragePlace",
        "RegionProofUnsupported candidate.key",
    ],
    "fresh witness authority root guard must accept only wrapper PrivateCacheStoragePlace source at ordinal 0",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_support_source_shape_result"),
    [
        "record.operation_ordinal candidate.support_operation_ordinal",
        "not eq record.operation_ordinal 1",
        "record.from_place.index 0",
        "record.to_place.index 0",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CloneOutOwnedValueEdge",
        "RegionProofUnsupported candidate.key",
    ],
    "fresh witness authority support guard must accept only wrapper CloneOutOwnedValueEdge source at ordinal 1",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_from_sources_result"),
    [
        "selfhost_memo_call_backend_private_cache_region_proof_table_from_sources_result &sources",
        "Result::Ok table:",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_from_table_result &table",
        "selfhost_memo_call_backend_private_cache_region_proof_table_free table",
        "Result::Ok candidate:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_closed_wrapper_source_shape_result &sources candidate",
        "Result::Ok _shape:",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_from_candidate_result candidate",
        "Result::Ok witnesses:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_new sources witnesses",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
    ],
    "fresh witness authority producer from sources must derive witness from the same accepted candidate, close proof table, and close source owner on candidate/witness failure",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_from_sources_result")),
    /witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|RegionFreshWitnessStatus|actual_traversal_fresh_witness_authority_bundle_stage0_with_sources_result|region_fresh_witness_stage0_table_result|SelfhostMemoCallBackendPrivateCacheActualTraversalBundle|actual_traversal_bundle_|actual_traversal_bundle_request_evidence_gate_result|region_fresh_witness_request_evidence_gate_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "fresh witness authority producer from sources must not accept external witness metadata, call fixture witness builders, traverse via ActualTraversalBundle/request evidence, or synthesize lower backend/artifact outputs",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_collector_owned_no_escape_coverage_authority_bundle_with_owners_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collect_from_walker_input_result &input &observations",
        "Result::Ok sources:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_stage0_with_sources_result sources witness_body_module_fingerprint graph_index root_operation_ordinal support_operation_ordinal status",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "Stage0SourceRejected e",
    ],
    "collector-owned no-escape coverage authority helper must collect sources, close walker/observation owners, and pass sources directly to the fresh witness authority wrapper",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_authority_bundle_from_split_events_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_event_split_result events",
        "Result::Ok output:",
        'field::get output "walker_input"',
        'field::get output "observations"',
        "selfhost_memo_call_backend_private_cache_collector_owned_no_escape_coverage_authority_bundle_with_owners_result input observations witness_body_module_fingerprint graph_index root_operation_ordinal support_operation_ordinal status",
        "Result::Err e:",
        "Stage0SourceRejected SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind::NormalizerRejected e",
    ],
    "operation-classified no-escape split helper must transfer split owners directly to collector-owned authority bundle path",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_authority_bundle_with_operations_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_events_from_hir_root_result module root fuel operation_body_module_fingerprint &operations",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_free operations",
        "Result::Ok events:",
        "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_authority_bundle_from_split_events_result events witness_body_module_fingerprint graph_index root_operation_ordinal support_operation_ordinal status",
        "Result::Err e:",
        "Stage0SourceRejected e",
    ],
    "operation-classified no-escape operation owner helper must classify through HIR-root request authority and keep classifier errors distinct",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_authority_bundle_from_operation_table_result"),
    [
        "Result::Ok operations:",
        "selfhost_memo_call_backend_private_cache_proof_gate_stage0_build_memoized_module function_ty span def_id",
        "Result::Ok module:",
        "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_authority_bundle_with_operations_result &module root 8 operation_body_module_fingerprint operations witness_body_module_fingerprint graph_index root_operation_ordinal support_operation_ordinal status",
        "selfhost_hir_module_free module",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_free operations",
        "Stage0FixtureAllocFailed e",
        "Result::Err e:",
        "Stage0SourceRejected e",
    ],
    "operation-classified no-escape operation-table wrapper must free operations on module fixture failure and map operation table build failures",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_accepted_authority_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_stage0_closed_clone_table_result",
        "witness_body_module_fingerprint 0 0 1 status",
    ],
    "operation-classified no-escape accepted authority bundle must pair closed private-cache storage and clone-out operation ordinals 0/1 with matching witness ordinals",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_stage0_summary_eq"),
    [
        "summary.accepted_no_escape_pair_code 13",
        "summary.missing_witness_rejected missing_expected",
        "summary.rejected_witness_rejected rejected_expected",
        "summary.witness_authority_mismatch_rejected witness_authority_expected",
        "summary.may_escape_rejected escape_expected",
        "summary.observation_rejected observation_expected",
        "summary.unsupported_rejected unsupported_expected",
        "summary.fingerprint_mismatch_rejected fingerprint_expected",
        "summary.graph_mismatch_rejected graph_expected",
    ],
    "operation-classified no-escape summary eq must prove accepted, witness status/mismatch, source rejection, and authority mismatch cases",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_stage0"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_stage0_authority 77 0",
        "accepted_no_escape_pair_code",
        "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_stage0_run_i32_result authority 77 77",
        "PrivateCacheRegionFreshWitnessCandidateAccepted",
        "missing_witness_rejected",
        "PrivateCacheRegionFreshWitnessMissing",
        "rejected_witness_rejected",
        "PrivateCacheRegionFreshWitnessRejected",
        "witness_authority_mismatch_rejected",
        "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_stage0_run_i32_result authority 77 78",
        "may_escape_rejected",
        "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_escape_result authority 77",
        "observation_rejected",
        "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_observation_result authority 77",
        "unsupported_rejected",
        "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_unsupported_result authority 77",
        "fingerprint_authority",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_stage0_authority 78 0",
        "graph_authority",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_stage0_authority 77 1",
    ],
    "operation-classified no-escape stage0 must cover accepted, witness failures, source failures, and authority mismatch paths",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_fresh_witness_authority_bundle_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_from_request_context_result module context resolutions",
        "Result::Ok output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_fresh_witness_authority_bundle_result output",
        "Result::Err _e:",
        'field::get context "key"',
        "ActualTraversalBodyInputUnavailable key",
    ],
    "body-reader fresh witness authority helper must derive witness authority from the origin-tagged source output envelope and keep lookup/source failures typed",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_fresh_witness_authority_bundle_from_request_context_result")),
    /witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|RegionFreshWitnessStatus|ActualWalkerEventSplitOutput|actual_traversal_body_reader_sources_from_request_context_result|actual_traversal_body_reader_events_from_request_context_result|actual_walker_event_split_result|actual_traversal_body_adapter_sources_from_request_context_output_result|actual_walker_traversal_source_collect_from_walker_input_result|actual_traversal_body_context_sources_validate_result|actual_traversal_body_adapter_sources_from_input_owners_result|actual_traversal_body_adapter_sources_from_request_context_result\b|actual_traversal_fresh_witness_authority_bundle_stage0_with_sources_result|region_fresh_witness_stage0_table_result/,
    "body-reader fresh witness authority helper must not accept external witness metadata or route around source output through direct reader/split-output/source-adapter fixture paths",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_lookup_result module context resolutions",
        "Result::Ok body_root:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_from_body_root_result module context body_root",
        "Result::Err _e:",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_source_rejected",
    ],
    "actual traversal source output request-context helper must resolve the body root once and pass that body root into the source output producer",
);
assert.equal(
    countOccurrences(
        stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_from_request_context_result")),
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_lookup_result",
    ),
    1,
    "actual traversal source output request-context helper must perform exactly one resolver lookup",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_from_request_context_result")),
    /body_reader_no_escape_coverage_handoff_pair_from_request_context_result|body_reader_no_escape_coverage_authority_bundle_from_request_context_result|actual_traversal_private_effect_coverage_authority_from_reader_context_result|actual_traversal_body_reader_fresh_witness_authority_bundle_from_request_context_result|actual_traversal_body_reader_sources_from_request_context_result|actual_traversal_body_reader_events_from_request_context_result|actual_walker_event_split_result|body_reader_no_escape_coverage_authority_bundle_from_split_output_result|actual_traversal_body_adapter_sources_from_request_context_output_result|actual_traversal_body_adapter_sources_from_request_context_result/,
    "actual traversal source output request-context helper must not route through existing handoff/authority helpers, second source lookup, or split-output/source adapters",
);
assert.doesNotMatch(
    source,
    /^fn\s+selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_from_split_output_result\b/m,
    "old body-reader no-escape split-output authority helper must not remain as a competing authority path",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheBodyReaderNoEscapeCoverageAuthorityBundle"),
    [
        "coverage_authority %SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageCompleteAuthority",
        "witness_bundle %SelfhostMemoCallBackendPrivateCacheActualTraversalFreshWitnessAuthorityBundle",
    ],
    "body-reader no-escape authority bundle must carry resolver-derived coverage authority and the owner-bearing fresh witness authority bundle",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalNoEscapeCoverageAuthorityBundle"),
    [
        "coverage_authority %SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageCompleteAuthority",
        "witness_bundle %SelfhostMemoCallBackendPrivateCacheActualTraversalFreshWitnessAuthorityBundle",
    ],
    "actual traversal no-escape authority bundle must carry production traversal coverage authority and the owner-bearing fresh witness authority bundle",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualTraversalNoEscapeCoverageAuthorityBundle\b/,
    "actual traversal no-escape authority bundle must stay module-private",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualTraversalNoEscapeCoverageAuthorityBundle\b/,
    "actual traversal no-escape authority bundle must not implement Clone or Copy because it owns the witness bundle",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_from_body_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_from_body_root_result module context body_root",
        "Result::Ok output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_no_escape_authority_bundle_result output",
        "Result::Err _e:",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_source_rejected",
    ],
    "body-reader no-escape body-root helper must route through the origin-tagged source output envelope before building the authority bundle",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_from_body_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_authority_new context.root_expr_id body_root context.body_module_fingerprint context.graph_id",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_root_result module context body_root",
        "Result::Ok sources:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_new",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin::HirReaderSourceDerived",
        "Result::Err _e:",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_source_rejected",
    ],
    "actual traversal source output body-root helper must build coverage authority and source owner from the same resolver-returned body root and mark the current path as HIR reader source-derived",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_sources_from_body_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_new",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_root_result module context body_root",
        "Result::Ok reader_sources:",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_sources_result producer_sources0 reader_sources",
        "Result::Ok producer_sources1:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_sources_validate_result context &producer_sources1",
        "Result::Ok _producer_identity:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_output_from_context_sources_result context producer_sources1",
        "Result::Ok output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_context_output_result context output",
        "Result::Ok lowered_sources:",
        "Result::Ok lowered_sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_fail_with_traversal_sources producer_sources0 e",
    ],
    "resource-lowering producer source helper must use the resolver body root, merge reader sources through the producer bridge, validate source identity, and pass producer-owned sources through event split and collector output",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_sources_from_body_root_result")),
    /actual_traversal_body_resolution_lookup_result|actual_traversal_source_output_from_body_root_result|actual_traversal_source_output_from_request_context_result|actual_traversal_source_output_new|SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin|actual_traversal_body_reader_sources_from_request_context_result|actual_traversal_body_reader_output_from_request_context_result|actual_traversal_body_adapter_sources_from_request_context_result\b|actual_traversal_bundle_|context_bound_reader_traversal_bundle|region_fresh_witness_request_evidence_gate_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering producer source helper must not redo resolver lookup, create source output envelopes, use request-context source adapters, request-evidence, proof, backend, effect, or artifact paths",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_from_body_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_authority_new context.root_expr_id body_root context.body_module_fingerprint context.graph_id",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_sources_from_body_root_result module context body_root",
        "Result::Ok lowered_sources:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_new context coverage_authority body_root lowered_sources",
        "Result::Err _e:",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_source_rejected",
    ],
    "resource-lowering producer output body-root helper must build same-body coverage authority and wrap only producer-owned sources in the producer output owner",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_from_body_root_result")),
    /actual_traversal_body_resolution_lookup_result|actual_traversal_source_output_from_body_root_result|actual_traversal_source_output_from_request_context_result|actual_traversal_source_output_new|SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin|actual_traversal_body_reader_sources_from_request_context_result|actual_traversal_body_reader_output_from_request_context_result|actual_traversal_body_adapter_sources_from_request_context_result\b|actual_traversal_bundle_|context_bound_reader_traversal_bundle|region_fresh_witness_request_evidence_gate_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering producer output body-root helper must not redo resolver lookup, create source output envelopes, use request-context reader adapters, request-evidence, proof, backend, effect, or artifact paths",
);
assert.equal(
    countOccurrences(code, "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_new"),
    2,
    "resource-lowering producer output constructor must appear only in its definition and the body-root producer output helper",
);
assert.equal(
    countOccurrences(code, "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_into_source_output"),
    2,
    "resource-lowering producer output conversion must appear only in its definition and the request-context source-output helper",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_into_source_output"),
    [
        'field::get output "context"',
        'field::get output "coverage_authority"',
        'field::get output "body_root"',
        'field::get output "sources"',
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_new SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin::ResourceLoweringTraversalProduced context coverage_authority body_root sources",
    ],
    "resource-lowering producer output conversion must be the only boundary that mints ResourceLoweringTraversalProduced source output",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_into_source_output")),
    /actual_traversal_private_effect_coverage_authority_new|actual_traversal_body_resolution_lookup_result|actual_traversal_resource_lowering_producer_sources_from_body_root_result|actual_traversal_source_output_from_body_root_result|actual_traversal_source_output_from_request_context_result|body_reader_no_escape_coverage_|region_fresh_witness_resource_table_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering producer output conversion must not rebuild coverage authority or sources, consume no-escape authority, request-evidence, backend, effect mask, or artifact records",
);
assert.equal(
    countOccurrences(code, "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_new"),
    2,
    "resource-lowering producer authority output constructor must appear only in its definition and the producer-output authority conversion",
);
assert.equal(
    countOccurrences(code, "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_sources_into_fresh_witness_authority_bundle_result"),
    2,
    "resource-lowering producer source-to-witness helper must appear only in its definition and the producer-output authority conversion",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_sources_into_fresh_witness_authority_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_sources_validate_result context &sources",
        "Result::Ok _valid:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_from_sources_result sources",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "Result::Err SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind::Stage0SourceRejected e",
    ],
    "resource-lowering producer source-to-witness helper must validate producer-owned source identity, move sources into the same-source witness authority producer, and close sources on validation failure",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_sources_into_fresh_witness_authority_bundle_result")),
    /actual_traversal_source_output_new|SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin|production_fresh_witness_authority_input_new|production_fresh_witness_authority_input_into_bundle_result|actual_traversal_body_resolution_lookup_result|actual_traversal_resource_lowering_producer_sources_from_body_root_result|actual_traversal_source_output_from_body_root_result|actual_traversal_source_output_from_request_context_result|source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering producer source-to-witness helper must not route through source output, production fresh-witness input, request-evidence, backend, effect mask, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_into_authority_output_result"),
    [
        'field::get output "context"',
        'field::get output "coverage_authority"',
        'field::get output "body_root"',
        'field::get output "sources"',
        "selfhost_memo_call_backend_private_cache_actual_traversal_production_coverage_authority_validate_result coverage_authority context body_root",
        "Result::Ok _identity:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_sources_into_fresh_witness_authority_bundle_result context sources",
        "Result::Ok witness_bundle:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_new context coverage_authority body_root witness_bundle",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_error_from_region_error e",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
    ],
    "resource-lowering producer output must validate coverage identity, move sources through the producer-owned source-to-witness boundary, and return producer authority output",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_into_authority_output_result")),
    /actual_traversal_source_output_new|SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin|production_fresh_witness_authority_input_new|production_fresh_witness_authority_input_into_bundle_result|actual_traversal_body_resolution_lookup_result|actual_traversal_resource_lowering_producer_sources_from_body_root_result|actual_traversal_source_output_from_body_root_result|actual_traversal_source_output_from_request_context_result|source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering producer output into-authority helper must not mint source output, rebuild sources, consume no-escape handoff, request-evidence, backend, effect mask, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_into_no_escape_authority_bundle_result"),
    [
        'field::get output "context"',
        'field::get output "coverage_authority"',
        'field::get output "body_root"',
        'field::get output "witness_bundle"',
        "selfhost_memo_call_backend_private_cache_actual_traversal_production_coverage_authority_validate_result coverage_authority context body_root",
        "Result::Ok _identity:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_authority_bundle_new coverage_authority witness_bundle",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_free witness_bundle",
    ],
    "resource-lowering producer authority output must validate coverage identity and move the producer-issued witness bundle into the no-escape authority bundle",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_into_no_escape_authority_bundle_result")),
    /actual_traversal_source_output_new|SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin|production_fresh_witness_authority_input_new|production_fresh_witness_authority_input_into_bundle_result|actual_traversal_fresh_witness_authority_bundle_from_sources_result|region_proof_table_from_sources_result|source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering producer authority output into-no-escape helper must not regenerate witnesses, source outputs, request-evidence, backend, effect mask, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_lookup_result module context resolutions",
        "Result::Ok body_root:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_from_body_root_result module context body_root",
        "Result::Ok producer_output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_into_source_output producer_output",
        "Result::Err e:",
        "Result::Err e",
    ],
    "resource-lowering source output request-context helper must resolve the body root once and move the producer output owner into a ResourceLoweringTraversalProduced source output",
);
assert.equal(
    countOccurrences(
        stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_from_request_context_result")),
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_lookup_result",
    ),
    1,
    "resource-lowering source output request-context helper must perform exactly one resolver lookup",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_from_request_context_result")),
    /actual_traversal_source_output_new|SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin|actual_traversal_resource_lowering_producer_sources_from_body_root_result|actual_traversal_source_output_from_request_context_result|actual_traversal_source_output_from_body_root_result|body_reader_no_escape_coverage_authority_bundle_from_request_context_result|body_reader_no_escape_coverage_handoff_pair_from_request_context_result|actual_traversal_body_reader_events_from_request_context_result|actual_traversal_body_reader_output_from_request_context_result|actual_traversal_body_adapter_sources_from_request_context_result\b|actual_traversal_bundle_|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering source output request-context helper must not directly mint source output, call producer sources directly, route through source-derived output, no-escape handoff, request-context reader output, proof, backend, effect, or artifact helpers",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_lookup_result module context resolutions",
        "Result::Ok body_root:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_from_body_root_result module context body_root",
        "Result::Ok producer_output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_output_into_authority_output_result producer_output",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_production_output_error_from_no_escape_error e",
    ],
    "resource-lowering producer authority request-context helper must resolve once and move the producer output into fresh-witness authority output without source-output minting",
);
assert.equal(
    countOccurrences(
        stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_from_request_context_result")),
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_lookup_result",
    ),
    1,
    "resource-lowering producer authority request-context helper must perform exactly one resolver lookup",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_from_request_context_result")),
    /actual_traversal_source_output_new|SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin|actual_traversal_resource_lowering_producer_sources_from_body_root_result|actual_traversal_resource_lowering_source_output_from_request_context_result|actual_traversal_source_output_from_request_context_result|actual_traversal_source_output_from_body_root_result|body_reader_no_escape_coverage_(?:authority|handoff|pair)|actual_traversal_body_reader_events_from_request_context_result|actual_traversal_body_reader_output_from_request_context_result|actual_traversal_body_adapter_sources_from_request_context_output_result\b|actual_traversal_bundle_|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering producer authority request-context helper must not mint source output, call producer sources directly, route through source-derived/body-reader paths, request-evidence, backend, effect, or artifact helpers",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_stage0_with_body_expr_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_gate_stage0_build_memoized_module_with_body_expr function_ty span def_id body_expr",
        "selfhost_memo_call_backend_request_table_from_hir_root_result &module root 8",
        "selfhost_memo_call_backend_request_table_get_entry &table 0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_request_context_from_entry_result &module entry root context_body_module_fingerprint 0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_stage0_resolution_table_result function_ty def_id context_body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_from_request_context_result &module context &resolutions",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_table_free resolutions",
        "selfhost_memo_call_backend_request_table_free table",
        "selfhost_hir_module_free module",
    ],
    "resource-lowering producer authority stage0 helper must build context/resolutions once, delegate to the producer authority request-context helper, and close outer owners",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_stage0_with_body_expr_result")),
    /resource_lowering_source_output_from_request_context_result|resource_lowering_source_output_stage0_with_body_expr_result|actual_traversal_source_output_new|SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin|production_output_pair_code_result|production_output_into_no_escape_authority_bundle_result|source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_(?:authority|handoff|pair)|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering producer authority stage0 helper must not mint source output, run pair/no-escape gates, request-evidence, backend, effect mask, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_no_escape_authority_bundle_result"),
    [
        'field::get output "origin"',
        "SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin::HirReaderSourceDerived:",
        'field::get output "context"',
        'field::get output "coverage_authority"',
        'field::get output "sources"',
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_from_sources_result coverage_authority context sources",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin::ResourceLoweringTraversalProduced:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_free output",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_source_rejected",
    ],
    "actual traversal source output body-reader helper must accept only source-derived output and must close/reject resource-lowering output",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_no_escape_authority_bundle_result")),
    /body_reader_no_escape_coverage_handoff_pair_from_authority_bundle_result|actual_traversal_no_escape_coverage_|actual_traversal_private_effect_no_escape_coverage_handoff_pair_from_authority_bundle_result|actual_traversal_private_effect_no_escape_coverage_handoff_pair_code_from_authority_bundle_result|GraphInput|proof_table_push|RequestEvidenceProven|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "source output into-authority helper must not route production traversal output, produce handoff pairs, compact codes, proof tables, backend bytes, effect masks, or artifacts",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_from_request_context_result module context resolutions",
        "Result::Ok output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_no_escape_authority_bundle_result output",
        "Result::Err e:",
        "Result::Err e",
    ],
    "body-reader no-escape request-context helper must route through the origin-tagged source output envelope before building the authority bundle",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_from_request_context_result")),
    /actual_traversal_body_resolution_lookup_result|actual_traversal_body_reader_sources_from_request_context_result|actual_traversal_body_adapter_sources_from_request_context_result|body_reader_no_escape_coverage_handoff_pair_from_request_context_result|actual_walker_event_split_result/,
    "body-reader no-escape request-context helper must not own resolver/source lookup or jump to handoff/split-output paths after source output boundary exists",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_from_sources_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_sources_validate_result context &sources",
        "Result::Ok _valid:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_from_sources_result sources",
        "Result::Ok witness_bundle:",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_new coverage_authority witness_bundle",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_error_from_region_error e",
        "Result::Err _e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_source_rejected",
    ],
    "body-reader no-escape source helper must validate the source owner, move it once into fresh witness authority, and close it on validation rejection",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_handoff_pair_from_authority_bundle_result"),
    [
        "field::get bundle \"coverage_authority\"",
        "field::get bundle \"witness_bundle\"",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_handoff_pair_from_authority_bundle_result coverage_authority witness_bundle",
    ],
    "body-reader no-escape handoff-pair helper must move the bundled witness owner into the lower no-escape coverage helper exactly once",
);
const bodyReaderNoEscapeHandoffPairBlock = stripDocComments(
    topLevelBlock(
        source,
        "fn",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_handoff_pair_from_authority_bundle_result",
    ),
);
assert.equal(
    countOccurrences(
        bodyReaderNoEscapeHandoffPairBlock,
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_handoff_pair_from_authority_bundle_result",
    ),
    1,
    "body-reader no-escape handoff-pair helper must call the lower pair-producing helper exactly once",
);
assert.doesNotMatch(
    bodyReaderNoEscapeHandoffPairBlock,
    /selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_handoff_pair_code_from_authority_bundle_result/,
    "body-reader no-escape handoff-pair helper must not skip the pair value boundary by calling the lower compact-code helper",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_handoff_pair_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_from_request_context_result module context resolutions",
        "Result::Ok authority_bundle:",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_handoff_pair_from_authority_bundle_result authority_bundle",
        "Result::Err e:",
        "Result::Err e",
    ],
    "body-reader no-escape request-context handoff helper must produce the backend-private handoff pair from the combined resolver-bound authority bundle",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_pair_code_from_handoff_pair"),
    [
        "Result::Ok selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_handoff_pair_code pair",
    ],
    "body-reader no-escape pair-code projection must derive compact test code from an already-built handoff pair",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_pair_code_from_authority_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_handoff_pair_from_authority_bundle_result bundle",
        "Result::Ok pair:",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_pair_code_from_handoff_pair pair",
        "Result::Err e:",
        "Result::Err e",
    ],
    "body-reader no-escape pair-code helper must go through the handoff pair value boundary before projecting to a compact code",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_handoff_pair_from_authority_bundle_result"),
    [
        'field::get bundle "coverage_authority"',
        'field::get bundle "witness_bundle"',
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_handoff_pair_from_authority_bundle_result coverage_authority witness_bundle",
    ],
    "actual traversal no-escape handoff-pair helper must consume the production traversal authority bundle without routing through body-reader helpers",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_handoff_pair_from_authority_bundle_result")),
    /body_reader_no_escape_coverage_|actual_traversal_private_effect_no_escape_coverage_handoff_pair_code_from_authority_bundle_result|GraphInput|proof_table_push|RequestEvidenceProven|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "actual traversal no-escape handoff-pair helper must not use body-reader helpers or skip the pair value boundary",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_pair_code_from_authority_bundle_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_handoff_pair_from_authority_bundle_result bundle",
        "Result::Ok pair:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_pair_code_from_handoff_pair pair",
        "Result::Err e:",
        "Result::Err e",
    ],
    "actual traversal no-escape pair-code helper must go through the production traversal handoff pair value boundary before projecting to compact code",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_no_escape_pair_code_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_no_escape_authority_bundle_result output",
        "Result::Ok bundle:",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_pair_code_from_authority_bundle_result bundle",
        "Result::Err e:",
        "Result::Err e",
    ],
    "source output pair-code helper must first convert the output into the existing no-escape authority bundle and then reuse the existing pair-code projection",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_no_escape_pair_code_result")),
    /actual_traversal_private_effect_no_escape_coverage_handoff_pair_from_authority_bundle_result|actual_traversal_private_effect_no_escape_coverage_handoff_pair_code_from_authority_bundle_result|GraphInput|proof_table_push|RequestEvidenceProven|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "source output pair-code helper must not bypass the body-reader authority bundle / handoff pair value boundary",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_production_fresh_witness_authority_input_result"),
    [
        'field::get output "origin"',
        "SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin::HirReaderSourceDerived:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_free output",
        "SourceDerivedHirBodyReaderRejected",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin::ResourceLoweringTraversalProduced:",
        'field::get output "context"',
        'field::get output "body_root"',
        'field::get output "sources"',
        "selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_input_new context body_root sources",
    ],
    "production fresh-witness input move helper must reject source-derived output and move only resource-lowering output sources into the module-private input owner",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_production_fresh_witness_authority_input_result")),
    /source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|actual_traversal_fresh_witness_authority_bundle_from_sources_result|region_proof_table_from_sources_result|region_fresh_witness_resource_table_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "production fresh-witness input move helper must not synthesize witnesses, consume no-escape authority, or build proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_input_into_bundle_result"),
    [
        'field::get input "context"',
        'field::get input "sources"',
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_sources_validate_result context &sources",
        "Result::Ok _valid:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_from_sources_result sources",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "Stage0SourceRejected e",
    ],
    "production fresh-witness input bundle helper must validate context-bound sources before moving the input source owner into the same-source fresh witness authority producer",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_input_into_bundle_result")),
    /source_output_into_fresh_witness_authority_bundle_result|source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|region_proof_table_from_sources_result|region_fresh_witness_table_from_candidate_result|actual_traversal_fresh_witness_authority_bundle_stage0_with_sources_result|RegionFreshWitnessStatus|region_fresh_witness_resource_table_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "production fresh-witness input bundle helper must not use source-derived output helpers, consume no-escape authority, or build request-evidence/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_coverage_authority_validate_result"),
    [
        "coverage_authority.request_root_expr_id",
        "context.root_expr_id",
        "SourceBodyIdentityMismatch",
        "coverage_authority.body_root_expr_id",
        "body_root",
        "SourceBodyIdentityMismatch",
        "coverage_authority.body_module_fingerprint context.body_module_fingerprint",
        "SourceBodyIdentityMismatch",
        "selfhost_memo_call_backend_private_cache_resource_graph_id_eq coverage_authority.graph_id context.graph_id",
        "SourceGraphIdentityMismatch",
        "Result::Ok unit",
    ],
    "production coverage authority validation must bind request root, body root, fingerprint, and graph before no-escape handoff",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_witness_count"),
    [
        'field::get bundle "sources"',
        'field::get bundle "witnesses"',
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_len &witnesses",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_free witnesses",
    ],
    "fresh-witness authority bundle witness-count helper must count witnesses and close both source and witness owners",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_witness_count"),
    [
        'field::get bundle "witness_bundle"',
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_witness_count witness_bundle",
    ],
    "no-escape authority bundle witness-count helper must delegate to the owner-closing fresh-witness bundle count helper",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_authority_bundle_witness_count"),
    [
        'field::get bundle "witness_bundle"',
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_witness_count witness_bundle",
    ],
    "actual traversal no-escape authority bundle witness-count helper must delegate to the owner-closing fresh-witness bundle count helper",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_stage0_fresh_witness_input_source_count_result")),
    /source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|actual_traversal_fresh_witness_authority_bundle_from_sources_result|source_output_into_fresh_witness_authority_bundle_result|region_proof_table_from_sources_result|region_fresh_witness_resource_table_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "source-derived fresh-witness input stage0 helper must not synthesize witnesses, consume no-escape authority, or build proof/backend/effect/artifact records",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_fresh_witness_input_source_count_result")),
    /source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|actual_traversal_fresh_witness_authority_bundle_from_sources_result|source_output_into_fresh_witness_authority_bundle_result|region_proof_table_from_sources_result|region_fresh_witness_resource_table_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering fresh-witness input stage0 helper must not synthesize witnesses, consume no-escape authority, or build proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_fresh_witness_bundle_witness_count_with_body_expr_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_with_body_expr_result context_body_module_fingerprint body_expr",
        "Result::Ok output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_production_fresh_witness_authority_input_result output",
        "Result::Ok input:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_input_into_bundle_result input",
        "Result::Ok bundle:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_fresh_witness_authority_bundle_witness_count bundle",
    ],
    "resource-lowering fresh-witness bundle stage0 helper must go through source output, production input owner, bundle producer, and owner-closing witness count in order",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_fresh_witness_bundle_witness_count_with_body_expr_result")),
    /source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|source_output_into_fresh_witness_authority_bundle_result|region_fresh_witness_resource_table_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering fresh-witness bundle stage0 helper must not consume no-escape authority, request-evidence, backend, effect mask, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_output_into_no_escape_authority_bundle_result"),
    [
        'field::get output "origin"',
        "SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin::HirReaderSourceDerived:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_free output",
        "SourceDerivedHirBodyReaderRejected",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutputOrigin::ResourceLoweringTraversalProduced:",
        'field::get output "coverage_authority"',
        'field::get output "context"',
        'field::get output "body_root"',
        'field::get output "sources"',
        "selfhost_memo_call_backend_private_cache_actual_traversal_production_coverage_authority_validate_result coverage_authority context body_root",
        "Result::Ok _identity:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_input_new context body_root sources",
        "selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_input_into_bundle_result input",
        "Result::Ok witness_bundle:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_authority_bundle_new coverage_authority witness_bundle",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_no_escape_coverage_error_from_region_error e",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
    ],
    "production output authority bundle helper must reject source-derived HIR output and move resource-lowering output through production fresh-witness input before creating the combined authority bundle",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_output_into_no_escape_authority_bundle_result")),
    /source_output_no_escape_pair_code_result|source_output_into_no_escape_authority_bundle_result|source_output_into_fresh_witness_authority_bundle_result|actual_traversal_body_reader_fresh_witness_authority_bundle_from_request_context_result|body_reader_no_escape_coverage_authority_bundle_new|body_reader_no_escape_coverage_pair_code_from_authority_bundle_result|body_reader_no_escape_coverage_handoff_pair_from_authority_bundle_result|region_fresh_witness_resource_table_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "production output authority bundle helper must not bypass production fresh-witness input, produce no-escape pair/code, or synthesize request-evidence/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_authority_bundle_witness_count_with_body_expr_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_stage0_with_body_expr_result context_body_module_fingerprint body_expr",
        "Result::Ok authority_output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_into_no_escape_authority_bundle_result authority_output",
        "Result::Ok authority_bundle:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_authority_bundle_witness_count authority_bundle",
    ],
    "resource-lowering authority bundle witness-count stage0 helper must use producer authority output before creating the production no-escape authority bundle and closing the owner after counting",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_authority_bundle_witness_count_with_body_expr_result")),
    /resource_lowering_source_output_stage0_with_body_expr_result|production_output_into_no_escape_authority_bundle_result|source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering authority bundle witness-count stage0 helper must not route back through source output, consume pair/code, request-evidence, backend, effect mask, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_output_pair_code_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_production_output_into_no_escape_authority_bundle_result output",
        "Result::Ok authority_bundle:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_pair_code_from_authority_bundle_result authority_bundle",
        "Result::Err e:",
        "Result::Err e",
    ],
    "production output gate must first build the production no-escape authority bundle and then project through the existing handoff-pair code path",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_output_pair_code_result")),
    /source_output_no_escape_pair_code_result|source_output_into_no_escape_authority_bundle_result|source_output_into_fresh_witness_authority_bundle_result|actual_traversal_body_reader_fresh_witness_authority_bundle_from_request_context_result|body_reader_no_escape_coverage_|region_fresh_witness_resource_table_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "production output gate must not bypass the production fresh-witness input owner or synthesize request-evidence/backend/effect/artifact records in this stage",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_pair_code_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_into_no_escape_authority_bundle_result output",
        "Result::Ok authority_bundle:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_no_escape_coverage_pair_code_from_authority_bundle_result authority_bundle",
        "Result::Err e:",
        "Result::Err e",
    ],
    "resource-lowering producer authority output pair-code helper must use the producer authority output no-escape authority bundle boundary before pair-code projection",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_pair_code_result")),
    /production_output_into_no_escape_authority_bundle_result|source_output_no_escape_pair_code_result|source_output_into_no_escape_authority_bundle_result|source_output_into_fresh_witness_authority_bundle_result|actual_traversal_body_reader_fresh_witness_authority_bundle_from_request_context_result|body_reader_no_escape_coverage_|region_fresh_witness_resource_table_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering producer authority output pair-code helper must not route through source output, production output, body-reader helper, request-evidence, backend, effect mask, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_production_gate_with_body_expr_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_stage0_with_body_expr_result context_body_module_fingerprint body_expr",
        "Result::Ok authority_output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_producer_authority_output_pair_code_result authority_output",
        "Result::Err e:",
        "Result::Err e",
    ],
    "resource-lowering stage0 production gate must use producer authority output rather than source output before no-escape pair-code projection",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_production_gate_with_body_expr_result")),
    /resource_lowering_source_output_stage0_with_body_expr_result|production_output_pair_code_result|production_output_into_no_escape_authority_bundle_result|source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "resource-lowering stage0 production gate must not route back through source output, production-output helper, request-evidence, backend, effect mask, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_output_stage0_summary_eq"),
    [
        "summary.source_derived_source_count 2",
        "summary.source_derived_pair_code 13",
        "SourceDerivedHirBodyReaderRejected",
        "summary.source_derived_production_rejected source_rejected_expected",
        "summary.source_derived_fresh_witness_input_rejected source_rejected_expected",
        "OutputRejected SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectNoEscapeCoverageErrorKind::WitnessUnsupportedSource",
        "summary.hir_body_private_cache_effect_rejected unsupported_expected",
        "summary.resource_lowering_source_count 2",
        "summary.resource_lowering_fresh_witness_input_source_count 2",
        "summary.resource_lowering_authority_bundle_witness_count 1",
        "summary.resource_lowering_no_escape_pair_code 13",
        "summary.resource_lowering_private_cache_effect_source_count 1",
        "summary.resource_lowering_private_cache_effect_no_escape_rejected unsupported_expected",
    ],
    "production output stage0 summary eq must prove source-derived rejection, resource-lowering source output production, fresh-witness input owner separation, no-escape pair acceptance, and private-effect no-escape rejection",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_output_stage0"),
    [
        "source_derived_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_stage0_source_count_result 77",
        "source_derived_pair_code",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_stage0_pair_code_result 77",
        "source_derived_production_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_stage0_production_gate_result 77",
        "source_derived_fresh_witness_input_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_stage0_fresh_witness_input_source_count_result 77",
        "SelfhostEffectKind::PrivateCache",
        "hir_body_private_cache_effect_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_stage0_pair_code_with_body_expr_result 77 private_cache_body_expr",
        "resource_lowering_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_source_count_result 77",
        "resource_lowering_fresh_witness_input_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_fresh_witness_input_source_count_result 77",
        "resource_lowering_authority_bundle_witness_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_authority_bundle_witness_count_result 77",
        "resource_lowering_no_escape_pair_code",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_production_gate_result 77",
        "resource_lowering_private_cache_effect_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_source_count_with_body_expr_result 77 private_cache_body_expr",
        "resource_lowering_private_cache_effect_no_escape_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_production_gate_with_body_expr_result 77 private_cache_body_expr",
    ],
    "production output stage0 must expose source-derived rejection plus resource-lowering source output, fresh-witness input separation, no-escape handoff pair acceptance, and private-effect rejection while keeping request-evidence/backend/effect/artifact outputs disconnected",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_bundle_stage0_summary_eq"),
    [
        "summary.resource_lowering_bundle_witness_count 1",
        "selfhost_memo_call_backend_private_cache_proof_gate_stage0_expected_key",
        "summary.resource_lowering_private_cache_effect_bundle_rejected expected_key",
    ],
    "production fresh-witness authority bundle summary eq must prove bundle witness creation and exact private-effect unsupported rejection",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_bundle_stage0"),
    [
        "resource_lowering_bundle_witness_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_fresh_witness_bundle_witness_count_result 77",
        "SelfhostEffectKind::PrivateCache",
        "resource_lowering_private_cache_effect_bundle_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_resource_lowering_source_output_stage0_fresh_witness_bundle_witness_count_with_body_expr_result 77 private_cache_body_expr",
    ],
    "production fresh-witness authority bundle stage0 must expose bundle witness count and private-effect bundle rejection",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_production_fresh_witness_authority_bundle_stage0")),
    /source_output_no_escape_pair_code_result|body_reader_no_escape_coverage_|source_output_into_fresh_witness_authority_bundle_result|region_fresh_witness_resource_table_result|resource_proof_gate_from_hir_root_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "production fresh-witness authority bundle stage0 must not consume no-escape authority, request-evidence, backend, effect mask, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_stage0_run_i32_with_body_expr_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_gate_stage0_build_memoized_module_with_body_expr function_ty span def_id body_expr",
        "selfhost_memo_call_backend_request_table_from_hir_root_result &module root 8",
        "selfhost_memo_call_backend_request_table_get_entry &table 0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_request_context_from_entry_result &module entry root context_body_module_fingerprint 0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_stage0_resolution_table_result function_ty def_id context_body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_handoff_pair_from_request_context_result &module context &resolutions",
        "Result::Ok pair:",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_pair_code_from_handoff_pair pair",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_table_free resolutions",
        "selfhost_memo_call_backend_request_table_free table",
        "selfhost_hir_module_free module",
    ],
    "body-reader no-escape stage0 runner must use one combined resolver-bound authority bundle before running no-escape coverage",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_stage0_summary_eq"),
    [
        "summary.accepted_no_escape_pair_code 13",
        "summary.hir_body_private_cache_effect_rejected unsupported_expected",
        "summary.hir_body_fn_value_observation_rejected unsupported_expected",
        "summary.hir_body_memoized_function_value_observation_rejected unsupported_expected",
        "summary.hir_body_pure_call_unsupported_rejected unsupported_expected",
    ],
    "body-reader no-escape summary eq must prove accepted same-source authority, private effect, observation, and unsupported HIR body rejection cases",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_stage0"),
    [
        "accepted_no_escape_pair_code",
        "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_stage0_run_i32_result 77",
        "SelfhostEffectKind::PrivateCache",
        "hir_body_private_cache_effect_rejected",
        "selfhost_hir_expr_fn_value",
        "hir_body_fn_value_observation_rejected",
        "selfhost_hir_expr_memoized_function_value",
        "hir_body_memoized_function_value_observation_rejected",
        "SelfhostEffectKind::Pure",
        "hir_body_pure_call_unsupported_rejected",
    ],
    "body-reader no-escape stage0 must cover accepted same-source authority, HIR private effect, HIR observation, and pure call unsupported source paths",
);
for (const helperName of [
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_source_rejected",
    "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_new",
    "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_free",
    "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_source_count",
    "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_from_body_root_result",
    "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_from_request_context_result",
    "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_no_escape_authority_bundle_result",
    "selfhost_memo_call_backend_private_cache_actual_traversal_source_output_into_fresh_witness_authority_bundle_result",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_new",
    "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_fresh_witness_authority_bundle_from_request_context_result",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_from_sources_result",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_from_body_root_result",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_authority_bundle_from_request_context_result",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_handoff_pair_from_authority_bundle_result",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_handoff_pair_from_request_context_result",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_pair_code_from_handoff_pair",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_pair_code_from_authority_bundle_result",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_stage0_run_i32_with_body_expr_result",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_stage0_run_i32_result",
    "selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_stage0",
]) {
    assert.doesNotMatch(
        stripDocComments(topLevelBlock(source, "fn", helperName)),
        /witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|RegionFreshWitnessStatus|actual_traversal_fresh_witness_authority_bundle_stage0_with_sources_result|SelfhostMemoCallBackendPrivateCacheActualTraversalBundle|actual_traversal_bundle_|actual_traversal_body_reader_bundle_|context_bound_reader_(?:traversal_bundle|coverage_witness)|actual_traversal_bundle_source_derived_witness_result|actual_traversal_bundle_request_evidence_gate_result|region_fresh_witness_request_evidence_gate_result|resource_proof_gate_from_hir_root_result|resource_proof_table_to_request_evidence_result|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|resource_graph_input_push|GraphInput|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend|neplobj|neplproof|artifact|actual_traversal_body_reader_events_from_request_context_result|actual_walker_event_split_result|body_reader_no_escape_coverage_authority_bundle_from_split_output_result|actual_traversal_body_adapter_sources_from_request_context_output_result|actual_traversal_body_adapter_sources_from_request_context_result\b|actual_traversal_private_effect_coverage_stage0_(?:authority|mismatched_authority)|actual_traversal_private_effect_no_escape_coverage_handoff_pair_code_from_authority_bundle_result|actual_walker_operation_classifier_events_from_hir_root_result|resource_walker_input_new|resource_walker_input_push_|SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::PrivateCacheStorage|SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::CloneOutOwnedValue|region_fresh_witness_stage0_table_result/i,
        `${helperName} must not route through external witness fixtures, traversal bundles, source-derived request evidence, root-wide classifier fixtures, direct source adapters, lower proof synthesis, GraphInput, backend bytes, effect masking, or artifact keys`,
    );
}
assert.doesNotMatch(
    code,
    /^pub\s+struct\s+SelfhostMemoCallBackendPrivateCacheBodyReaderNoEscapeCoverageAuthorityBundle\b/m,
    "body-reader no-escape coverage authority bundle must stay module-private",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*SelfhostMemoCallBackendPrivateCacheBodyReaderNoEscapeCoverageAuthorityBundle\b/m,
    "public functions must not expose body-reader no-escape coverage authority bundle in their signatures",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*SelfhostMemoCallBackendPrivateCacheActualTraversalSourceOutput\b/m,
    "public functions must not expose owner-bearing actual traversal source output in their signatures",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffPair\b/m,
    "public functions must not expose backend-private coverage handoff pair in their signatures",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheBodyReaderNoEscapeCoverageAuthorityBundle\b/,
    "body-reader no-escape coverage authority bundle must not implement Clone or Copy because it owns a witness authority bundle",
);
for (const helperName of [
    "selfhost_memo_call_backend_private_cache_collector_owned_no_escape_coverage_authority_bundle_with_owners_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_authority_bundle_from_split_events_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_authority_bundle_with_operations_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_authority_bundle_from_operation_table_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_accepted_authority_bundle_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_escape_authority_bundle_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_observation_authority_bundle_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_unsupported_authority_bundle_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_stage0_run_i32_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_escape_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_observation_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_unsupported_result",
    "selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_stage0",
]) {
    assert.doesNotMatch(
        stripDocComments(topLevelBlock(source, "fn", helperName)),
        /SelfhostMemoCallBackendPrivateCacheActualTraversalBundle|actual_traversal_bundle_|collector_owned_traversal_bundle|operation_classified_traversal_bundle|actual_traversal_bundle_source_derived_witness_result|context_bound_reader_traversal_bundle_from_context_result|actual_traversal_bundle_request_evidence_gate_result|region_fresh_witness_request_evidence_gate_result|resource_proof_gate_from_hir_root_result|resource_proof_table_to_request_evidence_result|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|resource_graph_input_push|GraphInput|Wasm|LLVM|PrivateCacheInPureFunction|mask_private|sealed backend|neplobj|neplproof|artifact/i,
        `${helperName} must not route through traversal bundles, source-derived witness, request-evidence, lower proof synthesis, GraphInput, backend bytes, effect masking, or artifact keys`,
    );
}
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_operation_classified_no_escape_coverage_(?!stage0(?:_summary_eq)?\b)/m,
    "operation-classified no-escape coverage internals must stay module-private; only stage0 and summary_eq may be public",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_body_reader_no_escape_coverage_(?!stage0(?:_summary_eq)?\b)/m,
    "body-reader no-escape coverage internals must stay module-private; only stage0 and summary_eq may be public",
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
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheBackendReadiness(?:MaskStatus|RequestEvidence|MaskEvidence|Summary|RequestEvidenceStatus|MaskEvidenceStatus)\b/,
    "backend readiness mask status, evidence, status, and readiness summary must stay private",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskStatus|SelfhostMemoCallBackendPrivateCacheBackendReadinessRequestEvidence|SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskEvidence|SelfhostMemoCallBackendPrivateCacheBackendReadinessSummary|SelfhostMemoCallBackendPrivateCacheBackendReadinessRequestEvidenceStatus|SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskEvidenceStatus)\b/m,
    "public functions must not expose private backend readiness mask status, evidence, status, or summary types in signatures",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_backend_readiness_(?!stage0\b|error_kind_eq\b|error_result_eq\b|count_from_gate_result_and_private_effect_handoff_evidence\b)/m,
    "backend readiness public surface must be limited to stage0 summary, public error comparison helpers, and the narrow private-effect handoff count helper",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_proof_(?:key_new|record_new|table_new|table_free|table_len)\b/m,
    "proof key/table constructors and owner operations must not be public accepted-path building blocks",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_unsupported_result_eq"),
    [
        "Result::Err actual:",
        "SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind::RegionProofUnsupported actual_key:",
        "selfhost_memo_call_backend_private_cache_proof_key_eq actual_key expected_key",
    ],
    "region proof unsupported result helper must verify the typed unsupported variant and expected proof key",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_proof_observation_result_eq"),
    [
        "Result::Err actual:",
        "SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind::RegionProofObservationRejected actual_key:",
        "selfhost_memo_call_backend_private_cache_proof_key_eq actual_key expected_key",
    ],
    "region proof observation result helper must verify the typed observation variant and expected proof key",
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
assert.deepStrictEqual(
    enumVariantNames(source, "SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskStatus"),
    [
        "PrivateEffectMaskProven",
        "PrivateEffectMaskRefuted",
        "PrivateEffectMaskMissing",
        "PrivateEffectMaskUnknown",
    ],
    "backend readiness mask status must distinguish proven, refuted, missing, and unknown private effect mask evidence",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheBackendReadinessRequestEvidence"),
    [
        "root_expr_id %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "request_count %i32",
        "proven_request_count %i32",
    ],
    "backend readiness request evidence must carry body identity alongside request/proof counts",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskEvidence"),
    [
        "root_expr_id %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "status %SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskStatus",
    ],
    "backend readiness mask evidence must carry body identity alongside mask status",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheBackendReadinessSummary"),
    [
        "root_expr_id %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "request_count %i32",
        "proven_request_count %i32",
        "mask_status %SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskStatus",
    ],
    "backend readiness summary must keep identity and counts without backend artifact payloads",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheBackendReadinessRequestEvidenceStatus"),
    [
        "RequestEvidenceReady %SelfhostMemoCallBackendPrivateCacheBackendReadinessRequestEvidence",
        "RequestEvidenceRejected",
        "RequestEvidenceEmpty",
        "RequestEvidenceIncomplete",
        "RequestEvidenceInconsistent",
        "RequestEvidenceBodyModuleFingerprintPlaceholder",
    ],
    "backend readiness request status must separate ready evidence, proof-gate rejection, count failures, and placeholder identity",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskEvidenceStatus"),
    [
        "MaskEvidenceReady %SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskEvidence",
        "MaskEvidenceBodyModuleFingerprintPlaceholder",
    ],
    "backend readiness mask status must separate ready identity-bearing evidence from placeholder identity",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind"),
    [
        "RequestEvidenceRejected",
        "RequestEvidenceEmpty",
        "RequestEvidenceIncomplete",
        "RequestEvidenceInconsistent",
        "BodyModuleFingerprintPlaceholder",
        "PrivateEffectMaskMissing",
        "PrivateEffectMaskUnknown",
        "PrivateEffectMaskRefuted",
        "PrivateEffectMaskIdentityMismatch",
    ],
    "backend readiness errors must keep request-evidence rejection, empty/incomplete/inconsistent counts, placeholder identity, mask statuses, and identity mismatch distinct",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheBackendReadinessStage0Summary")),
    /MaskStatus|RequestEvidence|MaskEvidence|BackendReadinessSummary|ProofTable|GraphInput|Wasm|LLVM|neplobj|neplproof|artifact/i,
    "backend readiness public stage0 summary must expose only counts and typed Result payloads, not private evidence, proof, graph, backend, or artifact payloads",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheBackendReadinessStage0Summary"),
    [
        "upstream_mask_accepted_count %i32",
        "upstream_mask_refuted_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
        "upstream_mask_missing_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
        "upstream_mask_unknown_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
        "upstream_mask_identity_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
        "upstream_mask_placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
    ],
    "backend readiness stage0 summary must include upstream private-effect mask conversion results as public counts/errors only",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheBackendReadinessUpstreamPrivateEffectStatus"),
    [
        "UpstreamPrivateEffectProven",
        "UpstreamPrivateEffectRefuted",
        "UpstreamPrivateEffectMissing",
        "UpstreamPrivateEffectUnknown",
    ],
    "backend readiness upstream private-effect status must distinguish proven, refuted, missing, and unknown without importing checker proof types",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheBackendReadinessUpstreamPrivateEffectEvidence"),
    [
        "root_expr_id %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "status %SelfhostMemoCallBackendPrivateCacheBackendReadinessUpstreamPrivateEffectStatus",
    ],
    "backend readiness upstream private-effect evidence must carry only body identity and neutral status",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_request_status_from_gate_result"),
    [
        "Result SelfhostMemoCallBackendPrivateCacheProofGateSummary SelfhostMemoCallBackendPrivateCacheProofGateErrorKind",
        "eq body_module_fingerprint 0",
        "RequestEvidenceBodyModuleFingerprintPlaceholder",
        "Result::Ok summary:",
        "eq summary.request_count 0",
        "RequestEvidenceEmpty",
        "gt summary.proven_request_count summary.request_count",
        "RequestEvidenceInconsistent",
        "not eq summary.proven_request_count summary.request_count",
        "RequestEvidenceIncomplete",
        "RequestEvidenceReady SelfhostMemoCallBackendPrivateCacheBackendReadinessRequestEvidence root_expr_id body_module_fingerprint summary.request_count summary.proven_request_count",
        "Result::Err _e:",
        "RequestEvidenceRejected",
    ],
    "backend readiness request side must consume the proof-gate Result, classify Err/empty/incomplete/inconsistent counts, and attach identity only to the ready status",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_mask_status_from_upstream_private_effect_status"),
    [
        "UpstreamPrivateEffectProven:",
        "PrivateEffectMaskProven",
        "UpstreamPrivateEffectRefuted:",
        "PrivateEffectMaskRefuted",
        "UpstreamPrivateEffectMissing:",
        "PrivateEffectMaskMissing",
        "UpstreamPrivateEffectUnknown:",
        "PrivateEffectMaskUnknown",
    ],
    "upstream private-effect status must map to private-effect mask status without treating missing/unknown as success",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_mask_status_from_upstream_private_effect_evidence"),
    [
        "selfhost_memo_call_backend_private_cache_backend_readiness_mask_status_from_upstream_private_effect_status evidence.status",
        "selfhost_memo_call_backend_private_cache_backend_readiness_mask_status_from_mask_result evidence.root_expr_id evidence.body_module_fingerprint mask_status",
    ],
    "upstream private-effect evidence conversion must carry evidence root/fingerprint into identity-bearing mask evidence",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_identity_matches"),
    [
        "eq selfhost_hir_expr_id_index request.root_expr_id selfhost_hir_expr_id_index mask.root_expr_id",
        "eq request.body_module_fingerprint mask.body_module_fingerprint",
    ],
    "backend readiness identity check must compare both root expression identity and body module fingerprint",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_gate_result"),
    [
        "not selfhost_memo_call_backend_private_cache_backend_readiness_identity_matches &request &mask",
        "PrivateEffectMaskIdentityMismatch",
        "match mask.status:",
        "PrivateEffectMaskProven:",
        "Result::Ok SelfhostMemoCallBackendPrivateCacheBackendReadinessSummary request.root_expr_id request.body_module_fingerprint request.request_count request.proven_request_count SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskStatus::PrivateEffectMaskProven",
        "PrivateEffectMaskRefuted:",
        "PrivateEffectMaskRefuted",
        "PrivateEffectMaskMissing:",
        "PrivateEffectMaskMissing",
        "PrivateEffectMaskUnknown:",
        "PrivateEffectMaskUnknown",
    ],
    "backend readiness gate must check identity before accepting only Proven mask status and must keep each non-Proven mask status fail-closed",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_gate_result"),
    /_:/,
    "backend readiness mask status fold must not use a wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_summary_from_status_result"),
    [
        "RequestEvidenceReady request:",
        "MaskEvidenceReady mask:",
        "selfhost_memo_call_backend_private_cache_backend_readiness_gate_result request mask",
        "MaskEvidenceBodyModuleFingerprintPlaceholder:",
        "BodyModuleFingerprintPlaceholder",
        "RequestEvidenceRejected:",
        "RequestEvidenceRejected",
        "RequestEvidenceEmpty:",
        "RequestEvidenceEmpty",
        "RequestEvidenceIncomplete:",
        "RequestEvidenceIncomplete",
        "RequestEvidenceInconsistent:",
        "RequestEvidenceInconsistent",
        "RequestEvidenceBodyModuleFingerprintPlaceholder:",
        "BodyModuleFingerprintPlaceholder",
    ],
    "backend readiness status wrapper must map request and mask readiness statuses into accepted summary or typed public errors",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_summary_from_gate_result"),
    [
        "selfhost_memo_call_backend_private_cache_backend_readiness_request_status_from_gate_result root_expr_id body_module_fingerprint gate_result",
        "selfhost_memo_call_backend_private_cache_backend_readiness_mask_status_from_mask_result root_expr_id body_module_fingerprint mask_status",
        "selfhost_memo_call_backend_private_cache_backend_readiness_summary_from_status_result request_status mask_ready_status",
    ],
    "backend readiness wrapper must classify proof-gate Result and create identity-bearing mask status before calling the readiness gate",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_summary_from_gate_result_and_upstream_private_effect_evidence"),
    [
        "selfhost_memo_call_backend_private_cache_backend_readiness_request_status_from_gate_result root_expr_id body_module_fingerprint gate_result",
        "selfhost_memo_call_backend_private_cache_backend_readiness_mask_status_from_upstream_private_effect_evidence upstream_evidence",
        "selfhost_memo_call_backend_private_cache_backend_readiness_summary_from_status_result request_status mask_ready_status",
    ],
    "upstream private-effect readiness wrapper must keep request identity and mask evidence identity separate until the readiness gate checks them",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_gate_result_and_upstream_private_effect_evidence"),
    [
        "selfhost_memo_call_backend_private_cache_backend_readiness_summary_from_gate_result_and_upstream_private_effect_evidence root_expr_id body_module_fingerprint gate_result upstream_evidence",
        "Result::Ok summary:",
        "Result::Ok summary.request_count",
        "Result::Err e:",
        "Result::Err e",
    ],
    "upstream private-effect readiness count helper must only project accepted readiness summaries and preserve typed errors",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_stage0"),
    [
        "accepted_gate_result",
        "selfhost_memo_call_backend_private_cache_backend_readiness_summary_from_gate_result root 77 accepted_gate_result SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskStatus::PrivateEffectMaskProven",
        "request_rejected_gate_result",
        "ProofMissing key",
        "empty_gate_result",
        "SelfhostMemoCallBackendPrivateCacheProofGateSummary 0 0",
        "partial_gate_result",
        "SelfhostMemoCallBackendPrivateCacheProofGateSummary 2 1",
        "inconsistent_gate_result",
        "SelfhostMemoCallBackendPrivateCacheProofGateSummary 1 2",
        "mask_missing_rejected",
        "PrivateEffectMaskMissing",
        "mask_unknown_rejected",
        "PrivateEffectMaskUnknown",
        "mask_refuted_rejected",
        "PrivateEffectMaskRefuted",
        "accepted_request_evidence",
        "mismatched_mask_evidence",
        "SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskEvidence root 78 SelfhostMemoCallBackendPrivateCacheBackendReadinessMaskStatus::PrivateEffectMaskProven",
        "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_evidence_result accepted_request_evidence mismatched_mask_evidence",
        "upstream_accepted_evidence",
        "UpstreamPrivateEffectProven",
        "upstream_refuted_evidence",
        "UpstreamPrivateEffectRefuted",
        "upstream_missing_evidence",
        "UpstreamPrivateEffectMissing",
        "upstream_unknown_evidence",
        "UpstreamPrivateEffectUnknown",
        "upstream_mismatch_evidence",
        "root 78",
        "upstream_placeholder_evidence",
        "root 0",
        "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_gate_result_and_upstream_private_effect_evidence root 77 accepted_gate_result upstream_accepted_evidence",
        "upstream_mask_refuted_rejected",
        "upstream_mask_missing_rejected",
        "upstream_mask_unknown_rejected",
        "upstream_mask_identity_mismatch_rejected",
        "upstream_mask_placeholder_rejected",
    ],
    "backend readiness stage0 must cover accepted, request rejected, empty, partial, inconsistent, direct mask failures, and upstream private-effect mask failures",
);
const readinessImplementation = [
    "selfhost_memo_call_backend_private_cache_backend_readiness_request_status_from_gate_result",
    "selfhost_memo_call_backend_private_cache_backend_readiness_mask_status_from_mask_result",
    "selfhost_memo_call_backend_private_cache_backend_readiness_gate_result",
    "selfhost_memo_call_backend_private_cache_backend_readiness_summary_from_status_result",
    "selfhost_memo_call_backend_private_cache_backend_readiness_summary_from_gate_result",
    "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_evidence_result",
    "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_gate_result",
].map((name) => stripDocComments(topLevelBlock(source, "fn", name))).join("\n");
assert.doesNotMatch(
    readinessImplementation,
    /memo_trait_operation_private_effect_(?:no_escape_gate|resource_no_escape_producer)|ResourceProof|GraphInput|resource_graph_input_push|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "backend readiness implementation must not call checker private-effect gates, synthesize proof records, GraphInput, backend bytes, effect masks, or artifact keys",
);
const upstreamMaskReadinessImplementation = [
    "selfhost_memo_call_backend_private_cache_backend_readiness_mask_status_from_upstream_private_effect_status",
    "selfhost_memo_call_backend_private_cache_backend_readiness_mask_status_from_upstream_private_effect_evidence",
    "selfhost_memo_call_backend_private_cache_backend_readiness_summary_from_gate_result_and_upstream_private_effect_evidence",
    "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_gate_result_and_upstream_private_effect_evidence",
    "selfhost_memo_call_backend_private_cache_upstream_status_from_private_effect_handoff_status",
    "selfhost_memo_call_backend_private_cache_upstream_evidence_from_private_effect_handoff_evidence",
    "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_gate_result_and_private_effect_handoff_evidence",
    "selfhost_memo_call_backend_private_cache_backend_readiness_stage0",
].map((name) => stripDocComments(topLevelBlock(source, "fn", name))).join("\n");
assert.doesNotMatch(
    upstreamMaskReadinessImplementation,
    /memo_trait_operation_private_effect_(?:no_escape_gate|resource_no_escape_producer)|ResourceProof|GraphInput|resource_graph_input_push|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "upstream private-effect readiness conversion must not call checker gates, read Resource proof records, push proof tables, build GraphInput, backend bytes, effect masks, or artifact keys",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_upstream_status_from_private_effect_handoff_status"),
    [
        "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus::Proven:",
        "UpstreamPrivateEffectProven",
        "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus::Refuted:",
        "UpstreamPrivateEffectRefuted",
        "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus::Missing:",
        "UpstreamPrivateEffectMissing",
        "SelfhostMemoCallBackendPrivateCachePrivateEffectReadinessHandoffStatus::Unknown:",
        "UpstreamPrivateEffectUnknown",
    ],
    "private-effect readiness handoff status conversion must preserve Proven, Refuted, Missing, and Unknown exactly",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_upstream_status_from_private_effect_handoff_status"),
    /_:/,
    "private-effect readiness handoff status conversion must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_upstream_evidence_from_private_effect_handoff_evidence"),
    [
        "selfhost_memo_call_backend_private_cache_upstream_status_from_private_effect_handoff_status evidence.status",
        "SelfhostMemoCallBackendPrivateCacheBackendReadinessUpstreamPrivateEffectEvidence evidence.root_expr_id evidence.body_module_fingerprint status",
    ],
    "private-effect readiness handoff evidence conversion must carry handoff identity into private upstream evidence",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_gate_result_and_private_effect_handoff_evidence"),
    [
        "selfhost_memo_call_backend_private_cache_upstream_evidence_from_private_effect_handoff_evidence handoff_evidence",
        "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_gate_result_and_upstream_private_effect_evidence root_expr_id body_module_fingerprint gate_result upstream_evidence",
    ],
    "public handoff count helper must convert public handoff evidence to private upstream evidence and reuse the existing readiness gate",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectReadinessStage0Summary"),
    [
        "closed_clone_missing_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
        "private_cache_effect_unknown_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
        "private_state_effect_unknown_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
        "escape_refuted_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
        "unavailable_unknown_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
        "empty_source_missing_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheBackendReadinessErrorKind",
    ],
    "actual traversal private-effect readiness stage0 summary must expose only typed readiness errors for missing, unknown, refuted, and placeholder cases",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectReadinessStage0Summary")),
    /UpstreamPrivateEffect|MaskEvidence|ProofTable|GraphInput|Wasm|LLVM|neplobj|neplproof|artifact/i,
    "actual traversal private-effect readiness public summary must not expose upstream evidence, mask evidence, proof, graph, backend, or artifact payloads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_status_from_source_kind"),
    [
        "PrivateCacheStoragePlace:",
        "UpstreamPrivateEffectMissing",
        "ReturnCacheReferencePlace:",
        "UpstreamPrivateEffectRefuted",
        "CacheLookupOperation:",
        "UpstreamPrivateEffectUnknown",
        "PrivateCacheEffectOperation:",
        "UpstreamPrivateEffectUnknown",
        "PrivateStateEffectOperation:",
        "UpstreamPrivateEffectUnknown",
        "CacheHitObservation:",
        "UpstreamPrivateEffectRefuted",
        "ResourceIrTraversalUnavailable:",
        "UpstreamPrivateEffectUnknown",
    ],
    "actual traversal private-effect source projection must treat accepted-shaped sources as missing coverage, private-effect operations as unknown, escapes/observations as refuted, and unavailable traversal as unknown",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_status_from_source_kind"),
    /UpstreamPrivateEffectProven|_:/,
    "actual traversal private-effect source projection must not infer Proven and must not use a wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_status_fold"),
    [
        "UpstreamPrivateEffectRefuted:",
        "UpstreamPrivateEffectRefuted",
        "UpstreamPrivateEffectMissing:",
        "UpstreamPrivateEffectRefuted:",
        "UpstreamPrivateEffectRefuted",
        "UpstreamPrivateEffectUnknown:",
        "UpstreamPrivateEffectMissing",
        "UpstreamPrivateEffectUnknown:",
        "UpstreamPrivateEffectMissing:",
        "UpstreamPrivateEffectMissing",
        "UpstreamPrivateEffectProven:",
        "UpstreamPrivateEffectUnknown",
    ],
    "actual traversal private-effect status fold must keep Refuted > Missing > Unknown > Proven fail-closed priority",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_status_fold"),
    /_:/,
    "actual traversal private-effect status fold must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_evidence_from_sources_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_len sources",
        "eq source_count 0",
        "UpstreamPrivateEffectMissing",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_status_from_sources_loop sources 0 source_count SelfhostMemoCallBackendPrivateCacheBackendReadinessUpstreamPrivateEffectStatus::UpstreamPrivateEffectProven",
    ],
    "actual traversal private-effect evidence projection must treat empty source table as Missing and only use Proven as fold identity for non-empty source tables",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_readiness_count_from_source_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_evidence_from_sources_result root_expr_id body_module_fingerprint &sources",
        "selfhost_memo_call_backend_private_cache_backend_readiness_count_from_gate_result_and_upstream_private_effect_evidence root_expr_id body_module_fingerprint gate_result evidence",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
    ],
    "actual traversal private-effect readiness projection must close source owners and pass only neutral upstream evidence into the existing readiness gate",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_readiness_stage0"),
    [
        "closed_clone_missing_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_readiness_count_from_closed_clone_result 77",
        "private_cache_effect_unknown_rejected",
        "PrivateCacheEffectOperation",
        "private_state_effect_unknown_rejected",
        "PrivateStateEffectOperation",
        "escape_refuted_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_readiness_count_from_escape_result 77",
        "unavailable_unknown_rejected",
        "ResourceIrTraversalUnavailable",
        "empty_source_missing_rejected",
        "placeholder_rejected",
    ],
    "actual traversal private-effect readiness stage0 must cover missing accepted-shaped sources, private effect unknowns, escape refutation, unavailable unknown, empty missing, and placeholder rejection",
);
const actualTraversalPrivateEffectReadinessImplementation = [
    "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_status_from_source_kind",
    "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_status_fold",
    "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_status_from_sources_loop",
    "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_upstream_evidence_from_sources_result",
    "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_readiness_count_from_source_result",
    "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_readiness_stage0",
].map((name) => stripDocComments(topLevelBlock(source, "fn", name))).join("\n");
assert.doesNotMatch(
    actualTraversalPrivateEffectReadinessImplementation,
    /SelfhostMemoTraitOperationPrivateEffect|memo_trait_operation_private_effect_|ResourceProof|GraphInput|resource_graph_input_push|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "actual traversal private-effect readiness projection must not call checker slot/proof/mask producers, synthesize Resource proof records, build GraphInput, backend bytes, effect masks, or artifact keys",
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
        "PrivateCacheOperationUnsupported",
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
        "ActualTraversalBodyInputEmpty %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputKeyMismatch %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputGraphMismatch %i32",
        "ActualWalkerTraversalBodyReadFailed %i32",
        "ActualWalkerTraversalBodyChildRangeInvalid %SelfhostHirRangeBuildError",
        "ActualWalkerTraversalBodyChildReadFailed %i32",
        "ActualWalkerTraversalBodyFuelExhausted %i32",
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
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_operations_from_hir_root_result module root fuel body_module_fingerprint resolutions",
        "Result::Ok operations:",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_events_from_hir_root_result module root fuel body_module_fingerprint &operations",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_free operations",
        "Result::Err e:",
        "Result::Err e",
    ],
    "actual walker event producer bridge must derive operations from the resolver-bound HIR body reader source plan, classify those operations into unified events, and close the private operation owner",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_events_from_hir_root_result")),
    /actual_walker_event_table_new|actual_walker_event_producer_bridge_append_requests_loop|proof_key_from_entry_result|resource_graph_id_new|ActualWalkerEventPayload::Unsupported|ResourceWalkerUnsupportedReason::UnknownResourceOperation|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual walker event producer bridge must not keep the old direct body/unsupported event builder or synthesize lower proof, backend, effect, or artifact records",
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
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_events_from_hir_root_result module root fuel body_module_fingerprint resolutions",
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
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_stage0_run_summary_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_stage0_resolution_table_result function_ty def_id body_module_fingerprint",
        "Result::Ok resolutions:",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_from_hir_root_result &module root 8 body_module_fingerprint &resolutions",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_table_free resolutions",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_bridge_error_from_availability_error e",
    ],
    "actual walker event producer bridge stage0 runner must build and close the resolver table around the HIR body reader event stream",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeStage0Summary"),
    [
        "accepted_result %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
    ],
    "actual walker event producer bridge stage0 summary must expose only typed result payloads for accepted reader-derived events, observation precedence, and placeholder rejection",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_stage0"),
    [
        "accepted_result",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_stage0_run_i32_result 77",
        "observation_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_stage0_observation_run_i32_result 77 SelfhostMemoCallBackendPrivateCacheObservationKind::CacheHitObserved",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_producer_bridge_stage0_run_i32_result 0",
    ],
    "actual walker event producer bridge stage0 must cover accepted reader-derived events, observation precedence, and placeholder fingerprint rejection without exposing private unified event tables",
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
        "CacheLookupOperation",
        "CacheInsertOperation",
        "PrivateCacheEffectOperation",
        "PrivateStateEffectOperation",
        "CacheHitObservation",
        "CacheMissObservation",
        "CacheSizeObservation",
        "CacheStatsObservation",
        "CacheClearObservation",
        "CacheDebugObservation",
        "CacheRegionIdentityObservation",
        "FunctionIdentityObservation",
        "FunctionHashObservation",
        "FunctionDebugObservation",
        "ClosureAllocationIdentityObservation",
        "RawIdentityObservation",
        "RawRepresentationObservation",
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
        "CacheLookupOperation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheLookupOperation",
        "CacheInsertOperation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheInsertOperation",
        "PrivateCacheEffectOperation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::PrivateCacheEffectOperation",
        "PrivateStateEffectOperation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::PrivateStateEffectOperation",
        "CacheHitObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheHitObservation",
        "CacheMissObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheMissObservation",
        "CacheSizeObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheSizeObservation",
        "CacheStatsObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheStatsObservation",
        "CacheClearObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheClearObservation",
        "CacheDebugObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheDebugObservation",
        "CacheRegionIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::CacheRegionIdentityObservation",
        "FunctionIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::FunctionIdentityObservation",
        "FunctionHashObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::FunctionHashObservation",
        "FunctionDebugObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::FunctionDebugObservation",
        "ClosureAllocationIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::ClosureAllocationIdentityObservation",
        "RawIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::RawIdentityObservation",
        "RawRepresentationObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerOperationKind::RawRepresentationObservation",
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
        "CacheLookupOperation",
        "CacheInsertOperation",
        "PrivateCacheEffectOperation",
        "PrivateStateEffectOperation",
        "UnsupportedTraversalSource",
        "UnsupportedObservationSource",
        "UnknownResourceOperation",
        "CacheHitObservation",
        "CacheMissObservation",
        "CacheSizeObservation",
        "CacheStatsObservation",
        "CacheClearObservation",
        "CacheDebugObservation",
        "CacheRegionIdentityObservation",
        "FunctionIdentityObservation",
        "FunctionHashObservation",
        "FunctionDebugObservation",
        "ClosureAllocationIdentityObservation",
        "RawIdentityObservation",
        "RawRepresentationObservation",
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
        "CacheLookupOperation:",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::PrivateCacheOperationUnsupported",
        "CacheInsertOperation:",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::PrivateCacheOperationUnsupported",
        "PrivateCacheEffectOperation:",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::PrivateStateBoundaryUnsupported",
        "PrivateStateEffectOperation:",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::PrivateStateBoundaryUnsupported",
        "UnsupportedTraversalSource:",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::UnknownResourceOperation",
        "UnsupportedObservationSource:",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::UnknownProjection",
        "UnknownResourceOperation:",
        "SelfhostMemoCallBackendPrivateCacheResourceWalkerUnsupportedReason::UnknownResourceOperation",
        "CacheHitObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::CacheHitObserved",
        "CacheMissObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::CacheMissObserved",
        "CacheSizeObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::CacheSizeObserved",
        "CacheStatsObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::CacheStatsObserved",
        "CacheClearObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::CacheClearObserved",
        "CacheDebugObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::CacheDebugObserved",
        "CacheRegionIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::CacheRegionIdentityObserved",
        "FunctionIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::FunctionEqualityObserved",
        "FunctionHashObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::FunctionHashObserved",
        "FunctionDebugObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::FunctionDebugObserved",
        "ClosureAllocationIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::ClosureAllocationIdentityObserved",
        "RawIdentityObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::RawIdentityObserved",
        "RawRepresentationObservation:",
        "SelfhostMemoCallBackendPrivateCacheObservationKind::RawRepresentationObserved",
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
        "accepted_result %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "cache_lookup_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "cache_insert_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "private_effect_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "placeholder_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
    ],
    "actual walker operation producer bridge stage0 summary must expose only typed result payloads for accepted, policy rejections, and placeholder rejection, not private operation tables",
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
        "CacheMissObservation",
        "CacheSizeObserved:",
        "CacheSizeObservation",
        "CacheStatsObserved:",
        "CacheStatsObservation",
        "CacheClearObserved:",
        "CacheClearObservation",
        "CacheDebugObserved:",
        "CacheDebugObservation",
        "CacheRegionIdentityObserved:",
        "CacheRegionIdentityObservation",
        "FunctionEqualityObserved:",
        "FunctionIdentityObservation",
        "FunctionHashObserved:",
        "FunctionHashObservation",
        "FunctionDebugObserved:",
        "FunctionDebugObservation",
        "ClosureAllocationIdentityObserved:",
        "ClosureAllocationIdentityObservation",
        "RawIdentityObserved:",
        "RawIdentityObservation",
        "RawRepresentationObserved:",
        "RawRepresentationObservation",
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
        "CacheLookupOperation:",
        "UnsupportedTraversal",
        "CacheInsertOperation:",
        "UnsupportedTraversal",
        "PrivateCacheEffectOperation:",
        "UnsupportedTraversal",
        "PrivateStateEffectOperation:",
        "UnsupportedTraversal",
        "CacheHitObservation:",
        "CacheStateObservation",
        "CacheMissObservation:",
        "CacheStateObservation",
        "CacheSizeObservation:",
        "CacheStateObservation",
        "CacheStatsObservation:",
        "CacheStateObservation",
        "CacheClearObservation:",
        "CacheStateObservation",
        "CacheDebugObservation:",
        "CacheStateObservation",
        "CacheRegionIdentityObservation:",
        "CacheStateObservation",
        "FunctionIdentityObservation:",
        "FunctionIdentityObservation",
        "FunctionHashObservation:",
        "FunctionIdentityObservation",
        "FunctionDebugObservation:",
        "FunctionIdentityObservation",
        "ClosureAllocationIdentityObservation:",
        "FunctionIdentityObservation",
        "RawIdentityObservation:",
        "RawIdentityObservation",
        "RawRepresentationObservation:",
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
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheContextBoundReaderTraversalBundleStage0Summary"),
    [
        "accepted_request_count %i32",
        "accepted_proof_count %i32",
        "actual_body_reader_bundle_accepted_proof_count %i32",
        "hir_body_private_cache_effect_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "hir_body_fn_value_observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "hir_body_memoized_function_value_observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "seed_key_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "seed_graph_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "seed_missing_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "seed_observation_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "seed_unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "seed_malformed_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "producer_not_connected_availability_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
        "missing_reader_availability_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind",
    ],
    "context-bound reader traversal bundle summary must expose only counts, HIR body problem-source rejections, and seed/availability typed Result payloads",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoCallBackendPrivateCacheContextBoundReaderCoverageWitnessBundleErrorKind"),
    ["CoverageRejected", "WitnessRejected"],
    "context-bound coverage witness bundle error must distinguish coverage producer rejection from source-derived witness/request-evidence rejection",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheContextBoundReaderCoverageWitnessBundleStage0Summary"),
    [
        "accepted_request_count %i32",
        "accepted_proof_count %i32",
        "accepted_coverage_pair_code %i32",
        "private_cache_effect_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheContextBoundReaderCoverageWitnessBundleErrorKind",
    ],
    "context-bound coverage witness bundle summary must expose only counts, coverage pair code, and typed rejection payloads",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheContextBoundReaderCoverageWitnessBundle\b/,
    "context-bound coverage witness combined owner must stay module-private",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*(?:SelfhostMemoCallBackendPrivateCacheContextBoundReaderCoverageWitnessBundle|SelfhostMemoCallBackendPrivateCacheContextBoundReaderCoverageWitnessBundleGateSummary|SelfhostMemoCallBackendPrivateCacheActualTraversalPrivateEffectCoverageHandoffPair|SelfhostMemoCallBackendPrivateCacheActualTraversalBundle)\b/m,
    "public functions must not expose combined coverage/witness owner, private gate summary, private handoff pair, or actual traversal bundle types",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheContextBoundReaderCoverageWitnessBundle\b/,
    "context-bound coverage witness combined owner must not implement Clone or Copy",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_coverage_witness_bundle_from_sources_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_handoff_pair_from_sources_result authority &sources",
        "Result::Ok coverage_pair:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_source_derived_witness_result sources",
        "Result::Ok bundle:",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_coverage_witness_bundle_new coverage_pair bundle",
        "Result::Err e:",
        "WitnessRejected e",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "CoverageRejected e",
    ],
    "context-bound coverage witness helper must borrow the same source owner for coverage before moving it into source-derived witness, and must close the source owner on coverage rejection",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_coverage_witness_bundle_from_sources_result")),
    /actual_traversal_bundle_stage0_with_sources_result|region_fresh_witness_stage0_table_result|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof|artifact/i,
    "context-bound coverage witness helper must not use stage0 witness fixtures or synthesize lower proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_coverage_witness_gate_from_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_authority_from_reader_context_result module context resolutions",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_context_result module context resolutions",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_coverage_witness_bundle_from_sources_result authority sources",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_coverage_witness_request_evidence_gate_result module root fuel context.body_module_fingerprint combined",
    ],
    "context-bound coverage witness gate must build resolver-derived coverage authority, create one source owner, and pass that owner through the combined coverage/witness boundary",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_coverage_witness_bundle_stage0"),
    [
        "selfhost_memo_call_backend_private_cache_context_bound_reader_coverage_witness_bundle_stage0_run_summary_result 77 0",
        "let private_cache_effect_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheContextBoundReaderCoverageWitnessBundleErrorKind selfhost_memo_call_backend_private_cache_context_bound_reader_coverage_witness_bundle_stage0_run_i32_with_body_expr_result 77 0 private_cache_body_expr",
        "accepted.coverage_pair_code",
        "private_cache_effect_rejected",
    ],
    "context-bound coverage witness stage0 must prove accepted coverage pair code comes from the same source owner and PrivateCache effect source remains fail-closed for witness/request evidence",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderSeed\b/,
    "actual traversal body reader seed must stay module-private until real body reader owns its construction",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderSeed\b/m,
    "public functions must not expose reader seed as an accepted-path input",
);
assert.doesNotMatch(
    code,
    /pub\s+(?:struct|enum)\s+SelfhostMemoCallBackendPrivateCacheActualTraversalBody(?:LoweringAvailabilityStatus|ResolutionRecord|ResolutionTable)\b/,
    "actual traversal body resolver status, record, and owner table must stay module-private until the real lowering/body reader owns construction",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+\w+[^\n]*SelfhostMemoCallBackendPrivateCacheActualTraversalBody(?:LoweringAvailabilityStatus|ResolutionRecord|ResolutionTable)\b/m,
    "public functions must not expose private body resolver status, records, or owner tables",
);
assert.doesNotMatch(
    code,
    /impl\s+(?:Clone|Copy)\s+for\s+SelfhostMemoCallBackendPrivateCacheActualTraversalBodyResolutionTable\b/,
    "actual traversal body resolution table owner must not implement Clone or Copy",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind"),
    [
        "ActualTraversalBodyInputProducerNotConnected %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputUnavailable %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputUnsupported %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyInputMalformed %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
        "ActualTraversalBodySourceTableAllocFailed %StdErrorKind",
        "ActualTraversalBodySourcePushFailed %StdErrorKind",
        "ActualTraversalBodySourceReadFailed %i32",
        "ActualTraversalBodyChildRangeInvalid %SelfhostHirRangeBuildError",
        "ActualTraversalBodyChildReadFailed %i32",
        "ActualTraversalBodyFuelExhausted %i32",
        "ActualTraversalBodyResolutionTableAllocFailed %StdErrorKind",
        "ActualTraversalBodyResolutionPushFailed %StdErrorKind",
        "ActualTraversalBodyResolutionReadFailed %i32",
        "ActualTraversalBodyResolutionMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionUnavailable %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionUnsupported %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionKeyMismatch %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionFingerprintMismatch %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionRootMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionDuplicate %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodySeedMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodySeedKeyMismatch %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodySeedGraphMismatch %i32",
        "ActualTraversalBodySeedUnsupported %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodySeedObservationUnsupported %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodySeedMalformed %SelfhostMemoCallBackendPrivateCacheResourceWalkerInputScannerErrorKind",
        "ActualTraversalBodySeedObservationBuildRejected %SelfhostMemoCallBackendPrivateCacheProofKey",
    ],
    "availability error taxonomy must keep body resolver and seed missing, key mismatch, graph mismatch, unsupported shape, observation, malformed, and observation-owner build rejection distinct",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_authority_validate_result"),
    [
        'field::get context "key"',
        'field::get context "graph_id"',
        "selfhost_memo_call_backend_private_cache_proof_key_eq seed.key expected_key",
        "selfhost_memo_call_backend_private_cache_resource_graph_id_eq seed.graph_id expected_graph_id",
        "ActualTraversalBodySeedGraphMismatch seed.graph_id.index",
        "ActualTraversalBodySeedKeyMismatch seed.key",
    ],
    "reader seed authority validation must compare the full proof key and graph id against the rechecked request context before owner creation",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_shape_validate_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_place_supported_result seed",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_edge_supported_result seed",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_observation_supported_result seed",
    ],
    "reader seed shape validation must validate place, edge, and observation status before owner-bearing output is built",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_place_supported_result"),
    /_:/,
    "reader seed place validation must not use wildcard fallback",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_edge_supported_result"),
    /_:/,
    "reader seed edge validation must not use wildcard fallback",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_observation_supported_result"),
    /_:/,
    "reader seed observation validation must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_output_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_walker_input_result seed",
        "Result::Ok input:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_empty_observations_result seed",
        "Result::Ok observations:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventSplitOutput input observations",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "ActualTraversalBodySeedMalformed scanner_error",
    ],
    "reader seed output helper must only build owner-bearing output after validation and must close walker input when observation owner creation fails",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_output_result")),
    /actual_traversal_bundle|region_fresh_witness|region_proof|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "reader seed output helper must not synthesize source-derived witness, proof, GraphInput, backend, effect, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_availability_from_seed_result"),
    [
        "match seed_option:",
        "Option::Some seed:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_authority_validate_result context seed",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_shape_validate_result seed",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_seed_output_result seed",
        "Option::None:",
        'field::get context "key"',
        "ActualTraversalBodySeedMissing key",
    ],
    "reader seed availability helper must reject missing seed without owner creation and route Some seed through authority, shape, and output boundaries",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_availability_from_seed_result")),
    /context_bound_reader_traversal_bundle_from_output_result|actual_traversal_bundle|region_fresh_witness|region_proof|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "reader seed availability helper must not bypass context-bound bundle validation or synthesize lower proof/backend artifacts",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_from_candidate_result"),
    [
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_candidate_validate_result candidate",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_new",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_record_new candidate.key candidate.graph_id candidate.root_operation_ordinal candidate.support_operation_ordinal SelfhostMemoCallBackendPrivateCacheRegionFreshWitnessStatus::PrivateCacheRegionFreshWitnessCandidateAccepted",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_push table0 record",
    ],
    "source-derived witness table helper must validate the candidate and derive witness authority from candidate key, graph, and root/support ordinals",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_from_candidate_result")),
    /witness_body_module_fingerprint|graph_index|RequestEvidenceProven|resource_graph_input_push|selfhost_memo_call_backend_private_cache_proof_table_push|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "source-derived witness table helper must not accept external witness metadata or synthesize lower proof, backend, effect, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_source_derived_witness_result"),
    [
        "selfhost_memo_call_backend_private_cache_region_proof_table_from_sources_result &sources",
        "Result::Ok table:",
        "selfhost_memo_call_backend_private_cache_region_no_escape_candidate_from_table_result &table",
        "selfhost_memo_call_backend_private_cache_region_proof_table_free table",
        "Result::Ok candidate:",
        "selfhost_memo_call_backend_private_cache_region_fresh_witness_table_from_candidate_result candidate",
        "Result::Ok witnesses:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_new sources witnesses",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
    ],
    "source-derived traversal bundle helper must build proof table, extract candidate, close proof table, derive witness, and close source owner on candidate or witness failure",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_source_derived_witness_result")),
    /actual_traversal_bundle_stage0_with_sources_result|witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|RequestEvidenceProven|resource_graph_input_push|selfhost_memo_call_backend_private_cache_proof_table_push|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "source-derived traversal bundle helper must not call the external-metadata fixture helper or synthesize lower proof, backend, effect, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_from_output_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_context_output_result context output",
        "Result::Ok sources:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_source_derived_witness_result sources",
        "Result::Err e:",
        "Stage0SourceRejected e",
    ],
    "context-bound reader traversal bundle helper must validate source owners through the context-bound output helper before deriving the witness owner from the source candidate",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_from_output_result")),
    /actual_traversal_body_adapter_sources_from_input_owners_result|actual_traversal_bundle_stage0_with_sources_result|witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|PrivateCacheNoEscapeProven|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "context-bound reader traversal bundle helper must not bypass the context-bound source adapter, call the external-metadata fixture helper, or synthesize proof, GraphInput, backend, effect, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_from_availability_result"),
    [
        "match availability_result:",
        "Result::Ok output:",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_from_output_result context output",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_bridge_error_from_availability_error e",
        "Stage0SourceRejected bridge",
    ],
    "context-bound reader traversal bundle availability helper must only pass Ok output to the context-bound output helper and must turn Err availability into typed rejection",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_from_availability_result")),
    /actual_traversal_body_adapter_sources_from_input_owners_result|actual_traversal_bundle_stage0_with_sources_result|witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|PrivateCacheNoEscapeProven|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "context-bound availability helper must not bypass output context validation, must not turn ProducerNotConnected into an accepted bundle path, and must not synthesize lower proof, backend, effect, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_request_context_result module context resolutions",
        "Result::Ok events:",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_split_result events",
        "Result::Ok output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_from_split_output_result context output",
        "Result::Err e:",
        "Stage0SourceRejected SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind::NormalizerRejected e",
        "Result::Err e:",
        "Stage0SourceRejected e",
    ],
    "actual body reader bundle producer must derive event owners from request context and then split them before the operation-classified collector bundle producer",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_from_request_context_result")),
    /actual_traversal_body_reader_sources_from_request_context_result|actual_traversal_body_adapter_sources_from_request_context_result|actual_traversal_body_reader_bundle_from_context_sources_result|actual_traversal_bundle_source_derived_witness_result sources|actual_traversal_body_reader_availability_from_seed_result|actual_traversal_body_adapter_input_availability_from_request_context_result|context_bound_reader_traversal_bundle_from_availability_result|context_bound_reader_traversal_bundle_from_output_result|actual_traversal_body_adapter_sources_from_input_owners_result|actual_traversal_bundle_stage0_with_sources_result|witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|PrivateCacheNoEscapeProven|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual body reader bundle producer must not bypass the operation-classified collector path, roundtrip through availability/output owners, inject witness metadata, call fixture witness helper, or synthesize proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_from_context_sources_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_context_sources_result context sources",
        "Result::Ok events:",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_split_result events",
        "Result::Ok output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_from_split_output_result context output",
        "Result::Err e:",
        "Stage0SourceRejected SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind::NormalizerRejected e",
        "Result::Err e:",
        "Stage0SourceRejected e",
    ],
    "actual body reader context-source bundle helper must delegate source validation and event build to the event producer, split the events, and only then build the bundle from split output",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_from_context_sources_result")),
    /actual_traversal_body_context_sources_validate_result|actual_walker_operation_producer_bridge_operations_from_sources_result|actual_traversal_body_reader_events_from_context_operations_result|actual_traversal_bundle_source_derived_witness_result sources|actual_traversal_bundle_stage0_with_sources_result|actual_walker_operation_classifier_events_from_hir_root_result|request_table_from_hir_root|witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|PrivateCacheRegionFreshWitnessCandidateAccepted|PrivateCacheNoEscapeProven|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual body reader context-source bundle helper must not use root-wide classifier, direct witness derivation, fixture witness metadata, or lower proof/backend/effect/artifact synthesis",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_from_split_output_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_collect_from_walker_input_result &input &observations",
        "Result::Ok sources:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_sources_validate_result context &sources",
        "Result::Ok _valid:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_source_derived_witness_result sources",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "Stage0SourceRejected e",
    ],
    "actual body reader split-output bundle helper must collect split owners into sources, close split owners, revalidate context, and only then derive witness from collector sources",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_from_split_output_result")),
    /actual_traversal_bundle_stage0_with_sources_result|region_fresh_witness_stage0_table_result|witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|PrivateCacheRegionFreshWitnessCandidateAccepted|PrivateCacheNoEscapeProven|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual body reader split-output bundle helper must not use fixture witness metadata or synthesize lower proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_from_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_from_request_context_result module context resolutions",
    ],
    "context-owned reader traversal bundle helper must delegate bundle ownership to the actual body reader bundle producer",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_from_context_result")),
    /actual_traversal_body_adapter_sources_from_request_context_result|actual_traversal_bundle_source_derived_witness_result|actual_traversal_body_adapter_input_availability_from_request_context_result|context_bound_reader_traversal_bundle_from_availability_result|context_bound_reader_traversal_bundle_from_output_result|actual_traversal_body_adapter_sources_from_input_owners_result|actual_traversal_bundle_stage0_with_sources_result|witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|PrivateCacheNoEscapeProven|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "context-owned reader traversal bundle helper must not duplicate body reader bundle producer work or synthesize proof, backend, effect, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_stage0_run_summary_with_body_expr_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_gate_stage0_build_memoized_module_with_body_expr function_ty span def_id body_expr",
        "selfhost_memo_call_backend_request_table_from_hir_root_result &module root 8",
        "selfhost_memo_call_backend_request_table_get_entry &table 0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_request_context_from_entry_result &module entry root context_body_module_fingerprint graph_index",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_stage0_resolution_table_result function_ty def_id context_body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_from_request_context_result &module context &resolutions",
        "Result::Ok bundle:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_request_evidence_gate_result &module root 8 context_body_module_fingerprint bundle",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_table_free resolutions",
        "selfhost_memo_call_backend_request_table_free table",
        "selfhost_hir_module_free module",
    ],
    "actual body reader bundle stage0 runner must rebuild request authority, call the body reader bundle producer, gate only produced bundles, and close resolution/request/module owners",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_stage0_run_summary_with_body_expr_result")),
    /actual_traversal_body_reader_seed_from_context|actual_traversal_body_reader_availability_from_seed_result|actual_traversal_body_adapter_input_availability_from_request_context_result|context_bound_reader_traversal_bundle_from_availability_result|context_bound_reader_traversal_bundle_from_output_result|actual_traversal_body_adapter_sources_from_input_owners_result|actual_traversal_bundle_stage0_with_sources_result|witness_body_module_fingerprint|root_operation_ordinal|support_operation_ordinal|PrivateCacheNoEscapeProven|resource_graph_input_push|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual body reader bundle stage0 runner must not use seed availability, roundtrip through availability/output owners, inject witness metadata, or synthesize lower proof/backend artifacts",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_stage0_run_summary_result"),
    [
        "let body_expr %SelfhostHirExpr selfhost_hir_expr_unit function_ty span",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_stage0_run_summary_with_body_expr_result context_body_module_fingerprint graph_index body_expr",
    ],
    "actual body reader bundle accepted runner must use the shared body-expr runner with a neutral Unit body",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_summary_with_body_expr_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_gate_stage0_build_memoized_module_with_body_expr function_ty span def_id body_expr",
        "selfhost_memo_call_backend_request_table_from_hir_root_result &module root 8",
        "selfhost_memo_call_backend_request_table_get_entry &table 0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_request_context_from_entry_result &module entry root context_body_module_fingerprint graph_index",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_stage0_resolution_table_result function_ty def_id context_body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_from_context_result &module context &resolutions",
        "Result::Ok bundle:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_request_evidence_gate_result &module root 8 context_body_module_fingerprint bundle",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_table_free resolutions",
        "selfhost_memo_call_backend_request_table_free table",
        "selfhost_hir_module_free module",
    ],
    "context-bound reader traversal bundle body-expr runner must rebuild request authority, route the resolver-provided body through the context-owned bundle helper, delegate bundle gate only when a bundle is produced, and close resolution/request/module owners",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_summary_with_body_expr_result")),
    /actual_traversal_body_reader_seed_from_context|actual_traversal_body_reader_availability_from_seed_result|actual_traversal_body_reader_split_output_from_parts_result|actual_traversal_body_adapter_input_availability_from_request_context_result|context_bound_reader_traversal_bundle_from_availability_result|context_bound_reader_traversal_bundle_from_output_result|actual_traversal_body_adapter_sources_from_input_owners_result|witness_body_module_fingerprint|root_operation_ordinal|support_operation_ordinal|PrivateCacheNoEscapeProven|resource_graph_input_push|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "context-bound reader traversal bundle body-expr runner must not use seed availability, roundtrip through availability/output owners, bypass source validation, or synthesize lower proof records, GraphInput, backend bytes, effect masks, or artifact keys",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_summary_result"),
    [
        "let body_expr %SelfhostHirExpr selfhost_hir_expr_unit function_ty span",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_summary_with_body_expr_result context_body_module_fingerprint graph_index body_expr",
    ],
    "context-bound reader traversal bundle accepted runner must use the shared body-expr runner with a neutral Unit body",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_summary_result")),
    /actual_traversal_body_reader_seed_from_context|actual_traversal_body_reader_availability_from_seed_result|actual_traversal_body_adapter_input_availability_from_request_context_result|context_bound_reader_traversal_bundle_from_availability_result|context_bound_reader_traversal_bundle_from_output_result|actual_traversal_body_adapter_sources_from_input_owners_result|actual_traversal_bundle_request_evidence_gate_result|PrivateCacheNoEscapeProven|resource_graph_input_push|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "neutral accepted runner must not duplicate request/module owner handling or lower proof/backend synthesis outside the shared body-expr runner",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_body_expr_result"),
    [
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_summary_with_body_expr_result context_body_module_fingerprint graph_index body_expr",
        "Result::Ok summary:",
        "Result::Ok summary.proven_request_count",
        "Result::Err e:",
        "Result::Err e",
    ],
    "context-bound reader traversal bundle body-expr i32 helper must only project the shared runner result and preserve typed source-derived rejection errors",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_body_expr_result")),
    /context_bound_reader_traversal_bundle_from_context_result|actual_traversal_bundle_request_evidence_gate_result|actual_traversal_bundle_stage0_with_sources_result|witness_body_module_fingerprint|root_operation_ordinal|support_operation_ordinal|RequestEvidenceProven|resource_graph_input_push|selfhost_memo_call_backend_private_cache_proof_table_push|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "body-expr i32 helper must not rebuild bundles, inject fixture witness metadata, or synthesize lower proof/backend artifacts",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_availability_error_result"),
    [
        "selfhost_memo_call_backend_request_table_from_hir_root_result &module root 8",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_request_context_from_entry_result &module entry root context_body_module_fingerprint graph_index",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_from_availability_result context Result::Err availability_error",
        "Result::Err e:",
        "selfhost_memo_call_backend_request_table_free table",
        "selfhost_hir_module_free module",
    ],
    "context-bound reader traversal bundle availability rejection runner must rebuild request authority and route Err availability through the availability helper without making split output fixtures",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_availability_error_result")),
    /actual_traversal_body_reader_split_output_from_parts_result|context_bound_reader_traversal_bundle_from_output_result|actual_traversal_body_adapter_sources_from_input_owners_result|PrivateCacheNoEscapeProven|resource_graph_input_push|selfhost_memo_call_backend_private_cache_proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "availability rejection runner must not create split output fixtures, bypass the availability helper, or synthesize lower proof/backend artifacts",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0"),
    [
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_summary_result 77 0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_stage0_run_summary_result 77 0",
        "Result::Ok actual_body_reader_bundle_accepted:",
        "SelfhostEffectKind::PrivateCache",
        "hir_body_private_cache_effect_rejected",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_body_expr_result 77 0 private_cache_body_expr",
        "selfhost_hir_expr_fn_value function_ty span fn_value_identity",
        "hir_body_fn_value_observation_rejected",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_body_expr_result 77 0 fn_value_body_expr",
        "selfhost_hir_expr_memoized_function_value function_ty span memoized_body_identity",
        "hir_body_memoized_function_value_observation_rejected",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_body_expr_result 77 0 memoized_body_expr",
        "seed_key_mismatch_rejected",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_seed_result 77 0 78 0",
        "seed_graph_mismatch_rejected",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_seed_result 77 0 77 1",
        "seed_missing_rejected",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_missing_seed_result 77 0",
        "SelfhostMemoCallBackendPrivateCacheObservationBanStatus::ObservationDetected",
        "seed_observation_rejected",
        "seed_unsupported_rejected",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::ReturnCacheReference",
        "seed_malformed_rejected",
        "SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::PrivateCacheStorage -1 SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::CloneOutOwnedValue",
        "producer_not_connected_availability_rejected",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_availability_error_result 77 0 SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind::ActualTraversalBodyInputProducerNotConnected producer_not_connected_key",
        "missing_reader_availability_rejected",
        "selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_stage0_run_i32_with_availability_error_result 77 0 SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind::ActualTraversalBodyInputMissing missing_key",
        "accepted.request_count",
        "accepted.proven_request_count",
        "actual_body_reader_bundle_accepted.proven_request_count",
    ],
    "context-bound reader traversal bundle stage0 must cover accepted production reader output, HIR body private-effect/function-observation source-derived rejections, seed mismatch, missing seed, observation/unsupported/malformed seed, and availability rejection paths",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_context_bound_reader_traversal_bundle_(?!stage0\b)/m,
    "context-bound reader traversal bundle helpers must stay module-private; only the typed stage0 summary function may be public",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_bundle_/m,
    "actual body reader bundle producer helpers must stay module-private until full Resource IR traversal owns the public boundary",
);
assert.doesNotMatch(
    code,
    /^pub\s+struct\s+SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderSourceState/m,
    "actual body reader source state must stay module-private",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_(source_state|append_problem_source_kind|append_accepted_wrapper_sources|fail_with_source_state|finalize_hir_body_sources|hir_body_sources)/m,
    "HIR body reader source-state helpers must stay module-private",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_output_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_request_context_result module context resolutions",
        "Result::Ok events:",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_split_result events",
        "Result::Ok output:",
        "Result::Ok output",
        "Result::Err e:",
        "ActualTraversalBodyNormalizerRejected e",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_availability_error_from_bridge_error context e",
    ],
    "production actual traversal body reader must build owner-bearing output through the request-context event producer and split helper",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_output_from_request_context_result")),
    /ActualTraversalBodyReaderSeed|actual_traversal_body_reader_seed|actual_traversal_body_reader_sources_from_request_context_result|actual_traversal_body_reader_output_from_context_sources_result|resource_walker_stage0_closed_place_edge_input_result|resource_walker_input_new|resource_walker_input_push_|SelfhostMemoCallBackendPrivateCacheResourcePlaceKind::PrivateCacheStorage|SelfhostMemoCallBackendPrivateCacheResourceEdgeKind::CloneOutOwnedValue|resource_walker_producer_bridge_input_from_hir_root_result|actual_walker_event_producer_bridge_from_hir_root_result|resource_walker_producer_bridge_from_hir_root_result|PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "production actual traversal body reader must not hard-code walker events, use seed fixtures, existing unsupported producer bridges, or synthesize proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_sources_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_lookup_result module context resolutions",
        "Result::Ok body_root:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_root_result module context body_root",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_bridge_error_from_availability_error e",
    ],
    "reader source plan helper must resolve DefId-linked body root and pass that root into the module-private HIR body reader before producing source authority",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_new",
        "Result::Ok sources0:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_source_state_new sources0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_expr_id_result module context state0 body_root 64",
        "Result::Ok state1:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_finalize_hir_body_sources_result state1 context",
    ],
    "HIR body reader root helper must allocate one source table, wrap it in source state, traverse the resolver-provided body root, and finalize wrapper sources only after traversal",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderSourceState"),
    [
        "sources %SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceTable",
        "problem_source_count %i32",
        "accepted_source_count %i32",
    ],
    "HIR body reader source state must keep source table owner separate from problem and accepted source counts",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_source_state_new"),
    [
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderSourceState sources 0 0",
    ],
    "HIR body reader source state must start with zero problem and accepted sources",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_source_state_free"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free field::get state \"sources\"",
    ],
    "HIR body reader source state free must close the owned source table",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_expr_id_result"),
    [
        "le fuel 0",
        "ActualWalkerTraversalBodyFuelExhausted idx",
        "selfhost_hir_module_get_expr module expr_id",
        "Option::Some expr:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_expr_result module context state expr fuel",
        "Option::None:",
        "ActualWalkerTraversalBodyReadFailed idx",
    ],
    "HIR body expr-id traversal must use module get_expr authority and keep fuel exhaustion distinct from missing expression reads",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_expr_result"),
    [
        "match expr.payload:",
        "SelfhostHirExprPayload::Error:",
        "UnsupportedTraversalSource",
        "SelfhostHirExprPayload::Unit:",
        "Result::Ok state",
        "SelfhostHirExprPayload::FnValue _identity:",
        "FunctionIdentityObservation",
        "SelfhostHirExprPayload::MemoizedFunctionValue _identity:",
        "FunctionIdentityObservation",
        "SelfhostHirExprPayload::Call call:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_call_result module context state call fuel",
        "SelfhostHirExprPayload::Block children:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_child_range_result module context state children fuel",
        "SelfhostHirExprPayload::If branches:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_child_range_result module context state branches fuel",
    ],
    "HIR body reader must classify typed payloads directly, treat Unit as a neutral leaf, and recurse through Call args, Block children, and If branches",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_expr_result")),
    /selfhost_hir_expr_kind|call\.name|span|diagnostic|source_text|resource_graph_input_push|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "HIR body source classification must not use lossy expr-kind tags, display names, spans, diagnostics, or synthesize proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_call_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_call_source_kind call",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_problem_source_kind_result state context source_kind",
        "Result::Ok state1:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_child_range_result module context state1 call.args fuel",
    ],
    "HIR call source traversal must emit the typed effect source before traversing argument children",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_validate_child_range_result"),
    [
        "selfhost_hir_child_range_first children",
        "selfhost_hir_child_range_count children",
        "selfhost_hir_module_child_len module",
        "selfhost_hir_child_range_new_bounded_result first_child child_count child_table_len",
        "ActualWalkerTraversalBodyChildRangeInvalid e",
    ],
    "HIR body child range validation must preserve range build errors instead of collapsing them into unsupported sources",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_child_range_loop_result"),
    [
        "ge idx n",
        "le fuel 0",
        "ActualWalkerTraversalBodyFuelExhausted idx",
        "selfhost_hir_module_get_child module children idx",
        "Option::Some child_expr_id:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_expr_id_result module context state child_expr_id child_fuel",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_hir_body_sources_from_child_range_loop_result module context next_state children add idx 1 n fuel",
        "Option::None:",
        "ActualWalkerTraversalBodyChildReadFailed idx",
    ],
    "HIR body child traversal must read child ids through the module child table, recurse in order, and keep child read and fuel failures typed",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_finalize_hir_body_sources_result"),
    [
        "field::get state \"problem_source_count\"",
        "field::get state \"accepted_source_count\"",
        "eq problem_source_count 0",
        "eq accepted_source_count 0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_accepted_wrapper_sources_result state context",
        "Result::Ok accepted_state:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_source_state_into_sources accepted_state",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_source_state_into_sources state",
    ],
    "HIR body source finalizer must append wrapper sources only when no problem or accepted source was emitted",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_finalize_hir_body_sources_result")),
    /actual_walker_traversal_source_table_len|\bsource_count\b/,
    "HIR body source finalizer must not use source table length as the problem-source predicate",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_call_source_kind"),
    [
        "match call.effect:",
        "SelfhostEffectKind::Pure:",
        "UnsupportedTraversalSource",
        "SelfhostEffectKind::PrivateState:",
        "PrivateStateEffectOperation",
        "SelfhostEffectKind::PrivateCache:",
        "PrivateCacheEffectOperation",
        "SelfhostEffectKind::ExternalIo:",
        "UnsupportedTraversalSource",
    ],
    "HIR call body classification must use typed call.effect and keep private effects distinct from unsupported ordinary calls",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_call_source_kind")),
    /call\.name|span|diagnostic|source_text|PrivateCacheNoEscapeProven|RequestEvidenceProven|resource_graph_input_push|proof_table_push|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "HIR call body classification must not use diagnostic call names or create proof/backend artifacts",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_context_operations_result"),
    [
        "field::get context \"key\"",
        "field::get context \"graph_id\"",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_table_new",
        "selfhost_memo_call_backend_private_cache_resource_walker_body_record_new key graph_id SelfhostMemoCallBackendPrivateCacheResourceGraphCompleteness::ClosedForPrivateCacheBoundary",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventPayload::Body body",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_table_push events0 body_payload",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_classifier_append_records_loop operations events1 key graph_id 0 selfhost_memo_call_backend_private_cache_actual_walker_operation_table_len operations",
        "ActualWalkerEventBuildRejected e",
    ],
    "reader source output bridge must reuse the existing operation classifier path when turning source-derived operations into split-output events",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_context_sources_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_sources_validate_result context &sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_operations_from_sources_result &sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "Result::Ok operations:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_context_operations_result context &operations",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_table_free operations",
        "event_result",
        "Result::Err e:",
        "Result::Err e",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "Result::Err e",
    ],
    "reader event producer from context sources must validate sources, project operations, close source owner, build context-owned events, close operation owner, and return only event owner or typed bridge error",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_context_sources_result")),
    /actual_walker_event_split_result|actual_walker_traversal_source_collect_from_walker_input_result|actual_traversal_bundle_source_derived_witness_result|actual_traversal_bundle_stage0_with_sources_result|actual_walker_operation_classifier_events_from_hir_root_result|request_table_from_hir_root|witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|PrivateCacheRegionFreshWitnessCandidateAccepted|PrivateCacheNoEscapeProven|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "reader event producer from context sources must not split events, collect sources, derive witnesses, use root-wide classifier, or synthesize lower proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_sources_from_request_context_result module context resolutions",
        "Result::Ok sources:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_context_sources_result context sources",
        "Result::Err e:",
        "Result::Err e",
    ],
    "request-context reader event producer must use the resolver-bound source reader once and delegate validation/projection/event build to the context-source event producer",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_request_context_result")),
    /actual_traversal_body_adapter_sources_from_request_context_result|actual_traversal_body_context_sources_validate_result|actual_walker_operation_producer_bridge_operations_from_sources_result|actual_walker_event_split_result|actual_walker_traversal_source_collect_from_walker_input_result|actual_traversal_bundle_source_derived_witness_result|actual_traversal_bundle_stage0_with_sources_result|actual_walker_operation_classifier_events_from_hir_root_result|request_table_from_hir_root|witness_body_module_fingerprint|graph_index|root_operation_ordinal|support_operation_ordinal|PrivateCacheRegionFreshWitnessCandidateAccepted|PrivateCacheNoEscapeProven|resource_graph_input_push|proof_table_push|RequestEvidenceProven|GraphInput|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "request-context reader event producer must not duplicate validation/projection, split events, collect sources, derive witnesses, use root-wide classifier, or synthesize lower proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_output_from_context_sources_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_events_from_context_sources_result context sources",
        "Result::Ok events:",
        "selfhost_memo_call_backend_private_cache_actual_walker_event_split_result events",
        "Result::Ok output:",
        "Result::Ok output",
        "Result::Err e:",
        "ActualTraversalBodyNormalizerRejected e",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_availability_error_from_bridge_error context e",
    ],
    "reader source output bridge must delegate validation/projection/event build to the event producer, split event owner, and preserve typed fail-closed errors",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_output_from_context_sources_result")),
    /actual_traversal_body_context_sources_validate_result|actual_walker_operation_producer_bridge_operations_from_sources_result|actual_traversal_body_reader_events_from_context_operations_result|actual_walker_traversal_source_collect_from_walker_input_result|actual_traversal_bundle_source_derived_witness_result|actual_traversal_bundle_stage0_with_sources_result|PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "reader source output bridge must not duplicate event producer work, collect sources, derive witnesses, or synthesize lower proof/backend/effect/artifact records",
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
        "selfhost_memo_call_backend_request_table_from_hir_root_result module root fuel",
        "selfhost_memo_call_backend_request_table_get_entry &table 0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_request_context_from_entry_result module entry root body_module_fingerprint 0",
        "field::get context \"key\"",
        "field::get context \"graph_id\"",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_unavailable_sources_from_request_result key graph_id",
        "selfhost_memo_call_backend_private_cache_actual_traversal_bundle_stage0_with_sources_result sources witness_body_module_fingerprint 0 root_operation_ordinal support_operation_ordinal status",
        "SelfhostMemoCallBackendPrivateCacheRegionProofProducerErrorKind::Stage0SourceRejected e",
        "selfhost_memo_call_backend_request_table_free table",
    ],
    "producer-owned unavailable traversal bundle helper must rebuild HIR-root request authority, explicitly build unavailable sources, and delegate source/witness cleanup to the existing bundle helper",
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
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_requests_loop module &table resolutions sources0 root body_module_fingerprint 0 request_count",
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
        "ActualTraversalBodySourceTableAllocFailed %StdErrorKind",
        "ActualTraversalBodySourcePushFailed %StdErrorKind",
        "ActualTraversalBodySourceReadFailed %i32",
        "ActualTraversalBodyChildRangeInvalid %SelfhostHirRangeBuildError",
        "ActualTraversalBodyChildReadFailed %i32",
        "ActualTraversalBodyFuelExhausted %i32",
        "ActualTraversalBodyResolutionTableAllocFailed %StdErrorKind",
        "ActualTraversalBodyResolutionPushFailed %StdErrorKind",
        "ActualTraversalBodyResolutionReadFailed %i32",
        "ActualTraversalBodyResolutionMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionUnavailable %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionUnsupported %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionKeyMismatch %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionFingerprintMismatch %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionRootMissing %SelfhostMemoCallBackendPrivateCacheProofKey",
        "ActualTraversalBodyResolutionDuplicate %SelfhostMemoCallBackendPrivateCacheProofKey",
    ],
    "actual traversal body input availability error taxonomy must distinguish producer-not-connected fallback, missing, real unavailable, unsupported, malformed body inputs, and resolver failures before source table production",
);
assert.doesNotMatch(
    code,
    /^pub\s+enum\s+SelfhostMemoCallBackendPrivateCacheActualTraversalBodyInputAvailabilityErrorKind\b/m,
    "actual traversal body input availability error must stay module-private until the real Resource IR body reader owns the public boundary",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext"),
    [
        "entry %SelfhostMemoCallBackendRequestTableEntry",
        "root_expr_id %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "key %SelfhostMemoCallBackendPrivateCacheProofKey",
        "graph_id %SelfhostMemoCallBackendPrivateCacheResourceGraphId",
    ],
    "actual traversal body reader request context must retain the rechecked request entry, root origin, body fingerprint, proof key, and graph id as owner-free authority material",
);
assert.doesNotMatch(
    code,
    /^pub\s+struct\s+SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext\b/m,
    "actual traversal body reader request context must stay module-private and must not become caller-supplied authority",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_traversal_body_(?:reader_request_context|adapter_.*request_context)/m,
    "actual traversal body reader context helpers must stay module-private until the real body reader owns the public boundary",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_request_context_from_entry_result"),
    [
        "selfhost_memo_call_backend_private_cache_proof_gate_recheck_entry_result module entry",
        "selfhost_memo_call_backend_private_cache_proof_key_from_entry_result entry root_expr_id body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_resource_graph_id_new graph_index",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext entry root_expr_id body_module_fingerprint key graph_id",
        "ProofKeyRejected e",
        "RequestRecheckRejected e",
    ],
    "actual traversal body reader request context helper must recheck the HIR entry, build the existing proof key, create graph id from request ordinal, and keep failures typed",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyResolutionRecord"),
    [
        "source_function_def_id %SelfhostDefId",
        "function_ty %SelfhostTypeId",
        "request_kind %SelfhostMemoCallBackendRequestKind",
        "source_effect %SelfhostEffectKind",
        "type_arg_count %i32",
        "body_root_expr_id %SelfhostHirExprId",
        "body_module_fingerprint %i32",
        "lowering_status %SelfhostMemoCallBackendPrivateCacheActualTraversalBodyLoweringAvailabilityStatus",
    ],
    "actual traversal body resolution record must bind DefId, type/effect/request identity, body root, body fingerprint, and lowering availability status",
);
assertOrdered(
    topLevelBlock(source, "struct", "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyResolutionTable"),
    [
        "records %Vec SelfhostMemoCallBackendPrivateCacheActualTraversalBodyResolutionRecord",
    ],
    "actual traversal body resolution table must be Vec-backed owner storage",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_record_matches_context"),
    [
        "selfhost_def_id_eq record.source_function_def_id context.entry.request.source_function_def_id",
        "selfhost_type_id_eq record.function_ty context.entry.request.function_ty",
        "selfhost_memo_call_backend_private_cache_request_kind_eq record.request_kind context.entry.request.request_kind",
        "selfhost_effect_kind_eq record.source_effect context.entry.request.source_effect",
        "eq record.type_arg_count context.entry.request.type_arg_count",
        "eq record.body_module_fingerprint context.body_module_fingerprint",
    ],
    "body resolver record matching must compare DefId plus type, request kind, effect, type argument count, and body module fingerprint",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_record_validate_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_record_matches_context context record",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_status_result context record",
        "selfhost_hir_module_get_expr module record.body_root_expr_id",
        "Result::Ok record.body_root_expr_id",
        "ActualTraversalBodyResolutionRootMissing key",
        "ActualTraversalBodyResolutionKeyMismatch key",
        "ActualTraversalBodyResolutionFingerprintMismatch key",
    ],
    "body resolver validation must reject identity/fingerprint mismatch, unavailable status, and missing HIR body root before source generation",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_lookup_loop"),
    [
        "ActualTraversalBodyResolutionMissing key",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_table_get resolutions idx",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_record_def_matches_context context record",
        "Option::Some _previous:",
        "ActualTraversalBodyResolutionDuplicate key",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_record_validate_result module context record",
        "ActualTraversalBodyResolutionReadFailed idx",
    ],
    "body resolver lookup must scan by DefId, reject duplicate candidates, validate the selected record, and keep missing/read failures typed",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_resolution_lookup_result")),
    /name|diagnostic|span|SelfhostHirFunctionId|selfhost_hir_module_get_function|resource_graph_input_push|proof_table_push|Wasm|LLVM|neplobj|neplproof/,
    "body resolver lookup must not infer body identity from names, diagnostics, spans, function arena ids, proof tables, backend bytes, or artifacts",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderOperationPolicyKind"),
    [
        "WrapperPrivateCacheStorage",
        "WrapperCloneOutOwnedValue",
        "CacheLookupOperation",
        "CacheInsertOperation",
        "PrivateCacheEffectOperation",
        "PrivateStateEffectOperation",
        "CacheHitObservation",
        "CacheMissObservation",
        "CacheSizeObservation",
        "CacheStatsObservation",
        "CacheClearObservation",
        "CacheDebugObservation",
        "CacheRegionIdentityObservation",
        "FunctionIdentityObservation",
        "FunctionHashObservation",
        "FunctionDebugObservation",
        "ClosureAllocationIdentityObservation",
        "RawIdentityObservation",
        "RawRepresentationObservation",
    ],
    "actual traversal body reader operation policy must keep wrapper, cache operation, private effect, and observation vocabulary typed before source projection",
);
assertOrdered(
    topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderAcceptedSourceKind"),
    [
        "PrivateCacheStoragePlace",
        "CloneOutOwnedValueEdge",
    ],
    "HIR body reader explicit accepted source vocabulary must contain only accepted graph candidate source tags",
);
assert.deepEqual(
    enumVariantNames(source, "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderAcceptedSourceKind"),
    [
        "PrivateCacheStoragePlace",
        "CloneOutOwnedValueEdge",
    ],
    "HIR body reader explicit accepted source vocabulary must not contain any additional accepted variants",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "enum", "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderAcceptedSourceKind")),
    /PrivateCacheEffect|PrivateStateEffect|Observation|Unsupported|Unavailable|CacheLookup|CacheInsert|ReturnCacheReference|PublicStore|ExternalHandle/,
    "HIR body reader explicit accepted source vocabulary must not accept problem, escaping, observation, unsupported, unavailable, or cache-operation source tags",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_operation_policy_source_kind"),
    /_:/,
    "actual traversal body reader operation policy source projection must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_operation_policy_source_kind"),
    [
        "WrapperPrivateCacheStorage:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::PrivateCacheStoragePlace",
        "WrapperCloneOutOwnedValue:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CloneOutOwnedValueEdge",
        "CacheLookupOperation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CacheLookupOperation",
        "CacheInsertOperation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CacheInsertOperation",
        "PrivateCacheEffectOperation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::PrivateCacheEffectOperation",
        "PrivateStateEffectOperation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::PrivateStateEffectOperation",
        "CacheMissObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CacheMissObservation",
        "FunctionHashObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::FunctionHashObservation",
        "RawRepresentationObservation:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::RawRepresentationObservation",
    ],
    "actual traversal body reader policy projection must map policy tags into typed traversal source tags without executing cache operations",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_policy_sources_from_request_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_new",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_wrapper_sources_result sources0 context",
    ],
    "production reader policy source helper must allocate a source table and delegate accepted wrapper pair creation to the wrapper append helper",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_wrapper_sources_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_len &sources",
        "add first_operation_ordinal 1",
        "sources context first_operation_ordinal 0 0 SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderOperationPolicyKind::WrapperPrivateCacheStorage",
        "Result::Ok sources1:",
        "sources1 context second_operation_ordinal 0 0 SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderOperationPolicyKind::WrapperCloneOutOwnedValue",
    ],
    "HIR body reader wrapper append helper must build exactly the accepted wrapper source pair starting from the current source table length",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_policy_sources_from_request_context_result")),
    /resource_graph_input_push|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "production reader policy source helper must not synthesize lower proof tables, backend bytes, effect masks, or artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_fail_with_source_state"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_source_state_free state",
        "Result::Err error",
    ],
    "HIR body reader must close the source state exactly at non-push traversal failure boundaries",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_problem_source_kind_result")),
    /fail_with_source_state|source_state_free|traversal_source_table_free/,
    "HIR body reader problem source append helper must rely on source table push cleanup and must not double-free on push failure",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_problem_source_kind_result"),
    [
        "field::get state \"problem_source_count\"",
        "field::get state \"accepted_source_count\"",
        "field::get state \"sources\"",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_len &sources",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_source_kind_result sources context operation_ordinal 0 0 source_kind",
        "Result::Ok next_sources:",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderSourceState next_sources add problem_source_count 1 accepted_source_count",
        "Result::Err e:",
        "Result::Err e",
    ],
    "HIR body reader problem source append helper must use source table length for traversal ordinal, preserve accepted count, and increment problem count only after successful append",
);
assert.doesNotMatch(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_accepted_source_traversal_source_kind"),
    /_:/,
    "HIR body reader explicit accepted source projection must not use wildcard fallback",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_accepted_source_traversal_source_kind"),
    [
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderAcceptedSourceKind::PrivateCacheStoragePlace:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::PrivateCacheStoragePlace",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderAcceptedSourceKind::CloneOutOwnedValueEdge:",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerTraversalSourceKind::CloneOutOwnedValueEdge",
    ],
    "HIR body reader explicit accepted source projection must map only accepted tags to traversal source tags",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_accepted_source_kind_result")),
    /fail_with_source_state|source_state_free|traversal_source_table_free/,
    "HIR body reader explicit accepted append helper must rely on source table push cleanup and must not double-free on push failure",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_accepted_source_kind_result"),
    [
        "field::get state \"problem_source_count\"",
        "field::get state \"accepted_source_count\"",
        "field::get state \"sources\"",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_len &sources",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_accepted_source_traversal_source_kind accepted_kind",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_source_kind_result sources context operation_ordinal from_index to_index source_kind",
        "Result::Ok next_sources:",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderSourceState next_sources problem_source_count add accepted_source_count 1",
        "Result::Err e:",
        "Result::Err e",
    ],
    "HIR body reader explicit accepted append helper must use source table length for ordinal, preserve problem count, and increment accepted count only after successful append",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_accepted_wrapper_sources_result")),
    /fail_with_source_state|source_state_free|traversal_source_table_free/,
    "HIR body reader accepted wrapper append helper must rely on wrapper source push cleanup and must not double-free on push failure",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_accepted_wrapper_sources_result"),
    [
        "field::get state \"problem_source_count\"",
        "field::get state \"accepted_source_count\"",
        "field::get state \"sources\"",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_wrapper_sources_result sources context",
        "Result::Ok next_sources:",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderSourceState next_sources problem_source_count add accepted_source_count 2",
        "Result::Err e:",
        "Result::Err e",
    ],
    "HIR body reader accepted wrapper append helper must count the wrapper pair as two accepted source records after successful append",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_problem_source_kind_result")),
    /resource_graph_input_push|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "HIR body reader source state append helper must not synthesize proof/backend/effect/artifact records",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_accepted_wrapper_sources_result")),
    /resource_graph_input_push|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "HIR body reader accepted wrapper state helper must not synthesize proof/backend/effect/artifact records",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_accepted_source_kind_result")),
    /resource_graph_input_push|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "HIR body reader explicit accepted source state helper must not synthesize proof/backend/effect/artifact records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_explicit_accepted_sources_from_context_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_new",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_source_state_new sources0",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_accepted_source_kind_result state0 context 0 0 SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderAcceptedSourceKind::PrivateCacheStoragePlace",
        "Result::Ok state1:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_append_accepted_source_kind_result state1 context 0 0 SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderAcceptedSourceKind::CloneOutOwnedValueEdge",
        "Result::Ok state2:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_finalize_hir_body_sources_result state2 context",
    ],
    "explicit accepted source smoke helper must append accepted source pair through source state and finalize without default wrapper duplication",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_explicit_accepted_sources_from_context_result")),
    /append_accepted_wrapper_sources|WrapperPrivateCacheStorage|WrapperCloneOutOwnedValue|resource_graph_input_push|proof_table_push|RequestEvidenceProven|PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "explicit accepted source smoke helper must not use default wrapper policy or synthesize lower proof/backend/effect/artifact records",
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
        "ActualTraversalBodyInputEmpty key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputEmpty key",
        "ActualTraversalBodyInputKeyMismatch key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputKeyMismatch key",
        "ActualTraversalBodyInputGraphMismatch graph_index",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputGraphMismatch graph_index",
        "ActualTraversalBodyResolutionTableAllocFailed e",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyResolutionTableAllocFailed e",
        "ActualTraversalBodyResolutionPushFailed e",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyResolutionPushFailed e",
        "ActualTraversalBodyResolutionReadFailed idx",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyResolutionReadFailed idx",
        "ActualTraversalBodyResolutionMissing key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyResolutionMissing key",
        "ActualTraversalBodyResolutionUnavailable key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyResolutionUnavailable key",
        "ActualTraversalBodyResolutionUnsupported key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyResolutionUnsupported key",
        "ActualTraversalBodyResolutionKeyMismatch key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyResolutionKeyMismatch key",
        "ActualTraversalBodyResolutionFingerprintMismatch key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyResolutionFingerprintMismatch key",
        "ActualTraversalBodyResolutionRootMissing key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyResolutionRootMissing key",
        "ActualTraversalBodyResolutionDuplicate key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyResolutionDuplicate key",
        "ActualTraversalBodySourceTableAllocFailed e",
        "ActualWalkerEventProducerBridgeErrorKind::ActualWalkerTraversalSourceTableAllocFailed e",
        "ActualTraversalBodySourcePushFailed e",
        "ActualWalkerEventProducerBridgeErrorKind::ActualWalkerTraversalSourcePushFailed e",
        "ActualTraversalBodySourceReadFailed idx",
        "ActualWalkerEventProducerBridgeErrorKind::ActualWalkerTraversalSourceReadFailed idx",
        "ActualTraversalBodyChildRangeInvalid range_error",
        "ActualWalkerEventProducerBridgeErrorKind::ActualWalkerTraversalBodyChildRangeInvalid range_error",
        "ActualTraversalBodyChildReadFailed idx",
        "ActualWalkerEventProducerBridgeErrorKind::ActualWalkerTraversalBodyChildReadFailed idx",
        "ActualTraversalBodyFuelExhausted idx",
        "ActualWalkerEventProducerBridgeErrorKind::ActualWalkerTraversalBodyFuelExhausted idx",
        "ActualTraversalBodyOperationTableAllocFailed e",
        "ActualWalkerEventProducerBridgeErrorKind::ActualWalkerOperationTableAllocFailed e",
        "ActualTraversalBodyOperationPushFailed e",
        "ActualWalkerEventProducerBridgeErrorKind::ActualWalkerOperationPushFailed e",
        "ActualTraversalBodyOperationReadFailed idx",
        "ActualWalkerEventProducerBridgeErrorKind::ActualWalkerOperationReadFailed idx",
        "ActualTraversalBodyEventBuildRejected e",
        "ActualWalkerEventProducerBridgeErrorKind::ActualWalkerEventBuildRejected e",
        "ActualTraversalBodyNormalizerRejected e",
        "ActualWalkerEventProducerBridgeErrorKind::NormalizerRejected e",
        "ActualTraversalBodySeedMissing key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputMissing key",
        "ActualTraversalBodySeedKeyMismatch key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputKeyMismatch key",
        "ActualTraversalBodySeedGraphMismatch graph_index",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputGraphMismatch graph_index",
        "ActualTraversalBodySeedUnsupported key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputUnsupported key",
        "ActualTraversalBodySeedObservationUnsupported key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputUnsupported key",
        "ActualTraversalBodySeedMalformed scanner_error",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputMalformed scanner_error",
        "ActualTraversalBodySeedObservationBuildRejected key",
        "ActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputUnavailable key",
    ],
    "actual traversal body adapter must map private availability and seed errors to public bridge errors without collapsing missing, unavailable, unsupported, key mismatch, graph mismatch, or malformed cases",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_availability_from_request_result"),
    [
        "SelfhostHirModule",
        "SelfhostMemoCallBackendRequestTableEntry",
        "SelfhostHirExprId",
        "body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_request_context_from_entry_result module entry root_expr_id body_module_fingerprint graph_id.index",
        "Result::Ok context:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_availability_from_request_context_result module context resolutions",
        "Result::Ok output:",
        "Result::Ok output",
        "Result::Err availability_error:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_bridge_error_from_availability_error availability_error",
        "Result::Err e:",
        "Result::Err e",
    ],
    "actual traversal body input availability boundary must derive reader authority through the recheck/proof-key context helper and delegate only rechecked contexts to the production reader boundary",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_availability_from_request_result")),
    /ActualTraversalBodyReaderRequestContext\s+entry\s+root_expr_id\s+body_module_fingerprint\s+key\s+graph_id|resource_walker_producer_bridge_input_from_hir_root_result|actual_walker_event_producer_bridge_from_hir_root_result|resource_walker_producer_bridge_from_hir_root_result/,
    "production body availability helper must not construct context from caller-supplied key/graph or reuse an existing unsupported producer bridge as if it were the real Resource IR body reader",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_availability_from_request_context_result"),
    [
        "SelfhostHirModule",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyResolutionTable",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_output_from_request_context_result module context resolutions",
    ],
    "actual traversal body context availability boundary must accept the rechecked context and return production reader output",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_availability_from_request_context_result")),
    /ProducerNotConnected|ActualTraversalBodyReaderSeed|actual_traversal_body_reader_seed|resource_walker_stage0_closed_place_edge_input_result|resource_walker_producer_bridge_input_from_hir_root_result|actual_walker_event_producer_bridge_from_hir_root_result|resource_walker_producer_bridge_from_hir_root_result/,
    "context availability helper must not keep the old ProducerNotConnected fallback, use seed fixtures, or reuse existing unsupported producer bridges as the real body reader",
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
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_availability_from_request_result module entry root_expr_id body_module_fingerprint key graph_id resolutions",
        "Result::Ok output:",
        "field::get output \"walker_input\"",
        "field::get output \"observations\"",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind::ActualTraversalBodyInputUnsupported key",
        "Result::Err e:",
        "Result::Err e",
    ],
    "actual traversal body adapter single-source compatibility helper must pass through the rechecked availability boundary, keep accepted real input out of the singular record path, and preserve typed bridge errors",
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
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_context_result"),
    [
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderRequestContext",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyResolutionTable",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_sources_from_request_context_result module context resolutions",
        "Result::Ok sources:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_sources_validate_result context &sources",
        "Result::Ok _valid:",
        "Result::Ok sources",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "Result::Err e",
    ],
    "actual traversal body context source helper must build shared reader source plan tables from the rechecked context and close rejected source owners",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_context_result")),
    /actual_traversal_body_adapter_sources_from_input_owners_result|actual_traversal_body_adapter_unavailable_sources_from_request_result|ActualTraversalBodyInputProducerNotConnected|actual_traversal_body_adapter_input_availability_from_request_context_result/,
    "actual traversal body request context helper must not bypass context-bound source validation, use split-output availability authority, or translate producer-not-connected into an unavailable source table",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_source_record_validate_result"),
    [
        "field::get context \"key\"",
        "field::get context \"graph_id\"",
        "selfhost_memo_call_backend_private_cache_proof_key_eq record.key expected_key",
        "selfhost_memo_call_backend_private_cache_resource_graph_id_eq record.graph_id expected_graph_id",
        "ActualTraversalBodyInputGraphMismatch record.graph_id.index",
        "ActualTraversalBodyInputKeyMismatch record.key",
    ],
    "actual traversal body context source validation must compare both proof key and graph id before accepting available reader output records",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_sources_validate_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_len sources",
        "eq n 0",
        "field::get context \"key\"",
        "ActualTraversalBodyInputEmpty key",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_sources_validate_loop context sources 0 n",
    ],
    "actual traversal body context source table validation must reject empty available outputs before the full record loop",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_context_output_result"),
    [
        "field::get output \"walker_input\"",
        "field::get output \"observations\"",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_input_owners_result input observations",
        "Result::Ok sources:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_sources_validate_result context &sources",
        "Result::Ok _valid:",
        "Result::Ok sources",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
        "Result::Err e",
    ],
    "actual traversal body context output helper must validate source tables after owner conversion and close rejected source owners",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_result"),
    [
        "SelfhostHirModule",
        "SelfhostMemoCallBackendRequestTableEntry",
        "SelfhostHirExprId",
        "body_module_fingerprint",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_input_availability_from_request_result module entry root_expr_id body_module_fingerprint key graph_id resolutions",
        "Result::Ok output:",
        "field::get output \"walker_input\"",
        "field::get output \"observations\"",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_input_owners_result input observations",
        "Result::Err e:",
        "Result::Err e",
    ],
    "actual traversal body adapter request boundary must pass through rechecked typed availability, consume available owners, and preserve bridge errors without unavailable fallback",
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
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_split_output_from_parts_result"),
    [
        "Result::Ok input:",
        "Result::Ok observations:",
        "Result::Ok SelfhostMemoCallBackendPrivateCacheActualWalkerEventSplitOutput input observations",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_resource_walker_input_free input",
        "Result::Err e",
        "Result::Err scanner_error:",
        "Result::Ok observations:",
        "selfhost_memo_call_backend_private_cache_observation_ban_table_free observations",
        "ActualTraversalBodyInputMalformed scanner_error",
        "Result::Err _observation_error:",
        "ActualTraversalBodyInputMalformed scanner_error",
    ],
    "actual traversal body reader connector must put owners in Ok only and close partial owners on every fail-closed path",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_split_output_from_parts_result")),
    /resource_walker_producer_bridge_input_from_hir_root_result|actual_walker_event_producer_bridge_from_hir_root_result|resource_walker_producer_bridge_from_hir_root_result/,
    "actual traversal body reader connector must not call stage0 HIR-root unsupported producer bridges",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_source_count_from_parts_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_split_output_from_parts_result input_result observations_result",
        "Result::Ok output:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_source_count_from_availability_result Result::Ok output",
        "Result::Err e:",
        "Result::Err e",
    ],
    "actual traversal body reader connector smoke must route Ok split output through the existing availability adapter",
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
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_context_result")),
    /PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual traversal body request context helper must only route rechecked context into availability/source adapters and must not synthesize proof, fresh witness, backend, effect, or artifact records",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_context_output_result")),
    /PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual traversal body context output helper must only validate and route source table owners and must not synthesize proof, fresh witness, backend, effect, or artifact records",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_context_source_record_validate_result")),
    /PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual traversal body context source record validation must only compare authority fields and must not synthesize proof, fresh witness, backend, effect, or artifact records",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_split_output_from_parts_result")),
    /PrivateCacheNoEscapeProven|PrivateCacheRegionFreshWitnessCandidateAccepted|resource_graph_input_push|proof_table_push|RequestEvidenceProven|Wasm|LLVM|mask_private|sealed backend|neplobj|neplproof/,
    "actual traversal body reader split connector must not synthesize proof, fresh witness, backend, effect, or artifact records",
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
        "reader_connector_available_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "reader_context_reader_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "explicit_accepted_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "hir_body_private_cache_effect_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "hir_body_private_state_effect_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "hir_body_function_identity_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "hir_body_memoized_function_identity_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "hir_body_pure_call_unsupported_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "hir_body_block_unit_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "hir_body_block_private_cache_effect_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "reader_context_available_source_count %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "reader_context_key_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "reader_context_graph_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "reader_context_empty_source_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "availability_missing_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "availability_unavailable_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "availability_unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "availability_malformed_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "resolution_missing_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "resolution_unavailable_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "resolution_unsupported_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "resolution_fingerprint_mismatch_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "resolution_root_missing_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
        "resolution_duplicate_rejected %Result i32 SelfhostMemoCallBackendPrivateCacheActualWalkerEventProducerBridgeErrorKind",
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
        "reader_connector_available_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_source_count_from_parts_result selfhost_memo_call_backend_private_cache_resource_walker_stage0_closed_place_edge_input_result",
        "reader_context_reader_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_source_count_result 77",
        "explicit_accepted_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_explicit_accepted_source_count_result 77",
        "hir_body_private_cache_effect_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_source_count_with_body_call_effect_result 77 SelfhostEffectKind::PrivateCache",
        "hir_body_private_state_effect_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_source_count_with_body_call_effect_result 77 SelfhostEffectKind::PrivateState",
        "hir_body_function_identity_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_source_count_with_body_fn_value_result 77",
        "hir_body_memoized_function_identity_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_source_count_with_body_memoized_function_value_result 77",
        "hir_body_pure_call_unsupported_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_source_count_with_body_call_effect_result 77 SelfhostEffectKind::Pure",
        "hir_body_block_unit_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_source_count_with_block_unit_result 77",
        "hir_body_block_private_cache_effect_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_source_count_with_block_private_cache_call_result 77",
        "reader_context_available_source_count",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_stage0_with_context_result 77 0",
        "reader_context_key_mismatch_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_stage0_with_context_result 78 0",
        "reader_context_graph_mismatch_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_stage0_with_context_result 77 1",
        "reader_context_empty_source_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_empty_source_count_result 77 0",
        "availability_missing_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_availability_missing_source_count_result",
        "availability_unavailable_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_availability_unavailable_source_count_result",
        "availability_unsupported_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_availability_unsupported_source_count_result",
        "availability_malformed_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_availability_malformed_source_count_from_input_result selfhost_memo_call_backend_private_cache_resource_walker_stage0_placeholder_input_result",
        "resolution_missing_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_missing_resolution_result 77",
        "resolution_unavailable_rejected",
        "BodyLoweringUnavailable",
        "resolution_unsupported_rejected",
        "BodyLoweringUnsupported",
        "resolution_fingerprint_mismatch_rejected",
        "77 78",
        "selfhost_hir_expr_id_new 99",
        "resolution_root_missing_rejected",
        "resolution_duplicate_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_context_duplicate_resolution_result 77",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_source_count_from_input_result selfhost_memo_call_backend_private_cache_resource_walker_stage0_placeholder_input_result",
    ],
    "actual traversal body input adapter stage0 must cover unavailable fallback, accepted-shaped, observation-shaped, unsupported, merged, typed availability, reader connector, production reader context output, resolver-bound private effects, function observations, pure-call unsupported source, context-bound available output, context mismatch rejections, and malformed placeholder body inputs",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_(?!input_stage0\b)/m,
    "actual traversal body adapter helpers must stay module-private; only the typed stage0 summary function may be public",
);
assert.doesNotMatch(
    code,
    /^pub\s+fn\s+selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_/m,
    "actual traversal body reader connector helpers must stay module-private until the real Resource IR body reader owns the public boundary",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_reader_request_context_from_entry_result module entry root_expr_id body_module_fingerprint graph_index",
        "Result::Ok context:",
        "selfhost_memo_call_backend_private_cache_actual_traversal_body_adapter_sources_from_request_context_result module context resolutions",
        "Result::Ok request_sources:",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_sources_result sources request_sources",
        "Result::Err e:",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_fail_with_traversal_sources sources e",
    ],
    "actual walker operation producer bridge must derive a rechecked reader request context and merge request-local source table owners from the context-based body adapter",
);
assert.doesNotMatch(
    stripDocComments(topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_result")),
    /proof_key_from_entry_result|resource_graph_id_new/,
    "actual walker operation producer append_request must not build proof keys or graph ids directly after the reader context boundary exists",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_requests_loop"),
    [
        "selfhost_memo_call_backend_request_table_get_entry table idx",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_request_result module entry root_expr_id body_module_fingerprint resolutions sources idx",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_append_requests_loop module table resolutions next_sources root_expr_id body_module_fingerprint add idx 1 n",
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
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_traversal_sources_from_hir_root_result module root fuel body_module_fingerprint resolutions",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_operations_from_sources_result &sources",
        "selfhost_memo_call_backend_private_cache_actual_walker_traversal_source_table_free sources",
    ],
    "actual walker operation producer bridge must build traversal sources first, project them to operation records, and close the source table",
);
assertOrdered(
    topLevelBlock(source, "fn", "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_from_hir_root_result"),
    [
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_operations_from_hir_root_result module root fuel body_module_fingerprint resolutions",
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
        "accepted_result",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_stage0_run_i32_result 77",
        "cache_lookup_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderOperationPolicyKind::CacheLookupOperation",
        "cache_insert_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderOperationPolicyKind::CacheInsertOperation",
        "private_effect_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderOperationPolicyKind::PrivateCacheEffectOperation",
        "observation_rejected",
        "SelfhostMemoCallBackendPrivateCacheActualTraversalBodyReaderOperationPolicyKind::CacheMissObservation",
        "placeholder_rejected",
        "selfhost_memo_call_backend_private_cache_actual_walker_operation_producer_bridge_stage0_run_i32_result 0",
    ],
    "actual walker operation producer bridge stage0 must cover production accepted traversal, lookup/insert/effect/observation policy rejections, and placeholder fingerprint rejection without exposing private operation tables",
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
    /cache_(?:lookup|insert)_(?:execute|run|write|read|alloc|drop)|execute_cache_(?:lookup|insert)|CacheAlloc|CacheDrop|Wasm|LLVM|wasm_|llvm_|sealed|backend_bytes|neplobj|neplproof/,
    "private-cache proof gate must not create executable cache operations, backend bytes, sealed representation, or persistent artifact IO",
);
assert.doesNotMatch(
    source,
    /line[_-]?count|doc(?:umentation)?[_-]?comment(?:s)?[^\n]*(?:limit|cap|max)|max[_-]?(?:lines|doc)/i,
    "private-cache proof gate source policy must not introduce line-count or doc-comment length limits",
);

console.log("selfhost memo_call backend private cache proof gate contract ok");
