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
    const queuedRecord = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "action",
        windowId: 4,
        actionId: 12,
        point: { x: 8.125, y: 6.5 },
    });
    assert.equal(queuedRecord.kind, "ok");
    const takenRecord = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(takenRecord.kind, "event");
    assert.equal(takenRecord.event.kind, "action");
    assert.equal(takenRecord.event.windowId, 4);
    assert.equal(takenRecord.event.actionId, 12);
    assert.equal(takenRecord.event.pointXMilli, 8125);
    assert.equal(takenRecord.event.pointYMilli, 6500);
    assert.equal(sharedQueue.takeGuiWebSharedInputEvent(created.value).kind, "empty");

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    const queuedPointer = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "pointer",
        windowId: 4,
        pointerKind: "down",
        pointerId: 8,
        button: "primary",
        point: { x: 1.5, y: 2.25 },
    });
    assert.equal(queuedPointer.kind, "ok");
    assert.equal(sharedQueue.takeGuiWebSharedActionId(created.value), 0);
    const takenPointer = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(takenPointer.kind, "event");
    assert.equal(takenPointer.event.kind, "pointer");
    assert.equal(takenPointer.event.windowId, 4);
    assert.equal(takenPointer.event.pointerKind, "down");
    assert.equal(takenPointer.event.pointerId, 8);
    assert.equal(takenPointer.event.button, "primary");
    assert.equal(takenPointer.event.pointXMilli, 1500);
    assert.equal(takenPointer.event.pointYMilli, 2250);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    const queuedKeyboard = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "keyboard",
        windowId: 4,
        keyboardKind: "down",
        keyCode: 1001,
        modifierBits: 5,
    });
    assert.equal(queuedKeyboard.kind, "ok");
    assert.equal(sharedQueue.takeGuiWebSharedActionId(created.value), 0);
    const takenKeyboard = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(takenKeyboard.kind, "event");
    assert.equal(takenKeyboard.event.kind, "keyboard");
    assert.equal(takenKeyboard.event.windowId, 4);
    assert.equal(takenKeyboard.event.keyboardKind, "down");
    assert.equal(takenKeyboard.event.keyCode, 1001);
    assert.equal(takenKeyboard.event.modifierBits, 5);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    const queuedTextInput = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "text-input",
        windowId: 4,
        scalarValue: 0x3042,
    });
    assert.equal(queuedTextInput.kind, "ok");
    assert.equal(sharedQueue.takeGuiWebSharedActionId(created.value), 0);
    const takenTextInput = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(takenTextInput.kind, "event");
    assert.equal(takenTextInput.event.kind, "text-input");
    assert.equal(takenTextInput.event.windowId, 4);
    assert.equal(takenTextInput.event.scalarValue, 0x3042);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    const queuedNulTextInput = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "text-input",
        windowId: 4,
        scalarValue: 0,
    });
    assert.equal(queuedNulTextInput.kind, "ok");
    const takenNulTextInput = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(takenNulTextInput.kind, "event");
    assert.equal(takenNulTextInput.event.kind, "text-input");
    assert.equal(takenNulTextInput.event.scalarValue, 0);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    const queuedWindow = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "window",
        windowId: 4,
        windowKind: "resized",
        size: { width: 640, height: 480 },
    });
    assert.equal(queuedWindow.kind, "ok");
    assert.equal(sharedQueue.takeGuiWebSharedActionId(created.value), 0);
    const takenWindow = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(takenWindow.kind, "event");
    assert.equal(takenWindow.event.kind, "window");
    assert.equal(takenWindow.event.windowId, 4);
    assert.equal(takenWindow.event.windowKind, "resized");
    assert.equal(takenWindow.event.width, 640);
    assert.equal(takenWindow.event.height, 480);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    const rawQueue = new Int32Array(created.value);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 1);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH, 99);
    const invalidRecord = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(invalidRecord.kind, "invalid");
    assert.equal(invalidRecord.rawKind, 99);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 1);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH, sharedQueue.GUI_WEB_EVENT_KIND_TEXT_INPUT);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + 2, 0xD800);
    const invalidScalarRecord = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(invalidScalarRecord.kind, "invalid");
    assert.equal(invalidScalarRecord.rawKind, sharedQueue.GUI_WEB_EVENT_KIND_TEXT_INPUT);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 1);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH, sharedQueue.GUI_WEB_EVENT_KIND_WINDOW);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + 1, 4);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + 2, 99);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + 5, 640);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + 6, 480);
    const invalidWindowKindRecord = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(invalidWindowKindRecord.kind, "invalid");
    assert.equal(invalidWindowKindRecord.rawKind, sharedQueue.GUI_WEB_EVENT_KIND_WINDOW);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_WRITE_INDEX, 1);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH, sharedQueue.GUI_WEB_EVENT_KIND_WINDOW);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + 1, 4);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + 2, sharedQueue.GUI_WEB_WINDOW_KIND_RESIZED);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + 5, 0);
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH + 6, 480);
    const invalidWindowSizeRecord = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(invalidWindowSizeRecord.kind, "invalid");
    assert.equal(invalidWindowSizeRecord.rawKind, sharedQueue.GUI_WEB_EVENT_KIND_WINDOW);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    const actionQueueBase = sharedQueue.GUI_WEB_EVENT_QUEUE_HEADER_LENGTH
        + sharedQueue.GUI_WEB_EVENT_QUEUE_CAPACITY * sharedQueue.GUI_WEB_EVENT_QUEUE_SLOT_LENGTH;
    Atomics.store(rawQueue, sharedQueue.GUI_WEB_ACTION_QUEUE_WRITE_INDEX, 1);
    Atomics.store(rawQueue, actionQueueBase, -5);
    assert.equal(sharedQueue.takeGuiWebSharedActionId(created.value), sharedQueue.GUI_WEB_EVENT_POLL_INVALID);

    assert.equal(sharedQueue.waitGuiWebSharedInputEvent(created.value, 0).kind, "empty");
    assert.equal(sharedQueue.GUI_WEB_EVENT_QUEUE_SLOT_LENGTH, 8);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    for (let i = 1; i < sharedQueue.GUI_WEB_EVENT_QUEUE_CAPACITY * 2; i++) {
        const result = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
            kind: "action",
            windowId: 3,
            actionId: 100 + i,
            point: { x: i, y: i },
        });
        assert.equal(result.kind, "ok");
    }
    const saturatedAction = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "action",
        windowId: 3,
        actionId: 999,
        point: { x: 0, y: 0 },
    });
    assert.equal(saturatedAction.kind, "ok");
    const projectedActions = [];
    for (let i = 0; i < sharedQueue.GUI_WEB_ACTION_QUEUE_CAPACITY; i++) {
        const actionId = sharedQueue.takeGuiWebSharedActionId(created.value);
        if (actionId > 0) {
            projectedActions.push(actionId);
        }
    }
    assert.ok(projectedActions.includes(999));

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    for (let i = 1; i < sharedQueue.GUI_WEB_EVENT_QUEUE_CAPACITY * 2; i++) {
        const result = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
            kind: "pointer",
            windowId: 3,
            pointerKind: "move",
            pointerId: 1,
            button: "primary",
            point: { x: i, y: i + 1 },
        });
        assert.equal(result.kind, "ok");
    }
    const coalescedPointer = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(coalescedPointer.kind, "event");
    assert.equal(coalescedPointer.event.kind, "pointer");
    assert.equal(coalescedPointer.event.pointerKind, "move");
    assert.equal(coalescedPointer.event.pointXMilli, (sharedQueue.GUI_WEB_EVENT_QUEUE_CAPACITY * 2 - 1) * 1000);
    assert.equal(sharedQueue.takeGuiWebSharedInputEvent(created.value).kind, "empty");

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    assert.equal(sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "pointer",
        windowId: 3,
        pointerKind: "move",
        pointerId: 1,
        button: "primary",
        point: { x: 1, y: 1 },
    }).kind, "ok");
    assert.equal(sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "pointer",
        windowId: 3,
        pointerKind: "up",
        pointerId: 1,
        button: "primary",
        point: { x: 2, y: 2 },
    }).kind, "ok");
    assert.equal(sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "pointer",
        windowId: 3,
        pointerKind: "move",
        pointerId: 1,
        button: "primary",
        point: { x: 3, y: 3 },
    }).kind, "ok");
    const pointerMoveBeforeUp = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(pointerMoveBeforeUp.kind, "event");
    assert.equal(pointerMoveBeforeUp.event.kind, "pointer");
    assert.equal(pointerMoveBeforeUp.event.pointerKind, "move");
    assert.equal(pointerMoveBeforeUp.event.pointXMilli, 1000);
    const pointerUpBarrier = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(pointerUpBarrier.kind, "event");
    assert.equal(pointerUpBarrier.event.kind, "pointer");
    assert.equal(pointerUpBarrier.event.pointerKind, "up");
    assert.equal(pointerUpBarrier.event.pointXMilli, 2000);
    const pointerMoveAfterUp = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(pointerMoveAfterUp.kind, "event");
    assert.equal(pointerMoveAfterUp.event.kind, "pointer");
    assert.equal(pointerMoveAfterUp.event.pointerKind, "move");
    assert.equal(pointerMoveAfterUp.event.pointXMilli, 3000);
    assert.equal(sharedQueue.takeGuiWebSharedInputEvent(created.value).kind, "empty");

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    for (let i = 1; i < sharedQueue.GUI_WEB_EVENT_QUEUE_CAPACITY * 2; i++) {
        const result = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
            kind: "window",
            windowId: 3,
            windowKind: "resized",
            size: { width: 600 + i, height: 400 + i },
        });
        assert.equal(result.kind, "ok");
    }
    const coalescedWindow = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(coalescedWindow.kind, "event");
    assert.equal(coalescedWindow.event.kind, "window");
    assert.equal(coalescedWindow.event.windowKind, "resized");
    assert.equal(coalescedWindow.event.width, 600 + sharedQueue.GUI_WEB_EVENT_QUEUE_CAPACITY * 2 - 1);
    assert.equal(sharedQueue.takeGuiWebSharedInputEvent(created.value).kind, "empty");

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    assert.equal(sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "window",
        windowId: 3,
        windowKind: "resized",
        size: { width: 640, height: 480 },
    }).kind, "ok");
    assert.equal(sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "action",
        windowId: 3,
        actionId: 333,
        point: { x: 0, y: 0 },
    }).kind, "ok");
    assert.equal(sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "window",
        windowId: 3,
        windowKind: "resized",
        size: { width: 800, height: 600 },
    }).kind, "ok");
    const windowResizeBeforeAction = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(windowResizeBeforeAction.kind, "event");
    assert.equal(windowResizeBeforeAction.event.kind, "window");
    assert.equal(windowResizeBeforeAction.event.windowKind, "resized");
    assert.equal(windowResizeBeforeAction.event.width, 640);
    const actionBarrier = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(actionBarrier.kind, "event");
    assert.equal(actionBarrier.event.kind, "action");
    assert.equal(actionBarrier.event.actionId, 333);
    const windowResizeAfterAction = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(windowResizeAfterAction.kind, "event");
    assert.equal(windowResizeAfterAction.event.kind, "window");
    assert.equal(windowResizeAfterAction.event.windowKind, "resized");
    assert.equal(windowResizeAfterAction.event.width, 800);
    assert.equal(sharedQueue.takeGuiWebSharedInputEvent(created.value).kind, "empty");

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    assert.equal(sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "window",
        windowId: 3,
        windowKind: "focused",
        size: { width: 640, height: 480 },
    }).kind, "ok");
    assert.equal(sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "window",
        windowId: 3,
        windowKind: "unfocused",
        size: { width: 640, height: 480 },
    }).kind, "ok");
    const focusedWindow = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(focusedWindow.kind, "event");
    assert.equal(focusedWindow.event.kind, "window");
    assert.equal(focusedWindow.event.windowKind, "focused");
    const unfocusedWindow = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(unfocusedWindow.kind, "event");
    assert.equal(unfocusedWindow.event.kind, "window");
    assert.equal(unfocusedWindow.event.windowKind, "unfocused");
    assert.equal(sharedQueue.takeGuiWebSharedInputEvent(created.value).kind, "empty");

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    for (let i = 1; i < sharedQueue.GUI_WEB_EVENT_QUEUE_CAPACITY; i++) {
        const result = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
            kind: "pointer",
            windowId: 3,
            pointerKind: "down",
            pointerId: i,
            button: "primary",
            point: { x: i, y: i },
        });
        assert.equal(result.kind, "ok");
    }
    const projectedAction = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "action",
        windowId: 3,
        actionId: 777,
        point: { x: 0, y: 0 },
    });
    assert.equal(projectedAction.kind, "ok");
    assert.equal(sharedQueue.takeGuiWebSharedActionId(created.value), 777);

    sharedQueue.resetGuiWebSharedEventBuffer(created.value);
    for (let i = 1; i < sharedQueue.GUI_WEB_ACTION_QUEUE_CAPACITY; i++) {
        const result = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
            kind: "action",
            windowId: 3,
            actionId: 200 + i,
            point: { x: i, y: i },
        });
        assert.equal(result.kind, "ok");
        assert.equal(sharedQueue.takeGuiWebSharedInputEvent(created.value).kind, "event");
    }
    const fullEventAction = sharedQueue.writeGuiWebSharedInputEvent(created.value, {
        kind: "action",
        windowId: 3,
        actionId: 888,
        point: { x: 0, y: 0 },
    });
    assert.equal(fullEventAction.kind, "ok");
    const takenFullEventAction = sharedQueue.takeGuiWebSharedInputEvent(created.value);
    assert.equal(takenFullEventAction.kind, "event");
    assert.equal(takenFullEventAction.event.kind, "action");
    assert.equal(takenFullEventAction.event.actionId, 888);

    const queueSource = readRepoFile("web", "src", "gui-preview", "shared-event-queue.ts");
    const workerSource = readRepoFile("web", "src", "runtime", "worker.ts");
    const shellSource = readRepoFile("web", "src", "terminal", "shell.ts");
    const windowManagerSource = readRepoFile("web", "src", "gui-preview", "window-manager.ts");
    const webInputSource = readRepoFile("stdlib", "platforms", "gui", "web", "input.nepl");
    const webFacadeSource = readRepoFile("stdlib", "platforms", "gui", "web.nepl");
    const counterSource = readRepoFile("examples", "gui_counter.nepl");
    const lifeSource = readRepoFile("examples", "gui_life.nepl");
    const mandelbrotSource = readRepoFile("examples", "gui_mandelbrot.nepl");
    const calculatorSource = readRepoFile("examples", "gui_calculator.nepl");
    const scientificCalculatorSource = readRepoFile("examples", "gui_scientific_calculator.nepl");
    const paintSource = readRepoFile("examples", "gui_paint.nepl");
    const breakoutSource = readRepoFile("examples", "gui_breakout.nepl");

    assert.match(queueSource, /GUI_WEB_EVENT_QUEUE_CAPACITY/);
    assert.match(queueSource, /writeGuiWebSharedInputEvent/);
    assert.match(queueSource, /takeGuiWebSharedInputEvent/);
    assert.match(queueSource, /waitGuiWebSharedInputEvent/);
    assert.match(queueSource, /GUI_WEB_EVENT_KIND_POINTER/);
    assert.match(queueSource, /GUI_WEB_EVENT_KIND_KEYBOARD/);
    assert.match(queueSource, /GUI_WEB_EVENT_KIND_TEXT_INPUT/);
    assert.match(queueSource, /GUI_WEB_EVENT_KIND_WINDOW/);
    assert.match(queueSource, /GUI_WEB_WINDOW_KIND_RESIZED/);
    assert.match(queueSource, /GUI_WEB_ACTION_QUEUE_WRITE_INDEX/);
    assert.match(queueSource, /guiWebSharedPointerKindToRaw/);
    assert.match(queueSource, /guiWebSharedKeyboardKindToRaw/);
    assert.match(queueSource, /guiWebSharedWindowKindToRaw/);
    assert.match(queueSource, /guiWebSharedWindowKindFromRaw/);
    assert.match(queueSource, /guiWebSharedIsUnicodeScalarValue/);
    assert.match(queueSource, /pointXMilli/);
    assert.match(queueSource, /GUI_WEB_EVENT_POLL_INVALID/);
    assert.doesNotMatch(queueSource, /guiWebSharedActionIdFromTakeResult/);
    assert.doesNotMatch(queueSource, /event-queue-full|action-queue-full/);
    assert.match(queueSource, /guiWebSharedFindPointerMoveSlot/);
    assert.match(queueSource, /guiWebSharedFindWindowStateSlot/);
    assert.match(queueSource, /guiWebSharedFindLatestEventSlot/);
    assert.doesNotMatch(queueSource, /while \(index !== writeIndex\)/);
    assert.match(queueSource, /Atomics\.store\(queue, GUI_WEB_EVENT_QUEUE_READ_INDEX, guiWebSharedEventNextIndex\(readIndex\)\)/);
    assert.match(queueSource, /waitGuiWebSharedActionId/);
    assert.match(workerSource, /nepl_gui_web/);
    assert.match(workerSource, /poll_action_id/);
    assert.match(workerSource, /wait_action_id/);
    assert.match(workerSource, /poll_event_kind/);
    assert.match(workerSource, /wait_event_kind/);
    assert.match(workerSource, /last_event_window_id/);
    assert.match(workerSource, /last_event_point_x_milli/);
    assert.match(workerSource, /last_event_pointer_kind/);
    assert.match(workerSource, /last_event_keyboard_kind/);
    assert.match(workerSource, /last_event_text_scalar_value/);
    assert.match(workerSource, /last_event_window_kind/);
    assert.match(workerSource, /last_event_window_width/);
    assert.match(workerSource, /last_event_window_height/);
    assert.match(workerSource, /GUI_WEB_EVENT_KIND_POINTER/);
    assert.match(workerSource, /GUI_WEB_EVENT_KIND_KEYBOARD/);
    assert.match(workerSource, /GUI_WEB_EVENT_KIND_TEXT_INPUT/);
    assert.match(workerSource, /GUI_WEB_EVENT_KIND_WINDOW/);
    assert.match(workerSource, /lastGuiWebInputEvent = \{ kind: 'empty' \}/);
    assert.match(workerSource, /return -1;/);
    assert.match(shellSource, /registerGuiWebInputEventListener/);
    assert.match(shellSource, /writeGuiWebSharedInputEvent/);
    assert.match(shellSource, /guiSab/);
    assert.match(shellSource, /guiRuntimeInputWindowIds/);
    assert.match(shellSource, /has\(event\.windowId\)/);
    assert.match(shellSource, /add\(event\.frame\.windowId\)/);
    assert.match(shellSource, /stopActiveGuiProcessFromWindowClose/);
    assert.match(shellSource, /closeGuiRuntimeWindows/);
    assert.match(shellSource, /closeGuiWebRuntimeHostFrameWindow/);
    assert.match(windowManagerSource, /queueHostWindowEvent/);
    assert.match(windowManagerSource, /closeHostFrameWindow/);
    assert.doesNotMatch(windowManagerSource, /source-path|preview-kind/);
    assert.match(windowManagerSource, /'close-requested'/);
    assert.match(windowManagerSource, /'resized'/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "poll_action_id"/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "wait_action_id"/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "poll_event_kind"/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "last_event_window_id"/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "last_event_pointer_kind"/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "last_event_keyboard_kind"/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "last_event_text_scalar_value"/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "last_event_window_kind"/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "last_event_window_width"/);
    assert.match(webInputSource, /#extern "nepl_gui_web" "last_event_window_height"/);
    assert.match(webInputSource, /pub struct GuiWebEvent/);
    assert.match(webInputSource, /pub fn gui_web_event_pointer %fn &GuiWebEvent Option PointerEvent/);
    assert.match(webInputSource, /pub fn gui_web_event_keyboard %fn &GuiWebEvent Option KeyboardEvent/);
    assert.match(webInputSource, /pub fn gui_web_event_text_input %fn &GuiWebEvent Option TextInputEvent/);
    assert.match(webInputSource, /pub fn gui_web_event_window %fn &GuiWebEvent Option WindowEvent/);
    assert.match(webInputSource, /pub fn gui_web_wait_event_result %impure fn i32 Result Option GuiWebEvent GuiError/);
    assert.match(webInputSource, /pub fn gui_web_wait_action %impure fn i32 Option ActionId/);
    assert.match(webInputSource, /pub fn gui_web_wait_action_result %impure fn i32 Result Option ActionId GuiError/);
    assert.match(webFacadeSource, /#import "\.\/web\/input" as @merge/);
    assert.match(counterSource, /gui_web_wait_action_result/);
    assert.match(lifeSource, /gui_web_wait_event_result/);
    assert.match(lifeSource, /life_next_action/);
    assert.match(lifeSource, /life_animate_action/);
    assert.match(lifeSource, /life_resolution_down_action/);
    assert.match(lifeSource, /life_resolution_up_action/);
    assert.match(lifeSource, /gui_web_stdout_action_rect/);
    assert.match(mandelbrotSource, /gui_web_wait_event_result/);
    assert.match(mandelbrotSource, /mandelbrot_preview_action/);
    assert.match(mandelbrotSource, /mandelbrot_hd_action/);
    assert.match(mandelbrotSource, /mandelbrot_detail_action/);
    assert.match(mandelbrotSource, /gui_web_stdout_action_rect/);
    assert.match(calculatorSource, /gui_web_wait_event_result/);
    assert.match(calculatorSource, /calculator_update_action/);
    assert.match(calculatorSource, /calculator_action_eq/);
    assert.match(scientificCalculatorSource, /gui_web_wait_event_result/);
    assert.match(scientificCalculatorSource, /sci_action_square/);
    assert.match(scientificCalculatorSource, /sci_integer_sqrt/);
    assert.match(paintSource, /gui_web_wait_event_result/);
    assert.match(paintSource, /gui_web_event_pointer/);
    assert.match(paintSource, /pointer_event_kind/);
    assert.match(paintSource, /pointer_event_button/);
    assert.match(paintSource, /PointerButton::Primary/);
    assert.match(breakoutSource, /gui_web_wait_event_result/);
    assert.match(breakoutSource, /breakout_tick/);
    assert.match(breakoutSource, /timeout_ms %i32 if animate 33 60000/);
    assert.doesNotMatch(queueSource, /\bas\b\s*any\b|:\s*any\b|<any>/);
    assert.doesNotMatch(queueSource, /\|\s*null|\|\s*undefined/);
    assert.doesNotMatch(queueSource, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);
    assert.doesNotMatch(workerSource, /createGuiPreviewScene/);
    assert.doesNotMatch(shellSource, /gui_counter|gui_life|gui_mandelbrot|gui_calculator|gui_scientific_calculator|gui_paint|gui_breakout/);
    assert.doesNotMatch(counterSource, /\bor [A-Za-z_][A-Za-z0-9_]* or|\band [A-Za-z_][A-Za-z0-9_]* and/);
    assert.doesNotMatch(lifeSource, /\bor [A-Za-z_][A-Za-z0-9_]* or|\band [A-Za-z_][A-Za-z0-9_]* and/);
    assert.doesNotMatch(mandelbrotSource, /\bor [A-Za-z_][A-Za-z0-9_]* or|\band [A-Za-z_][A-Za-z0-9_]* and/);
    assert.doesNotMatch(calculatorSource, /\bor [A-Za-z_][A-Za-z0-9_]* or|\band [A-Za-z_][A-Za-z0-9_]* and/);
    assert.doesNotMatch(scientificCalculatorSource, /\bor [A-Za-z_][A-Za-z0-9_]* or|\band [A-Za-z_][A-Za-z0-9_]* and/);
    assert.doesNotMatch(paintSource, /\bor [A-Za-z_][A-Za-z0-9_]* or|\band [A-Za-z_][A-Za-z0-9_]* and/);
    assert.doesNotMatch(breakoutSource, /\bor [A-Za-z_][A-Za-z0-9_]* or|\band [A-Za-z_][A-Za-z0-9_]* and/);

    return {
        ok: true,
        checks: [
            "Web GUI shared event queue transfers typed action events through SharedArrayBuffer",
            "Web GUI shared event queue exposes full action records with window and pointer fields",
            "Web GUI shared event queue exposes pointer records without consuming the legacy action projection",
            "Web GUI shared event queue exposes keyboard records without consuming the legacy action projection",
            "Web GUI shared event queue exposes Unicode scalar text input records without consuming the legacy action projection",
            "Web GUI shared event queue exposes window records with a fixed eight-slot layout",
            "Web GUI shared event queue reports invalid records instead of collapsing them into no event",
            "Web GUI shared event queue coalesces high-frequency state without producer overflow",
            "Web runtime worker exposes a dedicated nepl_gui_web host import module",
            "Web runtime worker exposes event-kind and last-event field imports for GuiEvent polling",
            "Web runtime worker exposes window event field imports for GuiEvent polling",
            "NEPL web GUI input wrapper returns Result Option ActionId instead of public raw sentinels",
            "NEPL web GUI input wrapper exposes Result Option GuiWebEvent for full event polling",
            "Web shell filters GUI action input to windows presented by the active run",
            "Counter example drives update/render from NEPL-side gui_web_wait_action_result",
            "Life and Mandelbrot examples drive interactive redraws from full NEPL-side GuiWebEvent polling",
            "Calculator, scientific calculator, paint, and breakout examples run as NEPL GUI apps without TS simulation",
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
