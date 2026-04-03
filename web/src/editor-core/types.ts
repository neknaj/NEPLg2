export type CoreHistoryEntry = {
    text: string;
    cursor: number;
    selectionStart: number;
    selectionEnd: number;
};

export type CoreEditorRuntimeState = {
    text: string;
    cursor: number;
    selectionStart: number;
    selectionEnd: number;
    preferredCursorColumn: number | null;
    isOverwriteMode: boolean;
    undoStack: CoreHistoryEntry[];
    redoStack: CoreHistoryEntry[];
};

export type CoreEditorSnapshot = {
    text: string;
    cursor: number;
    selectionStart: number;
    selectionEnd: number;
    hasSelection: boolean;
    selectedText: string;
    isOverwriteMode: boolean;
    undoDepth: number;
    redoDepth: number;
};

export type CoreEditorCommand =
    | { kind: 'select_all' }
    | { kind: 'toggle_overwrite' }
    | { kind: 'undo' }
    | { kind: 'redo' }
    | { kind: 'insert_text'; text: string }
    | { kind: 'delete_backward' }
    | { kind: 'delete_forward' }
    | { kind: 'move_cursor'; direction: 'left' | 'right'; extendSelection?: boolean }
    | { kind: 'move_cursor_vertical'; direction: 'up' | 'down'; extendSelection?: boolean }
    | { kind: 'move_cursor_line_boundary'; boundary: 'home' | 'end'; extendSelection?: boolean }
    | { kind: 'move_cursor_page'; direction: 'up' | 'down'; pageSize: number; extendSelection?: boolean }
    | { kind: 'set_cursor'; cursor: number }
    | { kind: 'set_selection'; selectionStart: number; selectionEnd: number }
    | { kind: 'replace_text'; text: string; cursor?: number; selectionStart?: number; selectionEnd?: number }
    | { kind: 'record_history' };

export type CoreKeyboardLikeEvent = {
    key: string;
    ctrlKey?: boolean;
    metaKey?: boolean;
    shiftKey?: boolean;
    altKey?: boolean;
};
