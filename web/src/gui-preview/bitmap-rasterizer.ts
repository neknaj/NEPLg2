import {
    fillGuiPreviewBitmapRect,
    type GuiPreviewBitmapBuffer,
} from './bitmap-buffer.js';
import type {
    GuiPreviewColor,
    GuiPreviewDrawCommand,
    GuiPreviewTextAlign,
} from './commands.js';

export type GuiPreviewBitmapViewport = {
    left: number;
    top: number;
    scale: number;
};

export type GuiPreviewRasterizeError =
    | { kind: 'unsupported-scalar'; scalar: number }
    | { kind: 'invalid-geometry'; commandKind: GuiPreviewDrawCommand['kind'] };

export type GuiPreviewRasterizeResult =
    | { kind: 'ok' }
    | { kind: 'err'; error: GuiPreviewRasterizeError };

type GuiPreviewBitmapGlyphsResult =
    | { kind: 'ok'; value: readonly (readonly string[])[] }
    | { kind: 'err'; error: GuiPreviewRasterizeError };

export function rasterizeGuiPreviewCommand(
    buffer: GuiPreviewBitmapBuffer,
    command: GuiPreviewDrawCommand,
    viewport: GuiPreviewBitmapViewport,
): GuiPreviewRasterizeResult {
    if (command.kind === 'fill-rect') {
        return rasterizeGuiPreviewFillRect(buffer, command, viewport);
    }
    if (command.kind === 'rgba-row') {
        return rasterizeGuiPreviewRgbaRow(buffer, command, viewport);
    }
    return rasterizeGuiPreviewTextRun(buffer, command, viewport);
}

export function rasterizeGuiPreviewRgbaRow(
    buffer: GuiPreviewBitmapBuffer,
    command: Extract<GuiPreviewDrawCommand, { kind: 'rgba-row' }>,
    viewport: GuiPreviewBitmapViewport,
): GuiPreviewRasterizeResult {
    if (!guiPreviewPositiveFinite(command.cellWidth) || !guiPreviewPositiveFinite(command.cellHeight)) {
        return guiPreviewRasterizeErr({ kind: 'invalid-geometry', commandKind: command.kind });
    }
    if (command.pixels.length === 0) {
        return guiPreviewRasterizeOk();
    }
    let runStart = 0;
    while (runStart < command.pixels.length) {
        const color = command.pixels[runStart];
        let runEnd = runStart + 1;
        while (runEnd < command.pixels.length && guiPreviewColorEquals(color, command.pixels[runEnd])) {
            runEnd += 1;
        }
        fillGuiPreviewBitmapRect(
            buffer,
            viewport.left + (command.origin.x + runStart * command.cellWidth) * viewport.scale,
            viewport.top + command.origin.y * viewport.scale,
            (runEnd - runStart) * command.cellWidth * viewport.scale,
            command.cellHeight * viewport.scale,
            color,
        );
        runStart = runEnd;
    }
    return guiPreviewRasterizeOk();
}

function rasterizeGuiPreviewFillRect(
    buffer: GuiPreviewBitmapBuffer,
    command: Extract<GuiPreviewDrawCommand, { kind: 'fill-rect' }>,
    viewport: GuiPreviewBitmapViewport,
): GuiPreviewRasterizeResult {
    if (!guiPreviewNonNegativeFinite(command.rect.width) || !guiPreviewNonNegativeFinite(command.rect.height)) {
        return guiPreviewRasterizeErr({ kind: 'invalid-geometry', commandKind: command.kind });
    }
    if (command.rect.width === 0 || command.rect.height === 0) {
        return guiPreviewRasterizeOk();
    }
    fillGuiPreviewBitmapRect(
        buffer,
        viewport.left + command.rect.x * viewport.scale,
        viewport.top + command.rect.y * viewport.scale,
        command.rect.width * viewport.scale,
        command.rect.height * viewport.scale,
        command.color,
    );
    return guiPreviewRasterizeOk();
}

function rasterizeGuiPreviewTextRun(
    buffer: GuiPreviewBitmapBuffer,
    command: Extract<GuiPreviewDrawCommand, { kind: 'text-run' }>,
    viewport: GuiPreviewBitmapViewport,
): GuiPreviewRasterizeResult {
    const glyphScale = Math.max(1, Math.floor(command.size * viewport.scale / 7));
    const glyphs = guiPreviewBitmapGlyphs(command.text);
    if (glyphs.kind === 'err') {
        return glyphs;
    }
    const textWidth = measureGuiPreviewBitmapText(glyphs.value.length, glyphScale);
    const startX = alignGuiPreviewBitmapText(
        viewport.left + command.origin.x * viewport.scale,
        textWidth,
        command.align,
    );
    const startY = viewport.top + command.origin.y * viewport.scale;
    let cursorX = startX;
    for (const glyph of glyphs.value) {
        drawGuiPreviewBitmapGlyph(buffer, glyph, cursorX, startY, glyphScale, command.color);
        cursorX += (GUI_PREVIEW_BITMAP_GLYPH_WIDTH + 1) * glyphScale;
    }
    return guiPreviewRasterizeOk();
}

function measureGuiPreviewBitmapText(glyphCount: number, glyphScale: number): number {
    if (glyphCount === 0) {
        return 0;
    }
    return (glyphCount * GUI_PREVIEW_BITMAP_GLYPH_WIDTH + Math.max(0, glyphCount - 1)) * glyphScale;
}

function alignGuiPreviewBitmapText(x: number, width: number, align: GuiPreviewTextAlign): number {
    if (align === 'left') {
        return x;
    }
    if (align === 'center') {
        return x - width / 2;
    }
    if (align === 'right') {
        return x - width;
    }
    const neverAlign: never = align;
    return neverAlign;
}

function drawGuiPreviewBitmapGlyph(
    buffer: GuiPreviewBitmapBuffer,
    glyph: readonly string[],
    x: number,
    y: number,
    glyphScale: number,
    color: GuiPreviewColor,
) {
    for (let row = 0; row < glyph.length; row += 1) {
        const line = glyph[row];
        for (let column = 0; column < line.length; column += 1) {
            if (line[column] !== '1') {
                continue;
            }
            fillGuiPreviewBitmapRect(
                buffer,
                x + column * glyphScale,
                y + row * glyphScale,
                glyphScale,
                glyphScale,
                color,
            );
        }
    }
}

function guiPreviewBitmapGlyphs(text: string): GuiPreviewBitmapGlyphsResult {
    const glyphs: (readonly string[])[] = [];
    for (const character of text) {
        const normalized = character.toUpperCase();
        const glyph = GUI_PREVIEW_BITMAP_FONT[normalized];
        if (typeof glyph === 'undefined') {
            return { kind: 'err', error: { kind: 'unsupported-scalar', scalar: character.codePointAt(0) ?? 0 } };
        }
        glyphs.push(glyph);
    }
    return { kind: 'ok', value: glyphs };
}

function guiPreviewColorEquals(left: GuiPreviewColor, right: GuiPreviewColor): boolean {
    return left.red === right.red
        && left.green === right.green
        && left.blue === right.blue
        && left.alpha === right.alpha;
}

function guiPreviewNonNegativeFinite(value: number): boolean {
    return Number.isFinite(value) && value >= 0;
}

function guiPreviewPositiveFinite(value: number): boolean {
    return Number.isFinite(value) && value > 0;
}

function guiPreviewRasterizeOk(): GuiPreviewRasterizeResult {
    return { kind: 'ok' };
}

function guiPreviewRasterizeErr(error: GuiPreviewRasterizeError): GuiPreviewRasterizeResult {
    return { kind: 'err', error };
}

const GUI_PREVIEW_BITMAP_GLYPH_WIDTH = 5;

const GUI_PREVIEW_BITMAP_FONT: Record<string, readonly string[]> = {
    ' ': [
        '00000',
        '00000',
        '00000',
        '00000',
        '00000',
        '00000',
        '00000',
    ],
    '!': [
        '00100',
        '00100',
        '00100',
        '00100',
        '00100',
        '00000',
        '00100',
    ],
    '"': [
        '01010',
        '01010',
        '01010',
        '00000',
        '00000',
        '00000',
        '00000',
    ],
    '#': [
        '01010',
        '01010',
        '11111',
        '01010',
        '11111',
        '01010',
        '01010',
    ],
    '%': [
        '11001',
        '11010',
        '00100',
        '01000',
        '10110',
        '00110',
        '00000',
    ],
    '&': [
        '01100',
        '10010',
        '10100',
        '01000',
        '10101',
        '10010',
        '01101',
    ],
    '\'': [
        '00100',
        '00100',
        '01000',
        '00000',
        '00000',
        '00000',
        '00000',
    ],
    '(': [
        '00010',
        '00100',
        '01000',
        '01000',
        '01000',
        '00100',
        '00010',
    ],
    ')': [
        '01000',
        '00100',
        '00010',
        '00010',
        '00010',
        '00100',
        '01000',
    ],
    '*': [
        '00000',
        '00100',
        '10101',
        '01110',
        '10101',
        '00100',
        '00000',
    ],
    '+': [
        '00000',
        '00100',
        '00100',
        '11111',
        '00100',
        '00100',
        '00000',
    ],
    ',': [
        '00000',
        '00000',
        '00000',
        '00000',
        '00100',
        '00100',
        '01000',
    ],
    '-': [
        '00000',
        '00000',
        '00000',
        '11111',
        '00000',
        '00000',
        '00000',
    ],
    '.': [
        '00000',
        '00000',
        '00000',
        '00000',
        '00000',
        '00100',
        '00100',
    ],
    '/': [
        '00001',
        '00010',
        '00100',
        '01000',
        '10000',
        '00000',
        '00000',
    ],
    '0': [
        '01110',
        '10001',
        '10011',
        '10101',
        '11001',
        '10001',
        '01110',
    ],
    '1': [
        '00100',
        '01100',
        '00100',
        '00100',
        '00100',
        '00100',
        '01110',
    ],
    '2': [
        '01110',
        '10001',
        '00001',
        '00010',
        '00100',
        '01000',
        '11111',
    ],
    '3': [
        '11110',
        '00001',
        '00001',
        '01110',
        '00001',
        '00001',
        '11110',
    ],
    '4': [
        '00010',
        '00110',
        '01010',
        '10010',
        '11111',
        '00010',
        '00010',
    ],
    '5': [
        '11111',
        '10000',
        '10000',
        '11110',
        '00001',
        '00001',
        '11110',
    ],
    '6': [
        '01110',
        '10000',
        '10000',
        '11110',
        '10001',
        '10001',
        '01110',
    ],
    '7': [
        '11111',
        '00001',
        '00010',
        '00100',
        '01000',
        '01000',
        '01000',
    ],
    '8': [
        '01110',
        '10001',
        '10001',
        '01110',
        '10001',
        '10001',
        '01110',
    ],
    '9': [
        '01110',
        '10001',
        '10001',
        '01111',
        '00001',
        '00001',
        '01110',
    ],
    ':': [
        '00000',
        '00100',
        '00100',
        '00000',
        '00100',
        '00100',
        '00000',
    ],
    ';': [
        '00000',
        '00100',
        '00100',
        '00000',
        '00100',
        '00100',
        '01000',
    ],
    '<': [
        '00010',
        '00100',
        '01000',
        '10000',
        '01000',
        '00100',
        '00010',
    ],
    '=': [
        '00000',
        '00000',
        '11111',
        '00000',
        '11111',
        '00000',
        '00000',
    ],
    '>': [
        '01000',
        '00100',
        '00010',
        '00001',
        '00010',
        '00100',
        '01000',
    ],
    '?': [
        '01110',
        '10001',
        '00001',
        '00010',
        '00100',
        '00000',
        '00100',
    ],
    '@': [
        '01110',
        '10001',
        '10111',
        '10101',
        '10111',
        '10000',
        '01110',
    ],
    A: [
        '01110',
        '10001',
        '10001',
        '11111',
        '10001',
        '10001',
        '10001',
    ],
    B: [
        '11110',
        '10001',
        '10001',
        '11110',
        '10001',
        '10001',
        '11110',
    ],
    C: [
        '01110',
        '10001',
        '10000',
        '10000',
        '10000',
        '10001',
        '01110',
    ],
    D: [
        '11110',
        '10001',
        '10001',
        '10001',
        '10001',
        '10001',
        '11110',
    ],
    E: [
        '11111',
        '10000',
        '10000',
        '11110',
        '10000',
        '10000',
        '11111',
    ],
    F: [
        '11111',
        '10000',
        '10000',
        '11110',
        '10000',
        '10000',
        '10000',
    ],
    G: [
        '01110',
        '10001',
        '10000',
        '10111',
        '10001',
        '10001',
        '01110',
    ],
    H: [
        '10001',
        '10001',
        '10001',
        '11111',
        '10001',
        '10001',
        '10001',
    ],
    I: [
        '01110',
        '00100',
        '00100',
        '00100',
        '00100',
        '00100',
        '01110',
    ],
    J: [
        '00111',
        '00010',
        '00010',
        '00010',
        '10010',
        '10010',
        '01100',
    ],
    K: [
        '10001',
        '10010',
        '10100',
        '11000',
        '10100',
        '10010',
        '10001',
    ],
    L: [
        '10000',
        '10000',
        '10000',
        '10000',
        '10000',
        '10000',
        '11111',
    ],
    M: [
        '10001',
        '11011',
        '10101',
        '10101',
        '10001',
        '10001',
        '10001',
    ],
    N: [
        '10001',
        '11001',
        '10101',
        '10011',
        '10001',
        '10001',
        '10001',
    ],
    O: [
        '01110',
        '10001',
        '10001',
        '10001',
        '10001',
        '10001',
        '01110',
    ],
    P: [
        '11110',
        '10001',
        '10001',
        '11110',
        '10000',
        '10000',
        '10000',
    ],
    Q: [
        '01110',
        '10001',
        '10001',
        '10001',
        '10101',
        '10010',
        '01101',
    ],
    R: [
        '11110',
        '10001',
        '10001',
        '11110',
        '10100',
        '10010',
        '10001',
    ],
    S: [
        '01111',
        '10000',
        '10000',
        '01110',
        '00001',
        '00001',
        '11110',
    ],
    T: [
        '11111',
        '00100',
        '00100',
        '00100',
        '00100',
        '00100',
        '00100',
    ],
    U: [
        '10001',
        '10001',
        '10001',
        '10001',
        '10001',
        '10001',
        '01110',
    ],
    V: [
        '10001',
        '10001',
        '10001',
        '10001',
        '10001',
        '01010',
        '00100',
    ],
    W: [
        '10001',
        '10001',
        '10001',
        '10101',
        '10101',
        '10101',
        '01010',
    ],
    X: [
        '10001',
        '10001',
        '01010',
        '00100',
        '01010',
        '10001',
        '10001',
    ],
    Y: [
        '10001',
        '10001',
        '01010',
        '00100',
        '00100',
        '00100',
        '00100',
    ],
    Z: [
        '11111',
        '00001',
        '00010',
        '00100',
        '01000',
        '10000',
        '11111',
    ],
    '[': [
        '01110',
        '01000',
        '01000',
        '01000',
        '01000',
        '01000',
        '01110',
    ],
    '\\': [
        '10000',
        '01000',
        '00100',
        '00010',
        '00001',
        '00000',
        '00000',
    ],
    ']': [
        '01110',
        '00010',
        '00010',
        '00010',
        '00010',
        '00010',
        '01110',
    ],
    '^': [
        '00100',
        '01010',
        '10001',
        '00000',
        '00000',
        '00000',
        '00000',
    ],
    '_': [
        '00000',
        '00000',
        '00000',
        '00000',
        '00000',
        '00000',
        '11111',
    ],
    '|': [
        '00100',
        '00100',
        '00100',
        '00100',
        '00100',
        '00100',
        '00100',
    ],
};
