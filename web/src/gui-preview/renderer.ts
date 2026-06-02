import {
    GuiPreviewDrawCommand,
    GuiPreviewInputTarget,
    GuiPreviewKind,
    guiPreviewRgb,
} from './commands.js';

export type { GuiPreviewKind } from './commands.js';

export type GuiPreviewHitTarget = {
    x: number;
    y: number;
    width: number;
    height: number;
    action: 'increment-counter';
};

export type GuiPreviewMetrics =
    | {
        kind: 'mandelbrot';
        commandCount: number;
        insideCount: number;
    }
    | {
        kind: 'life';
        commandCount: number;
        liveCells: number;
        checksum: number;
    }
    | {
        kind: 'counter';
        commandCount: number;
        counterValue: number;
        actionId: number;
        redrawTarget: number;
    };

export type GuiPreviewScene = {
    kind: GuiPreviewKind;
    title: string;
    width: number;
    height: number;
    commands: GuiPreviewDrawCommand[];
    inputTargets: GuiPreviewInputTarget[];
    hitTargets: GuiPreviewHitTarget[];
    metrics: GuiPreviewMetrics;
};

export type GuiPreviewSceneOptions =
    | { kind: 'default' }
    | { kind: 'counter'; counterValue: number };

const defaultSceneOptions: GuiPreviewSceneOptions = { kind: 'default' };

export function guiPreviewKindFromPath(path: string): GuiPreviewKind {
    const normalized = path.toLowerCase();
    if (normalized.includes('gui_life')) {
        return 'life';
    }
    if (normalized.includes('gui_counter')) {
        return 'counter';
    }
    return 'mandelbrot';
}

export function createGuiPreviewScene(kind: GuiPreviewKind, options: GuiPreviewSceneOptions = defaultSceneOptions): GuiPreviewScene {
    if (kind === 'life') {
        return createLifeScene();
    }
    if (kind === 'counter') {
        return createCounterScene(counterValueFromOptions(options));
    }
    return createMandelbrotScene();
}

export function summarizeGuiPreviewScene(scene: GuiPreviewScene): string {
    if (scene.metrics.kind === 'mandelbrot') {
        return `commands ${scene.metrics.commandCount} / inside ${scene.metrics.insideCount}`;
    }
    if (scene.metrics.kind === 'life') {
        return `commands ${scene.metrics.commandCount} / live ${scene.metrics.liveCells} / checksum ${scene.metrics.checksum}`;
    }
    return `action ${scene.metrics.actionId} / value ${scene.metrics.counterValue} / redraw ${scene.metrics.redrawTarget}`;
}

function counterValueFromOptions(options: GuiPreviewSceneOptions): number {
    if (options.kind === 'counter') {
        return options.counterValue;
    }
    return 0;
}

function createMandelbrotScene(): GuiPreviewScene {
    const width = 8;
    const height = 8;
    const cellSize = 18;
    const commands: GuiPreviewDrawCommand[] = [];
    let insideCount = 0;

    for (let y = 0; y < height; y += 1) {
        for (let x = 0; x < width; x += 1) {
            const iter = mandelbrotCellIter(x, y);
            if (iter === mandelbrotLimit()) {
                insideCount += 1;
            }
            commands.push({
                kind: 'fill-rect',
                rect: {
                    x: x * cellSize,
                    y: y * cellSize,
                    width: cellSize,
                    height: cellSize,
                },
                color: mandelbrotColor(iter),
            });
        }
    }

    return {
        kind: 'mandelbrot',
        title: 'GUI Mandelbrot',
        width: width * cellSize,
        height: height * cellSize,
        commands,
        inputTargets: [],
        hitTargets: [],
        metrics: {
            kind: 'mandelbrot',
            commandCount: commands.length,
            insideCount,
        },
    };
}

function mandelbrotLimit(): number {
    return 24;
}

function mandelbrotCx(x: number): number {
    return x * 50 - 200;
}

function mandelbrotCy(y: number): number {
    return y * 50 - 175;
}

function mandelbrotCellIter(x: number, y: number): number {
    const cx = mandelbrotCx(x);
    const cy = mandelbrotCy(y);
    let zx = 0;
    let zy = 0;
    let iter = 0;
    while (iter < mandelbrotLimit()) {
        const zx2 = zx * zx;
        const zy2 = zy * zy;
        if (zx2 + zy2 >= 40000) {
            break;
        }
        const nextZx = divS(zx2, 100) - divS(zy2, 100) + cx;
        const nextZy = divS(zx * zy * 2, 100) + cy;
        zx = nextZx;
        zy = nextZy;
        iter += 1;
    }
    return iter;
}

function mandelbrotColor(iter: number) {
    if (iter === mandelbrotLimit()) {
        return guiPreviewRgb(0, 0, 0);
    }
    const shade = Math.max(0, Math.min(255, iter * 10));
    return guiPreviewRgb(shade, shade, 255);
}

function createLifeScene(): GuiPreviewScene {
    const width = 5;
    const height = 5;
    const step = 3;
    const cellSize = 28;
    const commands: GuiPreviewDrawCommand[] = [];
    let liveCells = 0;
    let checksum = 0;

    for (let y = 0; y < height; y += 1) {
        for (let x = 0; x < width; x += 1) {
            const alive = lifeCellAtStep(x, y, step);
            if (alive) {
                liveCells += 1;
                checksum += (x + 1) * (y + 1);
            }
            commands.push({
                kind: 'fill-rect',
                rect: {
                    x: x * cellSize,
                    y: y * cellSize,
                    width: cellSize - 2,
                    height: cellSize - 2,
                },
                color: alive ? guiPreviewRgb(0, 180, 180) : guiPreviewRgb(24, 24, 24),
            });
        }
    }

    return {
        kind: 'life',
        title: 'GUI Life Step 3',
        width: width * cellSize - 2,
        height: height * cellSize - 2,
        commands,
        inputTargets: [],
        hitTargets: [],
        metrics: {
            kind: 'life',
            commandCount: commands.length,
            liveCells,
            checksum,
        },
    };
}

function lifeInitialCell(x: number, y: number): boolean {
    return (x === 1 && y === 0)
        || (x === 2 && y === 1)
        || (x === 0 && y === 2)
        || (x === 1 && y === 2)
        || (x === 2 && y === 2);
}

function lifeCellAtStep(x: number, y: number, step: number): boolean {
    if (x < 0 || x >= 5 || y < 0 || y >= 5) {
        return false;
    }
    let grid: boolean[][] = Array.from({ length: 5 }, (_, row) => (
        Array.from({ length: 5 }, (_, col) => lifeInitialCell(col, row))
    ));
    for (let currentStep = 0; currentStep < step; currentStep += 1) {
        const next = Array.from({ length: 5 }, () => Array.from({ length: 5 }, () => false));
        for (let row = 0; row < 5; row += 1) {
            for (let col = 0; col < 5; col += 1) {
                const alive = grid[row][col];
                const neighbors = lifeNeighborCount(grid, col, row);
                next[row][col] = alive ? neighbors === 2 || neighbors === 3 : neighbors === 3;
            }
        }
        grid = next;
    }
    return grid[y][x];
}

function lifeNeighborCount(grid: boolean[][], x: number, y: number): number {
    let count = 0;
    for (let dy = -1; dy <= 1; dy += 1) {
        for (let dx = -1; dx <= 1; dx += 1) {
            if (dx === 0 && dy === 0) {
                continue;
            }
            const nx = x + dx;
            const ny = y + dy;
            if (nx >= 0 && nx < 5 && ny >= 0 && ny < 5 && grid[ny][nx]) {
                count += 1;
            }
        }
    }
    return count;
}

function createCounterScene(counterValue: number): GuiPreviewScene {
    const value = Math.max(0, Math.trunc(counterValue));
    const commands: GuiPreviewDrawCommand[] = [
        {
            kind: 'fill-rect',
            rect: { x: 0, y: 0, width: 220, height: 142 },
            color: guiPreviewRgb(16, 24, 32),
        },
        {
            kind: 'fill-rect',
            rect: { x: 18, y: 20, width: 184, height: 50 },
            color: guiPreviewRgb(29, 43, 53),
        },
        {
            kind: 'fill-rect',
            rect: { x: 18, y: 88, width: 184, height: 34 },
            color: guiPreviewRgb(45, 125, 111),
        },
        {
            kind: 'text-run',
            origin: { x: 110, y: 36 },
            text: String(value),
            color: guiPreviewRgb(242, 247, 245),
            size: 28,
            align: 'center',
        },
        {
            kind: 'text-run',
            origin: { x: 110, y: 97 },
            text: 'Increment',
            color: guiPreviewRgb(242, 247, 245),
            size: 14,
            align: 'center',
        },
    ];

    return {
        kind: 'counter',
        title: 'GUI Counter',
        width: 220,
        height: 142,
        commands,
        inputTargets: [
            { kind: 'action-rect', rect: { x: 18, y: 88, width: 184, height: 34 }, actionId: 1 },
        ],
        hitTargets: [
            { x: 18, y: 88, width: 184, height: 34, action: 'increment-counter' },
        ],
        metrics: {
            kind: 'counter',
            commandCount: commands.length,
            counterValue: value,
            actionId: 1,
            redrawTarget: 0,
        },
    };
}

function divS(value: number, divisor: number): number {
    return Math.trunc(value / divisor);
}
