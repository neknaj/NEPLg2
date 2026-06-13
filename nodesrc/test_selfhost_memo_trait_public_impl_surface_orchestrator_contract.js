#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function listFiles(dirRelPath, predicate) {
    const dirPath = path.join(repoRoot, dirRelPath);
    const out = [];
    for (const entry of fs.readdirSync(dirPath, { withFileTypes: true })) {
        const relPath = path.join(dirRelPath, entry.name).replace(/\\/g, "/");
        if (entry.isDirectory()) {
            out.push(...listFiles(relPath, predicate));
        } else if (predicate(relPath)) {
            out.push(relPath);
        }
    }
    return out;
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

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_surface_orchestrator.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_public_impl_surface_orchestrator",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "public impl surface orchestrator must document purpose, contract, current limits, complexity, and a doctest",
);
{
    const lines = source.split("\n");
    const missingDocs = [];
    for (let i = 0; i < lines.length; i += 1) {
        if (/^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/.test(lines[i])) {
            let j = i - 1;
            while (j >= 0 && lines[j].trim() === "") {
                j -= 1;
            }
            if (j < 0 || !lines[j].trimStart().startsWith("//:")) {
                missingDocs.push(`${i + 1}: ${lines[i]}`);
            }
        }
    }
    assert.deepEqual(
        missingDocs,
        [],
        "every orchestrator declaration, including private stage0 helpers and impl blocks, must have a preceding doc comment",
    );
}
assert.ok(
    source.includes("scanner output の 2 系統を同じ境界で消費し") &&
        source.includes("public surface normalizer、full public surface hash composer、operation public impl materializer を順番に呼びます"),
    "docs must state that scanner output feeds both the public surface and operation materializer paths in one orchestration boundary",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、method name string、trait name string") &&
        source.includes("public surface、operation、HIR root を推測しません"),
    "docs must reject source-derived authority for surface, operation, or HIR root decisions",
);
assert.ok(
    source.includes("stage0 doctest は compile time を不必要に増やさないため") &&
        source.includes("synthetic typed `Result::Err` を使って wrapping contract を確認します"),
    "docs must explain why stage0 executes only the accepted path and uses typed synthetic rejection payloads",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_surface_orchestrator/,
    "public impl surface orchestrator must remain facade-private until the full proof orchestration boundary is ready",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_surface_orchestrator/,
    "checker-layer public impl surface orchestrator must not be registered in the ty source list",
);
assertOrdered(
    source,
    [
        "#import \"./memo_trait_operation_classifier\" as *",
        "#import \"./memo_trait_operation_impl_table\" as *",
        "#import \"./memo_trait_operation_public_impl_materializer\" as *",
        "#import \"./memo_trait_public_impl_header\" as *",
        "#import \"./memo_trait_public_impl_scanner\" as *",
        "#import \"./memo_trait_public_surface_hash\" as *",
        "#import \"./memo_trait_public_surface_normalizer\" as *",
        "#import \"./memo_trait_source_evidence_producer\" as *",
    ],
    "orchestrator imports must stay on scanner, surface hash, normalizer, operation materializer/table, header, classifier, and seed evidence boundaries",
);
{
    const memoTraitImports = [...source.matchAll(/^#import "(\.\/memo_trait[^"]+)" as \*/gm)].map((match) => match[1]);
    assert.deepEqual(
        memoTraitImports,
        [
            "./memo_trait_operation_classifier",
            "./memo_trait_operation_impl_table",
            "./memo_trait_operation_public_impl_materializer",
            "./memo_trait_public_impl_header",
            "./memo_trait_public_impl_scanner",
            "./memo_trait_public_surface_hash",
            "./memo_trait_public_surface_normalizer",
            "./memo_trait_source_evidence_producer",
        ],
        "orchestrator must keep an explicit memo_trait import allow-list and must not drift below the materializer boundary",
    );
}
assert.ok(
    source.includes("`memo_trait_public_impl_header` の直接 import は stage0 の typed resolver record fixture") &&
        source.includes("production path の impl header validation は scanner boundary が担当します"),
    "docs must explain that the header import is only for stage0 fixture construction and not production header authority",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop_impl_resolver|memo_trait_operation_impl_candidate_builder)/,
    "orchestrator must not import Resource IR, backend, proof store, canonical-key, producer, purity, method-body, body-check, Drop resolver, or direct candidate-builder layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplSurfaceState:",
        "public_surface_hash %i32",
        "operation_impls %SelfhostMemoTraitOperationImplTable",
    ],
    "surface state must carry the full public surface hash and the operation impl table owner together",
);
assert.ok(
    source.includes("transport owner state") &&
        source.includes("complete proof ではありません") &&
        source.includes("proof authority として直接構築してはいけません"),
    "surface state docs must say the state is transport, not final proof authority",
);
{
    const externalUses = listFiles("stdlib/neplg2", (candidateRelPath) => candidateRelPath.endsWith(".nepl")).filter(
        (candidateRelPath) =>
            candidateRelPath !== relPath && read(candidateRelPath).includes("SelfhostMemoTraitPublicImplSurfaceState"),
    );
    assert.deepEqual(
        externalUses,
        ["stdlib/neplg2/core/check/module/memo_trait_public_impl_operation_evidence_connector.nepl"],
        "surface state may only be consumed by the checker-layer operation evidence connector and must remain out of facade-level public APIs",
    );
}
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplSurfaceOrchestratorErrorKind:",
        "ScannerRejected %SelfhostMemoTraitPublicImplScannerErrorKind",
        "PublicSurfaceNormalizerRejected %SelfhostMemoTraitPublicSurfaceNormalizerErrorKind",
        "PublicSurfaceHashRejected %SelfhostMemoTraitPublicSurfaceHashErrorKind",
        "OperationMaterializerRejected %SelfhostMemoTraitOperationPublicImplMaterializerErrorKind",
    ],
    "orchestrator errors must preserve the rejecting lower boundary as typed enum payloads",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_state_free"),
    [
        "field::get state \"operation_impls\"",
        "selfhost_memo_trait_operation_impl_table_free operation_impls",
    ],
    "state_free must close the operation impl table owner stored in the state",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_from_scanner_output_result"),
    [
        "field::get_ref scanner_output \"public_declarations\"",
        "selfhost_memo_trait_public_surface_normalizer_partial_input_items_result graph dependencies reexports public_declarations",
        "Result::Ok partial_items:",
        "selfhost_memo_trait_public_surface_hash_from_seed_table_and_partial_items_result seed_table &partial_items",
        "Result::Ok public_surface_hash:",
        "field::get_ref scanner_output \"operation_records\"",
        "selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_result module operation_records",
        "Result::Ok operation_impls:",
        "v::free partial_items",
        "Result::Ok SelfhostMemoTraitPublicImplSurfaceState public_surface_hash operation_impls",
        "Result::Err materializer_error:",
        "v::free partial_items",
        "OperationMaterializerRejected materializer_error",
        "Result::Err hash_error:",
        "v::free partial_items",
        "PublicSurfaceHashRejected hash_error",
        "Result::Err normalizer_error:",
        "PublicSurfaceNormalizerRejected normalizer_error",
    ],
    "from_scanner_output_result must normalize, hash, and materialize in order, and free partial items on all downstream exits",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_from_ast_records_result"),
    [
        "selfhost_memo_trait_public_impl_scanner_result ast records",
        "Result::Ok scanner_output:",
        "selfhost_memo_trait_public_impl_surface_from_scanner_output_result module graph dependencies reexports seed_table &scanner_output",
        "selfhost_memo_trait_public_impl_scanner_output_free scanner_output",
        "Result::Err scanner_error:",
        "ScannerRejected scanner_error",
    ],
    "from_ast_records_result must close scanner output after downstream success or rejection and must not continue after scanner rejection",
);
assertOrdered(
    source,
    [
        "selfhost_memo_trait_public_impl_surface_orchestrator_accepted_summary_eq",
        "selfhost_memo_trait_public_impl_surface_orchestrator_scanner_missing_ordinal_3_eq",
        "selfhost_memo_trait_public_impl_surface_orchestrator_normalizer_graph_node_unavailable_eq",
        "selfhost_memo_trait_public_impl_surface_orchestrator_materializer_untrusted_eq_rejected_eq",
        "selfhost_memo_trait_public_impl_surface_orchestrator_stage0_summary_eq",
    ],
    "orchestrator must expose lightweight typed assertion helpers for the doctest without forcing direct lower-module imports",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_surface_stage0_with_module"),
    [
        "selfhost_memo_trait_public_impl_surface_stage0_prepare_with_records module root type_id registry.eq_source eq_shape_hash",
        "SelfhostMemoTraitPublicImplScannerErrorKind::TypedRecordMissing 3",
        "PublicSurfaceNormalizerRejected SelfhostMemoTraitPublicSurfaceNormalizerErrorKind::GraphNodeUnavailable",
        "SelfhostMemoTraitOperationClassifierErrorKind::TraitSourceNotTrusted SelfhostMemoTraitOperationEvidenceKind::Eq",
        "OperationMaterializerRejected materializer_error",
        "selfhost_memo_trait_public_impl_surface_stage0_summary_new accepted scanner_rejected normalizer_rejected materializer_rejected",
    ],
    "stage0 must execute the accepted owner path and keep rejection fields as typed synthetic payloads to bound compile-time resource checks",
);
assert.doesNotMatch(
    code,
    /string_slice::|str_eq|hash32\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|mix\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|\.path\b|\.alias\b|\.span\b|\.lexeme\b|display_name|diagnostic_text|method_name|trait_name/,
    "orchestrator implementation must not fold source text, spans, paths, aliases, display names, lexemes, method names, or trait names into accepted material",
);
assert.doesNotMatch(
    source,
    /maxLine|line count limit|line-count limit|doc comment length cap|documentation length cap|行数制限|コメント量制限|ドキュメントコメント.{0,12}上限/,
    "orchestrator contract must not introduce line-count or documentation-comment length limits",
);

console.log("selfhost memo trait public impl surface orchestrator contract ok");
