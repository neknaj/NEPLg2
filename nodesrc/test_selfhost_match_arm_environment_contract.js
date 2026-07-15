"use strict";

const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(
    path.join(root, "stdlib/neplg2/core/check/expr/match_arm_environment.nepl"),
    "utf8",
).replace(/\r\n/g, "\n");
const implementation = source.replace(/^\s*\/\/.*$/gm, "");

for (const needle of [
    "scope %SelfhostNameScope",
    "value_types %SelfhostValueTypeEvidenceTable",
    "arm_ordinal %i32",
    "def_id %SelfhostDefId",
    "selfhost_name_scope_fork parent_scope",
    "selfhost_value_type_evidence_table_fork parent_values",
    "selfhost_name_scope_add_result_def_id &added",
    "selfhost_value_type_evidence_new def_id bind_type span",
    "SelfhostMatchArmLocalId arm_ordinal def_id",
]) {
    if (!implementation.includes(needle)) {
        throw new Error(`match arm environment authority missing: ${needle}`);
    }
}

for (const cleanup of [
    /Result::Err e:\n\s+selfhost_name_scope_free scope0\n\s+Result::Err SelfhostMatchArmEnvironmentErrorKind::EvidenceFork e/,
    /Result::Err e:\n\s+selfhost_value_type_evidence_table_free values0\n\s+Result::Err SelfhostMatchArmEnvironmentErrorKind::ScopeAdd e/,
    /Result::Err e:\n\s+selfhost_name_scope_free scope\n\s+Result::Err SelfhostMatchArmEnvironmentErrorKind::EvidenceAdd e/,
]) {
    if (!cleanup.test(implementation)) {
        throw new Error(`match arm environment cleanup missing: ${cleanup}`);
    }
}

if (!source.includes("Borrowed bindをOwnedへfallbackしません")) {
    throw new Error("match arm environment must document the no-fallback source policy");
}

if (/pub fn selfhost_match_arm_environment_owned_mode_fixture_result/.test(implementation)) {
    throw new Error("raw match arm environment fixture must remain module-private");
}

console.log("selfhost match arm environment contract: pass");
