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
const tokenSeedScanRelPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_token_seed_scan.nepl";
const tySourceRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";

const source = read(relPath);
const facade = read(facadeRelPath);
const tokenGateSource = read(tokenGateRelPath);
const tokenSeedScanSource = read(tokenSeedScanRelPath);
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
    /source text、span、display name、path suffix、diagnostic text を accepted fingerprint authority にせず[\s\S]*kind、visibility、declaration ordinal、normalized signature seed、caller supplied dependency public surface hash だけを input field として使います/,
    "public surface hash docs must exclude source spelling and spans from accepted fingerprint authority",
);
assert.match(
    source,
    /token-aware API が計算する public surface hash は、local `MemoKey` \/ `MemoValue` marker trait pair と method-bearing trait pair に対応[\s\S]*AST-only 互換 API は token authority がないため marker trait pair だけに限定[\s\S]*import \/ use \/ prelude \/ public non-trait declaration は[\s\S]*拒否/,
    "hash materializer must document token-aware method-bearing support and fail-closed unsupported full public surface cases",
);
assert.match(
    source,
    /registry convenience path は candidate table と seed table を同じ item loop で作るため、module item stream を 1 回だけ走査し、token-aware method 正規化を含めても全体は O\(n \+ k\)[\s\S]*既存の scanner API と materialize API は単体検査と互換のため残します/,
    "hash materializer must document that registry convenience paths are single-pass while standalone scanner/materializer APIs remain",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitPublicSurfaceHashMaterialization:[\s\S]*module_seed %SelfhostMemoTraitStableSourceModuleSeed[\s\S]*seed_table %SelfhostMemoTraitStableSourceSeedTable/,
    "hash materialization output must carry both the derived module seed and the seed table that produced it",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitPublicSurfaceHashRegistryMaterialization:[\s\S]*candidates %SelfhostMemoTraitDefinitionSourceTable[\s\S]*materialization %SelfhostMemoTraitPublicSurfaceHashMaterialization/,
    "registry convenience path must carry candidate table and hash materialization from the same item pass",
);
assert.match(
    source,
    /struct SelfhostMemoTraitPublicSurfaceHashRegistryScanState:[\s\S]*candidates %SelfhostMemoTraitDefinitionSourceTable[\s\S]*seed_table %SelfhostMemoTraitStableSourceSeedTable/,
    "single-pass registry scan state must keep candidate table and seed table together",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicSurfaceHashInputKind:[\s\S]*LocalMemoTrait %SelfhostMemoTraitSourceKind[\s\S]*DependencyModule[\s\S]*ReExport[\s\S]*PublicFunction[\s\S]*PublicStruct[\s\S]*PublicEnum[\s\S]*PublicImpl/,
    "public surface hash must define typed input item kinds for local traits, dependencies, re-exports, and public declarations",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitPublicSurfaceHashInputItem:[\s\S]*kind %SelfhostMemoTraitPublicSurfaceHashInputKind[\s\S]*ordinal %i32[\s\S]*visibility %SelfhostModuleDeclarationVisibility[\s\S]*payload_hash %i32[\s\S]*dependency_public_surface_hash %Option i32/,
    "public surface hash input item must carry only typed authority fields",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitPublicSurfaceHashInputTable:[\s\S]*first %SelfhostMemoTraitPublicSurfaceHashInputItem[\s\S]*second %SelfhostMemoTraitPublicSurfaceHashInputItem/,
    "phase 1 public surface hash must use an ordered input table before folding the hash",
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
    /selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_result[\s\S]*selfhost_memo_trait_public_surface_hash_registry_materialize_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_seed_evidence_result/,
    "registry path must go through the single-pass registry materializer, seed evidence producer, and existing stable registry gate",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_with_tokens_result[\s\S]*selfhost_memo_trait_public_surface_hash_registry_materialize_with_tokens_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_seed_evidence_result/,
    "token-aware registry path must go through the single-pass token-aware registry materializer, seed evidence producer, and existing stable registry gate",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_registry_materialize_result[\s\S]*selfhost_memo_trait_public_surface_hash_validate_module_identity_result[\s\S]*selfhost_memo_trait_public_surface_hash_registry_scan_module_result[\s\S]*selfhost_memo_trait_public_surface_hash_registry_materialization_from_state_result/,
    "AST-only registry materializer must validate module identity and then use one combined module scan",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_registry_materialize_with_tokens_result[\s\S]*selfhost_memo_trait_public_surface_hash_validate_module_identity_result[\s\S]*selfhost_memo_trait_public_surface_hash_registry_scan_module_with_tokens_result[\s\S]*selfhost_memo_trait_public_surface_hash_registry_materialization_from_state_result/,
    "token-aware registry materializer must validate module identity and then use one combined module scan",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_registry_scan_item_result[\s\S]*selfhost_memo_trait_public_surface_hash_registry_scan_candidate_item_result[\s\S]*selfhost_memo_trait_public_surface_hash_scan_item_result/,
    "AST-only registry item scan must update candidate table and seed table in the same item pass",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_registry_scan_item_with_tokens_result[\s\S]*selfhost_memo_trait_public_surface_hash_registry_scan_candidate_item_result[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_item_result/,
    "token-aware registry item scan must update candidate table and shared-core seed table in the same item pass",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_registry_scan_item_result[\s\S]*candidate 分類を先に評価[\s\S]*malformed trait header は hash error ではなく `CandidateScanRejected`/,
    "AST-only combined registry scan must document candidate-first error precedence",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_registry_scan_item_with_tokens_result[\s\S]*candidate 分類を先に評価[\s\S]*shared core 由来の public surface hash error を同一 variant に潰しません/,
    "token-aware combined registry scan must document candidate-first error precedence and payload separation",
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
    /#import "\.\/memo_trait_public_surface_token_seed_scan" as \*/,
    "hash materializer must directly import the shared token-aware scan core",
);
assert.doesNotMatch(
    source,
    /#import "\.\/memo_trait_public_surface_token_gate" as \*/,
    "hash materializer must not depend on the token gate wrapper after the shared core split",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_scan_item_result[\s\S]*SelfhostModuleItemKind::TraitDecl:[\s\S]*selfhost_memo_trait_public_surface_hash_scan_trait_item_result[\s\S]*SelfhostModuleItemKind::ImportDirective:[\s\S]*ImportSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::UseDirective:[\s\S]*UseSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::PreludeDirective:[\s\S]*PreludeSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::NoPreludeDirective:[\s\S]*NoPreludeSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::FunctionDecl:[\s\S]*PublicFunctionSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::StructDecl:[\s\S]*PublicStructSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::EnumDecl:[\s\S]*PublicEnumSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::ImplDecl:[\s\S]*PublicImplSurfaceUnsupported/,
    "hash-owned AST item loop must combine unsupported public surface rejection with marker trait seed accumulation",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_scan_module_with_tokens_loop[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_item_result[\s\S]*selfhost_memo_trait_public_surface_hash_error_from_token_seed_scan_error/,
    "hash-owned token-aware loop must reuse the shared core item helper and map typed core errors into hash errors",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_seed_error_from_token_seed_scan_error[\s\S]*MemoKeyMethodSignatureRejected method_error:[\s\S]*SelfhostMemoTraitPublicSurfaceSeedErrorKind::MemoKeyMethodSignatureRejected method_error[\s\S]*MemoValueMethodSignatureRejected method_error:[\s\S]*SelfhostMemoTraitPublicSurfaceSeedErrorKind::MemoValueMethodSignatureRejected method_error/,
    "hash token-core mapping must preserve method normalizer payloads through the seed error wrapper",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_error_from_token_seed_scan_error[\s\S]*selfhost_memo_trait_public_surface_hash_error_from_seed_scan_error selfhost_memo_trait_public_surface_hash_seed_error_from_token_seed_scan_error error/,
    "hash must convert shared core errors through the seed taxonomy before applying the existing hash error mapping",
);
assert.match(
    tokenGateSource,
    /pub fn selfhost_memo_trait_public_surface_token_gate_scan_item_result[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_item_result source tokens table item[\s\S]*selfhost_memo_trait_public_surface_token_gate_seed_error_from_token_seed_scan_error token_error/,
    "token gate item API must be a thin wrapper over the shared core item scan",
);
assert.match(
    tokenGateSource,
    /pub fn selfhost_memo_trait_public_surface_token_gate_seed_table_with_tokens_result[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_module_result source tokens ast[\s\S]*selfhost_memo_trait_public_surface_token_gate_seed_error_from_token_seed_scan_error token_error/,
    "token gate module API must be a thin wrapper over the shared core module scan",
);
assert.doesNotMatch(
    tokenGateSource,
    /selfhost_memo_trait_public_surface_token_gate_non_declaration_item_result|selfhost_memo_trait_public_surface_token_gate_scan_declaration_item_result|selfhost_memo_trait_public_surface_token_gate_scan_trait_item_result|selfhost_module_item_kind_declaration item\.kind/,
    "token gate wrapper must not keep the token-aware scan implementation after the shared core split",
);
assert.match(
    tokenSeedScanSource,
    /pub fn selfhost_memo_trait_public_surface_token_seed_scan_item_result[\s\S]*selfhost_module_item_kind_declaration item\.kind[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_declaration_item_result[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_non_declaration_item_result/,
    "shared token seed scan core must own declaration dispatch and non-declaration policy",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_input_schema_code[\s\S]*ordered input item table を fold する schema[\s\S]*hash material の構造が変わる場合はこの schema code を更新/,
    "public surface hash must carry a dedicated full-input schema/domain code",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_input_kind_code[\s\S]*LocalMemoTrait source_kind:[\s\S]*DependencyModule:[\s\S]*ReExport:[\s\S]*PublicFunction:[\s\S]*PublicStruct:[\s\S]*PublicEnum:[\s\S]*PublicImpl:/,
    "input kind code must be exhaustive over current and reserved full public surface item kinds",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_input_table_from_seed_table_result[\s\S]*selfhost_memo_trait_public_surface_hash_key_input_item_result[\s\S]*selfhost_memo_trait_public_surface_hash_value_input_item_result[\s\S]*selfhost_memo_trait_public_surface_hash_input_table_new key_item value_item/,
    "seed table must be adapted into an ordered full-surface input table before hash folding",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_input_table_result[\s\S]*selfhost_memo_trait_public_surface_hash_input_schema_code[\s\S]*first_hash[\s\S]*second_hash/,
    "ordered input table folding must use the new full-input schema and preserve item order",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_from_seed_table_result[\s\S]*selfhost_memo_trait_public_surface_hash_input_table_from_seed_table_result seed_table[\s\S]*selfhost_memo_trait_public_surface_hash_input_table_result input_table/,
    "seed table folding API must route through the typed input table boundary",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_public_surface_hash_schema_code|selfhost_memo_trait_public_surface_hash_key_seed_result|selfhost_memo_trait_public_surface_hash_value_seed_result|211001|211002|212103/,
    "old direct seed-hash helpers and marker schema domain codes must not remain after the full-input boundary is introduced",
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
    /source text と span は引数に取りません[\s\S]*ordered full-surface input table[\s\S]*typed field だけ/,
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
const astRegistrySection = sectionBetween(
    source,
    "pub fn selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_result",
    "//: selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_with_tokens_result",
);
assert.doesNotMatch(
    astRegistrySection,
    /selfhost_memo_trait_definition_source_table_scan_module_result|selfhost_memo_trait_public_surface_hash_materialize_result/,
    "AST-only registry convenience API must not run candidate scanner and materializer as separate module passes",
);
const tokenRegistrySection = sectionBetween(
    source,
    "fn selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_with_tokens_result",
    "//: selfhost_memo_trait_public_surface_hash_stage0_header_named",
);
assert.doesNotMatch(
    tokenRegistrySection,
    /selfhost_memo_trait_definition_source_table_scan_module_result|selfhost_memo_trait_public_surface_hash_materialize_with_tokens_result/,
    "token-aware registry convenience API must not run candidate scanner and materializer as separate module passes",
);
assert.match(
    source,
    /method_surface_accepted_registry[\s\S]*selfhost_memo_trait_public_surface_hash_stage0_method_registry_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_with_tokens_result/,
    "stage0 smoke must cover token-aware method-bearing trait hash registry acceptance",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_key_input_item_result[\s\S]*selfhost_memo_trait_source_kind_eq seed\.kind SelfhostMemoTraitSourceKind::MemoKeyTrait[\s\S]*seed\.declaration_ordinal[\s\S]*seed\.normalized_signature_hash[\s\S]*SelfhostMemoTraitPublicSurfaceHashInputKind::LocalMemoTrait seed\.kind/,
    "MemoKey seed adapter must check source kind, ordinal, signature, visibility, and create a typed input item",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_value_input_item_result[\s\S]*selfhost_memo_trait_source_kind_eq seed\.kind SelfhostMemoTraitSourceKind::MemoValueTrait[\s\S]*seed\.declaration_ordinal[\s\S]*seed\.normalized_signature_hash[\s\S]*SelfhostMemoTraitPublicSurfaceHashInputKind::LocalMemoTrait seed\.kind/,
    "MemoValue seed adapter must check source kind, ordinal, signature, visibility, and create a typed input item",
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
    sourceCode,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded)/,
    "public surface hash module must not import HIR, Resource IR, backend, or proof artifact/store layers",
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
