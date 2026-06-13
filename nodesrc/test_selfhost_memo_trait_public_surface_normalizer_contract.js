#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_normalizer.nepl";
const hashRelPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_hash.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";

const source = read(relPath);
const hashSource = read(hashRelPath);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const sourceCode = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/module\/memo_trait_public_surface_normalizer" as \*$/m,
    "module checker facade must expose the stable public surface normalizer boundary",
);
assert.match(
    source,
    /# check\/module\/memo_trait_public_surface_normalizer[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "normalizer module must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /graph path、import alias、source span、source text、diagnostic text、display name は hash input に入りません/,
    "normalizer docs must explicitly keep graph path, alias, spans, source text, diagnostics, and display names out of hash input",
);
assert.match(
    source,
    /proof store、HIR、Resource IR、backend、serialized artifact を読みません/,
    "normalizer docs must keep proof store, HIR, Resource IR, backend, and serialized artifacts outside this boundary",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitPublicSurfaceDependencyEvidence:[\s\S]*module_index %i32[\s\S]*ordinal %i32[\s\S]*payload_hash %i32[\s\S]*dependency_public_surface_hash %Option i32/,
    "dependency evidence must carry graph node index, ordinal, stable payload hash, and typed dependency public surface hash",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitPublicSurfaceReExportEvidence:[\s\S]*module_index %i32[\s\S]*ordinal %i32[\s\S]*payload_hash %i32[\s\S]*dependency_public_surface_hash %Option i32/,
    "re-export evidence must carry graph node index, ordinal, stable payload hash, and typed target public surface hash",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicSurfacePublicDeclarationKind:[\s\S]*Function[\s\S]*Struct[\s\S]*Enum[\s\S]*Impl/,
    "public declaration evidence must use a typed declaration kind enum",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicSurfaceGraphRejectionKind:[\s\S]*OutOfMemory[\s\S]*MissingModule[\s\S]*Cycle[\s\S]*Internal[\s\S]*UnexpectedDiagnostic/,
    "graph diagnostic payloads must be normalized to typed graph rejection kinds",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicSurfaceNormalizerErrorKind:[\s\S]*GraphRejected %SelfhostMemoTraitPublicSurfaceGraphRejectionKind[\s\S]*GraphNodeUnavailable[\s\S]*InputVectorAllocationFailed[\s\S]*InputVectorReadFailed[\s\S]*DependencyPublicSurfaceHashMissing[\s\S]*DependencyPublicSurfaceHashPlaceholder[\s\S]*StablePayloadHashPlaceholder[\s\S]*HashRejected %SelfhostMemoTraitPublicSurfaceHashErrorKind/,
    "normalizer failures must be typed enum variants instead of bool or string collapse",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_public_surface_normalizer_partial_input_items_result %impure fn &SelfhostModuleGraph impure fn &Vec SelfhostMemoTraitPublicSurfaceDependencyEvidence impure fn &Vec SelfhostMemoTraitPublicSurfaceReExportEvidence impure fn &Vec SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence Result Vec SelfhostMemoTraitPublicSurfaceHashInputItem SelfhostMemoTraitPublicSurfaceNormalizerErrorKind/,
    "normalizer must expose a graph-authority arbitrary-length partial input stream producer rather than an incomplete final hash or fixed smoke shape",
);
assert.doesNotMatch(
    source,
    /pub fn selfhost_memo_trait_public_surface_normalizer_partial_input_items_result[^\n]*SelfhostMemoTraitPublicSurfaceDependencyEvidence impure fn SelfhostMemoTraitPublicSurfaceReExportEvidence/,
    "normalizer public producer must not take one fixed dependency/re-export/declaration tuple",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_normalizer_push_dependency_vec_result[\s\S]*v::get dependencies idx[\s\S]*selfhost_memo_trait_public_surface_normalizer_push_dependency_result graph items evidence[\s\S]*InputVectorReadFailed/,
    "dependency vector production must read arbitrary-length borrowed evidence and fail closed on impossible short reads",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_normalizer_dependency_input_item_result[\s\S]*selfhost_module_graph_node_at graph evidence\.module_index[\s\S]*SelfhostMemoTraitPublicSurfaceHashInputKind::DependencyModule/,
    "dependency input production must check graph node existence before creating a dependency module hash input item",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_normalizer_push_reexport_vec_result[\s\S]*v::get reexports idx[\s\S]*selfhost_memo_trait_public_surface_normalizer_push_reexport_result graph items evidence[\s\S]*InputVectorReadFailed/,
    "re-export vector production must read arbitrary-length borrowed evidence and fail closed on impossible short reads",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_normalizer_reexport_input_item_result[\s\S]*selfhost_module_graph_node_at graph evidence\.module_index[\s\S]*SelfhostMemoTraitPublicSurfaceHashInputKind::ReExport/,
    "re-export input production must check graph node existence before creating a re-export hash input item",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_normalizer_push_declaration_vec_result[\s\S]*v::get declarations idx[\s\S]*selfhost_memo_trait_public_surface_normalizer_push_declaration_result items evidence[\s\S]*InputVectorReadFailed/,
    "public declaration vector production must read arbitrary-length borrowed evidence and fail closed on impossible short reads",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_normalizer_declaration_input_item_result[\s\S]*PublicFunction[\s\S]*PublicStruct[\s\S]*PublicEnum[\s\S]*PublicImpl/,
    "public declaration input production must cover function, struct, enum, and impl item kinds",
);
assert.doesNotMatch(
    source,
    /pub fn selfhost_memo_trait_public_surface_normalizer_hash_from_graph_evidence_result|selfhost_memo_trait_public_surface_normalizer_hash_owned_items_result/,
    "normalizer must not expose or implement an incomplete final hash for partial public surface evidence",
);
assert.doesNotMatch(
    hashSource,
    /pub fn selfhost_memo_trait_public_surface_hash_input_items_result\b/,
    "hash vector fold gate must not be exposed through the module checker facade before full input composition is designed",
);
assert.match(
    hashSource,
    /\nfn selfhost_memo_trait_public_surface_hash_input_items_result %fn &Vec SelfhostMemoTraitPublicSurfaceHashInputItem Result i32 SelfhostMemoTraitPublicSurfaceHashErrorKind/,
    "hash module may keep a module-internal typed input vector fold gate for future full input composition",
);
assert.match(
    hashSource,
    /selfhost_memo_trait_public_surface_hash_input_items_loop[\s\S]*v::get items idx[\s\S]*selfhost_memo_trait_public_surface_hash_input_accumulator_push_result state item[\s\S]*selfhost_memo_trait_public_surface_hash_input_accumulator_finish_result state/,
    "hash vector gate must route through the existing accumulator rather than duplicating fold semantics",
);
assert.match(
    source,
    /summary\.accepted_item_count[\s\S]*summary\.dependency_hash_missing_rejected SelfhostMemoTraitPublicSurfaceNormalizerErrorKind::DependencyPublicSurfaceHashMissing[\s\S]*summary\.dependency_hash_placeholder_rejected SelfhostMemoTraitPublicSurfaceNormalizerErrorKind::DependencyPublicSurfaceHashPlaceholder[\s\S]*summary\.graph_node_missing_rejected SelfhostMemoTraitPublicSurfaceNormalizerErrorKind::GraphNodeUnavailable[\s\S]*summary\.stable_payload_placeholder_rejected SelfhostMemoTraitPublicSurfaceNormalizerErrorKind::StablePayloadHashPlaceholder/,
    "stage0 doctest must assert accepted graph path, dependency hash rejection, graph node rejection, and stable payload placeholder rejection",
);
const publicSignatures = Array.from(source.matchAll(/^pub fn[^\n]+$/gm), (match) => match[0]).join("\n");
assert.doesNotMatch(
    publicSignatures,
    /SelfhostVirtualFileSystem|selfhost_vfs|Vfs|vfs|source_text|source_slice|SelfhostModuleAst|SelfhostDiagnostic\b/,
    "normalizer public API must not accept VFS, loader, source text, AST, or diagnostic payload authority",
);
assert.doesNotMatch(
    sourceCode,
    /hash32\s+(?:source|span|path|alias|display|diag|diagnostic)|mix\s+(?:source|span|path|alias|display|diag|diagnostic)|\.path\b|\.alias\b|\.span\b|\.lexeme\b|display_name|diagnostic_text/,
    "normalizer implementation must not fold source text, spans, paths, aliases, display names, lexemes, or diagnostic text into accepted hash material",
);
assert.doesNotMatch(
    sourceCode,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader)/,
    "normalizer must not import HIR, Resource IR, backend, proof store, or serialized proof artifact layers",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_surface_normalizer/,
    "checker-layer public surface normalizer must not be registered in the ty source list",
);
assert.doesNotMatch(
    sourceCode,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "normalizer policy must not introduce line-count or doc-comment-length restrictions in English or Japanese",
);

console.log("selfhost memo trait public surface normalizer contract passed");
