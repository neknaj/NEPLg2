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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_function_signature.nepl";
const normalizerRelPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_normalizer.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const arenaRelPath = "stdlib/neplg2/core/ty/ty/arena.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";

const source = read(relPath);
const normalizer = read(normalizerRelPath);
const facade = read(facadeRelPath);
const arena = read(arenaRelPath);
const tySourceList = read(tySourceListRelPath);
const sourceCode = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    source,
    /# check\/module\/memo_trait_public_function_signature[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "public function signature producer must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /`fn void T` の 0 引数 function type と `fn unit T` の unit 引数 function type[\s\S]*hash material に反映/,
    "docs must explicitly preserve the void zero-argument versus unit-argument distinction",
);
assert.match(
    source,
    /source text、span、lexeme、display name、diagnostic text、path suffix、HIR、Resource IR、backend artifact、proof store record は accepted hash material に入りません/,
    "docs must exclude source, display, diagnostic, HIR, Resource IR, backend, and proof store authority",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_function_signature/,
    "function signature producer must remain facade-private until full public surface orchestration is designed",
);
assert.match(
    normalizer,
    /fn selfhost_memo_trait_public_surface_public_declaration_payload_hash_result %fn SelfhostMemoTraitPublicSurfacePublicDeclarationPayloadInput Result i32 SelfhostMemoTraitPublicSurfaceNormalizerErrorKind/,
    "normalizer must keep the common payload hash boundary in the implementation module so upstream producers can share it without facade exposure",
);
assert.doesNotMatch(
    normalizer,
    /pub fn selfhost_memo_trait_public_surface_public_declaration_payload_hash_result\b/,
    "normalizer must not expose the payload hash helper through the module checker public facade",
);
assert.doesNotMatch(
    normalizer,
    /pub fn selfhost_memo_trait_public_surface_public_declaration_evidence_from_payload_result\b/,
    "normalizer must still keep the evidence adapter internal until orchestration owns the final evidence stream",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitPublicFunctionSignatureInput:[\s\S]*visibility %SelfhostModuleDeclarationVisibility[\s\S]*declaration_ordinal %Option i32[\s\S]*stable_function_key_hash %Option i32[\s\S]*function_type %SelfhostTypeId[\s\S]*effect %SelfhostEffectKind/,
    "producer input must carry visibility, ordinal, stable key, function type, and effect as typed fields",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicFunctionSignatureTypeHashErrorKind:[\s\S]*MissingTypeRecord[\s\S]*FunctionSignatureTypeNotFunction[\s\S]*MissingFunctionArgument[\s\S]*MissingFunctionResult[\s\S]*MissingAppliedArgument[\s\S]*InvalidTypeParameterBinding[\s\S]*NominalKeyRejected %SelfhostMemoTraitCanonicalFingerprintErrorKind[\s\S]*TraversalFuelExhausted[\s\S]*DerivedTypeHashPlaceholder[\s\S]*DerivedSignatureShapeHashPlaceholder/,
    "type shape failures must be typed enum variants with nominal key payload preservation",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicFunctionSignatureErrorKind:[\s\S]*PrivateVisibilityRejected[\s\S]*DeclarationOrdinalMissing[\s\S]*DeclarationOrdinalPlaceholder[\s\S]*StableFunctionKeyMissing[\s\S]*StableFunctionKeyPlaceholder[\s\S]*TypeHashRejected %SelfhostMemoTraitPublicFunctionSignatureTypeHashErrorKind[\s\S]*PublicDeclarationPayloadRejected %SelfhostMemoTraitPublicSurfaceNormalizerErrorKind/,
    "producer failures must keep visibility, ordinal, key, type hash, and payload hash failures separate",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_public_function_signature_shape_hash_result %fn &SelfhostTypeArena fn &SelfhostMemoTraitStableNominalKeyTable fn SelfhostTypeId fn SelfhostEffectKind Result SelfhostMemoTraitPublicFunctionSignatureShapeHash SelfhostMemoTraitPublicFunctionSignatureTypeHashErrorKind/,
    "shape hash API must take typed arena, stable nominal table, root function type, and effect",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_public_function_signature_payload_input_result %fn &SelfhostTypeArena fn &SelfhostMemoTraitStableNominalKeyTable fn SelfhostMemoTraitPublicFunctionSignatureInput Result SelfhostMemoTraitPublicSurfacePublicDeclarationPayloadInput SelfhostMemoTraitPublicFunctionSignatureErrorKind/,
    "producer must expose a payload input adapter for normalizer composition",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_public_function_signature_evidence_result %fn &SelfhostTypeArena fn &SelfhostMemoTraitStableNominalKeyTable fn SelfhostMemoTraitPublicFunctionSignatureInput Result SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence SelfhostMemoTraitPublicFunctionSignatureErrorKind/,
    "producer must expose an evidence API that returns the existing normalizer-compatible public declaration evidence",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_public_function_signature_payload_input_result"),
    /selfhost_memo_trait_public_function_signature_visibility_result[\s\S]*selfhost_memo_trait_public_function_signature_stable_key_result[\s\S]*selfhost_memo_trait_public_function_signature_shape_hash_result[\s\S]*SelfhostMemoTraitPublicSurfacePublicDeclarationKind::Function/,
    "payload input producer must validate visibility/key, compute typed shape hash, and set declaration kind to Function",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_public_function_signature_evidence_result"),
    /selfhost_memo_trait_public_function_signature_ordinal_result[\s\S]*selfhost_memo_trait_public_function_signature_payload_input_result[\s\S]*selfhost_memo_trait_public_function_signature_payload_hash_result[\s\S]*SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence SelfhostMemoTraitPublicSurfacePublicDeclarationKind::Function ordinal payload_hash/,
    "evidence producer must validate ordinal, hash the typed normalizer payload input through its private payload boundary, and construct function evidence",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_public_function_signature_payload_hash_result"),
    /input\.stable_declaration_key_hash[\s\S]*StableDeclarationKeyHashPlaceholder[\s\S]*input\.normalized_declaration_shape_hash[\s\S]*NormalizedDeclarationShapeHashPlaceholder[\s\S]*selfhost_memo_trait_public_function_signature_payload_kind_code input\.kind[\s\S]*selfhost_memo_trait_public_function_signature_payload_schema_code[\s\S]*DerivedDeclarationPayloadHashPlaceholder/,
    "private payload hash helper must use only typed payload fields and preserve normalizer placeholder errors",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_public_function_signature_function_type_hash_result"),
    /selfhost_type_arena_function_arg_count[\s\S]*selfhost_type_arena_function_result[\s\S]*selfhost_memo_trait_public_function_signature_function_args_hash_result[\s\S]*selfhost_memo_trait_public_function_signature_type_hash_with_fuel_result[\s\S]*FunctionSignatureTypeNotFunction/,
    "function type hash must reject non-functions and recursively include argument and result type hashes",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_public_function_signature_named_type_hash_result"),
    /selfhost_memo_trait_stable_nominal_key_table_find_result[\s\S]*key\.nominal_key_hash[\s\S]*selfhost_memo_trait_public_function_signature_nominal_error/,
    "named type hash must go through the stable nominal key table rather than session-local ids",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_public_function_signature_applied_args_hash_from_range_result"),
    /selfhost_type_arena_type_arg_at[\s\S]*selfhost_memo_trait_public_function_signature_type_hash_with_fuel_result/,
    "applied type argument hashing must use the arena accessor instead of direct field-name coupling",
);
assert.match(
    arena,
    /pub fn selfhost_type_arena_type_arg_at %fn &SelfhostTypeArena fn i32 Option SelfhostTypeId[\s\S]*lt arg_index 0[\s\S]*v::get type_args arg_index/,
    "TypeArena must expose a typed applied-argument table accessor for producer modules",
);
assert.match(
    source,
    /zero_input[\s\S]*SelfhostModuleDeclarationVisibility::Public \(some 1\) \(some 4101\) zero_function_id SelfhostEffectKind::Pure[\s\S]*unit_input[\s\S]*SelfhostModuleDeclarationVisibility::Public \(some 2\) \(some 4101\) unit_function_id SelfhostEffectKind::Pure[\s\S]*let differ %bool selfhost_memo_trait_public_function_signature_stage0_compare_payload/,
    "stage0 must compare zero-argument and unit-argument function signatures using the same stable key",
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
    /memo_trait_public_function_signature/,
    "checker-layer public function signature producer must not be registered in the ty source list",
);
assert.doesNotMatch(
    sourceCode,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "producer policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait public function signature contract passed");
