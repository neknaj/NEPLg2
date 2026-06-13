#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");
const { Worker } = require("node:worker_threads");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

async function loadVideoMemorySurfaceModule() {
    const modulePath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "video-memory-surface.js");
    return import(pathToFileURL(modulePath).href);
}

async function runWebGuiVideoMemorySurfaceRegression() {
    const videoMemory = await loadVideoMemorySurfaceModule();
    const source = readRepoFile("web", "src", "gui-preview", "video-memory-surface.ts");
    const specSource = readRepoFile("doc", "neplg2", "gui_redesign_spec.md");
    const designSource = readRepoFile("doc", "neplg2", "gui_redesign_detailed_design.md");
    const planSource = readRepoFile("doc", "neplg2", "gui_redesign_implementation_plan.md");

    assert.match(source, /GUI_VIDEO_MEMORY_MIN_SLOT_COUNT = 2/);
    assert.match(source, /HEADER_STRIDE_BYTES = 4/);
    assert.match(source, /HEADER_FORMAT = 5/);
    assert.match(source, /HEADER_GENERATION = 6/);
    assert.match(source, /HEADER_SLOT_COUNT = 7/);
    assert.match(source, /HEADER_PUBLISHED_EPOCH = 8/);
    assert.match(source, /HEADER_PRESENTED_EPOCH = 9/);
    assert.match(source, /HEADER_SURFACE_STATE = 10/);
    assert.match(source, /HEADER_INT32_LENGTH = 12/);
    assert.match(source, /HEADER_PIXEL_PLANE_BYTE_OFFSET = 13/);
    assert.match(source, /HEADER_PIXEL_PLANE_BYTE_LENGTH = 14/);
    assert.match(source, /SharedArrayBuffer/);
    assert.match(source, /GUI_VIDEO_MEMORY_SLOT_FREE/);
    assert.match(source, /GUI_VIDEO_MEMORY_SLOT_WRITING/);
    assert.match(source, /GUI_VIDEO_MEMORY_SLOT_PUBLISHED/);
    assert.match(source, /GUI_VIDEO_MEMORY_SLOT_READING/);
    assert.match(source, /GUI_VIDEO_MEMORY_SLOT_CLOSED/);
    assert.match(source, /Atomics\.compareExchange/);
    assert.match(source, /Atomics\.store/);
    assert.match(source, /Atomics\.notify/);
    assert.match(source, /Atomics\.wait/);
    assert.match(source, /acquireGuiVideoMemoryWriteSlot/);
    assert.match(source, /publishGuiVideoMemoryWriteSlot/);
    assert.match(source, /acquireGuiVideoMemoryReadSlot/);
    assert.match(source, /releaseGuiVideoMemoryReadSlot/);
    assert.match(source, /writer-closed/);
    assert.match(source, /presenter-unavailable/);
    assert.match(source, /stale-resize-generation/);
    assert.match(source, /invalid-surface-config/);
    assert.match(source, /invalid-buffer-length/);
    assert.match(source, /shared-buffer-unavailable/);
    assert.match(source, /unsupported-header-version/);
    assert.match(source, /invalid-header-layout/);
    assert.match(source, /wait-unavailable/);
    assert.match(source, /unsupported-command/);
    assert.match(source, /const beforeWait = acquireGuiVideoMemoryReadSlot/);
    assert.doesNotMatch(source, /fallback/i);

    assert.match(specSource, /2 個以上/);
    assert.match(designSource, /Free[\s\S]*Writing[\s\S]*Published/);
    assert.match(designSource, /Published[\s\S]*Reading[\s\S]*Free/);
    assert.match(planSource, /単一 buffer の共有読み書きは禁止/);

    const invalidSurfaceConfig = videoMemory.createGuiVideoMemorySurface(0, 2, 2);
    assert.equal(invalidSurfaceConfig.kind, "err");
    assert.equal(invalidSurfaceConfig.error.kind, "invalid-surface-config");
    const invalidSlotConfig = videoMemory.createGuiVideoMemorySurface(1, 1, 1);
    assert.equal(invalidSlotConfig.kind, "err");
    assert.equal(invalidSlotConfig.error.kind, "invalid-surface-config");

    const shortOpen = videoMemory.openGuiVideoMemorySurface(new SharedArrayBuffer(4));
    assert.equal(shortOpen.kind, "err");
    assert.equal(shortOpen.error.kind, "invalid-buffer-length");

    const created = videoMemory.createGuiVideoMemorySurface(3, 2, 2);
    assert.equal(created.kind, "ok");
    const surface = created.value;
    const header = new Int32Array(surface.buffer, 0, 16);
    assert.equal(header[0], videoMemory.GUI_VIDEO_MEMORY_MAGIC);
    assert.equal(header[1], videoMemory.GUI_VIDEO_MEMORY_VERSION);
    assert.equal(header[2], 3);
    assert.equal(header[3], 2);
    assert.equal(header[4], 12);
    assert.equal(header[5], videoMemory.GUI_VIDEO_MEMORY_FORMAT_RGBA8888);
    assert.equal(header[6], 1);
    assert.equal(header[7], 2);
    assert.equal(header[8], 0);
    assert.equal(header[9], 0);
    assert.equal(header[10], videoMemory.GUI_VIDEO_MEMORY_SURFACE_READY);
    assert.equal(header[12], 16);
    assert.equal(header[13], 16 * 4 + 2 * 8 * 4);
    assert.equal(header[14], 24);

    const opened = videoMemory.openGuiVideoMemorySurface(surface.buffer);
    assert.equal(opened.kind, "ok");
    assert.equal(opened.value.strideBytes, 12);
    assert.equal(opened.value.pixelPlaneByteOffset, 128);

    const invalidFormat = videoMemory.createGuiVideoMemorySurface(1, 1, 2);
    assert.equal(invalidFormat.kind, "ok");
    const invalidFormatHeader = new Int32Array(invalidFormat.value.buffer, 0, 16);
    Atomics.store(invalidFormatHeader, 5, 999);
    const invalidFormatOpen = videoMemory.openGuiVideoMemorySurface(invalidFormat.value.buffer);
    assert.equal(invalidFormatOpen.kind, "err");
    assert.equal(invalidFormatOpen.error.kind, "unsupported-pixel-format");

    const invalidLayout = videoMemory.createGuiVideoMemorySurface(1, 1, 2);
    assert.equal(invalidLayout.kind, "ok");
    const invalidLayoutHeader = new Int32Array(invalidLayout.value.buffer, 0, 16);
    Atomics.store(invalidLayoutHeader, 13, 16 * 4);
    const invalidLayoutOpen = videoMemory.openGuiVideoMemorySurface(invalidLayout.value.buffer);
    assert.equal(invalidLayoutOpen.kind, "err");
    assert.equal(invalidLayoutOpen.error.kind, "invalid-header-layout");

    const writeSlot = videoMemory.acquireGuiVideoMemoryWriteSlot(surface);
    assert.equal(writeSlot.kind, "ok");
    writeSlot.value.pixels[0] = 10;
    writeSlot.value.pixels[1] = 20;
    writeSlot.value.pixels[2] = 30;
    writeSlot.value.pixels[3] = 255;
    const published = videoMemory.publishGuiVideoMemoryWriteSlot(writeSlot.value, { kind: "full" });
    assert.equal(published.kind, "ok");
    const waited = videoMemory.waitForGuiVideoMemoryReadSlot(surface, 0);
    assert.equal(waited.kind, "ok");
    assert.equal(waited.value.kind, "slot");
    assert.equal(waited.value.slot.pixels[0], 10);
    const released = videoMemory.releaseGuiVideoMemoryReadSlot(waited.value.slot);
    assert.equal(released.kind, "ok");
    assert.equal(header[9], waited.value.slot.epoch);

    const waitedFromWorker = await waitForWorkerPublishedSlot(videoMemory, path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "video-memory-surface.js"));
    assert.equal(waitedFromWorker.pixel0, 44);
    assert.equal(waitedFromWorker.presentedEpoch, waitedFromWorker.epoch);

    return {
        ok: true,
        checks: [
            "Web GUI video memory surface requires at least two pixel slots",
            "writer and presenter use explicit Atomics ownership transitions",
            "video memory errors are typed instead of falling back silently",
            "malformed shared buffers return typed errors instead of JavaScript exceptions",
            "invalid surface creation config returns typed errors instead of clamping dimensions",
            "implementation header ABI matches the documented video memory layout",
            "wait path receives a frame published from another worker thread",
            "docs and implementation agree that one shared pixel plane is forbidden",
        ],
    };
}

async function waitForWorkerPublishedSlot(videoMemory, modulePath) {
    const created = videoMemory.createGuiVideoMemorySurface(2, 2, 2);
    assert.equal(created.kind, "ok");
    const surface = created.value;
    const worker = new Worker(`
        const { workerData, parentPort } = require("node:worker_threads");
        (async () => {
            const api = await import(workerData.moduleHref);
            const opened = api.openGuiVideoMemorySurface(workerData.buffer);
            if (opened.kind === "err") {
                parentPort.postMessage({ kind: "err", error: opened.error.kind });
                return;
            }
            setTimeout(() => {
                const write = api.acquireGuiVideoMemoryWriteSlot(opened.value);
                if (write.kind === "err") {
                    parentPort.postMessage({ kind: "err", error: write.error.kind });
                    return;
                }
                write.value.pixels[0] = 44;
                write.value.pixels[1] = 55;
                write.value.pixels[2] = 66;
                write.value.pixels[3] = 255;
                const published = api.publishGuiVideoMemoryWriteSlot(write.value, { kind: "full" });
                parentPort.postMessage(published.kind === "ok"
                    ? { kind: "ok" }
                    : { kind: "err", error: published.error.kind });
            }, 25);
        })().catch((error) => parentPort.postMessage({ kind: "err", error: String(error && error.stack ? error.stack : error) }));
    `, {
        eval: true,
        workerData: {
            moduleHref: pathToFileURL(modulePath).href,
            buffer: surface.buffer,
        },
    });
    try {
        const workerResultPromise = new Promise((resolve, reject) => {
            worker.once("message", resolve);
            worker.once("error", reject);
        });
        const waited = videoMemory.waitForGuiVideoMemoryReadSlot(surface, 2000);
        assert.equal(waited.kind, "ok");
        assert.equal(waited.value.kind, "slot");
        const workerResult = await workerResultPromise;
        assert.deepEqual(workerResult, { kind: "ok" });
        const epoch = waited.value.slot.epoch;
        const pixel0 = waited.value.slot.pixels[0];
        const released = videoMemory.releaseGuiVideoMemoryReadSlot(waited.value.slot);
        assert.equal(released.kind, "ok");
        const header = new Int32Array(surface.buffer, 0, 16);
        return { epoch, pixel0, presentedEpoch: header[9] };
    } finally {
        await worker.terminate();
    }
}

if (require.main === module) {
    (async () => {
        const result = runWebGuiVideoMemorySurfaceRegression();
        process.stdout.write(JSON.stringify(await result, null, 2) + "\n");
    })().catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    runWebGuiVideoMemorySurfaceRegression,
};
