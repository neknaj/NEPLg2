import type { CoreEditorCommand, CoreEditorRuntimeState } from './types.js';
import { createEditorRuntimeState, createHistoryEntry, normalizeRuntimeState } from './state.js';

const MAX_HISTORY = 100;

function normalizeRange(selectionStart: number, selectionEnd: number): { start: number; end: number } {
    return {
        start: Math.min(selectionStart, selectionEnd),
        end: Math.max(selectionStart, selectionEnd),
    };
}

function getLines(text: string): string[] {
    return text.split('\n');
}

function getLineStarts(lines: string[]): number[] {
    const starts: number[] = [];
    let cursor = 0;
    for (const line of lines) {
        starts.push(cursor);
        cursor += line.length + 1;
    }
    return starts;
}

function getRowColFromIndex(text: string, index: number): { row: number; col: number; lines: string[]; lineStarts: number[] } {
    const lines = getLines(text);
    const lineStarts = getLineStarts(lines);
    for (let row = 0; row < lines.length; row++) {
        const lineStart = lineStarts[row];
        const lineEndExclusive = lineStart + lines[row].length + (row < lines.length - 1 ? 1 : 0);
        if (index < lineEndExclusive) {
            return {
                row,
                col: Math.min(lines[row].length, Math.max(0, index - lineStart)),
                lines,
                lineStarts,
            };
        }
    }
    const lastRow = Math.max(0, lines.length - 1);
    return {
        row: lastRow,
        col: lines[lastRow]?.length ?? 0,
        lines,
        lineStarts,
    };
}

function getIndexFromRowCol(lineStarts: number[], lines: string[], row: number, col: number): number {
    const safeRow = Math.max(0, Math.min(lines.length - 1, row));
    const safeCol = Math.max(0, Math.min(lines[safeRow].length, col));
    return lineStarts[safeRow] + safeCol;
}

function collapseSelectionForDirectionalMove(
    state: CoreEditorRuntimeState,
    direction: 'left' | 'right' | 'up' | 'down',
): CoreEditorRuntimeState | null {
    if (state.selectionStart === state.selectionEnd) {
        return null;
    }
    const range = normalizeRange(state.selectionStart, state.selectionEnd);
    const nextCursor = direction === 'left' || direction === 'up' ? range.start : range.end;
    return normalizeRuntimeState({
        ...state,
        cursor: nextCursor,
        selectionStart: nextCursor,
        selectionEnd: nextCursor,
        preferredCursorColumn: null,
    });
}

function replaceSelectedText(state: CoreEditorRuntimeState, insertedText: string): CoreEditorRuntimeState {
    const range = normalizeRange(state.selectionStart, state.selectionEnd);
    const nextText = state.text.slice(0, range.start) + insertedText + state.text.slice(range.end);
    const nextCursor = range.start + insertedText.length;
    return normalizeRuntimeState({
        ...state,
        text: nextText,
        cursor: nextCursor,
        selectionStart: nextCursor,
        selectionEnd: nextCursor,
        preferredCursorColumn: null,
    });
}

function pushUndoState(state: CoreEditorRuntimeState): CoreEditorRuntimeState {
    const nextUndoStack = [...state.undoStack, createHistoryEntry(state)];
    if (nextUndoStack.length > MAX_HISTORY) {
        nextUndoStack.shift();
    }
    return {
        ...state,
        undoStack: nextUndoStack,
        redoStack: [],
    };
}

export function reduceEditorCommand(
    runtimeState: CoreEditorRuntimeState,
    command: CoreEditorCommand,
): CoreEditorRuntimeState {
    const state = normalizeRuntimeState(runtimeState);

    switch (command.kind) {
        case 'record_history':
            return pushUndoState(state);
        case 'select_all':
            return {
                ...state,
                cursor: state.text.length,
                selectionStart: 0,
                selectionEnd: state.text.length,
                preferredCursorColumn: null,
            };
        case 'toggle_overwrite':
            return {
                ...state,
                isOverwriteMode: !state.isOverwriteMode,
            };
        case 'insert_text': {
            const nextState = pushUndoState(state);
            if (nextState.selectionStart !== nextState.selectionEnd) {
                return replaceSelectedText(nextState, command.text);
            }
            if (nextState.isOverwriteMode && nextState.cursor < nextState.text.length && command.text !== '\n') {
                const end = nextState.cursor + command.text.length;
                const nextText = nextState.text.slice(0, nextState.cursor) + command.text + nextState.text.slice(end);
                const nextCursor = nextState.cursor + command.text.length;
                return normalizeRuntimeState({
                    ...nextState,
                    text: nextText,
                    cursor: nextCursor,
                    selectionStart: nextCursor,
                    selectionEnd: nextCursor,
                    preferredCursorColumn: null,
                });
            }
            const nextText = nextState.text.slice(0, nextState.cursor) + command.text + nextState.text.slice(nextState.cursor);
            const nextCursor = nextState.cursor + command.text.length;
            return normalizeRuntimeState({
                ...nextState,
                text: nextText,
                cursor: nextCursor,
                selectionStart: nextCursor,
                selectionEnd: nextCursor,
                preferredCursorColumn: null,
            });
        }
        case 'delete_backward': {
            if (state.selectionStart !== state.selectionEnd) {
                const nextState = pushUndoState(state);
                return replaceSelectedText(nextState, '');
            }
            if (state.cursor <= 0) {
                return state;
            }
            const nextState = pushUndoState(state);
            const removeIndex = nextState.cursor - 1;
            const nextText = nextState.text.slice(0, removeIndex) + nextState.text.slice(nextState.cursor);
            return normalizeRuntimeState({
                ...nextState,
                text: nextText,
                cursor: removeIndex,
                selectionStart: removeIndex,
                selectionEnd: removeIndex,
                preferredCursorColumn: null,
            });
        }
        case 'delete_forward': {
            if (state.selectionStart !== state.selectionEnd) {
                const nextState = pushUndoState(state);
                return replaceSelectedText(nextState, '');
            }
            if (state.cursor >= state.text.length) {
                return state;
            }
            const nextState = pushUndoState(state);
            const nextText = nextState.text.slice(0, nextState.cursor) + nextState.text.slice(nextState.cursor + 1);
            return normalizeRuntimeState({
                ...nextState,
                text: nextText,
                selectionStart: nextState.cursor,
                selectionEnd: nextState.cursor,
                preferredCursorColumn: null,
            });
        }
        case 'move_cursor': {
            const collapsed = !command.extendSelection
                ? collapseSelectionForDirectionalMove(state, command.direction)
                : null;
            if (collapsed) {
                return collapsed;
            }
            let nextCursor = state.cursor;
            if (command.direction === 'left') {
                nextCursor = Math.max(0, state.cursor - 1);
            } else {
                nextCursor = Math.min(state.text.length, state.cursor + 1);
            }
            if (command.extendSelection) {
                return normalizeRuntimeState({
                    ...state,
                    cursor: nextCursor,
                    selectionEnd: nextCursor,
                    preferredCursorColumn: null,
                });
            }
            return normalizeRuntimeState({
                ...state,
                cursor: nextCursor,
                selectionStart: nextCursor,
                selectionEnd: nextCursor,
                preferredCursorColumn: null,
            });
        }
        case 'move_cursor_vertical': {
            const collapsed = !command.extendSelection
                ? collapseSelectionForDirectionalMove(state, command.direction)
                : null;
            if (collapsed) {
                return collapsed;
            }
            const { row, col, lines, lineStarts } = getRowColFromIndex(state.text, state.cursor);
            const preferredColumn = state.preferredCursorColumn ?? col;
            const nextRow = Math.max(0, Math.min(lines.length - 1, row + (command.direction === 'up' ? -1 : 1)));
            if (nextRow === row) {
                const edgeCursor = command.direction === 'up' ? 0 : state.text.length;
                if (command.extendSelection) {
                    return normalizeRuntimeState({
                        ...state,
                        cursor: edgeCursor,
                        selectionEnd: edgeCursor,
                        preferredCursorColumn: preferredColumn,
                    });
                }
                return normalizeRuntimeState({
                    ...state,
                    cursor: edgeCursor,
                    selectionStart: edgeCursor,
                    selectionEnd: edgeCursor,
                    preferredCursorColumn: preferredColumn,
                });
            }
            const nextCol = Math.min(lines[nextRow].length, preferredColumn);
            const nextCursor = getIndexFromRowCol(lineStarts, lines, nextRow, nextCol);
            if (command.extendSelection) {
                return normalizeRuntimeState({
                    ...state,
                    cursor: nextCursor,
                    selectionEnd: nextCursor,
                    preferredCursorColumn: preferredColumn,
                });
            }
            return normalizeRuntimeState({
                ...state,
                cursor: nextCursor,
                selectionStart: nextCursor,
                selectionEnd: nextCursor,
                preferredCursorColumn: preferredColumn,
            });
        }
        case 'move_cursor_line_boundary': {
            const { row, col, lines, lineStarts } = getRowColFromIndex(state.text, state.cursor);
            const line = lines[row] ?? '';
            const indentEndColumn = (line.match(/^\s*/) || [''])[0].length;
            const nextCol = command.boundary === 'home'
                ? ((col !== indentEndColumn && indentEndColumn !== line.length) ? indentEndColumn : 0)
                : line.length;
            const nextCursor = getIndexFromRowCol(lineStarts, lines, row, nextCol);
            if (command.extendSelection) {
                return normalizeRuntimeState({
                    ...state,
                    cursor: nextCursor,
                    selectionEnd: nextCursor,
                    preferredCursorColumn: null,
                });
            }
            return normalizeRuntimeState({
                ...state,
                cursor: nextCursor,
                selectionStart: nextCursor,
                selectionEnd: nextCursor,
                preferredCursorColumn: null,
            });
        }
        case 'move_cursor_page': {
            const { row, col, lines, lineStarts } = getRowColFromIndex(state.text, state.cursor);
            const preferredColumn = state.preferredCursorColumn ?? col;
            const pageSize = Math.max(1, Math.trunc(command.pageSize || 1));
            const direction = command.direction === 'up' ? -1 : 1;
            const nextRow = Math.max(0, Math.min(lines.length - 1, row + direction * pageSize));
            const nextCol = Math.min(lines[nextRow].length, preferredColumn);
            const nextCursor = getIndexFromRowCol(lineStarts, lines, nextRow, nextCol);
            if (command.extendSelection) {
                return normalizeRuntimeState({
                    ...state,
                    cursor: nextCursor,
                    selectionEnd: nextCursor,
                    preferredCursorColumn: preferredColumn,
                });
            }
            return normalizeRuntimeState({
                ...state,
                cursor: nextCursor,
                selectionStart: nextCursor,
                selectionEnd: nextCursor,
                preferredCursorColumn: preferredColumn,
            });
        }
        case 'set_cursor':
            return normalizeRuntimeState({
                ...state,
                cursor: command.cursor,
                selectionStart: command.cursor,
                selectionEnd: command.cursor,
                preferredCursorColumn: null,
            });
        case 'set_selection':
            return normalizeRuntimeState({
                ...state,
                selectionStart: command.selectionStart,
                selectionEnd: command.selectionEnd,
                cursor: command.selectionEnd,
                preferredCursorColumn: null,
            });
        case 'replace_text': {
            const nextState = pushUndoState(state);
            const cursor = command.cursor ?? nextState.cursor;
            const selectionStart = command.selectionStart ?? cursor;
            const selectionEnd = command.selectionEnd ?? cursor;
            return normalizeRuntimeState({
                ...nextState,
                text: command.text,
                cursor,
                selectionStart,
                selectionEnd,
                preferredCursorColumn: null,
            });
        }
        case 'undo': {
            if (state.undoStack.length === 0) {
                return state;
            }
            const previous = state.undoStack[state.undoStack.length - 1];
            const redoStack = [...state.redoStack, createHistoryEntry(state)];
            return normalizeRuntimeState({
                ...state,
                text: previous.text,
                cursor: previous.cursor,
                selectionStart: previous.selectionStart,
                selectionEnd: previous.selectionEnd,
                undoStack: state.undoStack.slice(0, -1),
                redoStack,
                preferredCursorColumn: null,
            });
        }
        case 'redo': {
            if (state.redoStack.length === 0) {
                return state;
            }
            const next = state.redoStack[state.redoStack.length - 1];
            const undoStack = [...state.undoStack, createHistoryEntry(state)];
            return normalizeRuntimeState({
                ...state,
                text: next.text,
                cursor: next.cursor,
                selectionStart: next.selectionStart,
                selectionEnd: next.selectionEnd,
                undoStack,
                redoStack: state.redoStack.slice(0, -1),
                preferredCursorColumn: null,
            });
        }
        default:
            return state;
    }
}

export function runEditorCommandSequence(commands: CoreEditorCommand[], initialText = ''): CoreEditorRuntimeState {
    let state = createEditorRuntimeState(initialText);
    for (const command of commands) {
        state = reduceEditorCommand(state, command);
    }
    return state;
}
