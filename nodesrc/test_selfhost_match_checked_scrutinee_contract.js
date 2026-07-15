const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const readImplementation = (file) => fs.readFileSync(path.join(root, file), "utf8").replace(/^\s*\/\/.*$/gm, "");
const bodyLine = readImplementation("stdlib/neplg2/core/check/expr/body_line.nepl");
const scrutinee = readImplementation("stdlib/neplg2/core/check/expr/match_checked_scrutinee.nepl");
const composite = readImplementation("stdlib/neplg2/core/check/expr/match_actual_member.nepl");
const fixture = readImplementation("stdlib/neplg2/core/check/expr/stage1_match_actual_direct_fixture.nepl");

for (const needle of [
  "pub struct SelfhostExpressionLineCheckSuccessParts",
  "pub fn selfhost_expression_line_check_success_into_parts",
  "field::get success \"arena\"",
  "field::get success \"checked_tree\"",
  "field::get success \"root_expr\"",
]) {
  if (!bodyLine.includes(needle)) throw new Error(`expression success owner decomposition missing: ${needle}`);
}

for (const needle of [
  "selfhost_match_context_scrutinee_range context",
  "selfhost_body_segment_expression_line_from_range range",
  "selfhost_check_expr_reduce_body_segment_with_constructors_and_arena",
  "signatures candidates none",
  "candidates none",
  "selfhost_checked_expr_tree_get_node &checked_tree root_expr",
  "selfhost_type_arena_get_record &arena root_type",
  "selfhost_type_arena_fork &arena",
  "selfhost_match_checked_scrutinee_result context constructors first_arena",
  "selfhost_match_checked_scrutinee_result_with_expected context constructors arena",
  "some expected",
]) {
  if (!scrutinee.includes(needle)) throw new Error(`checked scrutinee authority chain missing: ${needle}`);
}

for (const needle of [
  "pub struct SelfhostMatchCheckedActualMemberContext",
  "selfhost_match_checked_scrutinee_result &context &constructors arena",
  "selfhost_check_exported_enum_member_actual_context_result",
  "constructors root_type",
  "SelfhostMatchCheckedActualMemberContext member reduction checked_arguments checked_tree root_expr root_type",
  "selfhost_checked_exported_enum_member_context_free",
  "selfhost_checked_expr_tree_free",
  "selfhost_match_actual_expected_nominal",
  "SelfhostMatchActualExpectedNominal constructor.nominal_id (selfhost_type_constructor_kind_arg_count constructor.kind) spans.enum_name",
  "selfhost_type_arena_add_fresh_inference_variable",
  "selfhost_type_arena_add_applied_named",
  "SelfhostTypeExpectationSource::MatchArmDerived",
  "selfhost_match_checked_scrutinee_retry_result",
]) {
  if (!composite.includes(needle)) throw new Error(`checked Match actual member owner wiring missing: ${needle}`);
}

const signature = composite.match(/pub fn selfhost_check_match_checked_actual_member_result[^\n]*/)?.[0] ?? "";
for (const forbidden of ["SelfhostSourceSpan", "SelfhostSyntaxRange", "SelfhostTypeId", "SelfhostNamedTypeId"]) {
  if (signature.includes(forbidden)) throw new Error(`caller-supplied authority in checked composite: ${forbidden}`);
}
for (const forbiddenName of ["source", "tokens", "range", "span", "actual_type", "nominal", "visible_name", "member_name", "expected"]) {
  if (new RegExp(`\\\\${forbiddenName}(?:\\\\|:|$)`).test(signature)) {
    throw new Error(`caller-supplied semantic parameter in checked composite: ${forbiddenName}`);
  }
}

for (const needle of [
  "selfhost_check_expr_stage1_fixture_match_actual_source",
  "match %Choice i32 make:",
  "match %Choice bool make:",
  "selfhost_check_expr_stage1_fixture_match_actual_ascription",
  "SelfhostCallReduceErrorKind::ExpectedTypeMismatch",
  "selfhost_match_checked_actual_member_context_root_type &context",
  "selfhost_match_checked_actual_member_context_member &context",
  "match 1 |> make:",
  "selfhost_check_expr_stage1_fixture_match_actual_pipe",
  "SelfhostCallReduceErrorKind::PipeTargetAmbiguous",
  "selfhost_name_scope_add_binding scope1 second_binding",
  "selfhost_callable_signature_table_add signatures1 second_signature",
  "selfhost_check_expr_stage1_fixture_match_actual_arm_retry",
  "SelfhostCallReduceErrorKind::OverloadAmbiguous",
  "selfhost_type_arena_fork &arena",
  "pub enum Other:",
  "pub enum Generic<.T>:",
  "generic_retry",
  "nondirect_first_stays_ambiguous",
]) {
  if (!fixture.includes(needle)) throw new Error(`actual Match runtime gate missing: ${needle}`);
}

console.log("selfhost Match checked scrutinee contract: pass");
