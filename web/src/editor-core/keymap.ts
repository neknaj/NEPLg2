import type { CoreEditorCommand, CoreKeyboardLikeEvent } from './types.js';

function hasPrimaryModifier(event: CoreKeyboardLikeEvent): boolean {
    return Boolean(event.ctrlKey || event.metaKey);
}

export function mapKeyboardEventToCoreCommand(event: CoreKeyboardLikeEvent): CoreEditorCommand | null {
    if (event.altKey) {
        return null;
    }

    if (hasPrimaryModifier(event)) {
        const key = String(event.key || '').toLowerCase();
        if (key === 'a') {
            return { kind: 'select_all' };
        }
        if (key === 'z') {
            return { kind: 'undo' };
        }
        if (key === 'y') {
            return { kind: 'redo' };
        }
        return null;
    }

    if (event.key === 'Insert') {
        return { kind: 'toggle_overwrite' };
    }

    return null;
}
