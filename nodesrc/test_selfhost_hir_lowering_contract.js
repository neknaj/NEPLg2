#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { readCheckExprSource } = require("./selfhost_check_expr_sources");

const repoRoot = path.resolve(__dirname, "..");

function readRepoFile(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

const facade = readRepoFile("stdlib/neplg2/core/lower/hir.nepl");
const functionValue = readRepoFile("stdlib/neplg2/core/lower/hir/function_value.nepl");
const directCall = readRepoFile("stdlib/neplg2/core/lower/hir/direct_call.nepl");
const source = `${facade}\n${functionValue}\n${directCall}`;
const checkExprSource = readCheckExprSource(repoRoot);

assert.match(
    facade,
    /^pub #import "\.\/hir\/function_value" as \*$/m,
    "lower/hir facade must re-export the function value lowering split module",
);
assert.match(
    facade,
    /^pub #import "\.\/hir\/direct_call" as \*$/m,
    "lower/hir facade must re-export the direct call lowering split module",
);
assert.match(
    source,
    /# lower\/hir\/function_value[\s\S]*pub enum SelfhostFunctionValueLowerErrorKind:[\s\S]*GenericUnsupported[\s\S]*IdentityBuildFailed/,
    "function value HIR lowering must live in its own typed-error lower/hir split module",
);
assert.match(
    source,
    /#import "neplg2\/core\/check\/expr\/call_candidate" as \*[\s\S]*#import "neplg2\/core\/hir\/hir" as \*/,
    "function value lowering must be the explicit layer that knows both checker candidates and HIR records",
);
assert.match(
    source,
    /pub fn selfhost_function_value_candidate_is_accepted_monomorphic %fn SelfhostCallableCandidate bool[\s\S]*SelfhostGenericInferenceState::NoneRequired:[\s\S]*true[\s\S]*_:[\s\S]*false/,
    "function value lowering must accept only monomorphic candidates until stable type argument identity exists",
);
assert.match(
    source,
    /pub fn selfhost_function_value_identity_from_candidate %fn SelfhostCallableCandidate Result SelfhostHirFunctionValueIdentity SelfhostFunctionValueLowerError[\s\S]*not selfhost_function_value_candidate_is_accepted_monomorphic candidate[\s\S]*SelfhostFunctionValueLowerErrorKind::GenericUnsupported[\s\S]*selfhost_hir_function_value_identity_new_result candidate\.name some candidate\.def_id candidate\.callable_type candidate\.effect 0/,
    "function value lowering must preserve candidate DefId and reject unsupported generic candidates",
);
assert.match(
    source,
    /pub fn selfhost_hir_expr_fn_value_from_candidate %fn SelfhostCallableCandidate fn SelfhostSourceSpan Result SelfhostHirExpr SelfhostFunctionValueLowerError[\s\S]*selfhost_hir_expr_fn_value candidate\.callable_type span identity/,
    "function value lowering must build a FnValue HIR expression whose type is the candidate function type",
);
assert.match(
    source,
    /# lower\/hir\/direct_call[\s\S]*pub enum SelfhostDirectCallLowerErrorKind:[\s\S]*UnsupportedArgumentKind[\s\S]*FunctionValueFailed %SelfhostFunctionValueLowerErrorKind[\s\S]*ChildExprAllocFailed[\s\S]*ParentCallAllocFailed/,
    "direct call lowering must live in its own typed-error lower/hir split module",
);
assert.match(
    directCall,
    /#import "neplg2\/core\/check\/expr\/argument_payload" as \*[\s\S]*#import "neplg2\/core\/check\/expr\/body_line" as \*[\s\S]*#import "neplg2\/core\/hir\/hir" as \*[\s\S]*#import "\.\/function_value" as \*/,
    "direct call lowering must be the explicit layer that knows checked arguments, body-line success, HIR, and function-value lowering",
);
assert.match(
    directCall,
    /pub fn selfhost_hir_lower_checked_argument[\s\S]*SelfhostCheckedArgumentKind::FunctionValue candidate:[\s\S]*selfhost_hir_expr_fn_value_from_candidate candidate argument\.span[\s\S]*SelfhostDirectCallLowerErrorKind::UnsupportedArgumentKind/,
    "direct call lowering must lower FunctionValue payloads through the function-value boundary and fail closed for unsupported argument kinds",
);
assert.match(
    directCall,
    /pub fn selfhost_hir_lower_direct_call_result[\s\S]*SelfhostCallReduceResult::DirectCall call:[\s\S]*v::get candidates call\.candidate_index[\s\S]*selfhost_hir_lower_direct_call_plan module call checked_arguments candidate/,
    "direct call lowering must use the reducer-selected candidate index instead of re-reading source tokens",
);
assert.match(
    directCall,
    /fn selfhost_hir_lower_direct_call_plan[\s\S]*not eq v::len checked_arguments call\.argument_count[\s\S]*SelfhostDirectCallLowerErrorKind::CheckedArgumentCountMismatch/,
    "direct call lowering must reject mismatched checked argument payload counts",
);
assert.match(
    directCall,
    /pub fn selfhost_hir_lower_expression_line_success_direct_call[\s\S]*selfhost_expression_line_check_success_result success[\s\S]*selfhost_expression_line_check_success_checked_arguments success/,
    "body-line success lowering must consume the checked argument list exposed by expression checking",
);
assert.doesNotMatch(
    directCall,
    /SelfhostExprPrefixList|SelfhostExprPrefixItem|selfhost_expr_prefix_list_from_syntax_range|selfhost_callable_candidates_collect_for_head_item/,
    "direct call HIR lowering must not re-read prefix tokens to recover argument or function identity",
);
assert.doesNotMatch(
    readRepoFile("stdlib/neplg2/core/check/expr.nepl"),
    /function_value_lowering|SelfhostHirExpr|SelfhostHirFunctionValueIdentity/,
    "check/expr facade must not import HIR lowering directly",
);
assert.doesNotMatch(
    checkExprSource,
    /#import "neplg2\/core\/hir\/hir"|#import "neplg2\/core\/lower\/hir"|SelfhostHirExpr|SelfhostHirFunctionValueIdentity/,
    "check/expr split modules must not import or construct HIR records directly",
);

console.log("selfhost HIR lowering contract passed");
