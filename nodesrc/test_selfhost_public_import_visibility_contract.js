const assert = require("assert");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const scan = fs.readFileSync(path.join(root, "stdlib/neplg2/core/module/import_scan.nepl"), "utf8").replace(/\r\n/g, "\n");

assert.match(scan, /pub enum SelfhostImportVisibility:[\s\S]*Private[\s\S]*Public/);
assert.match(scan, /pub enum SelfhostImportClauseKind:[\s\S]*Alias[\s\S]*Open[\s\S]*Merge/);
assert.match(scan, /pub struct SelfhostImportRecord:[\s\S]*span %SelfhostSourceSpan[\s\S]*import_span %SelfhostSourceSpan[\s\S]*visibility %SelfhostImportVisibility[\s\S]*visibility_span %Option SelfhostSourceSpan/);
assert.match(scan, /selfhost_import_scan_public_directive_start[\s\S]*string_search::str_starts_with_at source line_start "pub"[\s\S]*selfhost_import_scan_skip_inline_space[\s\S]*gt directive_start after_pub[\s\S]*string_search::str_starts_with_at source directive_start "#import"/);
assert.match(scan, /source_span_new_unchecked file_id line_start line_end[\s\S]*source_span_new_unchecked file_id directive_start line_end/);
assert.match(scan, /selfhost_import_record_is_public/);
assert.match(scan, /selfhost_import_record_visibility_span/);
assert.match(scan, /selfhost_import_record_clause/);
assert.doesNotMatch(scan, /str_starts_with_at source line_start "pub #import"/);

console.log("selfhost public import visibility contract passed");
