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
    return payload.tokens.find((item) => item.startIndex === tokenSpan.start && item.endIndex === tokenSpan.end);
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
                token(source, "Ident", "unit", 0, "unit"),
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
                token(source, "Ident", "Err", 0, "Err"),
                token(source, "BoolLiteral", "false", 0, "false"),
            ],
            diagnostics: [],
        },
    };

    const payload = bridge.buildEditorUpdatePayloadFromAnalysis(source, analysis);

    assertHighlighted(source, payload, "//: current syntax", "comment");
    assertHighlighted(source, payload, "#entry", "keyword");
    assertHighlighted(source, payload, "@", "keyword");
    assertHighlighted(source, payload, "\"core/result\"", "string");
    assertHighlighted(source, payload, "as", "keyword");
    assertHighlighted(source, payload, "@merge", "keyword");
    assertHighlighted(source, payload, "pub", "keyword");
    assertHighlighted(source, payload, "%", "operator");
    assertHighlighted(source, payload, "i32", "type");
    assertHighlighted(source, payload, "App", "type");
    assertHighlighted(source, payload, "\\", "operator");
    assertHighlighted(source, payload, "impure", "keyword");
    assertHighlighted(source, payload, "&", "operator");
    assertHighlighted(source, payload, "Result", "type");
    assertHighlighted(source, payload, "unit", "type");
    assertHighlighted(source, payload, "::", "operator");
    assertHighlighted(source, payload, "Ok", "type");
    assertHighlighted(source, payload, "true", "boolean");
    assertHighlighted(source, payload, "false", "boolean");

    console.log("editor current syntax highlighting regression passed");
}

main().catch((error) => {
    console.error(error && error.stack ? error.stack : String(error));
    process.exit(1);
});
