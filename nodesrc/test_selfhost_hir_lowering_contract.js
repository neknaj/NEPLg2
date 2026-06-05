#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function readRepoFile(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

const facade = readRepoFile("stdlib/neplg2/core/lower/hir.nepl");
const functionValue = readRepoFile("stdlib/neplg2/core/lower/hir/function_value.nepl");
const source = `${facade}\n${functionValue}`;

assert.match(
    facade,
    /^pub #import "\.\/hir\/function_value" as \*$/m,
    "lower/hir facade must re-export the function value lowering split module",
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
assert.doesNotMatch(
    readRepoFile("stdlib/neplg2/core/check/expr.nepl"),
    /function_value_lowering|SelfhostHirExpr|SelfhostHirFunctionValueIdentity/,
    "check/expr facade must not import HIR lowering directly",
);

console.log("selfhost HIR lowering contract passed");
