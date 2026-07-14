const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const raw = (file) => fs.readFileSync(path.join(root, file), "utf8");
const implementation = (file) => raw(file).replace(/^\s*\/\/.*$/gm, "");
const vfs = implementation("stdlib/neplg2/core/module/vfs.nepl");
const loader = implementation("stdlib/neplg2/core/module/loader.nepl");
const context = implementation("stdlib/neplg2/core/syntax/parser/match_context.nepl");

for (const needle of ["SelfhostVfsExactLookupErrorKind::Missing", "SelfhostVfsExactLookupErrorKind::Duplicate", "SelfhostVfsExactLookupErrorKind::FileIdOrdinalMismatch", "not eq file.file_id idx", "selfhost_vfs_find_exact_loop vfs path selfhost_vfs_len vfs 0 none"]) {
  if (!vfs.includes(needle)) throw new Error(`exact VFS prerequisite missing: ${needle}`);
}

for (const needle of ["tokens %(Vec SelfhostToken)", "ast %SelfhostModuleAst", "selfhost_vfs_find_exact vfs path", "lex_all_with_file_id file.source file.file_id", "selfhost_parse_module_tokens file.source &tokens", "SelfhostLexerDiagnosticCode::OutOfMemory: SelfhostLoadedModuleTokensErrorKind::OutOfMemory", "SelfhostParserDiagnosticCode::OutOfMemory: SelfhostLoadedModuleTokensErrorKind::OutOfMemory", "v::free tokens", "selfhost_module_ast_free field::get module \"ast\""]) {
  if (!loader.includes(needle)) throw new Error(`current-source loaded owner missing: ${needle}`);
}

for (const needle of [
  "pub fn selfhost_match_context_from_vfs %impure fn &SelfhostVirtualFileSystem impure fn str impure fn i32 impure fn i32",
  "SelfhostModuleItemKind::FunctionDecl:",
  "selfhost_body_segment_list_from_envelope tokens body.envelope",
  "TokenKind::KwMatch: true",
  "selfhost_match_context_scrutinee tokens intro",
  "selfhost_match_variant_arm_list_from_intro tokens source intro",
  "loaded %SelfhostLoadedModuleTokens",
  "scrutinee %SelfhostSyntaxRange",
  "arms %SelfhostMatchVariantArmList",
  "selfhost_match_variant_arm_list_free field::get context \"arms\"",
  "selfhost_loaded_module_tokens_free field::get context \"loaded\"",
  "SelfhostBodySegmentErrorKind::OutOfMemory: Result::Err SelfhostMatchContextErrorKind::OutOfMemory",
  "SelfhostMatchArmSegmentErrorKind::OutOfMemory: Result::Err SelfhostMatchContextErrorKind::OutOfMemory",
]) {
  if (!context.includes(needle)) throw new Error(`current-VFS Match context missing: ${needle}`);
}

for (const forbidden of ["pub fn selfhost_match_context_from_vfs %impure fn str", "pub fn selfhost_match_context_from_vfs %impure fn &Vec SelfhostToken", "lex_all source", "selfhost_vfs_find vfs path", "SelfhostTypeId", "SelfhostResolvedEnumMemberId"]) {
  if (context.includes(forbidden)) throw new Error(`Match context crossed authority boundary: ${forbidden}`);
}

const source = raw("stdlib/neplg2/core/syntax/parser/match_context.nepl");
for (const evidence of ["selfhost_match_context_file_id &context 1", "selfhost_match_context_scrutinee_is_nonempty &context", "selfhost_match_context_spans_use_current_file &context", "selfhost_match_context_arm_len &context 2", "SelfhostMatchContextErrorKind::SourceMissing", "SelfhostMatchContextErrorKind::SourceDuplicate", "SelfhostMatchContextErrorKind::FileIdOrdinalMismatch", "SelfhostMatchContextErrorKind::SelectedSegmentNotMatch"]) {
  if (!source.includes(evidence)) throw new Error(`Match context matrix evidence missing: ${evidence}`);
}

console.log("selfhost current-VFS Match context contract: pass");
