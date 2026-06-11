#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

function lineColAt(text, index) {
    const prefix = text.slice(0, index);
    const lines = prefix.split("\n");
    return {
        line: lines.length - 1,
        col: lines[lines.length - 1].length,
    };
}

function findNth(text, needle, occurrence) {
    let index = -1;
    let cursor = 0;
    for (let count = 0; count <= occurrence; count += 1) {
        index = text.indexOf(needle, cursor);
        if (index === -1) {
            throw new Error(`missing fixture text: ${needle}`);
        }
        cursor = index + needle.length;
    }
    return index;
}

function span(text, needle, occurrence = 0) {
    const start = findNth(text, needle, occurrence);
    const end = start + needle.length;
    const startLoc = lineColAt(text, start);
    const endLoc = lineColAt(text, end);
    return {
        start,
        end,
        start_line: startLoc.line,
        start_col: startLoc.col,
        end_line: endLoc.line,
        end_col: endLoc.col,
    };
}

function token(text, kind, needle, occurrence = 0, value = undefined) {
    const item = {
        kind,
        span: span(text, needle, occurrence),
    };
    if (value !== undefined) {
        item.value = value;
    }
    return item;
}

function findPayloadToken(source, payload, needle, occurrence = 0) {
    const tokenSpan = span(source, needle, occurrence);
    const visibleTokens = [
        ...(payload.semanticHighlightTokens || []),
        ...(payload.tokens || []),
    ];
    return visibleTokens.find((item) => item.startIndex === tokenSpan.start && item.endIndex === tokenSpan.end);
}

function assertHighlighted(source, payload, needle, expectedType, occurrence = 0) {
    const hit = findPayloadToken(source, payload, needle, occurrence);
    assert.ok(hit, `missing editor token for ${needle}`);
    assert.equal(hit.type, expectedType, `${needle} token type`);
}

async function main() {
    const repo = path.resolve(__dirname, "..");
    const bridgePath = path.join(repo, "web", "dist_ts", "editor-core", "language-analysis.js");
    if (!fs.existsSync(bridgePath)) {
        throw new Error(`language analysis bridge not found: ${bridgePath}\nrun 'npm --prefix web run build:ts' first.`);
    }
    const bridge = await import(pathToFileURL(bridgePath).href);

    const source = [
        "//: current syntax",
        "#entry main",
        "#indent 4",
        "#target std",
        "@",
        "pub #import \"core/result\" as @merge",
        "",
        "pub enum Mode:",
        "    Idle",
        "    Running %i32",
        "",
        "pub struct App:",
        "    value %i32",
        "",
        "impl Copy for App:",
        "    fn copy_mark %fn &App App \\app:",
        "        app",
        "",
        "fn present %impure fn &App impure fn Result unit GuiError \\app\\result:",
        "    match result:",
        "        Result::Ok value:",
        "            true",
        "        Result::Err err:",
        "            false",
        "",
    ].join("\n");

    const analysis = {
        lex: {
            tokens: [
                token(source, "DocComment", "//: current syntax", 0, "current syntax"),
                token(source, "DirEntry", "#entry main", 0, "main"),
                token(source, "DirIndentWidth", "#indent 4", 0, "4"),
                token(source, "DirTarget", "#target std", 0, "std"),
                token(source, "At", "@", 0),
                token(source, "KwPub", "pub", 0),
                token(source, "DirImport", "#import \"core/result\" as @merge", 0, "\"core/result\" as @merge"),
                token(source, "KwPub", "pub", 1),
                token(source, "KwEnum", "enum", 0),
                token(source, "Ident", "Mode", 0, "Mode"),
                token(source, "Colon", ":", 0),
                token(source, "Ident", "Running", 0, "Running"),
                token(source, "Percent", "%", 0),
                token(source, "Ident", "i32", 0, "i32"),
                token(source, "KwPub", "pub", 2),
                token(source, "KwStruct", "struct", 0),
                token(source, "Ident", "App", 0, "App"),
                token(source, "KwImpl", "impl", 0),
                token(source, "Ident", "Copy", 0, "Copy"),
                token(source, "KwFor", "for", 0),
                token(source, "Ident", "App", 1, "App"),
                token(source, "KwFn", "fn", 0),
                token(source, "Ident", "copy_mark", 0, "copy_mark"),
                token(source, "Percent", "%", 2),
                token(source, "KwFn", "fn", 1),
                token(source, "Ampersand", "&", 0),
                token(source, "Ident", "App", 2, "App"),
                token(source, "Backslash", "\\", 0),
                token(source, "Ident", "app", 0, "app"),
                token(source, "KwFn", "fn", 2),
                token(source, "Ident", "present", 0, "present"),
                token(source, "Percent", "%", 3),
                token(source, "Ident", "impure", 0, "impure"),
                token(source, "KwFn", "fn", 3),
                token(source, "Ampersand", "&", 1),
                token(source, "Ident", "App", 4, "App"),
                token(source, "Ident", "impure", 1, "impure"),
                token(source, "KwFn", "fn", 4),
                token(source, "Ident", "Result", 0, "Result"),
                token(source, "UnitLiteral", "unit", 0),
                token(source, "Ident", "GuiError", 0, "GuiError"),
                token(source, "Backslash", "\\", 1),
                token(source, "Backslash", "\\", 2),
                token(source, "KwMatch", "match", 0),
                token(source, "Ident", "Result", 1, "Result"),
                token(source, "PathSep", "::", 0),
                token(source, "Ident", "Ok", 0, "Ok"),
                token(source, "BoolLiteral", "true", 0, "true"),
                token(source, "Ident", "Result", 2, "Result"),
                token(source, "PathSep", "::", 1),
                token(source, "Ident", "Err", 1, "Err"),
                token(source, "BoolLiteral", "false", 0, "false"),
            ],
            diagnostics: [],
        },
        semantics: {
            token_classifications: [
                {
                    token_index: 22,
                    category: "function",
                    role: "function_name",
                    span: span(source, "copy_mark", 0),
                    enclosing_span: span(source, "copy_mark", 0),
                },
                {
                    token_index: 28,
                    category: "variable",
                    role: "parameter_name",
                    span: span(source, "app", 0),
                    enclosing_span: span(source, "app", 0),
                },
                {
                    token_index: 30,
                    category: "function",
                    role: "function_name",
                    span: span(source, "present", 0),
                    enclosing_span: span(source, "present", 0),
                },
                {
                    token_index: 24,
                    category: "type-constructor",
                    role: "function_signature",
                    span: span(source, "fn", 1),
                    enclosing_span: span(source, "fn &App App", 0),
                },
                {
                    token_index: 32,
                    category: "type-constructor",
                    role: "function_signature",
                    span: span(source, "impure", 0),
                    enclosing_span: span(source, "impure fn &App impure fn Result unit GuiError", 0),
                },
                {
                    token_index: 33,
                    category: "type-constructor",
                    role: "function_signature",
                    span: span(source, "fn", 3),
                    enclosing_span: span(source, "impure fn &App impure fn Result unit GuiError", 0),
                },
                {
                    token_index: 36,
                    category: "type-constructor",
                    role: "function_type_result",
                    span: span(source, "impure", 1),
                    enclosing_span: span(source, "impure fn Result unit GuiError", 0),
                },
                {
                    token_index: 37,
                    category: "type-constructor",
                    role: "function_type_result",
                    span: span(source, "fn", 4),
                    enclosing_span: span(source, "impure fn Result unit GuiError", 0),
                },
                {
                    token_index: 38,
                    category: "type-constructor",
                    role: "type_constructor",
                    span: span(source, "Result", 0),
                    enclosing_span: span(source, "Result", 0),
                },
                {
                    token_index: 39,
                    category: "literal-unit",
                    role: "function_type_parameter",
                    span: span(source, "unit", 0),
                    enclosing_span: span(source, "Result unit GuiError", 0),
                },
                {
                    token_index: 44,
                    category: "namespace",
                    role: "path_namespace_name",
                    span: span(source, "Result", 1),
                    enclosing_span: span(source, "Result", 1),
                },
                {
                    token_index: 46,
                    category: "constant",
                    role: "path_member_name",
                    span: span(source, "Ok", 0),
                    enclosing_span: span(source, "Ok", 0),
                },
                {
                    token_index: 48,
                    category: "namespace",
                    role: "path_namespace_name",
                    span: span(source, "Result", 2),
                    enclosing_span: span(source, "Result", 2),
                },
                {
                    token_index: 50,
                    category: "constant",
                    role: "path_member_name",
                    span: span(source, "Err", 0),
                    enclosing_span: span(source, "Err", 0),
                },
            ],
        },
    };

    const payload = bridge.buildEditorUpdatePayloadFromAnalysis(source, analysis);

    assertHighlighted(source, payload, "//: current syntax", "comment");
    assertHighlighted(source, payload, "#entry", "keyword");
    assertHighlighted(source, payload, "@", "keyword");
    assertHighlighted(source, payload, "\"core/result\"", "literal-string");
    assertHighlighted(source, payload, "as", "keyword");
    assertHighlighted(source, payload, "@merge", "keyword");
    assertHighlighted(source, payload, "pub", "keyword");
    assertHighlighted(source, payload, "%", "operator");
    assertHighlighted(source, payload, "i32", "type");
    assertHighlighted(source, payload, "App", "type");
    assertHighlighted(source, payload, "\\", "operator");
    assertHighlighted(source, payload, "fn", "keyword", 0);
    assertHighlighted(source, payload, "fn", "type-constructor", 1);
    assertHighlighted(source, payload, "fn", "keyword", 2);
    assertHighlighted(source, payload, "fn", "type-constructor", 3);
    assertHighlighted(source, payload, "impure", "type-constructor");
    assertHighlighted(source, payload, "&", "operator");
    assertHighlighted(source, payload, "Result", "type-constructor");
    assertHighlighted(source, payload, "Result", "namespace", 1);
    assertHighlighted(source, payload, "unit", "literal-unit");
    assertHighlighted(source, payload, "::", "operator");
    assertHighlighted(source, payload, "Ok", "constant");
    assertHighlighted(source, payload, "true", "literal-bool");
    assertHighlighted(source, payload, "false", "literal-bool");

    const voidSource = [
        "fn main %fn void unit \\void:",
        "    unit",
        "",
    ].join("\n");
    const voidAnalysis = {
        lex: {
            tokens: [
                token(voidSource, "KwFn", "fn", 0),
                token(voidSource, "Ident", "main", 0, "main"),
                token(voidSource, "Percent", "%", 0),
                token(voidSource, "KwFn", "fn", 1),
                token(voidSource, "VoidMarker", "void", 0),
                token(voidSource, "UnitLiteral", "unit", 0),
                token(voidSource, "Backslash", "\\", 0),
                token(voidSource, "VoidMarker", "void", 1),
                token(voidSource, "UnitLiteral", "unit", 1),
            ],
            diagnostics: [],
        },
        semantics: {
            token_classifications: [
                {
                    token_index: 1,
                    category: "function",
                    role: "function_name",
                    span: span(voidSource, "main", 0),
                    enclosing_span: span(voidSource, "main", 0),
                },
                {
                    token_index: 3,
                    category: "type-constructor",
                    role: "function_signature",
                    span: span(voidSource, "fn", 1),
                    enclosing_span: span(voidSource, "fn void unit", 0),
                },
                {
                    token_index: 4,
                    category: "literal-void",
                    role: "zero_arg_void_marker",
                    span: span(voidSource, "void", 0),
                    enclosing_span: span(voidSource, "void", 0),
                },
                {
                    token_index: 5,
                    category: "literal-unit",
                    role: "function_type_result",
                    span: span(voidSource, "unit", 0),
                    enclosing_span: span(voidSource, "unit", 0),
                },
                {
                    token_index: 7,
                    category: "literal-void",
                    role: "zero_arg_void_marker",
                    span: span(voidSource, "void", 1),
                    enclosing_span: span(voidSource, "void", 1),
                },
                {
                    token_index: 8,
                    category: "literal-unit",
                    role: "unit_literal",
                    span: span(voidSource, "unit", 1),
                    enclosing_span: span(voidSource, "unit", 1),
                },
            ],
        },
    };
    const voidPayload = bridge.buildEditorUpdatePayloadFromAnalysis(voidSource, voidAnalysis);
    assertHighlighted(voidSource, voidPayload, "main", "function");
    assertHighlighted(voidSource, voidPayload, "fn", "keyword", 0);
    assertHighlighted(voidSource, voidPayload, "fn", "type-constructor", 1);
    assertHighlighted(voidSource, voidPayload, "void", "literal-void", 0);
    assertHighlighted(voidSource, voidPayload, "unit", "literal-unit", 0);
    assertHighlighted(voidSource, voidPayload, "void", "literal-void", 1);
    assertHighlighted(voidSource, voidPayload, "unit", "literal-unit", 1);

    const pathSource = [
        "group1::group2::name",
        "",
    ].join("\n");
    const pathAnalysis = {
        lex: {
            tokens: [
                token(pathSource, "Ident", "group1", 0, "group1"),
                token(pathSource, "PathSep", "::", 0),
                token(pathSource, "Ident", "group2", 0, "group2"),
                token(pathSource, "PathSep", "::", 1),
                token(pathSource, "Ident", "name", 0, "name"),
            ],
            diagnostics: [],
        },
        semantics: {
            token_classifications: [
                {
                    token_index: 0,
                    category: "namespace",
                    role: "path_namespace_name",
                    span: span(pathSource, "group1", 0),
                    enclosing_span: span(pathSource, "group1", 0),
                },
                {
                    token_index: 2,
                    category: "namespace",
                    role: "path_namespace_name",
                    span: span(pathSource, "group2", 0),
                    enclosing_span: span(pathSource, "group2", 0),
                },
                {
                    token_index: 4,
                    category: "constant",
                    role: "path_member_name",
                    span: span(pathSource, "name", 0),
                    enclosing_span: span(pathSource, "name", 0),
                },
            ],
        },
    };
    const pathPayload = bridge.buildEditorUpdatePayloadFromAnalysis(pathSource, pathAnalysis);
    assertHighlighted(pathSource, pathPayload, "group1", "namespace");
    assertHighlighted(pathSource, pathPayload, "group2", "namespace");
    assertHighlighted(pathSource, pathPayload, "name", "constant");

    const annotationSource = [
        "fn main %fn void unit \\void:",
        "    %widget_state value",
        "",
    ].join("\n");
    const annotationAnalysis = {
        lex: {
            tokens: [
                token(annotationSource, "Percent", "%", 1),
                token(annotationSource, "Ident", "widget_state", 0, "widget_state"),
            ],
            diagnostics: [],
        },
        semantics: {
            token_classifications: [
                {
                    token_index: 0,
                    category: "operator",
                    role: "prefix_type_annotation",
                    span: span(annotationSource, "%", 1),
                    enclosing_span: span(annotationSource, "%widget_state", 0),
                },
                {
                    token_index: 1,
                    category: "type",
                    role: "prefix_type_annotation_inner",
                    span: span(annotationSource, "widget_state", 0),
                    enclosing_span: span(annotationSource, "widget_state", 0),
                },
            ],
        },
    };
    const annotationPayload = bridge.buildEditorUpdatePayloadFromAnalysis(annotationSource, annotationAnalysis);
    assertHighlighted(annotationSource, annotationPayload, "%", "operator", 1);
    assertHighlighted(annotationSource, annotationPayload, "widget_state", "type");

    const authoritySource = "callable\n";
    const authorityAnalysis = {
        lex: {
            tokens: [
                token(authoritySource, "Ident", "callable", 0, "callable"),
            ],
            diagnostics: [],
        },
        resolve: {
            definitions: [
                {
                    id: 1,
                    name: "callable",
                    kind: "fn",
                    span: span(authoritySource, "callable", 0),
                },
            ],
        },
        semantics: {
            token_resolution: [
                {
                    token_index: 0,
                    resolved_def_id: 1,
                },
            ],
            token_classifications: [
                {
                    token_index: 0,
                    category: "constant",
                    role: "classification_authority",
                    span: span(authoritySource, "callable", 0),
                    enclosing_span: span(authoritySource, "callable", 0),
                },
            ],
        },
    };
    const authorityPayload = bridge.buildEditorUpdatePayloadFromAnalysis(authoritySource, authorityAnalysis);
    assertHighlighted(authoritySource, authorityPayload, "callable", "constant");

    const editorSource = fs.readFileSync(path.join(repo, "web", "src", "editor", "editor.ts"), "utf8");
    const expectedPalette = [
        ["keyword", "#f3a2a6"],
        ["type", "#c7ebcf"],
        ["type-constructor", "#84bf94"],
        ["constant", "#a5c2ff"],
        ["variable", "#9ee4ec"],
        ["function", "#f4df9a"],
        ["literal-string", "#f4bf8c"],
        ["literal-number", "#d7e99d"],
        ["literal-unit", "#f2add4"],
        ["literal-void", "#f2add4"],
        ["namespace", "#aeb8c7"],
    ];
    for (const [tokenType, color] of expectedPalette) {
        assert.match(
            editorSource,
            new RegExp(`['"]${tokenType}['"]:\\s*['"]${color}['"]`),
            `${tokenType} palette color`,
        );
    }

    console.log("editor current syntax highlighting regression passed");
}

main().catch((error) => {
    console.error(error && error.stack ? error.stack : String(error));
    process.exit(1);
});
