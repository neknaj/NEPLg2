#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8");
}

function methodBody(source, name) {
    const start = source.indexOf(`${name}(`);
    assert.notEqual(start, -1, `${name} must exist`);
    const brace = source.indexOf("{", start);
    assert.notEqual(brace, -1, `${name} must have a body`);
    let depth = 0;
    for (let index = brace; index < source.length; index += 1) {
        const ch = source[index];
        if (ch === "{") depth += 1;
        if (ch === "}") {
            depth -= 1;
            if (depth === 0) {
                return source.slice(brace + 1, index);
            }
        }
    }
    throw new Error(`${name} body is not closed`);
}

const editor = read("web/src/editor/editor.ts");
const inputHandler = read("web/src/editor/editor-input-handler.ts");
const provider = read("web/src/language/neplg2/neplg2-provider.ts");
const panelManager = read("web/src/workspace/panel-manager.ts");
const main = read("web/src/main.ts");
const html = read("web/index.html");

assert.match(
    editor,
    /scheduleCursorDerivedHighlights\(delay\s*=\s*this\.cursorDerivedHighlightDelayMs\)/,
    "Canvas editor must centralize cursor-derived highlight scheduling",
);
assert.match(
    editor,
    /this\.cursorDerivedHighlightTimer\s*=\s*setTimeout\(async\s*\(\)\s*=>/,
    "cursor-derived occurrences and bracket matching must be delayed off the key input path",
);
assert.doesNotMatch(
    methodBody(editor, "setCursor"),
    /this\.updateOccurrencesHighlight\(\)|this\.updateBracketMatching\(\)/,
    "setCursor must not synchronously compute occurrences and bracket matching",
);
assert.match(
    inputHandler,
    /this\.editor\.scheduleCursorDerivedHighlights\(\)/,
    "input handler must use the scheduled cursor-derived analysis hook",
);

assert.match(
    provider,
    /replaceDocument\(document\)[\s\S]*this\._replaceDocument\(nextPath,\s*nextText\)/,
    "document open must use the atomic path/text replacement contract",
);
assert.doesNotMatch(
    provider,
    /replaceDocumentText\(text\)[\s\S]*this\._scheduleAnalysis\(true\)/,
    "document open must not force immediate full semantic analysis",
);
assert.match(
    provider,
    /_scheduleStructuralAnalysis\(version,\s*this\.text\)/,
    "AST/folding parse must be scheduled after the semantic payload is published",
);
assert.doesNotMatch(
    methodBody(provider, "_analyzeAndPublish"),
    /wasm\.analyze_parse\(this\.text\)/,
    "semantic publish path must not run an extra parse before returning editor tokens",
);
assert.match(
    read("web/src/library/tabs.ts"),
    /replaceEditorDocument\(path:\s*string\s*\|\s*null,\s*content:\s*string,\s*isEditable:\s*boolean\)[\s\S]*this\.editor\.replaceDocument\(\{/,
    "tab activation must send path text and editable state through one editor replacement call",
);

assert.match(
    panelManager,
    /scheduleAnalysisInsight\(runtime:\s*EditorRuntime,\s*index:\s*number,\s*delayMs\s*=\s*55\)/,
    "status bar token insight must be debounced separately from cursor position updates",
);
assert.match(
    panelManager,
    /getCompilerMode:\s*\(\)\s*=>\s*string/,
    "panel manager must receive compiler mode as a provider so existing terminals observe mode changes",
);
assert.match(
    html,
    /id="compiler-mode-select"[\s\S]*value="rust" selected[\s\S]*value="selfhost"/,
    "playground header must expose a Rust default and selfhost experimental compiler selector",
);
assert.match(
    main,
    /let\s+compilerMode\s*=\s*normalizeCompilerMode\(compilerModeSelect\?\.value\)/,
    "main playground controller must keep Rust as the normalized default compiler mode",
);

console.log("playground editor performance policy passed");
