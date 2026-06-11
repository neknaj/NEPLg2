#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

async function main() {
    const repo = path.resolve(__dirname, "..");
    const providerPath = path.join(repo, "web", "dist_ts", "language", "neplg2", "neplg2-provider.js");
    if (!fs.existsSync(providerPath)) {
        throw new Error(`NEPLg2 provider build output not found: ${providerPath}\nrun 'npm --prefix web run build:ts' first.`);
    }

    const timers = [];
    const semanticCalls = {
        tokenInsight: 0,
        hover: 0,
        definition: 0,
        occurrences: 0,
    };
    const updates = [];

    const context = {
        console,
        setTimeout(callback) {
            const id = timers.length;
            timers.push({ callback, active: true });
            return id;
        },
        clearTimeout(id) {
            if (timers[id]) {
                timers[id].active = false;
            }
        },
        window: {
            wasmBindings: {
                analyze_lex() {
                    return { tokens: [], diagnostics: [] };
                },
                analyze_parse() {
                    return { module: null, diagnostics: [] };
                },
                analyze_semantics(source) {
                    const symbol = source.includes("freshSymbol") ? "freshSymbol" : "staleSymbol";
                    return {
                        ok: true,
                        tokens: [{
                            kind: "Ident",
                            value: symbol,
                            span: {
                                start: 0,
                                end: symbol.length,
                                start_line: 0,
                                start_col: 0,
                                end_line: 0,
                                end_col: symbol.length,
                            },
                        }],
                        diagnostics: [],
                        name_resolution: {
                            definitions: [{
                                id: 1,
                                name: symbol,
                                kind: "fn",
                                span: {
                                    start: 0,
                                    end: symbol.length,
                                    start_line: 0,
                                    start_col: 0,
                                    end_line: 0,
                                    end_col: symbol.length,
                                },
                            }],
                            references: [{
                                name: symbol,
                                resolved_def_id: 1,
                                span: {
                                    start: 0,
                                    end: symbol.length,
                                    start_line: 0,
                                    start_col: 0,
                                    end_line: 0,
                                    end_col: symbol.length,
                                },
                            }],
                            by_name: {
                                [symbol]: [{ id: 1 }],
                            },
                        },
                        token_resolution: [{
                            token_index: 0,
                            name: symbol,
                            resolved_def_id: 1,
                            candidate_def_ids: [1],
                        }],
                        token_semantics: [{
                            token_index: 0,
                            inferred_type: "unit",
                            expr_span: { start: 0, end: symbol.length },
                            arg_index: null,
                            arg_span: null,
                        }],
                        token_classifications: [],
                        syntax_ranges: [],
                    };
                },
            },
            NEPLPlaygroundLanguageAnalysis: {
                buildEditorUpdatePayloadFromAnalysis(text, snapshot) {
                    const symbol = text.includes("freshSymbol") ? "freshSymbol" : "staleSymbol";
                    return {
                        text,
                        snapshot,
                        tokens: [],
                        semanticHighlightTokens: [{
                            startIndex: 0,
                            endIndex: symbol.length,
                            type: "function",
                        }],
                        diagnostics: [{
                            startIndex: 0,
                            endIndex: symbol.length,
                            message: `${symbol} diagnostic`,
                            severity: "warning",
                        }],
                        foldingRanges: [{
                            startLine: 0,
                            endLine: 1,
                            placeholder: "...",
                        }],
                        semanticTokens: [{
                            tokenIndex: 0,
                            inferredType: "unit",
                            exprSpan: { start: 0, end: symbol.length },
                            argIndex: null,
                            argSpan: null,
                        }],
                        inlayHints: [{
                            kind: "type",
                            position: 0,
                            label: "<unit>",
                            exprSpan: { start: 0, end: symbol.length },
                        }],
                        config: {
                            highlightWhitespace: false,
                            highlightIndent: true,
                        },
                    };
                },
                remapEditorUpdatePayloadForTextChange(previousText, nextText, previousPayload) {
                    return {
                        ...previousPayload,
                        text: nextText,
                        remappedFrom: previousText,
                    };
                },
                getTokenInsightFromAnalysis() {
                    semanticCalls.tokenInsight += 1;
                    return {
                        definitionCandidates: [{ name: "semantic-definition" }],
                    };
                },
                getHoverInfoFromAnalysis() {
                    semanticCalls.hover += 1;
                    return { content: "hover", startIndex: 0, endIndex: 1 };
                },
                getDefinitionLocationFromAnalysis() {
                    semanticCalls.definition += 1;
                    return { targetIndex: 0 };
                },
                getOccurrencesFromAnalysis() {
                    semanticCalls.occurrences += 1;
                    return [{ startIndex: 0, endIndex: 1 }];
                },
            },
        },
    };

    vm.runInNewContext(fs.readFileSync(providerPath, "utf8"), context, { filename: providerPath });

    const provider = new context.window.NEPLg2LanguageProvider();
    provider.onUpdate((payload) => updates.push(payload));

    provider.replaceDocument({
        path: "/examples/main.nepl",
        text: "staleSymbol\n",
        editable: true,
    });
    runNextTimer(timers);

    const firstFresh = updates.at(-1);
    assert.equal(firstFresh.analysis.freshness, "fresh");
    assert.equal(firstFresh.analysis.isFresh, true);
    assert.equal(firstFresh.analysis.path, "/examples/main.nepl");
    assert.equal(firstFresh.analysis.documentVersion, 1);

    assert.equal(provider.getTokenInsight(0).definitionCandidates[0].name, "semantic-definition");
    assert.equal(semanticCalls.tokenInsight, 1);
    assert.ok((await completionLabels(provider, "staleSymbol\n".length)).includes("staleSymbol"));

    provider.updateText("freshSymbol\n");

    const provisional = updates.at(-1);
    assert.equal(provisional.analysis.freshness, "provisional");
    assert.equal(provisional.analysis.isFresh, false);
    assert.equal(provisional.analysis.documentVersion, 2);
    assert.equal(provisional.analysis.sourceDocumentVersion, 1);
    assert.equal(provisional.analysis.sourcePath, "/examples/main.nepl");
    assert.equal(provisional.semanticHighlightTokens.length, 0);
    assert.equal(provisional.diagnostics.length, 0);
    assert.equal(provisional.foldingRanges.length, 0);
    assert.equal(provisional.semanticTokens.length, 0);
    assert.equal(provisional.inlayHints.length, 0);

    assert.equal(provider.getTokenInsight(0), null);
    assert.equal((await provider.getDefinitionCandidates(0)).length, 0);
    assert.equal(await provider.getHoverInfo(0), null);
    assert.equal(await provider.getDefinitionLocation(0), null);
    assert.equal((await provider.getOccurrences(0)).length, 0);
    assert.equal(semanticCalls.tokenInsight, 1);
    assert.equal(semanticCalls.hover, 0);
    assert.equal(semanticCalls.definition, 0);
    assert.equal(semanticCalls.occurrences, 0);
    assert.ok(!(await completionLabels(provider, "freshSymbol\n".length)).includes("staleSymbol"));
    assert.ok(!(await completionLabels(provider, "freshSymbol\n".length)).includes("freshSymbol"));

    provider.updateText("freshSymbolTwo\n");
    const secondProvisional = updates.at(-1);
    assert.equal(secondProvisional.analysis.freshness, "provisional");
    assert.equal(secondProvisional.analysis.documentVersion, 3);
    assert.equal(secondProvisional.analysis.sourceDocumentVersion, 1);
    assert.equal(secondProvisional.analysis.sourcePath, "/examples/main.nepl");
    assert.equal(secondProvisional.semanticHighlightTokens.length, 0);

    runNextTimer(timers);

    const secondFresh = updates.at(-1);
    assert.equal(secondFresh.analysis.freshness, "fresh");
    assert.equal(secondFresh.analysis.isFresh, true);
    assert.equal(secondFresh.analysis.documentVersion, 3);
    assert.ok((await completionLabels(provider, "freshSymbolTwo\n".length)).includes("freshSymbol"));
    assert.ok(!(await completionLabels(provider, "freshSymbolTwo\n".length)).includes("staleSymbol"));
    assert.equal(provider.getTokenInsight(0).definitionCandidates[0].name, "semantic-definition");
    assert.equal(semanticCalls.tokenInsight, 2);

    await runWorkerAnalysisScenario(providerPath);

    console.log("playground analysis freshness regression passed");
}

async function runWorkerAnalysisScenario(providerPath) {
    const timers = [];
    const updates = [];
    const syncCalls = {
        semantics: 0,
        parse: 0,
        hover: 0,
    };
    const workerInstances = [];

    class FakeWorker {
        constructor(pathValue, options) {
            this.path = pathValue;
            this.options = options;
            this.messages = [];
            this.terminated = false;
            this.onmessage = null;
            this.onerror = null;
            workerInstances.push(this);
        }

        postMessage(message) {
            this.messages.push(message);
        }

        terminate() {
            this.terminated = true;
        }
    }

    const context = {
        console,
        Worker: FakeWorker,
        setTimeout(callback) {
            const id = timers.length;
            timers.push({ callback, active: true });
            return id;
        },
        clearTimeout(id) {
            if (timers[id]) {
                timers[id].active = false;
            }
        },
        window: {
            NEPLg2CompilerAssets: {
                moduleUrl: "compiler.js",
                wasmUrl: "compiler_bg.wasm",
            },
            wasmBindings: {
                analyze_lex() {
                    throw new Error("sync analyze_lex must not be called when analysis worker is available");
                },
                analyze_parse() {
                    syncCalls.parse += 1;
                    throw new Error("sync analyze_parse must not be called when analysis worker is available");
                },
                analyze_semantics() {
                    syncCalls.semantics += 1;
                    throw new Error("sync analyze_semantics must not be called when analysis worker is available");
                },
            },
            NEPLPlaygroundLanguageAnalysis: {
                buildEditorUpdatePayloadFromAnalysis(text, snapshot) {
                    return {
                        text,
                        snapshot,
                        tokens: [],
                        semanticHighlightTokens: [],
                        diagnostics: [],
                        foldingRanges: [],
                        semanticTokens: [],
                        inlayHints: [],
                        config: {
                            highlightWhitespace: false,
                            highlightIndent: true,
                        },
                    };
                },
                remapEditorUpdatePayloadForTextChange(previousText, nextText, previousPayload) {
                    return {
                        ...previousPayload,
                        text: nextText,
                        remappedFrom: previousText,
                    };
                },
                getTokenInsightFromAnalysis() {
                    return {
                        definitionCandidates: [],
                    };
                },
                getHoverInfoFromAnalysis() {
                    syncCalls.hover += 1;
                    return { content: "worker hover", startIndex: 0, endIndex: 1 };
                },
                getDefinitionLocationFromAnalysis() {
                    return { targetIndex: 0 };
                },
                getOccurrencesFromAnalysis() {
                    return [];
                },
            },
        },
    };

    vm.runInNewContext(fs.readFileSync(providerPath, "utf8"), context, { filename: providerPath });
    const provider = new context.window.NEPLg2LanguageProvider();
    provider.onUpdate((payload) => updates.push(payload));

    provider.replaceDocument({
        path: "/examples/main.nepl",
        text: "firstSymbol\n",
        editable: true,
    });
    runNextTimer(timers);

    assert.equal(workerInstances.length, 1);
    const firstWorker = workerInstances[0];
    assert.equal(firstWorker.options.type, "module");
    assert.match(String(firstWorker.path), /neplg2-analysis-worker\.js/);
    const firstRequest = firstWorker.messages.at(-1);
    assert.equal(firstRequest.type, "analyze");
    assert.equal(firstRequest.path, "/examples/main.nepl");
    assert.equal(firstRequest.text, "firstSymbol\n");
    assert.equal(firstRequest.compiler.moduleUrl, context.window.NEPLg2CompilerAssets.moduleUrl);
    assert.equal(firstRequest.compiler.wasmUrl, context.window.NEPLg2CompilerAssets.wasmUrl);
    assert.equal(syncCalls.semantics, 0);
    assert.equal(syncCalls.parse, 0);

    provider.updateText("secondSymbol\n");
    assert.equal(firstWorker.terminated, true);
    firstWorker.onmessage({
        data: buildWorkerAnalysisResult(firstRequest.requestId, "firstSymbol"),
    });
    await flushMicrotasks();
    assert.notEqual(updates.at(-1).analysis?.freshness, "fresh");

    runNextTimer(timers);
    assert.equal(workerInstances.length, 2);
    const secondWorker = workerInstances[1];
    const secondRequest = secondWorker.messages.at(-1);
    assert.equal(secondRequest.type, "analyze");
    assert.equal(secondRequest.text, "secondSymbol\n");
    secondWorker.onmessage({
        data: buildWorkerAnalysisResult(secondRequest.requestId, "secondSymbol"),
    });
    await flushMicrotasks();

    const fresh = updates.at(-1);
    assert.equal(fresh.analysis.freshness, "fresh");
    assert.equal(fresh.analysis.documentVersion, 2);
    assert.equal(fresh.analysis.sourceDocumentVersion, 2);
    assert.equal(syncCalls.semantics, 0);
    assert.equal(syncCalls.parse, 0);

    assert.deepEqual(await provider.getHoverInfo(0), { content: "worker hover", startIndex: 0, endIndex: 1 });
    assert.equal(syncCalls.hover, 1);
    assert.equal(syncCalls.parse, 0);

    runNextTimer(timers);
    const parseRequest = secondWorker.messages.at(-1);
    assert.equal(parseRequest.type, "parse");
    secondWorker.onmessage({
        data: {
            type: "structural-result",
            requestId: parseRequest.requestId,
            module: { root: { kind: "Module" } },
        },
    });
    await flushMicrotasks();
    assert.equal(syncCalls.parse, 0);
}

function buildWorkerAnalysisResult(requestId, symbol) {
    const span = {
        start: 0,
        end: symbol.length,
        start_line: 0,
        start_col: 0,
        end_line: 0,
        end_col: symbol.length,
    };
    const lex = {
        tokens: [{
            kind: "Ident",
            value: symbol,
            span,
        }],
        diagnostics: [],
    };
    const resolve = {
        definitions: [{
            id: 1,
            name: symbol,
            kind: "fn",
            span,
        }],
        references: [{
            name: symbol,
            resolved_def_id: 1,
            span,
        }],
        by_name: {
            [symbol]: [{ id: 1 }],
        },
    };
    const semantics = {
        ok: true,
        tokens: lex.tokens,
        diagnostics: [],
        name_resolution: resolve,
        token_resolution: [{
            token_index: 0,
            name: symbol,
            resolved_def_id: 1,
            candidate_def_ids: [1],
        }],
        token_semantics: [],
        token_classifications: [],
        syntax_ranges: [],
    };
    const parse = {
        ok: true,
        module: null,
        diagnostics: [],
    };
    return {
        type: "analysis-result",
        requestId,
        lex,
        parse,
        resolve,
        semantics,
        payload: {
            tokens: [],
            semanticHighlightTokens: [],
            diagnostics: [],
            foldingRanges: [],
            semanticTokens: [],
            inlayHints: [],
            config: {
                highlightWhitespace: false,
                highlightIndent: true,
            },
        },
    };
}

async function flushMicrotasks() {
    await Promise.resolve();
    await Promise.resolve();
}

function runNextTimer(timers) {
    const timer = timers.find((item) => item.active);
    assert.ok(timer, "expected a pending timer");
    timer.active = false;
    timer.callback();
}

async function completionLabels(provider, index) {
    return (await provider.getCompletions(index)).map((item) => item.label);
}

main().catch((error) => {
    console.error(error && error.stack ? error.stack : String(error));
    process.exit(1);
});
