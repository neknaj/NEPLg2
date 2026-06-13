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
    const mainSource = readRepoFile("web", "src", "main.ts");
    const css = readRepoFile("web", "styles.css");
    const guardedSources = [
        ["window-manager.ts", managerSource],
        ["panel.ts", panelSource],
    ];
    const videoMemoryPanelMethod = extractMethodSource(panelSource, "presentVideoMemorySurface", "focusInputSurface");

    assert.match(indexHtml, /id="gui-window-layer"/);
    assert.doesNotMatch(indexHtml, /id="gui-button"/);
    assert.match(managerSource, /class GuiFloatingWindowManager/);
    assert.match(managerSource, /WindowMoveState =[\s\S]*kind: 'idle'/);
    assert.match(managerSource, /WindowMode =[\s\S]*kind: 'normal'[\s\S]*kind: 'minimized'; previousMode: RestorableWindowMode[\s\S]*kind: 'maximized'/);
    assert.match(managerSource, /restorableMode/);
    assert.match(managerSource, /previousMode: this\.restorableMode\(windowState\.mode\)/);
    assert.match(managerSource, /windowState\.mode = windowState\.mode\.previousMode/);
    assert.match(managerSource, /WindowLookup =[\s\S]*kind: 'missing'/);
    assert.match(panelSource, /GuiHostFrameState =[\s\S]*kind: 'none'[\s\S]*kind: 'command-frame'[\s\S]*kind: 'video-memory'/);
    assert.match(panelSource, /GuiPreviewDebugSink =[\s\S]*kind: 'none'[\s\S]*kind: 'present'/);
    assert.doesNotMatch(panelSource, /metricsEl|gui-preview-metrics|host commands|queued action/);
    assert.match(managerSource, /presentHostFrame\(input: unknown\): GuiWebHostResult<string>/);
    assert.match(managerSource, /presentVideoMemorySurface\(input: GuiWebRuntimeVideoMemoryFrame\): GuiWebRuntimeResult<string>/);
    assert.match(managerSource, /preview\.presentVideoMemorySurface\(input\.buffer, input\.windowId\)/);
    assert.match(managerSource, /closeHostFrameWindow\(windowId: number\): GuiWebHostResult<string>/);
    assert.match(panelSource, /presentVideoMemorySurface\(buffer: SharedArrayBuffer, windowId: number\): GuiWebRuntimeResult<string>/);
    assert.match(panelSource, /openVideoMemorySurface\(buffer: SharedArrayBuffer\)/);
    assert.match(panelSource, /presentNewestGuiVideoMemoryFrameToCanvas/);
    assert.match(panelSource, /this\.hostFrame\.kind === 'video-memory'[\s\S]*return;/);
    assert.match(panelSource, /this\.hostFrame\.kind === 'video-memory' && this\.hostFrame\.buffer === buffer/);
    assert.match(panelSource, /activeHostWindow\(\): GuiActiveHostWindowLookup/);
    assert.match(managerSource, /class GuiWindowDebugPanel/);
    assert.match(managerSource, /new GuiPreviewPanel\(contentEl, \{[\s\S]*kind: 'present'/);
    assert.match(managerSource, /debugPanel\.record/);
    assert.match(managerSource, /setAttribute\('aria-live', 'off'\)/);
    assert.match(managerSource, /setAttribute\('aria-expanded'/);
    assert.match(managerSource, /setAttribute\('aria-hidden'/);
    assert.match(managerSource, /closeWindowState\(lookup\.windowState, \{ emitCloseRequest: true \}\)/);
    assert.match(managerSource, /closeWindowState\(lookup\.windowState, \{ emitCloseRequest: false \}\)/);
    assert.match(managerSource, /previousSurfaceSize = windowState\.preview\.drawableSurfaceCssSize\(\)/);
    assert.match(managerSource, /nextSurfaceSize = windowState\.preview\.drawableSurfaceCssSize\(\)/);
    assert.match(managerSource, /queueHostWindowEvent\(windowState, 'resized', nextSurfaceSize\)/);
    assert.match(panelSource, /drawableSurfaceCssSize\(\)/);
    assert.doesNotMatch(managerSource, /requestAnimationFrame\(\(\) => windowState\.preview\.resizeEditor\(\)\)/);
    assert.doesNotMatch(videoMemoryPanelMethod, /renderGuiPreviewFrameToCanvas/);
    assert.doesNotMatch(videoMemoryPanelMethod, /presentGuiPreviewCanvasBackground/);
    assert.doesNotMatch(videoMemoryPanelMethod, /presentHostFrame/);
    assert.match(managerSource, /minimizeWindow/);
    assert.match(managerSource, /toggleMaximizeWindow/);
    assert.match(managerSource, /startDrag/);
    assert.match(managerSource, /startResize/);
    assert.match(managerSource, /gui-window-dock/);
    assert.doesNotMatch(managerSource, /source-path|preview-kind|openWindowForSourcePath|openWindowForKind/);
    assert.doesNotMatch(panelSource, /createGuiPreviewScene|renderGuiPreviewSceneToCanvas|HTMLSelectElement/);
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
    assert.match(panelManagerSource, /stopActiveProcess\(\): boolean/);
    assert.match(panelManagerSource, /focused\.panelKind === 'terminal'[\s\S]*focused\.terminal\.shell\.isRunning/);
    assert.match(panelManagerSource, /runtime\.panelKind === 'terminal'[\s\S]*runtime\.terminal\.shell\.isRunning/);
    assert.match(mainSource, /stopBtn\.addEventListener\('click'[\s\S]*panelManager\.stopActiveProcess\(\)/);
    assert.doesNotMatch(mainSource, /stopBtn\.addEventListener\('click'[\s\S]*getFocusedTerminalRuntime\(\)[\s\S]*shell\.interrupt/);
    assert.doesNotMatch(panelManagerSource, /Open GUI preview/);
    assert.doesNotMatch(panelManagerSource, /createPanelButton\('G'/);
    assert.doesNotMatch(panelManagerSource, /showGuiPreviewForActiveFile|ensureGuiPreviewLeaf|createGuiPreviewRuntime/);
    assert.match(css, /\.gui-window-layer/);
    assert.match(css, /\.gui-floating-window/);
    assert.match(css, /\.gui-window-resize-se/);
    assert.doesNotMatch(css, /\.gui-preview-metrics/);
    assert.match(css, /\.gui-debug-panel/);
    assert.match(css, /\.gui-debug-detail/);
    assert.match(css, /\.gui-debug-panel[\s\S]*z-index: 70/);
    assert.match(css, /\.gui-debug-panel[\s\S]*pointer-events: none/);
    assert.match(css, /\.gui-debug-toggle[\s\S]*pointer-events: auto/);

    return {
        ok: true,
        checks: [
            "Web Playground exposes a GUI window layer above the workspace",
            "GUI floating windows support minimize, maximize, drag, and resize handlers",
            "manual GUI preview panes are not exposed; NEPL execution opens host-frame windows",
            "GUI floating windows report drawable surface resize instead of stretching content",
            "host event and queue status is separated from the GUI window content",
            "toolbar Stop targets the active running terminal owner instead of only the focused terminal",
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

function extractMethodSource(source, startName, nextName) {
    const start = source.indexOf(`    ${startName}`);
    assert.notEqual(start, -1, `${startName} method must exist`);
    const next = source.indexOf(`    ${nextName}`, start + 1);
    assert.notEqual(next, -1, `${nextName} method must follow ${startName}`);
    return source.slice(start, next);
}
