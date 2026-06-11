#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

function main() {
    const repo = path.resolve(__dirname, "..");
    const providerPath = path.join(repo, "web", "dist_ts", "language", "neplg2", "neplg2-provider.js");
    if (!fs.existsSync(providerPath)) {
        throw new Error(`NEPLg2 provider build output not found: ${providerPath}\nrun 'npm --prefix web run build:ts' first.`);
    }

    const calls = [];
    const timers = [];
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
                analyze_semantics() {
                    throw new Error("inline analyze_semantics should not be used for NEPL VFS files");
                },
                analyze_semantics_with_vfs(entryPath, source, vfs) {
                    calls.push({ entryPath, source, vfs });
                    return {
                        stage: "semantics",
                        ok: true,
                        tokens: [],
                        diagnostics: [],
                        name_resolution: null,
                        token_resolution: [],
                        token_semantics: [],
                        token_classifications: [],
                        syntax_ranges: [],
                    };
                },
            },
            NEPLPlaygroundLanguageAnalysis: {
                buildEditorUpdatePayloadFromAnalysis(text, snapshot) {
                    return {
                        text,
                        snapshot,
                        tokens: [],
                        diagnostics: [],
                        foldingRanges: [],
                        semanticTokens: [],
                        inlayHints: [],
                        config: {},
                    };
                },
            },
        },
    };

    vm.runInNewContext(fs.readFileSync(providerPath, "utf8"), context, { filename: providerPath });

    const vfs = {
        serializeForCompile() {
            return {
                "/examples/helper.nepl": "fn helper %fn i32 i32 \\x:\n    x\n",
                "/examples/main.nepl": "stale file content\n",
            };
        },
    };
    const provider = new context.window.NEPLg2LanguageProvider({ vfs });
    provider.onUpdate(() => {});
    provider.setPath("/examples/main.nepl");
    assert.equal(timers.filter((timer) => timer.active).length, 0);
    const currentText = "#no_prelude\nfn main %fn unit i32 \\u:\n    1\n";
    provider.replaceDocument({
        path: "/examples/main.nepl",
        text: currentText,
        editable: true,
    });

    assert.equal(calls.length, 0);
    const pendingTimer = timers.find((timer) => timer.active);
    assert.ok(pendingTimer);
    pendingTimer.active = false;
    pendingTimer.callback();

    assert.equal(calls.length, 1);
    assert.equal(calls[0].entryPath, "/examples/main.nepl");
    assert.equal(calls[0].source, currentText);
    assert.equal(calls[0].vfs["/examples/helper.nepl"], "fn helper %fn i32 i32 \\x:\n    x\n");
    assert.equal(calls[0].vfs["/examples/main.nepl"], currentText);

    console.log("NEPLg2 language provider VFS regression passed");
}

try {
    main();
} catch (error) {
    console.error(error && error.stack ? error.stack : String(error));
    process.exit(1);
}
