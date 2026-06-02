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

    const receivedFrames = [];
    const presenter = {
        presentHostFrame(input) {
            receivedFrames.push(input);
            return { kind: "ok", value: "gui-window-runtime" };
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
    const globalPresented = target.neplGuiHost.presentCommands(validFrame);
    assert.equal(globalPresented.kind, "ok");
    assert.equal(globalPresented.value, "gui-window-runtime");

    const invalidInstallTarget = runtimeBridge.installGuiWebRuntimeBridge(1);
    assert.equal(invalidInstallTarget.kind, "err");
    assert.equal(invalidInstallTarget.error.kind, "invalid-install-target");

    const clearedState = runtimeBridge.clearGuiWebRuntimePresenter(presenter);
    assert.equal(clearedState.kind, "missing");
    const missingAfterClear = target.neplGuiHost.presentCommands(validFrame);
    assert.equal(missingAfterClear.kind, "err");
    assert.equal(missingAfterClear.error.kind, "presenter-missing");

    assert.match(runtimeBridgeSource, /GuiWebRuntimePresenterState =[\s\S]*kind: 'missing'[\s\S]*kind: 'mounted'/);
    assert.match(runtimeBridgeSource, /GuiWebRuntimeResult<Value> =[\s\S]*kind: 'ok'[\s\S]*kind: 'err'/);
    assert.match(runtimeBridgeSource, /presentCommands: presentGuiWebRuntimeFrame/);
    assert.match(runtimeBridgeSource, /installGuiWebRuntimeBridge/);
    assert.match(panelManagerSource, /registerGuiWebRuntimePresenter\(this\.floatingGui\)/);
    assert.match(panelManagerSource, /installGuiWebRuntimeBridge\(globalThis\)/);
    assert.doesNotMatch(runtimeBridgeSource, /\bas\b\s*any\b|:\s*any\b|<any>/);
    assert.doesNotMatch(runtimeBridgeSource, /\|\s*null|\|\s*undefined/);
    assert.doesNotMatch(runtimeBridgeSource, /throw new Error|throw\s+/);
    assert.doesNotMatch(runtimeBridgeSource, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);

    return {
        ok: true,
        checks: [
            "Web GUI runtime bridge rejects present-commands before presenter registration",
            "Web GUI runtime bridge forwards frames through a typed presenter",
            "Web GUI runtime bridge installs a global neplGuiHost command surface",
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
