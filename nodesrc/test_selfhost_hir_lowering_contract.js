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

function assertDirectCallDoc(name, requiredParts) {
    const marker = `//: ${name}:`;
    const markerIndex = directCall.indexOf(marker);
    assert.notEqual(markerIndex, -1, `${name} must have a named doc comment`);
    const fnIndex = directCall.indexOf(`fn ${name} `, markerIndex);
    assert.notEqual(fnIndex, -1, `${name} doc comment must be immediately before its function declaration`);
    const doc = directCall.slice(markerIndex, fnIndex);
    for (const part of requiredParts) {
        assert.ok(doc.includes(part), `${name} doc comment must include ${part}`);
    }
}

function assertDirectCallDeclDoc(name, declarationKind, requiredParts) {
    const marker = `//: ${name}:`;
    const markerIndex = directCall.indexOf(marker);
    assert.notEqual(markerIndex, -1, `${name} must have a named doc comment`);
    const declIndex = directCall.indexOf(`${declarationKind} ${name}`, markerIndex);
    assert.notEqual(declIndex, -1, `${name} doc comment must be immediately before ${declarationKind}`);
    const doc = directCall.slice(markerIndex, declIndex);
    for (const part of requiredParts) {
        assert.ok(doc.includes(part), `${name} doc comment must include ${part}`);
    }
}

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
    /#import "core\/char" as \*[\s\S]*pub fn selfhost_hir_lower_checked_argument[\s\S]*SelfhostCheckedArgumentKind::UnitValue:[\s\S]*selfhost_hir_expr_unit argument\.value_type argument\.span[\s\S]*SelfhostCheckedArgumentKind::BoolLiteral value:[\s\S]*selfhost_hir_expr_bool_literal argument\.value_type argument\.span value[\s\S]*SelfhostCheckedArgumentKind::I32Literal value:[\s\S]*selfhost_hir_expr_i32_literal argument\.value_type argument\.span value[\s\S]*SelfhostCheckedArgumentKind::CharLiteral value:[\s\S]*selfhost_hir_expr_i32_literal argument\.value_type argument\.span char_to_i32 value[\s\S]*SelfhostCheckedArgumentKind::StrLiteral value:[\s\S]*selfhost_hir_expr_str_literal argument\.value_type argument\.span value[\s\S]*SelfhostCheckedArgumentKind::NamedValue identity:[\s\S]*selfhost_hir_value_identity_new identity\.name identity\.def_id argument\.value_type identity\.kind[\s\S]*selfhost_hir_expr_var argument\.value_type argument\.span hir_identity[\s\S]*SelfhostCheckedArgumentKind::FunctionValue candidate:[\s\S]*selfhost_hir_expr_fn_value_from_candidate candidate argument\.span[\s\S]*SelfhostDirectCallLowerErrorKind::UnsupportedArgumentKind/,
    "direct call lowering must lower UnitValue, literal value including i32-backed char payloads, NamedValue, and FunctionValue payloads without re-reading source and fail closed for unsupported argument kinds",
);
assert.match(
    directCall,
    /\[契約\/けいやく\]:[\s\S]*`UnitValue`、bool \/ i32 \/ char \/ simple string literal、`NamedValue`、`FunctionValue` argument は accepted HIR lowering[\s\S]*\[現状実装\/げんじょうじっそう\]:[\s\S]*char literal は Rust 実装の現在の HIR と同じく、型を `char` とした i32-backed literal payload に変換/,
    "direct call module docs must separate stable accepted-lowering contract from current Rust-compatible i32-backed char implementation detail",
);
assertDirectCallDeclDoc("SelfhostDirectCallLowerErrorKind", "pub enum", [
    "[目的/もくてき]",
    "[分類/ぶんるい]",
    "[契約/けいやく]",
    "`UnsupportedArgumentKind` は",
    "`ChildIdsAllocFailed`、`ChildExprAllocFailed`、`ChildIdPushFailed`、`ChildRangeAllocFailed`、`ParentCallAllocFailed`",
    "owner を閉じる必要がある失敗 path",
]);
assertDirectCallDeclDoc("SelfhostDirectCallLowerError", "pub struct", [
    "[目的/もくてき]",
    "[契約/けいやく]",
    "[計算量/けいさんりょう]",
    "表示文字列、翻訳、色付けはこの struct とは別の report 層で行います",
]);
for (const name of [
    "selfhost_direct_call_lower_error_new",
    "selfhost_direct_call_lower_error_kind_eq",
    "selfhost_direct_call_argument_lower_state_new",
    "selfhost_direct_call_lower_free_module_error",
    "selfhost_direct_call_lower_free_state_error",
    "selfhost_hir_lower_checked_argument",
    "selfhost_hir_lower_direct_call_arguments",
    "selfhost_hir_lower_direct_call_plan",
    "selfhost_hir_lower_direct_call_result",
    "selfhost_hir_lower_expression_line_success_direct_call",
]) {
    assertDirectCallDoc(name, ["[目的/もくてき]"]);
}
for (const name of [
    "selfhost_direct_call_lower_error_new",
    "selfhost_direct_call_argument_lower_state_new",
    "selfhost_direct_call_lower_free_module_error",
    "selfhost_direct_call_lower_free_state_error",
    "selfhost_hir_lower_direct_call_arguments",
    "selfhost_hir_lower_direct_call_plan",
]) {
    assertDirectCallDoc(name, ["[契約/けいやく]"]);
}
for (const name of [
    "selfhost_direct_call_lower_error_kind_eq",
    "selfhost_direct_call_lower_free_module_error",
    "selfhost_direct_call_lower_free_state_error",
    "selfhost_hir_lower_checked_argument",
    "selfhost_hir_lower_direct_call_arguments",
    "selfhost_hir_lower_direct_call_plan",
    "selfhost_hir_lower_direct_call_result",
    "selfhost_hir_lower_expression_line_success_direct_call",
]) {
    assertDirectCallDoc(name, ["[戻/もど]り[値/ち]"]);
}
for (const name of [
    "selfhost_direct_call_lower_error_new",
    "selfhost_direct_call_lower_error_kind_eq",
    "selfhost_direct_call_argument_lower_state_new",
    "selfhost_direct_call_lower_free_module_error",
    "selfhost_direct_call_lower_free_state_error",
    "selfhost_hir_lower_checked_argument",
    "selfhost_hir_lower_direct_call_arguments",
    "selfhost_hir_lower_direct_call_plan",
    "selfhost_hir_lower_direct_call_result",
    "selfhost_hir_lower_expression_line_success_direct_call",
]) {
    assertDirectCallDoc(name, ["[計算量/けいさんりょう]"]);
}
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
    /SelfhostExprPrefixList|SelfhostExprPrefixItem|SelfhostToken|selfhost_token_lexeme|selfhost_expr_prefix_list_from_syntax_range|selfhost_callable_candidates_collect_for_head_item/,
    "direct call HIR lowering must not re-read prefix tokens or source lexemes to recover argument or function identity",
);
assert.doesNotMatch(
    readRepoFile("stdlib/neplg2/core/check/expr.nepl"),
    /function_value_lowering|SelfhostHirExpr|SelfhostHirFunctionValueIdentity/,
    "check/expr facade must not import HIR lowering directly",
);
assert.doesNotMatch(
    checkExprSource,
    /#import "neplg2\/core\/hir\/hir"|#import "neplg2\/core\/lower\/hir"|SelfhostHirExpr|SelfhostHirFunctionValueIdentity|SelfhostHirValueIdentity/,
    "check/expr split modules must not import or construct HIR records directly",
);

console.log("selfhost HIR lowering contract passed");
