import {
    createGuiPreviewBitmapBuffer,
    fillGuiPreviewBitmapRect,
    type GuiPreviewBitmapBuffer,
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

const guiPreviewCanvasBitmapBuffers = new WeakMap<HTMLCanvasElement, GuiPreviewBitmapBuffer>();

export function renderGuiPreviewFrameToCanvas(
    ctx: CanvasRenderingContext2D,
    frame: GuiPreviewCommandFrame,
    width: number,
    height: number,
    options: GuiPreviewCanvasRenderOptions,
): GuiPreviewCanvasRenderResult {
    const viewport = calculateGuiPreviewCanvasViewport();
    const pixelRatio = calculateGuiPreviewCanvasPixelRatio(ctx, width, height);
    const pixelViewport = scaleGuiPreviewCanvasViewport(viewport, pixelRatio);
    const buffer = acquireGuiPreviewCanvasBitmapBuffer(ctx, guiPreviewRgb(13, 17, 23));
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

function acquireGuiPreviewCanvasBitmapBuffer(
    ctx: CanvasRenderingContext2D,
    background: ReturnType<typeof guiPreviewRgb>,
): GuiPreviewBitmapBuffer {
    const width = Math.max(1, ctx.canvas.width);
    const height = Math.max(1, ctx.canvas.height);
    const cached = guiPreviewCanvasBitmapBuffers.get(ctx.canvas);
    if (cached && cached.width === width && cached.height === height) {
        fillGuiPreviewBitmapRect(cached, 0, 0, width, height, background);
        return cached;
    }
    const buffer = createGuiPreviewBitmapBuffer(width, height, background);
    guiPreviewCanvasBitmapBuffers.set(ctx.canvas, buffer);
    return buffer;
}

function calculateGuiPreviewCanvasViewport(): GuiPreviewCanvasViewport {
    return {
        left: 0,
        top: 0,
        scale: 1,
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
