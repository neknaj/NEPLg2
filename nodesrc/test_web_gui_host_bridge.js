#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

async function loadHostBridgeModule() {
    const modulePath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "host-bridge.js");
    return import(pathToFileURL(modulePath).href);
}

async function runWebGuiHostBridgeRegression() {
    const hostBridge = await loadHostBridgeModule();
    const hostBridgeSource = readRepoFile("web", "src", "gui-preview", "host-bridge.ts");
    const panelSource = readRepoFile("web", "src", "gui-preview", "panel.ts");
    const managerSource = readRepoFile("web", "src", "gui-preview", "window-manager.ts");
    const canvasSource = readRepoFile("web", "src", "gui-preview", "canvas-renderer.ts");

    const validFrame = {
        windowId: 7,
        title: "Host GUI Frame",
        width: 120,
        height: 80,
        commands: [
            {
                kind: "fill-rect",
                rect: { x: 0, y: 0, width: 120, height: 80 },
                color: { kind: "rgba8888", red: 16, green: 24, blue: 32, alpha: 255 },
            },
            {
                kind: "text-run",
                origin: { x: 12, y: 10 },
                text: "host",
                color: { kind: "rgba8888", red: 242, green: 247, blue: 245, alpha: 255 },
                size: 14,
                align: "left",
            },
            {
                kind: "rgba-row",
                origin: { x: 0, y: 32 },
                sampleWidth: 2,
                cellWidth: 1,
                cellHeight: 1,
                pixels: [
                    { kind: "rgba8888", red: 1, green: 2, blue: 3, alpha: 255 },
                    { kind: "rgba8888", red: 4, green: 5, blue: 6, alpha: 255 },
                ],
            },
        ],
        inputTargets: [
            {
                kind: "action-rect",
                rect: { x: 8, y: 20, width: 60, height: 24 },
                actionId: 3,
            },
        ],
    };

    const decoded = hostBridge.decodeGuiWebHostPresentedFrame(validFrame);
    assert.equal(decoded.kind, "ok");
    assert.equal(decoded.value.windowId, 7);
    assert.equal(decoded.value.frame.commands.length, 3);
    assert.equal(decoded.value.frame.inputTargets.length, 1);
    assert.equal(decoded.value.frame.commands[0].kind, "fill-rect");
    assert.equal(decoded.value.frame.commands[1].kind, "text-run");
    assert.equal(decoded.value.frame.commands[2].kind, "rgba-row");
    assert.equal(decoded.value.frame.commands[2].pixels.length, 2);
    assert.equal(decoded.value.frame.inputTargets[0].kind, "action-rect");
    assert.equal(decoded.value.frame.inputTargets[0].actionId, 3);

    const invalidColor = hostBridge.decodeGuiWebHostPresentedFrame({
        ...validFrame,
        commands: [
            {
                kind: "fill-rect",
                rect: { x: 0, y: 0, width: 1, height: 1 },
                color: { kind: "rgba8888", red: 256, green: 0, blue: 0, alpha: 255 },
            },
        ],
    });
    assert.equal(invalidColor.kind, "err");
    assert.equal(invalidColor.error.kind, "invalid-color");
    assert.equal(invalidColor.error.path, "$.commands.0.color.red");

    const unsupportedCommand = hostBridge.decodeGuiWebHostPresentedFrame({
        ...validFrame,
        commands: [
            {
                kind: "draw-image",
                rect: { x: 0, y: 0, width: 1, height: 1 },
            },
        ],
    });
    assert.equal(unsupportedCommand.kind, "err");
    assert.equal(unsupportedCommand.error.kind, "unsupported-command");

    const invalidRgbaRow = hostBridge.decodeGuiWebHostPresentedFrame({
        ...validFrame,
        commands: [
            {
                kind: "rgba-row",
                origin: { x: 0, y: 0 },
                sampleWidth: 2,
                cellWidth: 1,
                cellHeight: 1,
                pixels: [
                    { kind: "rgba8888", red: 1, green: 2, blue: 3, alpha: 255 },
                ],
            },
        ],
    });
    assert.equal(invalidRgbaRow.kind, "err");
    assert.equal(invalidRgbaRow.error.kind, "invalid-command");
    assert.equal(invalidRgbaRow.error.path, "$.commands.0.pixels");

    const invalidInputTarget = hostBridge.decodeGuiWebHostPresentedFrame({
        ...validFrame,
        inputTargets: [
            {
                kind: "action-rect",
                rect: { x: 0, y: 0, width: 1, height: 1 },
                actionId: 0,
            },
        ],
    });
    assert.equal(invalidInputTarget.kind, "err");
    assert.equal(invalidInputTarget.error.kind, "invalid-input-target");
    assert.equal(invalidInputTarget.error.path, "$.inputTargets.0.actionId");

    assert.match(hostBridgeSource, /GuiWebHostResult<Value> =[\s\S]*kind: 'ok'[\s\S]*kind: 'err'/);
    assert.match(hostBridgeSource, /decodeGuiWebHostPresentedFrame/);
    assert.match(managerSource, /presentHostFrame\(input: unknown\): GuiWebHostResult<string>/);
    assert.match(panelSource, /presentHostFrame\(frame: GuiPreviewCommandFrame, windowId: number\)/);
    assert.match(panelSource, /queueGuiWebInputEvent/);
    assert.match(canvasSource, /renderGuiPreviewFrameToCanvas/);
    assert.doesNotMatch(hostBridgeSource, /\bas\b\s*any\b|:\s*any\b|<any>/);
    assert.doesNotMatch(hostBridgeSource, /\|\s*null|\|\s*undefined/);
    assert.doesNotMatch(hostBridgeSource, /throw new Error|throw\s+/);
    assert.doesNotMatch(hostBridgeSource, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);

    return {
        ok: true,
        checks: [
            "Web GUI host bridge decodes valid present-commands frames",
            "Web GUI host bridge decodes rgba row payload commands",
            "Web GUI host bridge rejects invalid color bytes with typed errors",
            "Web GUI host bridge decodes action hit targets as input metadata",
            "Web GUI host bridge rejects unsupported command variants",
            "Floating GUI windows expose a typed presentHostFrame boundary",
            "Host bridge keeps DOM and Canvas types out of decode logic",
        ],
    };
}

if (require.main === module) {
    runWebGuiHostBridgeRegression()
        .then((result) => process.stdout.write(JSON.stringify(result, null, 2) + "\n"))
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runWebGuiHostBridgeRegression,
};
