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
    const windowManagerSource = readRepoFile("web", "src", "gui-preview", "window-manager.ts");
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

    const queuedPointerMove = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 3,
        pointerKind: "move",
        pointerId: 5,
        button: "none",
        point: { x: 11, y: 12 },
    });
    assert.equal(queuedPointerMove.kind, "ok");
    const queuedPointerMoveLatest = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 3,
        pointerKind: "move",
        pointerId: 5,
        button: "none",
        point: { x: 13.5, y: 14.25 },
    });
    assert.equal(queuedPointerMoveLatest.kind, "ok");
    assert.equal(observed.length, 4);
    const takenPointerMove = inputBridge.takeGuiWebInputEvents();
    assert.equal(takenPointerMove.kind, "ok");
    assert.equal(takenPointerMove.value.length, 1);
    assert.equal(takenPointerMove.value[0].kind, "pointer");
    assert.equal(takenPointerMove.value[0].pointerKind, "move");
    assert.equal(takenPointerMove.value[0].point.x, 13.5);
    assert.equal(takenPointerMove.value[0].point.y, 14.25);

    const queuedPointerMoveBarrierA = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 3,
        pointerKind: "move",
        pointerId: 5,
        button: "none",
        point: { x: 21, y: 22 },
    });
    assert.equal(queuedPointerMoveBarrierA.kind, "ok");
    const queuedPointerMoveBarrierUp = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 3,
        pointerKind: "up",
        pointerId: 5,
        button: "primary",
        point: { x: 23, y: 24 },
    });
    assert.equal(queuedPointerMoveBarrierUp.kind, "ok");
    const queuedPointerMoveBarrierB = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 3,
        pointerKind: "move",
        pointerId: 5,
        button: "none",
        point: { x: 25, y: 26 },
    });
    assert.equal(queuedPointerMoveBarrierB.kind, "ok");
    const takenPointerMoveBarrier = inputBridge.takeGuiWebInputEvents();
    assert.equal(takenPointerMoveBarrier.kind, "ok");
    assert.equal(takenPointerMoveBarrier.value.length, 3);
    assert.equal(takenPointerMoveBarrier.value[0].kind, "pointer");
    assert.equal(takenPointerMoveBarrier.value[0].pointerKind, "move");
    assert.equal(takenPointerMoveBarrier.value[1].kind, "pointer");
    assert.equal(takenPointerMoveBarrier.value[1].pointerKind, "up");
    assert.equal(takenPointerMoveBarrier.value[2].kind, "pointer");
    assert.equal(takenPointerMoveBarrier.value[2].pointerKind, "move");

    const queuedPointerMoveIdentityA = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 3,
        pointerKind: "move",
        pointerId: 8,
        button: "none",
        point: { x: 31, y: 32 },
    });
    assert.equal(queuedPointerMoveIdentityA.kind, "ok");
    const queuedPointerMoveIdentityWindow = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 4,
        pointerKind: "move",
        pointerId: 8,
        button: "none",
        point: { x: 33, y: 34 },
    });
    assert.equal(queuedPointerMoveIdentityWindow.kind, "ok");
    const queuedPointerMoveIdentityButton = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 4,
        pointerKind: "move",
        pointerId: 8,
        button: "primary",
        point: { x: 35, y: 36 },
    });
    assert.equal(queuedPointerMoveIdentityButton.kind, "ok");
    const queuedPointerMoveIdentityPointer = inputBridge.queueGuiWebInputEvent({
        kind: "pointer",
        windowId: 4,
        pointerKind: "move",
        pointerId: 9,
        button: "primary",
        point: { x: 37, y: 38 },
    });
    assert.equal(queuedPointerMoveIdentityPointer.kind, "ok");
    const takenPointerMoveIdentity = inputBridge.takeGuiWebInputEvents();
    assert.equal(takenPointerMoveIdentity.kind, "ok");
    assert.equal(takenPointerMoveIdentity.value.length, 4);
    assert.equal(takenPointerMoveIdentity.value[0].kind, "pointer");
    assert.equal(takenPointerMoveIdentity.value[0].windowId, 3);
    assert.equal(takenPointerMoveIdentity.value[1].kind, "pointer");
    assert.equal(takenPointerMoveIdentity.value[1].windowId, 4);
    assert.equal(takenPointerMoveIdentity.value[2].kind, "pointer");
    assert.equal(takenPointerMoveIdentity.value[2].button, "primary");
    assert.equal(takenPointerMoveIdentity.value[3].kind, "pointer");
    assert.equal(takenPointerMoveIdentity.value[3].pointerId, 9);

    const observedBeforeKeyboard = observed.length;
    const queuedKeyboard = inputBridge.queueGuiWebInputEvent({
        kind: "keyboard",
        windowId: 3,
        keyboardKind: "down",
        keyCode: 9,
        modifierBits: 1,
    });
    assert.equal(queuedKeyboard.kind, "ok");
    assert.equal(observed.length, observedBeforeKeyboard + 1);
    assert.equal(observed[observedBeforeKeyboard].kind, "keyboard");
    assert.equal(observed[observedBeforeKeyboard].keyboardKind, "down");
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
    assert.equal(observed.length, observedBeforeKeyboard + 2);
    assert.equal(observed[observedBeforeKeyboard + 1].kind, "text-input");
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
    assert.equal(observed.length, observedBeforeKeyboard + 3);
    const takenNulTextInput = inputBridge.takeGuiWebInputEvents();
    assert.equal(takenNulTextInput.kind, "ok");
    assert.equal(takenNulTextInput.value.length, 1);
    assert.equal(takenNulTextInput.value[0].kind, "text-input");
    assert.equal(takenNulTextInput.value[0].scalarValue, 0);

    const queuedWindow = inputBridge.queueGuiWebInputEvent({
        kind: "window",
        windowId: 3,
        windowKind: "resized",
        size: { width: 640, height: 480 },
    });
    assert.equal(queuedWindow.kind, "ok");
    assert.equal(observed.length, observedBeforeKeyboard + 4);
    assert.equal(observed[observedBeforeKeyboard + 3].kind, "window");
    assert.equal(observed[observedBeforeKeyboard + 3].windowKind, "resized");
    const takenWindow = inputBridge.takeGuiWebInputEvents();
    assert.equal(takenWindow.kind, "ok");
    assert.equal(takenWindow.value.length, 1);
    assert.equal(takenWindow.value[0].kind, "window");
    assert.equal(takenWindow.value[0].windowId, 3);
    assert.equal(takenWindow.value[0].windowKind, "resized");
    assert.equal(takenWindow.value[0].size.width, 640);
    assert.equal(takenWindow.value[0].size.height, 480);

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

    const invalidWindowKind = inputBridge.queueGuiWebInputEvent({
        kind: "window",
        windowId: 3,
        windowKind: "moved",
        size: { width: 640, height: 480 },
    });
    assert.equal(invalidWindowKind.kind, "err");
    assert.equal(invalidWindowKind.error.kind, "invalid-window-event");
    assert.equal(invalidWindowKind.error.path, "$.windowKind");

    const invalidWindowWidth = inputBridge.queueGuiWebInputEvent({
        kind: "window",
        windowId: 3,
        windowKind: "resized",
        size: { width: 0, height: 480 },
    });
    assert.equal(invalidWindowWidth.kind, "err");
    assert.equal(invalidWindowWidth.error.kind, "invalid-window-event");
    assert.equal(invalidWindowWidth.error.path, "$.size.width");

    const invalidWindowHeight = inputBridge.queueGuiWebInputEvent({
        kind: "window",
        windowId: 3,
        windowKind: "resized",
        size: { width: 640, height: 480.5 },
    });
    assert.equal(invalidWindowHeight.kind, "err");
    assert.equal(invalidWindowHeight.error.kind, "invalid-window-event");
    assert.equal(invalidWindowHeight.error.path, "$.size.height");

    assert.match(inputBridgeSource, /GuiWebInputEvent =[\s\S]*kind: 'action'/);
    assert.match(inputBridgeSource, /GuiWebInputEvent =[\s\S]*kind: 'pointer'/);
    assert.match(inputBridgeSource, /GuiWebInputEvent =[\s\S]*kind: 'keyboard'/);
    assert.match(inputBridgeSource, /GuiWebInputEvent =[\s\S]*kind: 'text-input'/);
    assert.match(inputBridgeSource, /GuiWebInputEvent =[\s\S]*kind: 'window'/);
    assert.match(inputBridgeSource, /GuiWebInputResult<Value> =[\s\S]*kind: 'ok'[\s\S]*kind: 'err'/);
    assert.match(inputBridgeSource, /decodeGuiWebInputEvent/);
    assert.match(inputBridgeSource, /decodeGuiWebWindowInputEvent/);
    assert.match(inputBridgeSource, /readWindowKind/);
    assert.match(inputBridgeSource, /readSize/);
    assert.match(inputBridgeSource, /invalid-window-event/);
    assert.match(inputBridgeSource, /guiWebInputEventsWithPointerMove/);
    assert.match(inputBridgeSource, /isUnicodeScalarValue/);
    assert.match(inputBridgeSource, /registerGuiWebInputEventListener/);
    assert.match(panelSource, /queueGuiWebInputEvent/);
    assert.match(panelSource, /handleCanvasPointerDown/);
    assert.match(panelSource, /pointermove/);
    assert.match(panelSource, /handleCanvasPointerMove/);
    assert.match(panelSource, /queueHostPointerMoveEvent/);
    assert.match(panelSource, /requestAnimationFrame/);
    assert.match(panelSource, /flushHostPointerMoveEvent/);
    assert.match(panelSource, /queueHostPointerEvent[\s\S]*this\.flushHostPointerMoveEvent\(\);[\s\S]*queueGuiWebInputEvent/);
    assert.match(panelSource, /dispose\(\)[\s\S]*this\.hostPointerMove = \{ kind: 'idle' \};/);
    assert.match(panelSource, /guiWebPointerButtonFromDomButton/);
    assert.match(panelSource, /handleCanvasKeyDown/);
    assert.match(panelSource, /handleCanvasKeyUp/);
    assert.match(panelSource, /guiWebSingleScalarFromDomKey/);
    assert.match(panelSource, /event\.isComposing/);
    assert.match(panelSource, /event\.metaKey/);
    assert.match(panelSource, /queueHostKeyboardEvent[\s\S]*event\.metaKey/);
    assert.match(windowManagerSource, /queueGuiWebInputEvent/);
    assert.match(windowManagerSource, /queueHostWindowEvent/);
    assert.match(windowManagerSource, /source\.kind !== 'host-frame'/);
    assert.match(windowManagerSource, /'close-requested'/);
    assert.match(windowManagerSource, /previousWidth !== next\.width \|\| previousHeight !== next\.height/);
    assert.match(windowManagerSource, /this\.queueHostWindowEvent\(windowState, 'resized'\)/);
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
            "Web GUI input bridge stores only the latest consecutive pointer move",
            "Web GUI input bridge preserves pointer move ordering barriers and identity boundaries",
            "Web GUI panel flushes pending pointer move before immediate barrier events",
            "Web GUI input bridge queues keyboard events as typed values",
            "Web GUI input bridge queues Unicode scalar text input events as typed values",
            "Web GUI input bridge queues window events as typed values",
            "Web GUI floating windows publish host-frame resize and close requests through the input queue",
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
