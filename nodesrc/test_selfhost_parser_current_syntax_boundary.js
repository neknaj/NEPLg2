#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { readModuleParserSource } = require("./selfhost_module_parser_sources");
const { readTokenSource } = require("./selfhost_token_sources");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function functionBlock(src, name) {
    const lines = src.split("\n");
    const declaration = new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s+`);
    const start = lines.findIndex((line) => declaration.test(line));
    assert.notEqual(start, -1, `${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

const diag = read("stdlib/neplg2/core/infra/diag/code.nepl");
const ast = read("stdlib/neplg2/core/syntax/ast/module_ast.nepl");
const parser = readModuleParserSource(repoRoot);
const token = readTokenSource(repoRoot);
const parserFixture = read("tests/stdlib/neplg2_parser.n.md");

assert.match(
    diag,
    /pub enum SelfhostParserDiagnosticCode:[\s\S]*LegacySyntaxToken/,
    "legacy current-syntax boundary must use a typed parser diagnostic code",
);
assert.match(
    functionBlock(diag, "selfhost_parser_diag_code_name"),
    /SelfhostParserDiagnosticCode::LegacySyntaxToken:[\s\S]*"parser\.syntax\.legacy_token"/,
    "legacy syntax diagnostic must have a stable reporter code",
);
assert.match(
    parser,
    /pub enum SelfhostParserTokenAction:[\s\S]*LegacySyntax[\s\S]*Regular/,
    "parser loop action enum must distinguish legacy syntax tokens from ordinary regular tokens",
);
assert.match(
    functionBlock(parser, "selfhost_parser_token_role"),
    /TokenKind::LParen:\s*\n\s*SelfhostParserTokenRole::LegacySyntaxToken[\s\S]*TokenKind::RParen:\s*\n\s*SelfhostParserTokenRole::LegacySyntaxToken[\s\S]*TokenKind::LAngle:\s*\n\s*SelfhostParserTokenRole::LegacySyntaxToken[\s\S]*TokenKind::RAngle:\s*\n\s*SelfhostParserTokenRole::LegacySyntaxToken/,
    "parentheses and angle brackets must be classified as legacy syntax by the current NEPLg2.1 parser boundary",
);
assert.match(
    functionBlock(parser, "selfhost_parser_token_role_action"),
    /SelfhostParserTokenRole::LegacySyntaxToken:\s*\n\s*SelfhostParserTokenAction::LegacySyntax/,
    "legacy syntax roles must project to the parser loop legacy diagnostic action",
);
assert.match(
    functionBlock(parser, "selfhost_parse_module_loop"),
    /SelfhostParserTokenAction::LegacySyntax:\s*\n\s*selfhost_parser_legacy_syntax_token_error\s+&token\s+ast/,
    "module parser loop must turn legacy syntax tokens into typed diagnostics",
);
assert.match(
    functionBlock(token, "selfhost_token_is_expr_start"),
    /TokenKind::LParen:\s*\n\s*false[\s\S]*TokenKind::Percent:\s*\n\s*true/,
    "current expression start classification must keep % annotations and reject parenthesized grouping",
);
assert.doesNotMatch(
    ast,
    /\bGenericParams\b/,
    "module declaration head evidence must not keep legacy angle-bracket generic parameters",
);
assert.match(
    ast,
    /pub enum SelfhostSyntaxRange:[\s\S]*Empty[\s\S]*Range %SelfhostSyntaxRangeItems/,
    "module AST must model parser-provided prefix/type ranges without raw sentinel pairs",
);
assert.match(
    ast,
    /pub struct SelfhostModuleDeclarationHeader:[\s\S]*type_annotation %SelfhostSyntaxRange[\s\S]*lambda_header %SelfhostSyntaxRange/,
    "declaration header evidence must keep % type annotation and lambda header ranges",
);
assert.match(
    parser,
    /pub enum SelfhostParserPrefixRangeTokenRole:[\s\S]*TypeAnnotationMarker[\s\S]*LambdaMarker[\s\S]*HeaderTerminator/,
    "module parser must classify tokens for prefix range extraction with a typed role enum",
);
assert.match(
    functionBlock(parser, "selfhost_parser_header_type_annotation_range"),
    /selfhost_parser_type_annotation_range_loop\s+tokens\s+n\s+add\s+decl_idx\s+1/,
    "module parser must expose a dedicated % type annotation range boundary",
);
assert.match(
    functionBlock(parser, "selfhost_parser_header_lambda_range"),
    /selfhost_parser_lambda_header_range_loop\s+tokens\s+n\s+add\s+decl_idx\s+1/,
    "module parser must expose a dedicated lambda header range boundary",
);
assert.match(
    functionBlock(parser, "selfhost_parser_token_role_declaration_head_kind"),
    /SelfhostParserTokenRole::LegacySyntaxToken:\s*\n\s*none/,
    "declaration head projection must not promote legacy angle syntax to typed evidence",
);
assert.match(
    parserFixture,
    /fn add %fn i32 fn i32 i32 \\\\a\\\\b:/,
    "parser fixture must exercise current % type annotation and backslash lambda syntax",
);
assert.doesNotMatch(
    parserFixture,
    /"FunctionDecl"\s+"fn add <\(i32,i32\)->i32> \(a,b\):"/,
    "parser fixture must not use old angle type annotations as the successful FunctionDecl example",
);

console.log("selfhost parser current syntax boundary contract passed");
