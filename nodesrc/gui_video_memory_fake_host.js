"use strict";

const assert = require("node:assert/strict");

const HOST_OK = 0;
const HOST_UNSUPPORTED = -1;
const HOST_INVALID_COMMAND = -2;
const HOST_RESOURCE_EXHAUSTED = -3;

function normalizeExpectedSurfaces(options) {
    const surfaces = options.surfaces || [{
        width: options.width,
        height: options.height,
        slotCount: options.slotCount,
        windowId: options.windowId,
        title: options.title,
        expectedRgbaRow: options.expectedRgbaRow,
        surfaceId: options.surfaceId,
        frameId: options.frameId,
    }];
    return surfaces.map((surface, index) => ({
        width: surface.width,
        height: surface.height,
        slotCount: surface.slotCount || options.slotCount || 2,
        windowId: surface.windowId || options.windowId || 1,
        title: surface.title || options.title,
        expectedRgbaRow: surface.expectedRgbaRow || options.expectedRgbaRow,
        surfaceId: surface.surfaceId || (options.surfaceId || 1201) + index,
        frameId: surface.frameId || (options.frameId || 4501) + index,
    }));
}

function createGuiVideoMemoryFakeHost(options) {
    const windowId = options.windowId || 1;
    const expectedSurfaces = normalizeExpectedSurfaces(options);
    const events = options.events || [];

    let runtime = null;
    let activeImports = null;
    const calls = [];
    const violations = [];
    const surfaces = new Map();
    const createdSurfaceIds = [];
    const acquiredFrameIds = [];
    let nextSurfaceIndex = 0;
    let nextEventIndex = 0;
    let lastEvent = null;

    const fail = (message) => {
        violations.push(message);
        return HOST_INVALID_COMMAND;
    };

    const record = (name, args) => {
        calls.push({ name, args: Array.from(args) });
    };

    const getSurface = (id, operation) => {
        const surface = surfaces.get(id);
        if (!surface) {
            return { ok: false, status: fail(`${operation}: unknown surface ${id}`) };
        }
        if (surface.closed) {
            return { ok: false, status: HOST_INVALID_COMMAND };
        }
        return { ok: true, surface };
    };

    const readMemorySlice = (ptr, len) => {
        const memory = runtime ? runtime.getMemory() : null;
        if (!memory || !(memory.buffer instanceof ArrayBuffer)) {
            return { ok: false, status: fail("missing wasm memory") };
        }
        if (!Number.isInteger(ptr) || !Number.isInteger(len) || ptr < 0 || len < 0) {
            return { ok: false, status: fail(`invalid memory range ptr=${ptr} len=${len}`) };
        }
        if (ptr + len > memory.buffer.byteLength) {
            return { ok: false, status: fail(`memory range out of bounds ptr=${ptr} len=${len}`) };
        }
        return { ok: true, bytes: new Uint8Array(memory.buffer, ptr, len).slice() };
    };

    const eventKindRaw = (event) => {
        if (!event) return 0;
        if (event.kind === "window") return 5;
        return HOST_INVALID_COMMAND;
    };

    const windowKindRaw = (kind) => {
        switch (kind) {
            case "resized":
                return 1;
            case "focused":
                return 2;
            case "unfocused":
                return 3;
            case "close-requested":
                return 4;
            default:
                return HOST_INVALID_COMMAND;
        }
    };

    const takeEventKind = () => {
        if (nextEventIndex >= events.length) {
            lastEvent = null;
            return 0;
        }
        lastEvent = events[nextEventIndex];
        nextEventIndex += 1;
        return eventKindRaw(lastEvent);
    };

    const importsFactory = (context) => {
        runtime = context;
        activeImports = {
            nepl_gui_web: {
                video_memory_create_surface(requestedWidth, requestedHeight, requestedSlotCount) {
                    record("create", arguments);
                    if (nextSurfaceIndex >= expectedSurfaces.length) {
                        return fail(`created too many surfaces: ${requestedWidth}x${requestedHeight}`);
                    }
                    if (nextSurfaceIndex > 0) {
                        const previous = surfaces.get(expectedSurfaces[nextSurfaceIndex - 1].surfaceId);
                        if (!previous || !previous.closed) {
                            return fail("new surface created before previous surface close");
                        }
                    }
                    const expected = expectedSurfaces[nextSurfaceIndex];
                    if (requestedWidth !== expected.width || requestedHeight !== expected.height || requestedSlotCount !== expected.slotCount) {
                        return fail(`unexpected surface shape ${requestedWidth}x${requestedHeight} slots=${requestedSlotCount}`);
                    }
                    surfaces.set(expected.surfaceId, {
                        width: expected.width,
                        height: expected.height,
                        slotCount: expected.slotCount,
                        windowId: expected.windowId,
                        title: expected.title,
                        expectedRgbaRow: expected.expectedRgbaRow,
                        frameId: expected.frameId,
                        closed: false,
                        writingFrame: null,
                        publishedFrame: null,
                        presented: false,
                        rows: new Map(),
                    });
                    createdSurfaceIds.push(expected.surfaceId);
                    nextSurfaceIndex += 1;
                    return expected.surfaceId;
                },
                video_memory_acquire_write_slot(requestedSurfaceId) {
                    record("acquire", arguments);
                    const found = getSurface(requestedSurfaceId, "acquire");
                    if (!found.ok) return found.status;
                    if (found.surface.writingFrame !== null) {
                        return HOST_RESOURCE_EXHAUSTED;
                    }
                    if (found.surface.publishedFrame !== null) {
                        return HOST_RESOURCE_EXHAUSTED;
                    }
                    found.surface.writingFrame = found.surface.frameId;
                    acquiredFrameIds.push(found.surface.frameId);
                    return found.surface.frameId;
                },
                video_memory_write_slot_bytes() {
                    record("write-bytes", arguments);
                    return fail("write_slot_bytes must not be used by row host examples");
                },
                video_memory_write_rgba8888_row(requestedSurfaceId, requestedFrameId, x, y, rowWidth, srcPtr) {
                    record("write-row", arguments);
                    const found = getSurface(requestedSurfaceId, "write-row");
                    if (!found.ok) return found.status;
                    if (found.surface.writingFrame !== requestedFrameId) {
                        return fail(`write-row without matching writing frame ${requestedFrameId}`);
                    }
                    if (x !== 0 || rowWidth !== found.surface.width || y < 0 || y >= found.surface.height) {
                        return fail(`unexpected row geometry x=${x} y=${y} width=${rowWidth}`);
                    }
                    const read = readMemorySlice(srcPtr, rowWidth * 4);
                    if (!read.ok) return read.status;
                    assert.deepEqual(
                        Array.from(read.bytes),
                        found.surface.expectedRgbaRow(y),
                        `row ${y} RGBA payload must match expected NEPL output`,
                    );
                    found.surface.rows.set(y, Array.from(read.bytes));
                    return HOST_OK;
                },
                video_memory_fill_rect_rgba8888() {
                    record("fill-rect", arguments);
                    return fail("fill_rect must not be used by row host examples");
                },
                video_memory_discard_write_slot(requestedSurfaceId, requestedFrameId) {
                    record("discard", arguments);
                    const found = getSurface(requestedSurfaceId, "discard");
                    if (!found.ok) return found.status;
                    if (found.surface.writingFrame !== requestedFrameId) {
                        return fail(`discard without matching writing frame ${requestedFrameId}`);
                    }
                    found.surface.writingFrame = null;
                    found.surface.rows.clear();
                    return HOST_OK;
                },
                video_memory_publish_slot(requestedSurfaceId, requestedFrameId, dirtyKind, x, y, dirtyWidth, dirtyHeight) {
                    record("publish", arguments);
                    const found = getSurface(requestedSurfaceId, "publish");
                    if (!found.ok) return found.status;
                    if (found.surface.writingFrame !== requestedFrameId) {
                        return fail(`publish without matching writing frame ${requestedFrameId}`);
                    }
                    if (dirtyKind !== 1 || x !== 0 || y !== 0 || dirtyWidth !== 0 || dirtyHeight !== 0) {
                        return fail(`publish must use full dirty region, got ${dirtyKind}/${x}/${y}/${dirtyWidth}/${dirtyHeight}`);
                    }
                    assert.deepEqual(
                        Array.from(found.surface.rows.keys()).sort((a, b) => a - b),
                        Array.from({ length: found.surface.height }, (_unused, row) => row),
                        "publish must happen after every row is written",
                    );
                    found.surface.writingFrame = null;
                    found.surface.publishedFrame = requestedFrameId;
                    return HOST_OK;
                },
                video_memory_present_surface(requestedWindowId, titlePtr, titleLen, requestedSurfaceId) {
                    record("present", arguments);
                    const found = getSurface(requestedSurfaceId, "present");
                    if (!found.ok) return found.status;
                    if (found.surface.publishedFrame === null) {
                        return fail("present before publish");
                    }
                    if (requestedWindowId !== found.surface.windowId) {
                        return fail(`unexpected window id ${requestedWindowId}`);
                    }
                    const read = readMemorySlice(titlePtr, titleLen);
                    if (!read.ok) return read.status;
                    const decodedTitle = new TextDecoder("utf-8", { fatal: true }).decode(read.bytes);
                    if (decodedTitle !== found.surface.title) {
                        return fail(`unexpected title ${decodedTitle}`);
                    }
                    found.surface.presented = true;
                    return HOST_OK;
                },
                video_memory_close_surface(requestedSurfaceId) {
                    record("close", arguments);
                    const found = getSurface(requestedSurfaceId, "close");
                    if (!found.ok) return found.status;
                    if (found.surface.writingFrame !== null) {
                        return fail("close with an active writing frame");
                    }
                    if (!found.surface.presented) {
                        return fail("close before present");
                    }
                    found.surface.closed = true;
                    return HOST_OK;
                },
                poll_event_kind() {
                    return takeEventKind();
                },
                wait_event_kind() {
                    return takeEventKind();
                },
                last_event_window_id() {
                    return lastEvent && Number.isInteger(lastEvent.windowId) ? lastEvent.windowId : windowId;
                },
                last_event_action_id() {
                    return lastEvent && Number.isInteger(lastEvent.actionId) ? lastEvent.actionId : 0;
                },
                last_event_point_x_milli() {
                    return lastEvent && Number.isInteger(lastEvent.xMilli) ? lastEvent.xMilli : 0;
                },
                last_event_point_y_milli() {
                    return lastEvent && Number.isInteger(lastEvent.yMilli) ? lastEvent.yMilli : 0;
                },
                last_event_pointer_kind() {
                    return 0;
                },
                last_event_pointer_id() {
                    return 0;
                },
                last_event_pointer_button() {
                    return 0;
                },
                last_event_keyboard_kind() {
                    return 0;
                },
                last_event_key_code() {
                    return 0;
                },
                last_event_key_modifiers() {
                    return 0;
                },
                last_event_text_scalar_value() {
                    return 0;
                },
                last_event_window_kind() {
                    return lastEvent && lastEvent.kind === "window" ? windowKindRaw(lastEvent.windowKind) : 0;
                },
                last_event_window_width() {
                    return lastEvent && Number.isInteger(lastEvent.width) ? lastEvent.width : 0;
                },
                last_event_window_height() {
                    return lastEvent && Number.isInteger(lastEvent.height) ? lastEvent.height : 0;
                },
                last_event_timer_id() {
                    return 0;
                },
                last_event_timer_tick() {
                    return 0;
                },
            },
        };
        return activeImports;
    };

    const verify = (result) => {
        assert.equal(result.ok, true, JSON.stringify(result, null, 2));
        assert.equal(result.phase, "run");
        assert.ok(result.exit_code === 0 || result.exit_code === null, JSON.stringify(result, null, 2));
        assert.equal(result.stdout, "");
        assert.equal(result.stderr, "");
        assert.deepEqual(violations, []);
        assert.equal(nextEventIndex, events.length, "fake host event sequence must be fully consumed");
        assert.equal(createdSurfaceIds.length, expectedSurfaces.length);
        assert.equal(acquiredFrameIds.length, expectedSurfaces.length);

        const expectedCallNames = [];
        for (const surface of expectedSurfaces) {
            expectedCallNames.push("create", "acquire", ...Array.from({ length: surface.height }, () => "write-row"), "publish", "present", "close");
        }
        assert.deepEqual(calls.map((call) => call.name), expectedCallNames);

        let callIndex = 0;
        for (const surface of expectedSurfaces) {
            assert.deepEqual(calls[callIndex].args, [surface.width, surface.height, surface.slotCount]);
            callIndex += 1;
            assert.deepEqual(calls[callIndex].args, [surface.surfaceId]);
            callIndex += 1;
            for (let y = 0; y < surface.height; y += 1) {
                assert.deepEqual(calls[callIndex].args.slice(0, 5), [surface.surfaceId, surface.frameId, 0, y, surface.width]);
                callIndex += 1;
            }
            assert.deepEqual(calls[callIndex].args, [surface.surfaceId, surface.frameId, 1, 0, 0, 0, 0]);
            callIndex += 1;
            assert.equal(calls[callIndex].args[0], surface.windowId);
            assert.equal(calls[callIndex].args[3], surface.surfaceId);
            callIndex += 1;
            assert.deepEqual(calls[callIndex].args, [surface.surfaceId]);
            callIndex += 1;
            const actualSurface = surfaces.get(surface.surfaceId);
            assert.equal(actualSurface.closed, true);
            assert.equal(actualSurface.writingFrame, null);
            assert.equal(actualSurface.publishedFrame, surface.frameId);
            assert.equal(actualSurface.presented, true);
        }

        const lastSurface = expectedSurfaces[expectedSurfaces.length - 1];
        const probeSurfaceId = lastSurface.surfaceId + 1000;
        surfaces.set(probeSurfaceId, {
            width: lastSurface.width,
            height: lastSurface.height,
            slotCount: lastSurface.slotCount,
            closed: false,
            writingFrame: null,
            publishedFrame: null,
            presented: false,
            rows: new Map(),
        });
        assert.equal(activeImports.nepl_gui_web.video_memory_present_surface(windowId, 0, 0, probeSurfaceId), HOST_INVALID_COMMAND);
        surfaces.delete(probeSurfaceId);
        for (const surface of expectedSurfaces) {
            assert.equal(activeImports.nepl_gui_web.video_memory_acquire_write_slot(surface.surfaceId), HOST_INVALID_COMMAND);
            assert.equal(
                activeImports.nepl_gui_web.video_memory_write_rgba8888_row(surface.surfaceId, surface.frameId, 0, 0, surface.width, 0),
                HOST_INVALID_COMMAND,
            );
            assert.equal(
                activeImports.nepl_gui_web.video_memory_publish_slot(surface.surfaceId, surface.frameId, 1, 0, 0, 0, 0),
                HOST_INVALID_COMMAND,
            );
            assert.equal(activeImports.nepl_gui_web.video_memory_present_surface(surface.windowId, 0, 0, surface.surfaceId), HOST_INVALID_COMMAND);
            assert.equal(activeImports.nepl_gui_web.video_memory_close_surface(surface.surfaceId), HOST_INVALID_COMMAND);
        }
    };

    return {
        importsFactory,
        verify,
    };
}

module.exports = {
    HOST_INVALID_COMMAND,
    HOST_OK,
    HOST_RESOURCE_EXHAUSTED,
    HOST_UNSUPPORTED,
    createGuiVideoMemoryFakeHost,
};
