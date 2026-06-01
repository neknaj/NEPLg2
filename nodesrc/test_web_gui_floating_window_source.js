#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function runWebGuiFloatingWindowSourceRegression() {
    const indexHtml = readRepoFile("web", "index.html");
    const managerSource = readRepoFile("web", "src", "gui-preview", "window-manager.ts");
    const panelSource = readRepoFile("web", "src", "gui-preview", "panel.ts");
    const panelManagerSource = readRepoFile("web", "src", "workspace", "panel-manager.ts");
    const css = readRepoFile("web", "styles.css");
    const guardedSources = [
        ["window-manager.ts", managerSource],
        ["panel.ts", panelSource],
    ];

    assert.match(indexHtml, /id="gui-window-layer"/);
    assert.match(indexHtml, /id="gui-button"/);
    assert.match(managerSource, /class GuiFloatingWindowManager/);
    assert.match(managerSource, /WindowMoveState =[\s\S]*kind: 'idle'/);
    assert.match(managerSource, /WindowMode =[\s\S]*kind: 'normal'[\s\S]*kind: 'minimized'; previousMode: RestorableWindowMode[\s\S]*kind: 'maximized'/);
    assert.match(managerSource, /restorableMode/);
    assert.match(managerSource, /previousMode: this\.restorableMode\(windowState\.mode\)/);
    assert.match(managerSource, /windowState\.mode = windowState\.mode\.previousMode/);
    assert.match(managerSource, /WindowLookup =[\s\S]*kind: 'missing'/);
    assert.match(panelSource, /GuiPreviewSource =[\s\S]*kind: 'none'[\s\S]*kind: 'path'/);
    assert.match(managerSource, /openWindowForSourcePath/);
    assert.match(managerSource, /openWindowForKind/);
    assert.match(managerSource, /minimizeWindow/);
    assert.match(managerSource, /toggleMaximizeWindow/);
    assert.match(managerSource, /startDrag/);
    assert.match(managerSource, /startResize/);
    assert.match(managerSource, /gui-window-dock/);
    for (const [name, source] of guardedSources) {
        assert.doesNotMatch(source, /!\)/, `${name} must not use non-null assertion before a call`);
        assert.doesNotMatch(source, /!\./, `${name} must not use non-null assertion before property access`);
        assert.doesNotMatch(source, /!\[/, `${name} must not use non-null assertion before index access`);
        assert.doesNotMatch(source, /handle!/, `${name} must not force resize handle presence`);
        assert.doesNotMatch(source, /\|\s*null/, `${name} must model absence with explicit unions`);
        assert.doesNotMatch(source, /\|\s*undefined/, `${name} must model absence with explicit unions`);
        assert.doesNotMatch(source, /minimized:\s*boolean/, `${name} must not model window mode as independent booleans`);
        assert.doesNotMatch(source, /maximized:\s*boolean/, `${name} must not model window mode as independent booleans`);
    }
    assert.match(panelManagerSource, /new GuiFloatingWindowManager/);
    assert.match(panelManagerSource, /floatingGui\.openWindowForSourcePath/);
    assert.match(panelManagerSource, /floatingGui\.openWindowForKind/);
    assert.doesNotMatch(panelManagerSource, /showGuiPreviewForActiveFile\(\) \{\s*const previewLeafId = this\.ensureGuiPreviewLeaf/s);
    assert.match(css, /\.gui-window-layer/);
    assert.match(css, /\.gui-floating-window/);
    assert.match(css, /\.gui-window-resize-se/);

    return {
        ok: true,
        checks: [
            "Web Playground exposes a GUI window layer above the workspace",
            "GUI floating windows support minimize, maximize, drag, and resize handlers",
            "editor GUI action opens a floating window instead of requiring a layout pane",
        ],
    };
}

if (require.main === module) {
    try {
        const result = runWebGuiFloatingWindowSourceRegression();
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    } catch (error) {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    }
}

module.exports = {
    runWebGuiFloatingWindowSourceRegression,
};
