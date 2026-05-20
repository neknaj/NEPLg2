#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
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

const proofFacade = read("stdlib/neplg2/core/proof.nepl");
const proofFact = read("stdlib/neplg2/core/proof/fact.nepl");
const proofObligation = read("stdlib/neplg2/core/proof/obligation.nepl");
const proofQuery = read("stdlib/neplg2/core/proof/query.nepl");
const proofSolver = read("stdlib/neplg2/core/proof/solver.nepl");
const moduleChecker = read("stdlib/neplg2/core/check/module.nepl");
const checker = read("stdlib/neplg2/core/check/checker.nepl");

assert.match(proofFacade, /pub #import "\.\/proof\/fact" as \*/);
assert.match(proofFacade, /pub #import "\.\/proof\/obligation" as \*/);
assert.match(proofFacade, /pub #import "\.\/proof\/query" as \*/);
assert.match(proofFacade, /pub #import "\.\/proof\/solver" as \*/);

assert.match(proofFact, /pub enum SelfhostProofDomain:/, "proof domain must be a typed enum");
assert.match(proofFact, /pub enum SelfhostProofFact:/, "proof facts must be typed enum payloads");
assert.match(proofObligation, /pub enum SelfhostProofObligation:/, "proof obligations must be typed enum payloads");
assert.match(proofQuery, /pub enum SelfhostProofEvidence:/, "proof success must return typed evidence");
assert.match(proofQuery, /pub enum SelfhostProofRefutation:/, "proof failure must return typed refutation");
assert.match(proofQuery, /pub enum SelfhostProofResult:/, "proof results must be an evidence/refutation enum");
assert.match(proofQuery, /fact <SelfhostProofFact>/, "proof query must carry a typed fact");
assert.match(proofQuery, /obligation <SelfhostProofObligation>/, "proof query must carry a typed obligation");
assert.match(proofFact, /RawBackendItemObserved <SelfhostRawBackendItemFact>/, "raw backend facts must enter proof as typed facts");
assert.match(
    proofObligation,
    /RawBackendTransition <SelfhostRawBackendState>/,
    "raw backend transitions must enter proof as typed obligations",
);
assert.match(
    proofQuery,
    /RawBackendTransition <SelfhostRawBackendState>/,
    "raw backend transition evidence must carry the next typed state",
);
assert.match(
    proofQuery,
    /RawBackendBlockEmpty <SelfhostRawBackendOpenBlock>/,
    "raw backend empty-block failures must be typed refutations",
);
assert.match(
    proofFact,
    /ModuleDirectiveObserved <SelfhostModuleDirectiveFact>/,
    "module directive facts must enter proof as typed facts",
);
assert.match(
    proofObligation,
    /ModuleDirectiveTransition <SelfhostModuleDirectiveState>/,
    "module directive multiplicity must enter proof as a typed obligation",
);
assert.match(
    proofQuery,
    /ModuleDirectiveDuplicate <SelfhostModuleDirectiveDuplicate>/,
    "module directive duplicate failures must be typed refutations",
);
assert.match(
    proofFact,
    /ModuleDeclarationObserved <SelfhostModuleDeclarationFact>/,
    "module declaration facts must enter proof as typed facts",
);
assert.match(
    proofFact,
    /ResourceCellEventObserved <SelfhostResourceCellEventFact>/,
    "resource cell events must enter proof as typed facts",
);
assert.match(
    proofFact,
    /EffectObserved <SelfhostEffectObservationFact>/,
    "effect observations must enter proof as typed facts",
);
assert.match(
    proofFact,
    /TypeKindObserved <SelfhostTypeKindFact>/,
    "type kind observations must enter proof as typed facts",
);
assert.match(
    proofObligation,
    /ModuleDeclarationHeaderAvailable <SelfhostModuleDeclarationKind>/,
    "declaration header availability must enter proof as a typed obligation",
);
assert.match(
    proofObligation,
    /ResourceCellTransition <SelfhostResourceCellState>/,
    "resource cell transitions must enter proof as typed obligations",
);
assert.match(
    proofObligation,
    /EffectAllowedInContext <SelfhostEffectContext>/,
    "effect boundary checks must enter proof as typed obligations",
);
assert.match(
    proofObligation,
    /TypeKindCompatible <SelfhostTypeKind>/,
    "type kind compatibility must enter proof as a typed obligation",
);
assert.match(
    proofQuery,
    /ModuleDeclarationHeaderAvailable <SelfhostModuleDeclarationHeader>/,
    "declaration header proof evidence must carry the typed header",
);
assert.match(
    proofQuery,
    /ResourceCellTransition <SelfhostResourceCellState>/,
    "resource cell transition proof evidence must carry the next typed state",
);
assert.match(
    proofQuery,
    /EffectAllowed <SelfhostEffectContext>/,
    "effect boundary proof evidence must carry the proven context",
);
assert.match(
    proofQuery,
    /TypeKindCompatible <SelfhostTypeKind>/,
    "type kind compatibility proof evidence must carry the proven kind",
);
assert.match(
    proofQuery,
    /ModuleDeclarationHeaderMissing <SelfhostModuleDeclarationHeaderIssue>/,
    "declaration header missing failures must be typed refutations",
);
assert.match(
    proofQuery,
    /pub struct SelfhostProofMismatch:[\s\S]*fact_domain <SelfhostProofDomain>[\s\S]*obligation_domain <SelfhostProofDomain>/,
    "fact/obligation mismatch failures must retain typed proof domains",
);
assert.match(
    proofQuery,
    /FactObligationMismatch <SelfhostProofMismatch>/,
    "fact/obligation mismatch refutation must carry a typed payload",
);
assert.match(
    proofQuery,
    /ResourceCellTransitionInvalid <SelfhostResourceCellTransitionIssue>/,
    "invalid resource cell transitions must return typed refutations",
);
assert.match(
    proofQuery,
    /EffectBoundaryInvalid <SelfhostEffectBoundaryIssue>/,
    "invalid effect boundary checks must return typed refutations",
);
assert.match(
    proofQuery,
    /TypeKindMismatch <SelfhostTypeKindMismatch>/,
    "type kind mismatches must return typed refutations",
);
assert.doesNotMatch(
    proofQuery,
    /^\s+FactObligationMismatch\s*$/m,
    "fact/obligation mismatch must not be a payload-free catch-all",
);
assert.doesNotMatch(
    proofQuery,
    /selfhost_proof_result_is_proven/,
    "proof layer must not provide a public helper that collapses typed proof results to bool",
);

const solverBlock = functionBlock(proofSolver, "selfhost_proof_solve");
const publicSolverFunctions = Array.from(
    proofSolver.matchAll(/^pub fn\s+([A-Za-z0-9_]+)\b/gm),
    (match) => match[1],
);
const allowedPublicSolverFunctions = new Set([
    "selfhost_proof_solve",
    "selfhost_proof_source_span_valid",
    "selfhost_proof_raw_backend_transition",
    "selfhost_proof_module_directive_transition",
    "selfhost_proof_module_declaration_header",
    "selfhost_proof_resource_cell_transition",
    "selfhost_proof_type_kind_compatible",
    "selfhost_proof_effect_allowed",
]);
for (const fnName of publicSolverFunctions) {
    assert.ok(
        allowedPublicSolverFunctions.has(fnName),
        `proof solver must not expose internal proof rule helper ${fnName}`,
    );
}
for (const fnName of allowedPublicSolverFunctions) {
    assert.ok(publicSolverFunctions.includes(fnName), `proof solver public API must expose ${fnName}`);
}
assert.match(solverBlock, /\bmatch\s+(?:query\.)?obligation:/, "solver must match on obligation enum");
assert.match(solverBlock, /\bmatch\s+(?:query\.)?fact:/, "solver must match on fact enum");
assert.doesNotMatch(solverBlock, /^\s*_:/m, "solver must not hide new fact/obligation variants behind wildcard arms");
assert.doesNotMatch(solverBlock, /"[A-Za-z0-9_.:-]+"/, "proof solver must not depend on string codes or module names");
assert.match(
    proofSolver,
    /selfhost_proof_fact_domain\s+fact[\s\S]*selfhost_proof_obligation_domain\s+obligation/,
    "mismatch construction must derive domains from typed fact and obligation values",
);
assert.match(
    proofSolver,
    /(?:^|\n)fn\s+selfhost_proof_solve_raw_backend_transition\b[\s\S]*match\s+state:/,
    "raw backend state transitions must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)fn\s+selfhost_proof_solve_module_directive_transition\b[\s\S]*match\s+state:/,
    "module directive singleton transitions must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)fn\s+selfhost_proof_solve_module_declaration_header\b[\s\S]*match\s+fact\.declaration:/,
    "declaration header availability must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)fn\s+selfhost_proof_solve_resource_cell_transition\b[\s\S]*match\s+state:/,
    "resource cell transitions must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)fn\s+selfhost_proof_solve_effect_allowed\b[\s\S]*match\s+context:/,
    "effect boundary checks must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)fn\s+selfhost_proof_solve_type_kind_compatible\b[\s\S]*selfhost_type_kind_eq/,
    "type kind compatibility must live in the proof solver",
);
assert.match(
    proofSolver,
    /^pub fn\s+selfhost_proof_source_span_valid\b[^\n]*Result<\(\),SelfhostProofRefutation>/m,
    "source span validity must preserve typed refutations instead of returning bool",
);

assert.match(moduleChecker, /#import "neplg2\/core\/proof" as \*/, "module checker must depend on the generic proof facade");
const publicModuleCheckerFunctions = Array.from(
    moduleChecker.matchAll(/^pub fn\s+([A-Za-z0-9_]+)\b/gm),
    (match) => match[1],
);
const allowedPublicModuleCheckerFunctions = new Set([
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
    "selfhost_check_module_ast",
]);
for (const fnName of publicModuleCheckerFunctions) {
    assert.ok(
        allowedPublicModuleCheckerFunctions.has(fnName),
        `module checker must not expose internal proof adapter or state helper ${fnName}`,
    );
}
for (const fnName of allowedPublicModuleCheckerFunctions) {
    assert.ok(publicModuleCheckerFunctions.includes(fnName), `module checker public API must expose ${fnName}`);
}
assert.doesNotMatch(
    moduleChecker,
    /^pub struct SelfhostModuleCheckStep:/m,
    "module checker step state must stay private to avoid exposing checker-internal proof sequencing",
);
assert.match(
    moduleChecker,
    /match\s+selfhost_proof_source_span_valid\s+item\.span:/,
    "module item span validation must match on the proof solver's typed result",
);
assert.doesNotMatch(
    moduleChecker,
    /if:[\s\S]{0,120}selfhost_proof_source_span_valid\s+item\.span/,
    "module item span validation must not collapse proof result to a boolean predicate",
);
assert.doesNotMatch(
    moduleChecker,
    /source_span_is_valid\s+item\.span/,
    "module checker must not bypass proof for module item span validation",
);
assert.doesNotMatch(
    moduleChecker,
    /enum\s+SelfhostModuleRawState:/,
    "module checker must not own a checker-local raw backend proof state enum",
);
assert.match(
    moduleChecker,
    /selfhost_proof_raw_backend_transition\s+state\s+item/,
    "module checker must ask the proof solver for raw backend transitions",
);
assert.match(
    moduleChecker,
    /selfhost_proof_module_directive_transition\s+state\s+item/,
    "module checker must ask the proof solver for module directive transitions",
);
assert.match(
    moduleChecker,
    /selfhost_proof_module_declaration_header\s+kind\s+selfhost_module_declaration_item_fact\s+item/,
    "module checker must ask the proof solver for declaration header availability",
);
assert.doesNotMatch(
    moduleChecker,
    /if:\s*\n\s+gt\s+summary\.(?:entry_count|target_count)\s+1/,
    "module checker must not validate singleton directives by summary count checks",
);
assert.doesNotMatch(
    checker,
    /#import "neplg2\/core\/proof"/,
    "checker facade should stay orchestration-only and avoid direct proof implementation coupling",
);

console.log("selfhost proof entry contract passed");
