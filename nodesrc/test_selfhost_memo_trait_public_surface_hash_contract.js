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
    /Phase 1 の local `MemoKey` \/ `MemoValue` marker trait pair に限定[\s\S]*import \/ use \/ prelude \/ public non-trait declaration は[\s\S]*拒否/,
    "hash materializer must document that unsupported full public surface cases fail closed",
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
    /pub fn selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_result %fn Option i32 fn str fn &SelfhostModuleAst Result SelfhostMemoTraitTrustedSourceRegistry SelfhostMemoTraitPublicSurfaceHashRegistryErrorKind/,
    "registry convenience API must expose typed hash path errors",
);
assert.match(
    source,
    /selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_result[\s\S]*selfhost_memo_trait_definition_source_table_scan_module_result[\s\S]*selfhost_memo_trait_public_surface_hash_materialize_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_seed_evidence_result/,
    "registry path must go through scanner, hash materializer, seed evidence producer, and existing stable registry gate",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_materialize_result[\s\S]*selfhost_memo_trait_public_surface_hash_validate_module_identity_result[\s\S]*selfhost_memo_trait_public_surface_hash_supported_module_loop[\s\S]*selfhost_memo_trait_public_surface_seed_scan_module_result[\s\S]*selfhost_memo_trait_public_surface_hash_from_seed_table_result/,
    "materializer must validate module identity, reject unsupported surfaces, scan marker seeds, and only then fold a public surface hash",
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
    /selfhost_memo_trait_public_surface_hash_supported_item_result[\s\S]*SelfhostModuleItemKind::ImportDirective:[\s\S]*ImportSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::UseDirective:[\s\S]*UseSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::PreludeDirective:[\s\S]*PreludeSurfaceUnsupported[\s\S]*SelfhostModuleItemKind::NoPreludeDirective:[\s\S]*NoPreludeSurfaceUnsupported/,
    "import, use, and prelude surfaces must fail closed until dependency surface normalization exists",
);
assert.match(
    source,
    /SelfhostModuleItemKind::FunctionDecl:[\s\S]*selfhost_memo_trait_public_surface_hash_public_decl_supported_result[\s\S]*SelfhostModuleItemKind::StructDecl:[\s\S]*selfhost_memo_trait_public_surface_hash_public_decl_supported_result[\s\S]*SelfhostModuleItemKind::EnumDecl:[\s\S]*selfhost_memo_trait_public_surface_hash_public_decl_supported_result[\s\S]*SelfhostModuleItemKind::ImplDecl:[\s\S]*selfhost_memo_trait_public_surface_hash_public_decl_supported_result/,
    "public non-trait declarations must not be ignored by the MemoKey/MemoValue-only hash materializer",
);
assert.match(
    source,
    /selfhost_memo_trait_public_surface_hash_public_decl_supported_result[\s\S]*SelfhostModuleDeclarationVisibility::Public:[\s\S]*Result::Err public_error[\s\S]*SelfhostModuleDeclarationVisibility::Private:[\s\S]*Result::Ok unit/,
    "only private non-trait declarations may be ignored in this local marker-trait slice",
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
