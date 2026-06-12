#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

function sectionBetween(source, start, end) {
    const startIndex = source.indexOf(start);
    assert.notEqual(startIndex, -1, `missing section start: ${start}`);
    const endIndex = source.indexOf(end, startIndex + start.length);
    assert.notEqual(endIndex, -1, `missing section end: ${end}`);
    return source.slice(startIndex, endIndex);
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_hash.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tokenGateRelPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_token_gate.nepl";
const tySourceRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";

const source = read(relPath);
const facade = read(facadeRelPath);
const tokenGateSource = read(tokenGateRelPath);
const tySource = read(tySourceRelPath);
const proofStore = read(proofStoreRelPath);
const sourceCode = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/module\/memo_trait_public_surface_hash" as \*$/m,
    "module checker facade must expose the public surface hash materializer",
);
assert.match(
    source,
    /# check\/module\/memo_trait_public_surface_hash[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "public surface hash module must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /source text、span、display name、path suffix、diagnostic text を accepted fingerprint authority にせず[\s\S]*kind、visibility、declaration ordinal、normalized signature seed を使います/,
    "public surface hash docs must exclude source spelling and spans from accepted fingerprint authority",
);
assert.match(
    source,
    /token-aware API が計算する public surface hash は、local `MemoKey` \/ `MemoValue` marker trait pair と method-bearing trait pair に対応[\s\S]*AST-only 互換 API は token authority がないため marker trait pair だけに限定[\s\S]*import \/ use \/ prelude \/ public non-trait declaration は[\s\S]*拒否/,
    "hash materializer must document token-aware method-bearing support and fail-closed unsupported full public surface cases",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitPublicSurfaceHashMaterialization:[\s\S]*module_seed %SelfhostMemoTraitStableSourceModuleSeed[\s\S]*seed_table %SelfhostMemoTraitStableSourceSeedTable/,
    "hash materialization output must carry both the derived module seed and the seed table that produced it",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicSurfaceHashErrorKind:[\s\S]*ModuleIdentityFingerprintMissing[\s\S]*ModuleIdentityFingerprintPlaceholder[\s\S]*ImportSurfaceUnsupported[\s\S]*UseSurfaceUnsupported[\s\S]*PreludeSurfaceUnsupported[\s\S]*NoPreludeSurfaceUnsupported[\s\S]*PublicFunctionSurfaceUnsupported[\s\S]*PublicStructSurfaceUnsupported[\s\S]*PublicEnumSurfaceUnsupported[\s\S]*PublicImplSurfaceUnsupported[\s\S]*PublicSurfaceSeedRejected %SelfhostMemoTraitPublicSurfaceSeedErrorKind[\s\S]*MemoKeySeedMissing[\s\S]*MemoValueSeedMissing[\s\S]*MemoKeyPrivateVisibility[\s\S]*MemoValuePrivateVisibility[\s\S]*PublicSurfaceHashPlaceholder/,
    "hash failures must be typed enum variants covering identity, unsupported surface, seed scan, field, visibility, and derived hash errors",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicSurfaceHashRegistryErrorKind:[\s\S]*CandidateScanRejected %SelfhostMemoTraitDefinitionScanErrorKind[\s\S]*PublicSurfaceHashRejected %SelfhostMemoTraitPublicSurfaceHashErrorKind[\s\S]*SeedRegistryRejected %SelfhostMemoTraitStableSourceSeedRegistryErrorKind/,
    "registry wrapper must keep candidate scan, hash materializer, and seed registry errors separate",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_public_surface_hash_materialize_result %fn Option i32 fn str fn &SelfhostModuleAst Result SelfhostMemoTraitPublicSurfaceHashMaterialization SelfhostMemoTraitPublicSurfaceHashErrorKind/,
    "hash materialize API must take a caller-supplied module identity option and return a typed Result payload",
);
assert.match(
    source,
    /\nfn selfhost_memo_trait_public_surface_hash_materialize_with_tokens_result %impure fn Option i32 impure fn str impure fn &Vec SelfhostToken impure fn &SelfhostModuleAst Result SelfhostMemoTraitPublicSurfaceHashMaterialization SelfhostMemoTraitPublicSurfaceHashErrorKind/,
    "token-aware hash materialize gate must take parser tokens, AST, and a caller-supplied module identity option",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_result %fn Option i32 fn str fn &SelfhostModuleAst Result SelfhostMemoTraitTrustedSourceRegistry SelfhostMemoTraitPublicSurfaceHashRegistryErrorKind/,
    "registry convenience API must expose typed hash path errors",
);
assert.match(
    source,
    /\nfn selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_with_tokens_result %impure fn Option i32 impure fn str impure fn &Vec SelfhostToken impure fn &SelfhostModuleAst Result SelfhostMemoTraitTrustedSourceRegistry SelfhostMemoTraitPublicSurfaceHashRegistryErrorKind/,
    "token-aware registry gate must expose typed hash path errors",
);
assert.doesNotMatch(
    source,
    /pub fn selfhost_memo_trait_public_surface_hash_materialize_with_tokens_result|pub fn selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_with_tokens_result/,
    "token-aware hash gates must not be exported through the module checker facade until a stable orchestration boundary is designed",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_result[\s\S]*selfhost_memo_trait_definition_source_table_scan_module_result[\s\S]*selfhost_memo_trait_public_surface_hash_materialize_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_seed_evidence_result/,
    "registry path must go through scanner, hash materializer, seed evidence producer, and existing stable registry gate",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_with_tokens_result[\s\S]*selfhost_memo_trait_definition_source_table_scan_module_result[\s\S]*selfhost_memo_trait_public_surface_hash_materialize_with_tokens_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_seed_evidence_result/,
    "token-aware registry path must go through scanner, token-aware hash materializer, seed evidence producer, and existing stable registry gate",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_materialize_result[\s\S]*selfhost_memo_trait_public_surface_hash_validate_module_identity_result[\s\S]*selfhost_memo_trait_public_surface_hash_scan_module_result[\s\S]*selfhost_memo_trait_public_surface_hash_from_seed_table_result/,
    "materializer must validate module identity, run the hash-owned single-pass marker seed scan that rejects unsupported surfaces, and only then fold a public surface hash",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_materialize_with_tokens_result[\s\S]*selfhost_memo_trait_public_surface_hash_validate_module_identity_result[\s\S]*selfhost_memo_trait_public_surface_hash_scan_module_with_tokens_result[\s\S]*selfhost_memo_trait_public_surface_hash_from_seed_table_result/,
    "token-aware materializer must validate module identity, run the hash-owned single-pass token-aware seed scan that rejects unsupported surfaces, and only then fold a public surface hash",
);
assert.match(
    source,
    /#import "\.\/memo_trait_public_surface_token_gate" as \*/,
    "hash materializer must use the facade-external token-aware item gate instead of exposing token-aware hash APIs through the module facade",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_scan_item_result[\s\S]*SelfhostModuleItemKind::TraitDecl:[\s\S]*selfhost_memo_trait_public_surface_hash_scan_trait_item_result[\s\S]*SelfhostModuleItemKind::ImportDirective:[\s\S]*ImportSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::UseDirective:[\s\S]*UseSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::PreludeDirective:[\s\S]*PreludeSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::NoPreludeDirective:[\s\S]*NoPreludeSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::FunctionDecl:[\s\S]*PublicFunctionSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::StructDecl:[\s\S]*PublicStructSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::EnumDecl:[\s\S]*PublicEnumSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::ImplDecl:[\s\S]*PublicImplSurfaceUnsupported/,
    "hash-owned AST item loop must combine unsupported public surface rejection with marker trait seed accumulation",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_scan_module_with_tokens_loop[\s\S]*selfhost_memo_trait_public_surface_token_gate_scan_item_result[\s\S]*selfhost_memo_trait_public_surface_hash_error_from_seed_scan_error/,
    "hash-owned token-aware loop must reuse only the token gate item helper and map typed seed errors into hash errors",
);
assert.match(
    tokenGateSource,
    /selfhost_memo_trait_public_surface_token_gate_non_declaration_item_result[\s\S]*SelfhostModuleItemKind::ImportDirective:[\s\S]*ImportSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::UseDirective:[\s\S]*UseSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::PreludeDirective:[\s\S]*PreludeSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::NoPreludeDirective:[\s\S]*NoPreludeSurfaceUnsupported/,
    "token gate must keep unsupported non-declaration surface rejection in its non-declaration helper",
);
assert.match(
    tokenGateSource,
    /selfhost_memo_trait_public_surface_token_gate_scan_declaration_item_result[\s\S]*SelfhostModuleDeclarationKind::Trait:[\s\S]*selfhost_memo_trait_public_surface_token_gate_scan_trait_item_result[\s\S]*SelfhostModuleDeclarationKind::Function:[\s\S]*PublicFunctionSurfaceUnsupported[\s\S]*SelfhostModuleDeclarationKind::Struct:[\s\S]*PublicStructSurfaceUnsupported[\s\S]*SelfhostModuleDeclarationKind::Enum:[\s\S]*PublicEnumSurfaceUnsupported[\s\S]*SelfhostModuleDeclarationKind::Impl:[\s\S]*PublicImplSurfaceUnsupported/,
    "token gate must keep public non-trait declaration rejection in its declaration helper",
);
assert.match(
    tokenGateSource,
    /pub fn selfhost_memo_trait_public_surface_token_gate_scan_item_result[\s\S]*selfhost_module_item_kind_declaration item\.kind[\s\S]*Option::Some declaration_kind:[\s\S]*selfhost_memo_trait_public_surface_token_gate_scan_declaration_item_result[\s\S]*Option::None:[\s\S]*selfhost_memo_trait_public_surface_token_gate_non_declaration_item_result/,
    "token gate item scan must dispatch through the shared declaration classifier before token-aware trait normalization",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_schema_code[\s\S]*hash 材料の構造を変える場合は、この schema code も更新/,
    "public surface hash must carry an explicit schema/domain code",
);

const hashSection = sectionBetween(
    source,
    "selfhost_memo_trait_public_surface_hash_from_seed_table_result",
    "selfhost_memo_trait_public_surface_hash_validate_module_identity_result",
);
const hashCode = hashSection
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");
assert.doesNotMatch(
    hashCode,
    /\bsource\b|source\.|\bspan\b|span\.|\brange\b|range\.|\bname\b|name\.|str_slice|\bpath\b|path\.|\bdisplay\b|\bdiagnostic\b/i,
    "public surface hash folding must depend only on typed seed fields, not source text, spans, names, paths, display text, or diagnostics",
);
assert.match(
    hashSection,
    /source text と span は引数に取りません[\s\S]*seed table の typed field だけ/,
    "hash folding contract must explicitly state that source text and span are not hash authority",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_error_from_seed_scan_error[\s\S]*ImportSurfaceUnsupported:[\s\S]*SelfhostMemoTraitPublicSurfaceHashErrorKind::ImportSurfaceUnsupported[\s\S]*UseSurfaceUnsupported:[\s\S]*SelfhostMemoTraitPublicSurfaceHashErrorKind::UseSurfaceUnsupported[\s\S]*PreludeSurfaceUnsupported:[\s\S]*SelfhostMemoTraitPublicSurfaceHashErrorKind::PreludeSurfaceUnsupported[\s\S]*NoPreludeSurfaceUnsupported:[\s\S]*SelfhostMemoTraitPublicSurfaceHashErrorKind::NoPreludeSurfaceUnsupported[\s\S]*PublicFunctionSurfaceUnsupported:[\s\S]*SelfhostMemoTraitPublicSurfaceHashErrorKind::PublicFunctionSurfaceUnsupported[\s\S]*PublicStructSurfaceUnsupported:[\s\S]*SelfhostMemoTraitPublicSurfaceHashErrorKind::PublicStructSurfaceUnsupported[\s\S]*PublicEnumSurfaceUnsupported:[\s\S]*SelfhostMemoTraitPublicSurfaceHashErrorKind::PublicEnumSurfaceUnsupported[\s\S]*PublicImplSurfaceUnsupported:[\s\S]*SelfhostMemoTraitPublicSurfaceHashErrorKind::PublicImplSurfaceUnsupported[\s\S]*DeclarationHeaderMissing:[\s\S]*SelfhostMemoTraitPublicSurfaceHashErrorKind::DeclarationHeaderMissing/,
    "hash materializer must preserve direct unsupported-surface error kinds after moving the rejection into the seed scan",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_public_surface_hash_supported_module_loop|selfhost_memo_trait_public_surface_hash_supported_item_result|selfhost_memo_trait_public_surface_hash_public_decl_supported_result/,
    "hash materializer must not keep a separate unsupported-surface traversal after seed scan owns that check",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_public_surface_seed_scan_module_result|selfhost_memo_trait_public_surface_token_gate_seed_table_with_tokens_result/,
    "hash materializer must not call whole-module seed scanners after owning the materialization loop",
);
assert.match(
    source,
    /method_surface_accepted_registry[\s\S]*selfhost_memo_trait_public_surface_hash_stage0_method_registry_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_with_tokens_result/,
    "stage0 smoke must cover token-aware method-bearing trait hash registry acceptance",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_key_seed_result[\s\S]*selfhost_memo_trait_source_kind_eq seed\.kind SelfhostMemoTraitSourceKind::MemoKeyTrait[\s\S]*seed\.declaration_ordinal[\s\S]*seed\.normalized_signature_hash[\s\S]*selfhost_memo_trait_public_surface_hash_nonzero_result/,
    "MemoKey hash part must check source kind, ordinal, signature, visibility, and derived nonzero hash",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_value_seed_result[\s\S]*selfhost_memo_trait_source_kind_eq seed\.kind SelfhostMemoTraitSourceKind::MemoValueTrait[\s\S]*seed\.declaration_ordinal[\s\S]*seed\.normalized_signature_hash[\s\S]*selfhost_memo_trait_public_surface_hash_nonzero_result/,
    "MemoValue hash part must check source kind, ordinal, signature, visibility, and derived nonzero hash",
);
assert.doesNotMatch(
    sourceCode,
    /selfhost_memo_trait_source_identity_new/,
    "hash materializer must not construct accepted source identities directly",
);
assert.doesNotMatch(
    sourceCode,
    /selfhost_memo_trait_definition_source_record_new:[\s\S]*\n\s+true/,
    "hash materializer must not directly create signature_available=true source records",
);
assert.doesNotMatch(
    sourceCode,
    /hash32\s+source|mix\s+source|hash32\s+span|mix\s+span|hash32\s+name|mix\s+name|hash32\s+path|mix\s+path/,
    "source text, spans, names, and paths must not be folded into accepted public surface hash material",
);
assert.doesNotMatch(
    tySource,
    /memo_trait_public_surface_hash|#import "neplg2\/core\/check\/module\/memo_trait_public_surface_hash"/,
    "core/ty memo trait source registry must not depend on the checker-layer public surface hash module",
);
assert.doesNotMatch(
    proofStore,
    /memo_trait_public_surface_hash|selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_result/,
    "proof store must not depend on public surface hash output directly",
);
assert.doesNotMatch(
    sourceCode,
    /line count|comment length|file size|500 行/,
    "public surface hash policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait public surface hash contract passed");
