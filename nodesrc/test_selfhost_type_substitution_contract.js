#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const assert = require("assert");

const repo = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/substitution.nepl";
const facadeRelPath = "stdlib/neplg2/core/ty/ty.nepl";
const tySourcesRelPath = "nodesrc/selfhost_ty_sources.js";
const sourcePolicyRunnerRelPath = "nodesrc/run_source_policy_regressions.js";
const source = fs.readFileSync(path.join(repo, relPath), "utf8");
const facade = fs.readFileSync(path.join(repo, facadeRelPath), "utf8");
const tySources = fs.readFileSync(path.join(repo, tySourcesRelPath), "utf8");
const sourcePolicyRunner = fs.readFileSync(path.join(repo, sourcePolicyRunnerRelPath), "utf8");

function withoutDoc(src) {
    return src
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
}

function functionBlock(src, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(pub\\s+)?fn\\s+${name}\\b`);
    const topLevel = /^(pub\s+)?(fn|struct|enum|impl)\s+/;
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notStrictEqual(start, -1, `missing function ${name}`);
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

const code = withoutDoc(source);

assert(
    source.includes("# ty/substitution") &&
        source.includes("[目的/もくてき]") &&
        source.includes("[契約/けいやく]") &&
        source.includes("[現状/げんじょう]") &&
        source.includes("[計算量/けいさんりょう]") &&
        source.includes("neplg2:test"),
    "type substitution module must document purpose, contract, current limits, complexity, and doctest",
);

assert(
    source.includes("actual type traversal を通らない値") &&
        source.includes("typed step stream") &&
        source.includes("source text、span、lexeme、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store record は substitution authority にしません") &&
        source.includes("trait bound solver、generic coherence、operation candidate materializer"),
    "docs must explain the raw-hash hazard, typed traversal authority, forbidden authority, and deferred materializer/solver boundary",
);

assert(
    /^pub #import "\.\/ty\/substitution" as \*$/m.test(facade),
    "ty facade must export the reusable substitution engine",
);

assert(
    tySources.includes(`"${relPath}"`) &&
        sourcePolicyRunner.includes('"nodesrc/test_selfhost_type_substitution_contract.js"'),
    "substitution module and contract must be registered in selfhost ty source lists and source policy regression runner",
);

assert(
    !/#import .*check\/module|#import .*hir|#import .*resource|#import .*backend|#import .*proof|#import .*memo_trait_public_impl/m.test(code),
    "core type substitution module must not depend on checker/module, HIR, Resource IR, backend, proof, or memo public impl layers",
);

[
    "pub struct SelfhostTypeSubstitutionBindingRecord:",
    "pub struct SelfhostTypeSubstitutionBindingTable:",
    "pub enum SelfhostTypeSubstitutionStepKind:",
    "pub struct SelfhostTypeSubstitutionStepRecord:",
    "pub struct SelfhostTypeSubstitutionStepTable:",
    "pub struct SelfhostTypeSubstitutionEvidence:",
    "pub struct SelfhostTypeSubstitutionResult:",
    "pub enum SelfhostTypeSubstitutionErrorKind:",
].forEach((needle) => {
    assert(source.includes(needle), `missing ${needle}`);
});

[
    "PrimitiveKept",
    "NamedKept",
    "ParameterKept",
    "ParameterReplaced",
    "AppliedRebuilt",
    "FunctionRebuilt",
].forEach((variant) => {
    assert(source.includes(variant), `step kind must include ${variant}`);
});

[
    "BindingRecordReadFailed %i32",
    "InvalidBinding %SelfhostTypeParameterBinding",
    "DuplicateBinding %SelfhostTypeParameterBinding",
    "ReplacementTypeMissing %SelfhostTypeId",
    "MissingSourceTypeRecord %SelfhostTypeId",
    "MissingAppliedTypeArgument %SelfhostTypeId",
    "MissingFunctionTypeArgument %SelfhostTypeId",
    "MissingFunctionResult %SelfhostTypeId",
    "TraversalFuelExhausted %SelfhostTypeId",
    "StepStreamHashPlaceholder",
    "EvidenceHashMismatch",
].forEach((variant) => {
    assert(source.includes(variant), `error enum must include ${variant}`);
});

assert(
    functionBlock(source, "selfhost_type_substitution_binding_table_validate_result").includes("selfhost_type_substitution_binding_table_validate_loop"),
    "substitution must validate the binding table before traversal",
);

assert(
    functionBlock(source, "selfhost_type_substitution_binding_table_validate_loop").includes("selfhost_type_substitution_binding_record_validate_result") &&
        functionBlock(source, "selfhost_type_substitution_binding_table_validate_loop").includes("selfhost_type_substitution_binding_table_duplicate_probe") &&
        functionBlock(source, "selfhost_type_substitution_binding_table_validate_loop").includes("BindingRecordReadFailed idx"),
    "binding validation must reject invalid bindings, missing replacements, duplicates, and structurally broken record reads",
);

assert(
    functionBlock(source, "selfhost_type_substitution_binding_table_find_loop").includes("BindingRecordReadFailed idx") &&
        functionBlock(source, "selfhost_type_substitution_binding_table_duplicate_probe").includes("BindingRecordReadFailed idx"),
    "binding table find and duplicate probes must fail closed when idx is in range but record read fails",
);

assert(
    functionBlock(source, "selfhost_type_substitution_substitute_type_result").includes("SelfhostTypeRecord::Parameter parameter") &&
        functionBlock(source, "selfhost_type_substitution_substitute_type_result").includes("SelfhostTypeRecord::Applied applied") &&
        functionBlock(source, "selfhost_type_substitution_substitute_type_result").includes("SelfhostTypeRecord::Function function") &&
        functionBlock(source, "selfhost_type_substitution_substitute_type_result").includes("TraversalFuelExhausted"),
    "substitution core must dispatch on typed TypeRecord variants and guard traversal fuel",
);

assert(
    functionBlock(source, "selfhost_type_substitution_substitute_applied_result").includes("selfhost_type_substitution_applied_args_loop") &&
        functionBlock(source, "selfhost_type_substitution_substitute_applied_result").includes("selfhost_type_arena_add_applied_named") &&
        functionBlock(source, "selfhost_type_substitution_substitute_applied_result").includes("SelfhostTypeSubstitutionStepKind::AppliedRebuilt") &&
        functionBlock(source, "selfhost_type_substitution_substitute_applied_result").includes("selfhost_type_substitution_node_fail_after_arena_builder_consumed") &&
        functionBlock(source, "selfhost_type_substitution_substitute_applied_result").includes("selfhost_type_substitution_node_fail_after_step_table_consumed"),
    "applied type substitution must rebuild from substituted child TypeIds, record a typed step, and respect consumed-owner failure boundaries",
);

assert(
    functionBlock(source, "selfhost_type_substitution_substitute_function_result").includes("selfhost_type_substitution_function_args_loop") &&
        functionBlock(source, "selfhost_type_substitution_substitute_function_result").includes("selfhost_type_substitution_substitute_type_result build_arena build_steps bindings function.result") &&
        functionBlock(source, "selfhost_type_substitution_substitute_function_result").includes("selfhost_type_arena_add_function") &&
        functionBlock(source, "selfhost_type_substitution_substitute_function_result").includes("SelfhostTypeSubstitutionStepKind::FunctionRebuilt") &&
        functionBlock(source, "selfhost_type_substitution_substitute_function_result").includes("selfhost_type_substitution_node_fail_after_arena_builder_consumed") &&
        functionBlock(source, "selfhost_type_substitution_substitute_function_result").includes("selfhost_type_substitution_node_fail_after_step_table_consumed"),
    "function type substitution must rebuild argument/result types, record a typed step, and respect consumed-owner failure boundaries",
);

assert(
    functionBlock(source, "selfhost_type_substitution_applied_args_loop").includes("selfhost_type_substitution_arg_build_fail_after_args_consumed") &&
        functionBlock(source, "selfhost_type_substitution_function_args_loop").includes("selfhost_type_substitution_arg_build_fail_after_args_consumed") &&
        source.includes("selfhost_type_substitution_node_fail_after_step_table_consumed") &&
        source.includes("selfhost_type_substitution_node_fail_after_arena_builder_consumed"),
    "argument and step failure paths must not reuse owners consumed by Vec push, step push, or arena rebuild helpers",
);

assert(
    functionBlock(source, "selfhost_type_substitution_step_table_hash_result").includes("selfhost_type_substitution_step_table_hash_loop") &&
        functionBlock(source, "selfhost_type_substitution_evidence_new_result").includes("selfhost_type_substitution_step_table_hash_result") &&
        source.includes("step_stream_hash %i32"),
    "substitution evidence must be backed by a rehashable typed step stream",
);

assert(
    functionBlock(source, "selfhost_type_substitution_result").includes("selfhost_type_substitution_binding_table_validate_result &arena bindings") &&
        functionBlock(source, "selfhost_type_substitution_result").includes("selfhost_type_substitution_substitute_type_result arena step_table bindings root_type_id") &&
        functionBlock(source, "selfhost_type_substitution_result").includes("selfhost_type_substitution_evidence_new_result &next_steps root_type_id output_type_id"),
    "public result API must validate, perform actual traversal, and derive evidence from the step table",
);

assert(
    functionBlock(source, "selfhost_type_substitution_error_kind_code").includes("BindingRecordReadFailed _idx") &&
        functionBlock(source, "selfhost_type_substitution_error_kind_payload0").includes("BindingRecordReadFailed idx") &&
        functionBlock(source, "selfhost_type_substitution_error_kind_payload0").includes("InvalidBinding binding") &&
        functionBlock(source, "selfhost_type_substitution_error_kind_payload1").includes("DuplicateBinding binding") &&
        functionBlock(source, "selfhost_type_substitution_error_kind_eq").includes("payload2_eq"),
    "error equality must compare payloads through exhaustive code/payload normalization for regression checks",
);

assert(
    !/_:\s*\n\s*false/.test(functionBlock(source, "selfhost_type_substitution_error_kind_eq")) &&
        !/_:\s*\n\s*false/.test(functionBlock(source, "selfhost_type_substitution_error_kind_code")) &&
        !/_:\s*\n\s*false/.test(functionBlock(source, "selfhost_type_substitution_error_kind_payload0")) &&
        !/_:\s*\n\s*false/.test(functionBlock(source, "selfhost_type_substitution_error_kind_payload1")) &&
        !/_:\s*\n\s*false/.test(functionBlock(source, "selfhost_type_substitution_error_kind_payload2")),
    "error equality helpers must not use wildcard false arms that hide newly added variants",
);

assert(
    /accepted[\s\S]*accepted_step_hash_verified[\s\S]*duplicate_binding[\s\S]*invalid_binding[\s\S]*missing_replacement[\s\S]*missing_root/.test(source),
    "stage0 must exercise accepted traversal, step hash verification, duplicate binding, invalid binding, missing replacement, and missing root",
);

assert(
    !/source_text|display_name|diagnostic_text|module_path|public_surface_hash|span_lexeme/.test(code),
    "substitution implementation must not store source/display/diagnostic/module/public-surface authority fields",
);

assert(
    !/maxLine|line count limit|line-count limit|doc comment length cap|documentation length cap|行数制限|コメント量制限|ドキュメントコメント.{0,12}上限/.test(source),
    "type substitution contract must not introduce line-count or documentation-comment length limits",
);

console.log("selfhost type substitution contract ok");
