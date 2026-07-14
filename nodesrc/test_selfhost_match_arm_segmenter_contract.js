const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(path.join(root, "stdlib/neplg2/core/syntax/parser/match_arm_segmenter.nepl"), "utf8");
const implementation = source.replace(/^\s*\/\/.*$/gm, "");

for (const needle of [
  "items %(Vec SelfhostMatchVariantArm)",
  "segments %(Vec SelfhostMatchPatternSegment)",
  "colon_span %SelfhostSourceSpan",
  "TokenKind::KwMatch:",
  "TokenKind::Newline:",
  "TokenKind::Indent:",
  "selfhost_body_segment_list_from_envelope tokens match_intro.body",
  "selfhost_parse_match_variant_pattern tokens head.first_token",
  "selfhost_match_variant_pattern_next_token &pattern add head.first_token head.token_count",
  "selfhost_match_variant_pattern_free pattern",
  "selfhost_match_arm_tokens_fit_source tokens head.first_token head_end",
  "lt head.first_token 0",
  "le head.token_count 0",
  "gt head.token_count sub n head.first_token",
  "selfhost_match_arm_block_indent_index tokens add colon_index 2",
  "selfhost_match_arm_layout_is_valid tokens add head.first_token head.token_count segment.body",
  "v::free field::get list \"items\"",
  "v::free field::get list \"segments\"",
]) {
  if (!implementation.includes(needle)) throw new Error(`match arm segmenter contract missing: ${needle}`);
}

for (const needle of ["UnsupportedPattern", "TrailingPatternToken", "InvalidOuterLayout", "InvalidArmLayout", "ArmSegmentationFailed"]) {
  if (!implementation.includes(needle)) throw new Error(`match arm typed rejection missing: ${needle}`);
}

for (const needle of [
  "SelfhostMatchPatternParseErrorKind::SegmentAllocationFailed:",
  "SelfhostBodySegmentErrorKind::OutOfMemory:",
  "let count_ok eq arm.segment_count add index 1",
  "selfhost_match_variant_arm_direct_alias arms arm",
  "source_rejected_as \"match x:\\n    _:",
  "source_rejected_as \"match x:\\n    1:",
  "source_rejected_as \"match x:\\n    Enum value extra:",
  "source_rejected_as \"match x:\\n    Member:\\n    Next:",
]) {
  if (!source.includes(needle)) throw new Error(`match arm runtime/ownership evidence missing: ${needle}`);
}

for (const fixture of ["Member:", "Enum::Member:", "dep::Enum::Member value:", "root::dep::Enum::Member:"]) {
  if (!source.includes(fixture)) throw new Error(`match arm runtime matrix missing: ${fixture}`);
}

if (implementation.includes("Vec SelfhostMatchVariantPattern") || implementation.includes("Vec SelfhostMatchVariantArmPattern")) {
  throw new Error("move-only pattern owner was stored directly in a Vec");
}

console.log("selfhost match arm segmenter contract: pass");
