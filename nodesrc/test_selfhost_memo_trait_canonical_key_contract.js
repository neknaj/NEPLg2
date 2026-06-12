#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const {
    TY_FACADE,
    readRepoFile,
} = require("./selfhost_ty_sources");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/ty/ty/memo_trait_canonical_key.nepl";
const proofStoreRelPath = "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl";
const facade = readRepoFile(repoRoot, TY_FACADE);
const source = readRepoFile(repoRoot, relPath);
const proofStore = readRepoFile(repoRoot, proofStoreRelPath);
const codeOnly = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assert.match(
    facade,
    /^pub #import "\.\/ty\/memo_trait_canonical_key" as \*$/m,
    "ty facade must re-export the memo trait canonical key split module",
);
assert.match(
    source,
    /# ty\/memo_trait_canonical_key[\s\S]*\[目的\/もくてき\]:[\s\S]*\[契約\/けいやく\]:[\s\S]*\[現状\/げんじょう\]:[\s\S]*\[計算量\/けいさんりょう\]:[\s\S]*neplg2:test/,
    "memo trait canonical key module documentation must record purpose, contract, current limitations, complexity, and a doctest",
);
assert.match(
    source,
    /`SelfhostNamedTypeId` を保持します[\s\S]*永続 artifact の key としては不十分/,
    "module documentation must explain why session-local SelfhostNamedTypeId is insufficient for persistent proof artifacts",
);
assert.doesNotMatch(
    source,
    /#import "neplg2\/core\/(?:check|lower|hir|resource|backend)\//,
    "memo trait canonical key must stay in core/ty and must not depend on checker, HIR, Resource IR, or backend layers",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableNominalKey:[\s\S]*schema_version %i32[\s\S]*module_fingerprint %i32[\s\S]*definition_fingerprint %i32[\s\S]*constructor_ordinal %i32[\s\S]*type_arity %i32[\s\S]*nominal_key_hash %i32/,
    "stable nominal key must keep schema, module, definition, constructor ordinal, arity, and derived key hash as typed fields",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableNominalKeyRecord:[\s\S]*nominal_id %SelfhostNamedTypeId[\s\S]*key %SelfhostMemoTraitStableNominalKey/,
    "stable nominal key table records must map session-local named ids to stable nominal keys",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitStableNominalKeyTable:[\s\S]*records %Vec SelfhostMemoTraitStableNominalKeyRecord/,
    "stable nominal key table must own typed records instead of exposing loose Vec payloads",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitStableNominalKeyErrorKind:[\s\S]*ModuleFingerprintMissing[\s\S]*ModuleFingerprintPlaceholder[\s\S]*DefinitionFingerprintMissing[\s\S]*DefinitionFingerprintPlaceholder[\s\S]*ConstructorOrdinalMissing[\s\S]*ConstructorOrdinalPlaceholder[\s\S]*TypeArityNegative[\s\S]*DerivedNominalKeyPlaceholder/,
    "stable nominal key construction must reject missing, placeholder, negative, and derived-placeholder inputs with typed errors",
);
assert.match(
    source,
    /pub enum SelfhostMemoTraitCanonicalFingerprintErrorKind:[\s\S]*MissingCanonicalNode[\s\S]*MissingCanonicalArgument[\s\S]*InvalidCanonicalArgumentRange[\s\S]*DerivedFingerprintPlaceholder[\s\S]*TraversalFuelExhausted[\s\S]*MissingNominalKey[\s\S]*DuplicateNominalKey[\s\S]*TypeParameterUnsupported[\s\S]*FunctionTypeUnsupported/,
    "canonical fingerprint projection must expose typed fail-closed errors for malformed keys, traversal bounds, nominal table failures, generic parameters, and function types",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitCanonicalTypeFingerprint:[\s\S]*schema_version %i32[\s\S]*root_hash %i32/,
    "canonical type fingerprint must keep a schema version with the root hash",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_type_fingerprint_eq[\s\S]*eq a\.schema_version b\.schema_version[\s\S]*eq a\.root_hash b\.root_hash/,
    "canonical type fingerprint equality must compare both schema version and root hash",
);
assert.match(
    source,
    /pub struct SelfhostMemoTraitCanonicalKeyStage0Summary:[\s\S]*parameter_unsupported %Result SelfhostMemoTraitCanonicalTypeFingerprint SelfhostMemoTraitCanonicalFingerprintErrorKind[\s\S]*fuel_exhausted %Result SelfhostMemoTraitCanonicalTypeFingerprint SelfhostMemoTraitCanonicalFingerprintErrorKind/,
    "stage0 summary must execute-check the traversal fuel exhaustion boundary rather than relying only on source regex checks",
);
assert.match(
    source,
    /pub fn selfhost_memo_trait_stable_nominal_key_result[\s\S]*Option::None:[\s\S]*ModuleFingerprintMissing[\s\S]*DefinitionFingerprintMissing[\s\S]*ConstructorOrdinalMissing[\s\S]*TypeArityNegative[\s\S]*DerivedNominalKeyPlaceholder/,
    "stable nominal key constructor must use Option and enum errors rather than accepting raw placeholder integers",
);
assert.match(
    source,
    /selfhost_memo_trait_stable_nominal_key_eq[\s\S]*eq a\.schema_version b\.schema_version[\s\S]*eq a\.module_fingerprint b\.module_fingerprint[\s\S]*eq a\.definition_fingerprint b\.definition_fingerprint[\s\S]*eq a\.constructor_ordinal b\.constructor_ordinal[\s\S]*eq a\.type_arity b\.type_arity[\s\S]*eq a\.nominal_key_hash b\.nominal_key_hash/,
    "stable nominal key equality must compare all typed fields",
);
assert.match(
    source,
    /fn selfhost_memo_trait_stable_nominal_key_table_find_loop[\s\S]*Option SelfhostMemoTraitStableNominalKey[\s\S]*DuplicateNominalKey[\s\S]*MissingNominalKey/,
    "nominal key lookup must distinguish missing and duplicate records instead of using first-wins",
);
assert.match(
    source,
    /fn selfhost_memo_trait_canonical_type_fingerprint_node_result[\s\S]*SelfhostCanonicalTypeKeyNode::Primitive[\s\S]*SelfhostCanonicalTypeKeyNode::Named[\s\S]*SelfhostCanonicalTypeKeyNode::Parameter[\s\S]*TypeParameterUnsupported[\s\S]*SelfhostCanonicalTypeKeyNode::Applied[\s\S]*SelfhostCanonicalTypeKeyNode::Function[\s\S]*FunctionTypeUnsupported/,
    "canonical fingerprint projection must handle every canonical key node variant explicitly and fail closed for unsupported variants",
);
assert.match(
    source,
    /SelfhostCanonicalTypeKeyNode::Named nominal_id:[\s\S]*selfhost_memo_trait_stable_nominal_key_table_find_result nominal_table nominal_id/,
    "named canonical key nodes must resolve through the stable nominal key table",
);
assert.match(
    source,
    /SelfhostCanonicalTypeKeyNode::Applied applied:[\s\S]*selfhost_memo_trait_stable_nominal_key_table_find_result nominal_table applied\.nominal_id[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_args_result/,
    "applied canonical key nodes must resolve the stable nominal key and recursively fingerprint type arguments",
);
assert.match(
    source,
    /fn selfhost_memo_trait_canonical_type_fingerprint_args_result[\s\S]*selfhost_canonical_type_key_arg_range_is_valid[\s\S]*selfhost_canonical_type_key_arena_arg arena args idx[\s\S]*MissingCanonicalArgument/,
    "argument fingerprinting must validate canonical arg ranges and fail closed on missing argument slots",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_type_fingerprint_result[\s\S]*selfhost_canonical_type_key_arena_node_len arena[\s\S]*selfhost_canonical_type_key_arena_arg_len arena[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_node_result nominal_table arena root fuel/,
    "public canonical fingerprint projection must derive traversal fuel from arena node and argument counts",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_type_fingerprint_node_result[\s\S]*le fuel 0[\s\S]*TraversalFuelExhausted[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_args_result[\s\S]*le fuel 0[\s\S]*TraversalFuelExhausted/,
    "node and argument traversal must fail closed when malformed canonical key input exhausts traversal fuel",
);
assert.match(
    source,
    /fn selfhost_memo_trait_canonical_key_stage0_cyclic_fuel_projection %impure fn &SelfhostMemoTraitStableNominalKeyTable Result SelfhostMemoTraitCanonicalKeyStage0FuelProjection StdErrorKind \\table:[\s\S]*SelfhostCanonicalTypeKeyNode::Applied[\s\S]*v::push args0 root[\s\S]*selfhost_memo_trait_canonical_key_stage0_cyclic_fuel_projection_from_parts table nodes1 args1/,
    "stage0 must construct a cyclic canonical key arena for the traversal fuel smoke",
);
assert.match(
    source,
    /selfhost_memo_trait_canonical_key_stage0_cyclic_fuel_projection_from_parts[\s\S]*selfhost_memo_trait_canonical_type_fingerprint_result table &arena root[\s\S]*selfhost_canonical_type_key_arena_free arena[\s\S]*SelfhostMemoTraitCanonicalKeyStage0FuelProjection result[\s\S]*fuel_exhausted/,
    "stage0 must execute-check TraversalFuelExhausted and expose the result in the summary",
);
assert.doesNotMatch(
    codeOnly,
    /source_text|source_span|span|path_suffix|display_name|diagnostic|lexeme/,
    "accepted canonical key code must not use source text, spans, paths, display names, diagnostics, or lexemes as authority",
);
assert.doesNotMatch(
    codeOnly,
    /selfhost_memo_trait_stable_definition_key_result|memo_trait_definition_key|core\/check\/module/,
    "ty canonical key module must not depend on the checker-layer stable definition key producer",
);
assert.doesNotMatch(
    codeOnly,
    /selfhost_memo_trait_source_identity_new|signature_available\s*=\s*true/,
    "canonical key projection must not construct trusted source identities or source records directly",
);
assert.match(
    proofStore,
    /memo_trait_canonical_key\.nepl[\s\S]*stable nominal key table と canonical type fingerprint の sidecar projection[\s\S]*stable API[\s\S]*cross-arena canonical equality[\s\S]*policy equality[\s\S]*producer gate/,
    "proof store documentation must state that the stable sidecar path still requires canonical equality, policy equality, and producer gate validation",
);
assert.doesNotMatch(
    source,
    /line count|comment length|file size|500 行/,
    "memo trait canonical key policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait canonical key contract passed");
