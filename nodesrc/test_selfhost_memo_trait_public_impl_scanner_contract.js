#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function stripDocComments(src) {
    return src
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

function functionBlock(src, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_scanner.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const seedRelPath = "stdlib/neplg2/core/check/module/memo_trait_public_surface_token_seed_scan.nepl";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const tokenSeedScan = read(seedRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_public_impl_scanner",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "public impl scanner must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("parser が作った `SelfhostModuleAst` には `ImplDecl` と declaration header / body range だけがあり") &&
        source.includes("別の resolver / lowering stage が作った typed impl record と public `ImplDecl` の存在だけを照合します"),
    "docs must state that AST is alignment authority only and typed resolver records carry semantic payload",
);
assert.ok(
    source.includes("pairing key は 1-origin の public declaration ordinal") &&
        source.includes("AST index は error payload の位置情報にだけ使い"),
    "docs must use public declaration ordinal as the association key and keep AST index out of proof authority",
);
assert.ok(
    source.includes("ordinal alignment は header validation より先に module 全体で検査します") &&
        source.includes("余剰 record と不正 header が同時にある場合は、対応関係が壊れていることを先に報告します"),
    "docs must state that global ordinal alignment is validated before per-record header validation",
);
assert.ok(
    source.includes("operation kind はこの module では確定しません") &&
        source.includes("既存 classifier が shape-bound evidence として operation を決めます"),
    "docs must keep operation classification in the existing operation materializer/classifier boundary",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、method name string、trait name string") &&
        source.includes("target / trait / operation / HIR root を推測しません"),
    "docs must reject source-derived authority for impl association",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_scanner/,
    "public impl scanner must remain facade-private until full public surface orchestration consumes it",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_scanner/,
    "checker-layer public impl scanner must not be registered in the ty source list",
);
assert.match(
    tokenSeedScan,
    /SelfhostModuleItemKind::ImplDecl:[\s\S]*PublicImplSurfaceUnsupported/,
    "token seed scan must keep public impls unsupported until the typed public impl scanner/composer boundary consumes them",
);
assertOrdered(
    source,
    [
        "#import \"neplg2/core/syntax/ast/module_ast\" as *",
        "#import \"./memo_trait_operation_classifier\" as *",
        "#import \"./memo_trait_operation_public_impl_materializer\" as *",
        "#import \"./memo_trait_operation_public_impl_materializer_record\" as *",
        "#import \"./memo_trait_public_impl_header\" as *",
        "#import \"./memo_trait_public_surface_normalizer\" as *",
    ],
    "scanner imports must stay on AST alignment, operation classifier/materializer record, public impl header, and normalizer-compatible evidence boundaries",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_operation_impl_candidate_builder|memo_trait_operation_impl_table|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop_impl_resolver)/,
    "scanner must not import Resource IR, backend, proof store, canonical-key, candidate builder, impl table, producer, purity, method-body, body-check, or Drop resolver layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplScannerResolverRecord:",
        "type_id %SelfhostTypeId",
        "module_fingerprint %i32",
        "declaration_ordinal %Option i32",
        "visibility %SelfhostModuleDeclarationVisibility",
        "impl_kind %SelfhostMemoTraitPublicImplHeaderKind",
        "target_type_shape_hash %Option i32",
        "trait_source %SelfhostMemoTraitOperationSourceIdentity",
        "trait_type_argument_count %i32",
        "trait_application_shape_hash %Option i32",
        "type_parameter_count %i32",
        "type_parameter_bound_count %i32",
        "method_body_root %Option SelfhostHirExprId",
        "fuel %i32",
    ],
    "resolver record must carry typed impl header fields, operation materializer fields, method root, and traversal fuel",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplScannerOutput:",
        "operation_records %SelfhostMemoTraitOperationPublicImplMaterializerRecordTable",
        "public_declarations %Vec SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence",
    ],
    "scanner output must expose operation materializer records and normalizer-compatible public declaration evidence",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplScannerStage0Summary:",
        "accepted %Result SelfhostMemoTraitPublicImplScannerAcceptedSummary SelfhostMemoTraitPublicImplScannerErrorKind",
        "missing_typed_record %Result i32 SelfhostMemoTraitPublicImplScannerErrorKind",
        "duplicate_typed_record %Result i32 SelfhostMemoTraitPublicImplScannerErrorKind",
        "unmatched_typed_record %Result i32 SelfhostMemoTraitPublicImplScannerErrorKind",
        "inherent_rejected %Result i32 SelfhostMemoTraitPublicImplScannerErrorKind",
        "mixed_alignment_first %Result i32 SelfhostMemoTraitPublicImplScannerErrorKind",
    ],
    "stage0 summary must include the mixed alignment-before-header regression",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplScannerErrorKind:",
        "ResolverRecordTableAllocFailed %StdErrorKind",
        "ResolverRecordPushFailed %StdErrorKind",
        "OperationRecordTableAllocFailed %StdErrorKind",
        "PublicDeclarationVectorAllocFailed %StdErrorKind",
        "OperationRecordPushRejected %SelfhostMemoTraitOperationPublicImplMaterializerErrorKind",
        "PublicDeclarationPushFailed %StdErrorKind",
        "AstItemUnavailable %i32",
        "DeclarationHeaderMissing %i32",
        "DeclarationKindMismatch %i32",
        "TypedRecordMissing %i32",
        "TypedRecordDuplicate %i32",
        "TypedRecordUnmatched %i32",
        "HeaderRejected %SelfhostMemoTraitPublicImplHeaderErrorKind",
    ],
    "scanner errors must distinguish setup, owner push, AST alignment, typed-record alignment, and header producer failures",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_scanner_scan_loop"),
    [
        "selfhost_module_ast_get ast idx",
        "selfhost_memo_trait_public_impl_scanner_item_public_ordinal item public_ordinal",
        "selfhost_memo_trait_public_impl_scanner_scan_item_result state records item public_ordinal",
        "selfhost_memo_trait_public_impl_scanner_scan_loop ast records next_state add idx 1 n next_public_ordinal",
    ],
    "scan loop must derive the next public declaration ordinal from AST visibility without using AST index as pairing key",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_scanner_process_public_impl_result"),
    [
        "selfhost_memo_trait_public_impl_scanner_find_record_result records ordinal",
        "selfhost_memo_trait_public_impl_scanner_header_input record",
        "selfhost_memo_trait_public_impl_header_evidence_result header_input",
        "selfhost_memo_trait_public_impl_scanner_push_output_result state record evidence",
        "HeaderRejected header_error",
    ],
    "public impl processing must find exactly one typed record, validate it through public impl header producer, then push both outputs",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_scanner_alignment_result"),
    [
        "selfhost_memo_trait_public_impl_scanner_alignment_scan_loop ast records",
        "selfhost_memo_trait_public_impl_scanner_unmatched_loop ast records",
    ],
    "alignment preflight must check public impl missing/duplicate before unmatched resolver records",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_scanner_result"),
    [
        "selfhost_memo_trait_public_impl_scanner_alignment_result ast records",
        "selfhost_memo_trait_public_impl_scanner_output_new_result",
        "selfhost_memo_trait_public_impl_scanner_scan_loop ast records state0",
    ],
    "scanner result must validate global ordinal alignment before allocating output owner and before header validation",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_scanner_operation_record"),
    [
        "selfhost_memo_trait_operation_public_impl_materializer_record_new",
        "record.type_id",
        "record.module_fingerprint",
        "record.declaration_ordinal",
        "record.visibility",
        "record.impl_kind",
        "record.target_type_shape_hash",
        "record.trait_source",
        "record.trait_type_argument_count",
        "record.trait_application_shape_hash",
        "record.type_parameter_count",
        "record.type_parameter_bound_count",
        "record.method_body_root",
        "record.fuel",
    ],
    "scanner must construct operation materializer records through the existing materializer record constructor and transport method roots without inspection",
);
assert.doesNotMatch(
    code,
    /selfhost_memo_trait_operation_classifier_evidence_result|selfhost_memo_trait_operation_impl_candidate_table_from_builder_inputs_result|SelfhostMemoTraitOperationEvidenceProducerInput|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitAggregateProofStatus|selfhost_memo_trait_operation_method_body_|selfhost_memo_trait_operation_drop_impl_|selfhost_memo_trait_operation_impl_candidate_new|selfhost_memo_trait_operation_impl_table_push/,
    "scanner must not classify operations, build candidates, create evidence records, aggregate proof status, method facts, or Drop facts",
);
assert.doesNotMatch(
    code,
    /hash32\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|mix\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|string_slice::|str_eq|\.path\b|\.alias\b|\.span\b|\.lexeme\b|display_name|diagnostic_text|method_name|trait_name/,
    "scanner implementation must not fold source text, spans, paths, aliases, display names, lexemes, method names, or trait names into accepted material",
);
assertOrdered(
    source,
    [
        "selfhost_memo_trait_public_impl_scanner_stage0",
        "selfhost_memo_trait_public_impl_scanner_accepted_len_eq",
        "selfhost_memo_trait_public_impl_scanner_accepted_public_ordinal_eq",
        "selfhost_memo_trait_public_impl_scanner_accepted_method_root_present",
        "selfhost_memo_trait_public_impl_scanner_error_ordinal_eq",
        "selfhost_memo_trait_public_impl_scanner_error_header_eq",
    ],
    "scanner must expose stage0 smoke and typed assertion helpers for accepted, missing, duplicate, unmatched, and header rejection paths",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_scanner_stage0_accepted"),
    [
        "selfhost_memo_trait_public_impl_scanner_stage0_ast_result",
        "selfhost_memo_trait_public_impl_scanner_stage0_record_table_result type_id registry.eq_source 2",
        "some root",
        "selfhost_memo_trait_public_impl_scanner_stage0_run_owned ast records 2 true",
    ],
    "stage0 accepted path must prove public ordinal pairing and method-root transport without body inspection",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_scanner_stage0_mixed_alignment_first"),
    [
        "selfhost_memo_trait_public_impl_scanner_stage0_single_impl_ast_result",
        "selfhost_memo_trait_public_impl_scanner_stage0_mixed_record_table_result type_id registry",
        "selfhost_memo_trait_public_impl_scanner_stage0_run_len_owned ast records",
    ],
    "stage0 mixed path must keep surplus typed record detection separate from header rejection",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_scanner_stage0_with_registry"),
    [
        "selfhost_memo_trait_public_impl_scanner_stage0_accepted",
        "selfhost_memo_trait_public_impl_scanner_stage0_missing",
        "selfhost_memo_trait_public_impl_scanner_stage0_duplicate",
        "selfhost_memo_trait_public_impl_scanner_stage0_unmatched",
        "selfhost_memo_trait_public_impl_scanner_stage0_inherent",
        "selfhost_memo_trait_public_impl_scanner_stage0_mixed_alignment_first",
    ],
    "stage0 must cover accepted public impl, missing typed record, duplicate typed record, unmatched typed record, inherited header rejection, and mixed alignment priority",
);
assert.doesNotMatch(
    source,
    /maxLine|line count limit|line-count limit|doc comment length cap|documentation length cap|行数制限|コメント量制限|ドキュメントコメント.{0,12}上限/,
    "scanner contract must not introduce line-count or documentation-comment length limits",
);

console.log("selfhost memo trait public impl scanner contract ok");
