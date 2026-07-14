const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const consumer = fs.readFileSync(
  path.join(root, "stdlib/neplg2/core/check/expr/exported_enum_member.nepl"),
  "utf8",
);
const qualified = fs.readFileSync(
  path.join(root, "stdlib/neplg2/core/resolve/name_resolver/qualified_enum_import.nepl"),
  "utf8",
);
const facade = fs.readFileSync(path.join(root, "stdlib/neplg2/core/check/expr.nepl"), "utf8");
const implementation = consumer.replace(/^\s*\/\/.*$/gm, "");

for (const needle of [
  "pub struct SelfhostCheckedExportedEnumMemberContext:",
  "origin %SelfhostExportedEnumOriginContext",
  "checked %SelfhostCheckedEnumMember",
  "selfhost_module_order_validate_against graph vfs order",
  "selfhost_qualified_enum_import_table_build graph vfs source_path alias_span",
  "selfhost_qualified_enum_import_table_target_node_index &qualified",
  "string_slice::str_slice source enum_name_span.start enum_name_span.end",
  "selfhost_exported_enum_origin_context_result graph vfs order target_node visible_name arena constructors",
  "source_text_new source_file_id source_path source",
  "selfhost_check_enum_member_resolve_result selfhost_exported_enum_origin_context_arena &origin witness.scrutinee_type_id some witness.nominal_id",
  "selfhost_checked_exported_enum_member_context_free",
]) {
  if (!implementation.includes(needle)) throw new Error(`exported enum member consumer contract missing: ${needle}`);
}
if (!consumer.includes("actual Match scrutinee")) {
  throw new Error("nonproduction actual Match boundary documentation missing");
}

const publicSignature = implementation.match(/pub fn selfhost_check_exported_enum_member_context_result[^\n]*/)?.[0].trimEnd();
const expectedPublicSignature = "pub fn selfhost_check_exported_enum_member_context_result %impure fn &SelfhostModuleGraph impure fn &SelfhostVirtualFileSystem impure fn &SelfhostModuleOrder impure fn str impure fn SelfhostSourceSpan impure fn SelfhostSourceSpan impure fn SelfhostSourceSpan impure fn SelfhostTypeArena impure fn SelfhostTypeConstructorTable Result SelfhostCheckedExportedEnumMemberContext SelfhostCheckedExportedEnumMemberError \\graph\\vfs\\order\\source_path\\alias_span\\enum_name_span\\member_span\\arena\\constructors:";
if (publicSignature !== expectedPublicSignature) {
  throw new Error(`exported enum member public producer signature drifted: ${publicSignature}`);
}
for (const forbidden of [
  "SelfhostExportedEnumOriginContext",
  "SelfhostPublicExportTable",
  "SelfhostPublicExportEntry",
  "SelfhostQualifiedEnumImportTable",
  "SelfhostSourceText",
  "SelfhostNamedTypeId",
  "SelfhostTypeId",
]) {
  if (publicSignature.includes(forbidden)) {
    throw new Error(`caller supplied semantic authority reintroduced: ${forbidden}`);
  }
}
for (const forbiddenName of ["visible_name", "source", "query_text", "context", "entry", "table", "nominal", "type_id", "qualifier"]) {
  if (new RegExp(`\\\\${forbiddenName}(?:\\\\|:)$`).test(publicSignature)) {
    throw new Error(`caller supplied semantic parameter reintroduced: ${forbiddenName}`);
  }
}
if (/Result SelfhostCheckedEnumMember SelfhostCheckedExportedEnumMemberError/.test(publicSignature)) {
  throw new Error("public producer returned bare checked member without origin owner");
}

for (const needle of [
  "pub fn selfhost_qualified_enum_import_table_source",
  "pub fn selfhost_qualified_enum_import_table_source_file_id",
  "pub fn selfhost_qualified_enum_import_table_target_node_index",
]) {
  if (!qualified.includes(needle)) throw new Error(`qualified import observation missing: ${needle}`);
}
if (!facade.includes('pub #import "./expr/exported_enum_member" as *')) {
  throw new Error("exported enum member facade export missing");
}

console.log("selfhost exported enum member consumer contract: pass");
