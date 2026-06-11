#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repo = path.resolve(__dirname, "..");

function read(relativePath) {
    return fs.readFileSync(path.join(repo, relativePath), "utf8");
}

const editor = read("web/src/editor/editor.ts");
const inputHandler = read("web/src/editor/editor-input-handler.ts");
const adapter = read("web/src/editor-core/browser-adapter.ts");
const panelManager = read("web/src/workspace/panel-manager.ts");

assert.match(
    editor,
    /onDefinitionNavigation\s*=\s*typeof\s+options\.onDefinitionNavigation\s*===\s*'function'/,
    "CanvasEditor must keep cross-file definition navigation as an injected command callback",
);

assert.match(
    adapter,
    /onDefinitionNavigation\?:\s*\(location:\s*unknown\)\s*=>\s*void/,
    "PlaygroundEditor adapter must expose the definition navigation callback in its typed options",
);
assert.match(
    adapter,
    /\{\s*\.\.\.editorOptions,\s*onCursorChange,\s*onDefinitionNavigation\s*\}/,
    "PlaygroundEditor adapter must forward definition navigation to CanvasEditor",
);
assert.match(
    adapter,
    /moveCursorToRange\(range:\s*PlaygroundTextRange\)/,
    "PlaygroundEditor adapter must expose cursor movement without requiring workspace to touch raw editor state",
);

assert.match(
    inputHandler,
    /location\.isCrossFile\s*\|\|\s*\(location\.targetPath\s*&&\s*location\.targetPath\s*!==\s*this\.editor\.languageProvider\.path\)/,
    "F12 must distinguish cross-file definitions from active-file cursor movement",
);
assert.match(
    inputHandler,
    /this\.editor\.onDefinitionNavigation\(location\)/,
    "F12 cross-file definitions must dispatch a workspace navigation request",
);
assert.doesNotMatch(
    inputHandler,
    /location\.isCrossFile[\s\S]{0,240}this\.editor\.setCursor\(location\.targetIndex\)/,
    "F12 must not move the active editor cursor with a cross-file targetIndex",
);

assert.match(
    panelManager,
    /import\s*\{[\s\S]*mapAnalysisSpanToTextRange[\s\S]*type\s+DefinitionLocation[\s\S]*\}\s*from\s+'..\/editor-core\/language-analysis\.js'/,
    "PanelManager must reuse the language-analysis span mapper instead of reimplementing byte offset conversion",
);
assert.match(
    panelManager,
    /onDefinitionNavigation:\s*\(location:\s*unknown\)\s*=>\s*\{\s*this\.openDefinitionTarget\(location\s+as\s+DefinitionLocation\);?\s*\}/,
    "Each editor runtime must route definition navigation through the workspace manager",
);
assert.match(
    panelManager,
    /openDefinitionTarget\(location:\s*DefinitionLocation\s*\|\s*null\s*\|\s*undefined\):\s*boolean[\s\S]*editorRuntime\.tabManager\.openFile\(targetPath\)[\s\S]*resolveDefinitionNavigationRange\(editorRuntime,\s*targetPath,\s*location\)[\s\S]*editorRuntime\.editor\.moveCursorToRange\(range\)/,
    "PanelManager must open the target tab before resolving and applying the target-file cursor range",
);
assert.match(
    panelManager,
    /canOpenDefinitionTargetPath\(path:\s*string\):\s*boolean[\s\S]*this\.vfs\.exists\(path\)\s*===\s*true/,
    "PanelManager must fail closed when the target path is not present in the VFS",
);

console.log("playground definition navigation contract regression passed");
