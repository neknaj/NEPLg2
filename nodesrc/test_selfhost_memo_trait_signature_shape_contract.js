#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_signature_shape.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const seedRelPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_seed.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";

const source = read(relPath);
const facade = read(facadeRelPath);
const seed = read(seedRelPath);
const proofStore = read(proofStoreRelPath);
const sourceCode = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/module\/memo_trait_signature_shape" as \*$/m,
    "module checker facade must expose the memo trait signature shape evidence boundary",
);
assert.match(
    source,
    /# check\/module\/memo_trait_signature_shape[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "signature shape module must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /現在の `SelfhostModuleDeclarationBody` は body 全体と先頭 expression の `SelfhostSyntaxRange` だけを持ち[\s\S]*trait method declaration list や method signature AST をまだ持ちません/,
    "docs must state that the current AST has ranges rather than parsed trait method signatures",
);
assert.match(
    source,
    /source text、source span、syntax range、lexeme、path suffix、diagnostic text は hash material に入りません/,
    "accepted marker signature hash must explicitly exclude source text, spans, ranges, lexemes, paths, and diagnostics",
);
assert.match(
    source,
    /body range を method declaration list へ分割する body segmenter[\s\S]*method name \/ type annotation \/ default body の signature normalization/,
    "docs must name the later body segmenter and method signature normalization work instead of pretending it exists",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitSignatureShapeErrorKind:[\s\S]*ItemKindMismatch[\s\S]*HeaderKindMismatch[\s\S]*PrivateVisibilityRejected %SelfhostMemoTraitSourceKind[\s\S]*BodyEvidenceMissing %SelfhostMemoTraitSourceKind[\s\S]*HeaderTypeAnnotationPresent %SelfhostMemoTraitSourceKind[\s\S]*HeaderLambdaHeaderPresent %SelfhostMemoTraitSourceKind[\s\S]*BodyEnvelopePresent %SelfhostMemoTraitSourceKind[\s\S]*BodyFirstExpressionPresent %SelfhostMemoTraitSourceKind/,
    "signature normalization failures must be typed enum variants for AST kind mismatch and each unsupported header/body range",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitSignatureShapeEvidence:[\s\S]*kind %SelfhostMemoTraitSourceKind[\s\S]*shape %SelfhostMemoTraitSignatureShape[\s\S]*normalized_signature_hash %Option i32/,
    "accepted signature evidence must keep source kind, shape, and optional normalized fingerprint as typed fields",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_signature_shape_result %fn SelfhostMemoTraitSourceKind fn SelfhostModuleDeclarationHeader fn SelfhostModuleItem Result SelfhostMemoTraitSignatureShapeEvidence SelfhostMemoTraitSignatureShapeErrorKind/,
    "normalization API must return a typed Result rather than bool or string diagnostics",
);
assert.match(
    source,
    /SelfhostModuleItemKind::TraitDecl:[\s\S]*selfhost_memo_trait_signature_header_result[\s\S]*item\.declaration_body[\s\S]*selfhost_memo_trait_signature_body_result[\s\S]*selfhost_memo_trait_signature_marker_hash_result/,
    "shape gate must verify trait item kind, header, body, and marker signature fingerprint before accepted evidence",
);
assert.match(
    source,
    /selfhost_memo_trait_signature_header_result[\s\S]*SelfhostModuleDeclarationVisibility::Public[\s\S]*SelfhostModuleDeclarationVisibility::Private:[\s\S]*PrivateVisibilityRejected/,
    "private memo traits must not become accepted marker signature shape evidence",
);
assert.match(
    source,
    /selfhost_memo_trait_signature_body_result[\s\S]*selfhost_syntax_range_is_nonempty body\.envelope[\s\S]*BodyEnvelopePresent[\s\S]*selfhost_syntax_range_is_nonempty body\.first_expression[\s\S]*BodyFirstExpressionPresent/,
    "body envelope and first-expression ranges must fail closed until method signature normalization exists",
);
assert.match(
    source,
    /selfhost_memo_trait_signature_header_result[\s\S]*selfhost_syntax_range_is_nonempty header\.type_annotation[\s\S]*HeaderTypeAnnotationPresent[\s\S]*selfhost_syntax_range_is_nonempty header\.lambda_header[\s\S]*HeaderLambdaHeaderPresent/,
    "header type annotation and lambda header ranges must fail closed in the marker-only stage",
);
assert.match(
    source,
    /selfhost_memo_trait_signature_marker_hash_result[\s\S]*domain\/version と memo trait kind だけ[\s\S]*selfhost_memo_trait_signature_kind_code/,
    "marker signature hash must be documented as domain/kind material",
);
assert.doesNotMatch(
    sourceCode,
    /str_slice|str_eq|source\s*:|source\s+%str|lexeme|path|display|diagnostic/i,
    "signature normalization implementation must not classify or hash source text, lexemes, paths, display names, or diagnostics",
);
assert.doesNotMatch(
    sourceCode,
    /hash32\s+span|mix\s+span|hash32\s+range|mix\s+range|hash32\s+source|mix\s+source|hash32\s+lexeme|mix\s+lexeme/i,
    "source spans, ranges, source strings, and lexemes must not become fingerprint authority",
);
assert.doesNotMatch(
    sourceCode,
    /method_signature_hash|parse_trait_method|method_signature_parser|trait_method_list/i,
    "this slice must not add a fake method signature parser before the AST exposes method declarations",
);
assert.doesNotMatch(
    sourceCode,
    /selfhost_memo_trait_source_identity_new|selfhost_memo_trait_trusted_source_registry|signature_available\s+true/,
    "signature normalization must not construct trusted source identities or registry records directly",
);
assert.doesNotMatch(
    proofStore,
    /memo_trait_signature_shape|SelfhostMemoTraitSignatureShape|SelfhostMemoTraitSignatureNormalization/,
    "proof store must not depend directly on checker-layer signature normalization evidence",
);
assert.match(
    seed,
    /trait body \/ method signature normalization が必要な trait は typed error で拒否/,
    "existing public surface seed boundary must still reject unnormalized trait bodies and signatures",
);
assert.doesNotMatch(
    sourceCode,
    /line count|comment length|file size|500 行/,
    "signature normalization policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait signature shape contract passed");
