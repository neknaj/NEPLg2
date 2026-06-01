#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

async function loadRendererModule() {
    const modulePath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "renderer.js");
    return import(pathToFileURL(modulePath).href);
}

async function runWebGuiPreviewRendererRegression() {
    const renderer = await loadRendererModule();
    const rendererSource = readRepoFile("web", "src", "gui-preview", "renderer.ts");
    const commandSource = readRepoFile("web", "src", "gui-preview", "commands.ts");
    const canvasSource = readRepoFile("web", "src", "gui-preview", "canvas-renderer.ts");
    const panelSource = readRepoFile("web", "src", "gui-preview", "panel.ts");

    const mandelbrot = renderer.createGuiPreviewScene("mandelbrot");
    assert.equal(mandelbrot.metrics.commandCount, 64);
    assert.equal(mandelbrot.metrics.insideCount, 8);
    assert.equal(mandelbrot.commands.length, 64);
    assert.equal(mandelbrot.commands[0].kind, "fill-rect");
    assert.equal(renderer.guiPreviewKindFromPath("/examples/gui_mandelbrot.nepl"), "mandelbrot");

    const life = renderer.createGuiPreviewScene("life");
    assert.equal(life.metrics.commandCount, 25);
    assert.equal(life.metrics.liveCells, 5);
    assert.equal(life.metrics.checksum, 45);
    assert.equal(life.commands.length, 25);
    assert.equal(life.commands[0].kind, "fill-rect");
    assert.equal(renderer.guiPreviewKindFromPath("/examples/gui_life.nepl"), "life");

    const counter = renderer.createGuiPreviewScene("counter", { kind: "counter", counterValue: 1 });
    assert.equal(counter.metrics.counterValue, 1);
    assert.equal(counter.metrics.actionId, 1);
    assert.equal(counter.metrics.redrawTarget, 0);
    assert.equal(counter.metrics.commandCount, 5);
    assert.equal(counter.commands.filter((command) => command.kind === "text-run").length, 2);
    assert.equal(counter.hitTargets.length, 1);
    assert.equal(renderer.guiPreviewKindFromPath("/examples/gui_counter.nepl"), "counter");

    assert.match(commandSource, /GuiPreviewDrawCommand =[\s\S]*kind: 'fill-rect'[\s\S]*kind: 'text-run'/);
    assert.match(rendererSource, /GuiPreviewMetrics =[\s\S]*kind: 'mandelbrot'[\s\S]*kind: 'life'[\s\S]*kind: 'counter'/);
    assert.match(panelSource, /renderGuiPreviewSceneToCanvas/);
    assert.doesNotMatch(panelSource, /scene\.rects|scene\.texts/);
    for (const [name, source] of [
        ["commands.ts", commandSource],
        ["renderer.ts", rendererSource],
    ]) {
        assert.doesNotMatch(source, /CanvasRenderingContext2D|CanvasTextAlign|HTMLCanvasElement|document\.|window\./, `${name} must not depend on Canvas or DOM types`);
        assert.doesNotMatch(source, /\|\s*null|\|\s*undefined/, `${name} must not accept null or undefined in preview core DTOs`);
        assert.doesNotMatch(source, /\?:/, `${name} must not model command or metric absence with optional fields`);
    }
    assert.match(canvasSource, /CanvasRenderingContext2D/);
    assert.match(canvasSource, /guiPreviewCanvasColor/);

    return {
        ok: true,
        checks: [
            "Mandelbrot preview metrics match the NEPL GUI example contract",
            "Life preview metrics match the NEPL GUI example contract",
            "Counter preview keeps ActionId and redraw target metrics explicit",
            "Web preview scene uses typed DrawCommand DTOs before Canvas rendering",
            "Canvas and DOM types stay in the Web backend adapter, not the preview command model",
        ],
    };
}

if (require.main === module) {
    runWebGuiPreviewRendererRegression()
        .then((result) => process.stdout.write(JSON.stringify(result, null, 2) + "\n"))
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runWebGuiPreviewRendererRegression,
};
