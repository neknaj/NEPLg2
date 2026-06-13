"use strict";

const assert = require("node:assert/strict");

const HOST_OK = 0;
const HOST_INVALID_COMMAND = -2;
const HOST_RESOURCE_EXHAUSTED = -3;

function createGuiVideoMemoryFakeHost(options) {
    const width = options.width;
    const height = options.height;
    const slotCount = options.slotCount || 2;
    const windowId = options.windowId || 1;
    const title = options.title;
    const expectedRgbaRow = options.expectedRgbaRow;
    const surfaceId = options.surfaceId || 1201;
    const frameId = options.frameId || 4501;

    let runtime = null;
    let activeImports = null;
    const calls = [];
    const violations = [];
    const surfaces = new Map();
    let createdSurfaceId = null;
    let acquiredFrameId = null;

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

    const importsFactory = (context) => {
        runtime = context;
        activeImports = {
            nepl_gui_web: {
                video_memory_create_surface(requestedWidth, requestedHeight, requestedSlotCount) {
                    record("create", arguments);
                    if (requestedWidth !== width || requestedHeight !== height || requestedSlotCount !== slotCount) {
                        return fail(`unexpected surface shape ${requestedWidth}x${requestedHeight} slots=${requestedSlotCount}`);
                    }
                    if (surfaces.size !== 0) {
                        return fail("surface created more than once");
                    }
                    surfaces.set(surfaceId, {
                        width,
                        height,
                        slotCount,
                        closed: false,
                        writingFrame: null,
                        publishedFrame: null,
                        presented: false,
                        rows: new Map(),
                    });
                    createdSurfaceId = surfaceId;
                    return surfaceId;
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
                    found.surface.writingFrame = frameId;
                    acquiredFrameId = frameId;
                    return frameId;
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
                    if (x !== 0 || rowWidth !== width || y < 0 || y >= height) {
                        return fail(`unexpected row geometry x=${x} y=${y} width=${rowWidth}`);
                    }
                    const read = readMemorySlice(srcPtr, rowWidth * 4);
                    if (!read.ok) return read.status;
                    assert.deepEqual(
                        Array.from(read.bytes),
                        expectedRgbaRow(y),
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
                        Array.from({ length: height }, (_unused, row) => row),
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
                    if (requestedWindowId !== windowId) {
                        return fail(`unexpected window id ${requestedWindowId}`);
                    }
                    const read = readMemorySlice(titlePtr, titleLen);
                    if (!read.ok) return read.status;
                    const decodedTitle = new TextDecoder("utf-8", { fatal: true }).decode(read.bytes);
                    if (decodedTitle !== title) {
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
        assert.notEqual(createdSurfaceId, null);
        assert.notEqual(acquiredFrameId, null);
        assert.deepEqual(
            calls.map((call) => call.name),
            ["create", "acquire", ...Array.from({ length: height }, () => "write-row"), "publish", "present", "close"],
        );
        assert.deepEqual(calls[0].args, [width, height, slotCount]);
        assert.deepEqual(calls[1].args, [createdSurfaceId]);
        for (let y = 0; y < height; y += 1) {
            assert.deepEqual(calls[2 + y].args.slice(0, 5), [createdSurfaceId, acquiredFrameId, 0, y, width]);
        }
        assert.deepEqual(calls[2 + height].args, [createdSurfaceId, acquiredFrameId, 1, 0, 0, 0, 0]);
        assert.equal(calls[3 + height].args[0], windowId);
        assert.equal(calls[3 + height].args[3], createdSurfaceId);
        assert.deepEqual(calls[4 + height].args, [createdSurfaceId]);
        const surface = surfaces.get(createdSurfaceId);
        assert.equal(surface.closed, true);
        assert.equal(surface.writingFrame, null);
        assert.equal(surface.publishedFrame, acquiredFrameId);
        assert.equal(surface.presented, true);
        const probeSurfaceId = surfaceId + 1;
        surfaces.set(probeSurfaceId, {
            width,
            height,
            slotCount,
            closed: false,
            writingFrame: null,
            publishedFrame: null,
            presented: false,
            rows: new Map(),
        });
        assert.equal(activeImports.nepl_gui_web.video_memory_present_surface(windowId, 0, 0, probeSurfaceId), HOST_INVALID_COMMAND);
        surfaces.delete(probeSurfaceId);
        assert.equal(activeImports.nepl_gui_web.video_memory_acquire_write_slot(createdSurfaceId), HOST_INVALID_COMMAND);
        assert.equal(
            activeImports.nepl_gui_web.video_memory_write_rgba8888_row(createdSurfaceId, acquiredFrameId, 0, 0, width, 0),
            HOST_INVALID_COMMAND,
        );
        assert.equal(
            activeImports.nepl_gui_web.video_memory_publish_slot(createdSurfaceId, acquiredFrameId, 1, 0, 0, 0, 0),
            HOST_INVALID_COMMAND,
        );
        assert.equal(activeImports.nepl_gui_web.video_memory_present_surface(windowId, 0, 0, createdSurfaceId), HOST_INVALID_COMMAND);
        assert.equal(activeImports.nepl_gui_web.video_memory_close_surface(createdSurfaceId), HOST_INVALID_COMMAND);
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
    createGuiVideoMemoryFakeHost,
};
