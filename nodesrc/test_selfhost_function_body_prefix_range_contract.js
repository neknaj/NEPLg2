#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { readModuleParserSource } = require("./selfhost_module_parser_sources");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function stripDocComments(src) {
    return src
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//:"))
        .join("\n");
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

const ast = read("stdlib/neplg2/core/syntax/ast/module_ast.nepl");
const bodyRange = read("stdlib/neplg2/core/syntax/parser/module_parser/body_range.nepl");
const declaration = read("stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl");
const parser = readModuleParserSource(repoRoot);

assert.match(
    ast,
    /pub struct SelfhostModuleDeclarationBody:[\s\S]*\benvelope %SelfhostSyntaxRange[\s\S]*\bfirst_expression %SelfhostSyntaxRange/,
    "module AST must keep declaration body envelope and first expression as typed syntax ranges",
);
assert.match(
    ast,
    /pub struct SelfhostModuleItem:[\s\S]*\bdeclaration %Option SelfhostModuleDeclarationHeader[\s\S]*\bdeclaration_body %Option SelfhostModuleDeclarationBody/,
    "module item must keep declaration body evidence separately from declaration header evidence",
);
assert.match(
    ast,
    /pub fn selfhost_module_item_new_with_declaration_and_body\b[\s\S]*SelfhostModuleItem kind declaration\.header_span lexeme some declaration some body/,
    "parser-facing declaration constructor must attach header and body evidence together",
);

const bodyRangeCode = stripDocComments(bodyRange);
assert.doesNotMatch(
    bodyRangeCode,
    /\b(?:SelfhostHir|SelfhostHirExprPayload|TypeId|DefId|CallReduce|selfhost_hir_expr_call)\b/,
    "body range parser must not build HIR, allocate semantic IDs, or reduce calls",
);
assert.match(
    bodyRange,
    /pub fn selfhost_parser_declaration_body_range %fn &Vec SelfhostToken fn i32 fn i32 SelfhostModuleDeclarationBody/,
    "body range module must expose a declaration body range boundary",
);
assert.match(
    functionBlock(bodyRange, "selfhost_parser_declaration_body_range"),
    /selfhost_parser_syntax_range_from_indices\s+tokens\s+n\s+first_token\s+after_token|selfhost_parser_body_range_or_empty\s+tokens\s+n\s+first_expr\s+expr_end/,
    "body range boundary must produce SelfhostSyntaxRange from token indices instead of raw sentinel pairs",
);
assert.match(
    functionBlock(bodyRange, "selfhost_parser_body_envelope_end_loop"),
    /TokenKind::Indent:[\s\S]*add depth 1[\s\S]*TokenKind::Dedent:[\s\S]*sub depth 1/,
    "body envelope scan must track nested offside depth instead of stopping at the first dedent",
);
assert.match(
    functionBlock(bodyRange, "selfhost_parser_body_first_expression_end_loop"),
    /TokenKind::Newline:[\s\S]*idx[\s\S]*TokenKind::Dedent:[\s\S]*idx/,
    "first expression range must stop before newline or dedent so it can feed SelfhostExprPrefixList",
);
assert.match(
    declaration,
    /#import "\.\/body_range" as \*/,
    "declaration parser must depend on the body range helper through the split module boundary",
);
assert.match(
    functionBlock(declaration, "selfhost_parser_declaration_item"),
    /let body %SelfhostModuleDeclarationBody selfhost_parser_declaration_body_range tokens n idx[\s\S]*selfhost_module_item_new_with_declaration_and_body item_kind lexeme header body/,
    "declaration parser must attach body evidence when it creates a declaration item",
);
assert.match(
    parser,
    /pub #import "\.\/module_parser\/body_range" as \*/,
    "module parser facade must re-export the body range helper",
);

console.log("selfhost function body prefix range contract passed");
