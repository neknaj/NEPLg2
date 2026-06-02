import {
    GuiPreviewCommandFrame,
    GuiPreviewDrawCommand,
    GuiPreviewTextAlign,
    guiPreviewRgba,
} from './commands.js';

export const GUI_STDOUT_FRAME_BEGIN = 'NEPLG2_GUI_FRAME_BEGIN';
export const GUI_STDOUT_FILL_RECT = 'NEPLG2_GUI_FILL_RECT';
export const GUI_STDOUT_TEXT_RUN = 'NEPLG2_GUI_TEXT_RUN';
export const GUI_STDOUT_FRAME_END = 'NEPLG2_GUI_FRAME_END';

export type GuiWebStdoutProtocolErrorKind =
    | 'invalid-frame-begin'
    | 'invalid-frame-state'
    | 'invalid-fill-rect'
    | 'invalid-text-run'
    | 'invalid-color'
    | 'unsupported-protocol-line';

export type GuiWebStdoutProtocolError = {
    kind: GuiWebStdoutProtocolErrorKind;
    path: string;
    expected: string;
    actual: string;
};

export type GuiWebStdoutProtocolEvent =
    | { kind: 'text'; text: string }
    | { kind: 'frame'; frame: GuiPreviewCommandFrame & { windowId: number } }
    | { kind: 'error'; error: GuiWebStdoutProtocolError };

type GuiWebStdoutProtocolState =
    | { kind: 'idle' }
    | {
        kind: 'building-frame';
        windowId: number;
        title: string;
        width: number;
        height: number;
        commands: GuiPreviewDrawCommand[];
    };

type ProtocolResult<Value> =
    | { kind: 'ok'; value: Value }
    | { kind: 'err'; error: GuiWebStdoutProtocolError };

export class GuiWebStdoutProtocolParser {
    private pendingLine: string;
    private state: GuiWebStdoutProtocolState;

    constructor() {
        this.pendingLine = '';
        this.state = { kind: 'idle' };
    }

    reset() {
        this.pendingLine = '';
        this.state = { kind: 'idle' };
    }

    pushText(text: string): GuiWebStdoutProtocolEvent[] {
        const events: GuiWebStdoutProtocolEvent[] = [];
        this.pendingLine += normalizeChunkNewlines(text);
        let newlineIndex = this.pendingLine.indexOf('\n');
        while (newlineIndex >= 0) {
            const line = this.pendingLine.slice(0, newlineIndex);
            this.pendingLine = this.pendingLine.slice(newlineIndex + 1);
            events.push(...this.handleLine(line));
            newlineIndex = this.pendingLine.indexOf('\n');
        }
        return events;
    }

    flush(): GuiWebStdoutProtocolEvent[] {
        const events: GuiWebStdoutProtocolEvent[] = [];
        if (this.pendingLine.length > 0) {
            events.push(...this.handleLine(this.pendingLine));
            this.pendingLine = '';
        }
        if (this.state.kind === 'building-frame') {
            events.push({
                kind: 'error',
                error: err('invalid-frame-state', '$', GUI_STDOUT_FRAME_END, 'end of stdout'),
            });
            this.state = { kind: 'idle' };
        }
        return events;
    }

    private handleLine(rawLine: string): GuiWebStdoutProtocolEvent[] {
        const line = rawLine.trim();
        if (!isProtocolLine(line)) {
            if (this.state.kind === 'building-frame') {
                return this.abortFrameWithError(
                    err('unsupported-protocol-line', '$.line', 'GUI protocol command', rawLine),
                );
            }
            return [{ kind: 'text', text: `${rawLine}\n` }];
        }

        if (line.startsWith(GUI_STDOUT_FRAME_BEGIN)) {
            return this.handleFrameBegin(line);
        }
        if (line === GUI_STDOUT_FRAME_END) {
            return this.handleFrameEnd();
        }
        if (line.startsWith(GUI_STDOUT_FILL_RECT)) {
            return this.handleFillRect(line);
        }
        if (line.startsWith(GUI_STDOUT_TEXT_RUN)) {
            return this.handleTextRun(line);
        }
        return [{
            kind: 'error',
            error: err('unsupported-protocol-line', '$.line', 'known GUI protocol command', line),
        }];
    }

    private handleFrameBegin(line: string): GuiWebStdoutProtocolEvent[] {
        if (this.state.kind === 'building-frame') {
            return this.abortFrameWithError(
                err('invalid-frame-state', '$', GUI_STDOUT_FRAME_END, GUI_STDOUT_FRAME_BEGIN),
            );
        }

        const tokens = splitFields(line);
        const windowId = readInteger(tokens, 1, '$.windowId', 'invalid-frame-begin');
        if (windowId.kind === 'err') {
            return [{ kind: 'error', error: windowId.error }];
        }
        const width = readPositiveInteger(tokens, 2, '$.width', 'invalid-frame-begin');
        if (width.kind === 'err') {
            return [{ kind: 'error', error: width.error }];
        }
        const height = readPositiveInteger(tokens, 3, '$.height', 'invalid-frame-begin');
        if (height.kind === 'err') {
            return [{ kind: 'error', error: height.error }];
        }
        const title = readRest(tokens, 4, '$.title', 'invalid-frame-begin');
        if (title.kind === 'err') {
            return [{ kind: 'error', error: title.error }];
        }

        this.state = {
            kind: 'building-frame',
            windowId: windowId.value,
            title: title.value,
            width: width.value,
            height: height.value,
            commands: [],
        };
        return [];
    }

    private handleFrameEnd(): GuiWebStdoutProtocolEvent[] {
        if (this.state.kind === 'idle') {
            return [{
                kind: 'error',
                error: err('invalid-frame-state', '$', GUI_STDOUT_FRAME_BEGIN, GUI_STDOUT_FRAME_END),
            }];
        }
        const frame = this.state;
        this.state = { kind: 'idle' };
        return [{
            kind: 'frame',
            frame: {
                windowId: frame.windowId,
                title: frame.title,
                width: frame.width,
                height: frame.height,
                commands: frame.commands,
            },
        }];
    }

    private handleFillRect(line: string): GuiWebStdoutProtocolEvent[] {
        if (this.state.kind === 'idle') {
            return [{
                kind: 'error',
                error: err('invalid-frame-state', '$', GUI_STDOUT_FRAME_BEGIN, GUI_STDOUT_FILL_RECT),
            }];
        }
        const tokens = splitFields(line);
        const parsed = parseFillRect(tokens);
        if (parsed.kind === 'err') {
            return this.abortFrameWithError(parsed.error);
        }
        this.state.commands.push(parsed.value);
        return [];
    }

    private handleTextRun(line: string): GuiWebStdoutProtocolEvent[] {
        if (this.state.kind === 'idle') {
            return [{
                kind: 'error',
                error: err('invalid-frame-state', '$', GUI_STDOUT_FRAME_BEGIN, GUI_STDOUT_TEXT_RUN),
            }];
        }
        const tokens = splitFields(line);
        const parsed = parseTextRun(tokens);
        if (parsed.kind === 'err') {
            return this.abortFrameWithError(parsed.error);
        }
        this.state.commands.push(parsed.value);
        return [];
    }

    private abortFrameWithError(error: GuiWebStdoutProtocolError): GuiWebStdoutProtocolEvent[] {
        this.state = { kind: 'idle' };
        return [{ kind: 'error', error }];
    }
}

function parseFillRect(tokens: string[]): ProtocolResult<GuiPreviewDrawCommand> {
    const x = readInteger(tokens, 1, '$.rect.x', 'invalid-fill-rect');
    if (x.kind === 'err') return x;
    const y = readInteger(tokens, 2, '$.rect.y', 'invalid-fill-rect');
    if (y.kind === 'err') return y;
    const width = readNonNegativeInteger(tokens, 3, '$.rect.width', 'invalid-fill-rect');
    if (width.kind === 'err') return width;
    const height = readNonNegativeInteger(tokens, 4, '$.rect.height', 'invalid-fill-rect');
    if (height.kind === 'err') return height;
    const color = readColor(tokens, 5);
    if (color.kind === 'err') return color;
    return {
        kind: 'ok',
        value: {
            kind: 'fill-rect',
            rect: {
                x: x.value,
                y: y.value,
                width: width.value,
                height: height.value,
            },
            color: color.value,
        },
    };
}

function parseTextRun(tokens: string[]): ProtocolResult<GuiPreviewDrawCommand> {
    const x = readInteger(tokens, 1, '$.origin.x', 'invalid-text-run');
    if (x.kind === 'err') return x;
    const y = readInteger(tokens, 2, '$.origin.y', 'invalid-text-run');
    if (y.kind === 'err') return y;
    const size = readPositiveInteger(tokens, 3, '$.size', 'invalid-text-run');
    if (size.kind === 'err') return size;
    const align = readTextAlign(tokens, 4);
    if (align.kind === 'err') return align;
    const color = readColor(tokens, 5);
    if (color.kind === 'err') return color;
    const text = readRest(tokens, 9, '$.text', 'invalid-text-run');
    if (text.kind === 'err') return text;
    return {
        kind: 'ok',
        value: {
            kind: 'text-run',
            origin: {
                x: x.value,
                y: y.value,
            },
            text: text.value,
            color: color.value,
            size: size.value,
            align: align.value,
        },
    };
}

function readColor(tokens: string[], startIndex: number): ProtocolResult<ReturnType<typeof guiPreviewRgba>> {
    const red = readByte(tokens, startIndex, '$.color.red');
    if (red.kind === 'err') return red;
    const green = readByte(tokens, startIndex + 1, '$.color.green');
    if (green.kind === 'err') return green;
    const blue = readByte(tokens, startIndex + 2, '$.color.blue');
    if (blue.kind === 'err') return blue;
    const alpha = readByte(tokens, startIndex + 3, '$.color.alpha');
    if (alpha.kind === 'err') return alpha;
    return {
        kind: 'ok',
        value: guiPreviewRgba(red.value, green.value, blue.value, alpha.value),
    };
}

function readTextAlign(tokens: string[], index: number): ProtocolResult<GuiPreviewTextAlign> {
    const value = readToken(tokens, index, '$.align', 'invalid-text-run');
    if (value.kind === 'err') {
        return value;
    }
    if (value.value === 'left' || value.value === 'center' || value.value === 'right') {
        return { kind: 'ok', value: value.value };
    }
    return {
        kind: 'err',
        error: err('invalid-text-run', '$.align', 'left, center, or right', value.value),
    };
}

function readByte(tokens: string[], index: number, path: string): ProtocolResult<number> {
    const value = readInteger(tokens, index, path, 'invalid-color');
    if (value.kind === 'err') {
        return value;
    }
    if (value.value >= 0 && value.value <= 255) {
        return value;
    }
    return {
        kind: 'err',
        error: err('invalid-color', path, 'integer byte 0..255', String(value.value)),
    };
}

function readPositiveInteger(
    tokens: string[],
    index: number,
    path: string,
    kind: GuiWebStdoutProtocolErrorKind,
): ProtocolResult<number> {
    const value = readInteger(tokens, index, path, kind);
    if (value.kind === 'err') {
        return value;
    }
    if (value.value > 0) {
        return value;
    }
    return {
        kind: 'err',
        error: err(kind, path, 'integer greater than 0', String(value.value)),
    };
}

function readNonNegativeInteger(
    tokens: string[],
    index: number,
    path: string,
    kind: GuiWebStdoutProtocolErrorKind,
): ProtocolResult<number> {
    const value = readInteger(tokens, index, path, kind);
    if (value.kind === 'err') {
        return value;
    }
    if (value.value >= 0) {
        return value;
    }
    return {
        kind: 'err',
        error: err(kind, path, 'integer greater than or equal to 0', String(value.value)),
    };
}

function readInteger(
    tokens: string[],
    index: number,
    path: string,
    kind: GuiWebStdoutProtocolErrorKind,
): ProtocolResult<number> {
    const token = readToken(tokens, index, path, kind);
    if (token.kind === 'err') {
        return token;
    }
    const value = Number(token.value);
    if (Number.isInteger(value)) {
        return { kind: 'ok', value };
    }
    return {
        kind: 'err',
        error: err(kind, path, 'integer', token.value),
    };
}

function readToken(
    tokens: string[],
    index: number,
    path: string,
    kind: GuiWebStdoutProtocolErrorKind,
): ProtocolResult<string> {
    if (index < tokens.length) {
        return { kind: 'ok', value: tokens[index] };
    }
    return {
        kind: 'err',
        error: err(kind, path, 'field', 'missing'),
    };
}

function readRest(
    tokens: string[],
    index: number,
    path: string,
    kind: GuiWebStdoutProtocolErrorKind,
): ProtocolResult<string> {
    if (index < tokens.length) {
        return { kind: 'ok', value: tokens.slice(index).join(' ') };
    }
    return {
        kind: 'err',
        error: err(kind, path, 'text field', 'missing'),
    };
}

function splitFields(line: string): string[] {
    return line.trim().split(/\s+/);
}

function isProtocolLine(line: string): boolean {
    return line.startsWith('NEPLG2_GUI_');
}

function normalizeChunkNewlines(text: string): string {
    return text.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
}

function err(
    kind: GuiWebStdoutProtocolErrorKind,
    path: string,
    expected: string,
    actual: string,
): GuiWebStdoutProtocolError {
    return {
        kind,
        path,
        expected,
        actual,
    };
}
