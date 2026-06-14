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

function before(text, marker) {
    const index = text.indexOf(marker);
    assert.notEqual(index, -1, `missing marker ${marker}`);
    return text.slice(0, index);
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

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?${kind}\\s+${name}\\b`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${kind} ${name} not found`);
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

function assertOrdered(text, snippets, message) {
    let offset = 0;
    for (const snippet of snippets) {
        const found = text.indexOf(snippet, offset);
        assert.notEqual(found, -1, `${message}: missing ${snippet}`);
        offset = found + snippet.length;
    }
}

const relPath = "stdlib/neplg2/core/check/module/memo_trait_operation_drop_absence_producer.nepl";
const facadeRelPath = "stdlib/neplg2/core/check/module.nepl";
const tySourceListRelPath = "nodesrc/selfhost_ty_sources.js";
const runnerRelPath = "nodesrc/run_source_policy_regressions.js";
const source = read(relPath);
const code = stripDocComments(source);
const productionCode = stripDocComments(
    before(source, "//: selfhost_memo_trait_operation_drop_absence_error_kind_code"),
);
const facade = read(facadeRelPath);
const tySourceList = read(tySourceListRelPath);
const runner = read(runnerRelPath);

assertOrdered(
    source,
    [
        "# check/module/memo_trait_operation_drop_absence_producer",
        "[目的/もくてき]:",
        "[契約/けいやく]:",
        "[現状/げんじょう]:",
        "[計算量/けいさんりょう]:",
        "neplg2:test",
    ],
    "drop absence producer must document purpose, contract, current state, complexity, and doctest",
);
assert.ok(
    source.includes("complete public impl surface") &&
        source.includes("Drop impl が 0 件") &&
        source.includes("`NoDropRequired` evidence"),
    "docs must define this module as the complete-public-surface no-drop absence boundary",
);
assert.ok(
    source.includes("source text、span、lexeme、display name、diagnostic text、module path、public surface hash、payload hash、HIR body root、Resource IR graph、proof store record") &&
        source.includes("`DropImplPresent` を返した場合、この module は `PureDrop` へ進めません") &&
        source.includes("operation evidence record、aggregate proof、proof store、generic binder、PrivateCache / PrivateState masking、backend artifact、prechecked artifact"),
    "docs must reject textual/hash/proof-store authority and keep pure-Drop/resource/backend layers out of scope",
);
assert.ok(
    source.includes("行数や doc comment の長さによる制限は置きません"),
    "docs must explicitly prefer detailed comments over artificial volume gates",
);
{
    const lines = source.split("\n");
    const missingDocs = [];
    for (let i = 0; i < lines.length; i += 1) {
        if (/^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/.test(lines[i])) {
            let j = i - 1;
            while (j >= 0 && lines[j].trim() === "") {
                j -= 1;
            }
            if (j < 0 || !lines[j].trimStart().startsWith("//:")) {
                missingDocs.push(`${i + 1}: ${lines[i]}`);
            }
        }
    }
    assert.deepEqual(
        missingDocs,
        [],
        "every drop absence producer declaration, including private helpers and impl blocks, must have a preceding doc comment",
    );
}
assert.doesNotMatch(
    facade,
    /memo_trait_operation_drop_absence_producer/,
    "drop absence producer must remain facade-private until operation evidence orchestration chooses the public surface",
);
assert.doesNotMatch(
    tySourceList,
    /memo_trait_operation_drop_absence_producer/,
    "checker-layer drop absence producer must not be registered in the ty source list",
);
assert.ok(
    runner.includes("nodesrc/test_selfhost_memo_trait_operation_drop_absence_producer_contract.js"),
    "source policy runner must execute the drop absence producer contract",
);
{
    const memoTraitImports = [...source.matchAll(/^#import "(\.\/memo_trait[^"]+)" as \*/gm)].map(
        (match) => match[1],
    );
    assert.deepEqual(
        memoTraitImports,
        [
            "./memo_trait_operation_drop_impl_resolver",
            "./memo_trait_operation_evidence_producer",
            "./memo_trait_operation_purity_gate",
        ],
        "drop absence producer must keep a minimal checker-layer memo_trait import allow-list",
    );
}
assert.doesNotMatch(
    code,
    /#import ".*(?:resource|backend|memo_trait_proof_store|memo_trait_proof_artifact|memo_trait_proof_reader|memo_trait_proof_serializer|memo_trait_proof_preseed|memo_trait_proof_decoded|memo_trait_proof_payload_reader|memo_trait_canonical_key|memo_trait_public_surface|memo_trait_public_impl_scanner|memo_trait_operation_public_impl_materializer|memo_trait_operation_public_impl_drop_fact_orchestrator|memo_trait_operation_drop_resource|memo_trait_operation_method_body|private_cache|private_state)/,
    "drop absence producer must not import Resource IR, backend, proof store/artifact, canonical-key, public-surface, scanner/materializer, drop-resource, method-body, PrivateCache, or PrivateState layers",
);
assertOrdered(
    source,
    [
        "pub enum SelfhostMemoTraitOperationDropAbsenceProducerErrorKind:",
        "ResolverRejected %SelfhostMemoTraitOperationDropImplResolverErrorKind",
        "PurityGateRejected %SelfhostMemoTraitOperationPurityGateErrorKind",
        "DropImplPresent",
        "SurfaceMissing",
        "SurfaceUnknown",
        "DropCheckNotRequired",
        "UnexpectedDropEvidence %SelfhostMemoTraitOperationDropEvidence",
    ],
    "producer errors must preserve resolver, purity gate, present, incomplete surface, not-required, and unexpected evidence failures",
);
assert.doesNotMatch(
    topLevelBlock(source, "enum", "SelfhostMemoTraitOperationDropAbsenceProducerErrorKind"),
    /%bool|%str|%String|String|MlString|message|text/i,
    "producer errors must not encode structural failures as bool or string messages",
);
assert.doesNotMatch(
    code,
    /Result\s+bool|Result\s+str|Result\s+String|Result\s+MlString|Result::Err\s+(true|false)|Result::Err\s+"/,
    "producer APIs must return typed Result errors instead of bool/string errors",
);
assert.doesNotMatch(
    productionCode,
    /\b(SelfhostMemoTraitOperationEvidenceRecord|SelfhostMemoTraitAggregateProof|SelfhostMemoTraitProofStore|selfhost_memo_trait_operation_evidence_record_new|selfhost_memo_trait_aggregate_proof_to_record|PrivateCache|PrivateState|prechecked|backend)\b/,
    "production producer functions must not construct operation records, aggregate proof, proof-store, private-cache/state, prechecked, or backend values",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_absence_from_absent_check_result"),
    [
        "selfhost_memo_trait_operation_purity_gate_drop_evidence_result SelfhostMemoTraitOperationEvidenceKind::Drop check",
        "Result::Ok evidence:",
        "SelfhostMemoTraitOperationDropEvidence::NoDropRequired:",
        "Result::Ok evidence",
        "SelfhostMemoTraitOperationDropEvidence::PureDrop:",
        "UnexpectedDropEvidence evidence",
        "Result::Err gate_error:",
        "PurityGateRejected gate_error",
    ],
    "absent check conversion must go through the purity gate and accept only its NoDropRequired result",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_absence_from_check_result"),
    [
        "SelfhostMemoTraitOperationDropCheckKind::DropImplAbsent:",
        "selfhost_memo_trait_operation_drop_absence_from_absent_check_result check",
        "SelfhostMemoTraitOperationDropCheckKind::DropImplPresent:",
        "DropImplPresent",
        "SelfhostMemoTraitOperationDropCheckKind::Missing:",
        "SurfaceMissing",
        "SelfhostMemoTraitOperationDropCheckKind::Unknown:",
        "SurfaceUnknown",
        "SelfhostMemoTraitOperationDropCheckKind::NotRequired:",
        "DropCheckNotRequired",
    ],
    "check conversion must fail closed for present, missing, unknown, and not-required states",
);
assertOrdered(
    functionBlock(source, "selfhost_memo_trait_operation_drop_absence_evidence_result"),
    [
        "selfhost_memo_trait_operation_drop_impl_resolve_result surface table type_id",
        "Result::Ok check:",
        "selfhost_memo_trait_operation_drop_absence_from_check_result check",
        "Result::Err resolver_error:",
        "ResolverRejected resolver_error",
    ],
    "public API must delegate absence authority to the drop impl resolver before converting evidence",
);
assert.ok(
    source.includes("SelfhostMemoTraitOperationDropImplSurfaceState::Complete") &&
        source.includes("SelfhostMemoTraitOperationDropImplSurfaceState::Missing") &&
        source.includes("SelfhostMemoTraitOperationDropImplSurfaceState::Unknown") &&
        source.includes("SelfhostMemoTraitOperationDropImplResolverErrorKind::RecordDuplicate"),
    "stage0 doctest must cover complete absence, incomplete surface rejection, and duplicate resolver rejection",
);

console.log("selfhost memo trait operation drop absence producer contract passed");
