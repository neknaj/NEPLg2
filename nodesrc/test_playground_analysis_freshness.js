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

    console.log("playground analysis freshness regression passed");
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
