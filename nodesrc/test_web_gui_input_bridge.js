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

    const queuedPointer = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 3,
        pointerKind: "down",
        pointerId: 5,
        button: "primary",
        point: { x: 10, y: 12 },
    });
    assert.equal(queuedPointer.kind, "ok");
    assert.equal(observed.length, 2);
    assert.equal(observed[1].kind, "pointer");
    assert.equal(observed[1].pointerKind, "down");
    const takenPointer = inputBridge.takeGuiWebInputEvents();
    assert.equal(takenPointer.kind, "ok");
    assert.equal(takenPointer.value.length, 1);
    assert.equal(takenPointer.value[0].kind, "pointer");
    assert.equal(takenPointer.value[0].windowId, 3);
    assert.equal(takenPointer.value[0].pointerId, 5);
    assert.equal(takenPointer.value[0].button, "primary");

    const queuedKeyboard = inputBridge.queueGuiWebInputEvent({
        kind: "keyboard",
        windowId: 3,
        keyboardKind: "down",
        keyCode: 9,
        modifierBits: 1,
    });
    assert.equal(queuedKeyboard.kind, "ok");
    assert.equal(observed.length, 3);
    assert.equal(observed[2].kind, "keyboard");
    assert.equal(observed[2].keyboardKind, "down");
    const takenKeyboard = inputBridge.takeGuiWebInputEvents();
    assert.equal(takenKeyboard.kind, "ok");
    assert.equal(takenKeyboard.value.length, 1);
    assert.equal(takenKeyboard.value[0].kind, "keyboard");
    assert.equal(takenKeyboard.value[0].windowId, 3);
    assert.equal(takenKeyboard.value[0].keyCode, 9);
    assert.equal(takenKeyboard.value[0].modifierBits, 1);

    const queuedTextInput = inputBridge.queueGuiWebInputEvent({
        kind: "text-input",
        windowId: 3,
        scalarValue: 0x3042,
    });
    assert.equal(queuedTextInput.kind, "ok");
    assert.equal(observed.length, 4);
    assert.equal(observed[3].kind, "text-input");
    const takenTextInput = inputBridge.takeGuiWebInputEvents();
    assert.equal(takenTextInput.kind, "ok");
    assert.equal(takenTextInput.value.length, 1);
    assert.equal(takenTextInput.value[0].kind, "text-input");
    assert.equal(takenTextInput.value[0].windowId, 3);
    assert.equal(takenTextInput.value[0].scalarValue, 0x3042);

    const queuedNulTextInput = inputBridge.queueGuiWebInputEvent({
        kind: "text-input",
        windowId: 3,
        scalarValue: 0,
    });
    assert.equal(queuedNulTextInput.kind, "ok");
    assert.equal(observed.length, 5);
    const takenNulTextInput = inputBridge.takeGuiWebInputEvents();
    assert.equal(takenNulTextInput.kind, "ok");
    assert.equal(takenNulTextInput.value.length, 1);
    assert.equal(takenNulTextInput.value[0].kind, "text-input");
    assert.equal(takenNulTextInput.value[0].scalarValue, 0);

    const invalidAction = inputBridge.queueGuiWebInputEvent({
        kind: "action",
        windowId: 3,
        actionId: 0,
        point: { x: 1, y: 1 },
    });
    assert.equal(invalidAction.kind, "err");
    assert.equal(invalidAction.error.kind, "invalid-action-event");
    assert.equal(invalidAction.error.path, "$.actionId");

    const invalidPointer = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 3,
        pointerKind: "drag",
        pointerId: 5,
        button: "primary",
        point: { x: 1, y: 1 },
    });
    assert.equal(invalidPointer.kind, "err");
    assert.equal(invalidPointer.error.kind, "invalid-pointer-event");
    assert.equal(invalidPointer.error.path, "$.pointerKind");

    const invalidKeyboard = inputBridge.queueGuiWebInputEvent({
        kind: "keyboard",
        windowId: 3,
        keyboardKind: "press",
        keyCode: 9,
        modifierBits: 0,
    });
    assert.equal(invalidKeyboard.kind, "err");
    assert.equal(invalidKeyboard.error.kind, "invalid-keyboard-event");
    assert.equal(invalidKeyboard.error.path, "$.keyboardKind");

    const invalidTextScalar = inputBridge.queueGuiWebInputEvent({
        kind: "text-input",
        windowId: 3,
        scalarValue: 0xD800,
    });
    assert.equal(invalidTextScalar.kind, "err");
    assert.equal(invalidTextScalar.error.kind, "invalid-text-input-event");
    assert.equal(invalidTextScalar.error.path, "$.scalarValue");

    assert.match(inputBridgeSource, /GuiWebInputEvent =[\s\S]*kind: 'action'/);
    assert.match(inputBridgeSource, /GuiWebInputEvent =[\s\S]*kind: 'pointer'/);
    assert.match(inputBridgeSource, /GuiWebInputEvent =[\s\S]*kind: 'keyboard'/);
    assert.match(inputBridgeSource, /GuiWebInputEvent =[\s\S]*kind: 'text-input'/);
    assert.match(inputBridgeSource, /GuiWebInputResult<Value> =[\s\S]*kind: 'ok'[\s\S]*kind: 'err'/);
    assert.match(inputBridgeSource, /decodeGuiWebInputEvent/);
    assert.match(inputBridgeSource, /isUnicodeScalarValue/);
    assert.match(inputBridgeSource, /registerGuiWebInputEventListener/);
    assert.match(panelSource, /queueGuiWebInputEvent/);
    assert.match(panelSource, /handleCanvasPointerDown/);
    assert.match(panelSource, /guiWebPointerButtonFromDomButton/);
    assert.match(panelSource, /handleCanvasKeyDown/);
    assert.match(panelSource, /handleCanvasKeyUp/);
    assert.match(panelSource, /guiWebSingleScalarFromDomKey/);
    assert.match(panelSource, /event\.isComposing/);
    assert.match(panelSource, /event\.metaKey/);
    assert.match(panelSource, /queueHostKeyboardEvent[\s\S]*event\.metaKey/);
    assert.match(commandsSource, /GuiPreviewInputTarget/);
    assert.doesNotMatch(inputBridgeSource, /\bas\b\s*any\b|:\s*any\b|<any>/);
    assert.doesNotMatch(inputBridgeSource, /\|\s*null|\|\s*undefined/);
    assert.doesNotMatch(inputBridgeSource, /throw new Error|throw\s+/);
    assert.doesNotMatch(inputBridgeSource, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);

    return {
        ok: true,
        checks: [
            "Web GUI input bridge queues action events as typed values",
            "Web GUI input bridge queues pointer events as typed values",
            "Web GUI input bridge queues keyboard events as typed values",
            "Web GUI input bridge queues Unicode scalar text input events as typed values",
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
