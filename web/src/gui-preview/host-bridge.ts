import {
    GuiPreviewColor,
    GuiPreviewCommandFrame,
    GuiPreviewDrawCommand,
    GuiPreviewInputTarget,
    GuiPreviewRect,
    GuiPreviewTextAlign,
    guiPreviewRgba,
} from './commands.js';

export type GuiWebHostPresentedFrame = {
    windowId: number;
    frame: GuiPreviewCommandFrame;
};

export type GuiWebHostDecodeErrorKind =
    | 'invalid-frame'
    | 'invalid-command'
    | 'invalid-rect'
    | 'invalid-color'
    | 'invalid-text'
    | 'invalid-input-target'
    | 'unsupported-command';

export type GuiWebHostDecodeError = {
    kind: GuiWebHostDecodeErrorKind;
    path: string;
    expected: string;
    actual: string;
};

export type GuiWebHostResult<Value> =
    | { kind: 'ok'; value: Value }
    | { kind: 'err'; error: GuiWebHostDecodeError };

type DecodeContext = {
    path: string;
};

type UnknownRecord = Record<string, unknown>;

export function decodeGuiWebHostPresentedFrame(input: unknown): GuiWebHostResult<GuiWebHostPresentedFrame> {
    const root = asRecord(input, { path: '$' }, 'invalid-frame', 'object frame');
    if (root.kind === 'err') {
        return root;
    }

    const windowId = readPositiveInteger(root.value, 'windowId', { path: '$.windowId' }, 'invalid-frame');
    if (windowId.kind === 'err') {
        return windowId;
    }
    const frame = decodeGuiWebHostFrameFromRecord(root.value, { path: '$' });
    if (frame.kind === 'err') {
        return frame;
    }

    return {
        kind: 'ok',
        value: {
            windowId: windowId.value,
            frame: frame.value,
        },
    };
}

export function decodeGuiWebHostFrame(input: unknown): GuiWebHostResult<GuiPreviewCommandFrame> {
    const frame = asRecord(input, { path: '$' }, 'invalid-frame', 'object frame');
    if (frame.kind === 'err') {
        return frame;
    }
    return decodeGuiWebHostFrameFromRecord(frame.value, { path: '$' });
}

function decodeGuiWebHostFrameFromRecord(frame: UnknownRecord, context: DecodeContext): GuiWebHostResult<GuiPreviewCommandFrame> {
    const title = readString(frame, 'title', child(context, 'title'), 'invalid-frame');
    if (title.kind === 'err') {
        return title;
    }
    const width = readPositiveNumber(frame, 'width', child(context, 'width'), 'invalid-frame');
    if (width.kind === 'err') {
        return width;
    }
    const height = readPositiveNumber(frame, 'height', child(context, 'height'), 'invalid-frame');
    if (height.kind === 'err') {
        return height;
    }
    const commandValues = readArray(frame, 'commands', child(context, 'commands'), 'invalid-frame');
    if (commandValues.kind === 'err') {
        return commandValues;
    }
    const inputTargetValues = readOptionalArray(frame, 'inputTargets', child(context, 'inputTargets'), 'invalid-frame');
    if (inputTargetValues.kind === 'err') {
        return inputTargetValues;
    }

    const commands: GuiPreviewDrawCommand[] = [];
    for (let index = 0; index < commandValues.value.length; index += 1) {
        const command = decodeGuiWebHostCommand(commandValues.value[index], child(context, `commands.${index}`));
        if (command.kind === 'err') {
            return command;
        }
        commands.push(command.value);
    }
    const inputTargets: GuiPreviewInputTarget[] = [];
    for (let index = 0; index < inputTargetValues.value.length; index += 1) {
        const inputTarget = decodeGuiWebHostInputTarget(inputTargetValues.value[index], child(context, `inputTargets.${index}`));
        if (inputTarget.kind === 'err') {
            return inputTarget;
        }
        inputTargets.push(inputTarget.value);
    }

    return {
        kind: 'ok',
        value: {
            title: title.value,
            width: width.value,
            height: height.value,
            commands,
            inputTargets,
        },
    };
}

function decodeGuiWebHostCommand(input: unknown, context: DecodeContext): GuiWebHostResult<GuiPreviewDrawCommand> {
    const command = asRecord(input, context, 'invalid-command', 'object command');
    if (command.kind === 'err') {
        return command;
    }
    const kind = readString(command.value, 'kind', child(context, 'kind'), 'invalid-command');
    if (kind.kind === 'err') {
        return kind;
    }
    if (kind.value === 'fill-rect') {
        return decodeGuiWebHostFillRect(command.value, context);
    }
    if (kind.value === 'text-run') {
        return decodeGuiWebHostTextRun(command.value, context);
    }
    if (kind.value === 'rgba-row') {
        return decodeGuiWebHostRgbaRow(command.value, context);
    }
    return err('unsupported-command', child(context, 'kind'), 'fill-rect, text-run, or rgba-row', kind.value);
}

function decodeGuiWebHostInputTarget(input: unknown, context: DecodeContext): GuiWebHostResult<GuiPreviewInputTarget> {
    const target = asRecord(input, context, 'invalid-input-target', 'object input target');
    if (target.kind === 'err') {
        return target;
    }
    const kind = readString(target.value, 'kind', child(context, 'kind'), 'invalid-input-target');
    if (kind.kind === 'err') {
        return kind;
    }
    if (kind.value !== 'action-rect') {
        return err('invalid-input-target', child(context, 'kind'), 'action-rect', kind.value);
    }
    const rect = decodeGuiWebHostRect(target.value, child(context, 'rect'));
    if (rect.kind === 'err') {
        return remapDecodeErrorKind(rect.error, 'invalid-input-target');
    }
    const actionId = readPositiveInteger(target.value, 'actionId', child(context, 'actionId'), 'invalid-input-target');
    if (actionId.kind === 'err') {
        return actionId;
    }
    return {
        kind: 'ok',
        value: {
            kind: 'action-rect',
            rect: rect.value,
            actionId: actionId.value,
        },
    };
}

function decodeGuiWebHostFillRect(command: UnknownRecord, context: DecodeContext): GuiWebHostResult<GuiPreviewDrawCommand> {
    const rect = decodeGuiWebHostRect(command, child(context, 'rect'));
    if (rect.kind === 'err') {
        return rect;
    }
    const color = decodeGuiWebHostColor(command, child(context, 'color'));
    if (color.kind === 'err') {
        return color;
    }
    return {
        kind: 'ok',
        value: {
            kind: 'fill-rect',
            rect: rect.value,
            color: color.value,
        },
    };
}

function decodeGuiWebHostTextRun(command: UnknownRecord, context: DecodeContext): GuiWebHostResult<GuiPreviewDrawCommand> {
    const origin = decodeGuiWebHostPoint(command, child(context, 'origin'));
    if (origin.kind === 'err') {
        return origin;
    }
    const text = readString(command, 'text', child(context, 'text'), 'invalid-text');
    if (text.kind === 'err') {
        return text;
    }
    const color = decodeGuiWebHostColor(command, child(context, 'color'));
    if (color.kind === 'err') {
        return color;
    }
    const size = readPositiveNumber(command, 'size', child(context, 'size'), 'invalid-text');
    if (size.kind === 'err') {
        return size;
    }
    const align = decodeGuiWebHostTextAlign(command, child(context, 'align'));
    if (align.kind === 'err') {
        return align;
    }
    return {
        kind: 'ok',
        value: {
            kind: 'text-run',
            origin: origin.value,
            text: text.value,
            color: color.value,
            size: size.value,
            align: align.value,
        },
    };
}

function decodeGuiWebHostRgbaRow(command: UnknownRecord, context: DecodeContext): GuiWebHostResult<GuiPreviewDrawCommand> {
    const origin = decodeGuiWebHostPointField(command, 'origin', child(context, 'origin'), 'invalid-command');
    if (origin.kind === 'err') {
        return origin;
    }
    const sampleWidth = readPositiveInteger(command, 'sampleWidth', child(context, 'sampleWidth'), 'invalid-command');
    if (sampleWidth.kind === 'err') {
        return sampleWidth;
    }
    const cellWidth = readPositiveInteger(command, 'cellWidth', child(context, 'cellWidth'), 'invalid-command');
    if (cellWidth.kind === 'err') {
        return cellWidth;
    }
    const cellHeight = readPositiveInteger(command, 'cellHeight', child(context, 'cellHeight'), 'invalid-command');
    if (cellHeight.kind === 'err') {
        return cellHeight;
    }
    const pixelValues = readArray(command, 'pixels', child(context, 'pixels'), 'invalid-command');
    if (pixelValues.kind === 'err') {
        return pixelValues;
    }
    if (pixelValues.value.length !== sampleWidth.value) {
        return err(
            'invalid-command',
            child(context, 'pixels'),
            `array length ${sampleWidth.value}`,
            `array length ${pixelValues.value.length}`,
        );
    }

    const pixels: GuiPreviewColor[] = [];
    for (let index = 0; index < pixelValues.value.length; index += 1) {
        const color = asRecord(pixelValues.value[index], child(context, `pixels.${index}`), 'invalid-color', 'object color');
        if (color.kind === 'err') {
            return color;
        }
        const decoded = decodeGuiWebHostColorRecord(color.value, child(context, `pixels.${index}`));
        if (decoded.kind === 'err') {
            return decoded;
        }
        pixels.push(decoded.value);
    }

    return {
        kind: 'ok',
        value: {
            kind: 'rgba-row',
            origin: origin.value,
            sampleWidth: sampleWidth.value,
            cellWidth: cellWidth.value,
            cellHeight: cellHeight.value,
            pixels,
        },
    };
}

function decodeGuiWebHostRect(record: UnknownRecord, context: DecodeContext): GuiWebHostResult<GuiPreviewRect> {
    const rect = readRecord(record, 'rect', context, 'invalid-rect');
    if (rect.kind === 'err') {
        return rect;
    }
    const x = readNumber(rect.value, 'x', child(context, 'x'), 'invalid-rect');
    if (x.kind === 'err') {
        return x;
    }
    const y = readNumber(rect.value, 'y', child(context, 'y'), 'invalid-rect');
    if (y.kind === 'err') {
        return y;
    }
    const width = readNonNegativeNumber(rect.value, 'width', child(context, 'width'), 'invalid-rect');
    if (width.kind === 'err') {
        return width;
    }
    const height = readNonNegativeNumber(rect.value, 'height', child(context, 'height'), 'invalid-rect');
    if (height.kind === 'err') {
        return height;
    }
    return {
        kind: 'ok',
        value: {
            x: x.value,
            y: y.value,
            width: width.value,
            height: height.value,
        },
    };
}

function decodeGuiWebHostPoint(record: UnknownRecord, context: DecodeContext): GuiWebHostResult<{ x: number; y: number }> {
    return decodeGuiWebHostPointField(record, 'origin', context, 'invalid-text');
}

function decodeGuiWebHostPointField(
    record: UnknownRecord,
    name: string,
    context: DecodeContext,
    kind: GuiWebHostDecodeErrorKind,
): GuiWebHostResult<{ x: number; y: number }> {
    const point = readRecord(record, name, context, kind);
    if (point.kind === 'err') {
        return point;
    }
    const x = readNumber(point.value, 'x', child(context, 'x'), kind);
    if (x.kind === 'err') {
        return x;
    }
    const y = readNumber(point.value, 'y', child(context, 'y'), kind);
    if (y.kind === 'err') {
        return y;
    }
    return {
        kind: 'ok',
        value: {
            x: x.value,
            y: y.value,
        },
    };
}

function decodeGuiWebHostColor(record: UnknownRecord, context: DecodeContext): GuiWebHostResult<GuiPreviewColor> {
    const color = readRecord(record, 'color', context, 'invalid-color');
    if (color.kind === 'err') {
        return color;
    }
    return decodeGuiWebHostColorRecord(color.value, context);
}

function decodeGuiWebHostColorRecord(color: UnknownRecord, context: DecodeContext): GuiWebHostResult<GuiPreviewColor> {
    const kind = readString(color, 'kind', child(context, 'kind'), 'invalid-color');
    if (kind.kind === 'err') {
        return kind;
    }
    if (kind.value !== 'rgba8888') {
        return err('unsupported-command', child(context, 'kind'), 'rgba8888', kind.value);
    }
    const red = readByte(color, 'red', child(context, 'red'));
    if (red.kind === 'err') {
        return red;
    }
    const green = readByte(color, 'green', child(context, 'green'));
    if (green.kind === 'err') {
        return green;
    }
    const blue = readByte(color, 'blue', child(context, 'blue'));
    if (blue.kind === 'err') {
        return blue;
    }
    const alpha = readByte(color, 'alpha', child(context, 'alpha'));
    if (alpha.kind === 'err') {
        return alpha;
    }
    return {
        kind: 'ok',
        value: guiPreviewRgba(red.value, green.value, blue.value, alpha.value),
    };
}

function decodeGuiWebHostTextAlign(record: UnknownRecord, context: DecodeContext): GuiWebHostResult<GuiPreviewTextAlign> {
    const align = readString(record, 'align', context, 'invalid-text');
    if (align.kind === 'err') {
        return align;
    }
    if (align.value === 'left' || align.value === 'center' || align.value === 'right') {
        return { kind: 'ok', value: align.value };
    }
    return err('invalid-text', context, 'left, center, or right', align.value);
}

function readRecord(record: UnknownRecord, name: string, context: DecodeContext, kind: GuiWebHostDecodeErrorKind): GuiWebHostResult<UnknownRecord> {
    return asRecord(record[name], context, kind, 'object');
}

function readArray(record: UnknownRecord, name: string, context: DecodeContext, kind: GuiWebHostDecodeErrorKind): GuiWebHostResult<unknown[]> {
    const value = record[name];
    if (Array.isArray(value)) {
        return { kind: 'ok', value };
    }
    return err(kind, context, 'array', actualType(value));
}

function readOptionalArray(record: UnknownRecord, name: string, context: DecodeContext, kind: GuiWebHostDecodeErrorKind): GuiWebHostResult<unknown[]> {
    if (!Object.prototype.hasOwnProperty.call(record, name)) {
        return { kind: 'ok', value: [] };
    }
    return readArray(record, name, context, kind);
}

function readString(record: UnknownRecord, name: string, context: DecodeContext, kind: GuiWebHostDecodeErrorKind): GuiWebHostResult<string> {
    const value = record[name];
    if (typeof value === 'string') {
        return { kind: 'ok', value };
    }
    return err(kind, context, 'string', actualType(value));
}

function readNumber(record: UnknownRecord, name: string, context: DecodeContext, kind: GuiWebHostDecodeErrorKind): GuiWebHostResult<number> {
    const value = record[name];
    if (typeof value === 'number' && Number.isFinite(value)) {
        return { kind: 'ok', value };
    }
    return err(kind, context, 'finite number', actualType(value));
}

function readPositiveNumber(record: UnknownRecord, name: string, context: DecodeContext, kind: GuiWebHostDecodeErrorKind): GuiWebHostResult<number> {
    const value = readNumber(record, name, context, kind);
    if (value.kind === 'err') {
        return value;
    }
    if (value.value > 0) {
        return value;
    }
    return err(kind, context, 'number greater than 0', String(value.value));
}

function readNonNegativeNumber(record: UnknownRecord, name: string, context: DecodeContext, kind: GuiWebHostDecodeErrorKind): GuiWebHostResult<number> {
    const value = readNumber(record, name, context, kind);
    if (value.kind === 'err') {
        return value;
    }
    if (value.value >= 0) {
        return value;
    }
    return err(kind, context, 'number greater than or equal to 0', String(value.value));
}

function readPositiveInteger(record: UnknownRecord, name: string, context: DecodeContext, kind: GuiWebHostDecodeErrorKind): GuiWebHostResult<number> {
    const value = readPositiveNumber(record, name, context, kind);
    if (value.kind === 'err') {
        return value;
    }
    if (Number.isInteger(value.value)) {
        return value;
    }
    return err(kind, context, 'positive integer', String(value.value));
}

function readByte(record: UnknownRecord, name: string, context: DecodeContext): GuiWebHostResult<number> {
    const value = readNumber(record, name, context, 'invalid-color');
    if (value.kind === 'err') {
        return value;
    }
    if (Number.isInteger(value.value) && value.value >= 0 && value.value <= 255) {
        return value;
    }
    return err('invalid-color', context, 'integer byte 0..255', String(value.value));
}

function asRecord(input: unknown, context: DecodeContext, kind: GuiWebHostDecodeErrorKind, expected: string): GuiWebHostResult<UnknownRecord> {
    if (typeof input === 'object' && input !== null && !Array.isArray(input)) {
        return { kind: 'ok', value: input as UnknownRecord };
    }
    return err(kind, context, expected, actualType(input));
}

function child(context: DecodeContext, field: string): DecodeContext {
    return {
        path: `${context.path}.${field}`,
    };
}

function err(kind: GuiWebHostDecodeErrorKind, context: DecodeContext, expected: string, actual: string): GuiWebHostResult<never> {
    return {
        kind: 'err',
        error: {
            kind,
            path: context.path,
            expected,
            actual,
        },
    };
}

function remapDecodeErrorKind(error: GuiWebHostDecodeError, kind: GuiWebHostDecodeErrorKind): GuiWebHostResult<never> {
    return err(kind, { path: error.path }, error.expected, error.actual);
}

function actualType(value: unknown): string {
    if (Array.isArray(value)) {
        return 'array';
    }
    if (value === null) {
        return 'null';
    }
    return typeof value;
}
