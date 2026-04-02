import type { CoreEditorCommand, CoreEditorRuntimeState } from './types.js';
import { createEditorRuntimeState, createHistoryEntry, normalizeRuntimeState } from './state.js';

const MAX_HISTORY = 100;

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
            };
        case 'toggle_overwrite':
            return {
                ...state,
                isOverwriteMode: !state.isOverwriteMode,
            };
        case 'set_cursor':
            return normalizeRuntimeState({
                ...state,
                cursor: command.cursor,
                selectionStart: command.cursor,
                selectionEnd: command.cursor,
            });
        case 'set_selection':
            return normalizeRuntimeState({
                ...state,
                selectionStart: command.selectionStart,
                selectionEnd: command.selectionEnd,
                cursor: command.selectionEnd,
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
