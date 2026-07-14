const assert = require("assert");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(path.join(root, "stdlib/neplg2/core/resolve/name_resolver/qualified_enum_import.nepl"), "utf8").replace(/\r\n/g, "\n");
const facade = fs.readFileSync(path.join(root, "stdlib/neplg2/core/resolve/name_resolver.nepl"), "utf8").replace(/\r\n/g, "\n");

assert.match(facade, /pub #import "\.\/name_resolver\/qualified_enum_import" as \*/);
assert.match(source, /pub struct SelfhostQualifiedEnumImportTable:[\s\S]*target %SelfhostQualifiedEnumAliasTarget[\s\S]*target_ast %SelfhostModuleAst[\s\S]*target_scope %SelfhostNameScope/);
assert.match(source, /pub struct SelfhostQualifiedEnumDefinitionOrigin:[\s\S]*module_node_index %i32[\s\S]*module_file_id %i32[\s\S]*definition_id %SelfhostDefId[\s\S]*name_span %SelfhostSourceSpan/);
assert.match(source, /selfhost_scan_module_imports_with_file_id/);
assert.match(source, /selfhost_qualified_edge_count[\s\S]*string_search::str_eq edge\.from from[\s\S]*string_search::str_eq edge\.to to[\s\S]*selfhost_qualified_span_eq edge\.span directive_span/);
assert.match(source, /selfhost_qualified_graph_node_count/);
assert.match(source, /selfhost_qualified_vfs_file_count/);
assert.match(source, /record\.is_wildcard[\s\S]*AliasWildcard[\s\S]*selfhost_qualified_alias_name_count[\s\S]*AliasDuplicate/);
assert.match(source, /selfhost_parse_module_source_with_file_id/);
assert.match(source, /selfhost_name_scope_hoist_module_declarations/);
assert.match(source, /SelfhostModuleDeclarationVisibility::Public:[\s\S]*SelfhostModuleDeclarationKind::Enum:[\s\S]*selfhost_qualified_binding_matches/);
assert.match(source, /selfhost_qualified_utf8_boundary[\s\S]*string_utf8::string_utf8_is_continuation/);
assert.match(source, /selfhost_qualified_binding_matches[\s\S]*SelfhostQualifiedEnumImportErrorKind::BindingMismatch/);
assert.match(source, /selfhost_qualified_enum_definition_origin_name_span/);
assert.match(source, /SelfhostQualifiedEnumImportErrorKind::MemberSpanInvalid/);
assert.match(source, /SelfhostQualifiedEnumImportErrorKind::ImportScanFailed/);
assert.match(source, /SelfhostQualifiedEnumImportErrorKind::SourceFileIdMismatch/);
assert.match(source, /selfhost_qualified_edge_directive_count[\s\S]*SelfhostQualifiedEnumImportErrorKind::ResolvedTargetUnsupported/);
assert.match(source, /selfhost_qualified_enum_import_table_free[\s\S]*selfhost_module_ast_free[\s\S]*selfhost_name_scope_free/);
assert.doesNotMatch(source, /pub fn [^(\n]*new[^\n]*SelfhostQualifiedEnumDefinitionOrigin/);

console.log("selfhost qualified enum import contract passed");
