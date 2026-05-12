#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const ROOT = path.resolve(__dirname, "..");
const TYPECHECK_DIR = path.join(ROOT, "nepl-core", "src", "typecheck");
const FUNCTION_CHECK = path.join(TYPECHECK_DIR, "function_check.rs");
const MONOMORPHIZE = path.join(ROOT, "nepl-core", "src", "monomorphize.rs");
const PLAN = path.join(ROOT, "doc", "neplg2", "abstraction_static_verification_plan.md");
const RUNNER = path.join(ROOT, "nodesrc", "run_source_policy_regressions.js");

const BASELINE = {
    parseTraitRefName: 0,
    formatTraitRefName: 7,
    traitBoundRef: 0,
    implInfo: 8,
    traitLookupCache: 7,
    implInfoOptionString: 3,
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
assert(counts.implInfo <= BASELINE.implInfo, "ImplInfo old model usage must not grow");
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
    !traits.includes("parse_trait_ref_name"),
    "rendered trait application parser must not be reintroduced",
);

console.log("abstraction static verification policy baseline ok");
console.log(JSON.stringify(counts, null, 2));
