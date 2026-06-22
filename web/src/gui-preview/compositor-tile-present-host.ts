import {
    type GuiVideoMemoryError,
    acquireGuiVideoMemoryWriteSlot,
    publishGuiVideoMemoryWriteSlot,
    type GuiVideoMemorySurface,
    type GuiVideoMemoryWriteSlot,
} from './video-memory-surface.js';
import {
    GUI_VIDEO_MEMORY_HOST_STATUS_BACKEND_FAILURE,
    GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT,
    GUI_VIDEO_MEMORY_HOST_STATUS_NO_WRITABLE_SLOT,
    GUI_VIDEO_MEMORY_HOST_STATUS_OK,
    GUI_VIDEO_MEMORY_HOST_STATUS_RESOURCE_EXHAUSTED,
    GUI_VIDEO_MEMORY_HOST_STATUS_STALE_FRAME,
    GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED,
} from './video-memory-host-abi.js';

const COMPOSITOR_TARGET_WINDOW = 1;
const BYTES_PER_RGBA8888_PIXEL = 4;
const RLE_RECORD_BYTES = 12;

export type GuiWebCompositorTilePresentDescriptor = {
    targetKind: number;
    windowId: number;
    surfaceHandle: number;
    frameId: number;
    packetFrameId: number;
    batchIndex: number;
    tileIndex: number;
    planRowStart: number;
    planRowCount: number;
    rowStart: number;
    rowCount: number;
    width: number;
    height: number;
    strideBytes: number;
    tileRows: number;
    tileCount: number;
    pixelCount: number;
    totalRunCount: number;
    encodedByteCount: number;
    metadataFrameId: number;
    metadataWidth: number;
    metadataHeight: number;
    metadataRowStart: number;
    metadataRowCount: number;
    metadataBatchCount: number;
    metadataMaxRowsPerBatch: number;
};

export type GuiWebCompositorTilePresentRun = {
    pixelOffset: number;
    pixelCount: number;
    r: number;
    g: number;
    b: number;
    a: number;
};

type GuiWebCompositorTilePresentPacketState = {
    batchIndex: number;
    tileIndex: number;
    rowStart: number;
    rowCount: number;
    pixelCount: number;
    totalRunCount: number;
    seenRunCount: number;
    seenPixelCount: number;
};

type GuiWebCompositorTilePresentCurrentPacket =
    | { kind: 'none' }
    | { kind: 'active'; packet: GuiWebCompositorTilePresentPacketState };

type GuiWebCompositorTilePresentBatchState = {
    batchIndex: number;
    tileCount: number;
    completedTileIndices: number[];
};

type GuiWebCompositorTilePresentFrameState = {
    targetKind: typeof COMPOSITOR_TARGET_WINDOW;
    windowId: number;
    surfaceHandle: number;
    frameId: number;
    width: number;
    height: number;
    strideBytes: number;
    metadataBatchCount: number;
    metadataMaxRowsPerBatch: number;
    currentPacket: GuiWebCompositorTilePresentCurrentPacket;
    batches: GuiWebCompositorTilePresentBatchState[];
};

export type GuiWebCompositorTilePresentAttachment =
    | { kind: 'none' }
    | { kind: 'active'; state: GuiWebCompositorTilePresentFrameState };

export type GuiWebVideoMemoryHostFrameRecord = {
    frameId: number;
    slot: GuiVideoMemoryWriteSlot;
    compositor: GuiWebCompositorTilePresentAttachment;
};

type GuiWebActiveCompositorFrameRecord = {
    frameId: number;
    slot: GuiVideoMemoryWriteSlot;
    compositor: { kind: 'active'; state: GuiWebCompositorTilePresentFrameState };
};

export type GuiWebVideoMemoryHostSurfaceRecord = {
    handle: number;
    surface: GuiVideoMemorySurface;
    nextFrameId: number;
    frames: GuiWebVideoMemoryHostFrameRecord[];
};

export type GuiWebCompositorTilePresentEndResult =
    | { kind: 'status'; status: number }
    | { kind: 'present'; windowId: number; title: string; surface: GuiWebVideoMemoryHostSurfaceRecord };

export function createGuiWebVideoMemoryHostSurfaceRecord(
    handle: number,
    surface: GuiVideoMemorySurface,
): GuiWebVideoMemoryHostSurfaceRecord {
    return {
        handle,
        surface,
        nextFrameId: 1,
        frames: [],
    };
}

export function guiWebVideoMemoryHostFrameIsPlain(frame: GuiWebVideoMemoryHostFrameRecord): boolean {
    return frame.compositor.kind === 'none';
}

export function beginGuiWebCompositorTilePresent(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    descriptor: GuiWebCompositorTilePresentDescriptor,
): number {
    const descriptorStatus = validateCompositorDescriptor(surface, descriptor);
    if (descriptorStatus !== GUI_VIDEO_MEMORY_HOST_STATUS_OK) {
        return descriptorStatus;
    }
    let frame = findSurfaceFrame(surface, descriptor.frameId);
    if (!frame) {
        const acquired = acquireGuiVideoMemoryWriteSlot(surface.surface);
        if (acquired.kind === 'err') {
            return guiWebVideoMemoryHostStatusFromError(acquired.error);
        }
        frame = {
            frameId: descriptor.frameId,
            slot: acquired.value,
            compositor: {
                kind: 'active',
                state: initialCompositorFrameState(descriptor),
            },
        };
        surface.frames = [...surface.frames, frame];
        if (descriptor.frameId >= surface.nextFrameId) {
            surface.nextFrameId = descriptor.frameId + 1;
        }
    }
    if (frame.compositor.kind !== 'active') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    const state = frame.compositor.state;
    if (!compositorFrameMatchesDescriptor(state, descriptor)) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    if (state.currentPacket.kind !== 'none') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    if (compositorPacketAlreadyCompleted(state, descriptor.batchIndex, descriptor.tileIndex)) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    state.currentPacket = {
        kind: 'active',
        packet: {
            batchIndex: descriptor.batchIndex,
            tileIndex: descriptor.tileIndex,
            rowStart: descriptor.rowStart,
            rowCount: descriptor.rowCount,
            pixelCount: descriptor.pixelCount,
            totalRunCount: descriptor.totalRunCount,
            seenRunCount: 0,
            seenPixelCount: 0,
        },
    };
    return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
}

export function runGuiWebCompositorTilePresent(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    descriptor: GuiWebCompositorTilePresentDescriptor,
    run: GuiWebCompositorTilePresentRun,
): number {
    const descriptorStatus = validateCompositorDescriptor(surface, descriptor);
    if (descriptorStatus !== GUI_VIDEO_MEMORY_HOST_STATUS_OK) {
        return descriptorStatus;
    }
    if (!areValidColorChannels(run.r, run.g, run.b, run.a)) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    const frame = findActiveCompositorFrame(surface, descriptor);
    if (typeof frame === 'number') {
        return frame;
    }
    const packetState = activePacketForDescriptor(frame.compositor.state, descriptor);
    if (typeof packetState === 'number') {
        return packetState;
    }
    if (
        !isPositiveInteger(run.pixelCount)
        || run.pixelOffset !== packetState.seenPixelCount
        || run.pixelOffset + run.pixelCount > packetState.pixelCount
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    const writeStatus = writeCompositorRun(frame.slot, packetState, descriptor, run);
    if (writeStatus !== GUI_VIDEO_MEMORY_HOST_STATUS_OK) {
        return writeStatus;
    }
    packetState.seenRunCount += 1;
    packetState.seenPixelCount += run.pixelCount;
    return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
}

export function endGuiWebCompositorTilePresent(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    descriptor: GuiWebCompositorTilePresentDescriptor,
): GuiWebCompositorTilePresentEndResult {
    const descriptorStatus = validateCompositorDescriptor(surface, descriptor);
    if (descriptorStatus !== GUI_VIDEO_MEMORY_HOST_STATUS_OK) {
        return { kind: 'status', status: descriptorStatus };
    }
    const frame = findActiveCompositorFrame(surface, descriptor);
    if (typeof frame === 'number') {
        return { kind: 'status', status: frame };
    }
    const packetState = activePacketForDescriptor(frame.compositor.state, descriptor);
    if (typeof packetState === 'number') {
        return { kind: 'status', status: packetState };
    }
    if (
        packetState.seenRunCount !== packetState.totalRunCount
        || packetState.seenPixelCount !== packetState.pixelCount
    ) {
        return { kind: 'status', status: GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT };
    }
    const completedStatus = completeCompositorPacket(frame.compositor.state, descriptor);
    if (completedStatus !== GUI_VIDEO_MEMORY_HOST_STATUS_OK) {
        return { kind: 'status', status: completedStatus };
    }
    frame.compositor.state.currentPacket = { kind: 'none' };
    if (!allCompositorPacketsCompleted(frame.compositor.state)) {
        return { kind: 'status', status: GUI_VIDEO_MEMORY_HOST_STATUS_OK };
    }
    const published = publishGuiVideoMemoryWriteSlot(frame.slot, { kind: 'full' });
    if (published.kind === 'err') {
        return { kind: 'status', status: guiWebVideoMemoryHostStatusFromError(published.error) };
    }
    surface.frames = surface.frames.filter((candidate) => candidate.frameId !== descriptor.frameId);
    return {
        kind: 'present',
        windowId: frame.compositor.state.windowId,
        title: `NEPL compositor ${surface.handle}`,
        surface,
    };
}

function initialCompositorFrameState(
    descriptor: GuiWebCompositorTilePresentDescriptor,
): GuiWebCompositorTilePresentFrameState {
    return {
        targetKind: COMPOSITOR_TARGET_WINDOW,
        windowId: descriptor.windowId,
        surfaceHandle: descriptor.surfaceHandle,
        frameId: descriptor.frameId,
        width: descriptor.width,
        height: descriptor.height,
        strideBytes: descriptor.strideBytes,
        metadataBatchCount: descriptor.metadataBatchCount,
        metadataMaxRowsPerBatch: descriptor.metadataMaxRowsPerBatch,
        currentPacket: { kind: 'none' },
        batches: [],
    };
}

function validateCompositorDescriptor(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    descriptor: GuiWebCompositorTilePresentDescriptor,
): number {
    if (descriptor.targetKind !== COMPOSITOR_TARGET_WINDOW) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED;
    }
    if (
        !isPositiveInteger(descriptor.windowId)
        || descriptor.surfaceHandle !== surface.handle
        || !isPositiveInteger(descriptor.frameId)
        || descriptor.packetFrameId !== descriptor.frameId
        || descriptor.metadataFrameId !== descriptor.frameId
        || descriptor.width !== surface.surface.width
        || descriptor.height !== surface.surface.height
        || descriptor.strideBytes !== surface.surface.strideBytes
        || descriptor.metadataWidth !== descriptor.width
        || descriptor.metadataHeight !== descriptor.height
        || descriptor.strideBytes !== descriptor.width * BYTES_PER_RGBA8888_PIXEL
        || descriptor.encodedByteCount !== descriptor.totalRunCount * RLE_RECORD_BYTES
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    if (
        !isNonNegativeInteger(descriptor.batchIndex)
        || !isNonNegativeInteger(descriptor.tileIndex)
        || !isPositiveInteger(descriptor.planRowCount)
        || !isNonNegativeInteger(descriptor.planRowStart)
        || !isNonNegativeInteger(descriptor.rowStart)
        || !isPositiveInteger(descriptor.rowCount)
        || !isPositiveInteger(descriptor.tileRows)
        || !isPositiveInteger(descriptor.tileCount)
        || !isPositiveInteger(descriptor.pixelCount)
        || !isPositiveInteger(descriptor.totalRunCount)
        || !isPositiveInteger(descriptor.encodedByteCount)
        || !isPositiveInteger(descriptor.metadataBatchCount)
        || !isPositiveInteger(descriptor.metadataMaxRowsPerBatch)
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    if (descriptor.metadataRowStart !== 0 || descriptor.metadataRowCount !== descriptor.height) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED;
    }
    if (
        descriptor.batchIndex >= descriptor.metadataBatchCount
        || descriptor.tileIndex >= descriptor.tileCount
        || descriptor.metadataBatchCount !== expectedTileCount(descriptor.height, descriptor.metadataMaxRowsPerBatch)
        || descriptor.planRowStart + descriptor.planRowCount > descriptor.height
        || descriptor.rowStart + descriptor.rowCount > descriptor.height
        || descriptor.rowStart < descriptor.planRowStart
        || descriptor.rowStart + descriptor.rowCount > descriptor.planRowStart + descriptor.planRowCount
        || descriptor.pixelCount !== descriptor.rowCount * descriptor.width
        || descriptor.tileCount !== expectedTileCount(descriptor.planRowCount, descriptor.tileRows)
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    const expectedPlanRowStart = descriptor.batchIndex * descriptor.metadataMaxRowsPerBatch;
    const remainingRows = descriptor.height - expectedPlanRowStart;
    const expectedPlanRowCount = Math.min(descriptor.metadataMaxRowsPerBatch, remainingRows);
    const expectedRowStart = descriptor.planRowStart + descriptor.tileIndex * descriptor.tileRows;
    const remainingTileRows = descriptor.planRowStart + descriptor.planRowCount - expectedRowStart;
    const expectedRowCount = Math.min(descriptor.tileRows, remainingTileRows);
    if (
        expectedPlanRowStart < 0
        || expectedPlanRowCount <= 0
        || descriptor.planRowStart !== expectedPlanRowStart
        || descriptor.planRowCount !== expectedPlanRowCount
        || descriptor.rowStart !== expectedRowStart
        || descriptor.rowCount !== expectedRowCount
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
}

function compositorFrameMatchesDescriptor(
    state: GuiWebCompositorTilePresentFrameState,
    descriptor: GuiWebCompositorTilePresentDescriptor,
): boolean {
    return state.targetKind === descriptor.targetKind
        && state.windowId === descriptor.windowId
        && state.surfaceHandle === descriptor.surfaceHandle
        && state.frameId === descriptor.frameId
        && state.width === descriptor.width
        && state.height === descriptor.height
        && state.strideBytes === descriptor.strideBytes
        && state.metadataBatchCount === descriptor.metadataBatchCount
        && state.metadataMaxRowsPerBatch === descriptor.metadataMaxRowsPerBatch;
}

function findActiveCompositorFrame(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    descriptor: GuiWebCompositorTilePresentDescriptor,
): GuiWebActiveCompositorFrameRecord | number {
    const frame = findSurfaceFrame(surface, descriptor.frameId);
    if (!frame || frame.compositor.kind !== 'active') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    if (!compositorFrameMatchesDescriptor(frame.compositor.state, descriptor)) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    return frame as GuiWebActiveCompositorFrameRecord;
}

function activePacketForDescriptor(
    state: GuiWebCompositorTilePresentFrameState,
    descriptor: GuiWebCompositorTilePresentDescriptor,
): GuiWebCompositorTilePresentPacketState | number {
    if (state.currentPacket.kind !== 'active') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    const packet = state.currentPacket.packet;
    if (
        packet.batchIndex !== descriptor.batchIndex
        || packet.tileIndex !== descriptor.tileIndex
        || packet.rowStart !== descriptor.rowStart
        || packet.rowCount !== descriptor.rowCount
        || packet.pixelCount !== descriptor.pixelCount
        || packet.totalRunCount !== descriptor.totalRunCount
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    return packet;
}

function completeCompositorPacket(
    state: GuiWebCompositorTilePresentFrameState,
    descriptor: GuiWebCompositorTilePresentDescriptor,
): number {
    const existingBatch = findBatchState(state, descriptor.batchIndex);
    const batch = existingBatch || {
        batchIndex: descriptor.batchIndex,
        tileCount: descriptor.tileCount,
        completedTileIndices: [],
    };
    if (
        batch.tileCount !== descriptor.tileCount
        || batch.completedTileIndices.includes(descriptor.tileIndex)
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    batch.completedTileIndices = [...batch.completedTileIndices, descriptor.tileIndex];
    if (!existingBatch) {
        state.batches = [...state.batches, batch];
    }
    return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
}

function allCompositorPacketsCompleted(state: GuiWebCompositorTilePresentFrameState): boolean {
    if (state.batches.length !== state.metadataBatchCount) {
        return false;
    }
    for (let batchIndex = 0; batchIndex < state.metadataBatchCount; batchIndex += 1) {
        const batch = findBatchState(state, batchIndex);
        if (!batch || batch.completedTileIndices.length !== batch.tileCount) {
            return false;
        }
    }
    return true;
}

function compositorPacketAlreadyCompleted(
    state: GuiWebCompositorTilePresentFrameState,
    batchIndex: number,
    tileIndex: number,
): boolean {
    const batch = findBatchState(state, batchIndex);
    return !!batch && batch.completedTileIndices.includes(tileIndex);
}

function findBatchState(
    state: GuiWebCompositorTilePresentFrameState,
    batchIndex: number,
): GuiWebCompositorTilePresentBatchState | null {
    for (const batch of state.batches) {
        if (batch.batchIndex === batchIndex) {
            return batch;
        }
    }
    return null;
}

function findSurfaceFrame(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    frameId: number,
): GuiWebVideoMemoryHostFrameRecord | null {
    for (const frame of surface.frames) {
        if (frame.frameId === frameId) {
            return frame;
        }
    }
    return null;
}

function writeCompositorRun(
    slot: GuiVideoMemoryWriteSlot,
    packet: GuiWebCompositorTilePresentPacketState,
    descriptor: GuiWebCompositorTilePresentDescriptor,
    run: GuiWebCompositorTilePresentRun,
): number {
    const startByte = packet.rowStart * descriptor.strideBytes + run.pixelOffset * BYTES_PER_RGBA8888_PIXEL;
    const byteLength = run.pixelCount * BYTES_PER_RGBA8888_PIXEL;
    const packetEndByte = (packet.rowStart + packet.rowCount) * descriptor.strideBytes;
    if (
        !Number.isSafeInteger(startByte)
        || !Number.isSafeInteger(byteLength)
        || startByte < 0
        || byteLength <= 0
        || startByte + byteLength > packetEndByte
        || startByte + byteLength > slot.pixels.byteLength
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    for (let offset = startByte; offset < startByte + byteLength; offset += BYTES_PER_RGBA8888_PIXEL) {
        slot.pixels[offset] = run.r;
        slot.pixels[offset + 1] = run.g;
        slot.pixels[offset + 2] = run.b;
        slot.pixels[offset + 3] = run.a;
    }
    return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
}

function expectedTileCount(rowCount: number, tileRows: number): number {
    return Math.floor((rowCount + tileRows - 1) / tileRows);
}

function isNonNegativeInteger(value: number): boolean {
    return Number.isInteger(value) && value >= 0;
}

function isPositiveInteger(value: number): boolean {
    return Number.isInteger(value) && value > 0;
}

function areValidColorChannels(r: number, g: number, b: number, a: number): boolean {
    return isByte(r) && isByte(g) && isByte(b) && isByte(a);
}

function isByte(value: number): boolean {
    return Number.isInteger(value) && value >= 0 && value <= 255;
}

function guiWebVideoMemoryHostStatusFromError(error: GuiVideoMemoryError): number {
    if (
        error.kind === 'shared-buffer-unavailable'
        || error.kind === 'wait-unavailable'
        || error.kind === 'unsupported-pixel-format'
        || error.kind === 'unsupported-stride'
        || error.kind === 'unsupported-command'
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_UNSUPPORTED;
    }
    if (error.kind === 'no-writable-slot') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_NO_WRITABLE_SLOT;
    }
    if (error.kind === 'no-published-slot' || error.kind === 'resource-exhausted') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_RESOURCE_EXHAUSTED;
    }
    if (error.kind === 'stale-resize-generation' || error.kind === 'writer-closed') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_STALE_FRAME;
    }
    if (error.kind === 'presenter-unavailable' || error.kind === 'present-failed') {
        return GUI_VIDEO_MEMORY_HOST_STATUS_BACKEND_FAILURE;
    }
    return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
}
