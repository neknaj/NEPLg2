#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const ROOT = path.resolve(__dirname, "..");
const TYPECHECK_DIR = path.join(ROOT, "nepl-core", "src", "typecheck");
const FUNCTION_CHECK = path.join(TYPECHECK_DIR, "function_check.rs");
const SELECTED_CALL_APPLY = path.join(TYPECHECK_DIR, "selected_call_apply.rs");
const MONOMORPHIZE = path.join(ROOT, "nepl-core", "src", "monomorphize.rs");
const PLAN = path.join(ROOT, "doc", "neplg2", "abstraction_static_verification_plan.md");
const RUNNER = path.join(ROOT, "nodesrc", "run_source_policy_regressions.js");

const BASELINE = {
    parseTraitRefName: 0,
    formatTraitRefName: 7,
    traitBoundRef: 0,
    traitLookupCache: 7,
    implInfoOptionString: 1,
};

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function read(filePath) {
    return fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
}

function walkRustFiles(dir) {
    const files = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const child = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...walkRustFiles(child));
        } else if (entry.isFile() && entry.name.endsWith(".rs")) {
            files.push(child);
        }
    }
    return files;
}

function countOccurrences(text, needle) {
    return text.split(needle).length - 1;
}

function countAll(files, needle) {
    return files.reduce((sum, filePath) => sum + countOccurrences(read(filePath), needle), 0);
}

const plan = read(PLAN);
for (const marker of [
    "ISS-20260512T143721313Z-GENERIC-AND-TRAIT-ABSTRACTION-MODEL--1F2FF429",
    "TraitApplication",
    "ImplKind",
    "PendingTraitCheck",
    "MonoTraitLookupKey",
    "source policy",
]) {
    assert(plan.includes(marker), `abstraction static verification plan must mention ${marker}`);
}

const runner = read(RUNNER);
assert(
    runner.includes("nodesrc/test_abstraction_static_verification_policy.js"),
    "source policy runner must include abstraction static verification policy",
);

const files = walkRustFiles(TYPECHECK_DIR).concat([MONOMORPHIZE]);
const counts = {
    parseTraitRefName: countAll(files, "parse_trait_ref_name"),
    formatTraitRefName: countAll(files, "format_trait_ref_name"),
    traitBoundRef: countAll(files, "TraitBoundRef"),
    implInfo: countAll(files, "ImplInfo"),
    traitLookupCache: countAll(files, "trait_lookup_cache"),
    implInfoOptionString: countOccurrences(read(path.join(TYPECHECK_DIR, "traits.rs")), "Option<String>"),
};

assert(counts.parseTraitRefName <= BASELINE.parseTraitRefName, "trait ref string parser usage must not grow");
assert(counts.formatTraitRefName <= BASELINE.formatTraitRefName, "trait ref string formatting usage must not grow");
assert(counts.traitBoundRef <= BASELINE.traitBoundRef, "TraitBoundRef old model must not be reintroduced");
assert(counts.traitLookupCache <= BASELINE.traitLookupCache, "string-keyed trait lookup cache usage must not grow");
assert(
    counts.implInfoOptionString <= BASELINE.implInfoOptionString,
    "ImplInfo optional string model must not gain new optional string fields",
);

const traits = read(path.join(TYPECHECK_DIR, "traits.rs"));
assert(traits.includes("pub(super) enum TraitCapability"), "TraitCapability must remain an enum");
assert(traits.includes("TraitCapability::Copy"), "TraitCapability::Copy match coverage must remain visible");
assert(traits.includes("TraitCapability::Clone"), "TraitCapability::Clone match coverage must remain visible");
assert(traits.includes("TraitCapability::Drop"), "TraitCapability::Drop match coverage must remain visible");
assert(traits.includes("pub(super) struct TraitApplication"), "TraitApplication must be a typed model");
assert(traits.includes("pub(super) enum ImplKind"), "ImplKind must model impl identity as an enum");
assert(traits.includes("ImplKind::Inherent"), "ImplKind::Inherent match branch must remain visible");
assert(traits.includes("ImplKind::Trait"), "ImplKind::Trait match branch must remain visible");
assert(
    traits.includes("application: TraitApplication"),
    "trait impl identity must store a typed TraitApplication",
);
assert(
    traits.includes("self_ty: TypeId"),
    "trait impl identity must store trait self TypeId explicitly",
);
const implInfoStruct = traits.match(/pub\(super\) struct ImplInfo\s*\{[\s\S]*?\n\}/);
assert(implInfoStruct, "ImplInfo struct body must be visible to source policy");
assert(implInfoStruct[0].includes("kind: ImplKind"), "ImplInfo must own ImplKind");
assert(!implInfoStruct[0].includes("trait_name"), "ImplInfo must not store rendered trait names");
assert(!implInfoStruct[0].includes("trait_base_name"), "ImplInfo must not store split trait base names");
assert(!implInfoStruct[0].includes("trait_args"), "ImplInfo must not store split trait args");
assert(!implInfoStruct[0].includes("Option<"), "ImplInfo must not encode kind through optional fields");
assert(traits.includes("pub(super) struct TraitBound"), "typed trait bound model must be named TraitBound");
assert(
    !traits.includes("pub(super) struct TraitBoundRef"),
    "old TraitBoundRef model must not be reintroduced",
);
const traitBoundStruct = traits.match(/pub\(super\) struct TraitBound\s*\{[\s\S]*?\n\}/);
assert(traitBoundStruct, "TraitBound struct body must be visible to source policy");
assert(
    traitBoundStruct[0].includes("application: TraitApplication"),
    "TraitBound must own a typed TraitApplication",
);
assert(
    !/\n\s*pub\(super\) name:\s*String/.test(traitBoundStruct[0]),
    "TraitBound must not store rendered diagnostic names",
);
assert(
    !traitBoundStruct[0].includes("trait_base_name"),
    "TraitBound must not split trait application base name out of TraitApplication",
);
assert(
    !traitBoundStruct[0].includes("trait_args"),
    "TraitBound must not split trait application args out of TraitApplication",
);

const functionCheck = read(FUNCTION_CHECK);
assert(
    functionCheck.includes("type_param_has_trait_application_bound"),
    "deferred function-level trait checks must use typed trait application lookup",
);
assert(
    !functionCheck.includes("type_param_has_trait_bound("),
    "deferred function-level trait checks must not call rendered-name trait bound lookup",
);
assert(
    !functionCheck.includes("&bound.name"),
    "deferred function-level trait checks must not use rendered bound names as authority",
);
const traitCheck = read(path.join(TYPECHECK_DIR, "trait_check.rs"));
const traitCallApply = read(path.join(TYPECHECK_DIR, "trait_call_apply.rs"));
const selectedCallApply = read(SELECTED_CALL_APPLY);
assert(
    !traitCheck.includes("type_param_has_bound_ref"),
    "old type_param_has_bound_ref helper must not be reintroduced",
);
assert(
    !traitCheck.includes("same_label"),
    "BlockChecker trait bound lookup must not duplicate label fallback outside the typed helper",
);
assert(
    traitCheck.includes("type_param_has_trait_application_bound("),
    "BlockChecker trait bound lookup must delegate to the typed helper",
);
for (const [name, text] of [
    ["function_check.rs", functionCheck],
    ["trait_check.rs", traitCheck],
    ["trait_call_apply.rs", traitCallApply],
]) {
    assert(!text.includes("imp.trait_base_name"), `${name} must not inspect split impl trait base names`);
    assert(!text.includes("imp.trait_args"), `${name} must not inspect split impl trait args`);
}
assert(
    traitCallApply.includes("pub(super) enum TraitMethodResolution"),
    "trait method resolution must use a typed enum",
);
for (const variant of [
    "TraitMethodResolution::NotTraitMethod",
    "TraitMethodResolution::Resolved",
    "TraitMethodResolution::MissingSelfType",
    "TraitMethodResolution::UnsatisfiedBound",
    "TraitMethodResolution::PureCallsImpure",
]) {
    assert(traitCallApply.includes(variant), `trait_call_apply.rs must handle ${variant}`);
    assert(selectedCallApply.includes(variant), `selected_call_apply.rs must handle ${variant}`);
}
assert(
    !traitCallApply.includes("Option<FuncRef>"),
    "trait method resolution must not collapse resolution state into Option<FuncRef>",
);
assert(
    !traitCallApply.includes("infer_selected_trait_method_callee"),
    "selected trait method resolution must not use the old optional callee helper",
);
assert(
    selectedCallApply.includes("match trait_resolution"),
    "selected callable trait resolution must match TraitMethodResolution explicitly",
);

const typedLookup = traits.match(
    /pub\(super\) fn type_param_has_trait_application_bound[\s\S]*?\npub\(super\) fn merge_inferred_instantiation/,
);
assert(typedLookup, "typed trait application bound lookup must exist before inference helper boundary");
assert(
    !typedLookup[0].includes("parse_trait_ref_name("),
    "typed trait application bound lookup must not parse rendered trait names",
);
assert(
    !typedLookup[0].includes("b.name"),
    "typed trait application bound lookup must not compare rendered bound names",
);
assert(
    !typedLookup[0].includes("same_label"),
    "typed trait application bound lookup must not accept same-label TypeId fallback",
);
assert(
    !typedLookup[0].includes("v.label.as_deref"),
    "typed trait application bound lookup must not inspect TypeVar labels as identity",
);
assert(
    !traits.includes("parse_trait_ref_name"),
    "rendered trait application parser must not be reintroduced",
);

console.log("abstraction static verification policy baseline ok");
console.log(JSON.stringify(counts, null, 2));
