#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { runSingle } = require("./run_test");

const HOST_OK = 0;
const HOST_INVALID_COMMAND = -2;
const HOST_RESOURCE_EXHAUSTED = -3;

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function createVideoMemoryFakeHost() {
    let runtime = null;
    let activeImports = null;
    const calls = [];
    const violations = [];
    const surfaces = new Map();
    let createdSurfaceId = null;
    let acquiredFrameId = null;
    const surfaceId = 1201;
    const frameId = 4501;

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

    const expectedRgbaRow = (y) => {
        const bytes = [];
        for (let x = 0; x < 8; x += 1) {
            bytes.push(24 + x * 18);
            bytes.push(72 + y * 80);
            bytes.push(120 + x * 7);
            bytes.push(255);
        }
        return bytes;
    };

    const importsFactory = (context) => {
        runtime = context;
        activeImports = {
            nepl_gui_web: {
                video_memory_create_surface(width, height, slotCount) {
                    record("create", arguments);
                    if (width !== 8 || height !== 2 || slotCount !== 2) {
                        return fail(`unexpected surface shape ${width}x${height} slots=${slotCount}`);
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
                    return fail("write_slot_bytes must not be used by row example");
                },
                video_memory_write_rgba8888_row(requestedSurfaceId, requestedFrameId, x, y, width, srcPtr) {
                    record("write-row", arguments);
                    const found = getSurface(requestedSurfaceId, "write-row");
                    if (!found.ok) return found.status;
                    if (found.surface.writingFrame !== requestedFrameId) {
                        return fail(`write-row without matching writing frame ${requestedFrameId}`);
                    }
                    if (x !== 0 || width !== found.surface.width || y < 0 || y >= found.surface.height) {
                        return fail(`unexpected row geometry x=${x} y=${y} width=${width}`);
                    }
                    const read = readMemorySlice(srcPtr, width * 4);
                    if (!read.ok) return read.status;
                    assert.deepEqual(
                        Array.from(read.bytes),
                        expectedRgbaRow(y),
                        `row ${y} RGBA payload must match NEPL example output`,
                    );
                    found.surface.rows.set(y, Array.from(read.bytes));
                    return HOST_OK;
                },
                video_memory_fill_rect_rgba8888() {
                    record("fill-rect", arguments);
                    return fail("fill_rect must not be used by row example");
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
                video_memory_publish_slot(requestedSurfaceId, requestedFrameId, dirtyKind, x, y, width, height) {
                    record("publish", arguments);
                    const found = getSurface(requestedSurfaceId, "publish");
                    if (!found.ok) return found.status;
                    if (found.surface.writingFrame !== requestedFrameId) {
                        return fail(`publish without matching writing frame ${requestedFrameId}`);
                    }
                    if (dirtyKind !== 1 || x !== 0 || y !== 0 || width !== 0 || height !== 0) {
                        return fail(`publish must use full dirty region, got ${dirtyKind}/${x}/${y}/${width}/${height}`);
                    }
                    assert.deepEqual(
                        Array.from(found.surface.rows.keys()).sort(),
                        [0, 1],
                        "publish must happen after both rows are written",
                    );
                    found.surface.writingFrame = null;
                    found.surface.publishedFrame = requestedFrameId;
                    return HOST_OK;
                },
                video_memory_present_surface(windowId, titlePtr, titleLen, requestedSurfaceId) {
                    record("present", arguments);
                    const found = getSurface(requestedSurfaceId, "present");
                    if (!found.ok) return found.status;
                    if (found.surface.publishedFrame === null) {
                        return fail("present before publish");
                    }
                    if (windowId !== 1) {
                        return fail(`unexpected window id ${windowId}`);
                    }
                    const read = readMemorySlice(titlePtr, titleLen);
                    if (!read.ok) return read.status;
                    const title = new TextDecoder("utf-8", { fatal: true }).decode(read.bytes);
                    if (title !== "NEPLg2 Video Memory Rows") {
                        return fail(`unexpected title ${title}`);
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
        assert.equal(result.exit_code, 0, JSON.stringify(result, null, 2));
        assert.equal(result.stdout, "");
        assert.equal(result.stderr, "");
        assert.deepEqual(violations, []);
        assert.notEqual(createdSurfaceId, null);
        assert.notEqual(acquiredFrameId, null);
        assert.deepEqual(
            calls.map((call) => call.name),
            ["create", "acquire", "write-row", "write-row", "publish", "present", "close"],
        );
        assert.deepEqual(calls[0].args, [8, 2, 2]);
        assert.deepEqual(calls[1].args, [createdSurfaceId]);
        assert.deepEqual(calls[2].args.slice(0, 5), [createdSurfaceId, acquiredFrameId, 0, 0, 8]);
        assert.deepEqual(calls[3].args.slice(0, 5), [createdSurfaceId, acquiredFrameId, 0, 1, 8]);
        assert.deepEqual(calls[4].args, [createdSurfaceId, acquiredFrameId, 1, 0, 0, 0, 0]);
        assert.equal(calls[5].args[0], 1);
        assert.equal(calls[5].args[3], createdSurfaceId);
        assert.deepEqual(calls[6].args, [createdSurfaceId]);
        const surface = surfaces.get(createdSurfaceId);
        assert.equal(surface.closed, true);
        assert.equal(surface.writingFrame, null);
        assert.equal(surface.publishedFrame, acquiredFrameId);
        assert.equal(surface.presented, true);
        assert.equal(activeImports.nepl_gui_web.video_memory_acquire_write_slot(createdSurfaceId), HOST_INVALID_COMMAND);
        assert.equal(
            activeImports.nepl_gui_web.video_memory_write_rgba8888_row(createdSurfaceId, acquiredFrameId, 0, 0, 8, 0),
            HOST_INVALID_COMMAND,
        );
        assert.equal(
            activeImports.nepl_gui_web.video_memory_publish_slot(createdSurfaceId, acquiredFrameId, 1, 0, 0, 0, 0),
            HOST_INVALID_COMMAND,
        );
        assert.equal(activeImports.nepl_gui_web.video_memory_present_surface(1, 0, 0, createdSurfaceId), HOST_INVALID_COMMAND);
        assert.equal(activeImports.nepl_gui_web.video_memory_close_surface(createdSurfaceId), HOST_INVALID_COMMAND);
    };

    return {
        importsFactory,
        verify,
    };
}

async function runWebGuiVideoMemoryFakeHostHarnessRegression() {
    const source = readRepoFile("examples", "gui_video_memory_rows.nepl");
    assert.doesNotMatch(source, /argv: \["--contract"\][\s\S]*main[\s\S]*runtimeImportsFactory/);
    const fakeHost = createVideoMemoryFakeHost();
    const result = await runSingle({
        id: "examples/gui_video_memory_rows.nepl/fake-host-happy-path",
        source,
        file: path.resolve(__dirname, "..", "examples", "gui_video_memory_rows.nepl"),
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory: fakeHost.importsFactory,
    });
    fakeHost.verify(result);
    return {
        ok: true,
        checks: [
            "normal NEPL/Wasm path executes without --contract",
            "fake nepl_gui_web host validates create/acquire/write-row/publish/present/close ordering",
            "row writer reads and checks RGBA8888 bytes from Wasm memory",
            "default run_test unsupported stubs remain opt-out from this focused harness",
        ],
    };
}

if (require.main === module) {
    runWebGuiVideoMemoryFakeHostHarnessRegression().then((result) => {
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    }).catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    runWebGuiVideoMemoryFakeHostHarnessRegression,
};
