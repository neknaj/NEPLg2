#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const read = (p) => fs.readFileSync(path.join(root, p), "utf8").replace(/\r\n/g, "\n");
const projection = read("stdlib/neplg2/core/resolve/type_resolver/enum_payload_projection.nepl");
const substitution = read("stdlib/neplg2/core/ty/ty/substitution.nepl");
const origin = read("stdlib/neplg2/core/resolve/type_resolver/exported_enum_origin.nepl");

for (const needle of [
  "selfhost_resolved_enum_session_definition_parameter_name_span",
  "selfhost_type_parameter_env_add_checked",
  "selfhost_type_prefix_list_from_syntax_range",
  "selfhost_type_prefix_list_reduce_with_constructors_and_type_parameters",
  "selfhost_type_project_root_with_constructors_into_arena",
  "selfhost_type_arena_applied_arg arena actual_type ordinal",
  "selfhost_type_parameter_binding_new_unchecked 0 ordinal",
  "selfhost_type_substitution_result arena &complete payload_type",
  "selfhost_type_substitution_result_into_output",
  "selfhost_enum_payload_definition_count session member.nominal_id 0 0",
  "SelfhostResolvedEnumPayloadProjectionErrorKind::DefinitionDuplicate",
  "not eq definition_count 1",
]) assert.ok(projection.includes(needle), `missing payload authority step: ${needle}`);

assert.doesNotMatch(projection, /variant_name_eq|string_search|canonical.*payload|selfhost_inference_binding/, "payload projection must not derive type bindings from spelling or inference retry state");
assert.match(substitution, /pub struct SelfhostTypeSubstitutionOutput:[\s\S]*arena %SelfhostTypeArena[\s\S]*output_type_id %SelfhostTypeId/, "substitution output must retain its arena owner");
assert.match(origin, /selfhost_exported_enum_origin_context_project_payload_result[\s\S]*field::get context "enum_session"[\s\S]*field::get context "arena"[\s\S]*selfhost_resolved_enum_member_payload_project_result[\s\S]*SelfhostExportedEnumOriginContext next_arena session witness/, "origin must consume and reconstruct its owner around payload projection");

console.log("selfhost enum payload projection contract passed");
