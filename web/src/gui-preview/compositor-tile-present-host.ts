import {
    type GuiVideoMemoryError,
    type GuiVideoMemoryDirtyRegion,
    acquireGuiVideoMemoryWriteSlot,
    discardGuiVideoMemoryWriteSlot,
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

type GuiWebCompositorTilePresentCompletion =
    | { kind: 'status'; status: number }
    | { kind: 'state'; state: GuiWebCompositorTilePresentFrameState };

export type GuiWebVideoMemoryHostPublishedFrameSnapshot = {
    pixels: Uint8ClampedArray;
};

export type GuiWebVideoMemoryHostSnapshotPrepareResult =
    | { kind: 'status'; status: number }
    | { kind: 'snapshot'; snapshot: GuiWebVideoMemoryHostPublishedFrameSnapshot };

type GuiWebCompositorTilePresentFrameState = {
    targetKind: typeof COMPOSITOR_TARGET_WINDOW;
    windowId: number;
    surfaceHandle: number;
    frameId: number;
    width: number;
    height: number;
    strideBytes: number;
    metadataRowStart: number;
    metadataRowCount: number;
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
    lastPublishedPixels: Uint8ClampedArray | null;
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
        lastPublishedPixels: null,
    };
}

export function guiWebVideoMemoryHostFrameIsPlain(frame: GuiWebVideoMemoryHostFrameRecord): boolean {
    return frame.compositor.kind === 'none';
}

export function guiWebVideoMemoryHostSurfaceHasActiveFrame(surface: GuiWebVideoMemoryHostSurfaceRecord): boolean {
    return surface.frames.length > 0;
}

export function prepareGuiWebVideoMemoryHostPublishedFrameSnapshot(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    slot: GuiVideoMemoryWriteSlot,
    dirty: GuiVideoMemoryDirtyRegion,
): GuiWebVideoMemoryHostSnapshotPrepareResult {
    if (slot.surface !== surface.surface) {
        return { kind: 'status', status: GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT };
    }
    if (dirty.kind === 'full') {
        return copyGuiWebVideoMemoryHostSlotSnapshot(surface, slot);
    }
    return copyGuiWebVideoMemoryHostDirtyRowsToNewSnapshot(surface, slot, dirty);
}

export function commitGuiWebVideoMemoryHostPublishedFrameSnapshot(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    snapshot: GuiWebVideoMemoryHostPublishedFrameSnapshot,
): void {
    surface.lastPublishedPixels = snapshot.pixels;
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
        if (guiWebVideoMemoryHostSurfaceHasActiveFrame(surface)) {
            return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
        }
        const acquired = acquireGuiVideoMemoryWriteSlot(surface.surface);
        if (acquired.kind === 'err') {
            return guiWebVideoMemoryHostStatusFromError(acquired.error);
        }
        if (!compositorDescriptorIsFullFrame(descriptor)) {
            const copied = copyGuiWebVideoMemoryHostSnapshotToSlot(surface, acquired.value);
            if (copied !== GUI_VIDEO_MEMORY_HOST_STATUS_OK) {
                discardGuiVideoMemoryWriteSlot(acquired.value);
                return copied;
            }
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
    const completed = completeCompositorPacket(frame.compositor.state, descriptor);
    if (completed.kind === 'status') {
        return { kind: 'status', status: completed.status };
    }
    const nextState = completed.state;
    if (!allCompositorPacketsCompleted(nextState)) {
        frame.compositor.state = nextState;
        return { kind: 'status', status: GUI_VIDEO_MEMORY_HOST_STATUS_OK };
    }
    const dirty = compositorDirtyRegionForState(nextState);
    const snapshot = prepareGuiWebVideoMemoryHostPublishedFrameSnapshot(surface, frame.slot, dirty);
    if (snapshot.kind === 'status') {
        return { kind: 'status', status: snapshot.status };
    }
    const published = publishGuiVideoMemoryWriteSlot(frame.slot, dirty);
    if (published.kind === 'err') {
        return { kind: 'status', status: guiWebVideoMemoryHostStatusFromError(published.error) };
    }
    commitGuiWebVideoMemoryHostPublishedFrameSnapshot(surface, snapshot.snapshot);
    surface.frames = surface.frames.filter((candidate) => candidate.frameId !== descriptor.frameId);
    return {
        kind: 'present',
        windowId: nextState.windowId,
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
        metadataRowStart: descriptor.metadataRowStart,
        metadataRowCount: descriptor.metadataRowCount,
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
        || !isNonNegativeInteger(descriptor.metadataRowStart)
        || !isPositiveInteger(descriptor.metadataRowCount)
        || !isPositiveInteger(descriptor.metadataBatchCount)
        || !isPositiveInteger(descriptor.metadataMaxRowsPerBatch)
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    const metadataRowEnd = descriptor.metadataRowStart + descriptor.metadataRowCount;
    if (
        descriptor.batchIndex >= descriptor.metadataBatchCount
        || descriptor.tileIndex >= descriptor.tileCount
        || !Number.isSafeInteger(metadataRowEnd)
        || metadataRowEnd > descriptor.height
        || descriptor.metadataBatchCount !== expectedTileCount(descriptor.metadataRowCount, descriptor.metadataMaxRowsPerBatch)
        || descriptor.planRowStart + descriptor.planRowCount > descriptor.height
        || descriptor.rowStart + descriptor.rowCount > descriptor.height
        || descriptor.planRowStart < descriptor.metadataRowStart
        || descriptor.planRowStart + descriptor.planRowCount > metadataRowEnd
        || descriptor.rowStart < descriptor.planRowStart
        || descriptor.rowStart + descriptor.rowCount > descriptor.planRowStart + descriptor.planRowCount
        || descriptor.pixelCount !== descriptor.rowCount * descriptor.width
        || descriptor.tileCount !== expectedTileCount(descriptor.planRowCount, descriptor.tileRows)
    ) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    const expectedPlanRowStart = descriptor.metadataRowStart + descriptor.batchIndex * descriptor.metadataMaxRowsPerBatch;
    const remainingRows = metadataRowEnd - expectedPlanRowStart;
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
        && state.metadataRowStart === descriptor.metadataRowStart
        && state.metadataRowCount === descriptor.metadataRowCount
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
): GuiWebCompositorTilePresentCompletion {
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
        return { kind: 'status', status: GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT };
    }
    const completedBatch = {
        batchIndex: batch.batchIndex,
        tileCount: batch.tileCount,
        completedTileIndices: [...batch.completedTileIndices, descriptor.tileIndex],
    };
    const batches = existingBatch
        ? state.batches.map((candidate) => candidate.batchIndex === descriptor.batchIndex ? completedBatch : candidate)
        : [...state.batches, completedBatch];
    return {
        kind: 'state',
        state: {
            ...state,
            currentPacket: { kind: 'none' },
            batches,
        },
    };
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

function compositorDescriptorIsFullFrame(descriptor: GuiWebCompositorTilePresentDescriptor): boolean {
    return descriptor.metadataRowStart === 0 && descriptor.metadataRowCount === descriptor.height;
}

function compositorDirtyRegionForState(
    state: GuiWebCompositorTilePresentFrameState,
): GuiVideoMemoryDirtyRegion {
    if (state.metadataRowStart === 0 && state.metadataRowCount === state.height) {
        return { kind: 'full' };
    }
    return {
        kind: 'rect',
        x: 0,
        y: state.metadataRowStart,
        width: state.width,
        height: state.metadataRowCount,
    };
}

function copyGuiWebVideoMemoryHostSnapshotToSlot(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    slot: GuiVideoMemoryWriteSlot,
): number {
    const snapshot = surface.lastPublishedPixels;
    if (!snapshot || snapshot.byteLength !== surface.surface.pixelByteLength) {
        return GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT;
    }
    slot.pixels.set(snapshot);
    return GUI_VIDEO_MEMORY_HOST_STATUS_OK;
}

function copyGuiWebVideoMemoryHostSlotSnapshot(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    slot: GuiVideoMemoryWriteSlot,
): GuiWebVideoMemoryHostSnapshotPrepareResult {
    if (slot.pixels.byteLength !== surface.surface.pixelByteLength) {
        return { kind: 'status', status: GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT };
    }
    try {
        const pixels = new Uint8ClampedArray(surface.surface.pixelByteLength);
        pixels.set(slot.pixels);
        return { kind: 'snapshot', snapshot: { pixels } };
    } catch {
        return { kind: 'status', status: GUI_VIDEO_MEMORY_HOST_STATUS_RESOURCE_EXHAUSTED };
    }
}

function copyGuiWebVideoMemoryHostDirtyRowsToNewSnapshot(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    slot: GuiVideoMemoryWriteSlot,
    dirty: Extract<GuiVideoMemoryDirtyRegion, { kind: 'rect' }>,
): GuiWebVideoMemoryHostSnapshotPrepareResult {
    const snapshot = surface.lastPublishedPixels;
    if (
        !snapshot
        || snapshot.byteLength !== surface.surface.pixelByteLength
        || slot.pixels.byteLength !== surface.surface.pixelByteLength
        || !isGuiWebVideoMemoryHostDirtyRegionInBounds(surface, dirty)
    ) {
        return { kind: 'status', status: GUI_VIDEO_MEMORY_HOST_STATUS_INVALID_ARGUMENT };
    }
    try {
        const pixels = new Uint8ClampedArray(surface.surface.pixelByteLength);
        pixels.set(snapshot);
        for (let row = dirty.y; row < dirty.y + dirty.height; row += 1) {
            const start = row * surface.surface.strideBytes + dirty.x * BYTES_PER_RGBA8888_PIXEL;
            const end = start + dirty.width * BYTES_PER_RGBA8888_PIXEL;
            pixels.set(slot.pixels.subarray(start, end), start);
        }
        return { kind: 'snapshot', snapshot: { pixels } };
    } catch {
        return { kind: 'status', status: GUI_VIDEO_MEMORY_HOST_STATUS_RESOURCE_EXHAUSTED };
    }
}

function isGuiWebVideoMemoryHostDirtyRegionInBounds(
    surface: GuiWebVideoMemoryHostSurfaceRecord,
    dirty: Extract<GuiVideoMemoryDirtyRegion, { kind: 'rect' }>,
): boolean {
    return isNonNegativeInteger(dirty.x)
        && isNonNegativeInteger(dirty.y)
        && isPositiveInteger(dirty.width)
        && isPositiveInteger(dirty.height)
        && Number.isSafeInteger(dirty.x + dirty.width)
        && Number.isSafeInteger(dirty.y + dirty.height)
        && dirty.x + dirty.width <= surface.surface.width
        && dirty.y + dirty.height <= surface.surface.height;
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
