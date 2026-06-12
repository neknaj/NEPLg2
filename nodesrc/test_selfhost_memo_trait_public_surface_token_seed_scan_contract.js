#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

function codeOnly(source) {
    return source
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_token_seed_scan.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";

const source = read(relPath);
const sourceCode = codeOnly(source);
const facade = read(facadeRelPath);
const tySource = read(tySourceRelPath);
const proofStore = read(proofStoreRelPath);

assert.match(
    source,
    /# check\/module\/memo_trait_public_surface_token_seed_scan[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:/,
    "shared token seed scan core must document purpose, contract, current limitations, and complexity",
);
assert.match(
    source,
    /この module は `memo_trait_public_surface_seed`、`memo_trait_public_surface_hash`、`memo_trait_public_surface_token_gate` を import しません[\s\S]*DAG の下位/,
    "shared token seed scan core docs must state the lower-DAG dependency boundary",
);
assert.doesNotMatch(
    source,
    /#import "\.\/memo_trait_public_surface_seed" as \*|#import "\.\/memo_trait_public_surface_hash" as \*|#import "\.\/memo_trait_public_surface_token_gate" as \*/,
    "shared token seed scan core must not import seed, hash, or token gate modules",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_surface_token_seed_scan|memo_trait_public_surface_token_gate/,
    "module checker facade must not re-export shared token seed scan core or token gate wrapper",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicSurfaceTokenSeedScanErrorKind:[\s\S]*ItemUnavailable[\s\S]*MemoKeyCandidateMissing[\s\S]*MemoValueCandidateMissing[\s\S]*MemoKeyCandidateDuplicate[\s\S]*MemoValueCandidateDuplicate[\s\S]*MemoKeyMethodSignatureRejected %SelfhostMemoTraitMethodSignatureErrorKind[\s\S]*MemoValueMethodSignatureRejected %SelfhostMemoTraitMethodSignatureErrorKind[\s\S]*ImportSurfaceUnsupported[\s\S]*PublicImplSurfaceUnsupported[\s\S]*DeclarationHeaderMissing/,
    "shared token seed scan core must own a dedicated typed error enum with method payload variants",
);
assert.match(
    source,
    /impl Clone for SelfhostMemoTraitPublicSurfaceTokenSeedScanErrorKind[\s\S]*impl Copy for SelfhostMemoTraitPublicSurfaceTokenSeedScanErrorKind/,
    "shared token seed scan error enum must participate in Clone and Copy like the surrounding selfhost error enums",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_token_seed_scan_signature_from_shape_result[\s\S]*evidence\.normalized_signature_hash[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_signature_from_method_result[\s\S]*evidence\.normalized_signature_hash/,
    "shared token seed scan core must extract only normalized signature hashes from marker and method evidence",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_token_seed_scan_method_signature_from_body_result[\s\S]*selfhost_memo_trait_method_signature_result source tokens kind body\.envelope[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_signature_from_method_result/,
    "shared token seed scan core must send method-bearing trait bodies through the method signature normalizer",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_token_seed_scan_method_error_result[\s\S]*MemoKeyMethodSignatureRejected method_error[\s\S]*MemoValueMethodSignatureRejected method_error/,
    "shared token seed scan core must preserve method normalizer errors as typed payloads",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_public_surface_token_seed_scan_item_result[\s\S]*selfhost_module_item_kind_declaration item\.kind[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_declaration_item_result[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_non_declaration_item_result/,
    "shared token seed scan item helper must own declaration classification before token-aware trait normalization",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_token_seed_scan_non_declaration_item_result[\s\S]*SelfhostModuleItemKind::ImportDirective:[\s\S]*ImportSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::UseDirective:[\s\S]*UseSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::PreludeDirective:[\s\S]*PreludeSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::NoPreludeDirective:[\s\S]*NoPreludeSurfaceUnsupported/,
    "shared token seed scan core must reject unsupported non-declaration public surface in the same item pass",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_token_seed_scan_declaration_item_result[\s\S]*SelfhostModuleDeclarationKind::Trait:[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_trait_item_result[\s\S]*SelfhostModuleDeclarationKind::Function:[\s\S]*PublicFunctionSurfaceUnsupported[\s\S]*SelfhostModuleDeclarationKind::Struct:[\s\S]*PublicStructSurfaceUnsupported[\s\S]*SelfhostModuleDeclarationKind::Enum:[\s\S]*PublicEnumSurfaceUnsupported[\s\S]*SelfhostModuleDeclarationKind::Impl:[\s\S]*PublicImplSurfaceUnsupported/,
    "shared token seed scan core must reject public non-trait declarations in the declaration helper",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_public_surface_token_seed_scan_module_result[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_module_loop[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_complete_result/,
    "shared token seed scan module helper must scan once and complete the fixed MemoKey/MemoValue seed table",
);
assert.doesNotMatch(
    sourceCode,
    /selfhost_memo_trait_source_identity_new|selfhost_memo_trait_definition_source_record_new:[\s\S]*\n\s+true/,
    "shared token seed scan core must not construct trusted source identities or accepted source records",
);
assert.doesNotMatch(
    sourceCode,
    /hash32\s+source|mix\s+source|hash32\s+span|mix\s+span|hash32\s+name|mix\s+name|hash32\s+path|mix\s+path/,
    "shared token seed scan core must not fold source text, spans, names, or paths into accepted authority",
);
assert.doesNotMatch(
    sourceCode,
    /proof_store|Resource IR|HIR|backend|codegen/,
    "shared token seed scan core must not depend on proof store, HIR, resource, backend, or codegen layers",
);
assert.doesNotMatch(
    tySource,
    /memo_trait_public_surface_token_seed_scan/,
    "core/ty memo trait source registry must not depend on the checker-layer token seed scan core",
);
assert.doesNotMatch(
    proofStore,
    /memo_trait_public_surface_token_seed_scan/,
    "proof store must not depend on the checker-layer token seed scan core",
);
assert.doesNotMatch(
    sourceCode,
    /line count|comment length|file size|500 行/,
    "shared token seed scan policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait public surface token seed scan contract passed");
