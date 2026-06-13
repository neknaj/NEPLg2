import {
    createGuiPreviewBitmapBuffer,
    fillGuiPreviewBitmapRect,
} from './bitmap-buffer.js';
import { presentGuiPreviewBitmapToCanvas } from './bitmap-presenter.js';
import {
    rasterizeGuiPreviewCommand,
    type GuiPreviewBitmapViewport,
    type GuiPreviewRasterizeError,
} from './bitmap-rasterizer.js';
import { guiPreviewRgb } from './commands.js';
import type { GuiPreviewCommandFrame } from './commands.js';

export type GuiPreviewCanvasViewport = {
    left: number;
    top: number;
    scale: number;
};

export type GuiPreviewCanvasRenderOptions = {
    fontSize: number;
};

export type GuiPreviewCanvasRenderResult = {
    kind: 'ok';
    viewport: GuiPreviewCanvasViewport;
} | {
    kind: 'err';
    viewport: GuiPreviewCanvasViewport;
    error: GuiPreviewRasterizeError;
};

export function renderGuiPreviewFrameToCanvas(
    ctx: CanvasRenderingContext2D,
    frame: GuiPreviewCommandFrame,
    width: number,
    height: number,
    options: GuiPreviewCanvasRenderOptions,
): GuiPreviewCanvasRenderResult {
    const viewport = calculateGuiPreviewCanvasViewport(frame, width, height);
    const pixelRatio = calculateGuiPreviewCanvasPixelRatio(ctx, width, height);
    const pixelViewport = scaleGuiPreviewCanvasViewport(viewport, pixelRatio);
    const buffer = createGuiPreviewBitmapBuffer(
        Math.max(1, ctx.canvas.width),
        Math.max(1, ctx.canvas.height),
        guiPreviewRgb(13, 17, 23),
    );
    fillGuiPreviewBitmapRect(
        buffer,
        pixelViewport.left - pixelRatio,
        pixelViewport.top - pixelRatio,
        frame.width * pixelViewport.scale + pixelRatio * 2,
        frame.height * pixelViewport.scale + pixelRatio * 2,
        guiPreviewRgb(16, 24, 32),
    );
    for (const command of frame.commands) {
        const rasterized = rasterizeGuiPreviewCommand(buffer, command, pixelViewport);
        if (rasterized.kind === 'err') {
            return { kind: 'err', viewport, error: rasterized.error };
        }
    }
    presentGuiPreviewBitmapToCanvas(ctx, buffer);
    return { kind: 'ok', viewport };
}

function calculateGuiPreviewCanvasViewport(
    frame: GuiPreviewCommandFrame,
    width: number,
    height: number,
): GuiPreviewCanvasViewport {
    const padding = 18;
    const cssWidth = Math.max(1, width);
    const cssHeight = Math.max(1, height);
    const availableWidth = Math.max(1, cssWidth - padding * 2);
    const availableHeight = Math.max(1, cssHeight - padding * 2);
    const scale = Math.min(availableWidth / frame.width, availableHeight / frame.height);
    const sceneWidth = frame.width * scale;
    const sceneHeight = frame.height * scale;
    return {
        left: Math.floor((cssWidth - sceneWidth) / 2),
        top: Math.floor((cssHeight - sceneHeight) / 2),
        scale,
    };
}

function calculateGuiPreviewCanvasPixelRatio(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
): number {
    const scaleX = ctx.canvas.width / Math.max(1, width);
    const scaleY = ctx.canvas.height / Math.max(1, height);
    const scale = Math.min(scaleX, scaleY);
    if (!Number.isFinite(scale) || scale <= 0) {
        return 1;
    }
    return scale;
}

function scaleGuiPreviewCanvasViewport(
    viewport: GuiPreviewCanvasViewport,
    pixelRatio: number,
): GuiPreviewBitmapViewport {
    return {
        left: viewport.left * pixelRatio,
        top: viewport.top * pixelRatio,
        scale: viewport.scale * pixelRatio,
    };
}
