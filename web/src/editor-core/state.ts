import type { CoreEditorRuntimeState, CoreEditorSnapshot, CoreHistoryEntry } from './types.js';

function clampIndex(text: string, index: number): number {
    if (!Number.isFinite(index)) {
        return 0;
    }
    return Math.max(0, Math.min(text.length, Math.trunc(index)));
}

function normalizeSelection(text: string, start: number, end: number): { start: number; end: number } {
    const normalizedStart = clampIndex(text, start);
    const normalizedEnd = clampIndex(text, end);
    return {
        start: normalizedStart,
        end: normalizedEnd,
    };
}

export function createHistoryEntry(state: Pick<CoreEditorRuntimeState, 'text' | 'cursor' | 'selectionStart' | 'selectionEnd'>): CoreHistoryEntry {
    return {
        text: state.text,
        cursor: state.cursor,
        selectionStart: state.selectionStart,
        selectionEnd: state.selectionEnd,
    };
}

export function createEditorRuntimeState(text = ''): CoreEditorRuntimeState {
    return {
        text,
        cursor: 0,
        selectionStart: 0,
        selectionEnd: 0,
        isOverwriteMode: false,
        undoStack: [],
        redoStack: [],
    };
}

export function snapshotEditorRuntimeState(state: CoreEditorRuntimeState): CoreEditorSnapshot {
    const selectionStart = clampIndex(state.text, state.selectionStart);
    const selectionEnd = clampIndex(state.text, state.selectionEnd);
    const rangeStart = Math.min(selectionStart, selectionEnd);
    const rangeEnd = Math.max(selectionStart, selectionEnd);

    return {
        text: state.text,
        cursor: clampIndex(state.text, state.cursor),
        selectionStart,
        selectionEnd,
        hasSelection: rangeStart !== rangeEnd,
        selectedText: state.text.slice(rangeStart, rangeEnd),
        isOverwriteMode: Boolean(state.isOverwriteMode),
        undoDepth: state.undoStack.length,
        redoDepth: state.redoStack.length,
    };
}

export function normalizeRuntimeState(state: CoreEditorRuntimeState): CoreEditorRuntimeState {
    const cursor = clampIndex(state.text, state.cursor);
    const selection = normalizeSelection(state.text, state.selectionStart, state.selectionEnd);

    return {
        ...state,
        cursor,
        selectionStart: selection.start,
        selectionEnd: selection.end,
    };
}
