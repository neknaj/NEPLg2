#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

function utf8ByteIndex(text, index) {
    return Buffer.byteLength(text.slice(0, index), "utf8");
}

function byteSpan(text, needle, options = {}) {
    const occurrence = Number(options.occurrence || 0);
    let index = -1;
    let cursor = 0;
    for (let count = 0; count <= occurrence; count += 1) {
        index = text.indexOf(needle, cursor);
        if (index < 0) {
            throw new Error(`missing fixture text: ${needle}`);
        }
        cursor = index + needle.length;
    }
    return {
        file_path: options.filePath,
        start: utf8ByteIndex(text, index),
        end: utf8ByteIndex(text, index + needle.length),
    };
}

async function loadBridge() {
    const repo = path.resolve(__dirname, "..");
    const bridgePath = path.join(repo, "web", "dist_ts", "editor-core", "language-analysis.js");
    if (!fs.existsSync(bridgePath)) {
        throw new Error(`language analysis bridge not found: ${bridgePath}\nrun 'npm --prefix web run build:ts' first.`);
    }
    return import(pathToFileURL(bridgePath).href);
}

async function main() {
    const repo = path.resolve(__dirname, "..");
    const rustBridge = fs.readFileSync(path.join(repo, "nepl-web", "src", "lib.rs"), "utf8");
    assert.match(rustBridge, /fn\s+diagnostics_to_js_with_map\s*\(/, "Rust analysis bridge must expose source-map-aware diagnostics conversion");
    assert.match(rustBridge, /span_to_js_with_map\s*\(\s*source,\s*d\.primary\.span,\s*source_map\s*\)/, "diagnostic primary spans must keep file_path metadata when a SourceMap is available");
    assert.match(rustBridge, /diagnostics_to_js_with_map\s*\(\s*source,\s*Some\(&source_map\),\s*&all_diags\s*\)/, "VFS semantic analysis must emit file_path-bearing diagnostics");

    const providerSource = fs.readFileSync(path.join(repo, "web", "src", "language", "neplg2", "neplg2-provider.ts"), "utf8");
    assert.doesNotMatch(providerSource, /\b_spanFrom\s*\(/, "provider must not keep a second active-text-only span mapper");
    assert.match(providerSource, /activePath:\s*path/, "provider bridge snapshots must carry the active path");

    const bridge = await loadBridge();
    const activePath = "/workspace/main.nepl";
    const importedPath = "/workspace/lib.nepl";

    const source = "あadd value\n";
    const addSpan = byteSpan(source, "add", { filePath: activePath });
    const valueSpan = byteSpan(source, "value", { filePath: activePath });
    const importedSpan = {
        file_path: importedPath,
        start: 8,
        end: 11,
        start_line: 0,
        start_col: 0,
        end_line: 0,
        end_col: 3,
    };
    const snapshot = {
        path: activePath,
        activePath,
        semantics: {
            diagnostics: [
                {
                    severity: "error",
                    code: "active.problem",
                    message: "active diagnostic",
                    primary: valueSpan,
                },
                {
                    severity: "error",
                    code: "imported.problem",
                    message: "imported diagnostic",
                    primary: importedSpan,
                },
            ],
            token_semantics: [
                {
                    token_index: 0,
                    inferred_type: "fn i32 i32",
                    expr_span: addSpan,
                },
            ],
        },
        lex: {
            tokens: [
                {
                    kind: "Ident",
                    value: "add",
                    span: addSpan,
                },
            ],
        },
    };

    const payload = bridge.buildEditorUpdatePayloadFromAnalysis(source, snapshot);
    assert.equal(payload.diagnostics.length, 1, "imported-file diagnostics must not be shown in the active editor");
    assert.equal(payload.diagnostics[0].code, "active.problem");
    assert.equal(payload.diagnostics[0].startIndex, source.indexOf("value"));

    assert.equal(payload.semanticTokens.length, 1);
    assert.deepEqual(payload.semanticTokens[0].exprSpan, {
        start: source.indexOf("add"),
        end: source.indexOf("add") + "add".length,
    });
    assert.equal(payload.inlayHints.length, 1);
    assert.equal(payload.inlayHints[0].position, source.indexOf("add"));

    const hover = bridge.getHoverInfoFromAnalysis(source, snapshot, source.indexOf("add"));
    assert.ok(hover, "hover should use the mapped active-file span");
    assert.match(hover.content, /expr: add/);

    const definitionSnapshot = {
        ...snapshot,
        resolve: {
            references: [
                {
                    name: "add",
                    resolved_def_id: 10,
                    span: addSpan,
                },
                {
                    name: "add",
                    resolved_def_id: 10,
                    span: importedSpan,
                },
            ],
        },
        semantics: {
            token_resolution: [
                {
                    token_index: 0,
                    name: "add",
                    resolved_def_id: 10,
                    resolved_definition: {
                        id: 10,
                        name: "add",
                        kind: "fn",
                        span: importedSpan,
                    },
                    candidate_def_ids: [10],
                    candidate_definitions: [
                        {
                            id: 10,
                            name: "add",
                            kind: "fn",
                            span: importedSpan,
                        },
                    ],
                },
            ],
            token_semantics: snapshot.semantics.token_semantics,
        },
    };

    const location = bridge.getDefinitionLocationFromAnalysis(source, definitionSnapshot, source.indexOf("add"));
    assert.ok(location, "cross-file definition should preserve a location payload");
    assert.equal(location.isCrossFile, true);
    assert.equal(location.targetPath, importedPath);
    assert.equal(location.targetRange, null);
    assert.deepEqual(location.targetByteRange, { startByte: 8, endByte: 11 });
    assert.equal(location.targetIndex, 0, "cross-file definitions must not move the active editor cursor by a byte offset");

    const importedSource = "見出し😀\nfn add value\n";
    const mappedRange = bridge.mapAnalysisSpanToTextRange(
        importedSource,
        byteSpan(importedSource, "value", { filePath: importedPath }),
        importedPath,
    );
    assert.deepEqual(mappedRange, {
        startIndex: importedSource.indexOf("value"),
        endIndex: importedSource.indexOf("value") + "value".length,
    }, "definition navigation must map target-file byte spans to the target editor UTF-16 range");

    const occurrences = bridge.getOccurrencesFromAnalysis(source, definitionSnapshot, source.indexOf("add"));
    assert.deepEqual(occurrences, [{
        startIndex: source.indexOf("add"),
        endIndex: source.indexOf("add") + "add".length,
    }], "occurrence highlights must stay inside the active editor file");

    console.log("playground analysis span identity regression passed");
}

main().catch((error) => {
    console.error(error && error.stack ? error.stack : String(error));
    process.exit(1);
});
