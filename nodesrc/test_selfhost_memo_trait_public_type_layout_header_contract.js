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
    assert.notEqual(start, -1, `missing function: ${name}`);
    const next = source.indexOf("\nfn ", start + 1);
    const nextPub = source.indexOf("\npub fn ", start + 1);
    const candidates = [next, nextPub].filter((index) => index !== -1);
    const end = candidates.length === 0 ? source.length : Math.min(...candidates);
    return source.slice(start, end);
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_type_layout_header.nepl";
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
        "# check/module/memo_trait_public_type_layout_header",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "public type layout header producer must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.ok(
    source.includes("SelfhostMemoTraitStableNominalKey") &&
        source.includes("session-local `SelfhostNamedTypeId`、source span、display name を accepted hash material にしない"),
    "docs must require stable nominal key authority instead of session-local nominal ids or display/source data",
);
assert.ok(
    source.includes('#import "./memo_trait_stable_nominal_key_producer" as *'),
    "layout header stage0 must use the checker-layer stable nominal key producer rather than direct low-level key construction",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、path suffix、HIR、Resource IR、backend artifact、proof store record は accepted hash material に入りません"),
    "docs must exclude source, display, diagnostic, HIR, Resource IR, backend, and proof store authority",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_type_layout_header/,
    "type layout header producer must remain facade-private until full public surface orchestration is designed",
);
assert.ok(
    normalizer.includes("fn selfhost_memo_trait_public_surface_public_declaration_payload_hash_result %fn SelfhostMemoTraitPublicSurfacePublicDeclarationPayloadInput Result i32 SelfhostMemoTraitPublicSurfaceNormalizerErrorKind"),
    "normalizer must keep the common payload hash boundary in the implementation module",
);
assert.doesNotMatch(
    normalizer,
    /pub fn selfhost_memo_trait_public_surface_public_declaration_payload_hash_result\b/,
    "normalizer payload hash helper must stay internal",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicTypeLayoutHeaderKind:",
        "Struct",
        "Enum",
    ],
    "producer must distinguish struct and enum layout header domains",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicTypeLayoutHeaderInput:",
        "visibility %SelfhostModuleDeclarationVisibility",
        "declaration_ordinal %Option i32",
        "stable_nominal_key %Option SelfhostMemoTraitStableNominalKey",
        "kind %SelfhostMemoTraitPublicTypeLayoutHeaderKind",
        "type_arity %i32",
        "field_count %i32",
        "variant_count %i32",
    ],
    "producer input must carry visibility, ordinal, stable nominal key, kind, arity, field count, and variant count as typed fields",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicTypeLayoutHeaderErrorKind:",
        "PrivateVisibilityRejected",
        "DeclarationOrdinalMissing",
        "DeclarationOrdinalPlaceholder",
        "StableNominalKeyMissing",
        "StableNominalKeyPlaceholder",
        "StableNominalKeyArityMismatch",
        "TypeArityNegative",
        "FieldCountNegative",
        "VariantCountNegative",
        "StructVariantCountNonzero",
        "DerivedLayoutHeaderShapeHashPlaceholder",
        "PublicDeclarationPayloadRejected %SelfhostMemoTraitPublicSurfaceNormalizerErrorKind",
    ],
    "producer failures must keep visibility, ordinal, stable nominal key, count, shape hash, and payload hash failures separate",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_type_layout_header_shape_hash_result %fn SelfhostMemoTraitStableNominalKey fn SelfhostMemoTraitPublicTypeLayoutHeaderKind fn i32 fn i32 fn i32 Result SelfhostMemoTraitPublicTypeLayoutHeaderShapeHash SelfhostMemoTraitPublicTypeLayoutHeaderErrorKind"),
    "shape hash API must take stable nominal key, layout kind, arity, field count, and variant count",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_type_layout_header_payload_input_result %fn SelfhostMemoTraitPublicTypeLayoutHeaderInput Result SelfhostMemoTraitPublicSurfacePublicDeclarationPayloadInput SelfhostMemoTraitPublicTypeLayoutHeaderErrorKind"),
    "producer must expose a payload input adapter for normalizer composition",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_type_layout_header_evidence_result %fn SelfhostMemoTraitPublicTypeLayoutHeaderInput Result SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence SelfhostMemoTraitPublicTypeLayoutHeaderErrorKind"),
    "producer must expose an evidence API that returns the existing normalizer-compatible public declaration evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_type_layout_header_payload_input_result"),
    [
        "selfhost_memo_trait_public_type_layout_header_visibility_result",
        "selfhost_memo_trait_public_type_layout_header_stable_key_result",
        "selfhost_memo_trait_public_type_layout_header_shape_hash_result",
        "SelfhostMemoTraitPublicSurfacePublicDeclarationPayloadInput",
    ],
    "payload input producer must validate visibility/key, compute typed shape hash, and build normalizer payload input",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_type_layout_header_evidence_result"),
    [
        "selfhost_memo_trait_public_type_layout_header_ordinal_result",
        "selfhost_memo_trait_public_type_layout_header_payload_input_result",
        "selfhost_memo_trait_public_type_layout_header_payload_hash_result",
        "SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence payload_input.kind ordinal payload_hash",
    ],
    "evidence producer must validate ordinal, hash the typed normalizer payload input through its private payload boundary, and construct evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_type_layout_header_payload_hash_result"),
    [
        "input.stable_declaration_key_hash",
        "StableDeclarationKeyHashPlaceholder",
        "input.normalized_declaration_shape_hash",
        "NormalizedDeclarationShapeHashPlaceholder",
        "selfhost_memo_trait_public_type_layout_header_payload_kind_code input.kind",
        "selfhost_memo_trait_public_type_layout_header_payload_schema_code",
        "DerivedDeclarationPayloadHashPlaceholder",
    ],
    "private payload hash helper must use only typed payload fields and preserve normalizer placeholder errors",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_type_layout_header_stable_key_result"),
    [
        "key.nominal_key_hash",
        "StableNominalKeyPlaceholder",
        "key.type_arity type_arity",
        "StableNominalKeyArityMismatch",
    ],
    "stable key validation must reject placeholder nominal keys and arity mismatches",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_type_layout_header_stage0_key_result"),
    [
        "fn selfhost_memo_trait_public_type_layout_header_stage0_key_result %fn SelfhostMemoTraitStableNominalDeclarationKind fn i32 Result SelfhostMemoTraitStableNominalKey SelfhostMemoTraitStableNominalKeyProducerErrorKind \\kind\\type_arity:",
        "SelfhostMemoTraitStableNominalKeyProducerInput",
        "selfhost_memo_trait_stable_nominal_key_producer_input_new",
        "kind type_arity",
        "selfhost_memo_trait_stable_nominal_key_producer_result input",
    ],
    "layout header stage0 key construction must go through the stable nominal key producer with an explicit nominal declaration kind",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_type_layout_header_shape_hash_result"),
    [
        "selfhost_memo_trait_public_type_layout_header_counts_result",
        "stable_key.nominal_key_hash",
        "type_arity",
        "field_count",
        "variant_count",
        "DerivedLayoutHeaderShapeHashPlaceholder",
    ],
    "layout header shape hash must include stable nominal key hash, arity, field count, and variant count",
);
assertOrdered(
    source,
    [
        "struct_input",
        "SelfhostMemoTraitPublicTypeLayoutHeaderKind::Struct 0 2 0",
        "enum_input",
        "some enum_key",
        "SelfhostMemoTraitPublicTypeLayoutHeaderKind::Enum 0 2 2",
        "let differ %bool selfhost_memo_trait_public_type_layout_header_stage0_compare_payload",
    ],
    "stage0 must compare struct and enum layout headers using stable nominal keys from matching nominal declaration kinds",
);
assert.ok(
    source.includes("selfhost_memo_trait_public_type_layout_header_stage0_key_result SelfhostMemoTraitStableNominalDeclarationKind::Struct 0") &&
        source.includes("selfhost_memo_trait_public_type_layout_header_stage0_key_result SelfhostMemoTraitStableNominalDeclarationKind::Enum 0") &&
        source.includes("selfhost_memo_trait_public_type_layout_header_stage0_from_key struct_key enum_key generic_key"),
    "stage0 must build separate struct and enum keys through the stable nominal key producer",
);
assert.doesNotMatch(
    sourceCode,
    /hash32\s+(?:source|span|alias|display|diag|diagnostic|lexeme)|mix\s+(?:source|span|alias|display|diag|diagnostic|lexeme)|\.span\b|\.lexeme\b|display_name|diagnostic_text/,
    "producer implementation must not fold source text, spans, aliases, display names, lexemes, or diagnostic text into accepted hash material",
);
assert.doesNotMatch(
    sourceCode,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader)/,
    "producer must not import HIR, Resource IR, backend, proof store, or serialized proof artifact layers",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_type_layout_header/,
    "checker-layer public type layout header producer must not be registered in the ty source list",
);
assert.doesNotMatch(
    sourceCode,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "producer policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait public type layout header contract passed");
