import {
    createGuiPreviewBitmapBuffer,
    type GuiPreviewBitmapBuffer,
} from './bitmap-buffer.js';
import { guiPreviewRgb } from './commands.js';

export function presentGuiPreviewBitmapToCanvas(
    ctx: CanvasRenderingContext2D,
    buffer: GuiPreviewBitmapBuffer,
) {
    const imageData = new ImageData(buffer.pixels, buffer.width, buffer.height);
    ctx.putImageData(imageData, 0, 0);
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
