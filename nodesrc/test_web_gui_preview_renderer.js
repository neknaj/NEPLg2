#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

async function loadRendererModule() {
    const modulePath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "renderer.js");
    return import(pathToFileURL(modulePath).href);
}

async function runWebGuiPreviewRendererRegression() {
    const renderer = await loadRendererModule();

    const mandelbrot = renderer.createGuiPreviewScene("mandelbrot");
    assert.equal(mandelbrot.metrics.commandCount, 64);
    assert.equal(mandelbrot.metrics.insideCount, 8);
    assert.equal(renderer.guiPreviewKindFromPath("/examples/gui_mandelbrot.nepl"), "mandelbrot");

    const life = renderer.createGuiPreviewScene("life");
    assert.equal(life.metrics.commandCount, 25);
    assert.equal(life.metrics.liveCells, 5);
    assert.equal(life.metrics.checksum, 45);
    assert.equal(renderer.guiPreviewKindFromPath("/examples/gui_life.nepl"), "life");

    const counter = renderer.createGuiPreviewScene("counter", { counterValue: 3 });
    assert.equal(counter.metrics.counterValue, 3);
    assert.equal(counter.metrics.actionId, 1);
    assert.equal(counter.metrics.redrawTarget, 0);
    assert.equal(counter.hitTargets.length, 1);
    assert.equal(renderer.guiPreviewKindFromPath("/examples/gui_counter.nepl"), "counter");

    return {
        ok: true,
        checks: [
            "Mandelbrot preview metrics match the NEPL GUI example contract",
            "Life preview metrics match the NEPL GUI example contract",
            "Counter preview keeps ActionId and redraw target metrics explicit",
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
