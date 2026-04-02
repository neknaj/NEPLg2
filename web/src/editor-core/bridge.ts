import { createEditorRuntimeState, snapshotEditorRuntimeState } from './state.js';
import { reduceEditorCommand, runEditorCommandSequence } from './reducer.js';
import { mapKeyboardEventToCoreCommand } from './keymap.js';

const bridge = {
    createEditorRuntimeState,
    snapshotEditorRuntimeState,
    reduceEditorCommand,
    runEditorCommandSequence,
    mapKeyboardEventToCoreCommand,
};

declare global {
    interface Window {
        NEPLPlaygroundEditorCore?: typeof bridge;
    }
}

if (typeof window !== 'undefined') {
    window.NEPLPlaygroundEditorCore = bridge;
}

export { bridge as NEPLPlaygroundEditorCore };
