#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

function functionBlock(source, name) {
    const start = source.indexOf(`fn ${name}`);
    assert.notEqual(start, -1, `missing function: ${name}`);
    const next = source.indexOf("\nfn ", start + 1);
    const nextPub = source.indexOf("\npub fn ", start + 1);
    const candidates = [next, nextPub].filter((index) => index !== -1);
    const end = candidates.length === 0 ? source.length : Math.min(...candidates);
    return source.slice(start, end);
}

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_stable_nominal_key_producer.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";

const source = read(relPath);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const sourceCode = source
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("//:"))
    .join("\n");

assertOrdered(
    source,
    [
        "# check/module/memo_trait_stable_nominal_key_producer",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "stable nominal key producer must document purpose, contract, current limitations, complexity, and a doctest",
);
assert.ok(
    source.includes("session-local `SelfhostNamedTypeId`") &&
        source.includes("source spelling、span、display name、diagnostic text、path suffix を fingerprint authority にせず"),
    "docs must reject session-local or display/source authority for stable nominal identity",
);
assert.doesNotMatch(
    facade,
    /memo_trait_stable_nominal_key_producer/,
    "stable nominal key producer must remain facade-private until full public surface orchestration is designed",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitStableNominalDeclarationKind:",
        "Struct",
        "Enum",
    ],
    "producer must distinguish struct and enum nominal declaration domains",
);
assertOrdered(
    source,
    [
        "pub struct SelfhostMemoTraitStableNominalKeyProducerInput:",
        "visibility %SelfhostModuleDeclarationVisibility",
        "module_fingerprint %i32",
        "declaration_ordinal %Option i32",
        "constructor_ordinal %Option i32",
        "kind %SelfhostMemoTraitStableNominalDeclarationKind",
        "type_arity %i32",
    ],
    "producer input must carry typed public nominal declaration seed fields",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitStableNominalKeyProducerErrorKind:",
        "PrivateVisibilityRejected",
        "ModuleFingerprintPlaceholder",
        "DeclarationOrdinalMissing",
        "DeclarationOrdinalPlaceholder",
        "ConstructorOrdinalMissing",
        "ConstructorOrdinalPlaceholder",
        "TypeArityNegative",
        "DefinitionFingerprintPlaceholder",
        "StableNominalKeyRejected %SelfhostMemoTraitStableNominalKeyErrorKind",
    ],
    "producer failures must keep seed validation and low-level stable key rejection separate",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_stable_nominal_key_producer_definition_fingerprint_result %fn i32 fn SelfhostMemoTraitStableNominalDeclarationKind fn i32 fn i32 Result i32 SelfhostMemoTraitStableNominalKeyProducerErrorKind"),
    "producer must expose typed definition fingerprint creation",
);
assert.ok(
    source.includes("pub fn selfhost_memo_trait_stable_nominal_key_producer_result %fn SelfhostMemoTraitStableNominalKeyProducerInput Result SelfhostMemoTraitStableNominalKey SelfhostMemoTraitStableNominalKeyProducerErrorKind"),
    "producer must expose stable nominal key creation from typed input",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_stable_nominal_key_producer_definition_fingerprint_result"),
    [
        "module_fingerprint",
        "ModuleFingerprintPlaceholder",
        "type_arity",
        "TypeArityNegative",
        "selfhost_memo_trait_stable_nominal_key_producer_schema_version",
        "selfhost_memo_trait_stable_nominal_declaration_kind_code kind",
        "declaration_ordinal",
        "DefinitionFingerprintPlaceholder",
    ],
    "definition fingerprint must fold typed module, declaration kind, ordinal, schema, and arity with placeholder rejection",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_stable_nominal_key_producer_result"),
    [
        "selfhost_memo_trait_stable_nominal_key_producer_visibility_result",
        "selfhost_memo_trait_stable_nominal_key_producer_ordinal_result input.declaration_ordinal",
        "selfhost_memo_trait_stable_nominal_key_producer_ordinal_result input.constructor_ordinal",
        "selfhost_memo_trait_stable_nominal_key_producer_definition_fingerprint_result",
        "selfhost_memo_trait_stable_nominal_key_result",
        "StableNominalKeyRejected",
    ],
    "producer must validate visibility and ordinals, build definition fingerprint, then delegate to low-level stable nominal key constructor",
);
assertOrdered(
    source,
    [
        "struct_input",
        "SelfhostMemoTraitStableNominalDeclarationKind::Struct 0",
        "enum_input",
        "SelfhostMemoTraitStableNominalDeclarationKind::Enum 0",
        "let differ %bool selfhost_memo_trait_stable_nominal_key_producer_stage0_compare",
    ],
    "stage0 must compare struct and enum stable nominal keys",
);
assert.doesNotMatch(
    sourceCode,
    /hash32\s+(?:source|span|alias|display|diag|diagnostic|lexeme)|mix\s+(?:source|span|alias|display|diag|diagnostic|lexeme)|\.span\b|\.lexeme\b|display_name|diagnostic_text/,
    "producer implementation must not fold source text, spans, aliases, display names, lexemes, or diagnostic text into accepted hash material",
);
assert.doesNotMatch(
    sourceCode,
    /#import ".*(?:hir|resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader)/,
    "producer must not import HIR, Resource IR, backend, proof store, or serialized proof artifact layers",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_stable_nominal_key_producer/,
    "checker-layer stable nominal key producer must not be registered in the ty source list",
);
assert.doesNotMatch(
    sourceCode,
    /line count|comment length|file size|500 行|行数制限|行数上限|コメント長制限|コメント長上限|doc comment length cap|doc-comment-length cap/i,
    "producer policy must not introduce line-count or doc-comment-length restrictions",
);

console.log("selfhost memo trait stable nominal key producer contract passed");
