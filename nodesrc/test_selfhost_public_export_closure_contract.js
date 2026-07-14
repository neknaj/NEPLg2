"use strict";

const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const read = (rel) => fs.readFileSync(path.join(root, rel), "utf8").replace(/\r\n/g, "\n");
const order = read("stdlib/neplg2/core/resolve/name_resolver/module_order.nepl");
const table = read("stdlib/neplg2/core/resolve/name_resolver/public_export_table.nepl");
const facade = read("stdlib/neplg2/core/resolve/name_resolver.nepl");

function requireMatch(source, pattern, message) {
    if (!pattern.test(source)) throw new Error(message);
}

requireMatch(order, /pub fn selfhost_module_order_build[^\n]*&SelfhostModuleGraph[^\n]*&SelfhostVirtualFileSystem/, "module order must jointly validate graph and VFS");
requireMatch(order, /DuplicateNodePath[\s\S]*DuplicateNodeFile[\s\S]*SourceFileDuplicate[\s\S]*DanglingEdgeTo[\s\S]*Cycle[\s\S]*DuplicateExactEdge/, "module order must expose closed structural rejection kinds");
if (/impl Copy for SelfhostModuleOrder\b/.test(order)) throw new Error("module order owner must remain move-only");
requireMatch(table, /pub enum SelfhostPublicExportProvenance:\s*\n\s*Direct\s*\n\s*ReExport/, "export provenance must distinguish direct definitions from re-exports");
requireMatch(table, /visible_name %str[\s\S]*original_name %str[\s\S]*origin_module_node_index %i32[\s\S]*origin_module_file_id %i32[\s\S]*origin_def_id %SelfhostDefId/, "export entries must separate visible spelling from original declaration identity");
requireMatch(table, /reexport_directive_span %Option SelfhostSourceSpan[\s\S]*reexport_clause %Option SelfhostImportClauseKind[\s\S]*immediate_child_module_node_index %Option i32/, "re-export entries must retain immediate import provenance");
requireMatch(table, /SelfhostModuleDeclarationVisibility::Private:[\s\S]*selfhost_public_export_add_local_items/, "private declarations must not become direct exports");
requireMatch(table, /selfhost_import_record_is_public[\s\S]*selfhost_public_export_snapshot_child_loop[\s\S]*selfhost_public_export_append_snapshot_loop/, "only typed public import evidence may propagate an independently snapshotted child surface");
requireMatch(table, /DuplicateVisibleName/, "composed public export collisions must fail closed");
if (/selfhost_public_export_copy_child[^\n]*&Vec SelfhostPublicExportEntry[^\n]*Vec SelfhostPublicExportEntry/.test(table)) throw new Error("export composition must not borrow and move the same entry owner in one call");
if (/pub fn selfhost_public_export_push\b/.test(table)) throw new Error("callers must not inject export entries");
requireMatch(facade, /pub #import "\.\/name_resolver\/module_order" as \*[\s\S]*pub #import "\.\/name_resolver\/public_export_table" as \*/, "name resolver facade must expose validated order and export table APIs");

console.log("selfhost public export closure contract passed");
