export type GuiPreviewKind = 'mandelbrot' | 'life' | 'counter';

export type GuiPreviewPoint = {
    x: number;
    y: number;
};

export type GuiPreviewRect = {
    x: number;
    y: number;
    width: number;
    height: number;
};

export type GuiPreviewColor = {
    kind: 'rgba8888';
    red: number;
    green: number;
    blue: number;
    alpha: number;
};

export type GuiPreviewTextAlign = 'left' | 'center' | 'right';

export type GuiPreviewDrawCommand =
    | {
        kind: 'fill-rect';
        rect: GuiPreviewRect;
        color: GuiPreviewColor;
    }
    | {
        kind: 'text-run';
        origin: GuiPreviewPoint;
        text: string;
        color: GuiPreviewColor;
        size: number;
        align: GuiPreviewTextAlign;
    };

export type GuiPreviewCommandFrame = {
    title: string;
    width: number;
    height: number;
    commands: GuiPreviewDrawCommand[];
};

export function guiPreviewRgb(red: number, green: number, blue: number): GuiPreviewColor {
    return guiPreviewRgba(red, green, blue, 255);
}

export function guiPreviewRgba(red: number, green: number, blue: number, alpha: number): GuiPreviewColor {
    return {
        kind: 'rgba8888',
        red: guiPreviewByte(red),
        green: guiPreviewByte(green),
        blue: guiPreviewByte(blue),
        alpha: guiPreviewByte(alpha),
    };
}

function guiPreviewByte(value: number): number {
    return Math.max(0, Math.min(255, Math.trunc(value)));
}
