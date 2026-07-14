const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const implementation = (file) => read(file).replace(/^\s*\/\/.*$/gm, "");
const context = implementation("stdlib/neplg2/core/syntax/parser/match_context.nepl");
const scrutinee = implementation("stdlib/neplg2/core/check/expr/match_scrutinee.nepl");
const origin = implementation("stdlib/neplg2/core/resolve/type_resolver/exported_enum_origin.nepl");
const consumer = implementation("stdlib/neplg2/core/check/expr/exported_enum_member.nepl");
const composite = implementation("stdlib/neplg2/core/check/expr/match_actual_member.nepl");

for (const needle of ["selfhost_match_context_tokens", "selfhost_match_context_source", "selfhost_match_context_scrutinee_range"]) {
  if (!context.includes(needle)) throw new Error(`Match context observation missing: ${needle}`);
}
for (const needle of [
  "selfhost_expr_prefix_list_from_syntax_range tokens range",
  "not eq selfhost_expr_prefix_list_len &prefix 1",
  "selfhost_name_scope_find scope name",
  "selfhost_value_type_evidence_table_find value_types def_id",
  "selfhost_type_arena_get_record arena evidence.value_type",
]) {
  if (!scrutinee.includes(needle)) throw new Error(`actual scrutinee evidence chain missing: ${needle}`);
}
for (const needle of [
  "pub fn selfhost_exported_enum_origin_actual_context_result",
  "selfhost_type_arena_get_record &arena actual_type",
  "selfhost_resolved_enum_module_session_attach_existing_with_file_id_result constructors file.source file.file_id",
  "SelfhostTypeRecord::Named named:",
  "SelfhostTypeRecord::Applied applied:",
  "not selfhost_named_type_id_eq nominal definition.nominal_id",
  "SelfhostExportedEnumOriginContext arena session witness",
]) {
  if (!origin.includes(needle)) throw new Error(`actual origin validation missing: ${needle}`);
}
const surface = implementation("stdlib/neplg2/core/resolve/type_resolver/enum_surface.nepl");
for (const needle of [
  "pub fn selfhost_resolved_enum_module_session_attach_existing_with_file_id_result",
  "selfhost_type_constructor_table_find &constructors name",
  "not selfhost_type_constructor_kind_eq constructor.kind expected_kind",
  "not selfhost_resolved_enum_surface_span_eq constructor.span name_span",
  "SelfhostResolvedEnumSurfaceErrorKind::ExistingConstructorOriginMismatch",
  "constructor.nominal_id declaration_span name_span",
]) {
  if (!surface.includes(needle)) throw new Error(`existing constructor attach contract missing: ${needle}`);
}
if (!consumer.includes("selfhost_exported_enum_origin_actual_context_result graph vfs order target_node visible_name arena constructors actual_type")) {
  throw new Error("actual exported member path does not use the actual origin gate");
}
for (const needle of [
  "selfhost_match_context_from_vfs vfs path function_ordinal segment_ordinal",
  "selfhost_match_scrutinee_single_named_value_result selfhost_match_context_tokens &context",
  "selfhost_match_actual_arm_spans &context arm_ordinal",
  "spans.alias spans.enum_name spans.member",
  "selfhost_check_exported_enum_member_actual_context_result graph vfs order path",
]) {
  if (!composite.includes(needle)) throw new Error(`actual Match composite missing: ${needle}`);
}

const signature = composite.match(/pub fn selfhost_check_match_single_named_value_actual_member_result[^\n]*/)?.[0] ?? "";
for (const forbidden of ["SelfhostSourceSpan", "SelfhostSyntaxRange", "SelfhostTypeId", "SelfhostNamedTypeId"]) {
  if (signature.includes(forbidden)) throw new Error(`caller-supplied authority in public composite: ${forbidden}`);
}
for (const forbiddenName of ["source", "tokens", "range", "span", "actual_type", "nominal", "visible_name", "member_name"]) {
  if (new RegExp(`\\\\${forbiddenName}(?:\\\\|:)$`).test(signature)) {
    throw new Error(`caller-supplied semantic parameter in public composite: ${forbiddenName}`);
  }
}

console.log("selfhost actual Match member contract: pass");
