#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

async function loadDistModule(relPath) {
    const modulePath = path.resolve(__dirname, "..", "web", "dist_ts", ...relPath.split("/"));
    return import(pathToFileURL(modulePath).href);
}

async function runWebGuiCompositorTilePresentHostImportRegression() {
    const host = await loadDistModule("gui-preview/compositor-tile-present-host.js");
    const videoMemory = await loadDistModule("gui-preview/video-memory-surface.js");
    const abi = await loadDistModule("gui-preview/video-memory-host-abi.js");

    verifyFullFrameSinglePacket(host, videoMemory, abi);
    verifyFullFrameMultiBatch(host, videoMemory, abi);
    verifyPartialDirtyCopiesSnapshot(host, videoMemory, abi);
    verifyPartialDirtyMultiBatchAndValidation(host, videoMemory, abi);
    verifyRunOrderAndLifecycleFailures(host, videoMemory, abi);
    verifyUnsupportedAndResourceFailures(host, videoMemory, abi);

    return {
        ok: true,
        checks: [
            "Web compositor host import writes RLE runs into a GuiVideoMemorySurface write slot",
            "end publishes exactly after all full-frame batches complete",
            "row-crossing run offsets are validated as contiguous tile-local pixels",
            "partial dirty rows copy the last published snapshot before writing changed rows",
            "partial dirty validates absolute metadata batch and row ranges",
            "Offscreen and Device targets fail closed instead of routing to Window",
            "no writable slot and lifecycle violations return typed host statuses",
        ],
    };
}

function verifyFullFrameSinglePacket(host, videoMemory, abi) {
    const record = createHostSurfaceRecord(host, videoMemory, 101, 3, 2, 2);
    const descriptor = makeDescriptor({
        surfaceHandle: 101,
        frameId: 7,
        width: 3,
        height: 2,
        tileRows: 2,
        totalRunCount: 2,
    });
    assert.equal(host.beginGuiWebCompositorTilePresent(record, descriptor), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(record, descriptor, {
        pixelOffset: 0,
        pixelCount: 4,
        r: 10,
        g: 20,
        b: 30,
        a: 255,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(record, descriptor, {
        pixelOffset: 4,
        pixelCount: 2,
        r: 200,
        g: 180,
        b: 160,
        a: 128,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    const ended = host.endGuiWebCompositorTilePresent(record, descriptor);
    assert.equal(ended.kind, "present");
    assert.equal(ended.windowId, descriptor.windowId);
    assert.equal(ended.title, "NEPL compositor 101");
    assert.equal(record.frames.length, 0);

    const read = videoMemory.acquireGuiVideoMemoryReadSlot(record.surface);
    assert.equal(read.kind, "ok");
    assert.deepEqual(read.value.dirty, { kind: "full" });
    assertPixelBytes(read.value.pixels, [
        10, 20, 30, 255,
        10, 20, 30, 255,
        10, 20, 30, 255,
        10, 20, 30, 255,
        200, 180, 160, 128,
        200, 180, 160, 128,
    ]);
    assert.equal(videoMemory.releaseGuiVideoMemoryReadSlot(read.value).kind, "ok");
}

function verifyFullFrameMultiBatch(host, videoMemory, abi) {
    const record = createHostSurfaceRecord(host, videoMemory, 102, 2, 4, 2);
    const first = makeDescriptor({
        surfaceHandle: 102,
        frameId: 8,
        width: 2,
        height: 4,
        batchIndex: 0,
        metadataMaxRowsPerBatch: 2,
        tileRows: 2,
        totalRunCount: 1,
    });
    const second = makeDescriptor({
        surfaceHandle: 102,
        frameId: 8,
        width: 2,
        height: 4,
        batchIndex: 1,
        metadataMaxRowsPerBatch: 2,
        tileRows: 2,
        totalRunCount: 1,
    });
    assert.equal(host.beginGuiWebCompositorTilePresent(record, first), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(record, first, {
        pixelOffset: 0,
        pixelCount: 4,
        r: 1,
        g: 2,
        b: 3,
        a: 255,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.deepEqual(
        host.endGuiWebCompositorTilePresent(record, first),
        { kind: "status", status: abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK },
    );
    assert.equal(record.frames.length, 1);

    assert.equal(host.beginGuiWebCompositorTilePresent(record, second), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(record, second, {
        pixelOffset: 0,
        pixelCount: 4,
        r: 9,
        g: 8,
        b: 7,
        a: 200,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    const ended = host.endGuiWebCompositorTilePresent(record, second);
    assert.equal(ended.kind, "present");

    const read = videoMemory.acquireGuiVideoMemoryReadSlot(record.surface);
    assert.equal(read.kind, "ok");
    assertPixelBytes(read.value.pixels, [
        1, 2, 3, 255,
        1, 2, 3, 255,
        1, 2, 3, 255,
        1, 2, 3, 255,
        9, 8, 7, 200,
        9, 8, 7, 200,
        9, 8, 7, 200,
        9, 8, 7, 200,
    ]);
    assert.equal(videoMemory.releaseGuiVideoMemoryReadSlot(read.value).kind, "ok");
}

function verifyPartialDirtyCopiesSnapshot(host, videoMemory, abi) {
    const record = createHostSurfaceRecord(host, videoMemory, 107, 3, 3, 2);
    const full = makeDescriptor({
        surfaceHandle: 107,
        frameId: 20,
        width: 3,
        height: 3,
        tileRows: 3,
        totalRunCount: 1,
    });
    assert.equal(host.beginGuiWebCompositorTilePresent(record, full), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(record, full, {
        pixelOffset: 0,
        pixelCount: 9,
        r: 4,
        g: 5,
        b: 6,
        a: 255,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.endGuiWebCompositorTilePresent(record, full).kind, "present");
    const baselineRead = videoMemory.acquireGuiVideoMemoryReadSlot(record.surface);
    assert.equal(baselineRead.kind, "ok");
    assert.equal(videoMemory.releaseGuiVideoMemoryReadSlot(baselineRead.value).kind, "ok");

    const partial = makeDescriptor({
        surfaceHandle: 107,
        frameId: 21,
        width: 3,
        height: 3,
        metadataRowStart: 1,
        metadataRowCount: 1,
        tileRows: 1,
        totalRunCount: 1,
    });
    assert.equal(host.beginGuiWebCompositorTilePresent(record, partial), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(record, partial, {
        pixelOffset: 0,
        pixelCount: 3,
        r: 90,
        g: 80,
        b: 70,
        a: 255,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    const ended = host.endGuiWebCompositorTilePresent(record, partial);
    assert.equal(ended.kind, "present");
    const read = videoMemory.acquireGuiVideoMemoryReadSlot(record.surface);
    assert.equal(read.kind, "ok");
    assert.deepEqual(read.value.dirty, { kind: "rect", x: 0, y: 1, width: 3, height: 1 });
    assertPixelBytes(read.value.pixels, [
        4, 5, 6, 255,
        4, 5, 6, 255,
        4, 5, 6, 255,
        90, 80, 70, 255,
        90, 80, 70, 255,
        90, 80, 70, 255,
        4, 5, 6, 255,
        4, 5, 6, 255,
        4, 5, 6, 255,
    ]);
    assert.equal(videoMemory.releaseGuiVideoMemoryReadSlot(read.value).kind, "ok");
}

function verifyPartialDirtyMultiBatchAndValidation(host, videoMemory, abi) {
    const record = createHostSurfaceRecord(host, videoMemory, 108, 2, 5, 2);
    const full = makeDescriptor({
        surfaceHandle: 108,
        frameId: 30,
        width: 2,
        height: 5,
        tileRows: 5,
        totalRunCount: 1,
    });
    assert.equal(host.beginGuiWebCompositorTilePresent(record, full), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(record, full, {
        pixelOffset: 0,
        pixelCount: 10,
        r: 7,
        g: 8,
        b: 9,
        a: 255,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.endGuiWebCompositorTilePresent(record, full).kind, "present");
    const baselineRead = videoMemory.acquireGuiVideoMemoryReadSlot(record.surface);
    assert.equal(baselineRead.kind, "ok");
    assert.equal(videoMemory.releaseGuiVideoMemoryReadSlot(baselineRead.value).kind, "ok");

    const wrongBatchCount = makeDescriptor({
        surfaceHandle: 108,
        frameId: 31,
        width: 2,
        height: 5,
        metadataRowStart: 1,
        metadataRowCount: 3,
        metadataMaxRowsPerBatch: 2,
        metadataBatchCount: 1,
        tileRows: 1,
        totalRunCount: 1,
    });
    assert.equal(host.beginGuiWebCompositorTilePresent(record, wrongBatchCount), abi.GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT);

    const wrongPlanStart = makeDescriptor({
        surfaceHandle: 108,
        frameId: 31,
        width: 2,
        height: 5,
        metadataRowStart: 1,
        metadataRowCount: 3,
        metadataMaxRowsPerBatch: 2,
        batchIndex: 1,
        planRowStart: 2,
        tileRows: 1,
        totalRunCount: 1,
    });
    assert.equal(host.beginGuiWebCompositorTilePresent(record, wrongPlanStart), abi.GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT);

    const rowOutsideMetadata = makeDescriptor({
        surfaceHandle: 108,
        frameId: 31,
        width: 2,
        height: 5,
        metadataRowStart: 1,
        metadataRowCount: 3,
        metadataMaxRowsPerBatch: 2,
        planRowStart: 0,
        tileRows: 1,
        totalRunCount: 1,
    });
    assert.equal(host.beginGuiWebCompositorTilePresent(record, rowOutsideMetadata), abi.GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT);

    const first = makeDescriptor({
        surfaceHandle: 108,
        frameId: 31,
        width: 2,
        height: 5,
        metadataRowStart: 1,
        metadataRowCount: 3,
        metadataMaxRowsPerBatch: 2,
        batchIndex: 0,
        tileRows: 2,
        totalRunCount: 1,
    });
    const second = makeDescriptor({
        surfaceHandle: 108,
        frameId: 31,
        width: 2,
        height: 5,
        metadataRowStart: 1,
        metadataRowCount: 3,
        metadataMaxRowsPerBatch: 2,
        batchIndex: 1,
        tileRows: 1,
        totalRunCount: 1,
    });
    assert.equal(host.beginGuiWebCompositorTilePresent(record, first), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(record, first, {
        pixelOffset: 0,
        pixelCount: 4,
        r: 20,
        g: 30,
        b: 40,
        a: 255,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.deepEqual(
        host.endGuiWebCompositorTilePresent(record, first),
        { kind: "status", status: abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK },
    );
    assert.equal(host.beginGuiWebCompositorTilePresent(record, second), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(record, second, {
        pixelOffset: 0,
        pixelCount: 2,
        r: 50,
        g: 60,
        b: 70,
        a: 255,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    const ended = host.endGuiWebCompositorTilePresent(record, second);
    assert.equal(ended.kind, "present");
    const read = videoMemory.acquireGuiVideoMemoryReadSlot(record.surface);
    assert.equal(read.kind, "ok");
    assert.deepEqual(read.value.dirty, { kind: "rect", x: 0, y: 1, width: 2, height: 3 });
    assertPixelBytes(read.value.pixels, [
        7, 8, 9, 255,
        7, 8, 9, 255,
        20, 30, 40, 255,
        20, 30, 40, 255,
        20, 30, 40, 255,
        20, 30, 40, 255,
        50, 60, 70, 255,
        50, 60, 70, 255,
        7, 8, 9, 255,
        7, 8, 9, 255,
    ]);
    assert.equal(videoMemory.releaseGuiVideoMemoryReadSlot(read.value).kind, "ok");
}

function verifyRunOrderAndLifecycleFailures(host, videoMemory, abi) {
    const record = createHostSurfaceRecord(host, videoMemory, 103, 3, 2, 2);
    const descriptor = makeDescriptor({
        surfaceHandle: 103,
        frameId: 9,
        width: 3,
        height: 2,
        totalRunCount: 1,
    });
    assert.deepEqual(
        host.endGuiWebCompositorTilePresent(record, descriptor),
        { kind: "status", status: abi.GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT },
    );
    assert.equal(host.beginGuiWebCompositorTilePresent(record, descriptor), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.beginGuiWebCompositorTilePresent(record, descriptor), abi.GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT);
    assert.equal(host.beginGuiWebCompositorTilePresent(record, makeDescriptor({
        surfaceHandle: 103,
        frameId: 10,
        width: 3,
        height: 2,
        totalRunCount: 1,
    })), abi.GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT);
    assert.equal(host.runGuiWebCompositorTilePresent(record, descriptor, {
        pixelOffset: 1,
        pixelCount: 1,
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT);
}

function verifyUnsupportedAndResourceFailures(host, videoMemory, abi) {
    const unsupportedTarget = createHostSurfaceRecord(host, videoMemory, 104, 2, 2, 2);
    assert.equal(
        host.beginGuiWebCompositorTilePresent(unsupportedTarget, makeDescriptor({
            targetKind: 2,
            surfaceHandle: 104,
            frameId: 10,
            width: 2,
            height: 2,
        })),
        abi.GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED,
    );
    assert.equal(
        host.beginGuiWebCompositorTilePresent(unsupportedTarget, makeDescriptor({
            targetKind: 3,
            surfaceHandle: 104,
            frameId: 10,
            width: 2,
            height: 2,
        })),
        abi.GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED,
    );

    const partialDirty = createHostSurfaceRecord(host, videoMemory, 105, 2, 2, 2);
    assert.equal(
        host.beginGuiWebCompositorTilePresent(partialDirty, makeDescriptor({
            surfaceHandle: 105,
            frameId: 11,
            width: 2,
            height: 2,
            metadataRowStart: 1,
            metadataRowCount: 1,
        })),
        abi.GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT,
    );

    const noWritable = createHostSurfaceRecord(host, videoMemory, 106, 1, 1, 2);
    const first = makeDescriptor({
        surfaceHandle: 106,
        frameId: 12,
        width: 1,
        height: 1,
        totalRunCount: 1,
    });
    const second = makeDescriptor({
        surfaceHandle: 106,
        frameId: 13,
        width: 1,
        height: 1,
        totalRunCount: 1,
    });
    assert.equal(host.beginGuiWebCompositorTilePresent(noWritable, first), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(noWritable, first, {
        pixelOffset: 0,
        pixelCount: 1,
        r: 1,
        g: 1,
        b: 1,
        a: 255,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.endGuiWebCompositorTilePresent(noWritable, first).kind, "present");
    assert.equal(host.beginGuiWebCompositorTilePresent(noWritable, second), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.runGuiWebCompositorTilePresent(noWritable, second, {
        pixelOffset: 0,
        pixelCount: 1,
        r: 2,
        g: 2,
        b: 2,
        a: 255,
    }), abi.GUI_VIDEO_MEMORY_HOST_STATUS_OK);
    assert.equal(host.endGuiWebCompositorTilePresent(noWritable, second).kind, "present");
    assert.equal(host.beginGuiWebCompositorTilePresent(noWritable, makeDescriptor({
        surfaceHandle: 106,
        frameId: 14,
        width: 1,
        height: 1,
        totalRunCount: 1,
    })), abi.GUI_VIDEO_MEMORY_HOST_STATUS_NO_WRITABLE_SLOT);
}

function createHostSurfaceRecord(host, videoMemory, handle, width, height, slotCount) {
    const created = videoMemory.createGuiVideoMemorySurface(width, height, slotCount);
    assert.equal(created.kind, "ok");
    return host.createGuiWebVideoMemoryHostSurfaceRecord(handle, created.value);
}

function makeDescriptor(overrides) {
    const width = overrides.width ?? 3;
    const height = overrides.height ?? 2;
    const batchIndex = overrides.batchIndex ?? 0;
    const tileIndex = overrides.tileIndex ?? 0;
    const metadataRowStart = overrides.metadataRowStart ?? 0;
    const metadataRowCount = overrides.metadataRowCount ?? height;
    const metadataMaxRowsPerBatch = overrides.metadataMaxRowsPerBatch ?? metadataRowCount;
    const planRowStart = overrides.planRowStart ?? metadataRowStart + batchIndex * metadataMaxRowsPerBatch;
    const planRowCount = overrides.planRowCount ?? Math.min(metadataMaxRowsPerBatch, metadataRowStart + metadataRowCount - planRowStart);
    const tileRows = overrides.tileRows ?? planRowCount;
    const tileCount = overrides.tileCount ?? Math.ceil(planRowCount / tileRows);
    const rowStart = overrides.rowStart ?? planRowStart + tileIndex * tileRows;
    const rowCount = overrides.rowCount ?? Math.min(tileRows, planRowStart + planRowCount - rowStart);
    const totalRunCount = overrides.totalRunCount ?? 1;
    return {
        targetKind: overrides.targetKind ?? 1,
        windowId: overrides.windowId ?? 1,
        surfaceHandle: overrides.surfaceHandle,
        frameId: overrides.frameId,
        packetFrameId: overrides.packetFrameId ?? overrides.frameId,
        batchIndex,
        tileIndex,
        planRowStart,
        planRowCount,
        rowStart,
        rowCount,
        width,
        height,
        strideBytes: overrides.strideBytes ?? width * 4,
        tileRows,
        tileCount,
        pixelCount: overrides.pixelCount ?? rowCount * width,
        totalRunCount,
        encodedByteCount: overrides.encodedByteCount ?? totalRunCount * 12,
        metadataFrameId: overrides.metadataFrameId ?? overrides.frameId,
        metadataWidth: overrides.metadataWidth ?? width,
        metadataHeight: overrides.metadataHeight ?? height,
        metadataRowStart,
        metadataRowCount,
        metadataBatchCount: overrides.metadataBatchCount ?? Math.ceil(metadataRowCount / metadataMaxRowsPerBatch),
        metadataMaxRowsPerBatch,
    };
}

function assertPixelBytes(actual, expected) {
    assert.deepEqual(Array.from(actual.slice(0, expected.length)), expected);
}

if (require.main === module) {
    runWebGuiCompositorTilePresentHostImportRegression().then((result) => {
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    }).catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    runWebGuiCompositorTilePresentHostImportRegression,
};
