import type {
    GuiPreviewColor,
    GuiPreviewDrawCommand,
    GuiPreviewTextAlign,
} from './commands.js';
import type { GuiPreviewScene } from './renderer.js';

export type GuiPreviewCanvasViewport = {
    left: number;
    top: number;
    scale: number;
};

export type GuiPreviewCanvasRenderOptions = {
    fontSize: number;
};

export type GuiPreviewCanvasRenderResult = {
    viewport: GuiPreviewCanvasViewport;
};

export function renderGuiPreviewSceneToCanvas(
    ctx: CanvasRenderingContext2D,
    scene: GuiPreviewScene,
    width: number,
    height: number,
    options: GuiPreviewCanvasRenderOptions,
): GuiPreviewCanvasRenderResult {
    const padding = 18;
    const availableWidth = Math.max(1, width - padding * 2);
    const availableHeight = Math.max(1, height - padding * 2);
    const scale = Math.min(availableWidth / scene.width, availableHeight / scene.height);
    const sceneWidth = scene.width * scale;
    const sceneHeight = scene.height * scale;
    const left = Math.floor((width - sceneWidth) / 2);
    const top = Math.floor((height - sceneHeight) / 2);
    const viewport = { left, top, scale };

    ctx.fillStyle = '#101820';
    ctx.fillRect(left - 1, top - 1, sceneWidth + 2, sceneHeight + 2);
    ctx.textBaseline = 'top';

    for (const command of scene.commands) {
        renderGuiPreviewCommand(ctx, command, viewport);
    }

    ctx.textAlign = 'left';
    ctx.fillStyle = '#9fb1c1';
    ctx.font = `${Math.max(11, options.fontSize - 1)}px "HackGenConsoleNF", "JetBrains Mono", Consolas, monospace`;
    ctx.fillText(scene.title, 12, 10);

    return { viewport };
}

function renderGuiPreviewCommand(
    ctx: CanvasRenderingContext2D,
    command: GuiPreviewDrawCommand,
    viewport: GuiPreviewCanvasViewport,
) {
    if (command.kind === 'fill-rect') {
        ctx.fillStyle = guiPreviewCanvasColor(command.color);
        ctx.fillRect(
            viewport.left + command.rect.x * viewport.scale,
            viewport.top + command.rect.y * viewport.scale,
            Math.max(1, command.rect.width * viewport.scale),
            Math.max(1, command.rect.height * viewport.scale),
        );
        return;
    }

    ctx.fillStyle = guiPreviewCanvasColor(command.color);
    ctx.font = `${Math.max(8, command.size * viewport.scale)}px "HackGenConsoleNF", "JetBrains Mono", Consolas, monospace`;
    ctx.textAlign = guiPreviewCanvasTextAlign(command.align);
    ctx.fillText(
        command.text,
        viewport.left + command.origin.x * viewport.scale,
        viewport.top + command.origin.y * viewport.scale,
    );
}

function guiPreviewCanvasColor(color: GuiPreviewColor): string {
    const alpha = Math.max(0, Math.min(1, color.alpha / 255));
    return `rgba(${color.red}, ${color.green}, ${color.blue}, ${alpha})`;
}

function guiPreviewCanvasTextAlign(align: GuiPreviewTextAlign): CanvasTextAlign {
    if (align === 'left') {
        return 'left';
    }
    if (align === 'center') {
        return 'center';
    }
    if (align === 'right') {
        return 'right';
    }
    const neverAlign: never = align;
    return neverAlign;
}
