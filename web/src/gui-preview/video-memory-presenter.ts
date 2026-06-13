import {
    acquireGuiVideoMemoryReadSlot,
    discardGuiVideoMemoryReadSlot,
    GUI_VIDEO_MEMORY_BYTES_PER_PIXEL,
    releaseGuiVideoMemoryReadSlot,
    type GuiVideoMemoryDirtyRegion,
    type GuiVideoMemoryError,
    type GuiVideoMemoryReadSlot,
    type GuiVideoMemoryResult,
    type GuiVideoMemorySlotCleanupStatus,
    type GuiVideoMemorySurface,
} from './video-memory-surface.js';

export type GuiVideoMemoryPresentedFrame =
    | {
        kind: 'presented';
        epoch: number;
        width: number;
        height: number;
        dirty: GuiVideoMemoryDirtyRegion;
    }
    | {
        kind: 'zero-dirty-region';
        epoch: number;
        width: number;
        height: number;
        dirty: GuiVideoMemoryDirtyRegion;
    };

const guiVideoMemoryImageDataCache = new WeakMap<SharedArrayBuffer, Map<number, ImageData>>();

export function presentNewestGuiVideoMemoryFrameToCanvas(
    ctx: CanvasRenderingContext2D,
    surface: GuiVideoMemorySurface,
): GuiVideoMemoryResult<GuiVideoMemoryPresentedFrame> {
    const acquired = acquireGuiVideoMemoryReadSlot(surface);
    if (acquired.kind === 'err') {
        return acquired;
    }
    const slot = acquired.value;
    const strideCheck = validateGuiVideoMemoryCanvasStride(slot);
    if (strideCheck.kind === 'err') {
        const cleanup = discardGuiVideoMemorySlotAfterPresenterStop(slot);
        if (strideCheck.error.kind !== 'unsupported-stride') {
            return strideCheck;
        }
        return guiVideoMemoryErr({
            ...strideCheck.error,
            cleanup,
        });
    }
    const dirtyCheck = validateGuiVideoMemoryDirtyRegion(slot);
    if (dirtyCheck.kind === 'err') {
        const cleanup = discardGuiVideoMemorySlotAfterPresenterStop(slot);
        if (dirtyCheck.error.kind !== 'invalid-dirty-region') {
            return dirtyCheck;
        }
        return guiVideoMemoryErr({
            ...dirtyCheck.error,
            cleanup,
        });
    }
    const dirty = dirtyCheck.value;
    if (dirty.kind === 'rect' && (dirty.width === 0 || dirty.height === 0)) {
        const released = releaseGuiVideoMemoryReadSlot(slot);
        if (released.kind === 'err') {
            return released;
        }
        return guiVideoMemoryOk({
            kind: 'zero-dirty-region',
            epoch: slot.epoch,
            width: slot.surface.width,
            height: slot.surface.height,
            dirty,
        });
    }
    try {
        const imageData = guiVideoMemoryImageDataForSlot(slot);
        if (dirty.kind === 'full') {
            ctx.putImageData(imageData, 0, 0);
        } else {
            ctx.putImageData(imageData, 0, 0, dirty.x, dirty.y, dirty.width, dirty.height);
        }
    } catch {
        const cleanup = discardGuiVideoMemorySlotAfterPresenterStop(slot);
        return guiVideoMemoryErr({ kind: 'present-failed', cleanup });
    }
    const released = releaseGuiVideoMemoryReadSlot(slot);
    if (released.kind === 'err') {
        return released;
    }
    return guiVideoMemoryOk({
        kind: 'presented',
        epoch: slot.epoch,
        width: slot.surface.width,
        height: slot.surface.height,
        dirty,
    });
}

function validateGuiVideoMemoryCanvasStride(
    slot: GuiVideoMemoryReadSlot,
): GuiVideoMemoryResult<void> {
    const expectedStrideBytes = slot.surface.width * GUI_VIDEO_MEMORY_BYTES_PER_PIXEL;
    if (
        slot.surface.strideBytes !== expectedStrideBytes
        || slot.pixels.length !== expectedStrideBytes * slot.surface.height
    ) {
        return guiVideoMemoryErr({
            kind: 'unsupported-stride',
            strideBytes: slot.surface.strideBytes,
            expectedStrideBytes,
            cleanup: { kind: 'discarded' },
        });
    }
    return guiVideoMemoryOk(undefined);
}

function validateGuiVideoMemoryDirtyRegion(
    slot: GuiVideoMemoryReadSlot,
): GuiVideoMemoryResult<GuiVideoMemoryDirtyRegion> {
    const dirty = slot.dirty;
    if (dirty.kind === 'full') {
        return guiVideoMemoryOk(dirty);
    }
    const valid = Number.isInteger(dirty.x)
        && Number.isInteger(dirty.y)
        && Number.isInteger(dirty.width)
        && Number.isInteger(dirty.height)
        && dirty.x >= 0
        && dirty.y >= 0
        && dirty.width >= 0
        && dirty.height >= 0
        && dirty.x + dirty.width <= slot.surface.width
        && dirty.y + dirty.height <= slot.surface.height;
    if (valid) {
        return guiVideoMemoryOk(dirty);
    }
    return guiVideoMemoryErr({
        kind: 'invalid-dirty-region',
        x: dirty.x,
        y: dirty.y,
        width: dirty.width,
        height: dirty.height,
        surfaceWidth: slot.surface.width,
        surfaceHeight: slot.surface.height,
        cleanup: { kind: 'discarded' },
    });
}

function guiVideoMemoryImageDataForSlot(slot: GuiVideoMemoryReadSlot): ImageData {
    const cache = guiVideoMemoryImageDataSlotCache(slot.surface);
    const cached = cache.get(slot.slotIndex);
    if (cached) {
        return cached;
    }
    const imageData = new ImageData(
        slot.pixels as unknown as ImageDataArray,
        slot.surface.width,
        slot.surface.height,
    );
    cache.set(slot.slotIndex, imageData);
    return imageData;
}

function guiVideoMemoryImageDataSlotCache(surface: GuiVideoMemorySurface): Map<number, ImageData> {
    const cached = guiVideoMemoryImageDataCache.get(surface.buffer);
    if (cached) {
        return cached;
    }
    const cache = new Map<number, ImageData>();
    guiVideoMemoryImageDataCache.set(surface.buffer, cache);
    return cache;
}

function discardGuiVideoMemorySlotAfterPresenterStop(
    slot: GuiVideoMemoryReadSlot,
): GuiVideoMemorySlotCleanupStatus {
    const discarded = discardGuiVideoMemoryReadSlot(slot);
    if (discarded.kind === 'ok') {
        return { kind: 'discarded' };
    }
    return { kind: 'cleanup-failed', error: discarded.error };
}

function guiVideoMemoryOk<T>(value: T): GuiVideoMemoryResult<T> {
    return { kind: 'ok', value };
}

function guiVideoMemoryErr<T>(error: GuiVideoMemoryError): GuiVideoMemoryResult<T> {
    return { kind: 'err', error };
}
