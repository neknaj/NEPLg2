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
const proofFactFacade = read("stdlib/neplg2/core/proof/fact.nepl");
const proofDomain = read("stdlib/neplg2/core/proof/domain.nepl");
const proofFactModel = read("stdlib/neplg2/core/proof/fact/model.nepl");
const proofObligation = read("stdlib/neplg2/core/proof/obligation.nepl");
const proofQueryFacade = read("stdlib/neplg2/core/proof/query.nepl");
const proofEvidence = read("stdlib/neplg2/core/proof/evidence.nepl");
const proofRefutation = read("stdlib/neplg2/core/proof/refutation.nepl");
const proofQueryModel = read("stdlib/neplg2/core/proof/query/model.nepl");
const proofSolverFacade = read("stdlib/neplg2/core/proof/solver.nepl");
const proofSolverDispatch = read("stdlib/neplg2/core/proof/solver/dispatch.nepl");
const proofSolverSource = read("stdlib/neplg2/core/proof/solver/source.nepl");
const proofSolverModule = read("stdlib/neplg2/core/proof/solver/module.nepl");
const proofSolverResource = read("stdlib/neplg2/core/proof/solver/resource.nepl");
const proofSolverType = read("stdlib/neplg2/core/proof/solver/type.nepl");
const proofSolverEffect = read("stdlib/neplg2/core/proof/solver/effect.nepl");
const proofApiFacade = read("stdlib/neplg2/core/proof/api.nepl");
const proofApiSource = read("stdlib/neplg2/core/proof/api/source.nepl");
const proofApiModule = read("stdlib/neplg2/core/proof/api/module.nepl");
const proofApiResource = read("stdlib/neplg2/core/proof/api/resource.nepl");
const proofApiType = read("stdlib/neplg2/core/proof/api/type.nepl");
const proofApiEffect = read("stdlib/neplg2/core/proof/api/effect.nepl");
const proofFact = `${proofFactFacade}\n${proofDomain}\n${proofFactModel}`;
const proofQuery = `${proofQueryFacade}\n${proofEvidence}\n${proofRefutation}\n${proofQueryModel}`;
const proofSolverRules = [
    proofSolverSource,
    proofSolverModule,
    proofSolverResource,
    proofSolverType,
    proofSolverEffect,
].join("\n");
const proofSolver = `${proofSolverFacade}\n${proofSolverDispatch}\n${proofSolverRules}`;
const proofApiImpl = [proofApiSource, proofApiModule, proofApiResource, proofApiType, proofApiEffect].join("\n");
const proofApi = `${proofApiFacade}\n${proofApiImpl}`;
const moduleCheckerFacade = read("stdlib/neplg2/core/check/module.nepl");
const moduleCheckerSummary = read("stdlib/neplg2/core/check/module/summary.nepl");
const moduleCheckerSummaryUpdate = read("stdlib/neplg2/core/check/module/summary_update.nepl");
const moduleCheckerDiagnostic = read("stdlib/neplg2/core/check/module/diagnostic.nepl");
const moduleCheckerRawAdapter = read("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl");
const moduleCheckerDeclarationAdapter = read("stdlib/neplg2/core/check/module/declaration_adapter.nepl");
const moduleCheckerMemoTraitPublicSurfaceHash = read("stdlib/neplg2/core/check/module/memo_trait_public_surface_hash.nepl");
const moduleCheckerMemoTraitPublicSurfaceNormalizer = read("stdlib/neplg2/core/check/module/memo_trait_public_surface_normalizer.nepl");
const moduleCheckerMemoTraitPublicSurfaceSeed = read("stdlib/neplg2/core/check/module/memo_trait_public_surface_seed.nepl");
const moduleCheckerMemoTraitSourceEvidenceProducer = read("stdlib/neplg2/core/check/module/memo_trait_source_evidence_producer.nepl");
const moduleCheckerMemoTraitSourceScan = read("stdlib/neplg2/core/check/module/memo_trait_source_scan.nepl");
const moduleCheckerOrchestrate = read("stdlib/neplg2/core/check/module/orchestrate.nepl");
const moduleCheckerPublicSurface = `${moduleCheckerSummary}\n${moduleCheckerMemoTraitPublicSurfaceHash}\n${moduleCheckerMemoTraitPublicSurfaceNormalizer}\n${moduleCheckerMemoTraitPublicSurfaceSeed}\n${moduleCheckerMemoTraitSourceEvidenceProducer}\n${moduleCheckerMemoTraitSourceScan}\n${moduleCheckerOrchestrate}`;
const moduleCheckerImplementation = [
    moduleCheckerSummary,
    moduleCheckerSummaryUpdate,
    moduleCheckerDiagnostic,
    moduleCheckerRawAdapter,
    moduleCheckerDeclarationAdapter,
    moduleCheckerMemoTraitPublicSurfaceHash,
    moduleCheckerMemoTraitPublicSurfaceNormalizer,
    moduleCheckerMemoTraitPublicSurfaceSeed,
    moduleCheckerMemoTraitSourceEvidenceProducer,
    moduleCheckerMemoTraitSourceScan,
    moduleCheckerOrchestrate,
].join("\n");
const checker = read("stdlib/neplg2/core/check/checker.nepl");
const traitRef = read("stdlib/neplg2/core/ty/trait_ref.nepl");
const borrowState = read("stdlib/neplg2/core/resource/borrow_state.nepl");
const resourceCell = read("stdlib/neplg2/core/resource/init/cell.nepl");
const lifetime = read("stdlib/neplg2/core/resource/lifetime.nepl");
const owner = read("stdlib/neplg2/core/resource/owner.nepl");

assert.match(proofFacade, /pub #import "\.\/proof\/fact" as \*/);
assert.match(proofFacade, /pub #import "\.\/proof\/obligation" as \*/);
assert.match(proofFacade, /pub #import "\.\/proof\/query" as \*/);
assert.match(proofFacade, /pub #import "\.\/proof\/solver" as \*/);
assert.match(proofFacade, /pub #import "\.\/proof\/api" as \*/);
assert.match(proofFactFacade, /pub #import "\.\/domain" as \*/);
assert.match(proofFactFacade, /pub #import "\.\/fact\/model" as \*/);
assert.match(proofQueryFacade, /pub #import "\.\/evidence" as \*/);
assert.match(proofQueryFacade, /pub #import "\.\/refutation" as \*/);
assert.match(proofQueryFacade, /pub #import "\.\/query\/model" as \*/);
assert.match(proofSolverFacade, /pub #import "\.\/solver\/dispatch" as \*/);
assert.match(proofApiFacade, /pub #import "\.\/api\/source" as \*/);
assert.match(proofApiFacade, /pub #import "\.\/api\/module" as \*/);
assert.match(proofApiFacade, /pub #import "\.\/api\/resource" as \*/);
assert.match(proofApiFacade, /pub #import "\.\/api\/type" as \*/);
assert.match(proofApiFacade, /pub #import "\.\/api\/effect" as \*/);
for (const [name, src] of [
    ["proof/fact.nepl", proofFactFacade],
    ["proof/query.nepl", proofQueryFacade],
    ["proof/solver.nepl", proofSolverFacade],
    ["proof/api.nepl", proofApiFacade],
]) {
    assert.doesNotMatch(src, /^(?:pub\s+)?(?:fn|struct|enum)\s+/m, `${name} must stay an implementation-free facade`);
}
assert.match(proofFact, /pub enum SelfhostProofDomain:/, "proof domain must be a typed enum");
assert.match(proofFact, /pub enum SelfhostProofFact:/, "proof facts must be typed enum payloads");
assert.match(proofObligation, /pub enum SelfhostProofObligation:/, "proof obligations must be typed enum payloads");
assert.match(proofQuery, /pub enum SelfhostProofEvidence:/, "proof success must return typed evidence");
assert.match(proofQuery, /pub enum SelfhostProofEvidenceKind:/, "proof evidence kinds must be typed for API projection checks");
assert.match(proofQuery, /pub enum SelfhostProofRefutation:/, "proof failure must return typed refutation");
assert.match(proofQuery, /pub enum SelfhostProofResult:/, "proof results must be an evidence/refutation enum");
assert.match(proofQuery, /fact %SelfhostProofFact/, "proof query must carry a typed fact");
assert.match(proofQuery, /obligation %SelfhostProofObligation/, "proof query must carry a typed obligation");
assert.match(proofFact, /RawBackendItemObserved %SelfhostRawBackendItemFact/, "raw backend facts must enter proof as typed facts");
assert.match(
    proofObligation,
    /RawBackendTransition %SelfhostRawBackendState/,
    "raw backend transitions must enter proof as typed obligations",
);
assert.match(
    proofQuery,
    /RawBackendTransition %SelfhostRawBackendState/,
    "raw backend transition evidence must carry the next typed state",
);
assert.match(
    proofQuery,
    /RawBackendBlockEmpty %SelfhostRawBackendOpenBlock/,
    "raw backend empty-block failures must be typed refutations",
);
assert.match(
    proofFact,
    /ModuleDirectiveObserved %SelfhostModuleDirectiveFact/,
    "module directive facts must enter proof as typed facts",
);
assert.match(
    proofObligation,
    /ModuleDirectiveTransition %SelfhostModuleDirectiveState/,
    "module directive multiplicity must enter proof as a typed obligation",
);
assert.match(
    proofQuery,
    /ModuleDirectiveDuplicate %SelfhostModuleDirectiveDuplicate/,
    "module directive duplicate failures must be typed refutations",
);
assert.match(
    proofFact,
    /ModuleDeclarationObserved %SelfhostModuleDeclarationFact/,
    "module declaration facts must enter proof as typed facts",
);
assert.match(
    proofFact,
    /ResourceCellEventObserved %SelfhostResourceCellEventFact/,
    "resource cell events must enter proof as typed facts",
);
assert.match(
    proofFact,
    /OwnerEventObserved %SelfhostOwnerEventFact/,
    "owner obligation events must enter proof as typed facts",
);
assert.match(
    proofFact,
    /BorrowAccessObserved %SelfhostBorrowAccessFact/,
    "borrow access requests must enter proof as typed facts",
);
assert.match(
    proofFact,
    /LifetimeOutlivesObserved %SelfhostLifetimeOutlivesFact/,
    "lifetime outlives observations must enter proof as typed facts",
);
assert.match(
    proofFact,
    /EffectObserved %SelfhostEffectObservationFact/,
    "effect observations must enter proof as typed facts",
);
assert.match(
    proofFact,
    /TypeKindObserved %SelfhostTypeKindFact/,
    "type kind observations must enter proof as typed facts",
);
assert.match(
    proofFact,
    /TraitImplPairObserved %SelfhostTraitImplPairFact/,
    "trait impl coherence facts must enter proof as typed facts",
);
assert.match(
    proofObligation,
    /ModuleDeclarationHeaderAvailable %SelfhostModuleDeclarationKind/,
    "declaration header availability must enter proof as a typed obligation",
);
assert.match(
    proofObligation,
    /ResourceCellTransition %SelfhostResourceCellState/,
    "resource cell transitions must enter proof as typed obligations",
);
assert.match(
    proofObligation,
    /OwnerTransition %SelfhostOwnerState/,
    "owner obligation transitions must enter proof as typed obligations",
);
assert.match(
    proofObligation,
    /ResourceBorrowAccess %SelfhostBorrowState/,
    "borrow access checks must enter proof as typed obligations",
);
assert.match(
    proofObligation,
    /LifetimeOutlives %SelfhostLifetimeId/,
    "lifetime outlives checks must enter proof as typed obligations",
);
assert.match(
    proofObligation,
    /EffectAllowedInContext %SelfhostEffectContext/,
    "effect boundary checks must enter proof as typed obligations",
);
assert.match(
    proofObligation,
    /TypeKindCompatible %SelfhostTypeKind/,
    "type kind compatibility must enter proof as a typed obligation",
);
assert.match(
    proofObligation,
    /TraitImplNonOverlapping/,
    "trait impl coherence must enter proof as a typed obligation",
);
assert.match(
    proofQuery,
    /ModuleDeclarationHeaderAvailable %SelfhostModuleDeclarationHeader/,
    "declaration header proof evidence must carry the typed header",
);
assert.match(
    proofQuery,
    /ResourceCellTransition %SelfhostResourceCellState/,
    "resource cell transition proof evidence must carry the next typed state",
);
assert.match(
    proofQuery,
    /OwnerTransition %SelfhostOwnerState/,
    "owner transition proof evidence must carry the next typed owner state",
);
assert.match(
    proofQuery,
    /ResourceBorrowAccess %SelfhostBorrowState/,
    "borrow access proof evidence must carry the next typed state",
);
assert.match(
    proofQuery,
    /LifetimeOutlives %SelfhostLifetimeRelation/,
    "lifetime outlives proof evidence must carry the source-derived relation",
);
assert.match(
    proofQuery,
    /EffectAllowed %SelfhostEffectContext/,
    "effect boundary proof evidence must carry the proven context",
);
assert.match(
    proofQuery,
    /TypeKindCompatible %SelfhostTypeKind/,
    "type kind compatibility proof evidence must carry the proven kind",
);
assert.match(
    proofQuery,
    /TraitImplNonOverlapping %SelfhostTraitImplRelation/,
    "trait impl coherence proof evidence must carry the typed relation",
);
assert.match(
    proofQuery,
    /ModuleDeclarationHeaderMissing %SelfhostModuleDeclarationHeaderIssue/,
    "declaration header missing failures must be typed refutations",
);
assert.match(
    proofQuery,
    /pub struct SelfhostProofMismatch:[\s\S]*fact_domain %SelfhostProofDomain[\s\S]*obligation_domain %SelfhostProofDomain/,
    "fact/obligation mismatch failures must retain typed proof domains",
);
assert.match(
    proofQuery,
    /FactObligationMismatch %SelfhostProofMismatch/,
    "fact/obligation mismatch refutation must carry a typed payload",
);
assert.match(
    proofQuery,
    /UnexpectedEvidence %SelfhostProofUnexpectedEvidence/,
    "unexpected solver evidence must be separate from fact/obligation mismatch",
);
assert.match(
    proofQuery,
    /ResourceCellTransitionInvalid %SelfhostResourceCellTransitionIssue/,
    "invalid resource cell transitions must return typed refutations",
);
assert.match(
    proofQuery,
    /OwnerTransitionInvalid %SelfhostOwnerTransitionIssue/,
    "invalid owner transitions must return typed refutations",
);
assert.match(
    proofQuery,
    /BorrowAccessInvalid %SelfhostBorrowAccessIssue/,
    "invalid borrow access must return typed refutations",
);
assert.match(
    proofQuery,
    /LifetimeOutlivesInvalid %SelfhostLifetimeOutlivesIssue/,
    "invalid lifetime outlives checks must return typed refutations",
);
assert.match(
    proofQuery,
    /EffectBoundaryInvalid %SelfhostEffectBoundaryIssue/,
    "invalid effect boundary checks must return typed refutations",
);
assert.match(
    proofQuery,
    /TypeKindMismatch %SelfhostTypeKindMismatch/,
    "type kind mismatches must return typed refutations",
);
assert.match(
    proofQuery,
    /TraitImplCoherenceInvalid %SelfhostTraitImplCoherenceIssue/,
    "invalid trait impl coherence must return typed refutations",
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
const solverDispatchBlock = functionBlock(proofSolver, "selfhost_proof_solve_matching_domain");
const domainEqBlock = functionBlock(proofFact, "selfhost_proof_domain_eq");
const evidenceKindBlock = functionBlock(proofQuery, "selfhost_proof_evidence_kind");
const obligationEvidenceKindBlock = functionBlock(proofQuery, "selfhost_proof_obligation_evidence_kind");
const traitRelationBlock = functionBlock(traitRef, "selfhost_trait_impl_relation");
const publicSolverFunctions = Array.from(
    proofSolverDispatch.matchAll(/^pub fn\s+([A-Za-z0-9_]+)\b/gm),
    (match) => match[1],
);
const publicRuleFunctions = Array.from(
    proofSolverRules.matchAll(/^pub fn\s+([A-Za-z0-9_]+)\b/gm),
    (match) => match[1],
);
const publicApiFunctions = Array.from(
    proofApiImpl.matchAll(/^pub fn\s+([A-Za-z0-9_]+)\b/gm),
    (match) => match[1],
);
const allowedPublicSolverFunctions = new Set([
    "selfhost_proof_solve",
]);
const allowedPublicRuleFunctions = new Set([
    "selfhost_proof_solve_source_span_valid",
    "selfhost_proof_solve_raw_backend_transition",
    "selfhost_proof_solve_module_directive_transition",
    "selfhost_proof_solve_module_declaration_header",
    "selfhost_proof_solve_resource_cell_transition",
    "selfhost_proof_solve_owner_transition",
    "selfhost_proof_solve_borrow_access",
    "selfhost_proof_solve_lifetime_outlives",
    "selfhost_proof_solve_type_kind_compatible",
    "selfhost_proof_solve_trait_impl_non_overlapping",
    "selfhost_proof_solve_effect_allowed",
]);
const allowedPublicApiFunctions = new Set([
    "selfhost_proof_source_span_valid",
    "selfhost_proof_raw_backend_transition",
    "selfhost_proof_module_directive_transition",
    "selfhost_proof_module_declaration_header",
    "selfhost_proof_resource_cell_transition",
    "selfhost_proof_owner_transition",
    "selfhost_proof_borrow_access",
    "selfhost_proof_lifetime_outlives",
    "selfhost_proof_type_kind_compatible",
    "selfhost_proof_trait_impl_non_overlapping",
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
for (const fnName of publicRuleFunctions) {
    assert.ok(
        allowedPublicRuleFunctions.has(fnName),
        `proof solver rule modules must expose only dispatch entry rules, got ${fnName}`,
    );
}
for (const fnName of allowedPublicRuleFunctions) {
    assert.ok(publicRuleFunctions.includes(fnName), `proof solver rule module must expose ${fnName}`);
}
for (const fnName of publicApiFunctions) {
    assert.ok(allowedPublicApiFunctions.has(fnName), `proof api must expose only typed proof wrappers, got ${fnName}`);
}
for (const fnName of allowedPublicApiFunctions) {
    assert.ok(publicApiFunctions.includes(fnName), `proof api public API must expose ${fnName}`);
}
assert.match(proofApi, /#import "neplg2\/core\/proof\/solver" as \*/, "proof api must call the generic solver");
assert.doesNotMatch(
    proofApi,
    /(?:^|\n)fn\s+selfhost_proof_solve_[A-Za-z0-9_]+\b/,
    "proof api must not implement domain proof rules",
);
assert.match(
    proofApi,
    /selfhost_proof_solve\s+query/,
    "proof api wrappers must route through the generic proof solver",
);
assert.match(
    proofApi,
    /selfhost_proof_query_unexpected_evidence_refutation\s+query\s+evidence/,
    "proof api wrappers must report unexpected proven evidence as typed unexpected-evidence refutation",
);
assert.doesNotMatch(
    proofApi,
    /selfhost_proof_query_mismatch_refutation\s+query/,
    "proof api wrappers must not classify unexpected evidence as fact/obligation mismatch",
);
for (const variant of ["Source", "Module", "Type", "Trait", "Lifetime", "Owner", "Effect", "Resource"]) {
    assert.match(domainEqBlock, new RegExp(`SelfhostProofDomain::${variant}\\b`), `domain equality must cover ${variant}`);
}
assert.doesNotMatch(domainEqBlock, /^\s*_:/m, "domain equality must not hide new domains behind wildcard arms");
for (const variant of [
    "SourceSpanValid",
    "RawBackendTransition",
    "ModuleDirectiveTransition",
    "ModuleDeclarationHeaderAvailable",
    "TypeKindCompatible",
    "TraitImplNonOverlapping",
    "LifetimeOutlives",
    "ResourceCellTransition",
    "OwnerTransition",
    "ResourceBorrowAccess",
    "EffectAllowed",
]) {
    assert.match(
        evidenceKindBlock,
        new RegExp(`SelfhostProofEvidence::${variant}\\b`),
        `evidence kind extraction must cover evidence ${variant}`,
    );
    assert.match(
        evidenceKindBlock,
        new RegExp(`SelfhostProofEvidenceKind::${variant}\\b`),
        `evidence kind extraction must return kind ${variant}`,
    );
}
assert.doesNotMatch(evidenceKindBlock, /^\s*_:/m, "evidence kind extraction must not hide new evidence behind wildcard arms");
assert.match(
    obligationEvidenceKindBlock,
    /SelfhostProofObligation::SourceSpanValid[\s\S]*SelfhostProofEvidenceKind::SourceSpanValid/,
    "source span obligation must map to source span evidence kind",
);
assert.match(
    obligationEvidenceKindBlock,
    /SelfhostProofObligation::EffectAllowedInContext[\s\S]*SelfhostProofEvidenceKind::EffectAllowed/,
    "effect obligation must map to effect evidence kind",
);
assert.doesNotMatch(
    obligationEvidenceKindBlock,
    /^\s*_:/m,
    "obligation evidence kind mapping must not hide new obligations behind wildcard arms",
);
assert.match(
    solverBlock,
    /selfhost_proof_fact_domain\s+query\.fact/,
    "public solver must derive the fact domain from the typed fact",
);
assert.match(
    solverBlock,
    /selfhost_proof_obligation_domain\s+query\.obligation/,
    "public solver must derive the obligation domain from the typed obligation",
);
assert.match(
    solverBlock,
    /selfhost_proof_domain_eq\s+fact_domain\s+obligation_domain/,
    "public solver must precheck proof domains before dispatching to proof rules",
);
assert.match(
    solverBlock,
    /selfhost_proof_solve_matching_domain\s+query/,
    "public solver must route matching-domain queries through the internal dispatch",
);
assert.match(solverDispatchBlock, /\bmatch\s+(?:query\.)?obligation:/, "solver dispatch must match on obligation enum");
assert.match(solverDispatchBlock, /\bmatch\s+(?:query\.)?fact:/, "solver dispatch must match on fact enum");
assert.doesNotMatch(solverDispatchBlock, /^\s*_:/m, "solver dispatch must not hide new fact/obligation variants behind wildcard arms");
assert.doesNotMatch(solverBlock, /"[A-Za-z0-9_.:-]+"/, "proof solver must not depend on string codes or module names");
assert.doesNotMatch(solverDispatchBlock, /"[A-Za-z0-9_.:-]+"/, "proof dispatch must not depend on string codes or module names");
assert.match(
    proofQuery,
    /selfhost_proof_fact_domain\s+fact[\s\S]*selfhost_proof_obligation_domain\s+obligation/,
    "mismatch construction must derive domains from typed fact and obligation values",
);
assert.match(
    proofSolver,
    /(?:^|\n)(?:pub\s+)?fn\s+selfhost_proof_solve_raw_backend_transition\b[\s\S]*match\s+state:/,
    "raw backend state transitions must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)(?:pub\s+)?fn\s+selfhost_proof_solve_module_directive_transition\b[\s\S]*match\s+state:/,
    "module directive singleton transitions must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)(?:pub\s+)?fn\s+selfhost_proof_solve_module_declaration_header\b[\s\S]*match\s+fact\.declaration:/,
    "declaration header availability must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)(?:pub\s+)?fn\s+selfhost_proof_solve_resource_cell_transition\b[\s\S]*match\s+state:/,
    "resource cell transitions must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)(?:pub\s+)?fn\s+selfhost_proof_solve_owner_transition\b[\s\S]*match\s+state:/,
    "owner obligation transitions must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)(?:pub\s+)?fn\s+selfhost_proof_solve_borrow_access\b[\s\S]*match\s+state:/,
    "borrow access transitions must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)(?:pub\s+)?fn\s+selfhost_proof_solve_lifetime_outlives\b[\s\S]*SelfhostLifetimeOutlivesError::RequiredLifetimeMismatch/,
    "lifetime outlives checks must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)(?:pub\s+)?fn\s+selfhost_proof_solve_effect_allowed\b[\s\S]*match\s+context:/,
    "effect boundary checks must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)(?:pub\s+)?fn\s+selfhost_proof_solve_type_kind_compatible\b[\s\S]*selfhost_type_kind_eq/,
    "type kind compatibility must live in the proof solver",
);
assert.match(
    proofSolver,
    /(?:^|\n)(?:pub\s+)?fn\s+selfhost_proof_solve_trait_impl_non_overlapping\b[\s\S]*match\s+fact\.relation:/,
    "trait impl coherence must live in the proof solver",
);
assert.match(
    proofApi,
    /^pub fn\s+selfhost_proof_source_span_valid\b[^\n]*Result unit SelfhostProofRefutation/m,
    "source span validity must preserve typed refutations instead of returning bool",
);
assert.match(
    traitRef,
    /selfhost_trait_impl_relation[\s\S]*selfhost_type_arena_types_equal/,
    "trait impl relation must be derived from the typed source type arena",
);
assert.doesNotMatch(
    traitRelationBlock,
    /"[A-Za-z0-9_.:-]+"/,
    "trait impl relation must not depend on trait or module name strings",
);
assert.match(borrowState, /pub enum SelfhostBorrowState:/, "borrow state must be a typed enum");
assert.match(borrowState, /pub enum SelfhostBorrowRequestKind:/, "borrow requests must be a typed enum");
assert.match(resourceCell, /pub enum SelfhostResourceCellState:/, "Resource cell state must be a typed enum");
assert.match(resourceCell, /pub enum SelfhostResourceCellEventKind:/, "Resource cell events must be a typed enum");
assert.match(owner, /pub struct SelfhostOwnerStorageId:/, "owner storage id must be a typed value");
assert.match(owner, /pub enum SelfhostOwnerState:/, "owner state must be a typed enum");
assert.match(owner, /pub enum SelfhostOwnerEventKind:/, "owner events must be a typed enum");
assert.match(
    owner,
    /BorrowView/,
    "owner model must represent non-owning pointer view creation without treating MemPtr as an owner",
);
assert.match(lifetime, /pub struct SelfhostLifetimeId:/, "lifetime id must be a typed value");
assert.match(lifetime, /pub enum SelfhostLifetimeScopePathKind:/, "lifetime scope path relation must be a typed enum");
assert.match(lifetime, /pub enum SelfhostLifetimeRelation:/, "lifetime relation must be a typed enum");
assert.match(lifetime, /pub enum SelfhostLifetimeUseKind:/, "lifetime use kind must be a typed enum");
assert.doesNotMatch(
    functionBlock(lifetime, "selfhost_lifetime_relation_from_positions"),
    /"[A-Za-z0-9_.:-]+"/,
    "lifetime relation derivation must not depend on module or lifetime name strings",
);
assert.match(
    functionBlock(lifetime, "selfhost_lifetime_relation_from_positions"),
    /SelfhostLifetimeRelation::Unrelated/,
    "depth-only lifetime helper must not prove outlives for different lifetime ids",
);

assert.match(moduleCheckerFacade, /pub #import "\.\/module\/summary" as \*/);
assert.match(moduleCheckerFacade, /pub #import "\.\/module\/memo_trait_public_surface_hash" as \*/);
assert.match(moduleCheckerFacade, /pub #import "\.\/module\/memo_trait_public_surface_normalizer" as \*/);
assert.match(moduleCheckerFacade, /pub #import "\.\/module\/memo_trait_public_surface_seed" as \*/);
assert.match(moduleCheckerFacade, /pub #import "\.\/module\/memo_trait_signature_shape" as \*/);
assert.match(moduleCheckerFacade, /pub #import "\.\/module\/memo_trait_source_evidence_producer" as \*/);
assert.match(moduleCheckerFacade, /pub #import "\.\/module\/memo_trait_source_fingerprint" as \*/);
assert.match(moduleCheckerFacade, /pub #import "\.\/module\/memo_trait_source_scan" as \*/);
assert.match(moduleCheckerFacade, /pub #import "\.\/module\/orchestrate" as \*/);
assert.deepEqual(
    Array.from(moduleCheckerFacade.matchAll(/^pub #import "([^"]+)" as ([^\n]+)$/gm), (match) => `${match[1]} as ${match[2]}`)
        .sort(),
    [
        "./module/memo_trait_public_surface_hash as *",
        "./module/memo_trait_public_surface_normalizer as *",
        "./module/memo_trait_public_surface_seed as *",
        "./module/memo_trait_signature_shape as *",
        "./module/memo_trait_source_evidence_producer as *",
        "./module/memo_trait_source_fingerprint as *",
        "./module/memo_trait_source_scan as *",
        "./module/orchestrate as *",
        "./module/summary as *",
    ],
    "module checker facade must re-export only the intended public modules",
);
assert.doesNotMatch(
    moduleCheckerFacade,
    /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/m,
    "module checker facade must stay implementation-free",
);
assert.doesNotMatch(
    moduleCheckerFacade,
    /#import "neplg2\/core\/proof"/,
    "module checker facade must not depend on proof implementation details",
);
assert.match(moduleCheckerImplementation, /#import "neplg2\/core\/proof" as \*/, "module checker internals must depend on the generic proof facade");
const publicModuleCheckerFunctions = Array.from(
    moduleCheckerPublicSurface.matchAll(/^pub fn\s+([A-Za-z0-9_]+)\b/gm),
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
    "selfhost_memo_trait_public_surface_hash_error_kind_eq",
    "selfhost_memo_trait_public_surface_hash_registry_error_kind_eq",
    "selfhost_memo_trait_public_surface_hash_materialize_result",
    "selfhost_memo_trait_trusted_source_registry_from_public_surface_hash_result",
    "selfhost_memo_trait_public_surface_hash_stage0",
    "selfhost_memo_trait_public_surface_normalizer_error_kind_eq",
    "selfhost_memo_trait_public_surface_normalizer_partial_input_items_result",
    "selfhost_memo_trait_public_surface_normalizer_stage0",
    "selfhost_memo_trait_public_surface_seed_error_kind_eq",
    "selfhost_memo_trait_public_surface_seed_registry_error_kind_eq",
    "selfhost_memo_trait_public_surface_seed_scan_module_result",
    "selfhost_memo_trait_trusted_source_registry_from_public_surface_seed_result",
    "selfhost_memo_trait_public_surface_seed_stage0",
    "selfhost_memo_trait_definition_scan_error_kind_eq",
    "selfhost_memo_trait_definition_scan_registry_error_kind_eq",
    "selfhost_memo_trait_definition_source_table_scan_module_result",
    "selfhost_memo_trait_trusted_source_registry_scan_module_result",
    "selfhost_memo_trait_definition_scan_stage0",
    "selfhost_memo_trait_stable_source_module_seed_new",
    "selfhost_memo_trait_stable_source_trait_seed_new",
    "selfhost_memo_trait_stable_source_seed_table_empty",
    "selfhost_memo_trait_stable_source_seed_table_add_record",
    "selfhost_memo_trait_stable_source_seed_error_kind_eq",
    "selfhost_memo_trait_stable_source_seed_registry_error_kind_eq",
    "selfhost_memo_trait_stable_source_evidence_table_from_seed_table_result",
    "selfhost_memo_trait_trusted_source_registry_from_seed_evidence_result",
    "selfhost_memo_trait_stable_source_seed_stage0",
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
    moduleCheckerOrchestrate,
    /^pub struct SelfhostModuleCheckStep:/m,
    "module checker step state must stay private to avoid exposing checker-internal proof sequencing",
);
assert.match(
    moduleCheckerDiagnostic,
    /SelfhostProofRefutation::UnexpectedEvidence\s+_issue:/,
    "module checker must handle unexpected proof evidence explicitly",
);
assert.match(
    moduleCheckerDeclarationAdapter,
    /match\s+selfhost_proof_source_span_valid\s+item\.span:/,
    "module item span validation must match on the proof solver's typed result",
);
assert.doesNotMatch(
    moduleCheckerDeclarationAdapter,
    /if:[\s\S]{0,120}selfhost_proof_source_span_valid\s+item\.span/,
    "module item span validation must not collapse proof result to a boolean predicate",
);
assert.doesNotMatch(
    moduleCheckerImplementation,
    /source_span_is_valid\s+item\.span/,
    "module checker must not bypass proof for module item span validation",
);
assert.doesNotMatch(
    moduleCheckerImplementation,
    /enum\s+SelfhostModuleRawState:/,
    "module checker must not own a checker-local raw backend proof state enum",
);
assert.match(
    moduleCheckerRawAdapter,
    /selfhost_proof_raw_backend_transition\s+state\s+item/,
    "module checker must ask the proof solver for raw backend transitions",
);
assert.match(
    moduleCheckerDeclarationAdapter,
    /selfhost_proof_module_directive_transition\s+state\s+item/,
    "module checker must ask the proof solver for module directive transitions",
);
assert.match(
    moduleCheckerDeclarationAdapter,
    /selfhost_proof_module_declaration_header\s+kind\s+selfhost_module_declaration_item_fact\s+item/,
    "module checker must ask the proof solver for declaration header availability",
);
assert.doesNotMatch(
    moduleCheckerImplementation,
    /if:\s*\n\s+gt\s+summary\.(?:entry_count|target_count)\s+1/,
    "module checker must not validate singleton directives by summary count checks",
);
assert.doesNotMatch(
    checker,
    /#import "neplg2\/core\/proof"/,
    "checker facade should stay orchestration-only and avoid direct proof implementation coupling",
);

console.log("selfhost proof entry contract passed");
