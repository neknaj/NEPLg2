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

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_bound_solver.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const materializerRelPath =
    "stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer.nepl";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const materializer = read(materializerRelPath);
const contractSource = fs.readFileSync(__filename, "utf8").replace(/\r\n/g, "\n");
const proseCapPattern = new RegExp(
    [
        ["max", "Lines"].join(""),
        ["max", "_lines"].join(""),
        ["line", "Limit"].join(""),
        ["line", "_limit"].join(""),
        ["doc", "Comment", "Limit"].join(""),
        ["doc", "_comment", "_limit"].join(""),
        ["doc", "Comment", "Max"].join(""),
        ["doc", "_comment", "_max"].join(""),
        ["documentation", "Line", "Limit"].join(""),
        ["documentation", "_line", "_limit"].join(""),
        ["lines", "\\.length\\s*[<>]=?\\s*\\d+"].join(""),
    ].join("|"),
);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_public_impl_generic_bound_solver",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "generic bound solver module must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("caller が手で作った `AllSolved` をそのまま accepted authority にすると") &&
        source.includes("source text、span、lexeme、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store record、prechecked artifact は bound solving material に入りません") &&
        source.includes("`memo_trait_operation_public_impl_materializer` の `GenericImplInstantiationUnsupported` はこの slice では維持します") &&
        source.includes("actual trait selection、where 条件の探索、generic blanket impl の coherence 判定はまだ行いません"),
    "docs must explain the caller-supplied AllSolved hazard, authority exclusions, fail-closed materializer boundary, and current solver/coherence limits",
);
assert.doesNotMatch(contractSource, proseCapPattern, "bound solver source policy must not add prose-volume caps");
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_binder" as \*$/m,
    "bound solver must consume detailed generic binder tables",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_instantiation" as \*$/m,
    "bound solver must produce the existing instantiation gate bound solving status",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_operation_impl_candidate_builder|memo_trait_operation_classifier|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop|memo_trait_public_impl_header|private_cache|private_state|prechecked|neplmeta|neplobj)/,
    "bound solver must not import HIR, Resource IR, backend, proof store, operation candidate/proof layers, public impl header, private effect layers, or prechecked artifact layers",
);
assert.doesNotMatch(
    code,
    /\b(?:display_name|source_path|source_text|span|lexeme|diagnostic_text|module_path|public_surface_hash|SelfhostSource|SourceSpan|SourceText|Lexeme)\b|hash32\s+(?:source|span|lexeme|display|module|diagnostic)|mix[0-9]*\s+(?:source|span|lexeme|display|module|diagnostic)/,
    "accepted bound solving evidence must not derive authority from source, display, span, diagnostic, module-path, or public-surface material",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_generic_bound_solver/,
    "generic bound solver must remain facade-private until solver/coherence/materializer integration is complete",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_generic_bound_solver/,
    "checker-layer generic bound solver must not be registered in the ty source list before accepted path integration",
);
assert.match(
    materializer,
    /GenericImplInstantiationUnsupported/,
    "materializer must still reject detailed generic records until bound solver, coherence, and connector evidence are wired into the accepted path",
);
assert.doesNotMatch(
    materializer,
    /GenericBoundSolver|memo_trait_public_impl_generic_bound_solver/,
    "materializer must not consume the new bound solver in this slice",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplGenericBoundSolverProofStatus:",
        "Proven",
        "Missing",
        "Refuted",
        "Unknown",
    ],
    "proof status must distinguish proven, missing, refuted, and unknown states",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericBoundSolverProofRecord:",
        "bound_ordinal %i32",
        "parameter_ordinal %i32",
        "trait_application_shape_hash %i32",
        "trait_type_argument_count %i32",
        "status %SelfhostMemoTraitPublicImplGenericBoundSolverProofStatus",
        "proof_shape_hash %i32",
    ],
    "proof records must preserve bound ordinal, parameter ordinal, trait application shape, trait type argument count, typed status, and proof shape hash",
);
assert.doesNotMatch(
    source,
    /impl Clone for SelfhostMemoTraitPublicImplGenericBoundSolverProofTable/,
    "proof table owns a Vec buffer and must not provide a shallow Clone implementation",
);
assert.doesNotMatch(
    source,
    /impl Copy for SelfhostMemoTraitPublicImplGenericBoundSolverProofTable/,
    "proof table owns a Vec buffer and must not provide a Copy implementation",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplGenericBoundSolverErrorKind:",
        "BinderEvidenceSameOriginRejected %SelfhostMemoTraitPublicImplGenericBinderErrorKind",
        "ProofRecordCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "ProofBoundOrdinalMismatch %SelfhostMemoTraitPublicImplGenericBinderOrdinalMismatch",
        "ProofParameterOrdinalMismatch %SelfhostMemoTraitPublicImplGenericBinderOrdinalMismatch",
        "ProofTraitApplicationShapeMismatch %SelfhostMemoTraitPublicImplGenericBoundSolverShapeMismatch",
        "SolverPolicyHashPlaceholder",
        "ProofShapeHashPlaceholder %i32",
        "ProofStatusMissing %i32",
        "ProofStatusRefuted %i32",
        "ProofStatusUnknown %i32",
        "DerivedProofShapeHashPlaceholder",
    ],
    "bound solver errors must keep same-origin, count, ordinal, shape, policy, proof hash, and non-proven status failures as typed variants",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_bound_solver_status_result"),
    [
        "selfhost_memo_trait_public_impl_generic_binder_evidence_same_origin_result binder_evidence parameters bounds",
        "let bound_count %i32 same_origin_binder.type_parameter_bound_count",
        "let proof_count %i32 selfhost_memo_trait_public_impl_generic_bound_solver_proof_table_len proofs",
        "eq bound_count 0",
        "Result::Ok selfhost_memo_trait_public_impl_generic_bound_solving_no_bounds",
        "ProofRecordCountMismatch",
        "eq solver_policy_hash 0",
        "SolverPolicyHashPlaceholder",
        "selfhost_memo_trait_public_impl_generic_bound_solver_proof_hash_loop bounds proofs 0 bound_count",
        "selfhost_memo_trait_public_impl_generic_bound_solving_evidence_new selfhost_memo_trait_public_impl_generic_bound_solver_schema_version bound_count solver_policy_hash proof_shape_hash",
        "selfhost_memo_trait_public_impl_generic_bound_solving_all_solved evidence",
        "BinderEvidenceSameOriginRejected",
    ],
    "status producer must recheck binder origin, split no-bounds/all-solved, reject count mismatch, require solver policy, fold proof records, and return existing bound solving status",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_bound_solver_record_hash_result"),
    [
        "not eq proof.bound_ordinal bound.bound_ordinal",
        "ProofBoundOrdinalMismatch",
        "not eq proof.parameter_ordinal bound.parameter_ordinal",
        "ProofParameterOrdinalMismatch",
        "not eq proof.trait_application_shape_hash bound_shape",
        "ProofTraitApplicationShapeMismatch",
        "not eq proof.trait_type_argument_count bound.trait_type_argument_count",
        "ProofTraitTypeArgumentCountMismatch",
        "SelfhostMemoTraitPublicImplGenericBoundSolverProofStatus::Proven",
        "eq proof.proof_shape_hash 0",
        "ProofShapeHashPlaceholder",
        "SelfhostMemoTraitPublicImplGenericBoundSolverProofStatus::Missing",
        "ProofStatusMissing",
        "SelfhostMemoTraitPublicImplGenericBoundSolverProofStatus::Refuted",
        "ProofStatusRefuted",
        "SelfhostMemoTraitPublicImplGenericBoundSolverProofStatus::Unknown",
        "ProofStatusUnknown",
    ],
    "record validation must reject ordinal, shape, count, placeholder hash, and non-proven proof statuses before producing proof material",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_bound_solver_error_kind_eq"),
    [
        "BinderEvidenceSameOriginRejected",
        "selfhost_memo_trait_public_impl_generic_binder_error_kind_eq error other",
        "ProofRecordCountMismatch",
        "selfhost_memo_trait_public_impl_generic_bound_solver_count_mismatch_eq mismatch other",
        "ProofTraitApplicationShapeMismatch",
        "selfhost_memo_trait_public_impl_generic_bound_solver_shape_mismatch_eq mismatch other",
        "ProofStatusMissing",
        "eq ordinal other",
    ],
    "error equality must remain payload-aware for nested binder errors, count mismatches, shape mismatches, and status ordinals",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericBoundSolverStage0Summary:",
        "accepted %Result SelfhostMemoTraitPublicImplGenericBoundSolvingStatus SelfhostMemoTraitPublicImplGenericBoundSolverErrorKind",
        "no_bounds_accepted %Result SelfhostMemoTraitPublicImplGenericBoundSolvingStatus SelfhostMemoTraitPublicImplGenericBoundSolverErrorKind",
        "proof_count_mismatch %Result SelfhostMemoTraitPublicImplGenericBoundSolvingStatus SelfhostMemoTraitPublicImplGenericBoundSolverErrorKind",
        "proof_status_missing %Result SelfhostMemoTraitPublicImplGenericBoundSolvingStatus SelfhostMemoTraitPublicImplGenericBoundSolverErrorKind",
        "proof_shape_mismatch %Result SelfhostMemoTraitPublicImplGenericBoundSolvingStatus SelfhostMemoTraitPublicImplGenericBoundSolverErrorKind",
        "policy_placeholder %Result SelfhostMemoTraitPublicImplGenericBoundSolvingStatus SelfhostMemoTraitPublicImplGenericBoundSolverErrorKind",
        "proof_hash_placeholder %Result SelfhostMemoTraitPublicImplGenericBoundSolvingStatus SelfhostMemoTraitPublicImplGenericBoundSolverErrorKind",
    ],
    "stage0 summary must cover accepted, no-bounds, count mismatch, non-proven, shape mismatch, policy placeholder, and proof hash placeholder cases",
);

console.log("selfhost memo trait public impl generic bound solver contract: ok");
