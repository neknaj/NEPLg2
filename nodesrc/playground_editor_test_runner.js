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

function readJson(filePath) {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
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
    if (fs.existsSync(commandsPath)) {
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

    const source = fs.existsSync(sourcePath) ? fs.readFileSync(sourcePath, 'utf8') : '';
    const commands = fs.existsSync(commandsPath) ? readJson(commandsPath) : [];
    let state = editorBridge.createEditorRuntimeState(source);
    for (const command of commands) {
        state = applyScriptCommand(editorBridge, state, command);
    }
    const snapshot = editorBridge.snapshotEditorRuntimeState(state);
    const expected = fs.existsSync(expectedPath) ? readJson(expectedPath) : null;
    return { snapshot, expected };
}

async function runCases(inputs) {
    const bridge = await loadEditorCoreBridge();
    const caseDirs = [];
    for (const input of inputs) {
        caseDirs.push(...findCaseDirectories(input));
    }
    const uniqueCaseDirs = [...new Set(caseDirs)].sort();
    const results = [];

    for (const caseDir of uniqueCaseDirs) {
        try {
            const { snapshot, expected } = await runCase(caseDir, bridge);
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
    runCase,
    runCases,
};
