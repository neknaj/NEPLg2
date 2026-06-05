#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const selfhostRoot = path.join(repoRoot, "stdlib", "neplg2");
const DOC_GAP_TRACKING_ISSUE = "issues/items/ISS-20260605T150033175Z-SELFHOST-COMPILER-DOC-COMMENTS-NEED--FF439E41.md";

const BASELINE = {
    moduleNoDoc: 77,
    moduleNoDoctest: 60,
    declarationNoDoc: 304,
    declarationNoDoctest: 1434,
    publicNoDoc: 51,
    publicNoDoctest: 1239,
    privateNoDoc: 253,
    privateNoDoctest: 195,
};
const HARD_DOC_BASELINE_KEYS = [
    "moduleNoDoc",
    "declarationNoDoc",
    "publicNoDoc",
    "privateNoDoc",
];
const REPORT_ONLY_DOCTEST_BASELINE_KEYS = [
    "moduleNoDoctest",
    "declarationNoDoctest",
    "publicNoDoctest",
    "privateNoDoctest",
];

const PUBLIC_DOC_REQUIRED_PREFIXES = [
    "stdlib/neplg2/cli/args/emit.nepl",
    "stdlib/neplg2/core/check/expr/argument.nepl",
    "stdlib/neplg2/core/check/expr/ascription.nepl",
    "stdlib/neplg2/core/check/expr/call_reduce.nepl",
    "stdlib/neplg2/core/check/module/",
    "stdlib/neplg2/core/hir/hir/expr.nepl",
    "stdlib/neplg2/core/syntax/lexer/",
];
const REQUIRED_SCANNER_SENTINELS = [
    "stdlib/neplg2/cli/args/emit.nepl",
    "stdlib/neplg2/core/check/module/summary.nepl",
    "stdlib/neplg2/core/check/module/declaration_adapter.nepl",
    "stdlib/neplg2/core/hir/hir/expr.nepl",
    "stdlib/neplg2/core/syntax/lexer/byte.nepl",
];
const DOC_SECTION_REQUIREMENTS = [
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_new", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_empty", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_all", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_add", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "SelfhostExprArgumentMatchErrorKind", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "SelfhostExprArgumentMatchError", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "SelfhostExprArgumentOwnedMatch", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_match", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_checked_argument", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_expected_type_is_function", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_function_value_error_from_candidate_collect", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_function_value_candidate_is_monomorphic", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_function_value_candidate", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_function_value_candidates", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_function_value_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_range_from_prefix", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_find_prefix_item_by_token_loop", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_find_prefix_item_by_token", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_validate_ascription_expected", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_span_from_ascription_error", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_ascribed_with_projection", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_ascribed_at_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_at_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "SelfhostExprAscriptionError", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "SelfhostExprAscriptionProjection", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "SelfhostExprAscriptionHeadProjection", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_expectation", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_tail", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_type_id", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_expectation", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_expression_first_token", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_expectation", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_head_expectation", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_expectation_with_constructors", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_error_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_existing_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_argument_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "SelfhostCallReduceArgumentCheckState", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_check_state_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_check_state_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_check_state_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_argument_state_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_push_checked_argument", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_error_from_candidate_collect", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_error_from_block_body_result", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_generic_state_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_expected_result", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_match_direct_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_consume_loop_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_nested_single_named_candidate_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_nested_named_candidates_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_match_at_with_source_or_nested", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_single_named_candidate", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_single_named_candidate_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_named_prefix", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_named_prefix_with_source_and_trailing_block", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_named_prefix_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_prefix_with_source_and_trailing_block", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_prefix_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_directive_fact", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_item_directive_state", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_item_span", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_item_declaration_header", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_index_unavailable_diag", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_refutation_diag", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_check_raw_backend_fact", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_check_item_raw_state", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_check_finish_raw_state", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "SelfhostModuleCheckSummary", ["purpose"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_item_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_doc_comment_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_directive_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_entry_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_target_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_import_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_declaration_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_function_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_type_declaration_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_impl_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_raw_block_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_raw_text_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary_update.nepl", "selfhost_module_check_summary_new", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary_update.nepl", "selfhost_module_check_summary_record", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "selfhost_check_module_ast", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirExprKind", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirFunctionValueIdentityBuildError", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirCallExpr", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirValueIdentity", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirExprPayload", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirExpr", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/byte.nepl", "lex_byte_or_end", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/byte.nepl", "lex_is_digit", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/diagnostic.nepl", "LexDiagnostic", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/directive.nepl", "SelfhostLexerDirectiveKind", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/syntax/lexer/directive.nepl", "lex_directive_word_at", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/indent.nepl", "lex_line_indent_width", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/literal.nepl", "lex_is_hex_digit", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/raw_mode.nepl", "SelfhostLexerRawMode", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/syntax/lexer/token_build.nepl", "lex_token_slice", ["purpose", "contract", "complexity"]),
];

const SECTION_PATTERNS = {
    purpose: /\[目的\/もくてき\]/,
    contract: /\[契約\/けいやく\]/,
    returns: /\[戻\/もど\]り\[値\/ち\]/,
    complexity: /\[計算量\/けいさんりょう\]/,
};

function requirement(relPath, name, sections) {
    return { relPath, name, sections };
}

function sectionRequirementKey(relPath, name) {
    return `${relPath}#${name}`;
}

function docHasSection(docLines, section) {
    const pattern = SECTION_PATTERNS[section];
    assert.ok(pattern, `unknown documentation section requirement: ${section}`);
    return docLines.some((line) => pattern.test(line));
}

const docSectionRequirementByKey = new Map(
    DOC_SECTION_REQUIREMENTS.map((item) => [sectionRequirementKey(item.relPath, item.name), item.sections]),
);

function walkNeplFiles(dir) {
    const files = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const child = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...walkNeplFiles(child));
        } else if (entry.isFile() && entry.name.endsWith(".nepl")) {
            files.push(child);
        }
    }
    return files;
}

function toRepoPath(filePath) {
    return path.relative(repoRoot, filePath).split(path.sep).join("/");
}

function hasDoctest(docLines) {
    return docLines.some((line) => /\bneplg2:test\b/.test(line));
}

function declarationAt(line) {
    return line.match(/^\s*(pub\s+)?(fn|struct|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)\b/);
}

function moduleDocLines(lines) {
    for (let index = 0; index < lines.length; index += 1) {
        const trimmed = lines[index].trim();
        if (trimmed === "" || trimmed === "#indent 4") {
            continue;
        }
        if (declarationAt(lines[index]) || trimmed.startsWith("#import")) {
            return [];
        }
        if (!lines[index].trimStart().startsWith("//:")) {
            return [];
        }
        const doc = [];
        for (let cursor = index; cursor < lines.length; cursor += 1) {
            if (!lines[cursor].trimStart().startsWith("//:")) {
                break;
            }
            doc.push(lines[cursor]);
        }
        if (doc.length > 0 && doc[0].trimStart().startsWith("//: #")) {
            return doc;
        }
        return [];
    }
    return [];
}

function precedingDocLines(lines, index) {
    let cursor = index - 1;
    while (cursor >= 0 && lines[cursor].trim() === "") {
        cursor -= 1;
    }
    const doc = [];
    while (cursor >= 0 && lines[cursor].trimStart().startsWith("//:")) {
        doc.push(lines[cursor]);
        cursor -= 1;
    }
    return doc.reverse();
}

function indentOf(line) {
    const match = line.match(/^(\s*)/);
    return match ? match[1].length : 0;
}

function implHeaderAt(line) {
    return line.match(/^\s*impl(?:\b|<)/);
}

const stats = {
    files: 0,
    moduleNoDoc: 0,
    moduleNoDoctest: 0,
    declarations: 0,
    declarationNoDoc: 0,
    declarationNoDoctest: 0,
    publicNoDoc: 0,
    publicNoDoctest: 0,
    privateNoDoc: 0,
    privateNoDoctest: 0,
};

const samples = [];
const publicDocRequiredPrefixGaps = [];
const moduleDocRequiredPrefixGaps = [];
const docSectionGaps = [];
const seenDocSectionRequirementKeys = new Set();
const seenRepoPaths = new Set();

function sample(message) {
    if (samples.length < 60) {
        samples.push(message);
    }
}

for (const filePath of walkNeplFiles(selfhostRoot).sort()) {
    stats.files += 1;
    const repoPath = toRepoPath(filePath);
    seenRepoPaths.add(repoPath);
    const text = fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
    const lines = text.split("\n");
    const moduleDoc = moduleDocLines(lines);
    if (moduleDoc.length === 0) {
        stats.moduleNoDoc += 1;
        sample(`${repoPath}: module doc is missing`);
        if (PUBLIC_DOC_REQUIRED_PREFIXES.some((prefix) => repoPath.startsWith(prefix))) {
            moduleDocRequiredPrefixGaps.push(`${repoPath}: module doc heading is missing`);
        }
    } else if (!hasDoctest(moduleDoc)) {
        stats.moduleNoDoctest += 1;
    }

    let implBlockIndent = null;
    for (let index = 0; index < lines.length; index += 1) {
        const line = lines[index];
        const trimmed = line.trim();
        const indentation = indentOf(line);
        const startsImpl = implHeaderAt(line);
        if (
            implBlockIndent !== null
            && trimmed !== ""
            && !trimmed.startsWith("//:")
            && indentation <= implBlockIndent
            && !startsImpl
        ) {
            implBlockIndent = null;
        }
        if (startsImpl) {
            implBlockIndent = indentation;
            continue;
        }
        if (implBlockIndent !== null) {
            continue;
        }
        const declaration = declarationAt(line);
        if (!declaration) {
            continue;
        }

        stats.declarations += 1;
        const isPublic = Boolean(declaration[1]);
        const doc = precedingDocLines(lines, index);
        if (doc.length === 0) {
            stats.declarationNoDoc += 1;
            if (isPublic) {
                stats.publicNoDoc += 1;
            } else {
                stats.privateNoDoc += 1;
            }
            const gap = `${repoPath}:${index + 1}: ${declaration[2]} ${declaration[3]} doc is missing`;
            sample(gap);
            if (isPublic && PUBLIC_DOC_REQUIRED_PREFIXES.some((prefix) => repoPath.startsWith(prefix))) {
                publicDocRequiredPrefixGaps.push(gap);
            }
        } else {
            const requirementKey = sectionRequirementKey(repoPath, declaration[3]);
            const sectionRequirements = docSectionRequirementByKey.get(requirementKey);
            if (sectionRequirements) {
                seenDocSectionRequirementKeys.add(requirementKey);
                for (const section of sectionRequirements) {
                    if (!docHasSection(doc, section)) {
                        docSectionGaps.push(`${repoPath}:${index + 1}: ${declaration[2]} ${declaration[3]} doc is missing [${section}] section`);
                    }
                }
            }
        }
        if (doc.length > 0 && !hasDoctest(doc)) {
            stats.declarationNoDoctest += 1;
            if (isPublic) {
                stats.publicNoDoctest += 1;
            } else {
                stats.privateNoDoctest += 1;
            }
        }
    }
}

for (const repoPath of REQUIRED_SCANNER_SENTINELS) {
    assert.ok(
        seenRepoPaths.has(repoPath),
        `${repoPath} must be included in the selfhost documentation scan`,
    );
}
assert(
    fs.existsSync(path.join(repoRoot, DOC_GAP_TRACKING_ISSUE)),
    `selfhost documentation baseline gaps must be tracked by ${DOC_GAP_TRACKING_ISSUE}`,
);
const docGapTrackingIssueText = fs.readFileSync(path.join(repoRoot, DOC_GAP_TRACKING_ISSUE), "utf8").replace(/\r\n/g, "\n");
assert.match(
    docGapTrackingIssueText,
    /^status:\s*open$/m,
    "selfhost documentation baseline issue must remain open while baseline gaps remain",
);
assert.match(
    docGapTrackingIssueText,
    /^resolved:\s*false$/m,
    "selfhost documentation baseline issue must remain unresolved while baseline gaps remain",
);
assert.ok(
    docGapTrackingIssueText.includes("not an accepted quality level"),
    "selfhost documentation baseline issue must state that the baseline is not an accepted quality level",
);
assert.ok(
    docGapTrackingIssueText.includes("fail-closed debt boundary"),
    "selfhost documentation baseline issue must state that the baseline is a fail-closed debt boundary",
);
for (const [key, value] of Object.entries(BASELINE)) {
    assert.ok(
        docGapTrackingIssueText.includes(`${key}=${value}`),
        `selfhost documentation baseline issue must record ${key}=${value}`,
    );
}
for (const key of HARD_DOC_BASELINE_KEYS) {
    assert(
        stats[key] <= BASELINE[key],
        `selfhost documentation gaps increased for ${key}: ${stats[key]} > ${BASELINE[key]}`,
    );
}
for (const key of REPORT_ONLY_DOCTEST_BASELINE_KEYS) {
    assert.ok(
        Object.hasOwn(BASELINE, key),
        `selfhost doctest debt counter must remain visible in the baseline issue: ${key}`,
    );
}
assert.deepEqual(
    moduleDocRequiredPrefixGaps,
    [],
    `selfhost fixed documentation slices must have explicit module doc headings:\n${moduleDocRequiredPrefixGaps.join("\n")}`,
);
assert.deepEqual(
    publicDocRequiredPrefixGaps,
    [],
    `selfhost fixed public documentation slices must not have public declaration doc gaps:\n${publicDocRequiredPrefixGaps.join("\n")}`,
);
const missingSectionRequirementTargets = [...docSectionRequirementByKey.keys()]
    .filter((key) => !seenDocSectionRequirementKeys.has(key));
assert.deepEqual(
    missingSectionRequirementTargets,
    [],
    `selfhost documentation section requirement targets must be found:\n${missingSectionRequirementTargets.join("\n")}`,
);
assert.deepEqual(
    docSectionGaps,
    [],
    `selfhost fixed documentation slices must preserve the required Zenn-policy doc sections:\n${docSectionGaps.join("\n")}`,
);

console.log("selfhost documentation contract baseline ok");
console.log(JSON.stringify(stats, null, 2));
if (samples.length > 0) {
    console.log("sample gaps:");
    for (const line of samples) {
        console.log(`- ${line}`);
    }
}
