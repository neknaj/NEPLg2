export const GUI_VIDEO_MEMORY_MAGIC = 0x4e475632;
export const GUI_VIDEO_MEMORY_VERSION = 1;
export const GUI_VIDEO_MEMORY_MIN_SLOT_COUNT = 2;
export const GUI_VIDEO_MEMORY_BYTES_PER_PIXEL = 4;
export const GUI_VIDEO_MEMORY_FORMAT_RGBA8888 = 1;

const GUI_VIDEO_MEMORY_HEADER_WORDS = 16;
const GUI_VIDEO_MEMORY_SLOT_WORDS = 8;
const GUI_VIDEO_MEMORY_WORD_BYTES = 4;
const GUI_VIDEO_MEMORY_MAX_BUFFER_BYTES = 512 * 1024 * 1024;

const HEADER_MAGIC = 0;
const HEADER_VERSION = 1;
const HEADER_WIDTH = 2;
const HEADER_HEIGHT = 3;
const HEADER_STRIDE_BYTES = 4;
const HEADER_FORMAT = 5;
const HEADER_GENERATION = 6;
const HEADER_SLOT_COUNT = 7;
const HEADER_PUBLISHED_EPOCH = 8;
const HEADER_PRESENTED_EPOCH = 9;
const HEADER_SURFACE_STATE = 10;
const HEADER_ERROR_CODE = 11;
const HEADER_INT32_LENGTH = 12;
const HEADER_PIXEL_PLANE_BYTE_OFFSET = 13;
const HEADER_PIXEL_PLANE_BYTE_LENGTH = 14;

const SLOT_STATE = 0;
const SLOT_EPOCH = 1;
const SLOT_DIRTY_KIND = 2;
const SLOT_DIRTY_X = 3;
const SLOT_DIRTY_Y = 4;
const SLOT_DIRTY_WIDTH = 5;
const SLOT_DIRTY_HEIGHT = 6;

export const GUI_VIDEO_MEMORY_SURFACE_READY = 1;
export const GUI_VIDEO_MEMORY_SURFACE_CLOSING = 2;
export const GUI_VIDEO_MEMORY_SURFACE_CLOSED = 3;
export const GUI_VIDEO_MEMORY_SURFACE_UNAVAILABLE = 4;

export const GUI_VIDEO_MEMORY_SLOT_FREE = 1;
export const GUI_VIDEO_MEMORY_SLOT_WRITING = 2;
export const GUI_VIDEO_MEMORY_SLOT_PUBLISHED = 3;
export const GUI_VIDEO_MEMORY_SLOT_READING = 4;
export const GUI_VIDEO_MEMORY_SLOT_CLOSED = 5;

export type GuiVideoMemorySlotCleanupStatus =
    | { kind: 'discarded' }
    | { kind: 'cleanup-failed'; error: GuiVideoMemoryError };

export type GuiVideoMemoryError =
    | { kind: 'shared-buffer-unavailable' }
    | { kind: 'invalid-surface-config'; width: number; height: number; slotCount: number }
    | { kind: 'resource-exhausted'; byteLength: number }
    | { kind: 'invalid-buffer-length'; actual: number; minimum: number }
    | { kind: 'invalid-header-magic'; actual: number }
    | { kind: 'unsupported-header-version'; actual: number }
    | {
        kind: 'invalid-header-layout';
        slotCount: number;
        headerWords: number;
        pixelPlaneByteOffset: number;
        pixelPlaneByteLength: number;
    }
    | { kind: 'invalid-surface-state'; actual: number }
    | { kind: 'invalid-slot-state'; slotIndex: number; actual: number }
    | { kind: 'no-writable-slot' }
    | { kind: 'no-published-slot' }
    | { kind: 'stale-resize-generation'; expected: number; actual: number }
    | {
        kind: 'invalid-dirty-region';
        x: number;
        y: number;
        width: number;
        height: number;
        surfaceWidth: number;
        surfaceHeight: number;
        cleanup: GuiVideoMemorySlotCleanupStatus;
    }
    | { kind: 'presenter-unavailable' }
    | { kind: 'present-failed'; cleanup: GuiVideoMemorySlotCleanupStatus }
    | { kind: 'writer-closed' }
    | { kind: 'wait-unavailable' }
    | { kind: 'unsupported-pixel-format'; actual: number }
    | {
        kind: 'unsupported-stride';
        strideBytes: number;
        expectedStrideBytes: number;
        cleanup: GuiVideoMemorySlotCleanupStatus;
    }
    | { kind: 'unsupported-command'; commandKind: string };

export type GuiVideoMemoryResult<T> =
    | { kind: 'ok'; value: T }
    | { kind: 'err'; error: GuiVideoMemoryError };

export type GuiVideoMemoryDirtyRegion =
    | { kind: 'full' }
    | { kind: 'rect'; x: number; y: number; width: number; height: number };

export type GuiVideoMemorySurface = {
    buffer: SharedArrayBuffer;
    header: Int32Array;
    slots: Int32Array;
    width: number;
    height: number;
    strideBytes: number;
    slotCount: number;
    pixelPlaneByteOffset: number;
    pixelByteLength: number;
    generation: number;
};

export type GuiVideoMemoryWriteSlot = {
    surface: GuiVideoMemorySurface;
    slotIndex: number;
    generation: number;
    pixels: Uint8ClampedArray;
};

export type GuiVideoMemoryReadSlot = {
    surface: GuiVideoMemorySurface;
    slotIndex: number;
    generation: number;
    epoch: number;
    dirty: GuiVideoMemoryDirtyRegion;
    pixels: Uint8ClampedArray;
};

export type GuiVideoMemoryWaitResult =
    | { kind: 'slot'; slot: GuiVideoMemoryReadSlot }
    | { kind: 'timeout' };

export function createGuiVideoMemorySurface(
    width: number,
    height: number,
    slotCount = GUI_VIDEO_MEMORY_MIN_SLOT_COUNT,
): GuiVideoMemoryResult<GuiVideoMemorySurface> {
    if (typeof globalThis.SharedArrayBuffer !== 'function') {
        return guiVideoMemoryErr({ kind: 'shared-buffer-unavailable' });
    }
    if (
        !Number.isInteger(width)
        || !Number.isInteger(height)
        || !Number.isInteger(slotCount)
        || width <= 0
        || height <= 0
        || slotCount < GUI_VIDEO_MEMORY_MIN_SLOT_COUNT
    ) {
        return guiVideoMemoryErr({ kind: 'invalid-surface-config', width, height, slotCount });
    }
    const headerBytes = GUI_VIDEO_MEMORY_HEADER_WORDS * GUI_VIDEO_MEMORY_WORD_BYTES;
    const slotHeaderBytes = slotCount * GUI_VIDEO_MEMORY_SLOT_WORDS * GUI_VIDEO_MEMORY_WORD_BYTES;
    const strideBytes = width * GUI_VIDEO_MEMORY_BYTES_PER_PIXEL;
    const pixelByteLength = strideBytes * height;
    const pixelPlaneByteOffset = headerBytes + slotHeaderBytes;
    const totalByteLength = pixelPlaneByteOffset + pixelByteLength * slotCount;
    if (
        !Number.isSafeInteger(strideBytes)
        || !Number.isSafeInteger(pixelByteLength)
        || !Number.isSafeInteger(pixelPlaneByteOffset)
        || !Number.isSafeInteger(totalByteLength)
        || totalByteLength > GUI_VIDEO_MEMORY_MAX_BUFFER_BYTES
    ) {
        return guiVideoMemoryErr({ kind: 'resource-exhausted', byteLength: totalByteLength });
    }
    let buffer: SharedArrayBuffer;
    try {
        buffer = new SharedArrayBuffer(totalByteLength);
    } catch {
        return guiVideoMemoryErr({ kind: 'resource-exhausted', byteLength: totalByteLength });
    }
    const header = new Int32Array(buffer, 0, GUI_VIDEO_MEMORY_HEADER_WORDS);
    const slots = new Int32Array(buffer, headerBytes, slotCount * GUI_VIDEO_MEMORY_SLOT_WORDS);
    Atomics.store(header, HEADER_MAGIC, GUI_VIDEO_MEMORY_MAGIC);
    Atomics.store(header, HEADER_VERSION, GUI_VIDEO_MEMORY_VERSION);
    Atomics.store(header, HEADER_WIDTH, width);
    Atomics.store(header, HEADER_HEIGHT, height);
    Atomics.store(header, HEADER_STRIDE_BYTES, strideBytes);
    Atomics.store(header, HEADER_FORMAT, GUI_VIDEO_MEMORY_FORMAT_RGBA8888);
    Atomics.store(header, HEADER_GENERATION, 1);
    Atomics.store(header, HEADER_SLOT_COUNT, slotCount);
    Atomics.store(header, HEADER_PUBLISHED_EPOCH, 0);
    Atomics.store(header, HEADER_PRESENTED_EPOCH, 0);
    Atomics.store(header, HEADER_SURFACE_STATE, GUI_VIDEO_MEMORY_SURFACE_READY);
    Atomics.store(header, HEADER_ERROR_CODE, 0);
    Atomics.store(header, HEADER_INT32_LENGTH, GUI_VIDEO_MEMORY_HEADER_WORDS);
    Atomics.store(header, HEADER_PIXEL_PLANE_BYTE_OFFSET, pixelPlaneByteOffset);
    Atomics.store(header, HEADER_PIXEL_PLANE_BYTE_LENGTH, pixelByteLength);
    for (let slotIndex = 0; slotIndex < slotCount; slotIndex += 1) {
        Atomics.store(slots, guiVideoMemorySlotWord(slotIndex, SLOT_STATE), GUI_VIDEO_MEMORY_SLOT_FREE);
        Atomics.store(slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_KIND), 0);
    }
    return guiVideoMemoryOk({
        buffer,
        header,
        slots,
        width,
        height,
        strideBytes,
        slotCount,
        pixelPlaneByteOffset,
        pixelByteLength,
        generation: 1,
    });
}

export function openGuiVideoMemorySurface(buffer: SharedArrayBuffer): GuiVideoMemoryResult<GuiVideoMemorySurface> {
    const headerBytes = GUI_VIDEO_MEMORY_HEADER_WORDS * GUI_VIDEO_MEMORY_WORD_BYTES;
    if (buffer.byteLength < headerBytes) {
        return guiVideoMemoryErr({ kind: 'invalid-buffer-length', actual: buffer.byteLength, minimum: headerBytes });
    }
    const header = new Int32Array(buffer, 0, GUI_VIDEO_MEMORY_HEADER_WORDS);
    const magic = Atomics.load(header, HEADER_MAGIC);
    if (magic !== GUI_VIDEO_MEMORY_MAGIC) {
        return guiVideoMemoryErr({ kind: 'invalid-header-magic', actual: magic });
    }
    const version = Atomics.load(header, HEADER_VERSION);
    if (version !== GUI_VIDEO_MEMORY_VERSION) {
        return guiVideoMemoryErr({ kind: 'unsupported-header-version', actual: version });
    }
    const width = Atomics.load(header, HEADER_WIDTH);
    const height = Atomics.load(header, HEADER_HEIGHT);
    const strideBytes = Atomics.load(header, HEADER_STRIDE_BYTES);
    const format = Atomics.load(header, HEADER_FORMAT);
    const generation = Atomics.load(header, HEADER_GENERATION);
    const slotCount = Atomics.load(header, HEADER_SLOT_COUNT);
    const headerWords = Atomics.load(header, HEADER_INT32_LENGTH);
    const pixelPlaneByteOffset = Atomics.load(header, HEADER_PIXEL_PLANE_BYTE_OFFSET);
    const pixelByteLength = Atomics.load(header, HEADER_PIXEL_PLANE_BYTE_LENGTH);
    if (format !== GUI_VIDEO_MEMORY_FORMAT_RGBA8888) {
        return guiVideoMemoryErr({ kind: 'unsupported-pixel-format', actual: format });
    }
    const expectedPixelPlaneByteOffset = headerBytes + slotCount * GUI_VIDEO_MEMORY_SLOT_WORDS * GUI_VIDEO_MEMORY_WORD_BYTES;
    if (
        width <= 0
        || height <= 0
        || strideBytes < width * GUI_VIDEO_MEMORY_BYTES_PER_PIXEL
        || strideBytes % GUI_VIDEO_MEMORY_BYTES_PER_PIXEL !== 0
        || pixelByteLength < strideBytes * height
        || slotCount < GUI_VIDEO_MEMORY_MIN_SLOT_COUNT
        || headerWords !== GUI_VIDEO_MEMORY_HEADER_WORDS
        || pixelPlaneByteOffset !== expectedPixelPlaneByteOffset
    ) {
        return guiVideoMemoryErr({
            kind: 'invalid-header-layout',
            slotCount,
            headerWords,
            pixelPlaneByteOffset,
            pixelPlaneByteLength: pixelByteLength,
        });
    }
    const requiredByteLength = pixelPlaneByteOffset + pixelByteLength * slotCount;
    if (buffer.byteLength < requiredByteLength) {
        return guiVideoMemoryErr({
            kind: 'invalid-buffer-length',
            actual: buffer.byteLength,
            minimum: requiredByteLength,
        });
    }
    const state = Atomics.load(header, HEADER_SURFACE_STATE);
    if (state !== GUI_VIDEO_MEMORY_SURFACE_READY) {
        return guiVideoMemoryErr({ kind: 'invalid-surface-state', actual: state });
    }
    return guiVideoMemoryOk({
        buffer,
        header,
        slots: new Int32Array(
            buffer,
            headerBytes,
            slotCount * GUI_VIDEO_MEMORY_SLOT_WORDS,
        ),
        width,
        height,
        strideBytes,
        slotCount,
        pixelPlaneByteOffset,
        pixelByteLength,
        generation,
    });
}

export function acquireGuiVideoMemoryWriteSlot(
    surface: GuiVideoMemorySurface,
): GuiVideoMemoryResult<GuiVideoMemoryWriteSlot> {
    const state = Atomics.load(surface.header, HEADER_SURFACE_STATE);
    if (state === GUI_VIDEO_MEMORY_SURFACE_CLOSING || state === GUI_VIDEO_MEMORY_SURFACE_CLOSED) {
        return guiVideoMemoryErr({ kind: 'writer-closed' });
    }
    if (state !== GUI_VIDEO_MEMORY_SURFACE_READY) {
        return guiVideoMemoryErr({ kind: 'invalid-surface-state', actual: state });
    }
    const generation = Atomics.load(surface.header, HEADER_GENERATION);
    for (let slotIndex = 0; slotIndex < surface.slotCount; slotIndex += 1) {
        const stateIndex = guiVideoMemorySlotWord(slotIndex, SLOT_STATE);
        const previous = Atomics.compareExchange(
            surface.slots,
            stateIndex,
            GUI_VIDEO_MEMORY_SLOT_FREE,
            GUI_VIDEO_MEMORY_SLOT_WRITING,
        );
        if (previous !== GUI_VIDEO_MEMORY_SLOT_FREE) {
            continue;
        }
        Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_KIND), 0);
        return guiVideoMemoryOk({
            surface,
            slotIndex,
            generation,
            pixels: guiVideoMemorySlotPixels(surface, slotIndex),
        });
    }
    return guiVideoMemoryErr({ kind: 'no-writable-slot' });
}

export function publishGuiVideoMemoryWriteSlot(
    slot: GuiVideoMemoryWriteSlot,
    dirty: GuiVideoMemoryDirtyRegion,
): GuiVideoMemoryResult<void> {
    const currentGeneration = Atomics.load(slot.surface.header, HEADER_GENERATION);
    if (slot.generation !== currentGeneration) {
        return guiVideoMemoryErr({
            kind: 'stale-resize-generation',
            expected: currentGeneration,
            actual: slot.generation,
        });
    }
    const stateIndex = guiVideoMemorySlotWord(slot.slotIndex, SLOT_STATE);
    const currentState = Atomics.load(slot.surface.slots, stateIndex);
    if (currentState !== GUI_VIDEO_MEMORY_SLOT_WRITING) {
        return guiVideoMemoryErr({
            kind: 'invalid-slot-state',
            slotIndex: slot.slotIndex,
            actual: currentState,
        });
    }
    const epoch = Atomics.add(slot.surface.header, HEADER_PUBLISHED_EPOCH, 1) + 1;
    Atomics.store(slot.surface.slots, guiVideoMemorySlotWord(slot.slotIndex, SLOT_EPOCH), epoch);
    storeGuiVideoMemoryDirtyRegion(slot.surface, slot.slotIndex, dirty);
    Atomics.store(slot.surface.slots, stateIndex, GUI_VIDEO_MEMORY_SLOT_PUBLISHED);
    Atomics.notify(slot.surface.header, HEADER_PUBLISHED_EPOCH, 1);
    return guiVideoMemoryOk(undefined);
}

export function acquireGuiVideoMemoryReadSlot(
    surface: GuiVideoMemorySurface,
): GuiVideoMemoryResult<GuiVideoMemoryReadSlot> {
    const state = Atomics.load(surface.header, HEADER_SURFACE_STATE);
    if (state === GUI_VIDEO_MEMORY_SURFACE_CLOSING || state === GUI_VIDEO_MEMORY_SURFACE_CLOSED) {
        return guiVideoMemoryErr({ kind: 'presenter-unavailable' });
    }
    if (state !== GUI_VIDEO_MEMORY_SURFACE_READY) {
        return guiVideoMemoryErr({ kind: 'invalid-surface-state', actual: state });
    }
    const slotIndex = findNewestPublishedGuiVideoMemorySlot(surface);
    if (slotIndex < 0) {
        return guiVideoMemoryErr({ kind: 'no-published-slot' });
    }
    const stateIndex = guiVideoMemorySlotWord(slotIndex, SLOT_STATE);
    const previous = Atomics.compareExchange(
        surface.slots,
        stateIndex,
        GUI_VIDEO_MEMORY_SLOT_PUBLISHED,
        GUI_VIDEO_MEMORY_SLOT_READING,
    );
    if (previous !== GUI_VIDEO_MEMORY_SLOT_PUBLISHED) {
        return guiVideoMemoryErr({ kind: 'no-published-slot' });
    }
    return guiVideoMemoryOk({
        surface,
        slotIndex,
        generation: Atomics.load(surface.header, HEADER_GENERATION),
        epoch: Atomics.load(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_EPOCH)),
        dirty: loadGuiVideoMemoryDirtyRegion(surface, slotIndex),
        pixels: guiVideoMemorySlotPixels(surface, slotIndex),
    });
}

export function waitForGuiVideoMemoryReadSlot(
    surface: GuiVideoMemorySurface,
    timeoutMs: number,
): GuiVideoMemoryResult<GuiVideoMemoryWaitResult> {
    const deadline = Date.now() + Math.max(0, timeoutMs);
    while (true) {
        const immediate = acquireGuiVideoMemoryReadSlot(surface);
        if (immediate.kind === 'ok') {
            return guiVideoMemoryOk({ kind: 'slot', slot: immediate.value });
        }
        if (immediate.error.kind !== 'no-published-slot') {
            return immediate;
        }
        const epoch = Atomics.load(surface.header, HEADER_PUBLISHED_EPOCH);
        const beforeWait = acquireGuiVideoMemoryReadSlot(surface);
        if (beforeWait.kind === 'ok') {
            return guiVideoMemoryOk({ kind: 'slot', slot: beforeWait.value });
        }
        if (beforeWait.error.kind !== 'no-published-slot') {
            return beforeWait;
        }
        const remainingMs = deadline - Date.now();
        if (remainingMs <= 0) {
            return guiVideoMemoryOk({ kind: 'timeout' });
        }
        let waitResult: 'ok' | 'not-equal' | 'timed-out';
        try {
            waitResult = Atomics.wait(surface.header, HEADER_PUBLISHED_EPOCH, epoch, remainingMs);
        } catch {
            return guiVideoMemoryErr({ kind: 'wait-unavailable' });
        }
        if (waitResult === 'timed-out') {
            const afterTimeout = acquireGuiVideoMemoryReadSlot(surface);
            if (afterTimeout.kind === 'ok') {
                return guiVideoMemoryOk({ kind: 'slot', slot: afterTimeout.value });
            }
            if (afterTimeout.error.kind !== 'no-published-slot') {
                return afterTimeout;
            }
            return guiVideoMemoryOk({ kind: 'timeout' });
        }
    }
}

export function releaseGuiVideoMemoryReadSlot(
    slot: GuiVideoMemoryReadSlot,
): GuiVideoMemoryResult<void> {
    const currentGeneration = Atomics.load(slot.surface.header, HEADER_GENERATION);
    if (slot.generation !== currentGeneration) {
        return guiVideoMemoryErr({
            kind: 'stale-resize-generation',
            expected: currentGeneration,
            actual: slot.generation,
        });
    }
    const stateIndex = guiVideoMemorySlotWord(slot.slotIndex, SLOT_STATE);
    const previous = Atomics.compareExchange(
        slot.surface.slots,
        stateIndex,
        GUI_VIDEO_MEMORY_SLOT_READING,
        GUI_VIDEO_MEMORY_SLOT_FREE,
    );
    if (previous !== GUI_VIDEO_MEMORY_SLOT_READING) {
        return guiVideoMemoryErr({
            kind: 'invalid-slot-state',
            slotIndex: slot.slotIndex,
            actual: previous,
        });
    }
    Atomics.store(slot.surface.header, HEADER_PRESENTED_EPOCH, slot.epoch);
    Atomics.notify(slot.surface.slots, stateIndex, 1);
    return guiVideoMemoryOk(undefined);
}

export function discardGuiVideoMemoryReadSlot(
    slot: GuiVideoMemoryReadSlot,
): GuiVideoMemoryResult<void> {
    const currentGeneration = Atomics.load(slot.surface.header, HEADER_GENERATION);
    if (slot.generation !== currentGeneration) {
        return guiVideoMemoryErr({
            kind: 'stale-resize-generation',
            expected: currentGeneration,
            actual: slot.generation,
        });
    }
    const stateIndex = guiVideoMemorySlotWord(slot.slotIndex, SLOT_STATE);
    const previous = Atomics.compareExchange(
        slot.surface.slots,
        stateIndex,
        GUI_VIDEO_MEMORY_SLOT_READING,
        GUI_VIDEO_MEMORY_SLOT_FREE,
    );
    if (previous !== GUI_VIDEO_MEMORY_SLOT_READING) {
        return guiVideoMemoryErr({
            kind: 'invalid-slot-state',
            slotIndex: slot.slotIndex,
            actual: previous,
        });
    }
    Atomics.notify(slot.surface.slots, stateIndex, 1);
    return guiVideoMemoryOk(undefined);
}

export function closeGuiVideoMemorySurface(surface: GuiVideoMemorySurface): GuiVideoMemoryResult<void> {
    const previous = Atomics.compareExchange(
        surface.header,
        HEADER_SURFACE_STATE,
        GUI_VIDEO_MEMORY_SURFACE_READY,
        GUI_VIDEO_MEMORY_SURFACE_CLOSING,
    );
    if (previous !== GUI_VIDEO_MEMORY_SURFACE_READY) {
        return guiVideoMemoryErr({ kind: 'invalid-surface-state', actual: previous });
    }
    for (let slotIndex = 0; slotIndex < surface.slotCount; slotIndex += 1) {
        Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_STATE), GUI_VIDEO_MEMORY_SLOT_CLOSED);
    }
    Atomics.store(surface.header, HEADER_SURFACE_STATE, GUI_VIDEO_MEMORY_SURFACE_CLOSED);
    Atomics.notify(surface.header, HEADER_PUBLISHED_EPOCH, surface.slotCount);
    return guiVideoMemoryOk(undefined);
}

function findNewestPublishedGuiVideoMemorySlot(surface: GuiVideoMemorySurface): number {
    let selectedSlot = -1;
    let selectedEpoch = -1;
    for (let slotIndex = 0; slotIndex < surface.slotCount; slotIndex += 1) {
        const state = Atomics.load(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_STATE));
        if (state !== GUI_VIDEO_MEMORY_SLOT_PUBLISHED) {
            continue;
        }
        const epoch = Atomics.load(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_EPOCH));
        if (epoch > selectedEpoch) {
            selectedSlot = slotIndex;
            selectedEpoch = epoch;
        }
    }
    return selectedSlot;
}

function guiVideoMemorySlotPixels(surface: GuiVideoMemorySurface, slotIndex: number): Uint8ClampedArray {
    return new Uint8ClampedArray(
        surface.buffer,
        guiVideoMemoryPixelOffset(surface, slotIndex),
        surface.pixelByteLength,
    );
}

function guiVideoMemoryPixelOffset(surface: GuiVideoMemorySurface, slotIndex: number): number {
    return surface.pixelPlaneByteOffset + slotIndex * surface.pixelByteLength;
}

function storeGuiVideoMemoryDirtyRegion(
    surface: GuiVideoMemorySurface,
    slotIndex: number,
    dirty: GuiVideoMemoryDirtyRegion,
) {
    if (dirty.kind === 'full') {
        Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_KIND), 1);
        Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_X), 0);
        Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_Y), 0);
        Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_WIDTH), surface.width);
        Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_HEIGHT), surface.height);
        return;
    }
    Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_KIND), 2);
    Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_X), Math.trunc(dirty.x));
    Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_Y), Math.trunc(dirty.y));
    Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_WIDTH), Math.trunc(dirty.width));
    Atomics.store(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_HEIGHT), Math.trunc(dirty.height));
}

function loadGuiVideoMemoryDirtyRegion(
    surface: GuiVideoMemorySurface,
    slotIndex: number,
): GuiVideoMemoryDirtyRegion {
    const kind = Atomics.load(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_KIND));
    if (kind === 2) {
        return {
            kind: 'rect',
            x: Atomics.load(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_X)),
            y: Atomics.load(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_Y)),
            width: Atomics.load(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_WIDTH)),
            height: Atomics.load(surface.slots, guiVideoMemorySlotWord(slotIndex, SLOT_DIRTY_HEIGHT)),
        };
    }
    return { kind: 'full' };
}

function guiVideoMemorySlotWord(slotIndex: number, offset: number): number {
    return slotIndex * GUI_VIDEO_MEMORY_SLOT_WORDS + offset;
}

function guiVideoMemoryOk<T>(value: T): GuiVideoMemoryResult<T> {
    return { kind: 'ok', value };
}

function guiVideoMemoryErr<T>(error: GuiVideoMemoryError): GuiVideoMemoryResult<T> {
    return { kind: 'err', error };
}
