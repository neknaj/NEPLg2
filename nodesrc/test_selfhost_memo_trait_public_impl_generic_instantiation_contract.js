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

function topLevelBlock(src, name) {
    const pattern = new RegExp(
        `^(?:pub\\s+)?(?:struct|enum)\\s+${name}:[\\s\\S]*?(?=\\n(?:pub\\s+)?(?:struct|enum|impl|fn)\\s|\\n//: [^\\n]*\\n(?:pub\\s+)?(?:struct|enum|impl|fn)\\s|\\s*$)`,
        "m",
    );
    const match = src.match(pattern);
    assert.ok(match, `${name} not found`);
    return match[0];
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

function assertNoDuplicateErrorKindCodes(src, functionName) {
    const block = functionBlock(src, functionName);
    const entries = [...block.matchAll(/ErrorKind::([A-Za-z0-9_]+)(?: _)?[^\n]*:\n\s+(\d+)/g)].map(
        (match) => ({ variant: match[1], code: Number(match[2]) }),
    );
    assert.ok(entries.length > 0, `${functionName} must expose comparable error kind codes`);
    const seen = new Map();
    for (const entry of entries) {
        const previous = seen.get(entry.code);
        assert.equal(
            previous,
            undefined,
            `${functionName} reuses code ${entry.code} for ${previous} and ${entry.variant}`,
        );
        seen.set(entry.code, entry.variant);
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_instantiation.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const materializerRelPath = "stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer.nepl";
const source = read(relPath);
const code = stripDocComments(source);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const materializer = read(materializerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_public_impl_generic_instantiation",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "generic instantiation module must document purpose, contract, current limits, complexity, and a doctest",
);
assert.ok(
    source.includes("count だけや display string だけで accepted path へ進む退行を防ぎます") &&
        source.includes("evidence struct は public value として構築可能なので") &&
        source.includes("substitution traversal の root/output TypeId、step count、step stream hash") &&
        source.includes("substitution producer が導出した root hash も field から再計算して照合します") &&
        source.includes("root / output TypeId は canonical type key ではありません") &&
        source.includes("pre-substitution target / trait application shape が public impl header の original shape と一致するかどうか、substituted output TypeId から canonical type key / final shape hash を作るかどうかは、materializer accepted path へ接続する後続 connector の責務です") &&
        source.includes("この evidence は generic impl candidate acceptance の前段です") &&
        source.includes("既存 materializer の `GenericImplInstantiationUnsupported` はこの slice では維持します"),
    "docs must reject count/display-only acceptance, explain public evidence revalidation, preserve substitution traversal components, require substitution hash recomputation, separate local TypeId links from canonical identity, explain deferred header-shape matching, separate evidence from semantic candidate acceptance, and keep the materializer fail-closed",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store record は accepted instantiation hash material に入りません"),
    "docs must exclude source, display, public-surface-hash, HIR, Resource IR, backend, and proof-store authority",
);
assert.doesNotMatch(
    facade,
    /memo_trait_public_impl_generic_instantiation/,
    "generic instantiation gate must remain facade-private until materializer and solver integration are complete",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_public_impl_generic_instantiation/,
    "checker-layer generic instantiation gate must not be registered in the ty source list",
);
assert.match(
    source,
    /^#import "neplg2\/core\/ty\/ty\/memo_trait_type_argument_identity" as \*$/m,
    "generic instantiation gate must consume stable type argument identity evidence",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_binder" as \*$/m,
    "generic instantiation gate must consume detailed generic binder evidence",
);
assert.match(
    source,
    /^#import "\.\/memo_trait_public_impl_generic_substitution_shape" as \*$/m,
    "generic instantiation gate must consume generic substitution shape evidence instead of raw substituted shape hashes",
);
assert.doesNotMatch(
    code,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_operation_impl_candidate_builder|memo_trait_operation_classifier|memo_trait_operation_evidence_producer|memo_trait_operation_purity_gate|memo_trait_operation_body_check_resolver|memo_trait_operation_method_body|memo_trait_operation_drop|memo_trait_public_impl_header|private_cache|private_state|prechecked|neplmeta|neplobj)/,
    "generic instantiation gate must not import HIR, Resource IR, backend, proof store, operation classifier/candidate/proof layers, public impl header, private effect layers, or prechecked artifact layers",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericBoundSolvingEvidence:",
        "schema_version %i32",
        "solved_bound_count %i32",
        "solver_policy_hash %i32",
        "proof_shape_hash %i32",
    ],
    "bound solving evidence must carry schema, solved bound count, solver policy hash, and typed proof shape hash",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplGenericBoundSolvingStatus:",
        "NoBounds",
        "AllSolved %SelfhostMemoTraitPublicImplGenericBoundSolvingEvidence",
        "Unsolved %i32",
    ],
    "bound solving status must be an enum with no-bound, all-solved, and unsolved states",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericInstantiationInput:",
        "generic_binder_evidence %SelfhostMemoTraitPublicImplGenericBinderEvidence",
        "type_argument_count %i32",
        "type_argument_identity_hash %SelfhostMemoTraitStableTypeArgumentIdentityHash",
        "substitution_shape_evidence %SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence",
        "bound_solving_status %SelfhostMemoTraitPublicImplGenericBoundSolvingStatus",
    ],
    "instantiation input must keep binder evidence, type argument identity, substitution shape evidence, and bound solving status as distinct typed fields",
);
assert.doesNotMatch(
    topLevelBlock(source, "SelfhostMemoTraitPublicImplGenericInstantiationInput"),
    /substituted_target_type_shape_hash %Option i32|substituted_trait_application_shape_hash %Option i32/,
    "instantiation input must not accept raw substituted shape options after the substitution shape evidence producer exists",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitPublicImplGenericInstantiationEvidence:",
        "schema_version %i32",
        "type_parameter_count %i32",
        "type_argument_count %i32",
        "type_parameter_bound_count %i32",
        "generic_binder_shape_hash %i32",
        "generic_parameter_table_shape_hash %i32",
        "generic_bound_table_shape_hash %i32",
        "type_argument_identity_hash %SelfhostMemoTraitStableTypeArgumentIdentityHash",
        "substitution_shape_hash %i32",
        "substitution_trace_shape_hash %i32",
        "target_substitution_root_type_id %SelfhostTypeId",
        "target_substitution_output_type_id %SelfhostTypeId",
        "target_substitution_step_count %i32",
        "target_substitution_step_stream_hash %i32",
        "substituted_target_type_shape_hash %i32",
        "trait_application_substitution_root_type_id %SelfhostTypeId",
        "trait_application_substitution_output_type_id %SelfhostTypeId",
        "trait_application_substitution_step_count %i32",
        "trait_application_substitution_step_stream_hash %i32",
        "substituted_trait_application_shape_hash %i32",
        "bound_solution_shape_hash %i32",
        "instantiation_shape_hash %i32",
    ],
    "accepted evidence must preserve separate hashes, traversal TypeId links, step counts, and step hashes instead of collapsing everything to a single untyped hash",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitPublicImplGenericInstantiationErrorKind:",
        "BinderEvidenceSchemaPlaceholder",
        "BinderEvidenceHashPlaceholder",
        "TypeArgumentCountNegative",
        "TypeArgumentCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "BinderEvidenceParameterTableHashPlaceholder",
        "BinderEvidenceBoundTableHashPlaceholder",
        "TypeArgumentIdentitySchemaPlaceholder",
        "TypeArgumentIdentityHashPlaceholder",
        "SubstitutionShapeSchemaPlaceholder",
        "SubstitutionShapeSchemaMismatch",
        "SubstitutionShapeHashPlaceholder",
        "SubstitutionShapeHashMismatch",
        "SubstitutionShapeTypeParameterCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "SubstitutionShapeTypeArgumentCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "SubstitutionShapeTypeParameterBoundCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "SubstitutionShapeBinderHashMismatch",
        "SubstitutionShapeTypeArgumentIdentitySchemaPlaceholder",
        "SubstitutionShapeTypeArgumentIdentityHashPlaceholder",
        "SubstitutionShapeTypeArgumentIdentitySchemaMismatch",
        "SubstitutionShapeTypeArgumentIdentityHashMismatch",
        "SubstitutionShapePreTargetTypeShapeHashPlaceholder",
        "SubstitutionShapePreTraitApplicationShapeHashPlaceholder",
        "SubstitutionTraceShapeHashPlaceholder",
        "SubstitutionShapeTargetRootTypeIdInvalid",
        "SubstitutionShapeTargetOutputTypeIdInvalid",
        "SubstitutionShapeTargetStepCountMissing",
        "SubstitutionShapeTargetStepStreamHashPlaceholder",
        "SubstitutionShapeTargetTypeShapeHashPlaceholder",
        "SubstitutionShapeTraitApplicationRootTypeIdInvalid",
        "SubstitutionShapeTraitApplicationOutputTypeIdInvalid",
        "SubstitutionShapeTraitApplicationStepCountMissing",
        "SubstitutionShapeTraitApplicationStepStreamHashPlaceholder",
        "SubstitutionShapeTraitApplicationShapeHashPlaceholder",
        "BoundCountNegative",
        "BoundSolvingRequired",
        "BoundSolvingUnexpected",
        "BoundSolvingUnsolved %i32",
        "BoundSolvingSchemaPlaceholder",
        "BoundSolvingCountMismatch %SelfhostMemoTraitPublicImplGenericBinderCountMismatch",
        "BoundSolvingPolicyHashPlaceholder",
        "BoundSolvingProofShapeHashPlaceholder",
        "DerivedBoundSolutionShapeHashPlaceholder",
        "DerivedInstantiationShapeHashPlaceholder",
    ],
    "errors must preserve binder, type-argument, substituted-shape, bound-solving, and derived-hash failures as typed variants",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_binder_hash_result"),
    [
        "eq evidence.schema_version 0",
        "BinderEvidenceSchemaPlaceholder",
        "eq evidence.parameter_table_shape_hash 0",
        "BinderEvidenceParameterTableHashPlaceholder",
        "eq evidence.bound_table_shape_hash 0",
        "BinderEvidenceBoundTableHashPlaceholder",
        "eq evidence.shape_hash 0",
        "BinderEvidenceHashPlaceholder",
        "Result::Ok evidence.shape_hash",
    ],
    "binder gate must reject schema, parameter table hash, bound table hash, and root hash placeholders separately",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_substitution_shape_final_result"),
    [
        "eq evidence.substituted_trait_application_shape_hash 0",
        "SubstitutionShapeTraitApplicationShapeHashPlaceholder",
        "selfhost_memo_trait_public_impl_generic_substitution_shape_evidence_hash evidence",
        "not eq recomputed_shape_hash evidence.substitution_shape_hash",
        "SubstitutionShapeHashMismatch",
        "Result::Ok evidence",
    ],
    "instantiation final substitution-shape gate must reject missing substituted trait shape and recompute the substitution root hash from evidence fields",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_substitution_shape_result"),
    [
        "eq evidence.schema_version 0",
        "SubstitutionShapeSchemaPlaceholder",
        "not eq evidence.schema_version selfhost_memo_trait_public_impl_generic_substitution_shape_schema_version",
        "SubstitutionShapeSchemaMismatch",
        "eq evidence.substitution_shape_hash 0",
        "SubstitutionShapeHashPlaceholder",
        "not eq evidence.type_parameter_count binder.type_parameter_count",
        "SubstitutionShapeTypeParameterCountMismatch",
        "not eq evidence.type_argument_count type_argument_count",
        "SubstitutionShapeTypeArgumentCountMismatch",
        "not eq evidence.type_parameter_bound_count binder.type_parameter_bound_count",
        "SubstitutionShapeTypeParameterBoundCountMismatch",
        "not eq evidence.generic_binder_shape_hash binder_hash",
        "SubstitutionShapeBinderHashMismatch",
        "not eq evidence.generic_parameter_table_shape_hash binder.parameter_table_shape_hash",
        "SubstitutionShapeBinderHashMismatch",
        "not eq evidence.generic_bound_table_shape_hash binder.bound_table_shape_hash",
        "SubstitutionShapeBinderHashMismatch",
        "eq evidence.type_argument_identity_hash.schema_version 0",
        "SubstitutionShapeTypeArgumentIdentitySchemaPlaceholder",
        "eq evidence.type_argument_identity_hash.identity_hash 0",
        "SubstitutionShapeTypeArgumentIdentityHashPlaceholder",
        "not eq evidence.type_argument_identity_hash.schema_version type_argument_hash.schema_version",
        "SubstitutionShapeTypeArgumentIdentitySchemaMismatch",
        "not eq evidence.type_argument_identity_hash.identity_hash type_argument_hash.identity_hash",
        "SubstitutionShapeTypeArgumentIdentityHashMismatch",
        "eq evidence.pre_substitution_target_type_shape_hash 0",
        "SubstitutionShapePreTargetTypeShapeHashPlaceholder",
        "eq evidence.pre_substitution_trait_application_shape_hash 0",
        "SubstitutionShapePreTraitApplicationShapeHashPlaceholder",
        "eq evidence.substitution_trace_shape_hash 0",
        "SubstitutionTraceShapeHashPlaceholder",
        "lt selfhost_type_id_index evidence.target_substitution_root_type_id 0",
        "SubstitutionShapeTargetRootTypeIdInvalid",
        "lt selfhost_type_id_index evidence.target_substitution_output_type_id 0",
        "SubstitutionShapeTargetOutputTypeIdInvalid",
        "lt evidence.target_substitution_step_count 1",
        "SubstitutionShapeTargetStepCountMissing",
        "eq evidence.target_substitution_step_stream_hash 0",
        "SubstitutionShapeTargetStepStreamHashPlaceholder",
        "eq evidence.substituted_target_type_shape_hash 0",
        "SubstitutionShapeTargetTypeShapeHashPlaceholder",
        "lt selfhost_type_id_index evidence.trait_application_substitution_root_type_id 0",
        "SubstitutionShapeTraitApplicationRootTypeIdInvalid",
        "lt selfhost_type_id_index evidence.trait_application_substitution_output_type_id 0",
        "SubstitutionShapeTraitApplicationOutputTypeIdInvalid",
        "lt evidence.trait_application_substitution_step_count 1",
        "SubstitutionShapeTraitApplicationStepCountMissing",
        "eq evidence.trait_application_substitution_step_stream_hash 0",
        "SubstitutionShapeTraitApplicationStepStreamHashPlaceholder",
        "selfhost_memo_trait_public_impl_generic_instantiation_substitution_shape_final_result evidence",
    ],
    "instantiation gate must re-check schema, root hash, counts, binder hash, type argument identity, trace, traversal TypeId links, step counts, step hashes, and then delegate final substituted-shape/root-hash verification",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_bound_solution_hash_result"),
    [
        "lt expected_bound_count 0",
        "BoundCountNegative",
        "SelfhostMemoTraitPublicImplGenericBoundSolvingStatus::NoBounds:",
        "eq expected_bound_count 0",
        "selfhost_memo_trait_public_impl_generic_instantiation_no_bounds_hash_result",
        "BoundSolvingRequired",
        "SelfhostMemoTraitPublicImplGenericBoundSolvingStatus::AllSolved evidence:",
        "eq expected_bound_count 0",
        "BoundSolvingUnexpected",
        "selfhost_memo_trait_public_impl_generic_instantiation_solved_bound_hash_result expected_bound_count evidence",
        "SelfhostMemoTraitPublicImplGenericBoundSolvingStatus::Unsolved first_unsolved:",
        "BoundSolvingUnsolved first_unsolved",
    ],
    "bound solution gate must reject negative counts, require no-bound/all-solved consistency, and preserve unsolved ordinal",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_solved_bound_hash_result"),
    [
        "eq evidence.schema_version 0",
        "BoundSolvingSchemaPlaceholder",
        "not eq evidence.solved_bound_count expected_bound_count",
        "BoundSolvingCountMismatch",
        "eq evidence.solver_policy_hash 0",
        "BoundSolvingPolicyHashPlaceholder",
        "eq evidence.proof_shape_hash 0",
        "BoundSolvingProofShapeHashPlaceholder",
        "DerivedBoundSolutionShapeHashPlaceholder",
        "Result::Ok hash",
    ],
    "all-solved evidence must validate schema, solved count, solver policy hash, proof shape hash, and nonzero derived hash",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_evidence_result"),
    [
        "lt input.type_argument_count 0",
        "TypeArgumentCountNegative",
        "not eq input.generic_binder_evidence.type_parameter_count input.type_argument_count",
        "TypeArgumentCountMismatch",
        "selfhost_memo_trait_public_impl_generic_instantiation_binder_hash_result input.generic_binder_evidence",
        "selfhost_memo_trait_public_impl_generic_instantiation_type_argument_identity_result input.type_argument_identity_hash",
        "selfhost_memo_trait_public_impl_generic_instantiation_substitution_shape_result input.generic_binder_evidence input.type_argument_count type_argument_hash binder_hash input.substitution_shape_evidence",
        "selfhost_memo_trait_public_impl_generic_instantiation_bound_solution_hash_result input.generic_binder_evidence.type_parameter_bound_count input.bound_solving_status",
        "type_argument_hash.schema_version",
        "type_argument_hash.identity_hash",
        "target_substitution_id_material",
        "target_substitution_step_material",
        "trait_substitution_id_material",
        "trait_substitution_step_material",
        "substitution_traversal_material",
        "substitution_evidence.substitution_shape_hash",
        "substitution_evidence.substituted_target_type_shape_hash",
        "substitution_evidence.substituted_trait_application_shape_hash",
        "input.generic_binder_evidence.parameter_table_shape_hash",
        "input.generic_binder_evidence.bound_table_shape_hash",
        "DerivedInstantiationShapeHashPlaceholder",
        "SelfhostMemoTraitPublicImplGenericInstantiationEvidence schema input.generic_binder_evidence.type_parameter_count input.type_argument_count input.generic_binder_evidence.type_parameter_bound_count binder_hash input.generic_binder_evidence.parameter_table_shape_hash input.generic_binder_evidence.bound_table_shape_hash",
    ],
    "evidence API must validate argument count, binder evidence, type argument identity, substitution evidence, bound solving, and nonzero instantiation hash before success",
);
assertNoDuplicateErrorKindCodes(
    source,
    "selfhost_memo_trait_public_impl_generic_instantiation_error_kind_code",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_error_kind_eq"),
    [
        "TypeArgumentCountMismatch mismatch:",
        "TypeArgumentCountMismatch other:",
        "selfhost_memo_trait_public_impl_generic_instantiation_count_mismatch_eq mismatch other",
        "SubstitutionShapeTypeParameterCountMismatch mismatch:",
        "SubstitutionShapeTypeParameterCountMismatch other:",
        "selfhost_memo_trait_public_impl_generic_instantiation_count_mismatch_eq mismatch other",
        "SubstitutionShapeTypeArgumentCountMismatch mismatch:",
        "SubstitutionShapeTypeArgumentCountMismatch other:",
        "selfhost_memo_trait_public_impl_generic_instantiation_count_mismatch_eq mismatch other",
        "SubstitutionShapeTypeParameterBoundCountMismatch mismatch:",
        "SubstitutionShapeTypeParameterBoundCountMismatch other:",
        "selfhost_memo_trait_public_impl_generic_instantiation_count_mismatch_eq mismatch other",
        "BoundSolvingCountMismatch mismatch:",
        "BoundSolvingCountMismatch other:",
        "selfhost_memo_trait_public_impl_generic_instantiation_count_mismatch_eq mismatch other",
        "BoundSolvingUnsolved first:",
        "BoundSolvingUnsolved other:",
        "eq first other",
    ],
    "error equality must compare payloads for count mismatches and unsolved-bound ordinals",
);
assert.match(
    functionBlock(source, "selfhost_memo_trait_public_impl_generic_instantiation_stage0"),
    /accepted[\s\S]*no_bounds_accepted[\s\S]*type_argument_mismatch[\s\S]*identity_placeholder[\s\S]*substitution_shape_placeholder[\s\S]*substitution_binder_hash_mismatch[\s\S]*substitution_identity_mismatch[\s\S]*substitution_target_placeholder[\s\S]*bound_required[\s\S]*bound_unsolved[\s\S]*bound_count_mismatch/,
    "stage0 must exercise accepted, no-bound, argument mismatch, identity placeholder, substitution shape placeholder, binder mismatch, identity mismatch, target placeholder, required-bound, unsolved-bound, and bound count mismatch cases",
);
assert.doesNotMatch(
    code,
    /selfhost_memo_trait_public_impl_generic_instantiation_(?:target|trait)_shape_result|input\.substituted_target_type_shape_hash|input\.substituted_trait_application_shape_hash/,
    "instantiation gate must not keep the old raw substituted target/trait shape helper path",
);
assert.match(
    materializer,
    /GenericImplInstantiationUnsupported/,
    "materializer must still reject detailed generic records until this gate is wired to actual solver and substitution producers",
);
assert.doesNotMatch(
    code,
    /SelfhostMemoTraitOperationImplCandidate|SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitAggregateProofStatus|NoDropRequired|PureDrop|PrivateCache|PrivateState|memo_call|SourceBacked|public_surface_hash|hash32\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|mix\s+(?:source|span|path|alias|display|diag|diagnostic|lexeme)|\.path\b|\.alias\b|\.span\b|\.lexeme\b|display_name|diagnostic_text|source_path/,
    "generic instantiation gate must not fabricate candidates, operation evidence, aggregate proof status, private effects, memo_call acceptance, public surface hash authority, or source-derived hash material",
);
assert.doesNotMatch(
    topLevelBlock(source, "SelfhostMemoTraitPublicImplGenericInstantiationEvidence"),
    /source|span|path|display|diagnostic|lexeme|DefId|FileId/,
    "accepted evidence must not store source/display authority or non-type compiler-local ids",
);
assert.doesNotMatch(
    source,
    /maxLine|line count limit|line-count limit|doc comment length cap|documentation length cap|行数制限|コメント量制限|ドキュメントコメント.{0,12}上限/,
    "generic instantiation gate must not introduce line-count or documentation-comment length limits",
);

console.log("selfhost memo trait public impl generic instantiation contract ok");
