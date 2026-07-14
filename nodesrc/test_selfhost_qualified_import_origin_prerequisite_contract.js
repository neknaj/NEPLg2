const assert = require("assert");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const spec = fs.readFileSync(path.join(root, "stdlib/neplg2/core/module/import_spec.nepl"), "utf8").replace(/\r\n/g, "\n");
const scan = fs.readFileSync(path.join(root, "stdlib/neplg2/core/module/import_scan.nepl"), "utf8").replace(/\r\n/g, "\n");
const graph = fs.readFileSync(path.join(root, "stdlib/neplg2/core/module/graph.nepl"), "utf8").replace(/\r\n/g, "\n");

assert.match(spec, /selfhost_import_spec_alias_span[\s\S]*Option SelfhostSourceSpan[\s\S]*source_span_is_valid spec\.span[\s\S]*gt spec\.alias_end directive_len[\s\S]*gt spec\.alias_end sub 2147483647 spec\.span\.start[\s\S]*let invalid %bool[\s\S]*some source_span_new_unchecked/);
assert.match(scan, /pub struct SelfhostImportRecord:[\s\S]*alias %str[\s\S]*alias_span %SelfhostSourceSpan[\s\S]*is_wildcard %bool/);
assert.match(scan, /match selfhost_import_spec_alias_span spec:[\s\S]*Option::Some alias_span:[\s\S]*selfhost_import_record_new[\s\S]*alias alias_span[\s\S]*Option::None:[\s\S]*Result::Err selfhost_import_diag/);
assert.match(graph, /pub enum SelfhostModuleGraphImportEdgeLookup:[\s\S]*Missing[\s\S]*Duplicate[\s\S]*Invariant[\s\S]*Found %SelfhostModuleGraphEdge/);
assert.match(graph, /match selfhost_module_graph_edge_at graph idx:[\s\S]*Option::None:[\s\S]*SelfhostModuleGraphImportEdgeLookup::Invariant/);
assert.match(graph, /selfhost_module_graph_import_edge_exact_loop[\s\S]*eq count 0[\s\S]*::Missing[\s\S]*eq count 1[\s\S]*::Found edge[\s\S]*::Duplicate/);
assert.match(graph, /let same_span %bool[\s\S]*eq edge\.span\.file_id span\.file_id[\s\S]*eq edge\.span\.start span\.start[\s\S]*eq edge\.span\.end span\.end/);
assert.match(graph, /let matches %bool[\s\S]*string_search::str_eq edge\.from from[\s\S]*string_search::str_eq edge\.to to[\s\S]*same_span/);
assert.match(graph, /selfhost_module_graph_import_edge_exact %fn &SelfhostModuleGraph fn str fn str fn &SelfhostImportRecord[\s\S]*selfhost_import_record_span record/);
assert.match(graph, /let found %bool match selfhost_module_graph_import_edge_exact[\s\S]*let missing %bool match selfhost_module_graph_import_edge_exact[\s\S]*let duplicate %bool match selfhost_module_graph_import_edge_exact/);

console.log("selfhost qualified import origin prerequisite contract passed");
