#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function repoPathExists(...parts) {
    return fs.existsSync(path.resolve(__dirname, "..", ...parts));
}

async function loadGuiPreviewBitmapModules() {
    const bufferPath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "bitmap-buffer.js");
    const rasterizerPath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "bitmap-rasterizer.js");
    const commandsPath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "commands.js");
    const canvasRendererPath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "canvas-renderer.js");
    return {
        buffer: await import(pathToFileURL(bufferPath).href),
        rasterizer: await import(pathToFileURL(rasterizerPath).href),
        commands: await import(pathToFileURL(commandsPath).href),
        canvasRenderer: await import(pathToFileURL(canvasRendererPath).href),
    };
}

async function runWebGuiPreviewRendererRegression() {
    const modules = await loadGuiPreviewBitmapModules();
    const commandSource = readRepoFile("web", "src", "gui-preview", "commands.ts");
    const canvasSource = readRepoFile("web", "src", "gui-preview", "canvas-renderer.ts");
    const bitmapBufferSource = readRepoFile("web", "src", "gui-preview", "bitmap-buffer.ts");
    const bitmapRasterizerSource = readRepoFile("web", "src", "gui-preview", "bitmap-rasterizer.ts");
    const bitmapPresenterSource = readRepoFile("web", "src", "gui-preview", "bitmap-presenter.ts");
    const panelSource = readRepoFile("web", "src", "gui-preview", "panel.ts");
    const panelLayoutSource = readRepoFile("web", "src", "workspace", "panel-layout.ts");
    const panelManagerSource = readRepoFile("web", "src", "workspace", "panel-manager.ts");

    assert.equal(repoPathExists("web", "src", "gui-preview", "renderer.ts"), false);
    assert.match(commandSource, /GuiPreviewDrawCommand =/);
    assert.match(commandSource, /kind: 'fill-rect'/);
    assert.match(commandSource, /kind: 'rgba-row'/);
    assert.match(commandSource, /kind: 'text-run'/);
    assert.match(commandSource, /pixels: GuiPreviewColor\[\]/);
    assert.match(commandSource, /GuiPreviewCommandFrame/);
    assert.match(canvasSource, /renderGuiPreviewFrameToCanvas/);
    assert.match(canvasSource, /rasterizeGuiPreviewCommand/);
    assert.match(bitmapBufferSource, /createGuiPreviewBitmapBuffer/);
    assert.match(bitmapBufferSource, /Uint8ClampedArray/);
    assert.match(bitmapRasterizerSource, /rasterizeGuiPreviewRgbaRow/);
    assert.match(bitmapRasterizerSource, /GUI_PREVIEW_BITMAP_FONT/);
    assert.match(bitmapRasterizerSource, /unsupported-scalar/);
    assert.match(bitmapRasterizerSource, /invalid-geometry/);
    assert.doesNotMatch(bitmapRasterizerSource, /\?\] \?\? GUI_PREVIEW_BITMAP_FONT/);
    assert.match(bitmapPresenterSource, /putImageData/);
    assert.match(panelSource, /presentHostFrame\(frame: GuiPreviewCommandFrame, windowId: number\)/);
    assert.match(panelSource, /GuiPreviewDebugSink/);
    assert.match(panelSource, /waiting-for-frame/);
    assert.match(panelSource, /render-error/);
    assert.doesNotMatch(panelSource, /waiting for host frame/);
    assert.doesNotMatch(panelSource, /metricsEl|gui-preview-metrics|host commands/);
    assert.doesNotMatch(commandSource, /GuiPreviewKind/);
    assert.doesNotMatch(canvasSource, /renderGuiPreviewSceneToCanvas|GuiPreviewScene|renderer\.js/);
    assert.doesNotMatch(canvasSource, /frame\.title|fillText\(frame\.title/);
    assert.doesNotMatch(canvasSource, /ctx\.(fillRect|strokeRect|fillText|strokeText|stroke|drawImage|clearRect)\s*\(/);
    assert.doesNotMatch(panelSource, /ctx\.(fillRect|strokeRect|fillText|strokeText|stroke|drawImage|clearRect)\s*\(/);
    assert.doesNotMatch(bitmapPresenterSource, /ctx\.(fillRect|strokeRect|fillText|strokeText|stroke|drawImage|clearRect)\s*\(/);
    assert.doesNotMatch(panelSource, /createGuiPreviewScene|summarizeGuiPreviewScene|guiPreviewKindFromPath|renderGuiPreviewSceneToCanvas/);
    assert.doesNotMatch(panelSource, /HTMLSelectElement|gui-preview-select|mountToolbar|counterValue/);
    assert.doesNotMatch(panelLayoutSource, /'gui-preview'/);
    assert.doesNotMatch(panelManagerSource, /createGuiPreviewRuntime|showGuiPreviewForActiveFile|openWindowForSourcePath|openWindowForKind/);
    assert.doesNotMatch(panelManagerSource, /GuiPreviewPanel|GuiPreviewRuntime|previewKind/);

    const buffer = modules.buffer.createGuiPreviewBitmapBuffer(64, 32, modules.commands.guiPreviewRgb(0, 0, 0));
    const ascii = modules.rasterizer.rasterizeGuiPreviewCommand(buffer, {
        kind: "text-run",
        origin: { x: 0, y: 0 },
        text: "ABC?",
        color: modules.commands.guiPreviewRgb(255, 255, 255),
        size: 14,
        align: "left",
    }, { left: 0, top: 0, scale: 1 });
    assert.equal(ascii.kind, "ok");

    const unsupported = modules.rasterizer.rasterizeGuiPreviewCommand(buffer, {
        kind: "text-run",
        origin: { x: 0, y: 0 },
        text: "日本語",
        color: modules.commands.guiPreviewRgb(255, 255, 255),
        size: 14,
        align: "left",
    }, { left: 0, top: 0, scale: 1 });
    assert.equal(unsupported.kind, "err");
    assert.equal(unsupported.error.kind, "unsupported-scalar");

    const zeroRectBuffer = modules.buffer.createGuiPreviewBitmapBuffer(4, 4, modules.commands.guiPreviewRgb(0, 0, 0));
    const zeroRect = modules.rasterizer.rasterizeGuiPreviewCommand(zeroRectBuffer, {
        kind: "fill-rect",
        rect: { x: 0, y: 0, width: 0, height: 4 },
        color: modules.commands.guiPreviewRgb(255, 0, 0),
    }, { left: 0, top: 0, scale: 1 });
    assert.equal(zeroRect.kind, "ok");
    assert.deepEqual(Array.from(zeroRectBuffer.pixels.slice(0, 4)), [0, 0, 0, 255]);

    const invalidRect = modules.rasterizer.rasterizeGuiPreviewCommand(zeroRectBuffer, {
        kind: "fill-rect",
        rect: { x: 0, y: 0, width: -1, height: 4 },
        color: modules.commands.guiPreviewRgb(255, 0, 0),
    }, { left: 0, top: 0, scale: 1 });
    assert.equal(invalidRect.kind, "err");
    assert.equal(invalidRect.error.kind, "invalid-geometry");

    const fakeCanvas = renderWithFakeCanvas(modules);
    assert.equal(fakeCanvas.rendered.kind, "ok");
    assert.equal(fakeCanvas.putImageDataCalls, 1);
    assert.equal(fakeCanvas.setTransformCalls, 0);
    const unsupportedCanvas = renderUnsupportedTextWithFakeCanvas(modules);
    assert.equal(unsupportedCanvas.rendered.kind, "err");
    assert.equal(unsupportedCanvas.putImageDataCalls, 0);

    return {
        ok: true,
        checks: [
            "old TS GUI example renderer is removed",
            "Web GUI renderer accepts only typed host command frames including rgba row payloads",
            "Web GUI renderer rasterizes into a bitmap buffer before putImageData presentation",
            "Web GUI bitmap text returns typed unsupported errors instead of replacement glyph fallback",
            "Web GUI zero-size fill rectangles do not draw and invalid geometry returns typed errors",
            "Web GUI canvas presentation is putImageData-only at runtime",
            "Web GUI renderer leaves frame title to the floating window titlebar",
            "Web GUI visible canvas does not use direct Canvas2D drawing primitives",
            "Web GUI panel no longer simulates NEPL examples",
            "workspace panel layout no longer exposes GUI preview panes",
            "Web GUI panel keeps debug/status text out of the window content",
        ],
    };
}

if (require.main === module) {
    (async () => {
        const result = await runWebGuiPreviewRendererRegression();
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    })().catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

function renderWithFakeCanvas(modules) {
    const previousImageData = globalThis.ImageData;
    globalThis.ImageData = class FakeImageData {
        constructor(data, width, height) {
            this.data = data;
            this.width = width;
            this.height = height;
        }
    };
    let putImageDataCalls = 0;
    let setTransformCalls = 0;
    const ctx = {
        canvas: { width: 64, height: 48 },
        putImageData() {
            putImageDataCalls += 1;
        },
        setTransform() {
            setTransformCalls += 1;
            throw new Error("setTransform must not be used for GUI presentation");
        },
        fillRect() {
            throw new Error("fillRect must not be used for GUI presentation");
        },
        fillText() {
            throw new Error("fillText must not be used for GUI presentation");
        },
        stroke() {
            throw new Error("stroke must not be used for GUI presentation");
        },
        drawImage() {
            throw new Error("drawImage must not be used for GUI presentation");
        },
    };
    try {
        const rendered = modules.canvasRenderer.renderGuiPreviewFrameToCanvas(ctx, {
            title: "Runtime Canvas Test",
            width: 32,
            height: 24,
            commands: [
                {
                    kind: "fill-rect",
                    rect: { x: 0, y: 0, width: 32, height: 24 },
                    color: modules.commands.guiPreviewRgb(16, 24, 32),
                },
                {
                    kind: "text-run",
                    origin: { x: 2, y: 2 },
                    text: "OK",
                    color: modules.commands.guiPreviewRgb(255, 255, 255),
                    size: 14,
                    align: "left",
                },
            ],
            inputTargets: [],
        }, 64, 48, { fontSize: 14 });
        return { rendered, putImageDataCalls, setTransformCalls };
    } finally {
        globalThis.ImageData = previousImageData;
    }
}

function renderUnsupportedTextWithFakeCanvas(modules) {
    const previousImageData = globalThis.ImageData;
    globalThis.ImageData = class FakeImageData {
        constructor(data, width, height) {
            this.data = data;
            this.width = width;
            this.height = height;
        }
    };
    let putImageDataCalls = 0;
    const ctx = {
        canvas: { width: 64, height: 48 },
        putImageData() {
            putImageDataCalls += 1;
        },
    };
    try {
        const rendered = modules.canvasRenderer.renderGuiPreviewFrameToCanvas(ctx, {
            title: "Unsupported Text Test",
            width: 32,
            height: 24,
            commands: [
                {
                    kind: "text-run",
                    origin: { x: 2, y: 2 },
                    text: "日本語",
                    color: modules.commands.guiPreviewRgb(255, 255, 255),
                    size: 14,
                    align: "left",
                },
            ],
            inputTargets: [],
        }, 64, 48, { fontSize: 14 });
        return { rendered, putImageDataCalls };
    } finally {
        globalThis.ImageData = previousImageData;
    }
}

module.exports = {
    runWebGuiPreviewRendererRegression,
};
