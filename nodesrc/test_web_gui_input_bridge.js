#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

async function loadInputBridgeModule() {
    const modulePath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "input-bridge.js");
    return import(pathToFileURL(modulePath).href);
}

async function runWebGuiInputBridgeRegression() {
    const inputBridge = await loadInputBridgeModule();
    const inputBridgeSource = readRepoFile("web", "src", "gui-preview", "input-bridge.ts");
    const panelSource = readRepoFile("web", "src", "gui-preview", "panel.ts");
    const commandsSource = readRepoFile("web", "src", "gui-preview", "commands.ts");

    const reset = inputBridge.resetGuiWebInputEvents();
    assert.equal(reset.kind, "ok");
    assert.equal(reset.value, "reset");
    const cleared = inputBridge.clearGuiWebInputEventListeners();
    assert.equal(cleared.kind, "ok");
    assert.equal(cleared.value, "cleared");

    const observed = [];
    const registered = inputBridge.registerGuiWebInputEventListener({
        kind: "gui-input-listener",
        onInputEvent: (event) => observed.push(event),
    });
    assert.equal(registered.kind, "ok");
    assert.equal(registered.value, "registered");

    const queued = inputBridge.queueGuiWebInputEvent({
        kind: "action",
        windowId: 3,
        actionId: 7,
        point: { x: 18.5, y: 20.25 },
    });
    assert.equal(queued.kind, "ok");
    assert.equal(queued.value, "queued");
    assert.equal(observed.length, 1);
    assert.equal(observed[0].actionId, 7);

    const taken = inputBridge.takeGuiWebInputEvents();
    assert.equal(taken.kind, "ok");
    assert.equal(taken.value.length, 1);
    assert.equal(taken.value[0].kind, "action");
    assert.equal(taken.value[0].windowId, 3);
    assert.equal(taken.value[0].actionId, 7);
    assert.equal(taken.value[0].point.x, 18.5);

    const takenAgain = inputBridge.takeGuiWebInputEvents();
    assert.equal(takenAgain.kind, "ok");
    assert.equal(takenAgain.value.length, 0);

    const invalidAction = inputBridge.queueGuiWebInputEvent({
        kind: "action",
        windowId: 3,
        actionId: 0,
        point: { x: 1, y: 1 },
    });
    assert.equal(invalidAction.kind, "err");
    assert.equal(invalidAction.error.kind, "invalid-action-event");
    assert.equal(invalidAction.error.path, "$.actionId");

    assert.match(inputBridgeSource, /GuiWebInputEvent =[\s\S]*kind: 'action'/);
    assert.match(inputBridgeSource, /GuiWebInputResult<Value> =[\s\S]*kind: 'ok'[\s\S]*kind: 'err'/);
    assert.match(inputBridgeSource, /decodeGuiWebInputEvent/);
    assert.match(inputBridgeSource, /registerGuiWebInputEventListener/);
    assert.match(panelSource, /queueGuiWebInputEvent/);
    assert.match(commandsSource, /GuiPreviewInputTarget/);
    assert.doesNotMatch(inputBridgeSource, /\bas\b\s*any\b|:\s*any\b|<any>/);
    assert.doesNotMatch(inputBridgeSource, /\|\s*null|\|\s*undefined/);
    assert.doesNotMatch(inputBridgeSource, /throw new Error|throw\s+/);
    assert.doesNotMatch(inputBridgeSource, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);

    return {
        ok: true,
        checks: [
            "Web GUI input bridge queues action events as typed values",
            "Web GUI input bridge notifies typed listeners without app-state simulation",
            "Web GUI input bridge exposes take/reset event boundaries",
            "Web GUI input bridge keeps DOM and Canvas types out of the input queue",
        ],
    };
}

if (require.main === module) {
    runWebGuiInputBridgeRegression()
        .then((result) => process.stdout.write(JSON.stringify(result, null, 2) + "\n"))
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runWebGuiInputBridgeRegression,
};
