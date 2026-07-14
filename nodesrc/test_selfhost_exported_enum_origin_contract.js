const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const originPath = path.join(root, "stdlib/neplg2/core/resolve/type_resolver/exported_enum_origin.nepl");
const surfacePath = path.join(root, "stdlib/neplg2/core/resolve/type_resolver/enum_surface.nepl");
const facadePath = path.join(root, "stdlib/neplg2/core/resolve/type_resolver.nepl");
const origin = fs.readFileSync(originPath, "utf8");
const surface = fs.readFileSync(surfacePath, "utf8");
const facade = fs.readFileSync(facadePath, "utf8");

const requiredOrigin = [
  "pub struct SelfhostExportedEnumOriginContext:",
  "arena %SelfhostTypeArena",
  "enum_session %SelfhostResolvedEnumSession",
  "selfhost_module_order_validate_against graph vfs order",
  "selfhost_public_export_table_build graph vfs order",
  "selfhost_exported_enum_arena_contains_nominal_loop &arena 0",
  "selfhost_exported_enum_entry_count_loop",
  "selfhost_exported_enum_vfs_count_loop",
  "selfhost_exported_enum_span_in_source file.source file.file_id entry.original_name_span",
  "string_slice::str_slice file.source entry.original_name_span.start entry.original_name_span.end",
  "selfhost_exported_enum_definition_count_loop &session entry.original_name_span entry.original_name",
  "selfhost_resolved_enum_definition_binding_result &scope entry.origin_def_id &session definition.nominal_id",
  "selfhost_type_arena_add_named arena definition.nominal_id",
  "selfhost_type_arena_alloc_type_id &allocated",
  "selfhost_exported_enum_origin_context_free",
  "stable nominal key",
  "cross-session identity",
  "context値単体はcapabilityではなく",
];

for (const needle of requiredOrigin) {
  if (!origin.includes(needle)) throw new Error(`exported enum origin contract missing: ${needle}`);
}

for (const forbidden of [
  "fn SelfhostNamedTypeId impure fn SelfhostTypeId",
  "fn SelfhostPublicExportEntry impure fn SelfhostTypeArena impure fn SelfhostTypeConstructorTable Result",
  "pub fn selfhost_exported_enum_origin_context_result %impure fn &SelfhostPublicExportTable",
]) {
  if (origin.includes(forbidden)) throw new Error(`caller supplied identity authority reintroduced: ${forbidden}`);
}

if (!surface.includes("selfhost_resolved_enum_module_session_materialize_with_file_id_result")) {
  throw new Error("file-id-aware enum session materializer missing");
}
if (!surface.includes("lex_all_with_file_id source file_id")) {
  throw new Error("enum session materializer does not preserve VFS file identity");
}
if (!facade.includes('pub #import "./type_resolver/exported_enum_origin" as *')) {
  throw new Error("exported enum origin facade export missing");
}

console.log("selfhost exported enum origin contract: pass");
