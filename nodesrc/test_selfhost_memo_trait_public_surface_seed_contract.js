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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_seed.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";

const source = read(relPath);
const facade = read(facadeRelPath);
const tySource = read(tySourceRelPath);
const proofStore = read(proofStoreRelPath);
const sourceCode = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/module\/memo_trait_public_surface_seed" as \*$/m,
    "module checker facade must expose the public surface seed materializer",
);
assert.match(
    source,
    /# check\/module\/memo_trait_public_surface_seed[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "public surface seed module must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /source text slicing は `MemoKey` \/ `MemoValue` の候補分類にだけ使います[\s\S]*source span、syntax range は accepted fingerprint payload に混ぜません/,
    "public surface seed docs must exclude source text, spans, and syntax ranges from accepted fingerprint authority",
);
assert.match(
    source,
    /token-aware accepted path は public marker trait と、正規化済み method-bearing trait だけです[\s\S]*AST-only 互換 API は token authority がないため marker trait だけを受理/,
    "public surface seed module must distinguish token-aware method-bearing acceptance from AST-only marker compatibility",
);
assert.match(
    source,
    /AST-only marker trait は `memo_trait_signature_shape` に委譲し、token-aware method-bearing trait は `memo_trait_public_surface_token_seed_scan` の shared core に委譲/,
    "public surface seed must delegate token-aware method-bearing signature normalization to the shared core boundary",
);
assert.match(
    source,
    /module identity seed と public surface seed は caller が `SelfhostMemoTraitStableSourceModuleSeed` として渡します[\s\S]*module path や file path から identity を作りません/,
    "module identity must be provided as typed seed rather than derived from path text",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitPublicSurfaceSeedScan:[\s\S]*seed_table %SelfhostMemoTraitStableSourceSeedTable/,
    "public surface scan output must be separate from registry construction",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicSurfaceSeedErrorKind:[\s\S]*MemoKeyCandidateMissing[\s\S]*MemoValueCandidateMissing[\s\S]*MemoKeyCandidateDuplicate[\s\S]*MemoValueCandidateDuplicate[\s\S]*ModuleIdentitySeedMissing[\s\S]*PublicSurfaceSeedMissing[\s\S]*MemoKeyPrivateVisibility[\s\S]*MemoValuePrivateVisibility[\s\S]*MemoKeyTraitBodyNormalizationUnsupported[\s\S]*MemoValueTraitBodyNormalizationUnsupported[\s\S]*StableNominalKeyMissing[\s\S]*ReExportUnsupported[\s\S]*ImportSurfaceUnsupported[\s\S]*UseSurfaceUnsupported[\s\S]*PreludeSurfaceUnsupported[\s\S]*NoPreludeSurfaceUnsupported[\s\S]*PublicFunctionSurfaceUnsupported[\s\S]*PublicStructSurfaceUnsupported[\s\S]*PublicEnumSurfaceUnsupported[\s\S]*PublicImplSurfaceUnsupported[\s\S]*DeclarationHeaderMissing/,
    "public surface seed failures must be typed enum variants covering missing, duplicate, module seed, private visibility, and unsupported public surface cases",
);
assert.match(
    source,
    /MemoKeyMethodSignatureRejected %SelfhostMemoTraitMethodSignatureErrorKind[\s\S]*MemoValueMethodSignatureRejected %SelfhostMemoTraitMethodSignatureErrorKind/,
    "public surface seed failures must preserve method normalizer errors as typed payload variants",
);
assert.match(
    source,
    /import \/ use \/ prelude \/ public non-trait declaration は、full public surface materializer[\s\S]*この scan の 1 pass 内で fail-closed[\s\S]*StableNominalKeyMissing` と `ReExportUnsupported` は、後続 slice[\s\S]*token-aware accepted path は local public marker trait と正規化済み method-bearing trait[\s\S]*AST-only accepted path は token authority がないため marker trait に限定/,
    "unsupported public surface checks and currently unreachable stable nominal/re-export errors must be documented",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitPublicSurfaceSeedRegistryErrorKind:[\s\S]*PublicSurfaceRejected %SelfhostMemoTraitPublicSurfaceSeedErrorKind[\s\S]*CandidateScanRejected %SelfhostMemoTraitDefinitionScanErrorKind[\s\S]*SeedRegistryRejected %SelfhostMemoTraitStableSourceSeedRegistryErrorKind/,
    "registry wrapper must keep public surface, scanner, and seed registry errors separate",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_public_surface_seed_scan_module_result %fn str fn &SelfhostModuleAst Result SelfhostMemoTraitPublicSurfaceSeedScan SelfhostMemoTraitPublicSurfaceSeedErrorKind/,
    "public surface seed scan API must return a typed Result payload before registry construction",
);
assert.match(
    source,
    /\nfn selfhost_memo_trait_public_surface_seed_scan_module_with_tokens_result %impure fn str impure fn &Vec SelfhostToken impure fn &SelfhostModuleAst Result SelfhostMemoTraitPublicSurfaceSeedScan SelfhostMemoTraitPublicSurfaceSeedErrorKind/,
    "token-aware public surface seed scan gate must accept parser tokens and AST",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_trusted_source_registry_from_public_surface_seed_result %fn str fn &SelfhostModuleAst fn SelfhostMemoTraitStableSourceModuleSeed Result SelfhostMemoTraitTrustedSourceRegistry SelfhostMemoTraitPublicSurfaceSeedRegistryErrorKind/,
    "registry convenience API must accept an external module seed and return typed errors",
);
assert.match(
    source,
    /\nfn selfhost_memo_trait_trusted_source_registry_from_public_surface_seed_with_tokens_result %impure fn str impure fn &Vec SelfhostToken impure fn &SelfhostModuleAst impure fn SelfhostMemoTraitStableSourceModuleSeed Result SelfhostMemoTraitTrustedSourceRegistry SelfhostMemoTraitPublicSurfaceSeedRegistryErrorKind/,
    "token-aware registry gate must accept parser tokens, AST, and an external module seed",
);
assert.doesNotMatch(
    source,
    /pub fn selfhost_memo_trait_public_surface_seed_scan_module_with_tokens_result|pub fn selfhost_memo_trait_trusted_source_registry_from_public_surface_seed_with_tokens_result/,
    "token-aware seed gates must not be exported through the module checker facade until a stable orchestration boundary is designed",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_from_public_surface_seed_result[\s\S]*selfhost_memo_trait_definition_source_table_scan_module_result[\s\S]*selfhost_memo_trait_public_surface_seed_scan_module_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_seed_evidence_result/,
    "registry path must go through scanner, public surface seed scan, seed evidence producer, and existing registry validator",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_from_public_surface_seed_with_tokens_result[\s\S]*selfhost_memo_trait_definition_source_table_scan_module_result[\s\S]*selfhost_memo_trait_public_surface_seed_scan_module_with_tokens_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_seed_evidence_result/,
    "token-aware registry path must go through scanner, token-aware public surface seed scan, seed evidence producer, and existing registry validator",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_seed_module_seed_ready_result[\s\S]*module_seed\.module_identity_hash[\s\S]*ModuleIdentitySeedMissing[\s\S]*ModuleIdentitySeedPlaceholder[\s\S]*module_seed\.public_surface_hash[\s\S]*PublicSurfaceSeedMissing[\s\S]*PublicSurfaceSeedPlaceholder/,
    "module seed input must be checked for missing and placeholder values before registry construction",
);
const signatureSection = sectionBetween(
    source,
    "selfhost_memo_trait_public_surface_seed_signature_from_normalization_result",
    "selfhost_memo_trait_public_surface_seed_declaration_ordinal",
);
const signatureCode = signatureSection
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");
assert.doesNotMatch(
    signatureCode,
    /\bsource\b|source\.|\bspan\b|span\.|\brange\b|range\.|\bname\b|name\.|str_slice|\bpath\b|path\.|\bdisplay\b|\bdiagnostic\b/i,
    "Phase 1 accepted signature seed implementation must not depend on source text, spans, ranges, paths, display names, or diagnostic text",
);
assert.match(
    signatureSection,
    /normalization module が作った `normalized_signature_hash` だけ[\s\S]*evidence\.normalized_signature_hash/,
    "signature seed extraction must consume normalized signature evidence rather than local source ranges",
);
assert.match(
    source,
    /#import "\.\/memo_trait_method_signature" as \*[\s\S]*#import "\.\/memo_trait_public_surface_token_seed_scan" as \*[\s\S]*#import "\.\/memo_trait_signature_shape" as \*/,
    "public surface seed must import marker normalization, method error payload type support, and the shared token-aware core",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_seed_error_from_token_seed_scan_error[\s\S]*MemoKeyMethodSignatureRejected method_error:[\s\S]*SelfhostMemoTraitPublicSurfaceSeedErrorKind::MemoKeyMethodSignatureRejected method_error[\s\S]*MemoValueMethodSignatureRejected method_error:[\s\S]*SelfhostMemoTraitPublicSurfaceSeedErrorKind::MemoValueMethodSignatureRejected method_error/,
    "public surface seed must preserve shared core method normalizer payloads when mapping into seed errors",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_seed_error_from_token_seed_scan_error[\s\S]*ImportSurfaceUnsupported:[\s\S]*SelfhostMemoTraitPublicSurfaceSeedErrorKind::ImportSurfaceUnsupported[\s\S]*PublicFunctionSurfaceUnsupported:[\s\S]*SelfhostMemoTraitPublicSurfaceSeedErrorKind::PublicFunctionSurfaceUnsupported[\s\S]*DeclarationHeaderMissing:[\s\S]*SelfhostMemoTraitPublicSurfaceSeedErrorKind::DeclarationHeaderMissing/,
    "public surface seed must map shared core unsupported surface reasons into the seed taxonomy",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_seed_scan_module_with_tokens_result[\s\S]*selfhost_memo_trait_public_surface_token_seed_scan_module_result source tokens ast[\s\S]*Result::Ok complete_table:[\s\S]*selfhost_memo_trait_public_surface_seed_scan_new complete_table[\s\S]*Result::Err token_error:[\s\S]*selfhost_memo_trait_public_surface_seed_error_from_token_seed_scan_error token_error/,
    "token-aware public surface seed scan must delegate module scanning to the shared core and map only the typed error",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_seed_public_decl_supported_result[\s\S]*SelfhostModuleDeclarationVisibility::Public:[\s\S]*Result::Err public_error[\s\S]*SelfhostModuleDeclarationVisibility::Private:[\s\S]*Result::Ok unit[\s\S]*DeclarationHeaderMissing/,
    "public surface seed scan must reject public non-trait declarations in the same pass while allowing private non-trait declarations",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_seed_table_scan_non_declaration_item_result[\s\S]*SelfhostModuleItemKind::ImportDirective:[\s\S]*ImportSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::UseDirective:[\s\S]*UseSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::PreludeDirective:[\s\S]*PreludeSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::NoPreludeDirective:[\s\S]*NoPreludeSurfaceUnsupported/,
    "AST-only seed scan must keep unsupported non-declaration surface rejection in the shared non-declaration helper",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_seed_table_scan_declaration_item_result[\s\S]*SelfhostModuleDeclarationKind::Function:[\s\S]*PublicFunctionSurfaceUnsupported[\s\S]*SelfhostModuleDeclarationKind::Struct:[\s\S]*PublicStructSurfaceUnsupported[\s\S]*SelfhostModuleDeclarationKind::Enum:[\s\S]*PublicEnumSurfaceUnsupported[\s\S]*SelfhostModuleDeclarationKind::Impl:[\s\S]*PublicImplSurfaceUnsupported[\s\S]*selfhost_memo_trait_public_surface_seed_table_scan_item_result[\s\S]*selfhost_module_item_kind_declaration item\.kind[\s\S]*selfhost_memo_trait_public_surface_seed_table_scan_non_declaration_item_result/,
    "AST-only seed scan must dispatch through the shared declaration classifier and still reject public non-trait declarations",
);
assert.doesNotMatch(
    source,
    /selfhost_memo_trait_public_surface_seed_table_scan_declaration_item_with_tokens_result|selfhost_memo_trait_public_surface_seed_table_scan_trait_item_with_tokens_result|selfhost_memo_trait_public_surface_seed_table_scan_item_with_tokens_result|selfhost_memo_trait_public_surface_seed_method_signature_from_body_result|selfhost_memo_trait_public_surface_seed_signature_with_tokens_result/,
    "token-aware seed scan implementation must live in the shared core, not in the seed wrapper",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_stage0_trait_item_with_type_annotation[\s\S]*let trait_header_rejected/,
    "stage0 smoke must cover non-marker trait header rejection",
);
assert.match(
    source,
    /unwrap_err summary\.trait_header_rejected[\s\S]*MemoKeyTraitBodyNormalizationUnsupported/,
    "doctest must assert non-marker trait header rejection",
);
assert.match(
    source,
    /method_surface_accepted_registry[\s\S]*selfhost_memo_trait_public_surface_stage0_method_registry_from_source_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_public_surface_seed_with_tokens_result/,
    "stage0 smoke must cover token-aware method-bearing trait acceptance",
);
assert.match(
    source,
    /method_surface_rejected_registry/,
    "stage0 summary must expose token-aware method-bearing trait rejection",
);
assert.match(
    source,
    /expected_method_error[\s\S]*MemoKeyMethodSignatureRejected[\s\S]*MethodCountMismatch SelfhostMemoTraitSourceKind::MemoKeyTrait/,
    "stage0 smoke must cover token-aware method-bearing trait rejection with typed payload",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_seed_table_scan_trait_name_result[\s\S]*str_slice_result source span\.start span\.end[\s\S]*selfhost_memo_trait_public_surface_seed_table_add_name_result/,
    "source slicing may only be used for candidate name classification",
);
assert.doesNotMatch(
    sourceCode,
    /hash32\s+name|mix\s+name|hash32\s+source|mix\s+source|hash32\s+span|mix\s+span/,
    "source text and source spans must not be folded into accepted fingerprint seeds",
);
assert.doesNotMatch(
    sourceCode,
    /selfhost_memo_trait_source_identity_new/,
    "public surface seed module must not construct accepted source identities directly",
);
assert.doesNotMatch(
    sourceCode,
    /selfhost_memo_trait_definition_source_record_new:[\s\S]*\n\s+true/,
    "public surface seed module must not directly create signature_available=true source records",
);
assert.doesNotMatch(
    sourceCode,
    /display name|path suffix|diagnostic text[\s\S]*(?:hash|mix)|hash[\s\S]*(?:display name|path suffix|diagnostic text)/i,
    "display name, path suffix, and diagnostic text must not become seed authority",
);
assert.doesNotMatch(
    tySource,
    /memo_trait_public_surface_seed|#import "neplg2\/core\/check\/module\/memo_trait_public_surface_seed"/,
    "core/ty memo trait source registry must not depend on the checker-layer public surface seed module",
);
assert.doesNotMatch(
    proofStore,
    /memo_trait_public_surface_seed|selfhost_memo_trait_trusted_source_registry_from_public_surface_seed_result/,
    "proof store must not depend on public surface seed output directly",
);
assert.doesNotMatch(
    sourceCode,
    /line count|comment length|file size|500 行/,
    "public surface seed policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait public surface seed contract passed");
