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
const browserAdapter = read("web/src/editor-core/browser-adapter.ts");
const languageAnalysis = read("web/src/editor-core/language-analysis.ts");
const provider = read("web/src/language/neplg2/neplg2-provider.ts");
const analysisWorker = read("web/src/language/neplg2/neplg2-analysis-worker.ts");
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
    /_scheduleStructuralAnalysis\(version,\s*analysisText,\s*analysisDocumentVersion,\s*analysisPath\)/,
    "AST/folding parse must be scheduled after the semantic payload is published",
);
assert.match(
    provider,
    /new Worker\('dist_ts\/language\/neplg2\/neplg2-analysis-worker\.js',\s*\{\s*type:\s*'module'\s*\}\)/,
    "NEPLg2 semantic analysis must run through a dedicated module worker when compiler assets are available",
);
assert.match(
    provider,
    /_cancelActiveAnalysisWorkerRequests\('analysis input changed'\)/,
    "active analysis worker requests must be cancelled when editor input changes",
);
assert.match(
    provider,
    /_analyzeAndPublish\(version\)\s*\{[\s\S]*this\._canUseAnalysisWorker\(\)[\s\S]*this\._analyzeAndPublishWithWorker/,
    "semantic publish scheduling must choose the worker path before synchronous fallback",
);
assert.match(
    provider,
    /_requestStructuralParseWithWorker\(version,\s*text,\s*documentVersion,\s*path\)\s*\{[\s\S]*type:\s*'parse'[\s\S]*this\.parse\s*=\s*\{[\s\S]*module:\s*message\.module/,
    "structural parse must use the analysis worker and publish only returned parse modules",
);
assert.doesNotMatch(
    methodBody(provider, "getHoverInfo"),
    /_ensureStructuralParse\(/,
    "hover must not force a synchronous structural parse before the scheduled worker result",
);
assert.match(
    analysisWorker,
    /import\s+\{\s*buildEditorUpdatePayloadFromAnalysis\s*\}\s+from\s+'..\/..\/editor-core\/language-analysis\.js'/,
    "analysis worker must build editor payloads through the shared language-analysis bridge",
);
assert.match(
    analysisWorker,
    /analyze_semantics_with_vfs[\s\S]*analyze_semantics/,
    "analysis worker must preserve the VFS semantic analysis path before inline fallback",
);
assert.match(
    analysisWorker,
    /analyze_parse\(request\.text\)/,
    "analysis worker must own structural parse calls instead of running them on the UI thread",
);
assert.match(
    provider,
    /_analysisMetadata\(freshness,\s*options\s*=\s*\{\}\)[\s\S]*documentVersion:[\s\S]*sourceDocumentVersion[\s\S]*freshness,[\s\S]*isFresh:/,
    "analysis payloads must expose document identity and freshness metadata",
);
assert.match(
    languageAnalysis,
    /semanticHighlightTokens:\s*EditorToken\[\]/,
    "semantic highlight tokens must be separated from lexical tokens in the payload contract",
);
assert.match(
    languageAnalysis,
    /tokens:\s*buildEditorTokens\(prepared\),[\s\S]*semanticHighlightTokens:\s*buildSemanticHighlightTokens\(prepared\),/,
    "language analysis must publish lexical tokens and semantic highlight overlays separately",
);
assert.match(
    provider,
    /_hasFreshAnalysis\(\)[\s\S]*metadata\?\.isFresh\s*===\s*true[\s\S]*metadata\.documentVersion\s*===\s*this\.documentVersion[\s\S]*metadata\.path\s*===\s*this\.path/,
    "semantic-derived UI must require fresh analysis for the active document",
);
assert.match(
    provider,
    /_stripSemanticDerivedPayload\(payload\)[\s\S]*semanticHighlightTokens:\s*\[\][\s\S]*diagnostics:\s*\[\][\s\S]*foldingRanges:\s*\[\][\s\S]*semanticTokens:\s*\[\][\s\S]*inlayHints:\s*\[\]/,
    "provisional analysis payloads must not carry stale semantic-derived editor data",
);
assert.doesNotMatch(
    methodBody(provider, "_analyzeAndPublish"),
    /wasm\.analyze_parse\(this\.text\)/,
    "semantic publish path must not run an extra parse before returning editor tokens",
);
assert.match(
    methodBody(provider, "getTokenInsight"),
    /if\s*\(!this\._hasFreshAnalysis\(\)\)\s*\{\s*return null;/,
    "token insight must fail closed while semantic analysis is stale",
);
assert.match(
    methodBody(provider, "getHoverInfo"),
    /if\s*\(!this\._hasFreshAnalysis\(\)\)\s*\{\s*return null;/,
    "hover must fail closed while semantic analysis is stale",
);
assert.match(
    methodBody(provider, "getOccurrences"),
    /if\s*\(!this\._hasFreshAnalysis\(\)\)\s*\{\s*return \[\];/,
    "occurrences must fail closed while semantic analysis is stale",
);
assert.match(
    methodBody(editor, "registerLanguageProvider"),
    /canUseSemanticDerivedPayload[\s\S]*data\.analysis\.isFresh\s*===\s*true[\s\S]*semanticHighlightTokens\s*=\s*canUseSemanticDerivedPayload[\s\S]*this\.diagnostics\s*=\s*canUseSemanticDerivedPayload[\s\S]*this\.foldingRanges\s*=\s*canUseSemanticDerivedPayload/,
    "editor semantic highlighting diagnostics and folding ranges must reject stale analysis payloads",
);
assert.match(
    methodBody(browserAdapter, "getProblems"),
    /payload\?\.analysis[\s\S]*payload\.analysis\.isFresh\s*!==\s*true[\s\S]*return \[\];/,
    "problems API must reject stale analysis payloads",
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
assert.match(
    main,
    /NEPLg2CompilerAssets\s*=\s*readCompilerAssetsFromDocument\(document\)[\s\S]*new PlaygroundPanelManager/,
    "compiler asset metadata must be available before restored editor tabs create NEPLg2 analysis providers",
);

console.log("playground editor performance policy passed");
