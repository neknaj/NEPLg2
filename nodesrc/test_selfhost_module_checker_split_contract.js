#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

function publicFunctions(src) {
    return Array.from(src.matchAll(/^pub fn\s+([A-Za-z0-9_]+)\b/gm), (match) => match[1]);
}

const facade = read("stdlib/neplg2/core/check/module.nepl");
const checkerFacade = read("stdlib/neplg2/core/check/checker.nepl");
const summary = read("stdlib/neplg2/core/check/module/summary.nepl");
const summaryUpdate = read("stdlib/neplg2/core/check/module/summary_update.nepl");
const diagnostic = read("stdlib/neplg2/core/check/module/diagnostic.nepl");
const rawAdapter = read("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl");
const declarationAdapter = read("stdlib/neplg2/core/check/module/declaration_adapter.nepl");
const memoTraitSourceFingerprint = read("stdlib/neplg2/core/check/module/memo_trait_source_fingerprint.nepl");
const memoTraitSourceScan = read("stdlib/neplg2/core/check/module/memo_trait_source_scan.nepl");
const orchestrate = read("stdlib/neplg2/core/check/module/orchestrate.nepl");
const implementation = [
    summary,
    summaryUpdate,
    diagnostic,
    rawAdapter,
    declarationAdapter,
    memoTraitSourceFingerprint,
    memoTraitSourceScan,
    orchestrate,
].join("\n");

assert.match(facade, /pub #import "\.\/module\/summary" as \*/);
assert.match(facade, /pub #import "\.\/module\/memo_trait_source_fingerprint" as \*/);
assert.match(facade, /pub #import "\.\/module\/memo_trait_source_scan" as \*/);
assert.match(facade, /pub #import "\.\/module\/orchestrate" as \*/);
assert.deepEqual(
    Array.from(facade.matchAll(/^pub #import "([^"]+)" as ([^\n]+)$/gm), (match) => `${match[1]} as ${match[2]}`)
        .sort(),
    [
        "./module/memo_trait_source_fingerprint as *",
        "./module/memo_trait_source_scan as *",
        "./module/orchestrate as *",
        "./module/summary as *",
    ],
    "module facade must re-export only the public summary, memo trait source evidence, memo trait source scanner, and orchestration surfaces",
);
assert.doesNotMatch(facade, /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/m, "module facade must not own implementation");
assert.doesNotMatch(facade, /#import "neplg2\/core\/proof"/, "module facade must not import proof internals");
assert.doesNotMatch(facade, /syntax\/parser\/module_parser/, "module doctest must not pull the parser into focused checker coverage");
assert.doesNotMatch(
    facade,
    /selfhost_parse_module_source(?:_with_file_id)?/,
    "module doctest must use typed AST evidence instead of parser source text",
);

assert.doesNotMatch(
    checkerFacade,
    /syntax\/parser\/module_parser/,
    "checker smoke API must not pull the parser into focused checker coverage",
);
assert.doesNotMatch(
    checkerFacade,
    /selfhost_parse_module_source(?:_with_file_id)?/,
    "checker smoke API must use typed AST evidence instead of parser source text",
);
assert.match(
    checkerFacade,
    /selfhost_module_ast_new/,
    "checker smoke API should construct a minimal typed AST directly",
);
assert.match(
    checkerFacade,
    /selfhost_module_item_new_with_declaration/,
    "checker smoke API must include declaration header evidence instead of relying on parser text",
);

assert.match(summary, /pub struct SelfhostModuleCheckSummary:/);
assert.doesNotMatch(summary, /#import "neplg2\/core\/proof"/, "summary storage must not know proof details");
assert.doesNotMatch(summary, /#import "neplg2\/core\/infra\/diag"/, "summary storage must not know diagnostics");
assert.match(summaryUpdate, /match\s+kind:/, "summary updates must be driven by an exhaustive item kind match");
assert.doesNotMatch(
    summaryUpdate,
    /SelfhostCheckerDiagnosticCode|SelfhostProofRefutation|selfhost_proof_/,
    "summary updates must remain pure counting logic",
);

assert.match(diagnostic, /pub fn\s+selfhost_module_check_refutation_diag\b/);
assert.match(diagnostic, /SelfhostProofRefutation::UnexpectedEvidence\s+_issue:/);
assert.match(
    diagnostic,
    /SelfhostCheckerDiagnosticCode::ModuleUnexpectedProof/,
    "unexpected proof refutations must use a dedicated checker diagnostic code instead of ModuleItemIndex message text",
);
assert.doesNotMatch(diagnostic, /selfhost_proof_[a-z0-9_]+\s+/, "diagnostic mapping must not invoke proof solvers");

assert.match(rawAdapter, /pub fn\s+selfhost_module_check_raw_backend_fact\b/);
assert.match(rawAdapter, /match\s+selfhost_proof_raw_backend_transition\s+state\s+item:/);
assert.doesNotMatch(rawAdapter, /SelfhostDiagnostic/, "raw adapter must return proof refutations, not diagnostics");

assert.match(declarationAdapter, /match\s+selfhost_proof_source_span_valid\s+item\.span:/);
assert.match(declarationAdapter, /match\s+selfhost_proof_module_directive_transition\s+state\s+item:/);
assert.match(
    declarationAdapter,
    /match\s+selfhost_proof_module_declaration_header\s+kind\s+selfhost_module_declaration_item_fact\s+item:/,
);
assert.doesNotMatch(
    declarationAdapter,
    /SelfhostDiagnostic/,
    "declaration adapter must return proof refutations, not diagnostics",
);

assert.match(orchestrate, /^struct SelfhostModuleCheckStep:/m);
assert.doesNotMatch(orchestrate, /^pub struct SelfhostModuleCheckStep:/m);
assert.match(orchestrate, /selfhost_module_check_refutation_diag\s+refutation/);
assert.doesNotMatch(orchestrate, /selfhost_proof_[a-z0-9_]+\s+/, "orchestration must use adapters instead of proof calls");
assert.doesNotMatch(
    orchestrate,
    /"raw backend|"module item has|"multiple #|"declaration item is/,
    "orchestration must not own diagnostic message text",
);

const publicSurface = new Set(publicFunctions(`${summary}\n${orchestrate}`));
for (const publicName of publicFunctions(memoTraitSourceScan)) {
    publicSurface.add(publicName);
}
const expectedPublicSurface = [
    "selfhost_module_check_summary_item_count",
    "selfhost_module_check_summary_doc_comment_count",
    "selfhost_module_check_summary_directive_count",
    "selfhost_module_check_summary_entry_count",
    "selfhost_module_check_summary_target_count",
    "selfhost_module_check_summary_import_count",
    "selfhost_module_check_summary_declaration_count",
    "selfhost_module_check_summary_function_count",
    "selfhost_module_check_summary_type_declaration_count",
    "selfhost_module_check_summary_impl_count",
    "selfhost_module_check_summary_raw_block_count",
    "selfhost_module_check_summary_raw_text_count",
    "selfhost_memo_trait_definition_scan_error_kind_eq",
    "selfhost_memo_trait_definition_scan_registry_error_kind_eq",
    "selfhost_memo_trait_definition_source_table_scan_module_result",
    "selfhost_memo_trait_trusted_source_registry_scan_module_result",
    "selfhost_memo_trait_definition_scan_stage0",
    "selfhost_check_module_ast",
];
assert.deepEqual(Array.from(publicSurface).sort(), expectedPublicSurface.sort());

assert.doesNotMatch(
    implementation,
    /if:\s*\n\s+gt\s+summary\.(?:entry_count|target_count)\s+1/,
    "module singleton directive validation must stay in the proof solver",
);

console.log("selfhost module checker split contract passed");
