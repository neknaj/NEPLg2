#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

async function loadRuntimeBridgeModule() {
    const modulePath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "runtime-bridge.js");
    return import(pathToFileURL(modulePath).href);
}

async function runWebGuiRuntimeBridgeRegression() {
    const runtimeBridge = await loadRuntimeBridgeModule();
    const runtimeBridgeSource = readRepoFile("web", "src", "gui-preview", "runtime-bridge.ts");
    const panelManagerSource = readRepoFile("web", "src", "workspace", "panel-manager.ts");
    runtimeBridge.resetGuiWebRuntimeFrameStore();

    const validFrame = {
        windowId: 11,
        title: "Runtime GUI Frame",
        width: 96,
        height: 48,
        commands: [
            {
                kind: "fill-rect",
                rect: { x: 0, y: 0, width: 96, height: 48 },
                color: { kind: "rgba8888", red: 8, green: 14, blue: 22, alpha: 255 },
            },
        ],
    };

    const missingPresenter = runtimeBridge.presentGuiWebRuntimeFrame(validFrame);
    assert.equal(missingPresenter.kind, "err");
    assert.equal(missingPresenter.error.kind, "presenter-missing");
    assert.equal(missingPresenter.error.path, "$");
    const missingVideoMemoryPresenter = runtimeBridge.presentGuiWebRuntimeVideoMemory({
        windowId: 16,
        title: "Video Memory",
        buffer: new SharedArrayBuffer(64),
    });
    assert.equal(missingVideoMemoryPresenter.kind, "err");
    assert.equal(missingVideoMemoryPresenter.error.kind, "presenter-missing");

    const receivedFrames = [];
    const receivedVideoMemoryFrames = [];
    const closedWindowIds = [];
    const presenter = {
        presentHostFrame(input) {
            receivedFrames.push(input);
            return { kind: "ok", value: "gui-window-runtime" };
        },
        presentVideoMemorySurface(input) {
            receivedVideoMemoryFrames.push(input);
            return { kind: "ok", value: "gui-window-video-memory" };
        },
        closeHostFrameWindow(windowId) {
            closedWindowIds.push(windowId);
            return { kind: "ok", value: `closed:${windowId}` };
        },
    };
    const bridge = runtimeBridge.registerGuiWebRuntimePresenter(presenter);
    assert.equal(bridge.kind, "gui-runtime-bridge");
    assert.equal(runtimeBridge.getGuiWebRuntimePresenterState().kind, "mounted");

    const presented = runtimeBridge.presentGuiWebRuntimeFrame(validFrame);
    assert.equal(presented.kind, "ok");
    assert.equal(presented.value, "gui-window-runtime");
    assert.equal(receivedFrames.length, 1);
    assert.equal(receivedFrames[0], validFrame);

    const target = {};
    const installed = runtimeBridge.installGuiWebRuntimeBridge(target);
    assert.equal(installed.kind, "ok");
    assert.equal(target.neplGuiHost.kind, "gui-runtime-bridge");
    assert.equal(typeof target.neplGuiHost.closeWindow, "function");
    assert.equal(typeof target.neplGuiHost.presentVideoMemory, "function");
    assert.equal(typeof target.neplGuiHost.takeInputEvents, "function");
    assert.equal(typeof target.neplGuiHost.resetInputEvents, "function");
    const resetInput = target.neplGuiHost.resetInputEvents();
    assert.equal(resetInput.kind, "ok");
    const emptyInput = target.neplGuiHost.takeInputEvents();
    assert.equal(emptyInput.kind, "ok");
    assert.equal(emptyInput.value.length, 0);
    const globalPresented = target.neplGuiHost.presentCommands(validFrame);
    assert.equal(globalPresented.kind, "ok");
    assert.equal(globalPresented.value, "gui-window-runtime");
    const globalClosed = target.neplGuiHost.closeWindow({ windowId: 11 });
    assert.equal(globalClosed.kind, "ok");
    assert.equal(globalClosed.value, "closed:11");
    assert.deepEqual(closedWindowIds, [11]);
    const videoMemoryBuffer = new SharedArrayBuffer(64);
    const globalVideoMemoryPresented = target.neplGuiHost.presentVideoMemory({
        windowId: 16,
        title: "Runtime Video Memory",
        buffer: videoMemoryBuffer,
    });
    assert.equal(globalVideoMemoryPresented.kind, "ok");
    assert.equal(globalVideoMemoryPresented.value, "gui-window-video-memory");
    assert.equal(receivedVideoMemoryFrames.length, 1);
    assert.equal(receivedVideoMemoryFrames[0].windowId, 16);
    assert.equal(receivedVideoMemoryFrames[0].title, "Runtime Video Memory");
    assert.equal(receivedVideoMemoryFrames[0].buffer, videoMemoryBuffer);
    const invalidVideoMemoryArrayBuffer = target.neplGuiHost.presentVideoMemory({
        windowId: 17,
        title: "Invalid Video Memory",
        buffer: new ArrayBuffer(64),
    });
    assert.equal(invalidVideoMemoryArrayBuffer.kind, "err");
    assert.equal(invalidVideoMemoryArrayBuffer.error.kind, "invalid-video-memory-frame");
    assert.equal(invalidVideoMemoryArrayBuffer.error.path, "$.buffer");
    assert.equal(invalidVideoMemoryArrayBuffer.error.actual, "ArrayBuffer");
    const invalidVideoMemoryStringHandle = target.neplGuiHost.presentVideoMemory({
        windowId: 18,
        title: "Invalid Video Memory",
        buffer: "shared-buffer-id",
    });
    assert.equal(invalidVideoMemoryStringHandle.kind, "err");
    assert.equal(invalidVideoMemoryStringHandle.error.kind, "invalid-video-memory-frame");
    assert.equal(invalidVideoMemoryStringHandle.error.path, "$.buffer");
    const invalidVideoMemoryTypedArray = target.neplGuiHost.presentVideoMemory({
        windowId: 19,
        title: "Invalid Video Memory",
        buffer: new Uint8Array(64),
    });
    assert.equal(invalidVideoMemoryTypedArray.kind, "err");
    assert.equal(invalidVideoMemoryTypedArray.error.kind, "invalid-video-memory-frame");
    assert.equal(invalidVideoMemoryTypedArray.error.path, "$.buffer");
    const invalidVideoMemoryNumericHandle = target.neplGuiHost.presentVideoMemory({
        windowId: 20,
        title: "Invalid Video Memory",
        buffer: 64,
    });
    assert.equal(invalidVideoMemoryNumericHandle.kind, "err");
    assert.equal(invalidVideoMemoryNumericHandle.error.kind, "invalid-video-memory-frame");
    assert.equal(invalidVideoMemoryNumericHandle.error.path, "$.buffer");
    const invalidVideoMemoryTransferObject = target.neplGuiHost.presentVideoMemory({
        windowId: 21,
        title: "Invalid Video Memory",
        buffer: {
            byteLength: 64,
            detached: false,
        },
    });
    assert.equal(invalidVideoMemoryTransferObject.kind, "err");
    assert.equal(invalidVideoMemoryTransferObject.error.kind, "invalid-video-memory-frame");
    assert.equal(invalidVideoMemoryTransferObject.error.path, "$.buffer");

    const beginFrame = target.neplGuiHost.beginFrame({
        windowId: 12,
        title: "Runtime Stream Frame",
        width: 88,
        height: 44,
    });
    assert.equal(beginFrame.kind, "ok");
    assert.equal(beginFrame.value, 1);
    const pushedFill = target.neplGuiHost.pushCommand({
        frameId: beginFrame.value,
        command: {
            kind: "fill-rect",
            rect: { x: 4, y: 5, width: 20, height: 12 },
            color: { kind: "rgba8888", red: 12, green: 24, blue: 36, alpha: 255 },
        },
    });
    assert.equal(pushedFill.kind, "ok");
    assert.equal(pushedFill.value, "pushed");
    const pushedText = target.neplGuiHost.pushCommand({
        frameId: beginFrame.value,
        command: {
            kind: "text-run",
            origin: { x: 10, y: 20 },
            text: "stream",
            color: { kind: "rgba8888", red: 240, green: 245, blue: 250, alpha: 255 },
            size: 13,
            align: "center",
        },
    });
    assert.equal(pushedText.kind, "ok");
    const pushedRow = target.neplGuiHost.pushCommand({
        frameId: beginFrame.value,
        command: {
            kind: "rgba-row",
            origin: { x: 0, y: 32 },
            sampleWidth: 2,
            cellWidth: 1,
            cellHeight: 1,
            pixels: [
                { kind: "rgba8888", red: 12, green: 24, blue: 36, alpha: 255 },
                { kind: "rgba8888", red: 48, green: 60, blue: 72, alpha: 255 },
            ],
        },
    });
    assert.equal(pushedRow.kind, "ok");
    const streamed = target.neplGuiHost.endFrame({ frameId: beginFrame.value });
    assert.equal(streamed.kind, "ok");
    assert.equal(streamed.value, "gui-window-runtime");
    assert.equal(receivedFrames.length, 3);
    assert.equal(receivedFrames[2].windowId, 12);
    assert.equal(receivedFrames[2].title, "Runtime Stream Frame");
    assert.equal(receivedFrames[2].commands.length, 3);
    assert.equal(receivedFrames[2].commands[0].kind, "fill-rect");
    assert.equal(receivedFrames[2].commands[1].kind, "text-run");
    assert.equal(receivedFrames[2].commands[2].kind, "rgba-row");

    const invalidFrameId = target.neplGuiHost.pushCommand({
        frameId: beginFrame.value,
        command: {
            kind: "fill-rect",
            rect: { x: 0, y: 0, width: 1, height: 1 },
            color: { kind: "rgba8888", red: 0, green: 0, blue: 0, alpha: 255 },
        },
    });
    assert.equal(invalidFrameId.kind, "err");
    assert.equal(invalidFrameId.error.kind, "invalid-frame-state");

    const discardBegin = target.neplGuiHost.beginFrame({
        windowId: 13,
        title: "Discarded",
        width: 10,
        height: 10,
    });
    assert.equal(discardBegin.kind, "ok");
    const discarded = target.neplGuiHost.discardFrame({ frameId: discardBegin.value });
    assert.equal(discarded.kind, "ok");
    assert.equal(discarded.value, "discarded");
    const endDiscarded = target.neplGuiHost.endFrame({ frameId: discardBegin.value });
    assert.equal(endDiscarded.kind, "err");
    assert.equal(endDiscarded.error.kind, "invalid-frame-state");

    const invalidStreamCommandBegin = target.neplGuiHost.beginFrame({
        windowId: 14,
        title: "Invalid Stream Command",
        width: 10,
        height: 10,
    });
    assert.equal(invalidStreamCommandBegin.kind, "ok");
    const invalidStreamCommand = target.neplGuiHost.pushCommand({
        frameId: invalidStreamCommandBegin.value,
        command: {
            kind: "fill-rect",
            rect: { x: 0, y: 0, width: 1, height: 1 },
            color: { kind: "rgba8888", red: 300, green: 0, blue: 0, alpha: 255 },
        },
    });
    assert.equal(invalidStreamCommand.kind, "err");
    assert.equal(invalidStreamCommand.error.kind, "invalid-color");
    assert.equal(invalidStreamCommand.error.path, "$.command.color.red");
    const invalidStreamDiscarded = target.neplGuiHost.discardFrame({ frameId: invalidStreamCommandBegin.value });
    assert.equal(invalidStreamDiscarded.kind, "ok");

    const retryBegin = target.neplGuiHost.beginFrame({
        windowId: 15,
        title: "Retryable End",
        width: 16,
        height: 16,
    });
    assert.equal(retryBegin.kind, "ok");
    const retryPush = target.neplGuiHost.pushCommand({
        frameId: retryBegin.value,
        command: {
            kind: "fill-rect",
            rect: { x: 0, y: 0, width: 16, height: 16 },
            color: { kind: "rgba8888", red: 1, green: 2, blue: 3, alpha: 255 },
        },
    });
    assert.equal(retryPush.kind, "ok");
    runtimeBridge.clearGuiWebRuntimePresenter(presenter);
    const missingEndPresenter = target.neplGuiHost.endFrame({ frameId: retryBegin.value });
    assert.equal(missingEndPresenter.kind, "err");
    assert.equal(missingEndPresenter.error.kind, "presenter-missing");
    runtimeBridge.registerGuiWebRuntimePresenter(presenter);
    const retryEnd = target.neplGuiHost.endFrame({ frameId: retryBegin.value });
    assert.equal(retryEnd.kind, "ok");
    assert.equal(retryEnd.value, "gui-window-runtime");

    const invalidInstallTarget = runtimeBridge.installGuiWebRuntimeBridge(1);
    assert.equal(invalidInstallTarget.kind, "err");
    assert.equal(invalidInstallTarget.error.kind, "invalid-install-target");

    const clearedState = runtimeBridge.clearGuiWebRuntimePresenter(presenter);
    assert.equal(clearedState.kind, "missing");
    const missingAfterClear = target.neplGuiHost.presentCommands(validFrame);
    assert.equal(missingAfterClear.kind, "err");
    assert.equal(missingAfterClear.error.kind, "presenter-missing");
    const videoMemoryMissingAfterClear = target.neplGuiHost.presentVideoMemory({
        windowId: 16,
        title: "Runtime Video Memory",
        buffer: videoMemoryBuffer,
    });
    assert.equal(videoMemoryMissingAfterClear.kind, "err");
    assert.equal(videoMemoryMissingAfterClear.error.kind, "presenter-missing");
    const closeMissingAfterClear = target.neplGuiHost.closeWindow({ windowId: 11 });
    assert.equal(closeMissingAfterClear.kind, "err");
    assert.equal(closeMissingAfterClear.error.kind, "presenter-missing");

    assert.match(runtimeBridgeSource, /GuiWebRuntimePresenterState =[\s\S]*kind: 'missing'[\s\S]*kind: 'mounted'/);
    assert.match(runtimeBridgeSource, /GuiWebRuntimeResult<Value> =[\s\S]*kind: 'ok'[\s\S]*kind: 'err'/);
    assert.match(runtimeBridgeSource, /presentCommands: presentGuiWebRuntimeFrame/);
    assert.match(runtimeBridgeSource, /presentVideoMemory: presentGuiWebRuntimeVideoMemory/);
    assert.match(runtimeBridgeSource, /closeWindow: closeGuiWebRuntimeHostFrameWindow/);
    assert.match(runtimeBridgeSource, /beginFrame: beginGuiWebRuntimeFrame/);
    assert.match(runtimeBridgeSource, /pushCommand: pushGuiWebRuntimeCommand/);
    assert.match(runtimeBridgeSource, /endFrame: endGuiWebRuntimeFrame/);
    assert.match(runtimeBridgeSource, /discardFrame: discardGuiWebRuntimeFrame/);
    assert.match(runtimeBridgeSource, /presentVideoMemorySurface/);
    assert.match(runtimeBridgeSource, /readSharedArrayBuffer/);
    assert.match(runtimeBridgeSource, /invalid-video-memory-frame/);
    assert.match(runtimeBridgeSource, /video-memory-open-failed/);
    assert.match(runtimeBridgeSource, /video-memory-present-failed/);
    assert.match(runtimeBridgeSource, /actualType\(value\)[\s\S]*ArrayBuffer/);
    assert.match(runtimeBridgeSource, /closeHostFrameWindow/);
    assert.match(runtimeBridgeSource, /takeInputEvents: takeGuiWebInputEvents/);
    assert.match(runtimeBridgeSource, /resetInputEvents: resetGuiWebInputEvents/);
    assert.match(runtimeBridgeSource, /GuiWebRuntimeFrameStore/);
    assert.match(runtimeBridgeSource, /decodeGuiWebHostFrame/);
    assert.match(runtimeBridgeSource, /installGuiWebRuntimeBridge/);
    assert.match(panelManagerSource, /registerGuiWebRuntimePresenter\(this\.floatingGui\)/);
    assert.match(panelManagerSource, /installGuiWebRuntimeBridge\(globalThis\)/);
    assert.doesNotMatch(runtimeBridgeSource, /\bas\b\s*any\b|:\s*any\b|<any>/);
    assert.doesNotMatch(runtimeBridgeSource, /\|\s*null|\|\s*undefined/);
    assert.doesNotMatch(runtimeBridgeSource, /throw new Error|throw\s+/);
    assert.doesNotMatch(runtimeBridgeSource, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);
    assert.doesNotMatch(runtimeBridgeSource, /postMessage|transfer|presentCommands\(frame\.value\)|presentGuiWebRuntimeFrame\(frame\.value\)/);

    return {
        ok: true,
        checks: [
            "Web GUI runtime bridge rejects present-commands before presenter registration",
            "Web GUI runtime bridge forwards frames through a typed presenter",
            "Web GUI runtime bridge installs a global neplGuiHost command surface",
            "Web GUI runtime bridge exposes SharedArrayBuffer video memory presentation",
            "Web GUI runtime bridge rejects ArrayBuffer, typed arrays, numeric handles, string handles, and transfer-like objects for video memory frames",
            "Web GUI runtime bridge closes host-frame windows through the presenter",
            "Web GUI runtime bridge supports begin/push/end streaming frames",
            "Web GUI runtime bridge streams rgba row payload commands through host decode",
            "Web GUI runtime bridge validates pushed commands through host decode logic",
            "Web GUI runtime bridge keeps DOM and Canvas types out of the runtime boundary",
            "Playground panel manager registers the floating GUI window manager as presenter",
        ],
    };
}

if (require.main === module) {
    runWebGuiRuntimeBridgeRegression()
        .then((result) => process.stdout.write(JSON.stringify(result, null, 2) + "\n"))
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runWebGuiRuntimeBridgeRegression,
};
