import {
    createGuiPreviewBitmapBuffer,
    type GuiPreviewBitmapBuffer,
} from './bitmap-buffer.js';
import { guiPreviewRgb } from './commands.js';

const guiPreviewImageDataCache = new WeakMap<GuiPreviewBitmapBuffer, ImageData>();

export function presentGuiPreviewBitmapToCanvas(
    ctx: CanvasRenderingContext2D,
    buffer: GuiPreviewBitmapBuffer,
) {
    const imageData = guiPreviewImageDataForBuffer(buffer);
    ctx.putImageData(imageData, 0, 0);
}

function guiPreviewImageDataForBuffer(buffer: GuiPreviewBitmapBuffer): ImageData {
    const cached = guiPreviewImageDataCache.get(buffer);
    if (cached) {
        return cached;
    }
    const imageData = new ImageData(buffer.pixels, buffer.width, buffer.height);
    guiPreviewImageDataCache.set(buffer, imageData);
    return imageData;
}

export function presentGuiPreviewCanvasBackground(
    ctx: CanvasRenderingContext2D,
) {
    const buffer = createGuiPreviewBitmapBuffer(
        Math.max(1, ctx.canvas.width),
        Math.max(1, ctx.canvas.height),
        guiPreviewRgb(13, 17, 23),
    );
    presentGuiPreviewBitmapToCanvas(ctx, buffer);
}
