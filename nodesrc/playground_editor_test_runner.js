#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const { pathToFileURL } = require('node:url');

async function loadEditorCoreBridge() {
    const bridgePath = path.resolve(__dirname, '..', 'web', 'dist_ts', 'editor-core', 'bridge.js');
    if (!fs.existsSync(bridgePath)) {
        throw new Error(`editor-core bridge not found: ${bridgePath}\nrun 'npm --prefix web run build:ts' first.`);
    }
    const mod = await import(pathToFileURL(bridgePath).href);
    return mod.NEPLPlaygroundEditorCore || mod.default || mod;
}

async function loadLanguageAnalysisBridge() {
    const bridgePath = path.resolve(__dirname, '..', 'web', 'dist_ts', 'editor-core', 'language-analysis.js');
    if (!fs.existsSync(bridgePath)) {
        throw new Error(`language analysis bridge not found: ${bridgePath}\nrun 'npm --prefix web run build:ts' first.`);
    }
    const mod = await import(pathToFileURL(bridgePath).href);
    return mod.NEPLPlaygroundLanguageAnalysis || mod.default || mod;
}

function readJson(filePath) {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function readFixtureText(filePath) {
    return fs.readFileSync(filePath, 'utf8').replace(/\r\n?/g, '\n');
}

function isDirectory(filePath) {
    try {
        return fs.statSync(filePath).isDirectory();
    } catch {
        return false;
    }
}

function findCaseDirectories(inputPath) {
    const resolved = path.resolve(inputPath);
    const commandsPath = path.join(resolved, 'commands.json');
    const analysisPath = path.join(resolved, 'analysis.json');
    if (fs.existsSync(commandsPath) || fs.existsSync(analysisPath)) {
        return [resolved];
    }
    if (!isDirectory(resolved)) {
        return [];
    }

    const discovered = [];
    for (const entry of fs.readdirSync(resolved, { withFileTypes: true })) {
        if (!entry.isDirectory()) {
            continue;
        }
        const childPath = path.join(resolved, entry.name);
        discovered.push(...findCaseDirectories(childPath));
    }
    return discovered;
}

function applyScriptCommand(bridge, state, command) {
    if (!command || typeof command !== 'object') {
        throw new Error(`invalid command: ${JSON.stringify(command)}`);
    }
    if (command.kind === 'keyboard_event') {
        const mappedCommand = bridge.mapKeyboardEventToCoreCommand({
            key: command.key || '',
            ctrlKey: Boolean(command.ctrlKey),
            metaKey: Boolean(command.metaKey),
            shiftKey: Boolean(command.shiftKey),
            altKey: Boolean(command.altKey),
        });
        if (!mappedCommand) {
            if (command.allowUnmapped) {
                return state;
            }
            throw new Error(`keyboard event was not mapped: ${JSON.stringify(command)}`);
        }
        return bridge.reduceEditorCommand(state, mappedCommand);
    }
    return bridge.reduceEditorCommand(state, command);
}

async function runCase(caseDir, bridge = null) {
    const editorBridge = bridge || await loadEditorCoreBridge();
    const sourcePath = path.join(caseDir, 'source.nepl');
    const commandsPath = path.join(caseDir, 'commands.json');
    const expectedPath = path.join(caseDir, 'expected.json');

    const source = fs.existsSync(sourcePath) ? readFixtureText(sourcePath) : '';
    const commands = fs.existsSync(commandsPath) ? readJson(commandsPath) : [];
    let state = editorBridge.createEditorRuntimeState(source);
    for (const command of commands) {
        state = applyScriptCommand(editorBridge, state, command);
    }
    const snapshot = editorBridge.snapshotEditorRuntimeState(state);
    const expected = fs.existsSync(expectedPath) ? readJson(expectedPath) : null;
    return { snapshot, expected };
}

async function runAnalysisCase(caseDir, bridge = null) {
    const analysisBridge = bridge || await loadLanguageAnalysisBridge();
    const sourcePath = path.join(caseDir, 'source.nepl');
    const analysisPath = path.join(caseDir, 'analysis.json');
    const requestsPath = path.join(caseDir, 'requests.json');
    const expectedPath = path.join(caseDir, 'expected.json');

    const source = fs.existsSync(sourcePath) ? readFixtureText(sourcePath) : '';
    const analysis = fs.existsSync(analysisPath) ? readJson(analysisPath) : {};
    const requests = fs.existsSync(requestsPath) ? readJson(requestsPath) : [];
    const outputs = [];

    for (const request of requests) {
        if (!request || typeof request !== 'object') {
            throw new Error(`invalid analysis request: ${JSON.stringify(request)}`);
        }
        switch (request.kind) {
            case 'update_payload':
                outputs.push({
                    kind: request.kind,
                    value: analysisBridge.buildEditorUpdatePayloadFromAnalysis(source, analysis),
                });
                break;
            case 'token_insight':
                outputs.push({
                    kind: request.kind,
                    index: request.index,
                    value: analysisBridge.getTokenInsightFromAnalysis(source, analysis, request.index),
                });
                break;
            case 'hover':
                outputs.push({
                    kind: request.kind,
                    index: request.index,
                    value: analysisBridge.getHoverInfoFromAnalysis(source, analysis, request.index),
                });
                break;
            case 'definition':
                outputs.push({
                    kind: request.kind,
                    index: request.index,
                    value: analysisBridge.getDefinitionLocationFromAnalysis(source, analysis, request.index),
                });
                break;
            case 'occurrences':
                outputs.push({
                    kind: request.kind,
                    index: request.index,
                    value: analysisBridge.getOccurrencesFromAnalysis(source, analysis, request.index),
                });
                break;
            default:
                throw new Error(`unsupported analysis request kind: ${request.kind}`);
        }
    }

    const snapshot = { outputs };
    const expected = fs.existsSync(expectedPath) ? readJson(expectedPath) : null;
    return { snapshot, expected };
}

async function runCases(inputs) {
    const bridge = await loadEditorCoreBridge();
    const analysisBridge = await loadLanguageAnalysisBridge();
    const caseDirs = [];
    for (const input of inputs) {
        caseDirs.push(...findCaseDirectories(input));
    }
    const uniqueCaseDirs = [...new Set(caseDirs)].sort();
    const results = [];

    for (const caseDir of uniqueCaseDirs) {
        try {
            const isAnalysisCase = fs.existsSync(path.join(caseDir, 'analysis.json'));
            const { snapshot, expected } = isAnalysisCase
                ? await runAnalysisCase(caseDir, analysisBridge)
                : await runCase(caseDir, bridge);
            if (expected) {
                assert.deepStrictEqual(snapshot, expected);
            }
            results.push({
                caseDir,
                ok: true,
                snapshot,
            });
        } catch (error) {
            results.push({
                caseDir,
                ok: false,
                error: error && error.stack ? error.stack : String(error),
            });
        }
    }

    return {
        caseCount: results.length,
        passedCount: results.filter((result) => result.ok).length,
        failedCount: results.filter((result) => !result.ok).length,
        results,
    };
}

async function main() {
    const args = process.argv.slice(2);
    const inputIndex = args.indexOf('--case');
    const outputIndex = args.indexOf('--out');
    if (inputIndex === -1 || inputIndex + 1 >= args.length) {
        console.error('Usage: node nodesrc/playground_editor_test_runner.js --case <dir> [--out <file>]');
        process.exit(2);
    }

    const caseDir = path.resolve(args[inputIndex + 1]);
    const { snapshot, expected } = await runCase(caseDir);
    if (expected) {
        assert.deepStrictEqual(snapshot, expected);
    }
    const json = JSON.stringify(snapshot, null, 2);

    if (outputIndex !== -1 && outputIndex + 1 < args.length) {
        const outPath = path.resolve(args[outputIndex + 1]);
        fs.mkdirSync(path.dirname(outPath), { recursive: true });
        fs.writeFileSync(outPath, json);
    } else {
        process.stdout.write(json + '\n');
    }
}

if (require.main === module) {
    main().catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    findCaseDirectories,
    loadEditorCoreBridge,
    loadLanguageAnalysisBridge,
    runCase,
    runAnalysisCase,
    runCases,
};
