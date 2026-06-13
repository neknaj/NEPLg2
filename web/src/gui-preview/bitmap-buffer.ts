import type { GuiPreviewColor } from './commands.js';

export type GuiPreviewBitmapBuffer = {
    width: number;
    height: number;
    pixels: Uint8ClampedArray<ArrayBuffer>;
};

export function createGuiPreviewBitmapBuffer(
    width: number,
    height: number,
    color: GuiPreviewColor,
): GuiPreviewBitmapBuffer {
    const checkedWidth = guiPreviewPositiveSize(width);
    const checkedHeight = guiPreviewPositiveSize(height);
    const buffer = {
        width: checkedWidth,
        height: checkedHeight,
        pixels: new Uint8ClampedArray(checkedWidth * checkedHeight * 4),
    };
    fillGuiPreviewBitmapRect(buffer, 0, 0, checkedWidth, checkedHeight, color);
    return buffer;
}

export function fillGuiPreviewBitmapRect(
    buffer: GuiPreviewBitmapBuffer,
    x: number,
    y: number,
    width: number,
    height: number,
    color: GuiPreviewColor,
) {
    const left = Math.max(0, Math.floor(x));
    const top = Math.max(0, Math.floor(y));
    const right = Math.min(buffer.width, Math.ceil(x + width));
    const bottom = Math.min(buffer.height, Math.ceil(y + height));
    if (right <= left || bottom <= top) {
        return;
    }
    for (let row = top; row < bottom; row += 1) {
        let offset = (row * buffer.width + left) * 4;
        for (let column = left; column < right; column += 1) {
            writeGuiPreviewBitmapColor(buffer.pixels, offset, color);
            offset += 4;
        }
    }
}

export function writeGuiPreviewBitmapPixel(
    buffer: GuiPreviewBitmapBuffer,
    x: number,
    y: number,
    color: GuiPreviewColor,
) {
    const column = Math.floor(x);
    const row = Math.floor(y);
    if (column < 0 || row < 0 || column >= buffer.width || row >= buffer.height) {
        return;
    }
    const offset = (row * buffer.width + column) * 4;
    writeGuiPreviewBitmapColor(buffer.pixels, offset, color);
}

function writeGuiPreviewBitmapColor(pixels: Uint8ClampedArray<ArrayBuffer>, offset: number, color: GuiPreviewColor) {
    const alpha = color.alpha / 255;
    if (alpha >= 1) {
        pixels[offset] = color.red;
        pixels[offset + 1] = color.green;
        pixels[offset + 2] = color.blue;
        pixels[offset + 3] = 255;
        return;
    }
    if (alpha <= 0) {
        return;
    }
    const inverseAlpha = 1 - alpha;
    pixels[offset] = Math.round(color.red * alpha + pixels[offset] * inverseAlpha);
    pixels[offset + 1] = Math.round(color.green * alpha + pixels[offset + 1] * inverseAlpha);
    pixels[offset + 2] = Math.round(color.blue * alpha + pixels[offset + 2] * inverseAlpha);
    pixels[offset + 3] = Math.round(255 * alpha + pixels[offset + 3] * inverseAlpha);
}

function guiPreviewPositiveSize(value: number): number {
    const size = Math.floor(value);
    return Math.max(1, size);
}
