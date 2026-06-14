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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_header.nepl";
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
        "# check/module/memo_trait_public_impl_header",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "public impl header producer must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.ok(
    source.includes("stable impl key と normalized impl header shape を分け") &&
        source.includes("inherent impl は現行 Rust 実装の public surface materializer でも unsupported"),
    "docs must separate declaration identity from shape and keep inherent impl unsupported",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、path suffix、HIR、Resource IR、backend artifact、proof store record は accepted hash material に入りません"),
    "docs must exclude source, display, diagnostic, HIR, Resource IR, backend, and proof store authority",
);
assert.ok(
    source.includes("count だけでは受理せず、詳細 generic binder evidence の count と nonzero shape hash を必ず照合") &&
        source.includes("stable trait key と trait type argument identity を含む typed normalized hash"),
    "docs must reject count-only generic impl identity and require trait application shape to include stable trait identity",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_header/,
    "impl header producer must remain facade-private until full public surface orchestration is designed",
);
assert.ok(
    normalizer.includes("SelfhostMemoTraitPublicSurfacePublicDeclarationKind::Impl") &&
        normalizer.includes("SelfhostMemoTraitPublicSurfaceHashInputKind::PublicImpl"),
    "producer must target the existing normalizer Impl declaration and PublicImpl input item boundary",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplHeaderKind:",
        "TraitImpl",
        "InherentImpl",
    ],
    "producer must model trait impl and inherent impl as an enum rather than a bool",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplHeaderInput:",
        "visibility %SelfhostModuleDeclarationVisibility",
        "module_fingerprint %i32",
        "declaration_ordinal %Option i32",
        "kind %SelfhostMemoTraitPublicImplHeaderKind",
        "target_type_shape_hash %Option i32",
        "trait_application_shape_hash %Option i32",
        "type_parameter_count %i32",
        "type_parameter_bound_count %i32",
        "generic_binder_evidence %SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence",
    ],
    "producer input must carry typed authority needed for stable impl key, normalized impl shape, and generic binder evidence",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence:",
        "Monomorphic",
        "Detailed %SelfhostMemoTraitPublicImplGenericBinderEvidence",
    ],
    "producer must distinguish monomorphic headers from detailed generic binder evidence with an enum",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplHeaderErrorKind:",
        "PrivateVisibilityRejected",
        "ModuleFingerprintPlaceholder",
        "DeclarationOrdinalMissing",
        "DeclarationOrdinalPlaceholder",
        "DeclarationOrdinalNegative",
        "InherentImplUnsupported",
        "TargetTypeShapeHashMissing",
        "TargetTypeShapeHashPlaceholder",
        "TraitApplicationShapeHashMissing",
        "TraitApplicationShapeHashPlaceholder",
        "TypeParameterCountNegative",
        "TypeParameterBoundCountNegative",
        "GenericImplUnsupported",
        "TypeParameterBoundUnsupported",
        "GenericBinderEvidenceMissing",
        "GenericBinderEvidenceUnexpected",
        "GenericBinderEvidenceParameterCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "GenericBinderEvidenceBoundCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "GenericBinderEvidenceHashPlaceholder",
        "StableImplKeyHashPlaceholder",
        "DerivedImplHeaderShapeHashPlaceholder",
        "PublicDeclarationPayloadRejected %SelfhostMemoTraitPublicSurfaceNormalizerErrorKind",
    ],
    "producer failures must keep visibility, key seed, impl kind, typed shape seed, generic binder evidence, derived shape, and payload failures separate",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_impl_header_stable_key_hash_result %fn i32 fn i32 fn SelfhostMemoTraitPublicImplHeaderKind Result SelfhostMemoTraitPublicImplHeaderStableKeyHash SelfhostMemoTraitPublicImplHeaderErrorKind"),
    "producer must expose a stable impl key hash boundary",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_impl_header_shape_hash_result %fn SelfhostMemoTraitPublicImplHeaderKind fn Option i32 fn Option i32 fn i32 fn i32 Result SelfhostMemoTraitPublicImplHeaderShapeHash SelfhostMemoTraitPublicImplHeaderErrorKind"),
    "producer must expose a normalized impl header shape hash boundary",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_impl_header_shape_hash_with_generic_binder_result %fn SelfhostMemoTraitPublicImplHeaderKind fn Option i32 fn Option i32 fn i32 fn i32 fn SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence Result SelfhostMemoTraitPublicImplHeaderShapeHash SelfhostMemoTraitPublicImplHeaderErrorKind"),
    "producer must expose a generic-binder-aware normalized impl header shape hash boundary",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_impl_header_payload_input_result %fn SelfhostMemoTraitPublicImplHeaderInput Result SelfhostMemoTraitPublicSurfacePublicDeclarationPayloadInput SelfhostMemoTraitPublicImplHeaderErrorKind"),
    "producer must expose a payload input adapter for normalizer composition",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_public_impl_header_evidence_result %fn SelfhostMemoTraitPublicImplHeaderInput Result SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence SelfhostMemoTraitPublicImplHeaderErrorKind"),
    "producer must expose an evidence API that returns normalizer-compatible public declaration evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_kind_supported_result"),
    [
        "SelfhostMemoTraitPublicImplHeaderKind::TraitImpl:",
        "Result::Ok unit",
        "SelfhostMemoTraitPublicImplHeaderKind::InherentImpl:",
        "InherentImplUnsupported",
    ],
    "impl kind check must accept trait impl and fail closed on inherent impl",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_stable_key_hash_result"),
    [
        "module_fingerprint 0",
        "ModuleFingerprintPlaceholder",
        "selfhost_memo_trait_public_impl_header_kind_supported_result kind",
        "DeclarationOrdinalNegative",
        "DeclarationOrdinalPlaceholder",
        "selfhost_memo_trait_public_impl_header_schema_version",
        "selfhost_memo_trait_public_impl_header_kind_code kind",
        "declaration_ordinal",
        "StableImplKeyHashPlaceholder",
    ],
    "stable key hash must validate module fingerprint, impl kind, and explicit public declaration ordinal before hashing",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_shape_hash_result"),
    [
        "selfhost_memo_trait_public_impl_header_kind_supported_result kind",
        "selfhost_memo_trait_public_impl_header_zero_count_result type_parameter_count",
        "GenericImplUnsupported",
        "selfhost_memo_trait_public_impl_header_zero_count_result type_parameter_bound_count",
        "TypeParameterBoundUnsupported",
        "selfhost_memo_trait_public_impl_header_shape_seed_result target_type_shape_hash",
        "selfhost_memo_trait_public_impl_header_shape_seed_result trait_application_shape_hash",
        "target_hash",
        "trait_hash",
        "DerivedImplHeaderShapeHashPlaceholder",
    ],
    "shape hash must validate trait impl kind, generic counts, target shape hash, and trait application shape hash before folding",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_generic_binder_hash_result"),
    [
        "SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence::Monomorphic:",
        "GenericBinderEvidenceMissing",
        "SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence::Detailed evidence:",
        "GenericBinderEvidenceUnexpected",
        "GenericBinderEvidenceParameterCountMismatch",
        "GenericBinderEvidenceBoundCountMismatch",
        "GenericBinderEvidenceHashPlaceholder",
        "Result::Ok evidence.shape_hash",
    ],
    "generic binder seed must reject count-only generic acceptance, count mismatches, and placeholder evidence hash",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_shape_hash_with_generic_binder_result"),
    [
        "selfhost_memo_trait_public_impl_header_kind_supported_result kind",
        "selfhost_memo_trait_public_impl_header_generic_binder_hash_result type_parameter_count type_parameter_bound_count generic_binder_evidence",
        "generic_binder_hash",
        "selfhost_memo_trait_public_impl_header_shape_seed_result target_type_shape_hash",
        "selfhost_memo_trait_public_impl_header_shape_seed_result trait_application_shape_hash",
        "generic_binder_hash",
        "DerivedImplHeaderShapeHashPlaceholder",
    ],
    "generic-aware shape hash must include validated binder evidence hash in the folded shape material",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_payload_input_result"),
    [
        "selfhost_memo_trait_public_impl_header_visibility_result input.visibility",
        "selfhost_memo_trait_public_impl_header_ordinal_result input.declaration_ordinal",
        "selfhost_memo_trait_public_impl_header_stable_key_hash_result input.module_fingerprint ordinal input.kind",
        "selfhost_memo_trait_public_impl_header_shape_hash_with_generic_binder_result input.kind input.target_type_shape_hash input.trait_application_shape_hash input.type_parameter_count input.type_parameter_bound_count input.generic_binder_evidence",
        "SelfhostMemoTraitPublicSurfacePublicDeclarationPayloadInput SelfhostMemoTraitPublicSurfacePublicDeclarationKind::Impl stable_key.key_hash shape.root_hash",
    ],
    "payload input producer must validate visibility, ordinal, stable impl key, shape hash, and set declaration kind to Impl",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_evidence_result"),
    [
        "selfhost_memo_trait_public_impl_header_ordinal_result input.declaration_ordinal",
        "selfhost_memo_trait_public_impl_header_payload_input_result input",
        "selfhost_memo_trait_public_impl_header_payload_hash_result payload_input",
        "SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence SelfhostMemoTraitPublicSurfacePublicDeclarationKind::Impl ordinal payload_hash",
    ],
    "evidence producer must validate ordinal, hash the typed normalizer payload input, and construct Impl evidence",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_payload_hash_result"),
    [
        "input.stable_declaration_key_hash",
        "StableDeclarationKeyHashPlaceholder",
        "input.normalized_declaration_shape_hash",
        "NormalizedDeclarationShapeHashPlaceholder",
        "selfhost_memo_trait_public_impl_header_payload_kind_code input.kind",
        "selfhost_memo_trait_public_impl_header_payload_schema_code",
        "DerivedDeclarationPayloadHashPlaceholder",
    ],
    "private payload hash helper must use only typed payload fields and preserve normalizer placeholder errors",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_stage0_generic_binder_input"),
    [
        "SelfhostMemoTraitPublicImplGenericBinderEvidence 1 1 1 8301",
        "selfhost_memo_trait_public_impl_header_detailed_binder_evidence evidence",
    ],
    "stage0 helper must build detailed generic binder evidence before accepting a generic header",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_count_mismatch_eq"),
    [
        "a.expected",
        "b.expected",
        "a.actual",
        "b.actual",
    ],
    "producer must compare generic binder count mismatch payload fields, not only the variant",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_header_error_kind_eq"),
    [
        "GenericBinderEvidenceParameterCountMismatch mismatch:",
        "GenericBinderEvidenceParameterCountMismatch other:",
        "selfhost_memo_trait_public_impl_header_count_mismatch_eq mismatch other",
        "GenericBinderEvidenceBoundCountMismatch mismatch:",
        "GenericBinderEvidenceBoundCountMismatch other:",
        "selfhost_memo_trait_public_impl_header_count_mismatch_eq mismatch other",
        "PublicDeclarationPayloadRejected payload_error:",
        "selfhost_memo_trait_public_surface_normalizer_error_kind_eq payload_error other_payload_error",
    ],
    "producer error equality must preserve generic binder mismatch payloads and normalizer rejection payloads",
);
assertOrdered(
    source,
    [
        "trait_input",
        "SelfhostMemoTraitPublicImplHeaderKind::TraitImpl",
        "other_trait_input",
        "some 8102",
        "inherent_input",
        "SelfhostMemoTraitPublicImplHeaderKind::InherentImpl",
        "missing_trait_input",
        "none",
        "placeholder_target_input",
        "some 0",
        "generic_input",
        "1 0",
        "generic_with_binder_input",
        "selfhost_memo_trait_public_impl_header_stage0_generic_binder_input",
        "stable_key_inherent_result",
        "selfhost_memo_trait_public_impl_header_stable_key_hash_result 6101 8 SelfhostMemoTraitPublicImplHeaderKind::InherentImpl",
        "stable_key_ordinal_result",
        "selfhost_memo_trait_public_impl_header_stable_key_hash_result 6101 0 SelfhostMemoTraitPublicImplHeaderKind::TraitImpl",
        "let differ %bool selfhost_memo_trait_public_impl_header_stage0_compare_payload",
    ],
    "stage0 must exercise accepted trait impls, trait application difference, count-only generic rejection, detailed generic binder acceptance, and public stable key helper rejection paths",
);
assert.doesNotMatch(
    sourceCode,
    /hash32\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|mix\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|\.path\b|\.alias\b|\.span\b|\.lexeme\b|display_name|diagnostic_text|source_path/,
    "producer implementation must not fold source text, spans, paths, aliases, display names, lexemes, diagnostics, or source path into accepted hash material",
);
assert.doesNotMatch(
    sourceCode,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader)/,
    "producer must not import HIR, Resource IR, backend, proof store, or serialized proof artifact layers",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_header/,
    "checker-layer public impl header producer must not be registered in the ty source list",
);
assert.doesNotMatch(
    sourceCode,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "producer policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait public impl header contract passed");
