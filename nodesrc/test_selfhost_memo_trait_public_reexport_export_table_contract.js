#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

function functionBlock(source, name) {
    const start = source.indexOf(`fn ${name}`);
    const pubStart = source.indexOf(`pub fn ${name}`);
    const actualStart = start === -1 ? pubStart : pubStart === -1 ? start : Math.min(start, pubStart);
    assert.notEqual(actualStart, -1, `missing function: ${name}`);
    const next = source.indexOf("\nfn ", actualStart + 1);
    const nextPub = source.indexOf("\npub fn ", actualStart + 1);
    const candidates = [next, nextPub].filter((index) => index !== -1);
    const end = candidates.length === 0 ? source.length : Math.min(...candidates);
    return source.slice(actualStart, end);
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_reexport_export_table.nepl";
const normalizerRelPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_normalizer.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";

const source = read(relPath);
const normalizer = read(normalizerRelPath);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const sourceCode = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assertOrdered(
    source,
    [
        "# check/module/memo_trait_public_reexport_export_table",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "re-export export table producer must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.ok(
    source.includes("caller supplied の typed export table hash を検査して evidence を作ります") &&
        source.includes("export name resolution、selective export set の具体展開、target module public surface materialization は別 stage の責務"),
    "docs must separate the stable producer contract from the current caller-supplied export table hash checkpoint",
);
assert.ok(
    source.includes("import path、alias、source span、display name、diagnostic text は hash material にしません"),
    "docs must exclude path, alias, spans, display names, and diagnostics from re-export hash material",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_reexport_export_table/,
    "re-export export table producer must remain facade-private until full public surface orchestration is designed",
);
assert.ok(
    normalizer.includes("pub struct SelfhostMemoTraitPublicSurfaceReExportEvidence:"),
    "producer must target the existing public surface normalizer re-export evidence record",
);
assert.ok(
    normalizer.includes("pub fn selfhost_memo_trait_public_surface_normalizer_partial_input_items_result %impure fn &SelfhostModuleGraph impure fn &Vec SelfhostMemoTraitPublicSurfaceDependencyEvidence impure fn &Vec SelfhostMemoTraitPublicSurfaceReExportEvidence impure fn &Vec SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence"),
    "producer output must be compatible with the existing arbitrary-length normalizer partial stream boundary",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicReExportProjectionKind:",
        "Wildcard",
        "Selective",
    ],
    "producer must distinguish wildcard and selective re-export projection domains",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicReExportExportTableInput:",
        "module_index %i32",
        "ordinal %Option i32",
        "projection_kind %SelfhostMemoTraitPublicReExportProjectionKind",
        "export_count %i32",
        "export_table_hash %Option i32",
        "dependency_public_surface_hash %Option i32",
    ],
    "producer input must carry graph index, explicit ordinal, projection kind, export count, export table hash, and target public surface hash as typed fields",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicReExportExportTableErrorKind:",
        "InputVectorAllocationFailed",
        "InputVectorReadFailed",
        "GraphNodeUnavailable",
        "ReExportOrdinalMissing",
        "ReExportOrdinalPlaceholder",
        "ReExportOrdinalNegative",
        "ReExportPayloadHashMissing",
        "ReExportPayloadHashPlaceholder",
        "DependencyPublicSurfaceHashMissing",
        "DependencyPublicSurfaceHashPlaceholder",
        "ExportCountNegative",
        "DerivedReExportPayloadHashPlaceholder",
        "DuplicateReExportOrdinal",
    ],
    "producer failures must keep vector, graph, ordinal, payload, dependency hash, and duplicate causes separate",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_reexport_export_table_payload_hash_result %fn SelfhostMemoTraitPublicReExportProjectionKind fn i32 fn i32 Result SelfhostMemoTraitPublicReExportExportTablePayloadHash SelfhostMemoTraitPublicReExportExportTableErrorKind"),
    "producer must expose a fixed typed payload hash boundary",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_reexport_export_table_evidence_result %fn &SelfhostModuleGraph fn SelfhostMemoTraitPublicReExportExportTableInput Result SelfhostMemoTraitPublicSurfaceReExportEvidence SelfhostMemoTraitPublicReExportExportTableErrorKind"),
    "producer must expose a graph-checked single evidence boundary",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_reexport_export_table_evidence_vec_result %impure fn &SelfhostModuleGraph impure fn &Vec SelfhostMemoTraitPublicReExportExportTableInput Result Vec SelfhostMemoTraitPublicSurfaceReExportEvidence SelfhostMemoTraitPublicReExportExportTableErrorKind"),
    "producer must expose an arbitrary-length evidence vector boundary",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_reexport_export_table_payload_hash_result"),
    [
        "selfhost_memo_trait_public_reexport_export_table_payload_seed_result some export_table_hash",
        "checked_export_table_hash",
        "selfhost_memo_trait_public_reexport_export_table_count_result export_count",
        "selfhost_memo_trait_public_reexport_export_table_schema_version",
        "selfhost_memo_trait_public_reexport_export_table_projection_kind_code projection_kind",
        "checked_count",
        "checked_export_table_hash",
        "DerivedReExportPayloadHashPlaceholder",
        "Result::Err payload_error:",
    ],
    "payload hash must reject placeholder export table hashes before using schema, projection kind, export count, and checked export table hash",
);
assert.ok(
    source.includes("public helper 単体でも accepted payload hash boundary を弱めません"),
    "payload hash docs must state that the public helper rejects placeholder export table hash seeds by itself",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_reexport_export_table_evidence_result"),
    [
        "selfhost_memo_trait_public_reexport_export_table_graph_node_result graph input.module_index",
        "selfhost_memo_trait_public_reexport_export_table_ordinal_result input.ordinal",
        "selfhost_memo_trait_public_reexport_export_table_payload_seed_result input.export_table_hash",
        "selfhost_memo_trait_public_reexport_export_table_dependency_hash_result input.dependency_public_surface_hash",
        "selfhost_memo_trait_public_reexport_export_table_payload_hash_result input.projection_kind input.export_count export_hash",
        "SelfhostMemoTraitPublicSurfaceReExportEvidence input.module_index ordinal payload.root_hash some dependency_hash",
    ],
    "single evidence producer must validate graph, explicit ordinal, export payload hash, dependency hash, and derived payload before constructing normalizer-compatible evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_reexport_export_table_seen_ordinal_loop"),
    [
        "v::get inputs idx",
        "selfhost_memo_trait_public_reexport_export_table_ordinal_result previous.ordinal",
        "eq previous_ordinal ordinal",
        "DuplicateReExportOrdinal",
        "InputVectorReadFailed",
    ],
    "duplicate ordinal scan must read typed ordinals from input records and fail closed on impossible short reads",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_reexport_export_table_unique_ordinal_result"),
    [
        "current_index",
        "ordinal",
        "selfhost_memo_trait_public_reexport_export_table_seen_ordinal_loop inputs 0 current_index ordinal",
    ],
    "unique ordinal helper must scan only earlier entries and must not derive the ordinal from Vec index",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_reexport_export_table_push_input_result"),
    [
        "selfhost_memo_trait_public_reexport_export_table_ordinal_result input.ordinal",
        "selfhost_memo_trait_public_reexport_export_table_unique_ordinal_result inputs current_index ordinal",
        "selfhost_memo_trait_public_reexport_export_table_evidence_result graph input",
        "selfhost_memo_trait_public_reexport_export_table_push_evidence_result items evidence",
    ],
    "vector producer must validate explicit ordinal uniqueness before pushing evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_reexport_export_table_evidence_vec_result"),
    [
        "v::new",
        "v::len inputs",
        "selfhost_memo_trait_public_reexport_export_table_evidence_vec_loop_result graph inputs items0 0 input_count",
    ],
    "evidence vector producer must allocate an owned output vector and fold borrowed input records",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_reexport_export_table_stage0_graph_result"),
    [
        "selfhost_vfs_add vfs1",
        "Result::Err _add_error:",
        "Result::Err SelfhostMemoTraitPublicReExportExportTableErrorKind::InputVectorAllocationFailed",
        "Result::Err _add_error:",
        "Result::Err SelfhostMemoTraitPublicReExportExportTableErrorKind::InputVectorAllocationFailed",
    ],
    "stage0 graph fixture must document that selfhost_vfs_add consumes VFS owners and must not reread moved owners on add-failure paths",
);
assert.ok(
    source.includes("`selfhost_vfs_add` は入力 VFS owner を消費します") &&
        source.includes("この関数は moved owner を再読しません"),
    "stage0 graph fixture docs must describe the selfhost_vfs_add owner-consumption contract",
);
assert.doesNotMatch(
    functionBlock(source, "selfhost_memo_trait_public_reexport_export_table_stage0_graph_result"),
    /selfhost_vfs_free vfs[01]\b/,
    "stage0 graph fixture must not free vfs0 or vfs1 after selfhost_vfs_add has consumed those owners",
);
assertOrdered(
    source,
    [
        "selfhost_memo_trait_public_reexport_export_table_stage0_wildcard_input",
        "SelfhostMemoTraitPublicReExportProjectionKind::Wildcard",
        "selfhost_memo_trait_public_reexport_export_table_stage0_selective_input",
        "SelfhostMemoTraitPublicReExportProjectionKind::Selective",
        "let differ %bool selfhost_memo_trait_public_reexport_export_table_stage0_compare_payload",
        "selfhost_memo_trait_public_reexport_export_table_stage0_duplicate_check_result graph",
    ],
    "stage0 must exercise wildcard, selective, payload difference, and duplicate ordinal rejection without storing an owned Vec in the summary",
);
assert.ok(
    source.includes("duplicate_ordinal_rejected %Result unit SelfhostMemoTraitPublicReExportExportTableErrorKind"),
    "stage0 summary must store duplicate rejection as Result unit so owner-backed Vec values do not escape through summary fields",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_reexport_export_table_stage0_duplicate_check_result"),
    [
        "selfhost_memo_trait_public_reexport_export_table_stage0_duplicate_result graph",
        "Result::Ok evidences:",
        "v::free evidences",
        "Result::Ok unit",
        "Result::Err duplicate_error:",
    ],
    "duplicate stage0 check must close unexpected successful Vec owners before returning a summary-friendly Result unit",
);
const publicSignatures = Array.from(source.matchAll(/^pub fn[^\n]+$/gm), (match) => match[0]).join("\n");
assert.doesNotMatch(
    publicSignatures,
    /SelfhostVirtualFileSystem|selfhost_vfs|Vfs|vfs|source_text|source_slice|SelfhostModuleAst|SelfhostDiagnostic\b/,
    "producer public API must not accept VFS, loader, source text, AST, or diagnostic payload authority",
);
assert.doesNotMatch(
    sourceCode,
    /hash32\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|mix\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|\.path\b|\.alias\b|\.span\b|\.lexeme\b|display_name|diagnostic_text/,
    "producer implementation must not fold source text, spans, paths, aliases, display names, lexemes, or diagnostic text into accepted hash material",
);
assert.doesNotMatch(
    sourceCode,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader)/,
    "producer must not import HIR, Resource IR, backend, proof store, or serialized proof artifact layers",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_reexport_export_table/,
    "checker-layer re-export export table producer must not be registered in the ty source list",
);
assert.doesNotMatch(
    sourceCode,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "producer policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait public reexport export table contract passed");
