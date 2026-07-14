const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const ast = fs.readFileSync(path.join(root, "stdlib/neplg2/core/syntax/ast/match_pattern.nepl"), "utf8");
const parser = fs.readFileSync(path.join(root, "stdlib/neplg2/core/syntax/parser/match_pattern.nepl"), "utf8");
const astImplementation = ast.replace(/^\s*\/\/.*$/gm, "");
const implementation = parser.replace(/^\s*\/\/.*$/gm, "");

for (const needle of [
  "segments %(Vec SelfhostMatchPatternSegment)",
  "bind %(Option SelfhostMatchPatternBind)",
  "next_token %i32",
  "v::free field::get pattern \"segments\"",
  "not eq selfhost_match_variant_pattern_len pattern 3",
]) {
  if (!astImplementation.includes(needle)) throw new Error(`match pattern AST contract missing: ${needle}`);
}

for (const needle of [
  "TokenKind::PathSep:",
  "TokenKind::Ident:",
  "selfhost_parse_match_variant_pattern_tail tokens add ident_index 1 next_segments next_span",
  "ExpectedIdentifierAfterPathSeparator",
  "selfhost_match_pattern_push_segment",
]) {
  if (!implementation.includes(needle)) throw new Error(`match pattern parser contract missing: ${needle}`);
}

for (const fixture of [
  'path_ok "Member" 1 false',
  'path_ok "Enum::Member" 2 false',
  'path_ok "dep::Enum::Member" 3 true',
  'path_ok "root::dep::Enum::Member" 4 false',
  'missing_rejected',
]) {
  if (!parser.includes(fixture)) throw new Error(`match pattern runtime matrix missing: ${fixture}`);
}

if (ast.includes("qualified_name %str") || ast.includes("member_name %str")) {
  throw new Error("raw qualified spelling was stored in pattern evidence");
}
if (!implementation.includes("selfhost_match_variant_pattern_new segments bind path_span add cursor 1")) {
  throw new Error("optional bind incorrectly widened the Rust-compatible variant name span");
}

console.log("selfhost match pattern evidence contract: pass");
