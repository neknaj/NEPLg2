#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

async function loadSharedEventQueueModule() {
    const modulePath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "shared-event-queue.js");
    return import(pathToFileURL(modulePath).href);
}

async function runWebGuiSharedEventQueueRegression() {
    const sharedQueue = await loadSharedEventQueueModule();
    const created = sharedQueue.createGuiWebSharedEventBuffer();
    assert.equal(created.kind, "ok");
    const reset = sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    assert.equal(reset.kind, "ok");

    const queued = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "action",
        windowId: 3,
        actionId: 11,
        point: { x: 12.25, y: 9.5 },
    });
    assert.equal(queued.kind, "ok");
    assert.equal(queued.value, "queued");
    assert.equal(sharedQueue.takeGuiWebSharedActionId(created.value), 11);
    assert.equal(sharedQueue.takeGuiWebSharedActionId(created.value), 0);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    for (let i = 1; i < sharedQueue.GUI_WEB_EVENT_QUEUE_CAPACITY; i++) {
        const result = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
            kind: "action",
            windowId: 3,
            actionId: 100 + i,
            point: { x: i, y: i },
        });
        assert.equal(result.kind, "ok");
    }
    const overflow = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "action",
        windowId: 3,
        actionId: 999,
        point: { x: 0, y: 0 },
    });
    assert.equal(overflow.kind, "err");
    assert.equal(overflow.error.kind, "event-queue-full");
    assert.equal(sharedQueue.takeGuiWebSharedActionId(created.value), 101);

    const queueSource = readRepoFile("web", "src", "gui-preview", "shared-event-queue.ts");
    const workerSource = readRepoFile("web", "src", "runtime", "worker.ts");
    const shellSource = readRepoFile("web", "src", "terminal", "shell.ts");
    const webInputSource = readRepoFile("stdlib", "platforms", "gui", "web", "input.nepl");
    const webFacadeSource = readRepoFile("stdlib", "platforms", "gui", "web.nepl");
    const counterSource = readRepoFile("examples", "gui_counter.nepl");

    assert.match(queueSource, /GUI_WEB_EVENT_QUEUE_CAPACITY/);
    assert.match(queueSource, /writeGuiWebSharedInputEvent/);
    assert.match(queueSource, /waitGuiWebSharedActionId/);
    assert.match(workerSource, /nepl_gui_web/);
    assert.match(workerSource, /poll_action_id/);
    assert.match(workerSource, /wait_action_id/);
    assert.match(shellSource, /registerGuiWebInputEventListener/);
    assert.match(shellSource, /writeGuiWebSharedInputEvent/);
    assert.match(shellSource, /guiSab/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "poll_action_id"/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "wait_action_id"/);
    assert.match(webInputSource, /pub fn gui_web_wait_action %impure fn i32 Option ActionId/);
    assert.match(webFacadeSource, /#import "\.\/web\/input" as @merge/);
    assert.match(counterSource, /gui_web_wait_action/);
    assert.doesNotMatch(queueSource, /\bas\b\s*any\b|:\s*any\b|<any>/);
    assert.doesNotMatch(queueSource, /\|\s*null|\|\s*undefined/);
    assert.doesNotMatch(queueSource, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);
    assert.doesNotMatch(workerSource, /createGuiPreviewScene/);
    assert.doesNotMatch(shellSource, /gui_counter|gui_life|gui_mandelbrot/);
    assert.doesNotMatch(counterSource, /\bor [A-Za-z_][A-Za-z0-9_]* or|\band [A-Za-z_][A-Za-z0-9_]* and/);

    return {
        ok: true,
        checks: [
            "Web GUI shared event queue transfers typed action events through SharedArrayBuffer",
            "Web runtime worker exposes a dedicated nepl_gui_web host import module",
            "NEPL web GUI input wrapper returns Option ActionId instead of public raw sentinels",
            "Counter example drives update/render from NEPL-side gui_web_wait_action",
        ],
    };
}

if (require.main === module) {
    runWebGuiSharedEventQueueRegression()
        .then((result) => process.stdout.write(JSON.stringify(result, null, 2) + "\n"))
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runWebGuiSharedEventQueueRegression,
};
