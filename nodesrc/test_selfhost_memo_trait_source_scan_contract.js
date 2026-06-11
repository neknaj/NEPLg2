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

const scannerRelPath = "stdlib/neplg2/core/check/module/memo_trait_source_scan.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tyRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";
const scanner = read(scannerRelPath);
const facade = read(facadeRelPath);
const tySource = read(tyRelPath);
const proofStore = read(proofStoreRelPath);

assert.match(
    facade,
    /^pub #import "\.\/module\/memo_trait_source_scan" as \*$/m,
    "module checker facade must expose the memo trait source scanner as a public checker connection layer",
);
assert.match(
    scanner,
    /# check\/module\/memo_trait_source_scan[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "memo trait scanner must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    scanner,
    /syntax AST を読む処理を `core\/ty` に入れず[\s\S]*trusted source identity の authority ではありません/,
    "scanner documentation must state that it is a checker connection layer, not a trusted identity authority",
);
assert.match(
    scanner,
    /signature_available = false[\s\S]*stable public surface hash[\s\S]*stable trait definition key/,
    "scanner must document that discovered records are fail-closed until stable public surface identity exists",
);
assert.match(
    scanner,
    /^#import "neplg2\/core\/syntax\/ast\/module_ast" as \*$/m,
    "scanner must read typed module AST evidence from the checker layer",
);
assert.match(
    scanner,
    /^#import "neplg2\/core\/ty\/ty\/memo_trait_source" as \*$/m,
    "scanner must project into the existing memo trait definition source table",
);
assert.doesNotMatch(
    scanner,
    /#import "neplg2\/core\/(?:lower|hir|resource|backend)\//,
    "scanner must not depend on lowering, HIR, Resource IR, or backend layers",
);
assert.doesNotMatch(
    tySource,
    /#import "neplg2\/core\/(?:syntax|check)\//,
    "core/ty memo trait source registry must not import syntax AST or checker connection layers",
);
assert.match(
    scanner,
    /pub enum SelfhostMemoTraitDefinitionScanErrorKind:[\s\S]*ItemUnavailable[\s\S]*TraitHeaderMissing[\s\S]*TraitHeaderKindMismatch[\s\S]*TraitHeadMissing[\s\S]*TraitHeadKindMismatch[\s\S]*TraitNameUnavailable/,
    "scanner must preserve scan failures as a typed enum",
);
assert.match(
    scanner,
    /pub enum SelfhostMemoTraitDefinitionScanRegistryErrorKind:[\s\S]*ScanRejected %SelfhostMemoTraitDefinitionScanErrorKind[\s\S]*RegistryRejected %SelfhostMemoTraitTrustedSourceRegistryErrorKind/,
    "scanner combined API must keep scan errors and registry validation errors distinct",
);
assert.match(
    scanner,
    /pub fn selfhost_memo_trait_definition_source_table_scan_module_result %fn str fn &SelfhostModuleAst Result SelfhostMemoTraitDefinitionSourceTable SelfhostMemoTraitDefinitionScanErrorKind/,
    "scanner must expose a Result-returning table scan API",
);
assert.match(
    scanner,
    /selfhost_memo_trait_definition_source_table_scan_module_loop[\s\S]*selfhost_module_ast_len ast[\s\S]*selfhost_module_ast_get ast idx/,
    "scanner must walk the module AST by typed AST accessors rather than reparsing raw source",
);
assert.match(
    scanner,
    /match item\.kind:[\s\S]*SelfhostModuleItemKind::TraitDecl:[\s\S]*selfhost_memo_trait_definition_source_table_scan_trait_item_result[\s\S]*SelfhostModuleItemKind::FunctionDecl:[\s\S]*Result::Ok table[\s\S]*SelfhostModuleItemKind::ImplDecl:[\s\S]*Result::Ok table/,
    "scanner must classify only TraitDecl items and leave non-trait declarations as non-authoritative",
);
assert.match(
    scanner,
    /match header\.kind:[\s\S]*SelfhostModuleDeclarationKind::Trait:[\s\S]*SelfhostModuleDeclarationKind::Function:[\s\S]*TraitHeaderKindMismatch[\s\S]*SelfhostModuleDeclarationKind::Impl:[\s\S]*TraitHeaderKindMismatch/,
    "TraitDecl items with mismatched declaration headers must fail as typed scan errors",
);
assert.match(
    scanner,
    /match header\.head:[\s\S]*SelfhostModuleDeclarationHeadKind::Name:[\s\S]*selfhost_memo_trait_definition_source_table_scan_trait_name_result[\s\S]*SelfhostModuleDeclarationHeadKind::TypeLabel:[\s\S]*TraitHeadKindMismatch/,
    "scanner must require a normal declaration name head for MemoKey/MemoValue trait source candidates",
);
assert.match(
    scanner,
    /string_slice::str_slice_result source span\.start span\.end[\s\S]*Result::Err _slice_error:[\s\S]*TraitNameUnavailable/,
    "scanner must map source slicing failure into a typed scan error",
);
assert.match(
    scanner,
    /string_search::str_eq name "MemoKey"[\s\S]*SelfhostMemoTraitSourceKind::MemoKeyTrait[\s\S]*string_search::str_eq name "MemoValue"[\s\S]*SelfhostMemoTraitSourceKind::MemoValueTrait/,
    "scanner may use source spelling only for candidate classification of MemoKey/MemoValue trait names",
);
assert.match(
    scanner,
    /selfhost_memo_trait_definition_source_table_add_record[\s\S]*selfhost_memo_trait_definition_source_record_untrusted_scan_candidate/,
    "scanner must project discovered trait candidates into the existing definition source table",
);
assert.match(
    scanner,
    /selfhost_memo_trait_definition_source_record_untrusted_scan_candidate[\s\S]*selfhost_memo_trait_definition_fingerprint_new 0 0 0[\s\S]*false/,
    "scanner-created records must be explicitly untrusted and signature_available=false",
);
assert.doesNotMatch(
    scanner,
    /selfhost_memo_trait_source_identity_new/,
    "scanner must not construct accepted source identities from source spelling or spans",
);
assert.doesNotMatch(
    scanner,
    /signature_available\s*%\w+|selfhost_memo_trait_definition_source_record_new:[\s\S]*\n\s+true/,
    "scanner must not mark discovered source records as trusted before stable public surface identity exists",
);
assert.match(
    scanner,
    /selfhost_memo_trait_trusted_source_registry_from_scan_table_result[\s\S]*selfhost_memo_trait_trusted_source_registry_from_definition_table table[\s\S]*RegistryRejected registry_error/,
    "scanner combined API must reuse the existing table-backed registry validator",
);
assert.match(
    scanner,
    /selfhost_memo_trait_definition_scan_stage0[\s\S]*key_only_rejected[\s\S]*value_only_rejected[\s\S]*duplicate_key_rejected[\s\S]*duplicate_value_rejected[\s\S]*wrong_declaration_kind_rejected[\s\S]*stable_signature_rejected[\s\S]*malformed_header_rejected/,
    "scanner stage0 must cover missing, duplicate, wrong declaration kind, signature unavailable, and malformed header cases",
);
assert.match(
    scanner,
    /selfhost_memo_trait_definition_scan_stage0_module_registry_result[\s\S]*selfhost_module_ast_new[\s\S]*selfhost_module_ast_push ast0 key_item[\s\S]*selfhost_module_ast_push ast1 value_item[\s\S]*selfhost_memo_trait_trusted_source_registry_scan_module_result source &ast2[\s\S]*selfhost_module_ast_free ast2/,
    "scanner stage0 must exercise the public SelfhostModuleAst scan API and release the AST owner",
);
assert.match(
    scanner,
    /module_scan_signature_rejected[\s\S]*selfhost_memo_trait_definition_scan_stage0_module_registry_result source key_item value_item/,
    "scanner stage0 summary must include the public module-scan fail-closed result",
);
assert.match(
    scanner,
    /SelfhostMemoTraitDefinitionScanRegistryErrorKind::RegistryRejected SelfhostMemoTraitTrustedSourceRegistryErrorKind::MemoKeySourceRejected SelfhostMemoTraitSourceMaterializeErrorKind::SignatureMissing/,
    "scanner doctest must prove discovered records remain fail-closed at the stable signature boundary",
);
assert.doesNotMatch(
    scanner,
    /line count|comment length|file size|500 行/,
    "scanner policy must not introduce line-count or doc-comment-length restrictions",
);
assert.doesNotMatch(
    proofStore,
    /memo_trait_source_scan|selfhost_memo_trait_definition_source_table_scan_module_result|selfhost_memo_trait_trusted_source_registry_scan_module_result/,
    "proof store must not depend on scanner output directly; it should keep using trusted current source Result APIs",
);
assert.doesNotMatch(
    sectionBetween(
        proofStore,
        "selfhost_memo_trait_proof_store_stage0",
        "fn selfhost_memo_trait_proof_store_stage0_abort_with_store",
    ),
    /selfhost_memo_trait_source_identity_new/,
    "proof store stage0 must not bypass trusted registry APIs with raw source identity construction",
);

console.log("selfhost memo trait source scanner contract passed");
