#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

async function loadHostAbiModule() {
    const modulePath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "video-memory-host-abi.js");
    return import(pathToFileURL(modulePath).href);
}

async function runWebGuiVideoMemoryHostImportRegression() {
    const abi = await loadHostAbiModule();
    const workerSource = readRepoFile("web", "src", "runtime", "worker.ts");
    const shellSource = readRepoFile("web", "src", "terminal", "shell.ts");
    const surfaceSource = readRepoFile("stdlib", "platforms", "gui", "web", "surface.nepl");
    const surfaceCode = stripNeplLineComments(surfaceSource);
    const runTestSource = readRepoFile("nodesrc", "run_test.js");
    const designSource = readRepoFile("doc", "neplg2", "gui_redesign_detailed_design.md");
    const implementationPlanSource = readRepoFile("doc", "neplg2", "gui_redesign_implementation_plan.md");

    const ack = abi.createGuiVideoMemoryHostAckBuffer();
    assert.equal(ack.kind, "ok");
    abi.resolveGuiVideoMemoryHostAck(ack.value, abi.GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT);
    assert.equal(
        abi.waitGuiVideoMemoryHostAck(ack.value, 0),
        abi.GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT,
    );

    assert.match(workerSource, /video_memory_create_surface: this\.nepl_gui_web_video_memory_create_surface\.bind\(this\)/);
    assert.match(workerSource, /video_memory_acquire_write_slot: this\.nepl_gui_web_video_memory_acquire_write_slot\.bind\(this\)/);
    assert.match(workerSource, /video_memory_write_slot_bytes: this\.nepl_gui_web_video_memory_write_slot_bytes\.bind\(this\)/);
    assert.match(workerSource, /video_memory_discard_write_slot: this\.nepl_gui_web_video_memory_discard_write_slot\.bind\(this\)/);
    assert.match(workerSource, /video_memory_publish_slot: this\.nepl_gui_web_video_memory_publish_slot\.bind\(this\)/);
    assert.match(workerSource, /video_memory_present_surface: this\.nepl_gui_web_video_memory_present_surface\.bind\(this\)/);
    assert.match(workerSource, /video_memory_close_surface: this\.nepl_gui_web_video_memory_close_surface\.bind\(this\)/);
    assert.doesNotMatch(workerSource, /video_memory_open_surface:/);
    assert.doesNotMatch(workerSource, /video_memory_acquire_frame:/);
    assert.doesNotMatch(workerSource, /video_memory_copy_rgba8888:/);
    assert.doesNotMatch(workerSource, /video_memory_publish_frame:/);
    assert.doesNotMatch(workerSource, /video-memory-presenter/);
    assert.doesNotMatch(workerSource, /runtime-bridge/);

    const workerImportPath = extractClassSlice(
        workerSource,
        "nepl_gui_web_video_memory_create_surface",
        "private storeGuiWebInputEventTakeResult",
    );
    assert.match(workerImportPath, /createGuiVideoMemorySurface/);
    assert.match(workerImportPath, /acquireGuiVideoMemoryWriteSlot/);
    assert.match(workerImportPath, /discardGuiVideoMemoryWriteSlot/);
    assert.match(workerImportPath, /publishGuiVideoMemoryWriteSlot/);
    assert.doesNotMatch(workerImportPath, /presentGuiWebRuntimeVideoMemory/);
    assert.doesNotMatch(workerImportPath, /presentGuiWebRuntimeFrame/);
    assert.doesNotMatch(workerImportPath, /presentCommands|beginFrame|pushCommand|endFrame/);
    assert.doesNotMatch(workerImportPath, /stdout|GuiWebStdoutProtocol|parse/);
    assert.doesNotMatch(workerImportPath, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);
    assert.doesNotMatch(workerImportPath, /\b(?:drawImage|putImageData|fillRect|clearRect|scale|setTransform)\b/);
    assert.doesNotMatch(workerImportPath, /postMessage\(message,\s*\[/);
    assert.doesNotMatch(workerImportPath, /\bArrayBuffer\b(?!\s*;|\s*\|)/);

    const discardWriteSlotMethod = extractClassSlice(
        workerSource,
        "nepl_gui_web_video_memory_discard_write_slot",
        "nepl_gui_web_video_memory_publish_slot",
    );
    assert.match(discardWriteSlotMethod, /discardGuiVideoMemoryWriteSlot/);
    assert.match(discardWriteSlotMethod, /surface\.frames = surface\.frames\.filter/);
    assert.match(discardWriteSlotMethod, /return GUI_VIDEO_MEMORY_HOST_STATUS_OK/);
    assert.doesNotMatch(discardWriteSlotMethod, /publishGuiVideoMemoryWriteSlot/);
    assert.doesNotMatch(discardWriteSlotMethod, /presentGuiVideoMemorySurface/);
    assert.doesNotMatch(discardWriteSlotMethod, /presentCommands|beginFrame|pushCommand|endFrame/);
    assert.doesNotMatch(discardWriteSlotMethod, /stdout|GuiWebStdoutProtocol|parse/);
    assert.doesNotMatch(discardWriteSlotMethod, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);
    assert.doesNotMatch(discardWriteSlotMethod, /\b(?:drawImage|putImageData|fillRect|clearRect|scale|setTransform)\b/);

    const publishSlotMethod = extractClassSlice(
        workerSource,
        "nepl_gui_web_video_memory_publish_slot",
        "nepl_gui_web_video_memory_present_surface",
    );
    assert.match(publishSlotMethod, /publishGuiVideoMemoryWriteSlot/);
    assert.match(publishSlotMethod, /return GUI_VIDEO_MEMORY_HOST_STATUS_OK/);
    assert.doesNotMatch(publishSlotMethod, /presentGuiVideoMemorySurface/);

    const presentSurfaceMethod = extractClassSlice(
        workerSource,
        "nepl_gui_web_video_memory_present_surface",
        "nepl_gui_web_video_memory_close_surface",
    );
    assert.match(presentSurfaceMethod, /windowId/);
    assert.match(presentSurfaceMethod, /titlePtr/);
    assert.match(presentSurfaceMethod, /titleLen/);
    assert.match(presentSurfaceMethod, /surfaceHandle/);
    assert.match(presentSurfaceMethod, /decodeGuiVideoMemoryTitle/);
    assert.match(presentSurfaceMethod, /presentGuiVideoMemorySurface\(windowId, title, surface\)/);

    const privatePresentBridge = extractClassSlice(
        workerSource,
        "private presentGuiVideoMemorySurface",
        "private findGuiVideoMemorySurface",
    );
    assert.match(privatePresentBridge, /createGuiVideoMemoryHostAckBuffer/);
    assert.match(privatePresentBridge, /postWorkerMessage\(\{[\s\S]*type: 'gui_video_memory_present'/);
    assert.match(privatePresentBridge, /waitGuiVideoMemoryHostAck\(ack\.value\);/);
    assert.doesNotMatch(privatePresentBridge, /waitGuiVideoMemoryHostAck\(ack\.value,\s*\d+/);
    assert.doesNotMatch(privatePresentBridge, /presentGuiWebRuntimeVideoMemory/);

    assert.match(workerSource, /private decodeGuiVideoMemoryTitle\(ptr: number, len: number\): string \| number/);
    assert.match(workerSource, /this\.memoryBytes\(ptr, len\)/);
    assert.match(workerSource, /TextDecoder\('utf-8', \{ fatal: true \}\)/);

    const shellHandler = extractClassSlice(
        shellSource,
        "private handleGuiVideoMemoryPresentMessage",
        "private configureGuiRuntimeTimer",
    );
    assert.match(shellHandler, /presentGuiWebRuntimeVideoMemory\(\{/);
    assert.match(shellHandler, /resolveGuiVideoMemoryHostAck\(message\.ack, GUI_VIDEO_MEMORY_HOST_STATUS_OK\)/);
    assert.match(shellHandler, /guiVideoMemoryHostStatusFromRuntimeError/);
    assert.match(shellHandler, /this\.guiRuntimeInputWindowIds\.add\(message\.windowId\)/);
    assert.doesNotMatch(shellHandler, /handleGuiStdoutProtocolEvents|presentGuiWebRuntimeFrame|presentCommands|beginFrame/);

    assert.match(surfaceSource, /#extern "nepl_gui_web" "video_memory_create_surface"/);
    assert.match(surfaceSource, /#extern "nepl_gui_web" "video_memory_acquire_write_slot"/);
    assert.match(surfaceSource, /#extern "nepl_gui_web" "video_memory_write_slot_bytes"/);
    assert.match(surfaceSource, /#extern "nepl_gui_web" "video_memory_discard_write_slot"/);
    assert.match(surfaceSource, /#extern "nepl_gui_web" "video_memory_publish_slot"/);
    assert.match(surfaceSource, /#extern "nepl_gui_web" "video_memory_present_surface"/);
    assert.match(surfaceSource, /pub fn gui_web_video_memory_create_surface[\s\S]*Result GuiWebVideoMemorySurface GuiError/);
    assert.match(surfaceSource, /pub fn gui_web_video_memory_write_frame_bytes[\s\S]*Result unit GuiError/);
    assert.match(surfaceSource, /pub fn gui_web_video_memory_discard_write_frame[\s\S]*Result unit GuiError/);
    assert.match(surfaceSource, /pub fn gui_web_video_memory_present_surface[\s\S]*Result unit GuiError/);
    assert.match(surfaceSource, /fn gui_web_video_memory_status_error/);
    assert.doesNotMatch(surfaceSource, /pub fn gui_web_video_memory_status_error/);
    assert.doesNotMatch(surfaceSource, /video_memory_open_surface|video_memory_acquire_frame|video_memory_copy_rgba8888|video_memory_publish_frame/);
    assert.doesNotMatch(surfaceCode, /stdout_protocol|gui_web_stdout|presentCommands|beginFrame|fallback/);

    assert.match(runTestSource, /video_memory_create_surface: \(\) => -1/);
    assert.match(runTestSource, /video_memory_acquire_write_slot: \(\) => -1/);
    assert.match(runTestSource, /video_memory_write_slot_bytes: \(\) => -2/);
    assert.match(runTestSource, /video_memory_discard_write_slot: \(\) => -2/);
    assert.match(runTestSource, /video_memory_present_surface: \(\) => -2/);

    assert.match(designSource, /video memory host import ABI/);
    assert.match(designSource, /video_memory_create_surface/);
    assert.match(designSource, /gui_video_memory_present/);
    assert.match(designSource, /SharedArrayBuffer remains a Web backend detail/);
    assert.match(implementationPlanSource, /Web video memory host import/);
    assert.match(implementationPlanSource, /create_surface[\s\S]*acquire_write_slot[\s\S]*discard_write_slot[\s\S]*present_surface/);

    return {
        ok: true,
        checks: [
            "Web GUI video memory imports are scalar nepl_gui_web host imports",
            "Worker owns SharedArrayBuffer surfaces behind opaque positive ids",
            "discard import releases unpublished write slots without publish or present side effects",
            "publish and present are separate state transitions",
            "present import waits for the main-thread presenter ack before returning status",
            "worker host import path does not use stdout, command-frame, DOM, or Canvas drawing fallback",
            "stdlib Web wrappers map raw negative statuses to Result GuiError",
        ],
    };
}

function extractClassSlice(source, startText, endText) {
    const startNeedle = `    ${startText}`;
    const endNeedle = `    ${endText}`;
    const start = source.indexOf(startNeedle);
    assert.notEqual(start, -1, `${startText} must exist`);
    const end = source.indexOf(endNeedle, start + startNeedle.length);
    assert.notEqual(end, -1, `${endText} must follow ${startText}`);
    return source.slice(start, end);
}

if (require.main === module) {
    runWebGuiVideoMemoryHostImportRegression().then((result) => {
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    }).catch((error) => {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    });
}

module.exports = {
    runWebGuiVideoMemoryHostImportRegression,
};

function stripNeplLineComments(source) {
    return source
        .split(/\r?\n/)
        .filter((line) => !line.trimStart().startsWith("//"))
        .join("\n");
}
