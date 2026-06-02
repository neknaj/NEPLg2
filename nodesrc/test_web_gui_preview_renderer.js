#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function repoPathExists(...parts) {
    return fs.existsSync(path.resolve(__dirname, "..", ...parts));
}

function runWebGuiPreviewRendererRegression() {
    const commandSource = readRepoFile("web", "src", "gui-preview", "commands.ts");
    const canvasSource = readRepoFile("web", "src", "gui-preview", "canvas-renderer.ts");
    const panelSource = readRepoFile("web", "src", "gui-preview", "panel.ts");
    const panelLayoutSource = readRepoFile("web", "src", "workspace", "panel-layout.ts");
    const panelManagerSource = readRepoFile("web", "src", "workspace", "panel-manager.ts");

    assert.equal(repoPathExists("web", "src", "gui-preview", "renderer.ts"), false);
    assert.match(commandSource, /GuiPreviewDrawCommand =[\s\S]*kind: 'fill-rect'[\s\S]*kind: 'text-run'/);
    assert.match(commandSource, /GuiPreviewCommandFrame/);
    assert.match(canvasSource, /renderGuiPreviewFrameToCanvas/);
    assert.match(panelSource, /presentHostFrame\(frame: GuiPreviewCommandFrame, windowId: number\)/);
    assert.match(panelSource, /waiting for host frame/);
    assert.doesNotMatch(commandSource, /GuiPreviewKind/);
    assert.doesNotMatch(canvasSource, /renderGuiPreviewSceneToCanvas|GuiPreviewScene|renderer\.js/);
    assert.doesNotMatch(panelSource, /createGuiPreviewScene|summarizeGuiPreviewScene|guiPreviewKindFromPath|renderGuiPreviewSceneToCanvas/);
    assert.doesNotMatch(panelSource, /HTMLSelectElement|gui-preview-select|mountToolbar|counterValue/);
    assert.doesNotMatch(panelLayoutSource, /'gui-preview'/);
    assert.doesNotMatch(panelManagerSource, /createGuiPreviewRuntime|showGuiPreviewForActiveFile|openWindowForSourcePath|openWindowForKind/);
    assert.doesNotMatch(panelManagerSource, /GuiPreviewPanel|GuiPreviewRuntime|previewKind/);

    return {
        ok: true,
        checks: [
            "old TS GUI example renderer is removed",
            "Web GUI canvas renderer accepts only host command frames",
            "Web GUI panel no longer simulates NEPL examples",
            "workspace panel layout no longer exposes GUI preview panes",
        ],
    };
}

if (require.main === module) {
    try {
        const result = runWebGuiPreviewRendererRegression();
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    } catch (error) {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    }
}

module.exports = {
    runWebGuiPreviewRendererRegression,
};
